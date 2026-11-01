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
| `cascade` | Voice, captions | Any modern CPU, GPU optional | Apache-2.0, optional GPL voice | Default, working for Portuguese and English |
| `echo` | Development only | Any | MIT | Tests and demos |
| `hibiki` | Voice preserved | NVIDIA 8 to 12 GB VRAM | CC BY-NC-SA 4.0 | Optional premium |
| `cloud` | Voice | Any | Vendor terms | Opt-in, off by default |

The cascade adapter lives in [../engines/cascade](../engines/cascade) and runs end to end today in both directions: Parakeet TDT 0.6B v3 int8 through sherpa-onnx, Argos, and a Piper neutral voice in the target language. The recogniser is CC-BY-4.0, the runtime Apache-2.0. Measured at 0.113 real time factor on 4 threads with a 2.5 second window; Whisper base remains available with `PARLO_CASCADE_STT=whisper` at 0.309. The first start after a boot costs about 18 s here, sidecar and every model included, while a warmed process turns a segment around in 420.7 ms.

## Recogniser models

Parakeet is the default because it measured both faster and more accurate on the clips this project tests with. Whisper stays available as a fallback for machines that cannot spare the 640 MB.

| Setting | Recogniser | Download |
|---|---|---|
| `PARLO_CASCADE_STT=parakeet` (default) | Parakeet TDT 0.6B v3 int8 via sherpa-onnx | About 640 MB, into `engines/cascade/models/parakeet/` |
| `PARLO_CASCADE_STT=whisper` with `PARLO_CASCADE_MODEL=tiny`, `base`, `small`, `medium`, or `large-v3` | faster-whisper on CPU with int8 | The size the name implies, in the Hugging Face cache |

Both run on 4 threads unless `PARLO_CASCADE_THREADS` says otherwise.

An unknown `PARLO_CASCADE_MODEL` fails at startup with an error that lists the supported sizes, instead of handing the name to faster-whisper and downloading something unexpected. The cascade server refuses to start rather than run a model nobody asked for.

Fetch what you need before the first run, because the engine does not download models by itself:

```bash
python engines/cascade/scripts/setup_models.py --stt both --whisper small
python engines/cascade/scripts/setup_models.py --stt whisper --whisper medium
```

The first command installs Parakeet and caches Whisper `small`, so both can be compared; the second caches only `medium`. Voices and the Argos packages are always installed.

Two measurements that must not be read against each other, because they count different work:

| What was measured | Conditions | Result |
|---|---|---|
| Whole cascade, recognition plus translation plus synthesis | Ten 6 second Portuguese clips from a laptop Realtek microphone, Windows 11, 4 threads | Parakeet mean RTF 0.45, Whisper base mean RTF 1.27 |
| Recognition only | 4.95 seconds of synthetic Portuguese speech, 4 threads, 2.5 second window | Parakeet v3 0.113, Whisper base 0.309 |

The second row is why Parakeet is the default: it is faster on the recogniser and it keeps more words. The first row is what a user feels on ordinary hardware, where translation and synthesis are part of the wait.

Two policies are enforced by tests:

- The default engine requires no GPU and contains no non-commercial model.
- A non-commercial or GPU-bound engine is never the default and is never bundled in the release archive.

Speech synthesis is the one optional piece with a different licence. Piper is GPL-3.0-or-later and lives in [`engines/tts-piper`](../engines/tts-piper), outside the engine program: it is not in [`parlo-engine.spec`](../engines/cascade/parlo-engine.spec), not in `requirements.txt` and not in any release archive, and the user installs it with its own command. The engine talks to it over JSON lines on a pipe and reports synthesis as unavailable when the adapter is absent. A process boundary is not by itself a legal conclusion, which is why [docs/LICENSES.md](LICENSES.md) states the arrangement and no more than that.

The consequence is the reason the project wants to replace Piper with a permissive synthesiser (Kokoro, Apache-2.0, to verify): that would let the whole distribution stay MIT. It is a decision about measured quality, not a date. The component by component table is in [LICENSES.md](LICENSES.md), and `parlo engines` prints the same policy from the registry.

Hibiki-Zero's weights are **non-commercial**. It can be used at home and in research, not in paid work, and the app must say so before downloading it. The universal cascade exists so nobody is forced into that licence to get a working product.

## Captions mode

Captions are the same cascade with the last stage removed: the recogniser and the translation run, the synthesiser does not, and the text goes to the screen instead of a voice to the speaker. That is voice level 0 in the ladder below, and it is what an older four core laptop can afford.

```bash
parlo captions
parlo captions --from en --to pt --window 1.0
```

The command listens to the microphone, prints one translated line per segment as `[  12.4s] text`, and ends with the number of captions and the time of the first one. It never plays or writes audio. Flags: `--seconds` for how long to listen, `--window` for the segment window in seconds, `--from` and `--to`, `--stt` and `--device`, all the same values as the other commands.

Underneath, the engine runs with `PARLO_CASCADE_CAPTIONS=1`, which skips synthesis, and the segment window defaults to 1.2 s instead of the 2.5 s the voice path uses [in the code](../engines/cascade/parlo_cascade/engine.py). The shorter window is the trade captions make: less waiting, less context for the recogniser.

Measured warm on the reference machine with Parakeet, a 1.2 s window costs 958 to 1266 ms of processing, so a caption lands about 2.2 s after the words begin. The basic profile in [BENCHMARKS.md](BENCHMARKS.md) targets captions in under 2 s, and this does not reach it yet; a shorter window closes the gap and costs words. Publishing the number as it is beats pretending the target is met.

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
