# Engines

An engine turns audio in one language into audio, text, or both in another. Parlo does not care which model does the work, only that the adapter respects the contract and the license policy.

## The Rust contract

```rust
pub trait Engine: Send {
    fn id(&self) -> &str;
    fn target_languages(&self) -> Vec<String>;
    fn latency_ms(&self) -> u32;
    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError>;
}
```

Rules an adapter must follow:

1. Input is 16 kHz mono `f32` in `[-1.0, 1.0]`. Do not change the format.
2. Return `EngineOutput::empty()` rather than silence when there is nothing to say.
3. Report a realistic `latency_ms`. An optimistic number makes the budget lie.
4. Fail with `EngineError::Unavailable` when the model is missing, `Rejected` when the frame is invalid. Never panic, never invent audio.

## Engines and licenses

`parlo engines` prints this table from the registry encoded in `crates/parlo-cli/src/engines.rs`, so the product cannot drift from its own policy.

| Id | Mode | Hardware | License | Role |
|---|---|---|---|---|
| `cascade` | Voice, captions | Any modern CPU, GPU optional | MIT + GPL-3.0 TTS | Default, working for Portuguese and English |
| `echo` | Development only | Any | MIT | Tests and demos |
| `hibiki` | Voice preserved | NVIDIA 8 to 12 GB VRAM | CC BY-NC-SA 4.0 | Optional premium |
| `cloud` | Voice | Any | Vendor terms | Opt-in, off by default |

The cascade adapter lives in [../engines/cascade](../engines/cascade) and runs end to end today in both directions: Parakeet TDT 0.6B v3 int8 through sherpa-onnx, Argos, and a Piper neutral voice in the target language. The recogniser is CC-BY-4.0, the runtime Apache-2.0. Measured at 0.113 real time factor on 4 threads with a 2.5 second window; Whisper base remains available with `PARLO_CASCADE_STT=whisper` at 0.309.

Two policies are enforced by tests:

- The default engine requires no GPU and contains no non-commercial model.
- A non-commercial or GPU-bound engine is never the default and is never bundled in the installer.

One honest exception: the default cascade uses Piper for speech synthesis, and `piper-tts` is GPL-3.0-or-later. Parlo's code stays MIT, Piper runs as a separate user-installed program, and the licence is disclosed here and in `parlo engines`. A one-click installer cannot ship it without making the distribution GPL, so replacing Piper with a permissive synthesiser (Kokoro, Apache-2.0, to verify) is a v0.2 decision.

Hibiki-Zero's weights are **non-commercial**. It can be used at home and in research, not in paid work, and the app must say so before downloading it. The universal cascade exists so nobody is forced into that licence to get a working product.

## Voice fidelity ladder

A voice is a level, not a promise:

| Level | What you get | Typical hardware |
|---|---|---|
| 0 | Captions only, no synthesised voice | Any recent CPU |
| 1 | Good neutral voice, chosen gender and tone | 6+ core CPU |
| 2 | Neutral voice with your timbre approximation | 6 to 8 cores or a modest GPU |
| 3 | Cloned voice from a short sample of you | Mid GPU or a strong CPU |
| 4 | End to end voice transfer in the translation | NVIDIA 8 to 12 GB |

`parlo profile` recommends a level from the detected hardware and says plainly that GPU and memory are not inspected yet.

## Reference sidecar

`engines/hibiki` is the protocol reference. It speaks newline delimited JSON over stdio.

```json
{"op": "hello"}
{"op": "hello", "engine": "hibiki", "available": false, "sample_rate": 16000, "target_languages": ["en"]}

{"op": "translate", "seq": 1, "pcm16": "<base64 little endian pcm>"}
{"op": "translated", "seq": 1, "pcm16": "<base64>", "text": "hello", "latency_ms": 1400}

{"op": "translate", "seq": 2, "pcm16": "<base64>"}
{"op": "error", "seq": 2, "error": "torch is not installed; install the model extra from engines/hibiki"}
```

Base64 costs about 33 percent overhead. That is acceptable for a reference adapter and will move to a binary frame header in v0.2, when throughput matters.

## Using the sidecar from the CLI

```bash
parlo translate in.wav out.wav --engine sidecar
```

The CLI starts `serve.py` in `engines/hibiki`, performs the `hello` handshake, and sends every frame through the protocol.

| Variable | Default | Purpose |
|---|---|---|
| `PARLO_PYTHON` | `python` | Interpreter used to start the sidecar |
| `PARLO_HIBIKI_DIR` | `engines/hibiki` | Directory containing `serve.py` |

If the model is missing, the sidecar answers with an error and the CLI prints the model's own reason instead of hanging or inventing audio.

## Candidate cascade components

All must be checked for license, quality, and speed in v0.2. This is an evaluation list, not a decision.

| Stage | Candidates | Note |
|---|---|---|
| Voice detection | Silero VAD, TEN VAD | Light, CPU friendly |
| Speech recognition | Parakeet v3, chosen; whisper.cpp and sherpa-onnx streaming as alternatives | LocalAgreement streaming for stable text |
| Translation | Opus-MT via CTranslate2, MADLAD-400, small quantised LLMs | Avoid non-commercial weights by default |
| Speech synthesis | Kokoro, Piper, Chatterbox, OpenVoice, Pocket TTS | Cloning on CPU is the hard part |
| Voice conversion | OpenVoice, kNN-VC | Fast TTS, then match the timbre |

## Adding an engine

1. Create `engines/<name>` with a server answering `hello` and `translate`.
2. Map output to 16 kHz mono PCM16 and text.
3. Add a Rust adapter that spawns the sidecar and exchanges frames.
4. Add tests for the failure paths first: missing model, invalid audio, engine crash.
5. Add the engine to the registry with its real license and hardware requirement.
6. Document its languages and measured latency here.
