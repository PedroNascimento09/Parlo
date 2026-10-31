# Roadmap

Every phase has an exit criterion. A phase is done when the criterion is met, not when the code is written.

## v0.1: prototype (current)

Streaming pipeline, latency budget, engine contract, CLI, sidecar protocol.

**Exit criterion:** `cargo test` green on Linux, macOS, and Windows; `parlo doctor`, `demo`, and `translate` work without a model.

## v0.2: real speech engine

Wire Hibiki-Zero into the sidecar, stream frames to it, and stream translated audio back. Publish the first real latency and quality numbers.

**Exit criterion:** Portuguese to English call between two Parlo instances completes with recognisable voice and median latency under 4 seconds on high profile hardware.

## v0.3: virtual microphone

Linux PipeWire null sink shipped and automated; macOS and Windows documented guided setup.

**Exit criterion:** a real Zoom or Meet call works end to end on Linux without manual audio configuration after install.

## v0.4: desktop app

Tauri shell, tray icon, first run wizard, push to translate hotkey, captions overlay.

**Exit criterion:** ten testers complete install and first call without help.

## v0.5: measurable quality

Fixed test set, BLEU and COMET, voice similarity, resource usage published in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

**Exit criterion:** numbers published with hardware and test set named, reproducible by a second person.

## v1.0: public release

Signed installers for macOS, Windows, and Linux; automatic updates; interface in eight languages.

**Exit criterion:** no open critical bugs, install success above 90 percent among beta testers, privacy claims verified by an outside reader.

## v1.5: both directions

System audio capture, cascade engine for any pair, video dubbing mode, custom glossary.

**Exit criterion:** English to Portuguese works, and you can watch a talk in another language with translated audio.

## v2.0: ecosystem

Mobile app, engine plugin system, local API, in-person meeting mode, local transcripts.

**Exit criterion:** a third-party engine plugin ships without changes to the core.

## Deliberately out of scope

- Accounts, login, and cloud sync.
- Bots that join your calls.
- Paid tiers inside the open source project.
- Cloning a voice from a recording of someone else.

## How to influence this

Open an issue describing the problem you have, not the feature you want. Roadmap order follows real blockers reported by users.
