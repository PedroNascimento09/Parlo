use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::store::{JobRecord, Store};

static NEXT: AtomicU64 = AtomicU64::new(0);
static MANAGER: OnceLock<Manager> = OnceLock::new();
const STALL_SECONDS: i64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
    Stalled,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Stalled => "stalled",
        }
    }

    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Done | Self::Failed | Self::Cancelled | Self::Stalled
        )
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub state: State,
    pub stage: String,
    pub progress: f32,
    pub title: String,
    pub error: Option<String>,
    pub input_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Job {
    pub fn value(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "kind": self.kind,
            "state": self.state.as_str(),
            "stage": self.stage,
            "progress": self.progress,
            "title": self.title,
            "error": self.error,
            "input_hash": self.input_hash,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        })
    }
}

#[derive(Clone)]
struct Event {
    job: Job,
}

struct Inner {
    jobs: HashMap<String, Job>,
    watchers: HashMap<String, Vec<Sender<Event>>>,
    stops: HashMap<String, Arc<AtomicBool>>,
}

struct Shared {
    inner: Mutex<Inner>,
    store: Mutex<Option<Store>>,
}

type Runner = Arc<dyn Fn(Handle) -> Result<(), String> + Send + Sync>;

pub struct Manager {
    shared: Arc<Shared>,
    runners: HashMap<String, Runner>,
}

impl Manager {
    fn empty(store: Option<Store>) -> Self {
        Self {
            shared: Arc::new(Shared {
                inner: Mutex::new(Inner {
                    jobs: HashMap::new(),
                    watchers: HashMap::new(),
                    stops: HashMap::new(),
                }),
                store: Mutex::new(store),
            }),
            runners: HashMap::new(),
        }
    }

    #[cfg(not(test))]
    pub fn new() -> Self {
        let mut manager = Self::empty(Store::open_default().ok());
        manager.register("models", |handle| {
            if handle.completed("download") {
                return Ok(());
            }
            crate::models::download_remaining(
                |stage, progress| handle.progress(stage, progress),
                || handle.cancelled(),
            )
        });
        manager.recover();
        manager
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        let manager = Self::empty(Store::open_memory().ok());
        manager.recover();
        manager
    }

    #[cfg(test)]
    fn with_store(store: Store) -> Self {
        Self::empty(Some(store))
    }

    fn register(
        &mut self,
        kind: &str,
        runner: impl Fn(Handle) -> Result<(), String> + Send + Sync + 'static,
    ) {
        self.runners.insert(kind.to_string(), Arc::new(runner));
    }

    pub fn submit<F>(&self, kind: &str, title: &str, input_hash: &str, work: F) -> String
    where
        F: FnOnce(Handle) -> Result<(), String> + Send + 'static,
    {
        self.sweep();
        let id = new_id();
        let created = now();
        let job = Job {
            id: id.clone(),
            kind: kind.to_string(),
            state: State::Queued,
            stage: "starting".to_string(),
            progress: 0.0,
            title: title.to_string(),
            error: None,
            input_hash: input_hash.to_string(),
            created_at: created,
            updated_at: created,
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        {
            let mut inner = self.lock();
            inner.jobs.insert(id.clone(), job.clone());
            inner.stops.insert(id.clone(), Arc::clone(&cancelled));
            publish(&mut inner, &job);
        }
        self.shared.persist(&job);

        let handle = Handle {
            shared: Arc::clone(&self.shared),
            id: id.clone(),
            cancelled,
            completed: Arc::new(Mutex::new(HashSet::new())),
        };
        spawn(handle, work);
        id
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.sweep();
        self.lock().jobs.get(id).cloned()
    }

    pub fn list(&self) -> Vec<Job> {
        self.sweep();
        let mut jobs: Vec<Job> = self.lock().jobs.values().cloned().collect();
        jobs.sort_by_key(|job| job.created_at);
        jobs
    }

    pub fn cancel(&self, id: &str) -> Result<(), String> {
        self.sweep();
        let mut inner = self.lock();
        let cancelled = {
            let Some(job) = inner.jobs.get_mut(id) else {
                return Err(format!("no job {id}"));
            };
            if job.state.terminal() {
                return Err(format!("job {id} has already finished"));
            }
            job.state = State::Cancelled;
            job.error = Some("cancelled".to_string());
            job.updated_at = now();
            job.clone()
        };
        if let Some(stop) = inner.stops.remove(id) {
            stop.store(true, Ordering::Relaxed);
        }
        publish(&mut inner, &cancelled);
        drop(inner);
        self.shared.persist(&cancelled);
        Ok(())
    }

    fn subscribe(&self, id: &str) -> Option<mpsc::Receiver<Event>> {
        self.sweep();
        let mut inner = self.lock();
        let job = inner.jobs.get(id)?.clone();
        let (sender, receiver) = mpsc::channel();
        if job.state.terminal() {
            let _ = sender.send(Event { job });
            return Some(receiver);
        }
        inner
            .watchers
            .entry(id.to_string())
            .or_default()
            .push(sender);
        Some(receiver)
    }

    pub fn stream(&self, stream: &mut TcpStream, id: &str) -> Result<(), String> {
        let events = self.subscribe(id).ok_or_else(|| format!("no job {id}"))?;
        crate::serve::sse::stream(stream, events, |event| {
            (event.job.value().to_string(), event.job.state.terminal())
        })
    }

    pub fn recover(&self) -> usize {
        let stored = {
            let store = self
                .shared
                .store
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match store.as_ref() {
                Some(store) => store.unfinished_jobs().unwrap_or_default(),
                None => Vec::new(),
            }
        };
        let mut recovered = 0;
        for stored in stored {
            let runner = self.runners.get(&stored.kind).cloned();
            let completed = if runner.is_some() {
                let store = self
                    .shared
                    .store
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                match store.as_ref() {
                    Some(store) => store.completed_stages(&stored.id).unwrap_or_default(),
                    None => Vec::new(),
                }
            } else {
                Vec::new()
            };
            let resumable = runner.is_some();
            let (state, stage, progress, error): (State, String, f32, Option<String>) = if resumable
            {
                if completed.is_empty() {
                    (State::Queued, String::new(), 0.0, None)
                } else {
                    (State::Queued, stored.stage, stored.progress, None)
                }
            } else {
                (
                    State::Failed,
                    stored.stage,
                    stored.progress,
                    Some("this job cannot be resumed after a restart".to_string()),
                )
            };
            let job = Job {
                id: stored.id,
                kind: stored.kind,
                state,
                stage,
                progress,
                title: stored.title,
                error,
                input_hash: stored.input_hash,
                created_at: stored.created_at,
                updated_at: now(),
            };
            let mut stop = None;
            let mut handle = None;
            if resumable {
                let cancelled = Arc::new(AtomicBool::new(false));
                stop = Some(Arc::clone(&cancelled));
                handle = Some(Handle {
                    shared: Arc::clone(&self.shared),
                    id: job.id.clone(),
                    cancelled,
                    completed: Arc::new(Mutex::new(completed.into_iter().collect())),
                });
            }
            {
                let mut inner = self.lock();
                inner.jobs.insert(job.id.clone(), job.clone());
                if let Some(stop) = stop {
                    inner.stops.insert(job.id.clone(), stop);
                }
                publish(&mut inner, &job);
            }
            self.shared.persist(&job);
            if let (Some(runner), Some(handle)) = (runner, handle) {
                spawn(handle, move |handle| runner(handle));
            }
            recovered += 1;
        }
        recovered
    }

    pub fn sweep(&self) -> usize {
        self.sweep_at(now())
    }

    fn sweep_at(&self, at: i64) -> usize {
        let mut inner = self.lock();
        let mut stalled = Vec::new();
        for job in inner.jobs.values_mut() {
            if !job.state.terminal() && at.saturating_sub(job.updated_at) >= STALL_SECONDS {
                job.state = State::Stalled;
                job.error = Some("no progress for ten minutes".to_string());
                job.updated_at = at;
                stalled.push(job.clone());
            }
        }
        for job in &stalled {
            if let Some(stop) = inner.stops.remove(&job.id) {
                stop.store(true, Ordering::Relaxed);
            }
        }
        for job in &stalled {
            publish(&mut inner, job);
        }
        drop(inner);
        for job in &stalled {
            self.shared.persist(job);
        }
        stalled.len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        lock_shared(&self.shared)
    }
}

pub fn manager() -> &'static Manager {
    MANAGER.get_or_init(|| {
        #[cfg(test)]
        {
            Manager::in_memory()
        }
        #[cfg(not(test))]
        {
            Manager::new()
        }
    })
}

#[derive(Clone)]
pub struct Handle {
    shared: Arc<Shared>,
    id: String,
    cancelled: Arc<AtomicBool>,
    completed: Arc<Mutex<HashSet<String>>>,
}

impl Handle {
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn completed(&self, stage: &str) -> bool {
        self.completed
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .contains(stage)
    }

    pub fn progress(&self, stage: &str, progress: f32) {
        let (updated, previous, started) = {
            let mut inner = lock_shared(&self.shared);
            let Some(job) = inner.jobs.get_mut(&self.id) else {
                return;
            };
            if job.state.terminal() {
                return;
            }
            let previous = if !job.stage.is_empty() && job.stage != stage {
                Some(job.stage.clone())
            } else {
                None
            };
            let started = job.stage != stage;
            job.stage = stage.to_string();
            job.progress = progress.clamp(0.0, 1.0);
            job.updated_at = now();
            let updated = job.clone();
            publish(&mut inner, &updated);
            (updated, previous, started)
        };
        if let Some(previous) = previous {
            self.completed
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(previous.clone());
            self.shared.save_stage(&self.id, &previous, "done");
        }
        if started {
            self.shared.save_stage(&self.id, stage, "running");
        }
        self.shared.persist(&updated);
    }
}

fn spawn<F>(handle: Handle, work: F)
where
    F: FnOnce(Handle) -> Result<(), String> + Send + 'static,
{
    std::thread::spawn(move || {
        {
            let mut inner = lock_shared(&handle.shared);
            let started = inner.jobs.get_mut(&handle.id).and_then(|job| {
                if job.state == State::Queued {
                    job.state = State::Running;
                    job.updated_at = now();
                    Some(job.clone())
                } else {
                    None
                }
            });
            let Some(started) = started else {
                inner.stops.remove(&handle.id);
                return;
            };
            publish(&mut inner, &started);
        }
        let result = work(handle.clone());
        let finished = {
            let mut inner = lock_shared(&handle.shared);
            let finished = inner.jobs.get_mut(&handle.id).map(|job| {
                if !job.state.terminal() {
                    match result {
                        Ok(()) => {
                            job.state = State::Done;
                            job.progress = 1.0;
                        }
                        Err(error) => {
                            job.state = State::Failed;
                            job.error = Some(error);
                        }
                    }
                    job.updated_at = now();
                }
                job.clone()
            });
            let Some(finished) = finished else {
                inner.stops.remove(&handle.id);
                return;
            };
            publish(&mut inner, &finished);
            inner.stops.remove(&handle.id);
            finished
        };
        handle.shared.persist(&finished);
        if finished.state == State::Done && !finished.stage.is_empty() {
            handle
                .shared
                .save_stage(&handle.id, &finished.stage, "done");
        }
    });
}

fn lock_shared(shared: &Arc<Shared>) -> std::sync::MutexGuard<'_, Inner> {
    shared
        .inner
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn publish(inner: &mut Inner, job: &Job) {
    let terminal = job.state.terminal();
    let Some(senders) = inner.watchers.get_mut(&job.id) else {
        return;
    };
    senders.retain(|sender| sender.send(Event { job: job.clone() }).is_ok());
    if terminal {
        senders.clear();
    }
}

impl Shared {
    fn with_store(&self, call: impl FnOnce(&Store)) {
        let mut store = self.store.lock().unwrap_or_else(|error| error.into_inner());
        if store.is_none() {
            *store = Store::open_default().ok();
        }
        if let Some(store) = store.as_ref() {
            call(store);
        }
    }

    fn persist(&self, job: &Job) {
        self.with_store(|store| {
            let _ = store.save_job(&JobRecord {
                id: &job.id,
                kind: &job.kind,
                state: job.state.as_str(),
                stage: &job.stage,
                progress: job.progress,
                input_hash: &job.input_hash,
                title: &job.title,
                error: job.error.as_deref(),
                created_at: job.created_at,
                updated_at: job.updated_at,
            });
        });
    }

    fn save_stage(&self, id: &str, name: &str, state: &str) {
        self.with_store(|store| {
            let _ = store.save_stage(id, name, state);
        });
    }
}

fn new_id() -> String {
    let counter = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("j{}-{}-{counter}", now_millis(), std::process::id())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    fn wait_for(manager: &Manager, id: &str) -> Job {
        for _ in 0..400 {
            if let Some(job) = manager.get(id) {
                if job.state.terminal() {
                    return job;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("job {id} did not finish");
    }

    fn wait_until(manager: &Manager, id: &str, state: State) {
        for _ in 0..400 {
            if manager.get(id).map(|job| job.state) == Some(state) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("job {id} did not reach {state:?}");
    }

    fn stored_job(store: &Store, kind: &str, stage: &str) {
        store
            .save_job(&JobRecord {
                id: "j1",
                kind,
                state: "running",
                stage,
                progress: 0.4,
                input_hash: "",
                title: "A file",
                error: None,
                created_at: 1,
                updated_at: 2,
            })
            .unwrap();
    }

    fn stage_runner(
        ran: Arc<Mutex<Vec<String>>>,
    ) -> impl Fn(Handle) -> Result<(), String> + Send + Sync + 'static {
        move |handle| {
            for stage in ["transcribe", "translate", "export"] {
                if handle.completed(stage) {
                    continue;
                }
                ran.lock().unwrap().push(stage.to_string());
                handle.progress(stage, 0.5);
            }
            Ok(())
        }
    }

    #[test]
    fn a_job_advances_through_its_stages() {
        let manager = Manager::in_memory();
        let id = manager.submit("dub", "A file", "hash", |handle| {
            handle.progress("transcribe", 0.5);
            std::thread::sleep(Duration::from_millis(20));
            handle.progress("export", 0.9);
            Ok(())
        });
        let job = wait_for(&manager, &id);
        assert_eq!(job.state, State::Done);
        assert_eq!(job.stage, "export");
        assert_eq!(job.progress, 1.0);
    }

    #[test]
    fn progress_is_observable_before_the_job_finishes() {
        let manager = Manager::in_memory();
        let id = manager.submit("dub", "A file", "hash", |handle| {
            handle.progress("transcribe", 0.4);
            std::thread::sleep(Duration::from_millis(40));
            Ok(())
        });
        let events = manager.subscribe(&id).unwrap();
        let mut saw_progress = false;
        while let Ok(event) = events.recv_timeout(Duration::from_secs(2)) {
            if event.job.progress >= 0.4 {
                saw_progress = true;
            }
            if event.job.state.terminal() {
                break;
            }
        }
        assert!(saw_progress);
        wait_for(&manager, &id);
    }

    #[test]
    fn cancelling_reaches_a_terminal_state_and_stops_the_worker() {
        let manager = Manager::in_memory();
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stopped);
        let id = manager.submit("dub", "A file", "hash", move |handle| {
            while !handle.cancelled() {
                std::thread::sleep(Duration::from_millis(5));
            }
            flag.store(true, Ordering::Relaxed);
            Ok(())
        });
        wait_until(&manager, &id, State::Running);
        manager.cancel(&id).unwrap();
        let job = wait_for(&manager, &id);
        assert_eq!(job.state, State::Cancelled);

        let events = manager.subscribe(&id).unwrap();
        let event = events.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(event.job.state, State::Cancelled);
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(1)),
            Err(RecvTimeoutError::Disconnected)
        ));

        for _ in 0..400 {
            if stopped.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(stopped.load(Ordering::Relaxed));
    }

    #[test]
    fn a_recovered_job_resumes_after_its_last_completed_stage() {
        let store = Store::open_memory().unwrap();
        stored_job(&store, "fixture", "translate");
        store.save_stage("j1", "transcribe", "done").unwrap();
        store.save_stage("j1", "translate", "running").unwrap();

        let mut manager = Manager::with_store(store);
        let ran = Arc::new(Mutex::new(Vec::new()));
        manager.register("fixture", stage_runner(Arc::clone(&ran)));
        assert_eq!(manager.recover(), 1);

        let job = wait_for(&manager, "j1");
        assert_eq!(job.id, "j1");
        assert_eq!(job.state, State::Done);
        assert_eq!(job.stage, "export");
        assert_eq!(job.progress, 1.0);
        assert_eq!(job.error, None);
        assert_eq!(
            *ran.lock().unwrap(),
            vec!["translate".to_string(), "export".to_string()]
        );
    }

    #[test]
    fn a_recovered_job_without_a_completed_stage_starts_from_the_first() {
        let store = Store::open_memory().unwrap();
        stored_job(&store, "fixture", "translate");
        store.save_stage("j1", "translate", "running").unwrap();

        let mut manager = Manager::with_store(store);
        let ran = Arc::new(Mutex::new(Vec::new()));
        manager.register("fixture", stage_runner(Arc::clone(&ran)));
        assert_eq!(manager.recover(), 1);

        let job = wait_for(&manager, "j1");
        assert_eq!(job.state, State::Done);
        assert_eq!(job.stage, "export");
        assert_eq!(
            *ran.lock().unwrap(),
            vec![
                "transcribe".to_string(),
                "translate".to_string(),
                "export".to_string()
            ]
        );
    }

    #[test]
    fn a_recovered_job_without_a_runner_ends_as_failed() {
        let store = Store::open_memory().unwrap();
        stored_job(&store, "dub", "transcribe");

        let manager = Manager::with_store(store);
        assert_eq!(manager.recover(), 1);

        let job = manager.get("j1").unwrap();
        assert_eq!(job.state, State::Failed);
        assert!(job.error.unwrap().contains("cannot be resumed"));
    }

    #[test]
    fn a_job_without_progress_for_ten_minutes_is_stalled() {
        let manager = Manager::in_memory();
        let id = manager.submit("dub", "A file", "hash", |handle| {
            for _ in 0..200 {
                if handle.cancelled() {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        });
        assert_eq!(manager.sweep_at(now() + STALL_SECONDS + 1), 1);
        let job = manager.get(&id).unwrap();
        assert_eq!(job.state, State::Stalled);
        let events = manager.subscribe(&id).unwrap();
        let event = events.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(event.job.state, State::Stalled);
    }

    #[test]
    fn sse_closes_after_the_terminal_event() {
        let manager = Manager::in_memory();
        let id = manager.submit("dub", "A file", "hash", |handle| {
            handle.progress("transcribe", 0.5);
            std::thread::sleep(Duration::from_millis(20));
            Ok(())
        });
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            manager.stream(&mut stream, &id).unwrap();
        });
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut answer = String::new();
        use std::io::Read;
        client.read_to_string(&mut answer).unwrap();
        worker.join().unwrap();
        assert!(answer.contains("Content-Type: text/event-stream"));
        assert!(answer.contains("data: "));
        assert!(answer.contains("\"state\":\"done\""));
    }
}
