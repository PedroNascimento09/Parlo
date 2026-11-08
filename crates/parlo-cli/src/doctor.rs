use std::env;
use std::process::Command;

use parlo_core::{AudioFormat, LatencyBudget};

use crate::{config, engines, models, sidecar, ENGINE_LATENCY_MS, VERSION};

pub(crate) fn doctor() -> Result<(), String> {
    let input = AudioFormat::mono(48_000).map_err(|e| e.to_string())?;
    let output = AudioFormat::mono(16_000).map_err(|e| e.to_string())?;
    let budget = LatencyBudget::new(20, ENGINE_LATENCY_MS, 5);

    println!("parlo {VERSION} doctor");
    println!();
    println!(
        "platform          {} {}",
        env::consts::OS,
        env::consts::ARCH
    );
    println!(
        "capture format    {} Hz mono, 20 ms frames",
        input.sample_rate()
    );
    println!("engine format     {} Hz mono", output.sample_rate());
    println!("engines           echo, sidecar (needs python and engines/hibiki)");
    println!();
    println!("latency budget");
    println!("  capture         {:>5} ms", budget.capture_ms);
    println!("  engine          {:>5} ms", budget.engine_ms);
    println!("  output          {:>5} ms", budget.output_ms);
    println!(
        "  total           {:>5} ms (target {} ms)",
        budget.total_ms(),
        LatencyBudget::TARGET_MS
    );
    println!();
    if budget.is_within_target() {
        println!("status            within target");
    } else if budget.is_within_ceiling() {
        println!("status            above target, within ceiling");
    } else {
        println!("status            above the usable ceiling");
    }
    println!("privacy           local only, no network calls, no telemetry");
    println!("engine binary     {}", engine_binary());
    println!("models            {}", models_summary());
    println!("config            {}", config_summary());
    println!("data              {}", config::data_root().display());
    println!("ffmpeg            {}", ffmpeg_version());
    println!();
    println!("sessions");
    for line in parlo_core::sessions::Log::default_log()
        .report(Some(parlo_core::sessions::current()))
        .lines()
    {
        println!("  {line}");
    }

    let cores = engines::detect_cores();
    let profile = engines::profile_for_cores(cores);
    println!(
        "profile           {} ({} cores), voice level {}",
        profile.label(),
        cores,
        engines::fidelity_for(profile)
    );
    println!();
    println!("first session");
    let checks = first_session_checks();
    let mut failed = 0;
    for (ok, line) in &checks {
        if !ok {
            failed += 1;
        }
        println!("  {:<4} {line}", if *ok { "ok" } else { "fail" });
    }
    println!();
    if failed > 0 {
        println!("status            not ready: {failed} checks need attention");
        let missing = models::status()
            .iter()
            .filter(|model| !model.installed)
            .count();
        if let Some(advice) = next_step(failed, missing) {
            println!("next              {advice}");
        }
        parlo_core::sessions::finish_current();
        std::process::exit(1);
    }
    println!("status            ready for a first session");
    Ok(())
}

fn next_step(failed: usize, missing: usize) -> Option<String> {
    if failed == 0 {
        return None;
    }
    if missing > 0 {
        return Some(format!(
            "open parlo app and press Install models: {missing} of them are not on this computer"
        ));
    }
    Some("fix the lines marked fail, then run parlo doctor again".to_string())
}

fn first_session_checks() -> Vec<(bool, String)> {
    let models = models::status();
    let mut checks = Vec::new();
    let missing: Vec<String> = models
        .iter()
        .filter(|model| !model.installed)
        .map(|model| model.name.clone())
        .collect();
    checks.push((
        missing.is_empty(),
        if missing.is_empty() {
            "every model this build knows is on disk".to_string()
        } else {
            format!("missing {}", missing.join(", "))
        },
    ));
    let chosen = config::load().stt;
    let dictation = models::dictation_ready_for(&models, &chosen);
    checks.push((
        dictation,
        format!(
            "dictation uses {chosen}, and that recogniser is {}",
            if dictation { "on disk" } else { "not on disk" }
        ),
    ));
    let shortcut = config::load().shortcut;
    checks.push((
        !shortcut.trim().is_empty(),
        format!("dictation shortcut {shortcut}"),
    ));
    let settings = config::path();
    checks.push((
        settings.is_file(),
        format!("settings at {}", settings.display()),
    ));
    let data = config::data_root();
    let probe = data.join(".first-session-check");
    let writable = std::fs::create_dir_all(&data)
        .and_then(|()| std::fs::write(&probe, b"ok"))
        .is_ok();
    let _ = std::fs::remove_file(&probe);
    checks.push((
        writable,
        format!(
            "data folder {} is {}",
            data.display(),
            if writable { "writable" } else { "not writable" }
        ),
    ));
    checks.push(match sidecar::bundled_engine() {
        Some(path) => (true, format!("engine at {}", path.display())),
        None => (
            true,
            "no bundled engine; a source checkout starts python instead".to_string(),
        ),
    });
    checks
}

fn engine_binary() -> String {
    match sidecar::bundled_engine() {
        Some(path) => path.display().to_string(),
        None => "none, this build uses python from a source checkout".to_string(),
    }
}

fn models_summary() -> String {
    let models = models::status();
    let installed = models.iter().filter(|model| model.installed).count();
    format!(
        "{} ({} of {} installed)",
        models::models_root().display(),
        installed,
        models.len()
    )
}

fn config_summary() -> String {
    let path = config::path();
    if path.is_file() {
        path.display().to_string()
    } else {
        format!("{} (not written yet, defaults in use)", path.display())
    }
}

fn ffmpeg_version() -> String {
    let output = Command::new("ffmpeg").arg("-version").output();
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("found, version unknown")
            .trim()
            .to_string(),
        _ => "not found, dubbing needs it".to_string(),
    }
}

#[cfg(test)]
mod next_step_tests {
    use super::*;

    #[test]
    fn a_ready_machine_gets_no_advice() {
        assert_eq!(next_step(0, 0), None);
    }

    #[test]
    fn missing_models_point_at_the_window() {
        let advice = next_step(2, 8).expect("advice");
        assert!(advice.contains("parlo app"), "{advice}");
        assert!(advice.contains("8"), "{advice}");
    }

    #[test]
    fn another_failure_points_at_the_failed_lines() {
        let advice = next_step(1, 0).expect("advice");
        assert!(advice.contains("marked fail"), "{advice}");
    }
}
