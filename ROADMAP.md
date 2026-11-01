# Roadmap

No dates. A phase ends when its exit criterion is met, not when the code is written. If a criterion fails, the phase does not happen.

The milestones below come from `Parlo_Plano_Mestre_v12_3_FINAL.docx`: section 68 for the sequence, section 69 for the phases and the exit criteria, sections 66.1 to 66.3 for Parlo Live, and sections 15 to 20 for the reported state, the model catalogue and the decisions still open. A milestone is a roadmap label and never a version number: the version advances one step from the last published tag, so the next release is `v0.1.8` whatever milestone it carries. Section 46 of the plan states the same rule, so plan and repository no longer disagree about numbering.

## The committed scope

The plan narrows what is promised to what one person can carry: **dictating and writing in another language, locally, on an ordinary computer** (section 40). Everything after that stays documented as vision and research and must not compete with these gates.

- P0, in order: v0.3 Ouvir, v0.4 Ditar, v0.5 Escrever noutra língua.
- P1, re-evaluated after the v0.5 gate from real use: captions, voice, the remaining surfaces.
- Exploratory, with no calendar: full Meeting with diarisation and search, advanced dubbing, team features.

Parlo Live is not on that list. It is the objective the plan is organised around (sections 66.1 to 66.3): two people in different languages, each hearing the other translated in the speaker's own voice, inside the call application they already use. It arrives in three steps and only the first is close: translated captions (v0.6), then one-way voice through a virtual microphone (v0.9), then two-way conversation (v1.0). Forward engineering decisions follow that order, and no other feature is allowed to crowd it out.

Dubbing is published and is not removed for being early. It is an anticipated capability with its own support, limits and maintenance, and it does not widen the main product now.

## Where the work stands (1 November 2026)

Published: **v0.1.7**, the native window, the voice profile panel and a translation stage that no longer needs the Argos runtime. The licence gate passes with `--strict`, and the one copyleft crate it still lists, `r-efi`, is checked against every release target and fails the build if it ever reaches one.

Translation has a gate of its own: 100 original sentences, 50 per direction, checked by `engines/cascade/scripts/mt_gate.py` for numbers, clocks, negations, names and completeness. Both directions pass every criterion; PT→EN afternoon times kept losing the half of the day until the backend restored the marker from the source clock. Four PT→EN rows still translate a name, so the pair is not declared validated: that is the human reading, not a count.

The plan calls **v0.2 Foundation** complete, and it is: the CPU cascade translates Portuguese and English offline, the local API answers only with a token on loopback, thirty minutes of audio ran at 420.7 ms mean with no drift, and cold start is measured at 18.1 s. The plan keeps those numbers as reported measurements and asks for hardware, method and commit to be attached before they are used in public.

The plan's next milestone, **v0.3 Ouvir**, is close and is what comes next. Four text states ship through the engine, the sidecar protocol and `parlo stream`: `partial`, `stable`, `final` and `corrected`, with stable words confirmed by agreement between the last two hypotheses and a segment that closes after 0.45 s of silence. There is a timestamped harness and a 20 sentence European Portuguese corpus, and captions now reach the screen about **1.6 s** after speech starts at a 1.2 s window, inside the two second target. What is still missing for G0: the recogniser reprocesses a growing buffer rather than streaming natively, no real recorded speaker has been tested, and neither Nemotron 3.5 ASR nor Qwen3-ASR has been tried. Capture is stable and measured and the licence of the recogniser in use is verified.

Four capabilities exist ahead of where the plan schedules them: **dubbing** (plan v1.5), the **interface** in its own window with install and uninstall (plan v0.4 to v0.9), **captions** with translation (plan v0.6, half of it), and a **model panel** with integrity checks (plan v0.7, partial). They are shipped, measured and documented. The plan's order is not rearranged to fit them, and the next work is still the plan's next work.

## Milestones

| Milestone | What a person gains | Gate | Status |
|---|---|---|---|
| v0.2 Foundation | Offline PT↔EN cascade in a prototype | — | Met |
| v0.3 Ouvir | Nothing new; the milestone decides with data | G0 | **Current**, two of three parts met |
| v0.4 Ditar | Dictate PT/EN in any application, floating bar, live partials | G2 | **Started:** `parlo dictate`, the **Dictate** button in the window, and a floating island on **Ctrl+Alt+D** that listens while shown and copies the text when hidden. Missing: insertion into the focused field, a shortcut the user can change, and macOS or Linux |
| v0.5 Escrever noutra língua | Speak Portuguese, write English, German, Spanish or French in the active field | G1 + G2 | Planned, not complete; the MT stage exists, the dictation surfaces do not |
| v0.6 Legendas | Live captions from the microphone and the system audio, translated, exported | G1 | Captions and translation shipped ahead; system audio not |
| v0.7 Modelos | Choose, compare and switch models safely, recommended by hardware | — | Partial: download with integrity checks and a model panel, no catalogue, no rollback |
| v0.8 Voz | Read text in a neutral voice, read translations aloud | — | Partial: a voice in the cascade, no picker, no blind MOS; not an immediate commitment |
| v0.9 Live | Spoken two-way conversation through a virtual microphone, in the speaker's own voice | G1 + G3 | Not started; no virtual microphone, no echo cancellation. The objective the plan is built towards, sequenced after captions and voice |
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

Section 48 of the plan lists a second, differently numbered set of gates, reusing G1 and G2 for dictation and translation and adding two that the table above does not carry:

- **Hardware.** At least one CPU-only path measured on an 8 GB corporate PC and one balanced 16 GB path, plus a modern machine, with what is not supported written down.
- **Release alignment.** `README.md`, `ROADMAP.md`, `PROGRESS.md`, `AGENTS.md`, the changelog, the cover, the index and the change log of the plan updated together, with the tag following the policy in force.

Both are treated as requirements here; the numbering of section 14.4 stays the one releases declare.

## Hardware, as the plan defines it

CPU-only is a first class platform, not a fallback, and a machine is classified by what it can actually do rather than by the marketing name of its processor.

| Profile | Typical machine | Model policy | Expected experience |
|---|---|---|---|
| Leve | Older x86-64, 8 GB RAM or less, no dedicated GPU | Small quantised models, one heavy task at a time, block recognition | Dictation and text translation at higher latency; no promise of Live or real time speech |
| Equilibrado | Recent or mid range x86-64, 16 GB, Intel or AMD iGPU or none | Optimised CPU, quantisation, Parakeet or Whisper as candidates | Dictation usable every day, sequential tasks |
| Alto | 32 GB, multicore CPU, entry or mid range GPU | More options per stage, acceleration only where a backend is validated | Lower latency and more quality |
| Ultra | 64 GB or more and a large dedicated GPU | Larger models and a simultaneous pipeline | Best quality; never the minimum requirement |

Detection reads the operating system, architecture, CPU and SIMD extensions, total and free RAM, GPUs and VRAM, supported backend, free disk, install permissions and power state. It collects no personal identifiers and sends nothing anywhere. Accelerators are a matrix tested per model, version, runtime, operating system and hardware; Vulkan, OpenVINO, DirectML, ROCm, Metal and CUDA are options, and none of them is assumed. Corporate machines need a per-user and a silent managed install, an offline package or internal mirror, configurable model directories, no administrator rights for ordinary work, proxy and firewall behaviour that is tested, and hashes, notices and rollback.

## v0.3 Ouvir, what it takes

- Confirm the reference tool and the exact mode used for comparison, record a demonstration, and measure when partials and finals appear.
- A common audio harness with timestamps, a Portuguese corpus and metrics, so a recognition change shows up as a number.
- Streaming adapters for Parakeet and Whisper, and a first proof of Nemotron 3.5 ASR and Qwen3-ASR on the same hardware, without assuming a CPU path or an NVIDIA card.
- A partial, stable and final protocol, written down and versioned, with a simple diagnostic view of what each stage is doing.
- A side by side run against the reference tool on the same recordings and the same machine.
- A licence check on every candidate, weights and runtime judged separately, before it is wired in.

Out of scope for this milestone: translation changes, voice, system audio, a public installer.

## Open decisions

- **Piper and the licence of the package.** Piper and the espeak-ng data left `parlo-engine`: the synthesiser is now the optional adapter in `engines/tts-piper`, GPL-3.0-or-later, installed by the user and never in an archive, so the packaged engine carries no GPL code. One item stays open: a lawyer's view of the process boundary. The original code is Apache-2.0 since the copyright holders confirmed the whole history, and `LICENSES.md` at the root indexes every licence the project carries.
- **The neutral Portuguese voice.** The plan drops Kokoro as a PT-PT voice, because the official voice list covers Brazilian Portuguese, and treats the Chatterbox pt-PT fine-tune as an experimental candidate rather than a default. Without a validated PT-PT voice, the product offers text and captions instead of a synthetic voice presented as natural in Portugal.
- **Translation quality by pair.** Pairs are approved one at a time and **PT→DE is the central gate of v0.5**: names, dates, amounts, negations, register and business phrasing are checked by competent speakers, not only by an automatic score. Pivoting through English is allowed but never hidden.
- **Where dubbing belongs.** It works and is measured, and the plan places it at v1.5. Nothing is being removed; the question is only how it is presented until then.

## Definition of done, for any release

Installer tested on a clean machine and as an upgrade, with rollback proven; the corpus benchmark updated and published with hardware and versions; licence matrix reviewed for everything the release contains, with third party notices updated; local network audit showing no external connection during use; changelog, known limitations and the state of each language pair; feedback from at least five external users before the next one is announced.

## Deliberately out of scope

- Accounts, login, cloud sync.
- Bots that join your calls.
- Monetisation inside the open source project.
- Cloning a voice from someone else's recording.
- Requiring an NVIDIA GPU for any core feature.
