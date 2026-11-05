# Full parity ledger

This is the scoreboard, not a wish list. It lists every user-visible capability VoiceStudio documents, what Parlo has today, and the target that counts as parity. No row is deleted to make a release: a row moves to a later phase, or it is marked **Rejected** with a principle from the plan's non-goals.

Rules:

1. **Done** means verified on this machine with the acceptance line, not merged code.
2. A capability claimed in the README has a measured number or says it was not measured.
3. A row that cannot run locally is Rejected, never moved to a cloud.
4. The breadth targets are floors, not aspirations: 20 speech engines, 12 recognisers, the 646-language class, 21 interface locales, six platforms.
5. Every row has one owner-phase. A phase that ships without closing its rows is late, not smaller.

## Destination

| Dimension | VoiceStudio (v0.5.7) | Parlo target | Phase |
|---|---|---|---|
| Speech engines | 17 TTS, 11 ASR | 20 TTS, 12 ASR | 1–2 |
| Languages | 646 | 646-language class, licence-gated | 1 |
| Interface locales | 21 | 21 | 5 |
| Platforms | Windows, macOS, Linux, Docker, Android, iOS | Same six | 0–5 |
| Stores | Homebrew, AUR, Snap, DEB, AppImage, Nix, MS Store, Mac App Store, F-Droid | Same list | 5 |
| API | OpenAI-compatible, WS, MCP HTTP+stdio, LAN PIN | Same, plus token by default | 1 |
| Dubbing | Multi-speaker, multi-target, timing modes, QC | Same | 3 |
| Longform | Batch, stories, audiobooks | Same | 4 |
| Licence | AGPL-3.0 | Apache-2.0 | — |
| Privacy | Local, optional analytics | Local, no analytics | — |

## Dictation

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| D1 | Global shortcut, hold and toggle | Yes | Yes | Parity, plus per-program profiles | 0 | Hold inserts, tap stays listening, both on a clean install |
| D2 | Insertion into the focused app | Win, macOS, X11 validated | Windows validated; macOS/X11 written, unverified | All three validated | 1 | One sentence lands in Notepad, TextEdit and xterm |
| D3 | Clipboard snapshot and restore | Yes, with generation lease | Done · 7ac7ac4: copied text and images come back after a copy, and a clipboard holding files or formatted text is never cleared | Parity | 1 | A copied image survives a dictation |
| D4 | Pause and resume while recording | Yes | Done · fc42061: the island counts the take in m:ss, pause freezes the clock and the audio, and resume continues one session | Parity | 1 | Pause freezes the timer, resume keeps one session |
| D5 | Partials, stable, final | Yes | Partial: partials, stable and final work and the WebSocket landed in P3, but the first partial arrives at 849 ms on 20 sentences, not 300 ms | Keep, add WebSocket | 1 | Partial within 300 ms of speech |
| D6 | Cleanup levels and spoken commands | Yes | Done · 50f4b6b: `scratch that`, `risca isso` and `apaga isso` drop the phrase it follows, and cleanup keeps the three levels original, light and intelligent | Keep, extend languages | 1 | "scratch that" removes the phrase, both languages |
| D7 | Undo the last dictation | Yes (session-bound) | Done · 587e105: erase refuses a window that is no longer in front, and an undo older than five minutes is refused | Keep | 0 | Refuses when another window is in front |
| D8 | Style, language and voice per program | Per program style | Partial: a program owns a style and a written language, but the voice is still global because dictation takes one engine voice | Add voice | 2 | Slack reads in the cloned voice, Mail in the neutral one |
| D9 | Snippets and vocabulary | Yes | Yes | Keep | 0 | `{{date}}` fills; a term survives cleanup |
| D10 | Pronunciation dictionary | Respelling, IPA, CMU | No | Parity | 2 | A respelled name is spoken as written |

## Transcription

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| T1 | Recogniser breadth | 11 engines | 2 | 12 engines | 1 | Each adapter has a WER row on the FalAR corpus |
| T2 | Language breadth | 646 | 2 | 646 class, permissive weights only | 1 | Published count equals the measured count; NLLB excluded by licence |
| T3 | Speaker diarisation | pyannote, Sortformer | None | Rust clustering on sherpa embeddings; DER under 15% | 1 | Ten minutes, two speakers, DER under 15% |
| T4 | Word timestamps | Yes | No | Parity | 1 | An SRT from a dictation has word bounds |
| T5 | History, search, export | Yes | Yes | Keep | 0 | Find a line by app and by text |
| T6 | System audio capture | Yes, into captions and dubbing | `record --system` only | Wire into captions and meetings | 1 | A call recording transcribes with no microphone |
| T7 | URL and media ingest | yt-dlp, cookies | Partial: public video pages via yt-dlp, no cookies; private videos refused | Parity, public URLs only | 3 | A public video transcribes; a private address is refused |
| T8 | Speaker count hints | 1–20 | No | Parity | 1 | A hint reaches the engine and is recorded |
| T9 | Transcript correction and QC | Yes | Partial: a corrected line keeps its original in the project and editor | Parity | 3 | A corrected line keeps the original beside it |

## Meetings

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| M1 | Record and import | Yes | Yes | Keep | 0 | A transcript imports and lists |
| M2 | Live transcription | Yes | No | Parity | 1 | Lines appear while the meeting runs |
| M3 | Decisions and action items | LLM summary | Rule-based, cited | Keep as an edge | 0 | Each item links to speaker and minute |
| M4 | Ask with citations | Not documented | Yes | Keep as an edge | 0 | An answer with no match says so |
| M5 | Local summary model | Yes | Provider only | Local model, rule fallback | 4 | The screen says which produced the summary |
| M6 | Two-hour reliability | Claimed | Unproven | 2 hours, no crash, flat memory | 6 | A two-hour run with a memory chart |

## Voice

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| V1 | Clone from 5–75 s | Yes | No | Parity | 2 | A 15-second sample previews in under 4 s |
| V2 | Design a voice from words | Yes | No | Parity | 2 | Same seed and description give the same voice |
| V3 | Consent, revoke, identity lock | Yes | Profile only | Parity | 2 | Revoking deletes the sample and the next render uses the default |
| V4 | TTS engine breadth | 17 | 1 (Piper) | 20 | 2 | Every engine has a licence row and a measured speed |
| V5 | Device selection | CUDA, ROCm, MPS, Vulkan | CPU only | Parity, per engine | 2 | A GPU row exists only where measured |
| V6 | Expressive controls | Emotion, direction, cues | Partial: per-cue pace reaches Piper and is saved; no emotion or pitch | Parity | 3 | A cue changes the render and is saved |
| V7 | Previews and A/B | Yes | No | Parity | 2 | Two voices play the same sentence back to back |
| V8 | Voice gallery | Online community | None | Local packs, import/export, opt-in LAN | 2 | A pack imports; nothing uploads |
| V9 | Persona import/export | Yes | No | Parity | 2 | A voice moves between machines as one file |
| V10 | Per-agent voices | Yes | No | Parity | 1 | An MCP client id resolves to its bound voice |
| V11 | Interface locales | 21 | 1 | 21 | 5 | Zero missing keys per locale |

## Dubbing

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| U1 | File dubbing | Yes | Yes, one speaker | Keep | 0 | A clip dubs with subs |
| U2 | Multi-speaker casting | Yes | Yes | Parity | 3 | Two speakers get two voices, line overrides work |
| U3 | Multiple target languages | Yes | Yes | Parity | 3 | One project renders two languages |
| U4 | SRT/VTT import | Yes | Yes | Parity | 3 | Imported cues replace text and timing |
| U5 | URL ingest | Yes | Yes, direct links | Parity, public only | 3 | A URL is refused when it points at localhost |
| U6 | Timing modes | Concise, Smart Fit, Stretch, Lip sync | Yes, all four; lip sync aligns to the cue, not to lips | Partial | 3 | Each mode has a rendered fixture |
| U7 | QC and retry | Yes | Yes, per line; render stays synchronous and unverified means no audio | Parity | 3 | A failed line retries alone |
| U8 | Export matrix | Video, burned subs, dual, karaoke, stems, WAV/MP3 | Yes, 11 formats; karaoke is line-level and stems are voice/original/mix | Parity | 3 | Every export format opens in a player |
| U9 | Projects and recovery | Yes | Yes, persistent manifest and clean restart; unfinished synthesis is redone | Parity | 3 | A render survives a window restart |
| U10 | Vocal separation | Yes | Partial: center-panned speech is removed; mono or centered stereo falls back to ducking with a note | Parity, stereo center only | 3 | Original speech is removed, music kept |

## Longform and batch

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| L1 | Batch queue | Yes | Done · 669600c | Parity | 4 | Twenty files, two languages, one report |
| L2 | Watch folders | Yes | Done · ae61477 | Parity | 4 | A dropped file is ingested once |
| L3 | Stories with a cast | Yes | Done · 4d2fc3c | Parity | 4 | A three-chapter story renders with per-line voices |
| L4 | Audiobooks, M4B | Yes | Done · 7bff831: M4B chapters and attached cover verified with ffprobe | Parity | 4 | Chapters and cover play in a player |
| L5 | Loudness and metadata | Yes | Done · 33992c8: EBU R128 loudness and input metadata written to the M4B tags | Parity | 4 | The M4B states measured LUFS, true peak and LRA |

## Platform

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| P1 | OpenAI transcription | Yes | Done · 181d5f2: SDK-style multipart audio reaches the dictation pipeline and returns the transcript | Parity | 1 | The OpenAI SDK works against `/v1` |
| P2 | OpenAI speech, translation, models | Yes | Done · a94d130: speech, translation and models tested end to end with transcriptions | Parity | 2 | Four endpoints answer with the documented shapes |
| P3 | WebSocket streaming | Yes, versioned | Done · 1236c51: one socket emits partial, stable and final with a session id | Parity | 1 | Partial, stable and final over one socket |
| P4 | Capabilities document | Yes | Done · 1236c51: engines and request limits published at `/.well-known/parlo` | Parity | 1 | `/.well-known/parlo` lists engines and limits |
| P5 | MCP HTTP and stdio | 8 tools | Done · 2f10f47: 12 tools over `/mcp` and stdio | 12 tools, both transports | 1 | One agent uses each transport |
| P6 | LAN sharing with a PIN | Yes | No | Parity, off by default | 5 | A phone on the LAN transcribes with the PIN |
| P7 | Remote backend | Yes | No | Parity, HTTPS only | 5 | A laptop drives a desktop GPU |
| P8 | Keys and tickets | API key, WS ticket | Token per start | Parity | 5 | A short-lived ticket cannot be replayed |
| P9 | SDK and examples | Python client | No | Parity | 5 | One example per endpoint runs |

## Models

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| X1 | Model catalogue | Yes | 52 entries | Keep, generated from the registry | 0 | A card cannot exist without a licence |
| X2 | Per-engine benchmarks | Yes | Partial | One row per engine and device | 6 | No card claims a number without a row |
| X3 | GPU auto-detect | Yes | No | Parity | 2 | The chosen device is shown and overridable |
| X4 | Download resume and repair | Yes | Partial | Parity | 1 | A killed download resumes at the byte |
| X5 | Model packs | Yes | No | Parity | 5 | A pack installs a working set in one action |
| X6 | Licence inventory in the UI | Yes | Partial | Parity | 1 | Each model shows its licence before download |

## Packaging and reach

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| R1 | Signed installer | Yes | No | Authenticode and notarised macOS | 5 | SmartScreen does not warn |
| R2 | Package managers | 9 channels | None | winget, brew, AUR, Snap, DEB, AppImage, Nix | 5 | One command installs on each |
| R3 | Docker | CPU, CUDA, ROCm | No | Parity | 5 | `docker run` serves the OpenAI endpoint |
| R4 | Android app | Preview, offline | No | Native, offline after download | 5 | A phone transcribes a file in airplane mode |
| R5 | iOS app | Preview, sideload | No | Native, offline | 5 | The same file transcribes on an iPhone |
| R6 | Stores | MS Store, Mac App Store, F-Droid | No | Parity | 5 | A store build installs and updates |
| R7 | Update channels | Stable and preview | Stable only | Parity | 5 | A preview build updates to stable |
| R8 | Website and docs | Yes, 20 languages | Partial | Site, quick start, benchmarks, privacy | 5 | A stranger installs in under five minutes |

## Quality

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| Q1 | Automated tests | Large suite | 311 Rust + full Python suite | 600+, route and flow tests | 6 | Every route has a test |
| Q2 | Published benchmarks | Yes | Yes, one machine | Per engine, per device | 6 | Hardware and commit beside every number |
| Q3 | Licence gate | Yes | Yes | Keep, cover every model | 6 | A copyleft crate fails the build |
| Q4 | Network audit | Documented | Partial | Automated, per release | 6 | A release fails if a request leaves the machine |
| Q5 | Crash-free sessions | Claimed | Unproven | Two hours, no crash | 6 | A recorded two-hour run |
| Q6 | External testers | Community | None | Ten testers before 1.0 | 6 | Their hardware rows are in BENCHMARKS |

## The edges Parlo keeps

These are not parity rows; they are why someone chooses Parlo when both can do the job.

1. **No account, no telemetry, no gallery upload.** VoiceStudio has optional analytics and a community gallery; Parlo has neither, by principle.
2. **Apache-2.0.** VoiceStudio is AGPL-3.0. A company can embed Parlo in a product; that is a real adoption advantage.
3. **Meetings with citations by rule.** Decisions, action items and answers work without an LLM, and each one points at the line, speaker and minute.
4. **PT-PT measured.** The recogniser and translation gate were measured on Portuguese parliamentary speech, not only on English benchmarks.
5. **CPU first.** A laptop with no GPU finishes a first session; GPU rows exist only where measured.
6. **Personalisation.** Style, language and voice per program, plus undo, retention, snippets and spoken commands, all in the same window.

## Timeline

| Phase | Scope | One developer | Three developers |
|---|---|---|---|
| 0 Foundations | Registry, store, jobs, serve split, capabilities | 6 weeks | 3 weeks |
| 1 Transcription and platform | 12 ASR, languages, diarisation, WS, OpenAI, MCP | 12 weeks | 6 weeks |
| 2 Voice | Clone, design, 20 engines, devices, previews | 16 weeks | 8 weeks |
| 3 Dubbing | Multi-speaker, multi-target, QC, export matrix | 16 weeks | 9 weeks |
| 4 Longform and batch | Queue, watch folders, stories, audiobooks | 12 weeks | 6 weeks |
| 5 Reach | Signed builds, stores, mobile, 21 locales, site | 16 weeks | 8 weeks |
| 6 Audit | Every ledger row, benchmarks, testers, 1.0 | 10 weeks | 5 weeks |
| **Total** | | **88 weeks** | **45 weeks** |

The destination is full parity. The dates are honest: this is not a quarter, and pretending otherwise would only produce the lite edition the plan refuses to build.

## Progress

33 of 80 rows are Done (41.3%). Eleven Partial rows counted at half give 48.1%.

## Next

D9 Snippets and vocabulary: prove `{{date}}` fills and a term survives cleanup, then close the ledger row.
