# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org).

## [Unreleased]

### Changed

- The cascade defaults to the `base` recogniser, which reaches 0.56 real time factor on 4 threads; `tiny` is available for weak hardware and `small` needs 8 or more cores.
- Models warm up at startup, moving the 12 to 21 second load time away from the first sentence.
- A glossary file corrects names the recogniser mishears, starting with `Parlo`.

- Piper is GPL-3.0-or-later, not MIT. It stays a separate, user-installed program; the registry and documentation now say so.
- The cascade segment window defaults to 2.5 seconds and is configurable with `PARLO_CASCADE_SEGMENT`; at 1.2 seconds the real time factor was 2.66.
- Speech recognition runs without beam search and without a second voice activity filter, since the pipeline already gates silence.

- The default engine is now the universal CPU cascade with permissive licenses; Hibiki-Zero becomes an optional premium engine because its weights are CC BY-NC-SA 4.0.

### Added

- CPU cascade engine: faster-whisper, Argos, and Piper translate Portuguese and English in both directions, offline, at about 2.6 seconds per 1.2 second segment on a 12 core CPU. The output voice follows the target language.
- `parlo translate --engine cascade` streams audio through the cascade sidecar.
- `parlo engines` lists available engines with licenses, hardware, and voice levels.
- `parlo profile` recommends a hardware profile and voice level from the detected CPU.
- `parlo mic setup`, `parlo mic status`, and `parlo mic remove` manage a PipeWire null sink on Linux.
- `parlo translate --engine sidecar` drives the Python sidecar over the JSON protocol instead of the echo engine.
- Engines report their target languages as owned strings, so adapters are no longer limited to static language lists.

### Planned

- Hibiki-Zero inference behind the engine boundary.
- Measured latency and quality benchmarks.
- Linux virtual microphone automation.

## [0.1.0] - 2026-10-31

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
