# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org).

## [Unreleased]


## [0.1.7] - 2026-11-01


### Added

- `parlo-app`, a native window built on Tauri 2, so Parlo no longer opens in a browser. It starts `parlo serve` on a free loopback port with a fresh token, waits for `/health`, and shows the page in its own window with a starting screen while the engine loads and a plain failure screen, naming both log files, after four minutes without an answer.
- Closing the window stops the engine, and `PARLO_APP_GUARD` makes the engine exit when its parent's pipe closes, so a crashed window leaves no process holding the port.
- `parlo app` opens the window when `parlo-app` sits beside `parlo`, and falls back to the browser otherwise.
- The release archive carries `parlo-app` next to `parlo` and `parlo-engine`.
- A licence gate, `tools/licenses.py`, that reads the Rust closure from `cargo metadata`, the Python closure from the pinned requirements and the model catalogue, and fails when a licence is missing, denied, outside the allow list for that scope, or attached to an unpinned requirement.
- `THIRD_PARTY_NOTICES.md` and a CycloneDX SBOM generated from the same inventory, both included in every release archive next to `LICENSE` and `NOTICE`.
- `docs/MODEL_LICENSES.md`, `SUPPORT.md`, `GOVERNANCE.md` and a model catalogue at `catalog/models.json`, so each downloadable artefact carries its own licence instead of inheriting the one of the code.
- Text states for streaming recognition: `partial`, `stable`, `final` and `corrected`, with stable words confirmed by local agreement between the last two hypotheses, a segment that closes after 0.45 s of silence, and the mode reported as `reprocess` rather than native streaming.
- `parlo stream <file.wav>` prints those states as they arrive, over a new `stream` operation on the sidecar protocol.
- `engines/cascade/scripts/stream_bench.py` and a 20 sentence European Portuguese corpus measure the first partial, the first stable words, the final text and the word error rate of each stage.
- `setup_models.py --voice pt_pt` fetches the Piper `pt_PT-tugão-medium` voice.
- `parlo_cascade/mt.py`, a translator that runs an OPUS-MT model through CTranslate2 with its own SentencePiece vocabulary, so a model translates without the Argos runtime; it uses the folder in `PARLO_CASCADE_MT_DIR` when set, otherwise the package installed for the pair, and reports a missing model instead of guessing.
- The model catalogue carries the two OPUS-MT packages as CC BY 4.0 with their authors named, read from the licence line the package itself ships.
- `parlo_cascade/packs.py`, a downloader for those packages: it reads the published index, streams the archive with a 512 MB ceiling, refuses entries that escape the folder, checks the files a model needs, writes the SHA-256 and the index it came from next to the copy, and leaves an existing install untouched.
- **Voice profiles in the app.** Create a profile for your own voice, or for someone
  else's with their permission, and Parlo records the consent locally with the purpose,
  the notice version and the time. Revoke or delete it from the same panel; deleting
  removes the folder and everything in it. Recording the sample needs the voice engine,
  which is not in this build yet, so a profile can be created but not used.

### Changed

- `parlo install` copies both programs and points the Start Menu or desktop entry at the window instead of a console.
- The engine log is written to `parlo-engine.log` in the temporary folder when the window starts it, because a double-clicked app has no console.
- The minimum Rust version is 1.85.
- `scripts/parlo-app.cmd` and `scripts/parlo-app.sh` build both programs and open the window.
- Captions reach the screen about 1.6 s after speech starts at a 1.2 s window, 444 ms of processing over 20 sentences, inside the two second target the earlier 2.2 s run missed; the translation stage is the part that changed.
- A two minute soak of the current program turns a segment around in 699 ms mean, p95 940 ms, 66 segments and no errors, voice synthesis included; the run and its conditions are in `docs/BENCHMARKS.md`.
- The cascade translates through that backend and nothing else: `argostranslate` left the engine, the requirements, the PyInstaller spec and the licence closure, taking the AGPL `minisbd` sentence splitter with it. The SentencePiece word marker is cleaned out of the output.
- `setup_models.py` installs the packages itself instead of calling the Argos tooling, and warms the pair through the same backend the engine uses.
- Piper left the engine program: `parlo-engine` no longer bundles `piper`, `piper_phonemize` or the espeak-ng data, so the packaged engine carries no GPL code. The synthesiser is an optional adapter in `engines/tts-piper`, installed separately by the user and driven over JSON lines on a pipe; without it Parlo translates and recognises as before and says the voice is unavailable.
- `ctranslate2` and `sentencepiece` are declared and pinned in the engine requirements, because the translation backend imports them directly now that they no longer arrive through Argos.
- The licence gate no longer treats `r-efi` as an open issue it can only report: it asks cargo for the dependency graph of each release target and fails when a crate marked not shipped appears in one. `r-efi` is absent from all three.
- Engine requirements are pinned to exact versions, every commit in a pull request must carry a DCO sign-off, and the release workflow refuses to build while the licence gate reports an open issue.
- The original code moved to the Apache License 2.0 once the copyright holders confirmed the whole history; releases up to v0.1.7 keep the MIT text they were published with, in `licenses/MIT-Parlo-0.1.x.txt`.
- `LICENSES.md` at the root indexes every licence the project carries, by section, with the code licence, the model or voice licence, the holder, whether it is required or optional, whether it was confirmed and what it obliges; CI regenerates it and fails when it drifts.
- On the European Portuguese voice the cascade reaches the first partial at 849 ms and the final text at 3.36 s, with 0.231 final word error rate for Parakeet and 0.279 for Whisper base, and 81% of the sentence confirmed before the segment closes. The Brazilian voice reading the same European sentences scored about 0.66 for both, so the corpus now uses the PT-PT voice.

## [0.1.6] - 2026-10-31

### Changed

- The release archive now carries the engine, and models, settings and downloads moved to the per-user data directory: `%APPDATA%\parlo` on Windows, `~/Library/Application Support/parlo` on macOS, `$XDG_DATA_HOME/parlo` elsewhere, overridable with `PARLO_MODELS_DIR` and `PARLO_DATA_DIR`.
- The local API refuses a `Host` header that is not the local address and a write whose `Origin` is not the page, which closes DNS rebinding and cross-site writes against the loopback port.
- The hardware profiles follow the plan again: `ultra`, `high`, `medium` and `basic`, with GPU memory read from `nvidia-smi` when it exists and the high and ultra profiles reserved for a GPU the machine actually has.
- On the same 24 sentence battery, Whisper `small` scored 0.217 clean against 0.215 for Parakeet, and degraded faster under noise (0.309 against 0.197 at 10 dB), so Parakeet stays the default on accuracy as well as speed.
- The cascade uses Parakeet TDT 0.6B v3 through sherpa-onnx for recognition: 0.113 real time factor on 4 threads with a 2.5 second window, against 0.309 for Whisper `base`, with fewer name errors. `PARLO_CASCADE_STT=whisper` keeps the old recogniser, and `PARLO_CASCADE_MODEL` applies only to it.
- The setup script downloads Parakeet by default, and caches Whisper only when asked with `--stt whisper`.
- A window sweep with Parakeet confirms the 2.5 second default: shorter windows save about half a second and lose words, so the next latency gain has to come from a sentence boundary, not a smaller cut.
- Models warm up at startup, moving the 12 to 21 second load time away from the first sentence.
- A glossary file corrects names the recogniser mishears, starting with `Parlo`.
- Piper is GPL-3.0-or-later, not MIT. The packaged engine now contains Piper and the espeak-ng data, so `parlo-engine` is distributed under the GPL-3.0-or-later while the `parlo` command line program stays MIT; `docs/LICENSES.md` states the split.
- The cascade segment window defaults to 2.5 seconds and is configurable with `PARLO_CASCADE_SEGMENT`; at 1.2 seconds the real time factor was 2.66.
- Speech recognition runs without beam search and without a second voice activity filter, since the pipeline already gates silence.
- The default engine is the universal CPU cascade, MIT apart from the GPL synthesiser it bundles in `parlo-engine`; Hibiki-Zero becomes an optional premium engine because its weights are CC BY-NC-SA 4.0.
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
- Offline dubbing: `parlo dub video.mp4 --to en` extracts the audio, recognises each spoken segment, translates it, speaks it in the target voice at its original start time, and writes a video with a replaced audio track plus a `.srt`; `--keep-background` mixes music under the dub at 0.18 and `--json` writes the lines.
- A dub panel in the app: drop a video or audio file on the page, watch elapsed time while the local server works, then download the dubbed file and read the translated lines.
- Captions mode: `parlo captions` translates the microphone to text with no speech synthesis, using a 1.2 s window by default. Measured warm on Parakeet at 958 to 1266 ms of processing per segment, so a caption lands about 2.2 s after the words start.
- Language and recogniser settings inside the page, with `GET /settings` and `POST /settings` on the local API. Applying a change reloads the models in about twenty seconds and is written to the config file.
- `--stt parakeet|whisper` on `config`, `app`, `serve`, `listen` and `captions`, and an `stt` field in the config file.
- `parlo speakers` lists output devices, and `parlo listen --play --speaker N` plays the translated voice to one of them through a bounded queue and rate matched resampling.
- `parlo-engine`: the cascade packaged with PyInstaller into one self-contained executable, preferred by the CLI when it sits beside the binary, so a release user installs no Python. The release workflow builds it per platform and ships it inside the archives.
- `parlo config` reads and writes `config.json` (engine, recogniser, language pair, port, device), and `app`, `serve`, `listen` and `translate` start from it.
- `POST /dub` on the local API, uploads limited to 512 MiB, and `GET /dub/<id>` serving the produced file back only from an allowlist of files the server wrote.
- `docs/THREAT-MODEL.md`, an initial threat model written against the code, with what is protected, who could attack and what is still open.
- A reference battery of 24 Portuguese sentences with seeded noise and automatic word error rate scoring, so a recognition regression shows up as a number without anyone recording anything.

### Fixed

- The live test produced no translated audio at all from 20 seconds of speech on a quiet microphone, because so few frames cleared the gate that the cascade never completed a 2.5 second segment.
- The input stream error was swallowed, so a device that disappeared mid recording ended as a short silent file; capture now ends with `the input device stopped` instead.

### Planned

- Hibiki-Zero inference behind the engine boundary.
- Measured latency and quality benchmarks.
- Linux virtual microphone automation.

## [0.1.5] - 2026-10-31

### Added

- `--from` and `--to` on `app`, `serve`, `listen` and `translate`, validated and passed to the cascade sidecar; English to Portuguese was verified end to end, 6.33 s of input giving 4.21 s of output.
- `scripts/parlo-app.cmd` and `scripts/parlo-app.sh` build once and open the interface, so the app starts without a terminal command.

### Changed

- The app documentation describes both launchers and the one-direction-per-process rule.

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
