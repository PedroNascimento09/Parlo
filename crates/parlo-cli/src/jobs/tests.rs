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
    let mut client = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
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
