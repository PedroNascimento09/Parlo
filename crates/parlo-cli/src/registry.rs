#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Parakeet,
    Whisper,
    Voice(&'static str),
    Translation(&'static str, &'static str),
    Speaker(&'static str),
}

pub struct ModelSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub feature: &'static str,
    pub unit: Option<&'static str>,
    pub licence: &'static str,
    pub languages: &'static [&'static str],
    pub devices: &'static [&'static str],
    pub bytes: u64,
    pub capabilities: &'static [&'static str],
    pub source: Source,
    pub default: bool,
}

pub struct PipelineSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub licence: &'static str,
    pub hardware: &'static str,
    pub requires_gpu: bool,
    pub available: bool,
    pub default: bool,
    pub fidelity: &'static [&'static str],
}

const CPU: &[&str] = &["cpu"];

pub const PIPELINES: &[PipelineSpec] = &[
    PipelineSpec {
        id: "cascade",
        name: "Universal cascade",
        licence: "Apache-2.0, optional GPL voice",
        hardware: "any recent CPU",
        requires_gpu: false,
        available: true,
        default: true,
        fidelity: &[
            "captions",
            "neutral voice",
            "approximate voice",
            "cloned voice",
        ],
    },
    PipelineSpec {
        id: "echo",
        name: "Echo for development",
        licence: "Apache-2.0",
        hardware: "any",
        requires_gpu: false,
        available: true,
        default: false,
        fidelity: &["neutral voice"],
    },
    PipelineSpec {
        id: "hibiki",
        name: "Hibiki-Zero premium",
        licence: "CC BY-NC-SA 4.0",
        hardware: "NVIDIA 8 to 12 GB VRAM",
        requires_gpu: true,
        available: false,
        default: false,
        fidelity: &["end to end transfer"],
    },
    PipelineSpec {
        id: "cloud",
        name: "Cloud with your key",
        licence: "vendor terms",
        hardware: "any",
        requires_gpu: false,
        available: false,
        default: false,
        fidelity: &["end to end transfer"],
    },
];

pub const MODELS: &[ModelSpec] = &[
    ModelSpec {
        id: "parakeet",
        name: "Parakeet 0.6B v3",
        kind: "recogniser",
        feature: "Dictation",
        unit: Some("recognition"),
        licence: "CC-BY-4.0",
        languages: &["pt", "en"],
        devices: CPU,
        bytes: 671_000_000,
        capabilities: &["stream", "timestamps"],
        source: Source::Parakeet,
        default: true,
    },
    ModelSpec {
        id: "whisper",
        name: "Whisper base",
        kind: "recogniser",
        feature: "Dictation",
        unit: Some("recognition"),
        licence: "MIT",
        languages: &["pt", "en"],
        devices: CPU,
        bytes: 145_000_000,
        capabilities: &["timestamps"],
        source: Source::Whisper,
        default: false,
    },
    ModelSpec {
        id: "voice-pt",
        name: "Portuguese voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["pt"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("pt_PT-tugão-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-en",
        name: "English voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["en"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("en_US-lessac-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-de",
        name: "German voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["de"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("de_DE-thorsten-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-fr",
        name: "French voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["fr"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("fr_FR-siwis-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-es",
        name: "Spanish voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["es"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("es_ES-davefx-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-it",
        name: "Italian voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["it"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("it_IT-paola-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-nl",
        name: "Dutch voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["nl"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("nl_NL-mls-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "voice-ru",
        name: "Russian voice",
        kind: "voice",
        feature: "Read aloud",
        unit: Some("voice"),
        licence: "MIT",
        languages: &["ru"],
        devices: CPU,
        bytes: 63_000_000,
        capabilities: &["speak"],
        source: Source::Voice("ru_RU-irina-medium.onnx"),
        default: false,
    },
    ModelSpec {
        id: "speakers",
        name: "Speaker separation",
        kind: "speaker",
        feature: "Meetings",
        unit: None,
        licence: "Apache-2.0",
        languages: &["en"],
        devices: CPU,
        bytes: 28_000_000,
        capabilities: &["diarize"],
        source: Source::Speaker("3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-en",
        name: "Portuguese to English",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "CC-BY-4.0",
        languages: &["pt", "en"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "en"),
        default: false,
    },
    ModelSpec {
        id: "mt-en-pt",
        name: "English to Portuguese",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "CC-BY-4.0",
        languages: &["en", "pt"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("en", "pt"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-es",
        name: "Portuguese to Spanish",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["pt", "es"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "es"),
        default: false,
    },
    ModelSpec {
        id: "mt-es-pt",
        name: "Spanish to Portuguese",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["es", "pt"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("es", "pt"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-fr",
        name: "Portuguese to French",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["pt", "fr"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "fr"),
        default: false,
    },
    ModelSpec {
        id: "mt-fr-pt",
        name: "French to Portuguese",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["fr", "pt"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("fr", "pt"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-it",
        name: "Portuguese to Italian",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["pt", "it"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "it"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-nl",
        name: "Portuguese to Dutch",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["pt", "nl"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "nl"),
        default: false,
    },
    ModelSpec {
        id: "mt-pt-ru",
        name: "Portuguese to Russian",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["pt", "ru"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("pt", "ru"),
        default: false,
    },
    ModelSpec {
        id: "mt-en-es",
        name: "English to Spanish",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["en", "es"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("en", "es"),
        default: false,
    },
    ModelSpec {
        id: "mt-en-fr",
        name: "English to French",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["en", "fr"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("en", "fr"),
        default: false,
    },
    ModelSpec {
        id: "mt-en-it",
        name: "English to Italian",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["en", "it"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("en", "it"),
        default: false,
    },
    ModelSpec {
        id: "mt-de-en",
        name: "German to English",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["de", "en"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("de", "en"),
        default: false,
    },
    ModelSpec {
        id: "mt-fr-de",
        name: "French to German",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["fr", "de"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("fr", "de"),
        default: false,
    },
    ModelSpec {
        id: "mt-es-en",
        name: "Spanish to English",
        kind: "translation",
        feature: "Translation",
        unit: Some("translation"),
        licence: "Apache-2.0",
        languages: &["es", "en"],
        devices: CPU,
        bytes: 83_000_000,
        capabilities: &["translate"],
        source: Source::Translation("es", "en"),
        default: false,
    },
];

pub const ENGINE_VOICES: [&str; 7] = ["pt", "en", "es", "fr", "de", "it", "nl"];

pub fn model(id: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|spec| spec.id == id)
}

pub fn voice_file(language: &str) -> Option<(&'static str, &'static str)> {
    MODELS
        .iter()
        .find(|spec| spec.kind == "voice" && spec.languages == [language])
        .and_then(|spec| match spec.source {
            Source::Voice(file) => Some((spec.name, file)),
            _ => None,
        })
}

pub fn remaining_units() -> Vec<&'static str> {
    MODELS
        .iter()
        .filter(|spec| {
            matches!(
                spec.source,
                Source::Voice(_) | Source::Translation(_, _) | Source::Speaker(_)
            )
        })
        .map(|spec| spec.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_a_unique_id_and_a_licence() {
        let mut ids: Vec<&str> = MODELS.iter().map(|spec| spec.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), MODELS.len());
        assert!(MODELS.iter().all(|spec| !spec.licence.trim().is_empty()));
        assert!(MODELS.iter().all(|spec| !spec.languages.is_empty()));
    }

    #[test]
    fn every_language_parlo_translates_to_has_a_voice() {
        for spec in MODELS.iter().filter(|spec| spec.kind == "translation") {
            if let Source::Translation(_, target) = spec.source {
                assert!(
                    voice_file(target).is_some(),
                    "{target} has no voice to read it aloud"
                );
            }
        }
    }

    #[test]
    fn every_voice_can_be_downloaded() {
        let catalogue = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../engines/cascade/scripts/setup_models.py"),
        )
        .expect("the model setup script");
        for spec in MODELS.iter().filter(|spec| spec.kind == "voice") {
            if let Source::Voice(file) = spec.source {
                let name = file.trim_end_matches(".onnx");
                assert!(catalogue.contains(name), "{name} is not in the catalogue");
            }
        }
    }

    #[test]
    fn the_shared_voices_are_the_ones_the_engine_picks_itself() {
        let engine = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../engines/cascade/parlo_cascade/engine.py"),
        )
        .expect("the cascade engine");
        let table = engine
            .split("VOICES = {")
            .nth(1)
            .expect("the voice table")
            .split('}')
            .next()
            .expect("a closed voice table");
        let mut languages: Vec<&str> = table
            .lines()
            .filter_map(|line| line.split('"').nth(1))
            .collect();
        let mut expected = ENGINE_VOICES.to_vec();
        languages.sort_unstable();
        expected.sort_unstable();
        assert_eq!(languages, expected, "{table}");
    }

    #[test]
    fn exactly_one_model_is_the_dictation_default() {
        assert_eq!(MODELS.iter().filter(|spec| spec.default).count(), 1);
        assert_eq!(
            MODELS.iter().find(|spec| spec.default).map(|spec| spec.id),
            Some("parakeet")
        );
    }

    #[test]
    fn exactly_one_pipeline_is_the_default() {
        assert_eq!(PIPELINES.iter().filter(|spec| spec.default).count(), 1);
        assert_eq!(
            PIPELINES
                .iter()
                .find(|spec| spec.default)
                .map(|spec| spec.id),
            Some("cascade")
        );
    }

    #[test]
    fn the_portuguese_voice_is_european() {
        let spec = model("voice-pt").expect("a Portuguese voice is offered");
        match spec.source {
            Source::Voice(file) => assert_eq!(file, "pt_PT-tugão-medium.onnx"),
            _ => panic!("the Portuguese voice must point at a file"),
        }
    }

    #[test]
    fn the_remaining_units_cover_voices_pairs_and_speakers() {
        let units = remaining_units();
        assert!(units.contains(&"voice-en"));
        assert!(units.contains(&"mt-pt-en"));
        assert!(units.contains(&"speakers"));
        assert!(!units.contains(&"parakeet"));
        assert!(!units.contains(&"whisper"));
    }
}
