use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use parlo_core::sessions::Log;

pub const DEFAULT_SECONDS: u64 = 180;
pub const DEFAULT_INTERVAL_SECONDS: u64 = 15;
pub const MAX_SAMPLES: usize = 480;
pub const READY_TIMEOUT_SECONDS: u64 = 60;
pub const STOP_TIMEOUT_SECONDS: u64 = 10;

pub fn command(args: &[String]) -> Result<(), String> {
    let mut seconds = DEFAULT_SECONDS;
    let mut interval = DEFAULT_INTERVAL_SECONDS;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--seconds" => {
                index += 1;
                seconds = args
                    .get(index)
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            "--interval" => {
                index += 1;
                interval = args
                    .get(index)
                    .ok_or("--interval needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }
    if seconds == 0 {
        return Err("--seconds must be at least 1".to_string());
    }
    if interval == 0 {
        return Err("--interval must be at least 1".to_string());
    }

    let port = free_port()?;
    let data = std::env::temp_dir().join(format!("parlo-soak-{}", std::process::id()));
    std::fs::create_dir_all(&data).map_err(|error| format!("{}: {error}", data.display()))?;
    let log = Log::at(data.join("sessions.jsonl"));
    let parent = parlo_core::sessions::current();
    let trace = data.join("serve.log");
    let errors = File::create(&trace).map_err(|error| format!("{}: {error}", trace.display()))?;

    let mut child = Command::new(std::env::current_exe().map_err(|error| error.to_string())?)
        .arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .arg("--token")
        .arg("soak")
        .arg("--engine")
        .arg("echo")
        .env("PARLO_DATA_DIR", &data)
        .env("PARLO_CONFIG", data.join("config.json"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::from(errors))
        .spawn()
        .map_err(|error| format!("cannot start the server: {error}"))?;

    if let Err(error) = wait_ready(&mut child, port, &trace) {
        stop(&mut child, &log, parent);
        return Err(error);
    }

    let started = Instant::now();
    let mut first = 0u64;
    let mut last = 0u64;
    let mut peak = 0u64;
    let mut samples = 0usize;
    while started.elapsed().as_secs() < seconds && samples < MAX_SAMPLES {
        std::thread::sleep(Duration::from_secs(interval));
        let rss = match rss_bytes(child.id()) {
            Ok(rss) => rss,
            Err(error) => {
                let error = format!("{error}: {}", tail(&trace));
                stop(&mut child, &log, parent);
                return Err(error);
            }
        };
        if !healthy(port) {
            let error = format!("the server stopped answering /health: {}", tail(&trace));
            stop(&mut child, &log, parent);
            return Err(error);
        }
        if samples == 0 {
            first = rss;
        }
        last = rss;
        peak = peak.max(rss);
        samples += 1;
    }
    let measured = started.elapsed();

    stop(&mut child, &log, parent);
    let report = log.report(Some(parent));
    let rate = report.rate().unwrap_or(100.0);

    println!(
        "soak              {seconds} s asked, {} s measured",
        measured.as_secs()
    );
    println!(
        "memory            {} MB first, {} MB last, {} MB peak, {samples} samples every {interval} s",
        megabytes(first),
        megabytes(last),
        megabytes(peak)
    );
    println!(
        "sessions          {} recorded, {} crashed ({rate:.1}% crash free)",
        report.sessions, report.crashed
    );
    if samples == 0 {
        return Err("the run finished before the first sample".to_string());
    }
    if report.crashed > 0 {
        return Err(format!(
            "{} of {} sessions crashed during the run",
            report.crashed, report.sessions
        ));
    }
    Ok(())
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1_048_576.0)
}

fn tail(path: &std::path::Path) -> String {
    std::fs::read_to_string(path)
        .map(|text| text.lines().last().unwrap_or_default().to_string())
        .unwrap_or_default()
}

fn free_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| error.to_string())
}

fn wait_ready(child: &mut Child, port: u16, trace: &std::path::Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(READY_TIMEOUT_SECONDS);
    while Instant::now() < deadline {
        if healthy(port) {
            return Ok(());
        }
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Err(format!("the server stopped with {status}: {}", tail(trace)));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!(
        "the server did not answer /health on {port}: {}",
        tail(trace)
    ))
}

fn healthy(port: u16) -> bool {
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_secs(2)) else {
        return false;
    };
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut answer = String::new();
    stream.read_to_string(&mut answer).is_ok() && answer.contains(" 200 ")
}

fn stop(child: &mut Child, log: &Log, parent: u64) {
    let deadline = Instant::now() + Duration::from_secs(STOP_TIMEOUT_SECONDS);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(100));
                if Instant::now() >= deadline {
                    child.kill().ok();
                    child.wait().ok();
                    break;
                }
            }
            _ => {
                child.kill().ok();
                child.wait().ok();
                break;
            }
        }
    }
    if let Some(id) = log.open_session(Some(parent)) {
        log.end(id);
    }
}

#[cfg(target_os = "linux")]
fn rss_bytes(pid: u32) -> Result<u64, String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"))
        .map_err(|error| format!("cannot read the memory of {pid}: {error}"))?;
    for line in status.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let kilobytes: u64 = value
                .trim()
                .trim_end_matches(" kB")
                .parse()
                .map_err(|_| "VmRSS is not a number".to_string())?;
            return Ok(kilobytes * 1024);
        }
    }
    Err("VmRSS is missing from /proc".to_string())
}

#[cfg(target_os = "macos")]
fn rss_bytes(pid: u32) -> Result<u64, String> {
    let output = Command::new("ps")
        .arg("-o")
        .arg("rss=")
        .arg("-p")
        .arg(pid.to_string())
        .output()
        .map_err(|error| format!("cannot read the memory of {pid}: {error}"))?;
    let kilobytes: u64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| format!("the memory of {pid} is not a number"))?;
    Ok(kilobytes * 1024)
}

#[cfg(target_os = "windows")]
fn rss_bytes(pid: u32) -> Result<u64, String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("(Get-Process -Id {pid} -ErrorAction Stop).WorkingSet64"),
        ])
        .output()
        .map_err(|error| format!("cannot read the memory of {pid}: {error}"))?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| format!("the memory of {pid} is not a number"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn rss_bytes(_pid: u32) -> Result<u64, String> {
    Err("this platform has no memory reader".to_string())
}
