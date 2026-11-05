use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fidelity {
    Captions,
    Neutral,
    Approximate,
    Cloned,
    EndToEnd,
}

impl Fidelity {
    pub fn level(self) -> u8 {
        match self {
            Self::Captions => 0,
            Self::Neutral => 1,
            Self::Approximate => 2,
            Self::Cloned => 3,
            Self::EndToEnd => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Captions => "captions",
            Self::Neutral => "neutral voice",
            Self::Approximate => "approximate voice",
            Self::Cloned => "cloned voice",
            Self::EndToEnd => "end to end transfer",
        }
    }
}

impl fmt::Display for Fidelity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.level(), self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum License {
    Apache2,
    Apache2OptionalGpl,
    CcByNcSa4,
    Proprietary,
}

impl License {
    pub fn label(self) -> &'static str {
        match self {
            Self::Apache2 => "Apache-2.0",
            Self::Apache2OptionalGpl => "Apache-2.0, optional GPL voice",
            Self::CcByNcSa4 => "CC BY-NC-SA 4.0",
            Self::Proprietary => "vendor terms",
        }
    }

    pub fn commercial_note(self) -> &'static str {
        match self {
            Self::Apache2 => "commercial use ok",
            Self::Apache2OptionalGpl => "Apache-2.0 engine, GPL voice optional",
            Self::CcByNcSa4 => "non commercial",
            Self::Proprietary => "check vendor terms",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EngineInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub fidelity: &'static [Fidelity],
    pub license: License,
    pub hardware: &'static str,
    pub requires_gpu: bool,
    pub available: bool,
    pub default: bool,
}

pub static ENGINES: LazyLock<Vec<EngineInfo>> = LazyLock::new(|| {
    crate::registry::PIPELINES
        .iter()
        .map(|spec| EngineInfo {
            id: spec.id,
            name: spec.name,
            fidelity: fidelity_set(spec),
            license: licence(spec.licence),
            hardware: spec.hardware,
            requires_gpu: spec.requires_gpu,
            available: spec.available,
            default: spec.default,
        })
        .collect()
});

fn fidelity_set(spec: &crate::registry::PipelineSpec) -> &'static [Fidelity] {
    let steps: Vec<Fidelity> = spec.fidelity.iter().map(|label| fidelity(label)).collect();
    Box::leak(steps.into_boxed_slice())
}

fn fidelity(label: &str) -> Fidelity {
    match label {
        "captions" => Fidelity::Captions,
        "neutral voice" => Fidelity::Neutral,
        "approximate voice" => Fidelity::Approximate,
        "cloned voice" => Fidelity::Cloned,
        _ => Fidelity::EndToEnd,
    }
}

fn licence(label: &str) -> License {
    match label {
        "Apache-2.0" => License::Apache2,
        "Apache-2.0, optional GPL voice" => License::Apache2OptionalGpl,
        "CC BY-NC-SA 4.0" => License::CcByNcSa4,
        _ => License::Proprietary,
    }
}

pub fn default_engine() -> &'static EngineInfo {
    ENGINES
        .iter()
        .find(|engine| engine.default)
        .expect("a default engine is declared")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Basic,
    Medium,
    High,
    Ultra,
}

impl Profile {
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn target_ms(self) -> (u32, u32) {
        match self {
            Self::Basic => (0, 2_000),
            Self::Medium => (3_000, 5_000),
            Self::High => (2_500, 4_000),
            Self::Ultra => (2_000, 3_000),
        }
    }
}

pub fn profile_for_cores(cores: usize) -> Profile {
    match cores {
        0..=5 => Profile::Basic,
        _ => Profile::Medium,
    }
}

pub fn profile_for(cores: usize, gpu_memory_mb: Option<u32>) -> Profile {
    match gpu_memory_mb {
        Some(memory) if memory >= 12 * 1024 => Profile::Ultra,
        Some(memory) if memory >= 6 * 1024 => Profile::High,
        _ => profile_for_cores(cores),
    }
}

pub fn detect_gpu_memory_mb() -> Option<u32> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.trim().parse::<u32>().ok())
}

pub fn fidelity_for(profile: Profile) -> Fidelity {
    match profile {
        Profile::Basic => Fidelity::Captions,
        Profile::Medium => Fidelity::Neutral,
        Profile::High => Fidelity::Approximate,
        Profile::Ultra => Fidelity::Cloned,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub requested: Fidelity,
    pub chosen: Fidelity,
    pub profile: Profile,
    pub reason: String,
}

pub fn chain(requested: Fidelity) -> Vec<Fidelity> {
    let mut steps = vec![requested];
    let mut level = requested.level();
    while level > 0 {
        level -= 1;
        steps.push(match level {
            0 => Fidelity::Captions,
            1 => Fidelity::Neutral,
            2 => Fidelity::Approximate,
            _ => Fidelity::Cloned,
        });
    }
    steps
}

pub fn choose(cores: usize, gpu_memory_mb: Option<u32>, requested: Fidelity) -> Choice {
    choose_with(
        cores,
        gpu_memory_mb,
        requested,
        &parlo_core::device::profile().describe(),
    )
}

pub fn choose_with(
    cores: usize,
    gpu_memory_mb: Option<u32>,
    requested: Fidelity,
    machine: &str,
) -> Choice {
    let profile = profile_for(cores, gpu_memory_mb);
    let supported = fidelity_for(profile);
    if requested.level() <= supported.level() {
        return Choice {
            requested,
            chosen: requested,
            profile,
            reason: format!(
                "{} is in use: this machine has {machine}.",
                requested.label()
            ),
        };
    }
    Choice {
        requested,
        chosen: supported,
        profile,
        reason: format!(
            "{} needs more than this machine has, so {} is used instead. This machine has {machine}.",
            requested.label(),
            supported.label()
        ),
    }
}

pub fn compute_is_forced_to_cpu(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("cpu")
}

pub fn choose_preferred(
    cores: usize,
    gpu_memory_mb: Option<u32>,
    requested: Fidelity,
    compute: &str,
    machine: &str,
) -> Choice {
    let forced = compute_is_forced_to_cpu(compute);
    let mut choice = choose_with(
        cores,
        if forced { None } else { gpu_memory_mb },
        requested,
        machine,
    );
    if forced {
        choice.reason = format!(
            "The processor is used because the setting says so, not the graphics card. {}",
            choice.reason
        );
    }
    choice
}

pub fn device_label(compute: &str, gpu_memory_mb: Option<u32>) -> &'static str {
    if compute_is_forced_to_cpu(compute) || gpu_memory_mb.is_none() {
        "cpu"
    } else {
        "gpu"
    }
}

fn log_path() -> std::path::PathBuf {
    crate::config::data_root()
        .join("logs")
        .join("engines.jsonl")
}

pub fn log_choice(choice: &Choice) -> Option<std::path::PathBuf> {
    static LAST: std::sync::OnceLock<std::sync::Mutex<Option<String>>> = std::sync::OnceLock::new();
    let line = serde_json::json!({
        "at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or_default(),
        "profile": choice.profile.label(),
        "requested": choice.requested.label(),
        "chosen": choice.chosen.label(),
        "reason": choice.reason,
    })
    .to_string();
    let last = LAST.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut last) = last.lock() {
        if last.as_deref() == Some(line.as_str()) {
            return Some(log_path());
        }
        *last = Some(line.clone());
    }
    let path = log_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }
    let entry = format!("{line}\n");
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    file.write_all(entry.as_bytes()).ok()?;
    Some(path)
}

pub fn recent_switches(count: usize) -> Vec<serde_json::Value> {
    let Ok(text) = std::fs::read_to_string(log_path()) else {
        return Vec::new();
    };
    let mut rows: Vec<serde_json::Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect();
    if rows.len() > count {
        rows = rows.split_off(rows.len() - count);
    }
    rows
}

pub fn detect_cores() -> usize {
    std::thread::available_parallelism()
        .map(|cores| cores.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chain_walks_down_to_captions() {
        let steps = chain(Fidelity::Cloned);
        assert_eq!(steps[0], Fidelity::Cloned);
        assert_eq!(*steps.last().unwrap(), Fidelity::Captions);
        assert_eq!(steps.len(), 4);
    }

    #[test]
    fn choosing_the_processor_lowers_the_profile_and_says_why() {
        let choice = choose_preferred(12, Some(16_384), Fidelity::Cloned, "cpu", "8 cores");
        assert_eq!(choice.profile, Profile::Medium);
        assert_eq!(choice.chosen, Fidelity::Neutral);
        assert!(choice.reason.contains("because the setting says so"));
        assert_eq!(device_label("cpu", Some(16_384)), "cpu");
        let automatic = choose_preferred(12, Some(16_384), Fidelity::Cloned, "auto", "8 cores");
        assert_eq!(automatic.profile, Profile::Ultra);
        assert_eq!(automatic.chosen, Fidelity::Cloned);
        assert_eq!(device_label("auto", Some(16_384)), "gpu");
        assert_eq!(device_label("auto", None), "cpu");
    }

    #[test]
    fn a_machine_without_a_gpu_keeps_a_neutral_voice_and_says_why() {
        let choice = choose_with(12, None, Fidelity::Cloned, "8 cores");
        assert_eq!(choice.chosen, Fidelity::Neutral);
        assert!(choice.reason.contains("neutral voice"));
        assert!(choice.reason.contains("8 cores"));
    }

    #[test]
    fn a_machine_that_can_clone_keeps_the_cloned_voice() {
        let choice = choose_with(12, Some(16_384), Fidelity::Cloned, "8 cores");
        assert_eq!(choice.chosen, Fidelity::Cloned);
        assert!(choice.reason.contains("cloned voice"));
    }

    #[test]
    fn cores_without_a_gpu_stay_at_medium() {
        assert_eq!(profile_for(12, None), Profile::Medium);
        assert_eq!(profile_for(4, None), Profile::Basic);
        assert_eq!(profile_for(12, Some(2_048)), Profile::Medium);
        assert_eq!(profile_for(4, Some(8_192)), Profile::High);
        assert_eq!(profile_for(4, Some(16_384)), Profile::Ultra);
    }

    #[test]
    fn declares_a_cpu_default_without_restricted_models() {
        let engine = default_engine();
        assert!(!engine.requires_gpu);
        assert_ne!(engine.license, License::CcByNcSa4);
        assert_ne!(engine.license, License::Proprietary);
    }

    #[test]
    fn keeps_non_commercial_engines_optional() {
        let hibiki = ENGINES.iter().find(|engine| engine.id == "hibiki").unwrap();
        assert_eq!(hibiki.license.commercial_note(), "non commercial");
        assert!(!hibiki.default);
        assert!(hibiki.requires_gpu);
    }

    #[test]
    fn has_exactly_one_default_engine() {
        assert_eq!(ENGINES.iter().filter(|engine| engine.default).count(), 1);
    }

    #[test]
    fn maps_cores_to_profiles() {
        assert_eq!(profile_for_cores(2), Profile::Basic);
        assert_eq!(profile_for_cores(4), Profile::Basic);
        assert_eq!(profile_for_cores(6), Profile::Medium);
        assert_eq!(profile_for_cores(16), Profile::Medium);
    }

    #[test]
    fn recommends_readable_modes_on_slow_machines() {
        assert_eq!(fidelity_for(Profile::Basic), Fidelity::Captions);
        assert_eq!(fidelity_for(Profile::Medium), Fidelity::Neutral);
    }

    #[test]
    fn fidelity_levels_are_ordered() {
        assert!(Fidelity::Captions.level() < Fidelity::Neutral.level());
        assert!(Fidelity::Cloned.level() < Fidelity::EndToEnd.level());
    }

    #[test]
    fn targets_narrow_with_better_hardware() {
        let medium = Profile::Medium.target_ms().1;
        let high = Profile::High.target_ms().1;
        assert!(high < medium);
    }

    #[test]
    fn labels_vendor_licenses_separately() {
        assert_eq!(License::Apache2.commercial_note(), "commercial use ok");
        assert_eq!(License::CcByNcSa4.commercial_note(), "non commercial");
        assert_eq!(License::Proprietary.commercial_note(), "check vendor terms");
        assert_eq!(
            License::Apache2OptionalGpl.commercial_note(),
            "Apache-2.0 engine, GPL voice optional"
        );
        assert_eq!(
            License::Apache2OptionalGpl.label(),
            "Apache-2.0, optional GPL voice"
        );
    }

    #[test]
    fn gpu_engines_are_flagged() {
        let hibiki = ENGINES.iter().find(|engine| engine.id == "hibiki").unwrap();
        assert!(hibiki.requires_gpu);
        assert!(!default_engine().requires_gpu);
    }
}
