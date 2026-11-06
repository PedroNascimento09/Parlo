use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const MAX_RECORDS: usize = 400;

thread_local! {
    static CURRENT: Cell<u64> = const { Cell::new(0) };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    event: String,
    id: u64,
    at: u64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    location: String,
}

#[derive(Debug, Clone)]
pub struct Log {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Last {
    pub seconds: u64,
    pub clean: bool,
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
    pub sessions: usize,
    pub clean: usize,
    pub crashed: usize,
    pub last: Option<Last>,
}

pub struct Session {
    log: Log,
    id: u64,
    closed: bool,
}

impl Log {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_log() -> Self {
        Self {
            path: crate::settings::data_root().join("sessions.jsonl"),
        }
    }

    pub fn begin(&self) -> Session {
        let id = now_millis();
        CURRENT.with(|current| current.set(id));
        self.append(&Record {
            event: "start".to_string(),
            id,
            at: now(),
            message: String::new(),
            location: String::new(),
        });
        Session {
            log: self.clone(),
            id,
            closed: false,
        }
    }

    pub fn end(&self, id: u64) {
        CURRENT.with(|current| {
            if current.get() == id {
                current.set(0);
            }
        });
        self.append(&Record {
            event: "end".to_string(),
            id,
            at: now(),
            message: String::new(),
            location: String::new(),
        });
    }

    pub fn crash(&self, id: u64, message: &str, location: &str) {
        self.append(&Record {
            event: "crash".to_string(),
            id,
            at: now(),
            message: message.to_string(),
            location: location.to_string(),
        });
    }

    pub fn report(&self, skip: Option<u64>) -> Report {
        let records = self.records();
        let mut starts: Vec<&Record> = records
            .iter()
            .filter(|record| record.event == "start" && Some(record.id) != skip)
            .collect();
        starts.sort_by_key(|record| record.id);
        let mut report = Report::default();
        for start in starts {
            report.sessions += 1;
            let end = records
                .iter()
                .find(|record| record.event == "end" && record.id == start.id);
            let crash = records
                .iter()
                .find(|record| record.event == "crash" && record.id == start.id);
            let clean = end.is_some() && crash.is_none();
            if clean {
                report.clean += 1;
            } else {
                report.crashed += 1;
            }
            report.last = Some(match (crash, end) {
                (Some(crash), _) => Last {
                    seconds: crash.at.saturating_sub(start.at),
                    clean: false,
                    location: crash.location.clone(),
                    message: crash.message.clone(),
                },
                (None, Some(end)) => Last {
                    seconds: end.at.saturating_sub(start.at),
                    clean: true,
                    location: String::new(),
                    message: String::new(),
                },
                (None, None) => Last {
                    seconds: 0,
                    clean: false,
                    location: String::new(),
                    message: "the process stopped without a clean exit".to_string(),
                },
            });
        }
        report
    }

    pub fn open_session(&self, skip: Option<u64>) -> Option<u64> {
        let records = self.records();
        records
            .iter()
            .rev()
            .find(|start| {
                start.event == "start"
                    && Some(start.id) != skip
                    && !records.iter().any(|record| {
                        (record.event == "end" || record.event == "crash") && record.id == start.id
                    })
            })
            .map(|start| start.id)
    }

    fn records(&self) -> Vec<Record> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(_) => return Vec::new(),
        };
        text.lines()
            .filter_map(|line| serde_json::from_str::<Record>(line).ok())
            .collect()
    }

    fn append(&self, record: &Record) {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let line = match serde_json::to_string(record) {
            Ok(line) => line + "\n",
            Err(_) => return,
        };
        let written = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .and_then(|mut file| file.write_all(line.as_bytes()));
        if written.is_ok() {
            self.trim();
        }
    }

    fn trim(&self) {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(_) => return,
        };
        let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
        if lines.len() <= MAX_RECORDS {
            return;
        }
        let kept = lines[lines.len() - MAX_RECORDS..].join("\n") + "\n";
        let temporary = self.path.with_extension("jsonl.tmp");
        if fs::write(&temporary, kept).is_ok() {
            fs::rename(&temporary, &self.path).ok();
        }
    }
}

impl Session {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn finish(mut self) {
        self.close();
    }

    fn close(&mut self) {
        if !self.closed {
            self.closed = true;
            self.log.end(self.id);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

impl Report {
    pub fn rate(&self) -> Option<f64> {
        (self.sessions > 0).then(|| self.clean as f64 * 100.0 / self.sessions as f64)
    }

    pub fn lines(&self) -> Vec<String> {
        let Some(rate) = self.rate() else {
            return vec!["no session has been recorded yet".to_string()];
        };
        let mut lines = vec![format!(
            "{} recorded, {} clean, {} crashed ({rate:.1}% crash free)",
            self.sessions, self.clean, self.crashed
        )];
        if let Some(last) = &self.last {
            lines.push(if last.clean {
                format!("last ran {} s and ended clean", last.seconds)
            } else if last.location.is_empty() {
                format!("last {}", last.message)
            } else {
                format!("last crashed in {}: {}", last.location, last.message)
            });
        }
        lines
    }
}

pub fn current() -> u64 {
    CURRENT.with(Cell::get)
}

pub fn finish_current() {
    let id = current();
    if id != 0 {
        Log::default_log().end(id);
    }
}

pub fn install_hook(log: Log) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let id = current();
        if id != 0 {
            let location = info
                .location()
                .map(|place| format!("{}:{}:{}", place.file(), place.line(), place.column()))
                .unwrap_or_default();
            log.crash(id, &payload(info), &location);
        }
        previous(info);
    }));
}

fn payload(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(text) = info.payload().downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = info.payload().downcast_ref::<String>() {
        return text.clone();
    }
    "panic without a message".to_string()
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary() -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "parlo-sessions-{}-{unique}/log",
            std::process::id()
        ))
    }

    #[test]
    fn a_panic_is_recorded_as_a_crashed_session() {
        let log = Log::at(temporary().join("sessions.jsonl"));
        install_hook(log.clone());
        let session = log.begin();
        let outcome = std::panic::catch_unwind(|| panic!("boom in the session log test"));
        assert!(outcome.is_err());
        let report = log.report(None);
        assert_eq!(report.sessions, 1, "{report:?}");
        assert_eq!(report.crashed, 1, "{report:?}");
        let last = report.last.as_ref().expect("the last session");
        assert!(
            last.message.contains("boom in the session log test"),
            "{last:?}"
        );
        assert!(last.location.contains("sessions.rs"), "{last:?}");
        CURRENT.with(|current| current.set(0));
        drop(session);
        let _ = fs::remove_dir_all(temporary().parent().unwrap_or(&temporary()));
    }

    #[test]
    fn a_clean_session_is_not_counted_as_a_crash() {
        let log = Log::at(temporary().join("sessions.jsonl"));
        let session = log.begin();
        session.finish();
        let report = log.report(None);
        assert_eq!(report.sessions, 1, "{report:?}");
        assert_eq!(report.clean, 1, "{report:?}");
        assert_eq!(report.crashed, 0, "{report:?}");
        assert_eq!(report.rate(), Some(100.0));
        let lines = report.lines();
        assert!(lines[0].contains("100.0% crash free"), "{lines:?}");
        assert!(lines[1].starts_with("last ran"), "{lines:?}");
    }

    #[test]
    fn a_corrupt_log_does_not_stop_the_startup_or_the_report() {
        let path = temporary().join("sessions.jsonl");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "not json at all\n{\"event\":\"start\",\"id\":1,\"at\":10}\n{\"event\":\n",
        )
        .unwrap();
        let log = Log::at(path.clone());
        let session = log.begin();
        session.finish();
        let report = log.report(None);
        assert_eq!(report.sessions, 2, "{report:?}");
        assert_eq!(report.crashed, 1, "{report:?}");
        assert_eq!(report.clean, 1, "{report:?}");
        let created = Log::at(temporary().join("missing/sessions.jsonl"));
        created.begin().finish();
        assert_eq!(created.report(None).clean, 1);
    }

    #[test]
    fn the_report_names_the_rate_to_one_decimal() {
        let report = Report {
            sessions: 3,
            clean: 2,
            crashed: 1,
            last: Some(Last {
                seconds: 12,
                clean: true,
                location: String::new(),
                message: String::new(),
            }),
        };
        let lines = report.lines();
        assert_eq!(
            lines[0],
            "3 recorded, 2 clean, 1 crashed (66.7% crash free)"
        );
        assert_eq!(lines[1], "last ran 12 s and ended clean");
        let empty = Report::default();
        assert_eq!(empty.lines(), vec!["no session has been recorded yet"]);
        assert_eq!(empty.rate(), None);
    }

    #[test]
    fn the_log_keeps_only_the_newest_records() {
        let log = Log::at(temporary().join("sessions.jsonl"));
        for _ in 0..(MAX_RECORDS / 2 + 20) {
            log.begin().finish();
        }
        let text = fs::read_to_string(&log.path).unwrap();
        assert_eq!(
            text.lines().count(),
            MAX_RECORDS,
            "the log grew past its cap"
        );
        assert_eq!(log.report(None).sessions, MAX_RECORDS / 2);
    }
}
