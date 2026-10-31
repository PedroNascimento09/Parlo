# Roadmap

No dates. A phase ends when its exit criterion is met, not when the code is written. If a criterion fails, the phase does not happen.

## v0.1 Spark (released as v0.1.0)

Prove the pipeline on modest hardware. Command line only, echo engine, no models.

**Exit criterion:** pipeline, CLI, engine registry, and microphone commands tested on Linux, macOS, and Windows; licences verified; a demo that makes people ask where to get it. The pipeline, the CLI, and the microphone commands ran end to end on Windows, while the Linux and macOS paths are covered by unit tests only, so this criterion is met on one platform and outstanding on two.

## v0.2 Foundation (current)

Turn the prototype into something that survives a real call: a local engine service with a stable protocol, cold start measured, a reproducible benchmark, and an authenticated local API bound to `127.0.0.1`.

**Exit criterion:** 30 minutes of continuous audio without failure or latency drift, and a benchmark a second person can reproduce. The thirty minute soak passed on Windows with zero errors and no latency drift, while cold start and an independent reproduction remain open.

## v0.3 Engines

The universal cascade arrives: streaming speech recognition, translation, and text to speech on CPU. Captions mode, hardware profiles, and a switchable engine selector.

**Exit criterion:** two engines run the same test suite; the medium profile works inside its documented latency; switching engines requires no core changes.

## v0.4 Dub (first public preview)

Something shareable that needs neither a virtual microphone nor an NVIDIA card: dubbing audio and video with the person's own voice, offline. File mode, not real time: transcription, translation and the best voice the profile can afford, with `.srt` subtitles, audio and video through ffmpeg, and the new voice mixed under music and effects where possible. Interface is `parlo dub video.mp4 --to en` plus a simple drag and drop window.

**Exit criterion:** dub a five minute video with your own voice in three commands, on a PC without an NVIDIA card, and at least three outsiders use it without help.

Note: the browser interface from `parlo app` shipped ahead of this milestone as a preview. It does not change the order here, and the tray app, installers and virtual microphone stay in v0.5.

## v0.5 Native

Own virtual microphone: a PipeWire node on Linux, an Audio Server Plug-in on macOS and a signed driver on Windows, or a documented third party cable as an interim answer. Clean install and uninstall, the right name and icon in call apps, a minimal tray app to switch on and off and pick languages, code signing and notarisation, and `.dmg`, `.msi`, AppImage and `.deb` built in CI.

**Exit criterion:** a clean machine installs and shows Parlo in Zoom, Meet, and Teams in under five minutes, on two operating systems.

## v0.6 Alpha

A complete experience for people who do not program: model manager with integrity checks and disk usage, adjustable voice fidelity, per app profiles, stability when headphones come and go, a first run wizard, a captions overlay, a latency indicator, a global shortcut and push to translate, an exportable diagnostic mode, and a closed alpha with 10 to 20 people.

**Exit criterion:** at least 80 percent of testers install alone and complete a call, including those on modest hardware.

## v0.8 Beta

Three platforms at parity, eight interface languages, accessibility, update channels with rollback, published benchmarks.

**Exit criterion:** zero critical bugs in the last round, 90 percent install success.

## v0.9 Release candidate

Frozen model versions, checksums, SBOM, licence audit, final security review, launch checklist.

**Exit criterion:** checklist at 100 percent, five outsiders approve the demo.

## v1.0 Launch

Signed release on GitHub, Homebrew, winget, and Flathub. Launch day plan.

**Exit criterion:** 90 percent install success and a visible, fast response to issues.

## v1.1 to v2.x

| Version | Focus |
|---|---|
| v1.1 Hardening | Top 10 issues, more calling apps, better echo handling |
| v1.2 Listen | Hear others in your language, system audio capture without echo |
| v1.3 Anything | 20+ input languages with published quality per language |
| v1.5 Two-way | Two streams, in-person mode, local transcripts |
| v2.0 Everywhere | Plugin system, mobile app, authenticated local API |
| v2.x Languages | Community language programme with native speaker review |

## Deliberately out of scope

- Accounts, login, cloud sync.
- Bots that join your calls.
- Monetisation inside the open source project.
- Cloning a voice from someone else's recording.
- Requiring an NVIDIA GPU for any core feature.
