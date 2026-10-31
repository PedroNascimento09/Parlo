# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org).

## [Unreleased]

### Changed

- The cascade uses Parakeet TDT 0.6B v3 through sherpa-onnx for recognition: 0.113 real time factor on 4 threads with a 2.5 second window, against 0.309 for Whisper `base`, with fewer name errors. `PARLO_CASCADE_STT=whisper` keeps the old recogniser, and `PARLO_CASCADE_MODEL` applies only to it.
- The setup script downloads Parakeet by default, and caches Whisper only when asked with `--stt whisper`.
- A window sweep with Parakeet confirms the 2.5 second default: shorter windows save about half a second and lose words, so the next latency gain has to come from a sentence boundary, not a smaller cut.
- Models warm up at startup, moving the 12 to 21 second load time away from the first sentence.
- A glossary file corrects names the recogniser mishears, starting with `Parlo`.

- Piper is GPL-3.0-or-later, not MIT. It stays a separate, user-installed program; the registry and documentation now say so.
- The cascade segment window defaults to 2.5 seconds and is configurable with `PARLO_CASCADE_SEGMENT`; at 1.2 seconds the real time factor was 2.66.
- Speech recognition runs without beam search and without a second voice activity filter, since the pipeline already gates silence.

- The default engine is now the universal CPU cascade with permissive licenses; Hibiki-Zero becomes an optional premium engine because its weights are CC BY-NC-SA 4.0.

- The recorder peak normalises every clip towards -1 dBFS with a +24 dB ceiling instead of writing the raw signal, after a laptop Realtek microphone measured between -24.7 and -32.2 dBFS peak.
- The voice gate default moved from -45 to -55 dBFS RMS, because that microphone's speech sits between -39.7 and -46.2 dBFS in 100 millisecond windows against a room noise floor of -63.9 to -67.0 dBFS.
- Live listening and the local API apply automatic gain inside the pipeline after the voice gate, up to +24 dB, so a quiet microphone still reaches the cascade engine.

### Added

- CPU cascade engine: faster-whisper, Argos, and Piper translate Portuguese and English in both directions, offline, at about 2.6 seconds per 1.2 second segment on a 12 core CPU. The output voice follows the target language.
- `parlo translate --engine cascade` streams audio through the cascade sidecar.
- `parlo record` captures the microphone to a WAV for benchmarks, and `scripts/compare.py` runs two recognisers over the same recordings.
- `scripts/stt_bench.py` measures a recogniser alone, with a fixed thread count and window.
- `scripts/memory.py` measures peak memory of a real run: 1302 MB with Parakeet, including the 640 MB model.
- `parlo listen` captures the microphone continuously, translates it through the cascade, and reports the time until the first translated audio.
- `parlo serve` exposes an authenticated local HTTP API on `127.0.0.1` with `/health` and `/translate` for 16 kHz mono WAV audio.
- `scripts/soak.py` loops audio through the engine and reports latency drift: two minutes and 86 segments showed no drift and no errors.
- `parlo engines` lists available engines with licenses, hardware, and voice levels.
- `parlo profile` recommends a hardware profile and voice level from the detected CPU.
- `parlo mic setup`, `parlo mic status`, and `parlo mic remove` manage a PipeWire null sink on Linux.
- `parlo translate --engine sidecar` drives the Python sidecar over the JSON protocol instead of the echo engine.
- Engines report their target languages as owned strings, so adapters are no longer limited to static language lists.

### Fixed

- The live test produced no translated audio at all from 20 seconds of speech on a quiet microphone, because so few frames cleared the gate that the cascade never completed a 2.5 second segment.

### Planned

- Hibiki-Zero inference behind the engine boundary.
- Measured latency and quality benchmarks.
- Linux virtual microphone automation.

## [0.1.4] - 2026-10-31

### Added

- A models panel in the app lists the recogniser and both voices with their size on disk or `Missing`, and `Download models` starts the fetch from the page and refreshes itself until the translator is ready.
- `GET /models` and `POST /models` expose the same state on the local API.

### Changed

- The first start after a boot is published as 18.1 s on Windows 11, from process start to a translated file, with the sidecar and every model included.

## [0.1.3] - 2026-10-31

### Added

- The interface shows the translated sentence as selectable text, returned by the local endpoint in a base64 header next to the audio.
- The update banner installs the release on the spot: a token bearing request to the local server stages the new binary and lets the page explain that the connection closing is the update.

### Changed

- The screen is redesigned around one record card that holds the input level and pulses while listening, a result card with the sentence and the playback, and a three fact status group.
- Updates follow tagged releases only. The `nightly` channel is removed, so nobody is asked to install a commit.
- A thirty minute soak closed the audio half of the v0.2 criterion: 652 cycles, 1095 segments, 420.7 ms mean, p95 485 ms, zero errors and no drift.

## [0.1.2] - 2026-10-31

### Added

- `parlo update` reads the release feed, downloads the build for the platform, and replaces the running binary about two seconds after it exits, keeping the old one beside it as `parlo.previous`; `--check` reports the two versions and changes nothing.

### Changed

- Release titles are the version alone, `vX.Y.Z`, with no codename after it.

## [0.1.1] - 2026-10-31

### Added

- `parlo app` serves one page from the same binary and opens it in the browser with the session token, so the translator needs no command in front of you: hold the control to speak, hear the translated voice, watch the input level, and see the engine and version.
- The page offers a newer release when the GitHub feed has one, with the notes linked, and never installs anything without a click.
- Every push to `main` publishes the `nightly` prerelease for the three build targets.

### Changed

- The roadmap gains a `v0.4 App` phase and the later phases move up one number, since an interface comes before dubbing a video and a native shell.

## [0.1.0] - 2026-10-31

First tagged release of the command line prototype. Everything under `[Unreleased]` above describes this same code, and the release notes live in `docs/releases/v0.1.0.md`.

### Added

- Streaming pipeline with 20 ms frames, voice gate, resampling from 48 kHz to 16 kHz, and an engine boundary.
- `Engine` trait plus an echo engine for testing without a model.
- Latency budget with a 3 second target and a 5 second ceiling.
- Bounded ring buffer that keeps the newest audio and reports evictions.
- PCM16 WAV reader and writer with header validation.
- `parlo` CLI: `doctor`, `demo`, and `translate`.
- Reference Hibiki sidecar with a JSON lines protocol, honest unavailability errors, and tests.
- Documentation for architecture, engines, languages, privacy, virtual microphones, benchmarks, and troubleshooting.
- CI for Linux, macOS, and Windows, release builds on tags, CodeQL, and dependency updates.

[Unreleased]: https://github.com/PedroNascimento09/parlo/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/PedroNascimento09/parlo/releases/tag/v0.1.0
