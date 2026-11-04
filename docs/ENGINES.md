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

The cascade adapter lives in [../engines/cascade](../engines/cascade) and runs end to end today in both directions: Parakeet TDT 0.6B v3 int8 through sherpa-onnx, an OPUS-MT package read through CTranslate2, and a Piper neutral voice in the target language. The recogniser is CC-BY-4.0, the runtime Apache-2.0. Measured at 0.113 real time factor on 4 threads with a 2.5 second window; Whisper base remains available with `PARLO_CASCADE_STT=whisper` at 0.309. The first start after a boot costs about 18 s here, sidecar and every model included, while a warmed process turns a segment around in 420.7 ms.

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

The first command installs Parakeet and caches Whisper `small`, so both can be compared; the second caches only `medium`. Voices and the two OPUS-MT packages are installed as well; each package is fetched from the published index, checked before it is unpacked, and kept outside the program.

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

## The Rust engine

`crates/parlo-engine-rs` is the engine written in Rust. It speaks the same line protocol as the Python sidecar and answers `hello`, `dictate`, `write`, `translate`, `stream` and `stream_close`.

Measured on the reference machine with the same models and the same six second recording: dictation returns the same text as the Python engine, decoding takes 647 ms against 652 ms, and the translation returns the same sentence as `mt.py` word for word, through CTranslate2 and sentencepiece called from Rust. The whole binary is 6 MB and carries no interpreter and no numpy.

Live mode confirms words instead of rewriting them. A hypothesis stays provisional until two rounds agree, then the agreed words are announced once as `stable`; when the end of a segment changes what was already announced, it is announced again as `corrected` before the `final` text. The interval between decodes follows the time the last decode took, one and a half times it, so a slower machine decodes less often instead of falling further behind: on the same recording and machine a six and a third second clip is decoded nine times rather than fifteen, the first stable words arrive after one second of audio, the whole run costs 4.7 s against 12.6 s when every chunk was decoded, and the final text matches the dictation word for word.

Read aloud is wired into the Rust engine. Piper voices carry no sample rate or voice name in their ONNX metadata, so the engine appends those fields to a temporary copy before loading it, builds the token table from `phoneme_id_map`, and points sherpa at the `espeak-ng-data` folder from `PARLO_ESPEAK_DATA` or the voice folder. On the reference machine, `speak` returned 2.03 s of 22 050 Hz audio for “Olá, isto é um teste do motor em Rust.” with the European Portuguese voice; that is a smoke result, not a quality claim.

The crate is outside the Cargo workspace because the shared sherpa-onnx runtime carries espeak-ng for speech, which is GPL-3.0-or-later. That is why the Rust engine is optional: `PARLO_ENGINE=rust` tries it first and falls back to the Python engine, `PARLO_ENGINE=python` skips it, and with no variable the Python engine stays the default. The Windows release carries `parlo-engine-rs` beside the Python engine, so the same installer offers both.

It builds for the MSVC target with the runtime aligned to static:

```
cargo build --manifest-path crates/parlo-engine-rs/Cargo.toml --release --target x86_64-pc-windows-msvc
```

with `RUSTFLAGS="-C target-feature=+crt-static"`, `CC` and `CXX` unset so cc-rs picks the Microsoft compiler for the target and the system compiler for build scripts, and the Microsoft linker named through `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER`. The crate keeps `ct2rs` without its default features, plus `ruy` for the int8 kernels, so the tokenizer dependency that refuses to build is left out. The build needs CMake and the MSVC build tools; GitHub's Windows runner has both.

## Changing languages without a restart

A new language pair used to cost a restart: the app killed the engine process, started another one, and loaded the recogniser again before it could hear the first word. On the reference machine that is the first start in the table at the top of this page, paid again on every swap.

The engine process now stays, and `pair` changes the pair inside it:

```json
{"op": "pair", "seq": 7, "from": "pt", "to": "de"}
{"op": "switched", "seq": 7, "target_languages": ["de"]}
```

The recogniser stays loaded, because it is the largest model in the cascade and it serves every pair the same way. The translator and the voice of the old target are released, the audio buffer and the caption session are dropped, and the next `write` or `translate` loads what the new pair needs. The Rust engine keeps the last two translators and the last two voices, so a pair used a moment ago is still warm when someone switches back.

In the app, `POST /settings` compares the requested engine and recogniser with the running ones. When both match, it asks the engine for the new pair and leaves the process alone; when either differs, or the swap is refused, the session rebuilds the pipeline exactly as before. Changing only the language dictation writes in no longer restarts anything, because nothing in the engine depends on it.

## Streaming, the decision

The 0.3 milestone asked whether the windowed pipeline stays or a native streaming recogniser replaces it. On ten recorded European Portuguese segments (`docs/BENCHMARKS.md`), Parakeet keeps a real time factor of 1.023 with flat drift (x0.84 decode, x0.80 real time factor), and local agreement confirms 84.4% of a segment's words before it closes. **The pipeline stays for v0.3**, because it meets the latency target on the reference machine and neither replacement is shippable yet:

- `nvidia/nemotron-3.5-asr-streaming-0.6b` streams natively, lists pt-PT in its transcription-ready tier, and runs on CPU through NVIDIA's NeMo-Speech.cpp in GGUF form. Two blockers: its licence, OpenMDW-1.1, is not approved in `tools/license-policy.json`, and the weights do not decode through the transformers build in this repository. `Nemotron3_5AsrForRNNT` loads, but it has no token generation path this project can drive, and an RNNT decode needs the vendor runtime. Its published Portuguese number is pt-BR, not pt-PT.
- `Qwen/Qwen3-ASR-0.6B` is Apache-2.0 and lists Portuguese without distinguishing Portugal from Brazil. Its streaming path needs vLLM on a GPU, and its CPU path is a community ONNX export rather than an official release. That is not a local-first default.

What would change the decision: an approved licence, a measured CPU runtime on these clips, and a word error rate at or below Parakeet's 0.243 with a real time factor under 1. Until then reprocessing is the shipped answer, and the plan's criterion is met where it can be measured on this machine.
