use std::path::Path;

use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineTransducerModelConfig};

pub const SAMPLE_RATE: i32 = 16_000;

const FILES: [&str; 4] = [
    "encoder.int8.onnx",
    "decoder.int8.onnx",
    "joiner.int8.onnx",
    "tokens.txt",
];

pub struct Recognizer {
    inner: OfflineRecognizer,
}

impl Recognizer {
    pub fn open(directory: &Path, threads: i32) -> Result<Self, String> {
        for name in FILES {
            if !directory.join(name).is_file() {
                return Err(format!(
                    "dictation needs {name} in {}; download the dictation model first",
                    directory.display()
                ));
            }
        }
        let path = |name: &str| Some(directory.join(name).to_string_lossy().into_owned());
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.transducer = OfflineTransducerModelConfig {
            encoder: path("encoder.int8.onnx"),
            decoder: path("decoder.int8.onnx"),
            joiner: path("joiner.int8.onnx"),
        };
        config.model_config.tokens = path("tokens.txt");
        config.model_config.num_threads = threads.max(1);
        config.model_config.model_type = Some("nemo_transducer".into());
        config.model_config.provider = Some("cpu".into());
        let inner = OfflineRecognizer::create(&config).ok_or_else(|| {
            format!(
                "the dictation model in {} could not be loaded; it may be incomplete",
                directory.display()
            )
        })?;
        Ok(Self { inner })
    }

    pub fn transcribe(&self, samples: &[f32]) -> String {
        if samples.is_empty() {
            return String::new();
        }
        let stream = self.inner.create_stream();
        stream.accept_waveform(SAMPLE_RATE, samples);
        self.inner.decode(&stream);
        stream
            .get_result()
            .map(|result| result.text.trim().to_string())
            .unwrap_or_default()
    }
}
