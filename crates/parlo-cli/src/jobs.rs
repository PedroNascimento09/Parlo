use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
}

struct Shared {
    inner: Mutex<Inner>,
    store: Mutex<Option<Store>>,
}

pub struct Manager {
    shared: Arc<Shared>,
}

impl Manager {
    pub fn new() -> Self {
        let manager = Self {
            shared: Arc::new(Shared {
                inner: Mutex::new(Inner {
                    jobs: HashMap::new(),
                    watchers: HashMap::new(),
                }),
                store: Mutex::new(Store::open_default().ok()),
            }),
        };
        manager.recover();
        manager
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        let manager = Self {
            shared: Arc::new(Shared {
                inner: Mutex::new(Inner {
                    jobs: HashMap::new(),
                    watchers: HashMap::new(),
                }),
                store: Mutex::new(Store::open_memory().ok()),
            }),
        };
        manager.recover();
        manager
    }

    #[cfg(test)]
    fn with_store(store: Store) -> Self {
        let manager = Self {
            shared: Arc::new(Shared {
                inner: Mutex::new(Inner {
                    jobs: HashMap::new(),
                    watchers: HashMap::new(),
                }),
                store: Mutex::new(Some(store)),
            }),
        };
        manager.recover();
        manager
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
            stage: String::new(),
            progress: 0.0,
            title: title.to_string(),
            error: None,
            input_hash: input_hash.to_string(),
            created_at: created,
            updated_at: created,
        };
        {
            let mut inner = self.lock();
            inner.jobs.insert(id.clone(), job.clone());
            publish(&mut inner, &job);
        }
        self.shared.persist(&job);

        let handle = Handle {
            shared: Arc::clone(&self.shared),
            id: id.clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        std::thread::spawn(move || {
            {
                let mut inner = lock_shared(&handle.shared);
                let started = inner.jobs.get_mut(&handle.id).and_then(|job| {
                    if job.state == State::Queued {
                        job.state = State::Running;
                        job.stage = "starting".to_string();
                        job.updated_at = now();
                        Some(job.clone())
                    } else {
                        None
                    }
                });
                if let Some(job) = started {
                    publish(&mut inner, &job);
                }
            }
            let result = work(handle.clone());
            let mut inner = lock_shared(&handle.shared);
            let finished = {
                let Some(job) = inner.jobs.get_mut(&handle.id) else {
                    return;
                };
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
            };
            publish(&mut inner, &finished);
            drop(inner);
            handle.shared.persist(&finished);
        });
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
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            )
            .map_err(|error| error.to_string())?;
        loop {
            match events.recv_timeout(Duration::from_secs(15)) {
                Ok(event) => {
                    let terminal = event.job.state.terminal();
                    let payload = format!("data: {}\n\n", event.job.value());
                    if stream.write_all(payload.as_bytes()).is_err() {
                        break;
                    }
                    let _ = stream.flush();
                    if terminal {
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    if stream.write_all(b": keep-alive\n\n").is_err() {
                        break;
                    }
                    let _ = stream.flush();
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
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
        let recovered: Vec<Job> = stored
            .into_iter()
            .map(|stored| Job {
                id: stored.id,
                kind: stored.kind,
                state: State::Failed,
                stage: stored.stage,
                progress: stored.progress,
                title: stored.title,
                error: Some("the app stopped before this job finished".to_string()),
                input_hash: stored.input_hash,
                created_at: stored.created_at,
                updated_at: now(),
            })
            .collect();
        let mut inner = self.lock();
        for job in &recovered {
            inner.jobs.insert(job.id.clone(), job.clone());
            publish(&mut inner, job);
        }
        drop(inner);
        for job in &recovered {
            self.shared.persist(job);
        }
        recovered.len()
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
    MANAGER.get_or_init(Manager::new)
}

#[derive(Clone)]
pub struct Handle {
    shared: Arc<Shared>,
    id: String,
    cancelled: Arc<AtomicBool>,
}

impl Handle {
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn progress(&self, stage: &str, progress: f32) {
        let mut inner = lock_shared(&self.shared);
        let updated = {
            let Some(job) = inner.jobs.get_mut(&self.id) else {
                return;
            };
            if job.state.terminal() {
                return;
            }
            job.stage = stage.to_string();
            job.progress = progress.clamp(0.0, 1.0);
            job.updated_at = now();
            job.clone()
        };
        publish(&mut inner, &updated);
        drop(inner);
        self.shared.persist(&updated);
    }
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
    fn persist(&self, job: &Job) {
        let store = self.store.lock();
        let mut store = match store {
            Ok(store) => store,
            Err(_) => return,
        };
        if store.is_none() {
            *store = Store::open_default().ok();
        }
        if let Some(store) = store.as_ref() {
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
        }
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
        loop {
            match events.recv_timeout(Duration::from_secs(2)) {
                Ok(event) => {
                    if event.job.progress >= 0.4 {
                        saw_progress = true;
                    }
                    if event.job.state.terminal() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        assert!(saw_progress);
        wait_for(&manager, &id);
    }

    #[test]
    fn cancelling_reaches_a_terminal_state_and_ends_the_stream() {
        let manager = Manager::in_memory();
        let id = manager.submit("dub", "A file", "hash", |handle| {
            while !handle.cancelled() {
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        });
        std::thread::sleep(Duration::from_millis(20));
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
    }

    #[test]
    fn a_running_job_without_a_worker_ends_as_failed_on_recovery() {
        let store = Store::open_memory().unwrap();
        store
            .save_job(&JobRecord {
                id: "j1",
                kind: "models",
                state: "running",
                stage: "download",
                progress: 0.5,
                input_hash: "",
                title: "A file",
                error: None,
                created_at: 1,
                updated_at: 2,
            })
            .unwrap();
        let manager = Manager::with_store(store);
        let job = manager.get("j1").unwrap();
        assert_eq!(job.state, State::Failed);
        assert!(job.error.unwrap().contains("stopped"));
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
