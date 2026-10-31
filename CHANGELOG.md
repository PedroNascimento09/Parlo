# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org).

## [Unreleased]

### Added

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
