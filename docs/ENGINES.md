# Engines

An engine turns audio in one language into audio in another. Parlo does not care which model does the work, only that the adapter respects the contract.

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
4. Fail with `EngineError::Unavailable` when the model is missing, and with `Rejected` when the frame is invalid. Never panic and never return invented audio.

## Reference sidecar

`engines/hibiki` is the protocol reference. It communicates over newline delimited JSON on stdio.

```json
{"op": "hello"}
{"op": "hello", "engine": "hibiki", "available": false, "sample_rate": 16000, "target_languages": ["en"]}

{"op": "translate", "seq": 1, "pcm16": "<base64 little endian pcm>"}
{"op": "translated", "seq": 1, "pcm16": "<base64>", "text": "hello", "latency_ms": 1400}

{"op": "translate", "seq": 2, "pcm16": "<base64>"}
{"op": "error", "seq": 2, "error": "torch is not installed; install the model extra from engines/hibiki"}
```

Base64 costs about 33 percent overhead. That is acceptable for a reference adapter and will move to a binary frame header before v1.0, when audio throughput matters.

## Using the sidecar from the CLI

```bash
parlo translate in.wav out.wav --engine sidecar
```

The CLI starts `serve.py` in `engines/hibiki`, performs the `hello` handshake, and sends every frame through the protocol. Two environment variables control it:

| Variable | Default | Purpose |
|---|---|---|
| `PARLO_PYTHON` | `python` | Interpreter used to start the sidecar |
| `PARLO_HIBIKI_DIR` | `engines/hibiki` | Directory containing `serve.py` |

If the model is missing, the sidecar answers with an error and the CLI prints the model's own reason instead of hanging or inventing audio. That is the intended behaviour before weights are wired up.

## Adding an engine

1. Create `engines/<name>` with a small server that answers `hello` and `translate`.
2. Map its output to 16 kHz mono PCM16 and its text to the `text` field.
3. Add a Rust adapter that spawns the sidecar, writes frames, and reads responses.
4. Add tests for the failure paths first: missing model, invalid audio, engine crash.
5. Document its languages, license, and measured latency in this file.

Engines that need a GPU belong in a separate process. Engines that are pure Rust and latency-critical can implement the trait directly.

## Candidate engines

| Engine | Direction | License | Status |
|---|---|---|---|
| [Hibiki-Zero](https://github.com/kyutai-labs/hibiki) | pt/es/fr/de/it → en | Check model weights before redistribution | Adapter scaffolded, inference pending |
| Whisper + NLLB + TTS cascade | any → any | MIT / CC-BY-NC for some NLLB weights | Planned |
| Bring-your-own-key cloud | vendor | Commercial | Optional, off by default |

Model weights are never bundled. The user downloads them from the original source, which keeps the installer small and the licensing clean.
