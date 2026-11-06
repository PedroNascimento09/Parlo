use std::sync::{Arc, Mutex};

use parlo_core::audio::{AudioFormat, Frame};
use parlo_core::engine::{Engine, EngineError, EngineOutput};
use parlo_core::{Pipeline, PipelineConfig};

struct Recorder {
    spoken: Arc<Mutex<Vec<String>>>,
}

impl Engine for Recorder {
    fn id(&self) -> &str {
        "recorder"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        0
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn speak(&mut self, text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        self.spoken
            .lock()
            .expect("the recorder lock")
            .push(text.to_string());
        Ok((vec![0, 0], 16_000))
    }
}

fn speaking_in(text: &str, language: Option<&str>) -> String {
    let spoken = Arc::new(Mutex::new(Vec::new()));
    let format = AudioFormat::mono(16_000).expect("16 kHz mono");
    let config = PipelineConfig::new(format, format);
    let mut pipeline = Pipeline::new(
        config,
        Box::new(Recorder {
            spoken: Arc::clone(&spoken),
        }),
    )
    .expect("the pipeline starts");
    pipeline
        .speak_in(text, language)
        .expect("the recorder speaks");
    let heard = spoken.lock().expect("the recorder lock");
    assert_eq!(heard.len(), 1, "the engine heard {heard:?}");
    heard[0].clone()
}

fn speaking(text: &str) -> String {
    speaking_in(text, None)
}

#[test]
fn the_pipeline_speaks_the_language_it_is_given() {
    assert_eq!(
        speaking_in("Parlo lê o JSON", Some("pt")),
        "Parlo lê o Jêison"
    );
    assert_eq!(
        speaking_in("Parlo speaks JSON", Some("en")),
        "PAR-loh speaks JAY-son"
    );
}

#[test]
fn the_pipeline_hands_the_engine_a_respelled_sentence() {
    assert_eq!(speaking("Parlo reads JSON"), "PAR-loh reads JAY-son");
}

#[test]
fn the_pipeline_leaves_an_unknown_word_alone() {
    assert_eq!(
        speaking("O pardal comeu o API."),
        "O pardal comeu o ay-pee-EYE."
    );
}
