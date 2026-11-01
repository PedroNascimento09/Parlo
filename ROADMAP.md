# Roadmap

No dates. A phase ends when its exit criterion is met, not when the code is written. If a criterion fails, the phase does not happen.

The milestones below come from `Parlo_Plano_Mestre_v9.docx`, section 15, which is the authority on product direction. A milestone is a roadmap label and never a version number: the version advances one step from the last published tag, so the next release is `v0.1.8` whatever milestone it carries.

## Where the work stands (1 November 2026)

Published: **v0.1.6**. In the tree and ready to tag: **v0.1.7**, the native window.

The plan calls **v0.2 Foundation** complete, and it is: the CPU cascade translates Portuguese and English offline, the local API answers only with a token on loopback, thirty minutes of audio ran at 420.7 ms mean with no drift, and cold start is measured at 18.1 s.

The plan's next milestone, **v0.3 Ouvir**, is not complete, and it is what comes next. Recognition still cuts the audio into 2.5 second windows and commits each one whole, so there is no partial, stable and final text; there is no common audio harness with timestamps, no Portuguese corpus built for streaming, and neither Nemotron nor Qwen3-ASR has been tried. G0 is met on two of its three parts: capture is stable and measured, the licence of the recogniser in use is verified, the streaming behaviour is not there.

Three capabilities were built ahead of where the plan schedules them: **dubbing** (plan v1.5), the **interface** in its own window with install and uninstall (plan v0.4 to v0.9), and **captions** with translation (plan v0.6). They are shipped, measured and documented. The plan's order is not being rearranged to fit them, and the next work is still the plan's next work.

## Milestones

| Milestone | What a person gains | Gate | Status |
|---|---|---|---|
| v0.2 Foundation | Offline PT↔EN cascade in a prototype | — | Met |
| v0.3 Ouvir | Nothing new; the milestone decides with data | G0 | **Current**, two of three parts met |
| v0.4 Ditar | Dictate PT/EN in any application, floating bar, live partials | G2 | Not started |
| v0.5 Escrever noutra língua | Speak Portuguese, write English, German, Spanish or French in the active field | G1 + G2 | Not started |
| v0.6 Legendas | Live captions from the microphone and the system audio, translated, exported | G1 | Captions and translation shipped ahead; system audio not |
| v0.7 Modelos | Choose, compare and switch models safely, recommended by hardware | — | Partial: download with integrity checks and a model panel, no catalogue |
| v0.8 Voz | Read text in a neutral voice, read translations aloud | — | Partial: Piper voice in the cascade, no voice picker, no blind MOS |
| v0.9 Live | Spoken two-way conversation through a virtual microphone | G1 + G3 | Not started |
| v1.0 Estável | Dictate, Captions, Live and Models stable, PT↔EN validated, signed installers | G4 | Not started |
| v1.1 Meeting Lite | Import a recording, get a transcript with speakers, export it | G4 | Not started |
| v1.2 Meeting Live | Transcribe meetings live, summary, decisions, tasks | G4 | Not started |
| v1.3 Meeting Chat | Ask the meetings, with verifiable citations | G4 | Not started |
| v1.4 Plataforma | SDK, plugins, stable local API, MCP | G4 | Not started |
| v1.5 Dub e Voz própria | Dub files, clone your own voice with consent | G4 | Dubbing shipped ahead; voice cloning not started |
| v1.6 Equipa e áudio pro | Workspaces, retention, advanced denoise and echo cancellation, public benchmark | G4 | Not started |

Windows first, up to v1.0. macOS and Linux arrive after it, first with dictation and captions and only later with Live; on Linux, X11 and Wayland are separate problems.

## Gates

A release declares the gate it has to pass. If the gate fails the release does not ship; features can move, the gate cannot.

| Gate | Minimum to advance | Release |
|---|---|---|
| G0 · Viabilidade | Streaming recognition of real PT-PT speech without accumulating delay; licence verified; stable capture | v0.3 |
| G1 · Pipeline de texto | STT→MT for 30 minutes; p95 latency published; an engine failure loses no text. Extends to STT→MT→TTS | v0.5, v0.9 |
| G2 · Produto | 5 external users install and finish a first session without critical help | v0.4, v0.5 |
| G3 · Beta | 2 hours without a crash; no sustained queue growth; zero critical bugs; local network audit | v0.9 |
| G4 · Release | Signed installers, SBOM, licences, documentation, rollback and a compatibility matrix | v1.0 onward |

## v0.3 Ouvir, what it takes

- A common audio harness with timestamps, a Portuguese corpus and metrics, so a recognition change shows up as a number.
- Streaming adapters for Parakeet and Whisper, and a first proof of Nemotron 3.5 ASR and Qwen3-ASR on the same hardware.
- A partial, stable and final protocol, written down and versioned, with a simple diagnostic view of what each stage is doing.
- A side by side run against the reference tool, measuring when partials and finals appear.
- A licence check on every candidate before it is wired in.

Out of scope for this milestone: translation changes, voice, system audio, a public installer.

## Open decisions

- **The neutral voice.** The plan keeps Piper out of the distributable core while GPL compatibility is unresolved, and makes Kokoro the baseline with Piper as a reference point. The shipped engine bundles Piper and is therefore GPL-3.0-or-later as a whole, which `docs/LICENSES.md` states. Whether to keep that split or move to a permissively licensed voice is undecided.
- **Where dubbing belongs.** It works and is measured, and the plan places it at v1.5. Nothing is being removed; the question is only how it is presented until then.

## Definition of done, for any release

Installer tested on a clean machine and as an upgrade, with rollback proven; the corpus benchmark updated and published with hardware and versions; licence matrix reviewed for everything the release contains, with notices updated; local network audit showing no external connection during use; changelog, known limitations and the state of each language pair; feedback from at least five external users before the next one is announced.

## Deliberately out of scope

- Accounts, login, cloud sync.
- Bots that join your calls.
- Monetisation inside the open source project.
- Cloning a voice from someone else's recording.
- Requiring an NVIDIA GPU for any core feature.
