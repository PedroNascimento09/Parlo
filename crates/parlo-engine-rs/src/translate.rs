use std::path::{Path, PathBuf};

use ct2rs::sys::{ComputeType, Config, Device, TranslationOptions, Translator as Ct2};
use sentencepiece::SentencePieceProcessor;

pub struct Translator {
    inner: Ct2,
    source: SentencePieceProcessor,
    target: SentencePieceProcessor,
}

impl Translator {
    pub fn open(folder: &Path, threads: usize) -> Result<Self, String> {
        let weights = find_weights(folder)?;
        let (source_model, target_model) = find_vocabulary(&weights)?;
        let config = Config {
            device: Device::CPU,
            compute_type: ComputeType::INT8,
            num_threads_per_replica: threads,
            ..Config::default()
        };
        let inner = Ct2::new(&weights, &config)
            .map_err(|error| format!("the translation model could not be loaded: {error}"))?;
        Ok(Self {
            inner,
            source: open_pieces(&source_model)?,
            target: open_pieces(&target_model)?,
        })
    }

    pub fn translate(&self, text: &str) -> Result<String, String> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(String::new());
        }
        let pieces: Vec<String> = self
            .source
            .encode(text)
            .map_err(|error| format!("the source text could not be split: {error}"))?
            .into_iter()
            .map(|piece| piece.piece)
            .collect();
        let options = TranslationOptions::<String, String> {
            beam_size: 4,
            repetition_penalty: 1.2,
            no_repeat_ngram_size: 3,
            ..TranslationOptions::default()
        };
        let results = self
            .inner
            .translate_batch(&[pieces], &options, None)
            .map_err(|error| format!("the translation failed: {error}"))?;
        let hypothesis = results
            .first()
            .and_then(|result| result.hypotheses.first())
            .ok_or_else(|| "the translation came back empty".to_string())?;
        let translated = self
            .target
            .decode_pieces(hypothesis)
            .map_err(|error| format!("the translation could not be read back: {error}"))?;
        Ok(strip_language_token(&translated))
    }
}

fn open_pieces(path: &Path) -> Result<SentencePieceProcessor, String> {
    SentencePieceProcessor::open(path)
        .map_err(|error| format!("{} could not be opened: {error}", path.display()))
}

pub fn strip_language_token(text: &str) -> String {
    let mut cleaned = text
        .replace('\u{2581}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    while let Some(start) = cleaned.rfind(">>") {
        let tail = &cleaned[start..];
        let Some(end) = tail.find("<<") else { break };
        let token = &tail[2..end];
        if token.is_empty()
            || !token
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            break;
        }
        cleaned.truncate(start);
        cleaned = cleaned
            .trim_end()
            .trim_end_matches([' ', ',', ';', ':', '.'])
            .to_string();
    }
    cleaned
}

fn find_weights(folder: &Path) -> Result<PathBuf, String> {
    for candidate in [
        folder.join("ct2"),
        folder.to_path_buf(),
        folder.join("model"),
    ] {
        if candidate.join("model.bin").is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "no CTranslate2 model in {}; download the translation model first",
        folder.display()
    ))
}

fn find_vocabulary(weights: &Path) -> Result<(PathBuf, PathBuf), String> {
    let mut bases = vec![weights.to_path_buf()];
    if let Some(parent) = weights.parent() {
        bases.push(parent.to_path_buf());
        bases.push(parent.join("hf"));
    }
    for base in &bases {
        let source = base.join("source.spm");
        let target = base.join("target.spm");
        if source.is_file() && target.is_file() {
            return Ok((source, target));
        }
    }
    for base in bases.iter().take(2) {
        for name in ["sentencepiece.model", "sentencepiece.bpe.model"] {
            let single = base.join(name);
            if single.is_file() {
                return Ok((single.clone(), single));
            }
        }
    }
    Err(format!(
        "no sentencepiece model beside {}; the translation model is incomplete",
        weights.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trailing_language_token_is_removed() {
        assert_eq!(strip_language_token("hello there >>pt_br<<"), "hello there");
    }

    #[test]
    fn a_missing_model_says_so() {
        let error = Translator::open(Path::new("does-not-exist"), 1)
            .err()
            .unwrap();
        assert!(error.contains("no CTranslate2 model"));
    }

    #[test]
    fn weights_are_found_in_a_ct2_folder() {
        let root = std::env::temp_dir().join(format!("parlo-mt-{}", std::process::id()));
        std::fs::create_dir_all(root.join("ct2")).unwrap();
        std::fs::write(root.join("ct2").join("model.bin"), b"x").unwrap();
        assert_eq!(find_weights(&root).unwrap(), root.join("ct2"));
        std::fs::remove_dir_all(&root).ok();
    }
}
