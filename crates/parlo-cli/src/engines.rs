use std::fmt;

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
    Mit,
    CcByNcSa4,
    Proprietary,
}

impl License {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mit => "MIT",
            Self::CcByNcSa4 => "CC BY-NC-SA 4.0",
            Self::Proprietary => "vendor terms",
        }
    }

    pub fn commercial_note(self) -> &'static str {
        match self {
            Self::Mit => "commercial use ok",
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

pub const ENGINES: &[EngineInfo] = &[
    EngineInfo {
        id: "cascade",
        name: "Universal cascade",
        fidelity: &[
            Fidelity::Captions,
            Fidelity::Neutral,
            Fidelity::Approximate,
            Fidelity::Cloned,
        ],
        license: License::Mit,
        hardware: "any recent CPU",
        requires_gpu: false,
        available: true,
        default: true,
    },
    EngineInfo {
        id: "echo",
        name: "Echo for development",
        fidelity: &[Fidelity::Neutral],
        license: License::Mit,
        hardware: "any",
        requires_gpu: false,
        available: true,
        default: false,
    },
    EngineInfo {
        id: "hibiki",
        name: "Hibiki-Zero premium",
        fidelity: &[Fidelity::EndToEnd],
        license: License::CcByNcSa4,
        hardware: "NVIDIA 8 to 12 GB VRAM",
        requires_gpu: true,
        available: false,
        default: false,
    },
    EngineInfo {
        id: "cloud",
        name: "Cloud with your key",
        fidelity: &[Fidelity::EndToEnd],
        license: License::Proprietary,
        hardware: "any",
        requires_gpu: false,
        available: false,
        default: false,
    },
];

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
}

impl Profile {
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn target_ms(self) -> (u32, u32) {
        match self {
            Self::Basic => (0, 2_000),
            Self::Medium => (3_000, 5_000),
            Self::High => (2_500, 4_000),
        }
    }
}

pub fn profile_for_cores(cores: usize) -> Profile {
    match cores {
        0..=7 => Profile::Basic,
        8..=15 => Profile::Medium,
        _ => Profile::High,
    }
}

pub fn fidelity_for(profile: Profile) -> Fidelity {
    match profile {
        Profile::Basic => Fidelity::Captions,
        Profile::Medium => Fidelity::Neutral,
        Profile::High => Fidelity::Approximate,
    }
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
    fn declares_a_permissive_cpu_default() {
        let engine = default_engine();
        assert_eq!(engine.license.commercial_note(), "commercial use ok");
        assert!(!engine.requires_gpu);
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
        assert_eq!(profile_for_cores(6), Profile::Basic);
        assert_eq!(profile_for_cores(8), Profile::Medium);
        assert_eq!(profile_for_cores(16), Profile::High);
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
        assert_eq!(License::Mit.commercial_note(), "commercial use ok");
        assert_eq!(License::CcByNcSa4.commercial_note(), "non commercial");
        assert_eq!(License::Proprietary.commercial_note(), "check vendor terms");
    }

    #[test]
    fn gpu_engines_are_flagged() {
        let hibiki = ENGINES.iter().find(|engine| engine.id == "hibiki").unwrap();
        assert!(hibiki.requires_gpu);
        assert!(!default_engine().requires_gpu);
    }
}
