use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use parlo_core::engine::EngineError;
use parlo_core::{AudioFormat, Pipeline, PipelineConfig};

use crate::insert;
use crate::models;

use super::json_string;

pub(super) struct Chosen {
    pub(super) engine: String,
    pub(super) stt: String,
    pub(super) from: String,
    pub(super) to: String,
    pub(super) write: String,
    pub(super) shortcut: String,
    pub(super) hands_free: String,
    pub(super) captions: String,
    pub(super) command: String,
    pub(super) copy_shortcut: String,
    pub(super) undo: String,
}

#[derive(Clone)]
pub(super) struct Target {
    pub(super) window: usize,
    pub(super) app: String,
}

pub(super) struct Session {
    pub(super) pipeline: Mutex<Pipeline>,
    pub(super) settings: RwLock<Chosen>,
    pub(super) target: Mutex<Option<Target>>,
    pub(super) busy: AtomicBool,
    pub(super) known: Mutex<Option<Vec<parlo_core::UnitState>>>,
}

pub(super) struct Working<'a>(&'a AtomicBool);

impl Drop for Working<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

pub(super) fn rooms() -> &'static Mutex<std::collections::HashMap<String, crate::speakers::Room>> {
    static ROOMS: std::sync::OnceLock<
        Mutex<std::collections::HashMap<String, crate::speakers::Room>>,
    > = std::sync::OnceLock::new();
    ROOMS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

pub(super) fn build(from: &str, to: &str, engine: &str, stt: &str) -> Result<Pipeline, String> {
    crate::config::check_pair(from, to)?;
    crate::config::check_stt(stt)?;
    std::env::set_var("PARLO_CASCADE_SOURCE", from);
    std::env::set_var("PARLO_CASCADE_TARGET", to);
    std::env::set_var("PARLO_CASCADE_STT", stt);
    let format = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    Pipeline::new(
        PipelineConfig::new(format, format),
        crate::build_engine(engine)?,
    )
    .map_err(|error| error.to_string())
}

impl Session {
    pub(super) fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(super) fn working(&self) -> Working<'_> {
        self.busy.store(true, Ordering::Relaxed);
        Working(&self.busy)
    }

    pub(super) fn is_working(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }

    pub(super) fn warm(&self) {
        static WARM: std::sync::Once = std::sync::Once::new();
        WARM.call_once(|| self.do_warm());
    }

    fn do_warm(&self) {
        let stt = self.chosen().stt.clone();
        let format = match AudioFormat::mono(16_000) {
            Ok(format) => format,
            Err(error) => {
                println!("  warm       {stt} stays cold: {error}");
                return;
            }
        };
        let silence = parlo_core::wav::write_pcm16(&parlo_core::Wav {
            format,
            samples: vec![0.0; 48_000],
        });
        match self.dictate(&silence) {
            Ok(_) => println!(
                "  warm       {stt} answered a silent clip, so the first sentence starts at once"
            ),
            Err(error) => println!("  warm       {stt} stays cold: {error}"),
        }
        match self.clean_text("Parlo", None) {
            Ok(_) => println!("  warm       the cleanup stage is ready"),
            Err(error) => println!("  warm       the cleanup stage stays cold: {error}"),
        }
    }

    pub(super) fn lifecycle_snapshot(&self) -> models::Lifecycle {
        let manage = crate::config::load().manage_memory;
        if self.is_working() {
            let cached = self.known.lock().map(|known| known.clone()).unwrap_or(None);
            return models::Lifecycle {
                manage,
                units: cached.unwrap_or_default(),
            };
        }
        let units = self.pipeline().lifecycle().unwrap_or_default();
        if let Ok(mut known) = self.known.lock() {
            *known = Some(units.clone());
        }
        models::Lifecycle { manage, units }
    }

    pub(super) fn load_unit(&self, unit: &str) -> Result<(), String> {
        self.pipeline()
            .load_unit(unit)
            .map_err(|error| error.to_string())
    }

    pub(super) fn unload_unit(&self, unit: &str) -> Result<(), String> {
        self.pipeline()
            .unload_unit(unit)
            .map_err(|error| error.to_string())
    }

    pub(super) fn chosen(&self) -> RwLockReadGuard<'_, Chosen> {
        self.settings
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(super) fn choose(&self) -> RwLockWriteGuard<'_, Chosen> {
        self.settings
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(super) fn open(
        engine: String,
        stt: String,
        from: String,
        to: String,
    ) -> Result<Self, String> {
        let pipeline = build(&from, &to, &engine, &stt)?;
        let stored = crate::config::load();
        Ok(Self {
            pipeline: Mutex::new(pipeline),
            settings: RwLock::new(Chosen {
                engine,
                stt,
                from,
                to,
                write: stored.write,
                shortcut: stored.shortcut,
                hands_free: stored.hands_free,
                captions: stored.captions,
                command: stored.command,
                copy_shortcut: stored.copy_shortcut,
                undo: stored.undo,
            }),
            target: Mutex::new(None),
            busy: AtomicBool::new(false),
            known: Mutex::new(None),
        })
    }

    pub(super) fn switch(
        &self,
        engine: String,
        stt: String,
        from: String,
        to: String,
        write: String,
    ) -> Result<(), String> {
        crate::config::check_pair(&from, &to)?;
        crate::config::check_stt(&stt)?;
        let mut pipeline = self.pipeline();
        let chosen = self.chosen();
        let same_backend = chosen.engine == engine && chosen.stt == stt;
        let same_pair = chosen.from == from && chosen.to == to;
        let (shortcut, hands_free, captions, command, copy_shortcut, undo) = (
            chosen.shortcut.clone(),
            chosen.hands_free.clone(),
            chosen.captions.clone(),
            chosen.command.clone(),
            chosen.copy_shortcut.clone(),
            chosen.undo.clone(),
        );
        let mut kept = same_backend;
        if same_backend && !same_pair {
            match pipeline.set_languages(&from, &to) {
                Ok(()) => {
                    std::env::set_var("PARLO_CASCADE_SOURCE", &from);
                    std::env::set_var("PARLO_CASCADE_TARGET", &to);
                }
                Err(error) => {
                    eprintln!(
                        "changing the language pair in place failed, restarting the engine: {error}"
                    );
                    kept = false;
                }
            }
        }
        drop(chosen);
        if !kept {
            *pipeline = build(&from, &to, &engine, &stt)?;
        }
        {
            let mut chosen = self.choose();
            *chosen = Chosen {
                engine,
                stt,
                from,
                to,
                write,
                shortcut,
                hands_free,
                captions,
                command,
                copy_shortcut,
                undo,
            };
        }
        drop(pipeline);
        if !kept {
            if let Ok(mut target) = self.target.lock() {
                *target = None;
            }
        }
        Ok(())
    }

    pub(super) fn capture_target(&self) -> Result<(), String> {
        let mut stored = self
            .target
            .lock()
            .map_err(|_| "the target field is unavailable".to_string())?;
        *stored = None;
        let (window, app) = insert::foreground()?;
        *stored = Some(Target { window, app });
        Ok(())
    }

    pub(super) fn target_app(&self) -> Option<String> {
        self.target
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(|target| target.app.clone()))
    }

    pub(super) fn captured_window(&self) -> Option<usize> {
        self.target
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(|target| target.window))
    }

    fn target_window(&self) -> Result<usize, String> {
        match self.target.lock() {
            Ok(target) => match target.as_ref() {
                Some(target) => Ok(target.window),
                None => Err(
                    "Parlo has no window to write in. Copy the text and paste it yourself."
                        .to_string(),
                ),
            },
            Err(_) => Err(
                "Parlo cannot read the window it captured. Copy the text and paste it yourself."
                    .to_string(),
            ),
        }
    }

    pub(super) fn insert_text(
        &self,
        text: &str,
        replace: &str,
        latency_ms: u64,
    ) -> Result<(), String> {
        let window = self.target_window()?;
        if replace.is_empty() {
            let field = insert::field();
            insert::replace_span(&*field, window, replace, text)?;
        } else {
            insert::replace_recorded(window, replace, text)?;
        }
        if latency_ms > 0 {
            println!("the dictation landed in {latency_ms} ms after the key was released");
            if let Err(error) = crate::history::stamp_latency(latency_ms) {
                println!("could not keep the dictation timing: {error}");
            }
        }
        Ok(())
    }

    pub(super) fn live_text(&self, text: &str, replace: &str) -> Result<(), String> {
        let window = self.target_window()?;
        if replace.is_empty() {
            let field = insert::field();
            insert::replace_span(&*field, window, replace, text)
        } else {
            insert::replace_recorded(window, replace, text)
        }
    }

    pub(super) fn clean_text(&self, text: &str, clean: Option<&str>) -> Result<String, String> {
        let _working = self.working();
        self.pipeline()
            .clean_text(text, clean)
            .map_err(|error| error.to_string())
    }

    pub(super) fn dictate(&self, pcm16: &[u8]) -> Result<String, String> {
        if let Ok(text) = std::env::var("PARLO_SELFTEST_TRANSCRIPT") {
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
        let _working = self.working();
        self.pipeline()
            .dictate(pcm16)
            .map_err(|error| error.to_string())
    }

    pub(super) fn embed(&self, pcm16: &[u8]) -> Result<Vec<f32>, String> {
        let _working = self.working();
        self.pipeline()
            .embed(pcm16)
            .map_err(|error| error.to_string())
    }

    pub(super) fn translate_audio(&self, pcm16: &[u8]) -> Result<String, String> {
        let _working = self.working();
        let (_, translation) = self
            .pipeline()
            .write(pcm16, None, None)
            .map_err(|error| error.to_string())?;
        let translation = translation.trim().to_string();
        if translation.is_empty() {
            return Err("no translation model is installed for this language pair".to_string());
        }
        Ok(translation)
    }

    pub(super) fn speak(&self, text: &str) -> Result<(Vec<u8>, u32), String> {
        let language = self.chosen().to.clone();
        self.speak_in(text, &language)
    }

    pub(super) fn speak_in(&self, text: &str, language: &str) -> Result<(Vec<u8>, u32), String> {
        let _working = self.working();
        let chosen = self.chosen();
        let cascade = chosen.engine == "cascade";
        let voice = crate::registry::voice_file(language);
        if cascade {
            let (name, file) =
                voice.ok_or_else(|| format!("Parlo has no voice for {language} yet"))?;
            if !crate::models::models_root().join(file).is_file() {
                return Err(format!(
                    "the {name} is not installed; download it in Models before reading aloud"
                ));
            }
        }
        let installed = voice.map(|(_, file)| crate::models::models_root().join(file));
        let shared = chosen.to == language
            && (!cascade || crate::registry::ENGINE_VOICES.contains(&language));
        if shared {
            drop(chosen);
            return self
                .pipeline()
                .speak_in(text, Some(language))
                .map_err(|error| error.to_string());
        }
        let engine = chosen.engine.clone();
        let stt = chosen.stt.clone();
        let source_language = if chosen.from == language {
            chosen.to.clone()
        } else {
            chosen.from.clone()
        };
        drop(chosen);
        let source = std::env::var("PARLO_CASCADE_SOURCE").ok();
        let target = std::env::var("PARLO_CASCADE_TARGET").ok();
        let previous = std::env::var("PARLO_CASCADE_VOICE").ok();
        if let (true, Some(path)) = (cascade, installed.as_ref()) {
            std::env::set_var("PARLO_CASCADE_VOICE", path);
        }
        let started = build(&source_language, language, &engine, &stt);
        for (key, value) in [
            ("PARLO_CASCADE_SOURCE", source),
            ("PARLO_CASCADE_TARGET", target),
            ("PARLO_CASCADE_VOICE", previous),
        ] {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
        let mut pipeline = started?;
        pipeline
            .speak_in(text, Some(language))
            .map_err(|error| error.to_string())
    }

    pub(super) fn speak_with_voice(
        &self,
        file: &std::path::Path,
        text: &str,
    ) -> Result<(Vec<u8>, u32), String> {
        let _working = self.working();
        let chosen = self.chosen();
        let previous = std::env::var("PARLO_CASCADE_VOICE").ok();
        std::env::set_var("PARLO_CASCADE_VOICE", file);
        let started = build(&chosen.from, &chosen.to, &chosen.engine, &chosen.stt);
        match previous {
            Some(value) => std::env::set_var("PARLO_CASCADE_VOICE", value),
            None => std::env::remove_var("PARLO_CASCADE_VOICE"),
        }
        drop(chosen);
        let mut pipeline = started?;
        pipeline.speak(text).map_err(|error| error.to_string())
    }

    pub(super) fn diarize(
        &self,
        pcm16: &[u8],
        hint: Option<usize>,
    ) -> Result<crate::diarize::Diarization, String> {
        let _working = self.working();
        let samples: Vec<f32> = pcm16
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
            .collect();
        const RATE: usize = 16_000;
        const WINDOW: usize = 24_000;
        const HOP: usize = 12_000;
        const FLOOR: f32 = 0.003;
        if samples.len() < RATE {
            return Err("speaker separation needs at least a second of audio".to_string());
        }
        let mut pipeline = self.pipeline();
        let mut vectors = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        let mut offset = 0;
        while offset + WINDOW <= samples.len() {
            let window = &samples[offset..offset + WINDOW];
            let energy = window.iter().map(|sample| sample * sample).sum::<f32>() / WINDOW as f32;
            if energy.sqrt() >= FLOOR {
                let mut bytes = Vec::with_capacity(WINDOW * 2);
                for sample in window {
                    bytes.extend_from_slice(
                        &((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes(),
                    );
                }
                match pipeline.embed(&bytes) {
                    Ok(vector) if !vector.is_empty() => {
                        vectors.push(vector);
                        starts.push((offset as u64 * 1_000) / RATE as u64);
                        ends.push(((offset + WINDOW) as u64 * 1_000) / RATE as u64);
                    }
                    Ok(_) => {}
                    Err(EngineError::Rejected(_)) => {}
                    Err(error) => return Err(error.to_string()),
                }
            }
            offset += HOP;
        }
        drop(pipeline);
        let result = crate::diarize::group(&vectors, &starts, &ends, hint);
        if result.turns.is_empty() {
            return Err("no voice was found in that recording".to_string());
        }
        Ok(result)
    }

    pub(super) fn write(
        &self,
        pcm16: &[u8],
        target: &str,
        clean: Option<&str>,
    ) -> Result<(String, String), String> {
        if let Ok(text) = std::env::var("PARLO_SELFTEST_TRANSCRIPT") {
            if !text.trim().is_empty() {
                return Ok((text.clone(), text));
            }
        }
        let _working = self.working();
        let target = if target.is_empty() {
            None
        } else {
            Some(target)
        };
        self.pipeline()
            .write(pcm16, target, clean)
            .map_err(|error| error.to_string())
    }

    pub(super) fn stream_window(
        &self,
        samples: &[f32],
        segment: Option<f32>,
    ) -> Result<Vec<parlo_core::engine::StreamPiece>, String> {
        let _working = self.working();
        let mut pipeline = self.pipeline();
        match segment {
            Some(segment) => pipeline.stream_window(samples, segment),
            None => pipeline.stream(samples),
        }
        .map_err(|error| error.to_string())
    }

    pub(super) fn finish_stream(&self) -> Result<Vec<parlo_core::engine::StreamPiece>, String> {
        let _working = self.working();
        self.pipeline()
            .close_stream()
            .map_err(|error| error.to_string())
    }

    pub(super) fn as_json(&self) -> String {
        let chosen = self.chosen();
        let stored = crate::config::load();
        format!(
            "{{\"engine\":\"{}\",\"stt\":\"{}\",\"from\":\"{}\",\"to\":\"{}\",\"write\":\"{}\",\"shortcut\":\"{}\",\"hands_free\":\"{}\",\"captions\":\"{}\",\"command\":\"{}\",\"copy_shortcut\":\"{}\",\"undo\":\"{}\",\"recognisers\":[\"parakeet\",\"whisper\"],\"manage_memory\":{},\"unload\":{},\"name\":\"{}\",\"history_days\":{},\"compute\":\"{}\",\"channel\":\"{}\",\"share\":{},\"share_pin\":\"{}\",\"share_port\":{},\"locale\":\"{}\",\"translation\":{}}}",
            json_string(&chosen.engine),
            json_string(&chosen.stt),
            json_string(&chosen.from),
            json_string(&chosen.to),
            json_string(&chosen.write),
            json_string(&chosen.shortcut),
            json_string(&chosen.hands_free),
            json_string(&chosen.captions),
            json_string(&chosen.command),
            json_string(&chosen.copy_shortcut),
            json_string(&chosen.undo),
            stored.manage_memory,
            stored.unload,
            json_string(&stored.name),
            stored.history_days,
            json_string(&stored.compute),
            json_string(&stored.update_channel),
            stored.share,
            json_string(&stored.share_pin),
            stored.share_port,
            json_string(&stored.locale),
            stored.features.translation
        )
    }
}
