use std::path::Path;
use std::process::Command;
use std::time::Instant;

use parlo_core::{AudioFormat, EchoEngine, Pipeline, PipelineConfig};

pub const DEVICES: &[&str] = &["cpu", "gpu"];
const CASCADE_AUDIO: &str = "tests/fixtures/timing/source.wav";
type Runner<'a> = dyn Fn(&[String], &Path) -> Result<String, String> + 'a;

pub struct Reading {
    pub engine: &'static str,
    pub device: &'static str,
    pub state: String,
    pub rtf: Option<f64>,
    pub seconds: Option<f64>,
    pub frames: u64,
    pub hardware: String,
    pub profile: &'static str,
    pub commit: String,
    pub measured_at: u64,
}

impl Reading {
    fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "engine": self.engine,
            "device": self.device,
            "state": self.state,
            "rtf": self.rtf,
            "seconds": self.seconds,
            "frames": self.frames,
            "hardware": self.hardware,
            "profile": self.profile,
            "commit": self.commit,
            "measured_at": self.measured_at,
        })
    }
}

pub fn readings() -> Vec<Reading> {
    let cores = crate::engines::detect_cores();
    let gpu = crate::engines::detect_gpu_memory_mb();
    let profile = crate::engines::profile_for(cores, gpu).label();
    let hardware = hardware(cores, gpu);
    let commit = commit();
    let measured_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    let models = crate::models::status();
    let engine_dir = engine_dir();
    let run = |args: &[String], root: &Path| run_benchmark(&engine_dir, args, root);
    let mut readings = Vec::new();
    for spec in crate::registry::PIPELINES {
        for device in DEVICES {
            let (state, rtf, seconds, frames) =
                outcome(spec, device, gpu, &models, &run, &engine_dir);
            readings.push(Reading {
                engine: spec.id,
                device,
                state,
                rtf,
                seconds,
                frames,
                hardware: hardware.clone(),
                profile,
                commit: commit.clone(),
                measured_at,
            });
        }
    }
    readings
}

fn engine_dir() -> std::path::PathBuf {
    let configured = crate::models::cascade_dir();
    let script = |dir: &std::path::Path| dir.join("scripts").join("benchmark.py").is_file();
    if script(&configured) {
        return configured;
    }
    let relative = std::path::PathBuf::from("engines").join("cascade");
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(current) = std::env::current_dir() {
        roots.push(current);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    for root in roots {
        for ancestor in root.ancestors() {
            let candidate = ancestor.join(&relative);
            if script(&candidate) {
                return candidate;
            }
        }
    }
    configured
}

fn outcome(
    spec: &crate::registry::PipelineSpec,
    device: &str,
    gpu: Option<u32>,
    models: &[crate::models::Model],
    run: &Runner<'_>,
    engine_dir: &Path,
) -> (String, Option<f64>, Option<f64>, u64) {
    if spec.id == "echo" && device == "cpu" {
        let (frames, seconds, rtf) = throughput();
        return (
            "measured on this machine".to_string(),
            Some(rtf),
            Some(seconds),
            frames,
        );
    }
    if device == "gpu" && gpu.is_none() {
        return ("no gpu on this machine".to_string(), None, None, 0);
    }
    if !spec.available {
        return ("not in this build".to_string(), None, None, 0);
    }
    if spec.id == "cascade" && device == "cpu" {
        if !absent_models(models).is_empty() {
            return (missing(spec, models), None, None, 0);
        }
        return match cascade(engine_dir, run) {
            Ok((rtf, seconds, frames)) => (
                "measured on this machine".to_string(),
                Some(rtf),
                Some(seconds),
                frames,
            ),
            Err(reason) => (
                format!("no number: the cascade benchmark did not run: {reason}"),
                None,
                None,
                0,
            ),
        };
    }
    (missing(spec, models), None, None, 0)
}

fn absent_models(models: &[crate::models::Model]) -> Vec<&str> {
    models
        .iter()
        .filter(|model| (model.id == "parakeet" || model.id == "whisper") && !model.installed)
        .map(|model| model.name.as_str())
        .collect()
}

fn cascade(engine_dir: &Path, run: &Runner<'_>) -> Result<(f64, f64, u64), String> {
    let engine = engine_dir
        .canonicalize()
        .map_err(|error| format!("{}: {error}", engine_dir.display()))?;
    let args = benchmark_arguments(&engine)?;
    let stdout = run(&args, &crate::models::models_root())?;
    let report = report(&stdout)?;
    let rtf = report["rtf"]
        .as_f64()
        .ok_or_else(|| "the benchmark reported no real time factor".to_string())?;
    let seconds = report["wall_seconds"]
        .as_f64()
        .ok_or_else(|| "the benchmark reported no wall time".to_string())?;
    if rtf <= 0.0 || seconds <= 0.0 {
        return Err("the benchmark reported an empty run".to_string());
    }
    Ok((
        rtf,
        seconds,
        report["segments"].as_u64().unwrap_or_default(),
    ))
}

fn report(stdout: &str) -> Result<serde_json::Value, String> {
    let normalized = stdout.replace('\r', "\n");
    for line in normalized.lines().rev() {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            continue;
        }
        let mut stream =
            serde_json::Deserializer::from_str(trimmed).into_iter::<serde_json::Value>();
        if let Some(Ok(value)) = stream.next() {
            if value.get("rtf").is_some() {
                return Ok(value);
            }
        }
    }
    let starts: Vec<usize> = normalized
        .match_indices('{')
        .map(|(index, _)| index)
        .collect();
    for start in starts.into_iter().rev() {
        let slice = normalized[start..].trim();
        let mut stream = serde_json::Deserializer::from_str(slice).into_iter::<serde_json::Value>();
        let Ok(value) = stream
            .next()
            .unwrap_or(Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "empty",
            ))))
        else {
            continue;
        };
        if value.get("rtf").is_some() {
            return Ok(value);
        }
    }
    let last = stdout.lines().rev().take(8).collect::<Vec<_>>().join("\n");
    Err(format!(
        "the benchmark printed a report this app cannot read:\n{last}"
    ))
}

fn benchmark_arguments(engine_dir: &Path) -> Result<Vec<String>, String> {
    let script = engine_dir.join("scripts").join("benchmark.py");
    if !script.is_file() {
        return Err(format!("cannot find {}", script.display()));
    }
    let audio = engine_dir.join(CASCADE_AUDIO);
    if !audio.is_file() {
        return Err(format!("cannot find {}", audio.display()));
    }
    Ok(vec![
        script.to_string_lossy().into_owned(),
        audio.to_string_lossy().into_owned(),
        "--model".to_string(),
        "small".to_string(),
        "--source".to_string(),
        "pt".to_string(),
        "--target".to_string(),
        "en".to_string(),
    ])
}

fn run_benchmark(engine_dir: &Path, args: &[String], models_root: &Path) -> Result<String, String> {
    let python = crate::sidecar::python_for(engine_dir);
    let output = Command::new(&python)
        .args(args)
        .current_dir(engine_dir)
        .env("PYTHONPATH", engine_dir)
        .env("PARLO_MODELS_DIR", models_root)
        .output()
        .map_err(|error| format!("cannot start {python}: {error}"))?;
    let printed = String::from_utf8_lossy(&output.stdout);
    let logged = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let reason = logged
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("the benchmark printed no reason");
        return Err(reason.trim().to_string());
    }
    Ok(format!("{printed}\n{logged}"))
}

fn missing(spec: &crate::registry::PipelineSpec, models: &[crate::models::Model]) -> String {
    let absent = absent_models(models);
    if absent.is_empty() {
        return format!("no number: {} has no benchmark in this command", spec.name);
    }
    format!(
        "no number: {} needs {} on disk, and the python stack that reads them",
        spec.name,
        absent.join(" and ")
    )
}

fn throughput() -> (u64, f64, f64) {
    let input = AudioFormat::mono(48_000).expect("48 kHz mono is a valid format");
    let output = AudioFormat::mono(16_000).expect("16 kHz mono is a valid format");
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(input, output),
        Box::new(EchoEngine::new("echo", 1_500, 0.8)),
    )
    .expect("the echo pipeline starts");
    let speech = crate::synthetic_speech(input.sample_rate(), 2_500);
    let mut frames = 0;
    let started = Instant::now();
    for chunk in speech.chunks(480) {
        for _ in pipeline.push(chunk) {
            frames += 1;
        }
    }
    let seconds = started.elapsed().as_secs_f64();
    let audio = speech.len() as f64 / input.sample_rate() as f64;
    (frames, seconds, seconds / audio)
}

fn hardware(cores: usize, gpu: Option<u32>) -> String {
    let memory = crate::hardware::detect().memory_mb / 1024;
    match gpu {
        Some(gigabytes) => format!(
            "{cores} cores, {memory} GB of memory, a gpu with {} MB",
            gigabytes
        ),
        None => format!("{cores} cores, {memory} GB of memory, no gpu"),
    }
}

fn commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|commit| !commit.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn as_json() -> String {
    let rows: Vec<serde_json::Value> = readings().iter().map(Reading::as_json).collect();
    serde_json::json!({
        "schema": 1,
        "audio_seconds": 2.5,
        "rows": rows,
    })
    .to_string()
}

pub fn as_markdown() -> String {
    let rows: Vec<serde_json::Value> = readings().iter().map(Reading::as_json).collect();
    table(&rows)
}

fn table(rows: &[serde_json::Value]) -> String {
    let mut text = String::from(
        "| Engine | Device | Result | RTF | Frames | Hardware | Commit |\n|---|---|---|---|---|---|---|\n",
    );
    for row in rows {
        let rtf = match row["rtf"].as_f64() {
            Some(value) => format!("{value:.3}"),
            None => "-".to_string(),
        };
        text.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            row["engine"].as_str().unwrap_or("-"),
            row["device"].as_str().unwrap_or("-"),
            row["state"].as_str().unwrap_or("-"),
            rtf,
            row["frames"].as_u64().unwrap_or_default(),
            row["hardware"].as_str().unwrap_or("-"),
            row["commit"].as_str().unwrap_or("-"),
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_units() -> Vec<String> {
        crate::catalog::artifacts()
            .iter()
            .filter(|artifact| {
                ["rtf", "wer", "speed", "bench"]
                    .iter()
                    .any(|field| artifact.get(*field).is_some())
            })
            .map(|artifact| {
                artifact
                    .get("unit")
                    .and_then(|unit| unit.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn every_engine_has_a_row_for_every_device() {
        let readings = readings();
        for spec in crate::registry::PIPELINES {
            for device in DEVICES {
                let found = readings
                    .iter()
                    .filter(|reading| reading.engine == spec.id && reading.device == *device)
                    .count();
                assert_eq!(found, 1, "{} on {device} has {found} rows", spec.id);
            }
        }
        assert_eq!(
            readings.len(),
            crate::registry::PIPELINES.len() * DEVICES.len()
        );
    }

    #[test]
    fn every_row_names_its_hardware_its_profile_and_its_commit() {
        for reading in readings() {
            assert!(!reading.hardware.trim().is_empty(), "{}", reading.engine);
            assert!(!reading.state.trim().is_empty(), "{}", reading.engine);
            assert!(
                reading.commit.len() >= 4 && reading.commit.chars().all(|c| c.is_ascii_hexdigit()),
                "{}: {}",
                reading.engine,
                reading.commit
            );
            assert!(!reading.profile.is_empty(), "{}", reading.engine);
            assert!(reading.measured_at > 0, "{}", reading.engine);
        }
    }

    #[test]
    fn a_row_with_a_number_names_the_device_it_ran_on() {
        for reading in readings() {
            let Some(rtf) = reading.rtf else {
                assert_eq!(reading.frames, 0, "{}", reading.engine);
                assert!(reading.seconds.is_none(), "{}", reading.engine);
                continue;
            };
            assert!(rtf > 0.0, "{}", reading.engine);
            assert!(
                reading.seconds.unwrap_or_default() > 0.0,
                "{}",
                reading.engine
            );
            assert!(reading.frames > 0, "{}", reading.engine);
            assert!(
                reading.hardware.contains("cores"),
                "{}: {}",
                reading.engine,
                reading.hardware
            );
            let device = crate::engines::device_label(
                &crate::config::load().compute,
                crate::engines::detect_gpu_memory_mb(),
            );
            assert_eq!(reading.device, device, "{}", reading.engine);
        }
    }

    fn published() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../docs/benchmarks.json"))
            .expect("docs/benchmarks.json is readable")
    }

    fn documented_table() -> String {
        let document = include_str!("../../../docs/BENCHMARKS.md");
        let open = "<!-- bench:start -->";
        let start = document.find(open).expect("the table is marked") + open.len();
        let end = start
            + document[start..]
                .find("<!-- bench:end -->")
                .expect("the table ends");
        document[start..end].replace('\r', "")
    }

    #[test]
    fn the_published_table_is_the_one_the_run_generates() {
        let published = published();
        let rows = published["rows"].as_array().expect("the table has rows");
        assert_eq!(documented_table().trim(), table(rows).trim());
    }

    #[test]
    fn every_published_number_names_real_hardware_and_a_real_commit() {
        let reachable = std::process::Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !reachable {
            return;
        }
        let published = published();
        for row in published["rows"].as_array().expect("the table has rows") {
            let hardware = row["hardware"].as_str().unwrap_or("");
            assert!(hardware.contains("cores"), "{hardware}");
            let commit = row["commit"].as_str().unwrap_or("");
            let found = std::process::Command::new("git")
                .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            assert!(found, "{commit} is not a commit in this repository");
        }
    }

    #[test]
    fn the_published_table_matches_a_fresh_run() {
        let published = published();
        let rows = published["rows"].as_array().expect("the table has rows");
        let fresh = readings();
        assert_eq!(
            rows.len(),
            fresh.len(),
            "the published table has a different number of rows"
        );
        for reading in fresh {
            let row = rows
                .iter()
                .find(|row| row["engine"] == reading.engine && row["device"] == reading.device)
                .unwrap_or_else(|| {
                    panic!(
                        "{} on {} is missing from the table",
                        reading.engine, reading.device
                    )
                });
            assert_eq!(row["state"].as_str(), Some(reading.state.as_str()));
            assert!(!row["hardware"].as_str().unwrap_or("").is_empty());
            assert!(row["commit"].as_str().unwrap_or("").len() >= 4);
            if let Some(rtf) = row["rtf"].as_f64() {
                assert!(rtf > 0.0, "{}", reading.engine);
            }
        }
    }

    #[test]
    fn report_reads_json_mixed_with_warmup_warnings() {
        let stdout = "warmup deferred for recognition: bad allocation

{\"model\":\"small\",\"rtf\":2.02,\"wall_seconds\":5.67,\"segments\":1}
";
        let value = report(stdout).expect("report");
        assert_eq!(value["rtf"].as_f64(), Some(2.02));
        assert_eq!(value["segments"].as_u64(), Some(1));
    }

    #[test]
    fn no_card_claims_a_number_without_a_row() {
        let readings = readings();
        let claimed = catalog_units();
        assert!(!claimed.is_empty(), "no card claims a number");
        for unit in claimed {
            let spec = crate::registry::MODELS
                .iter()
                .find(|spec| spec.id == unit || spec.unit == Some(unit.as_str()));
            let devices: Vec<&str> = match spec {
                Some(spec) => spec.devices.to_vec(),
                None => vec!["cpu"],
            };
            for device in devices {
                let covered = readings
                    .iter()
                    .any(|reading| reading.engine == "cascade" && reading.device == device);
                assert!(
                    covered,
                    "{} claims a number but {device} has no row",
                    if unit.is_empty() { "a card" } else { &unit }
                );
            }
        }
    }
}

#[cfg(test)]
mod cascade_tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("parlo-bench-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn engine(folder: &Path, script: bool) -> PathBuf {
        let engine = folder.join("engine");
        fs::create_dir_all(engine.join("scripts")).unwrap();
        if script {
            fs::write(engine.join("scripts").join("benchmark.py"), "").unwrap();
        } else {
            let _ = fs::remove_file(engine.join("scripts").join("benchmark.py"));
        }
        let audio = engine.join("tests").join("fixtures").join("timing");
        fs::create_dir_all(&audio).unwrap();
        fs::write(audio.join("source.wav"), b"speech").unwrap();
        engine
    }

    fn parakeet(installed: bool) -> Vec<crate::models::Model> {
        vec![crate::models::Model {
            id: "parakeet",
            name: "Parakeet 0.6B v3".to_string(),
            kind: "stt",
            feature: "Dictation",
            installed,
            bytes: 671_088_640,
        }]
    }

    fn spec() -> &'static crate::registry::PipelineSpec {
        crate::registry::PIPELINES
            .iter()
            .find(|spec| spec.id == "cascade")
            .unwrap()
    }

    #[test]
    fn the_cascade_row_is_measured_by_the_sidecar_benchmark() {
        let folder = scratch("measured");
        let directory = engine(&folder, true);
        let seen = RefCell::new(Vec::<String>::new());
        let root = RefCell::new(PathBuf::new());
        let run = |args: &[String], models: &Path| {
            *seen.borrow_mut() = args.to_vec();
            *root.borrow_mut() = models.to_path_buf();
            Ok(
                r#"{"audio_seconds":2.81,"wall_seconds":1.18,"rtf":0.42,"segments":125}"#
                    .to_string(),
            )
        };
        let (state, rtf, seconds, frames) =
            outcome(spec(), "cpu", None, &parakeet(true), &run, &directory);
        assert_eq!(state, "measured on this machine");
        assert_eq!(rtf, Some(0.42));
        assert_eq!(seconds, Some(1.18));
        assert_eq!(frames, 125);
        let args = seen.borrow();
        assert!(args[0].ends_with("benchmark.py"), "{args:?}");
        assert!(args[1].ends_with("source.wav"), "{args:?}");
        assert!(args.contains(&"--model".to_string()), "{args:?}");
        assert_eq!(*root.borrow(), crate::models::models_root());
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_cascade_row_without_models_does_not_run_the_benchmark() {
        let folder = scratch("absent");
        let directory = engine(&folder, true);
        let run = |_: &[String], _: &Path| -> Result<String, String> {
            panic!("the benchmark must not run without its models")
        };
        let (state, rtf, _, frames) =
            outcome(spec(), "cpu", None, &parakeet(false), &run, &directory);
        assert!(state.contains("needs Parakeet 0.6B v3"), "{state}");
        assert_eq!(rtf, None);
        assert_eq!(frames, 0);
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_cascade_row_says_why_when_the_benchmark_fails_or_is_missing() {
        let folder = scratch("fails");
        let directory = engine(&folder, true);
        let failing = |_: &[String], _: &Path| Err("sherpa-onnx is not installed".to_string());
        let (state, rtf, _, _) =
            outcome(spec(), "cpu", None, &parakeet(true), &failing, &directory);
        assert!(state.contains("no number"), "{state}");
        assert!(state.contains("sherpa-onnx"), "{state}");
        assert_eq!(rtf, None);
        let without = engine(&folder, false);
        let unused = |_: &[String], _: &Path| Ok(String::new());
        let (state, _, _, _) = outcome(spec(), "cpu", None, &parakeet(true), &unused, &without);
        assert!(state.contains("benchmark.py"), "{state}");
        fs::remove_dir_all(&folder).unwrap();
    }
}
