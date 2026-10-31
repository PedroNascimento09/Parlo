# Roadmap

No dates. A phase ends when its exit criterion is met, not when the code is written. If a criterion fails, the phase does not happen.

## v0.1 Spark (current)

Prove the pipeline on modest hardware. Command line only, echo engine, no models.

**Exit criterion:** pipeline, CLI, engine registry, and microphone commands tested on Linux, macOS, and Windows; licences verified; a demo that makes people ask where to get it.

## v0.2 Foundation

Turn the prototype into something that survives a real call: a local engine service with a stable protocol, cold start measured, a reproducible benchmark, and an authenticated local API bound to `127.0.0.1`.

**Exit criterion:** 30 minutes of continuous audio without failure or latency drift, and a benchmark a second person can reproduce.

## v0.3 Engines

The universal cascade arrives: streaming speech recognition, translation, and text to speech on CPU. Captions mode, hardware profiles, and a switchable engine selector.

**Exit criterion:** two engines run the same test suite; the medium profile works inside its documented latency; switching engines requires no core changes.

## v0.4 Dub

First public preview that needs no driver and no NVIDIA card: `parlo dub video.mp4 --to en`, with subtitles, authored voice level, and a visible synthetic speech notice.

**Exit criterion:** three people outside the project dub a five minute video without help.

## v0.5 Native

Own virtual microphone: PipeWire on Linux, an audio plug-in on macOS, a signed driver on Windows. Tray app, installers built in CI.

**Exit criterion:** a clean machine installs and shows Parlo in Zoom, Meet, and Teams in under five minutes, on two operating systems.

## v0.6 Alpha

First run wizard, model manager, captions overlay, global hotkey, per-app profiles. Closed alpha with 10 to 20 people across languages and hardware.

**Exit criterion:** 80 percent of testers install alone and complete a call.

## v0.8 Beta

Three platforms at parity, eight interface languages, accessibility, automatic updates, published benchmarks.

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
