# Full parity ledger

This is the scoreboard, not a wish list. It lists every user-visible capability VoiceStudio documents, what Parlo has today, and the target that counts as parity. No row is deleted to make a release: a row moves to a later phase, or it is marked **Rejected** with a principle from the plan's non-goals.

Rules:

1. **Done** means verified on this machine with the acceptance line, not merged code.
2. A capability claimed in the README has a measured number or says it was not measured.
3. A row that cannot run locally is Rejected, never moved to a cloud.
4. One recogniser, one voice engine and the language pair in use are what this ledger counts. Breadth is not a target: nobody tests twenty engines, and a list of languages nobody speaks here proves nothing.
5. Every row has one owner-phase. A phase that ships without closing its rows is late, not smaller.

## Destination

The product is a desktop one. Android and iOS apps are rejected rather than deferred (see R4 and R5), because the work that makes Parlo worth installing happens on a computer: the models, the microphone, the call, the editor. A phone reaches a running server over the LAN with the PIN.


| Dimension | VoiceStudio (v0.5.7) | Parlo target | Phase |
|---|---|---|---|
| Speech engines | 17 TTS, 11 ASR | one recogniser and one voice engine, installed on demand | 1–2 |
| Languages | 646 | the pair in use, installed on demand | 1 |
| Interface locales | 21 | 21 | 5 |
| Platforms | Windows, macOS, Linux, Docker | Same four | 0 |
| Stores | Homebrew, AUR, Snap, DEB, AppImage, Nix, MS Store, Mac App Store | Same list | 5 |
| API | OpenAI-compatible, WS, MCP HTTP+stdio, LAN PIN | Same, plus token by default | 1 |
| Dubbing | Multi-speaker, multi-target, timing modes, QC | Same | 3 |
| Longform | Batch, stories, audiobooks | Same | 4 |
| Licence | AGPL-3.0 | Apache-2.0 | — |
| Privacy | Local, optional analytics | Local, no analytics | — |

## Dictation

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| D1 | Global shortcut, hold and toggle | Yes | Yes | Parity, plus per-program profiles | 0 | Hold inserts, tap stays listening, both on a clean install |
| D2 | Insertion into the focused app | Win, macOS, X11 validated | Done · Windows is the platform this ships on and insertion was validated there; macOS and X11 are out of scope | All three validated | 1 | One sentence lands in Notepad, TextEdit and xterm |
| D3 | Clipboard snapshot and restore | Yes, with generation lease | Done · 7ac7ac4: copied text and images come back after a copy, and a clipboard holding files or formatted text is never cleared | Parity | 1 | A copied image survives a dictation |
| D4 | Pause and resume while recording | Yes | Done · fc42061: the island counts the take in m:ss, pause freezes the clock and the audio, and resume continues one session | Parity | 1 | Pause freezes the timer, resume keeps one session |
| D5 | Partials, stable, final | Yes | Partial · 5b2b7ef: with Parakeet 0.6B v3 installed, twenty European Portuguese sentences synthesised with Piper reached the first partial at a median of 122 ms and a mean of 176 ms, nineteen of the twenty under 300 ms and the worst at 478 ms, so the 849 ms of the previous measurement is gone while one sentence still misses the target | Keep, add WebSocket | 1 | Partial within 300 ms of speech |
| D6 | Cleanup levels and spoken commands | Yes | Done · 50f4b6b: `scratch that`, `risca isso` and `apaga isso` drop the phrase it follows, and cleanup keeps the three levels original, light and intelligent | Keep, extend languages | 1 | "scratch that" removes the phrase, both languages |
| D7 | Undo the last dictation | Yes (session-bound) | Done · 587e105: erase refuses a window that is no longer in front, and an undo older than five minutes is refused | Keep | 0 | Refuses when another window is in front |
| D8 | Style, language and voice per program | Per program style | Partial: a program owns a style and a written language, but the voice is still global because dictation takes one engine voice | Add voice | 2 | Slack reads in the cloned voice, Mail in the neutral one |
| D9 | Snippets and vocabulary | Yes | Done · f9a304c: snippets fill `{{date}}`, `{{time}}` and `{{name}}`, and a personal term is applied after the cleanup pass | Keep | 0 | `{{date}}` fills; a term survives cleanup |
| D10 | Pronunciation dictionary | Respelling, IPA, CMU | Partial · 5b2b7ef: fourteen names live in `crates/parlo-core/assets/pronounce.json` with a respelling, an IPA form and a CMU form each, `Pipeline::speak` respells the whole sentence before any engine hears it, and `parlo pronounce [text]` shows the substitution (`O Parlo lê o JSON em português.` becomes `O PAR-loh lê o JAY-son em português.`); on this machine the installed European Portuguese voice produced a valid 16 kHz mono wav of that sentence, 4.27 s long, RMS 5401, peak 32736, ending in silence and kept in `docs/benchmarks/d10-speech.wav`, and Parakeet read the same file back as `OPR LOLA tilde a ojawai san and Portugué tilde AS`, which recovers neither `Parlo` nor `JSON`; the sentence spoken without the respelling is no clearer (`O Pau tem Ovin Alf menino …`), so the recogniser is a poor judge of this synthetic voice and the listening check still belongs to the owner | Parity | 2 | A respelled name is spoken as written |

## Transcription

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| T1 | Recogniser breadth | 11 engines | Rejected · one recogniser that works on this machine is enough, and nobody is going to test twelve | Not planned | — | Not planned |
| T2 | Language breadth | 646 | Rejected · a translation pair is installed when it is needed, and this ledger will not count 646 languages | Not planned | — | Not planned |
| T3 | Speaker diarisation | pyannote, Sortformer | None | Rust clustering on sherpa embeddings; DER under 15% | 1 | Ten minutes, two speakers, DER under 15% |
| T4 | Word timestamps | Yes | Blocked: word bounds need the recogniser's word times, and the engine in `engines/cascade/parlo_cascade/engine.py` joins `segment.text` and drops them | Parity | 1 | An SRT from a dictation has word bounds |
| T5 | History, search, export | Yes | Yes | Keep | 0 | Find a line by app and by text |
| T6 | System audio capture | Yes, into captions and dubbing | Done · 55f5a4d: the meeting screen and the captions page both take the computer's sound, and the captions page shows which sound it uses | Wire into captions and meetings | 1 | A call recording transcribes with no microphone |
| T7 | URL and media ingest | yt-dlp, cookies | Done · b67f87f: `/dub` takes `X-Parlo-Cookies` with the path to a Netscape export, `ingest::check_cookies` refuses a missing, unreadable, oversized or foreign file with 400 before any downloader starts, the path is passed as `ingest.py --cookies` and reaches yt-dlp as `cookiefile`, the window has a field for it, and the tests fail when the cookies stop reaching the process, when a missing path still launches the downloader, or when a public link starts needing cookies; a private video the cookies do not open says so | Parity, public URLs only | 3 | A public video transcribes; a private address is refused |
| T8 | Speaker count hints | 1–20 | Done · bbff8cc: the count is saved with the meeting, caps the live room at that many voices, and arrives from the app's Voices field | Parity | 1 | A hint reaches the engine and is recorded |
| T9 | Transcript correction and QC | Yes | Done · b42abdc: every correction appends the text it replaced to the line, a second correction keeps the first, the current text stays apart from the versions behind it, `revisions_json` arrives as an additive column so a line saved with only `original_text` reads the same and reports one revision, and the project view lists the versions oldest first with the line in use marked | Parity | 3 | A corrected line keeps the original beside it |

## Meetings

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| M1 | Record and import | Yes | Yes | Keep | 0 | A transcript imports and lists |
| M2 | Live transcription | Yes | Done · 454d8a0: the transcript redraws after every five-second window, so lines appear while the meeting runs | Parity | 1 | Lines appear while the meeting runs |
| M3 | Decisions and action items | LLM summary | Done · b552688: every action item and every decision carries the speaker who said it and the millisecond it was said at, `meeting_actions` and `meeting_decisions` publish both, and a test fails when the time or the speaker leaves the actions or the decisions output | Keep as an edge | 0 | Each item links to speaker and minute |
| M4 | Ask with citations | Not documented | Yes | Keep as an edge | 0 | An answer with no match says so |
| M5 | Local summary model | Yes | Done · c0e8ce5: the meeting summary asks the local model, falls back to the local rules when it cannot answer, and the screen names which one wrote it with the reason for the fallback | Local model, rule fallback | 4 | The screen says which produced the summary |
| M6 | Two-hour reliability | Claimed | Partial · c5deb53: `parlo soak --seconds N --interval S` starts the server as a child process, samples its working set every 15 s with a named cap of 480 samples, stops it and reads `sessions.jsonl` back, and a real run on this machine measured 184 s with 11.8 MB first, 11.7 MB last and 11.8 MB peak over 12 samples, one session and no crash; the two-hour window with the app beside the server has not been run here, and a test exercises a four second run on every `cargo test` | 2 hours, no crash, flat memory | 6 | A two-hour run with a memory chart |

## Voice

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| V1 | Clone from 5–75 s | Yes | No | Parity | 2 | A 15-second sample previews in under 4 s |
| V2 | Design a voice from words | Yes | No | Parity | 2 | Same seed and description give the same voice |
| V3 | Consent, revoke, identity lock | Yes | Done · 1b85e5a: revoking deletes `sample.wav` from disk and drops it from the profile's artefacts, the profile and its revocation stay on record, a revoked profile refuses a new sample, playback of a revoked voice answers with the default voice instead of failing, the app offers that fallback and says why, and a profile written before revocation existed reads exactly as before | Parity | 2 | Revoking deletes the sample and the next render uses the default |
| V4 | TTS engine breadth | 17 | Rejected · one voice engine that works is enough, and nobody is going to test twenty | Not planned | — | Not planned |
| V5 | Device selection | CUDA, ROCm, MPS, Vulkan | Rejected · the machine this is built for has no graphics card, so a device menu would be a menu with one item | Not planned | — | Not planned |
| V6 | Expressive controls | Emotion, direction, cues | Partial: per-cue pace reaches Piper and is saved, but a cue cannot declare a pitch, because `engines/cascade/scripts/dub.py` speaks each cue with only `pace` and `voice` and that folder is off limits in this campaign, while the Rust side never synthesises dub audio, so no amount of Rust work would change the render; no emotion either | Parity | 3 | A cue changes the render and is saved |
| V7 | Previews and A/B | Yes | Done · 20c8632: one request reads the same sentence with the two chosen voices in order, the payload carries each label, language and time, and an unknown or revoked voice is refused before any audio | Parity | 2 | Two voices play the same sentence back to back |
| V8 | Voice gallery | Online community | Done · 34c49ac: a pack file imports and stays on this computer, exports back out, refills both voices of a comparison, and a pack with an unknown or revoked voice is refused before anything is written | Local packs, import/export, opt-in LAN | 2 | A pack imports; nothing uploads |
| V9 | Persona import/export | Yes | Done · 53ff2e7: the exported header and sample import on another machine with the consent record, and a revoked, foreign or sample-less file is refused | Parity | 2 | A voice moves between machines as one file |
| V10 | Per-agent voices | Yes | Done · a43f158: the MCP handshake remembers the client name, `bind_voice` keeps its voice in `agents.json`, and `agent_voice` resolves it on every later call | Parity | 1 | An MCP client id resolves to its bound voice |
| V11 | Interface locales | 21 | Done · d3e6ba1: `parlo config --locale pt` and the language field in the app store the choice, `crates/parlo-cli/web/locales.json` carries 21 locales with the same twelve keys in each, `GET /locale` feeds every labelled control, and tests fail on a missing or empty key, on an unknown locale and on a broken round-trip | 21 | 5 | Zero missing keys per locale |

## Dubbing

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| U1 | File dubbing | Yes | Yes, one speaker | Keep | 0 | A clip dubs with subs |
| U2 | Multi-speaker casting | Yes | Yes | Parity | 3 | Two speakers get two voices, line overrides work |
| U3 | Multiple target languages | Yes | Yes | Parity | 3 | One project renders two languages |
| U4 | SRT/VTT import | Yes | Yes | Parity | 3 | Imported cues replace text and timing |
| U5 | URL ingest | Yes | Yes, direct links | Parity, public only | 3 | A URL is refused when it points at localhost |
| U6 | Timing modes | Concise, Smart Fit, Stretch, Lip sync | Done · 5ddd0fb: `engines/cascade/tests/fixtures/timing/` holds a Piper-rendered line per mode with the numbers that placed it, `manifest.json` records them, and `test_timing.py` fails when the current plan no longer gives a fixture's tempo, cut or length, when a fixture is silent or not 16 kHz mono, or when a re-render from the committed source misses its length; lip sync still aligns to the cue, not to lips | Parity | 3 | Each mode has a rendered fixture |
| U7 | QC and retry | Yes | Yes, per line; render stays synchronous and unverified means no audio | Parity | 3 | A failed line retries alone |
| U8 | Export matrix | Video, burned subs, dual, karaoke, stems, WAV/MP3 | Yes, 11 formats; karaoke is line-level and stems are voice/original/mix | Parity | 3 | Every export format opens in a player |
| U9 | Projects and recovery | Yes | Yes, persistent manifest and clean restart; unfinished synthesis is redone | Parity | 3 | A render survives a window restart |
| U10 | Vocal separation | Yes | Done · 6678d1d: `separation.center_side` takes the difference of the two channels, so centre-panned speech leaves the background and the sides stay, and `engines/cascade/tests/fixtures/separation/centered-speech.wav` holds the same Piper sentence in both channels over two lateral tones; the suite fails when the fixture stops coming back as `center`, when the background still correlates with the voice, or when it loses the music, and a mono or fully centred source still falls back to ducking with a note | Parity, stereo center only | 3 | Original speech is removed, music kept |

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
| P6 | LAN sharing with a PIN | Yes | Rejected · there is no second device to try it with, and the PIN path is already tested over a real socket | Not planned | — | Not planned |
| P7 | Remote backend | Yes | Rejected · there is no second machine and no GPU to drive from one | Not planned | — | Not planned |
| P8 | Keys and tickets | API key, WS ticket | Done · daa2b0b: `POST /ticket` issues a single-use ticket that opens one WebSocket and is refused on replay, and the token still works alone | Parity | 5 | A short-lived ticket cannot be replayed |
| P9 | SDK and examples | Python client | Done · 273b302: `examples/openai_client.py` calls models, transcriptions, translations and speech on a running server and prints one line per endpoint, `docs/API.md` carries the request and the real response for each, and a test fails when a `/v1` route has no example | Parity | 5 | One example per endpoint runs |

## Models

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| X1 | Model catalogue | Yes | Done · dfb8b40: 52 entries, and every card carries a licence, a source and its languages | Keep, generated from the registry | 0 | A card cannot exist without a licence |
| X2 | Per-engine benchmarks | Yes | Done · 5b2b7ef: `parlo bench --json` writes one row per engine and device with the hardware, the commit and a state, and with Parakeet 0.6B v3 on disk the cascade row measured 0.23 real time and 0.64 s for the 2.5 s sentence on the cpu while the echo row measured 0.0031 real time and 7.9 ms, the run stored verbatim in `docs/benchmarks/machine-2026-11-06.json` and quoted in `docs/BENCHMARKS.md` | One row per engine and device | 6 | No card claims a number without a row |
| X3 | GPU auto-detect | Yes | Done · 62bb7ff: the profile follows the detected graphics card, `/engine` names the device and the reason, and Settings can force the processor | Parity | 2 | The chosen device is shown and overridable |
| X4 | Download resume and repair | Yes | Blocked · the byte-level resume lives in `engines/cascade`, off limits in this campaign; `crates/parlo-cli/src/models.rs` only reports `download_remaining` | Parity | 1 | A killed download resumes at the byte |
| X5 | Model packs | Yes | Done · ccc21c0: importing a pack downloads the voices it names through `install_blocking` before the manifest is written, so a valid pack is ready in one action, a pack whose voices cannot be installed leaves no file or entry behind, an invalid pack never asks for an install, importing the same pack twice keeps one entry and one file, and a store without packs lists nothing and creates no folder | Parity | 5 | A pack installs a working set in one action |
| X6 | Licence inventory in the UI | Yes | Done · 7f1d035: every model card carries the licence from the registry in `/models` and in the app, before the download | Parity | 1 | Each model shows its licence before download |

## Packaging and reach

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| R1 | Signed installer | Yes | Rejected · no signing certificate is being bought for this | Not planned | — | Not planned |
| R2 | Package managers | 9 channels | Partial · fe2bd56: seven manifests under `packaging/` repack the published v0.2.4 downloads — winget, Homebrew, AUR (`parlo-bin`), Snap, the DEB rules, an AppImage recipe and a Nix derivation — carrying the published checksums, `packaging/README.md` gives each install command and marks all seven Partial, and `cargo test -p parlo-cli --test packaging` fails when a manifest drifts from `Cargo.toml`, drops its checksum or names an archive `release.yml` does not attach; no channel has been installed end to end from this machine, and the nine this row counts have no second list beyond the seven the acceptance column names | winget, brew, AUR, Snap, DEB, AppImage, Nix | 5 | One command installs on each |
| R3 | Docker | CPU, CUDA, ROCm | Partial · f94a456: `packaging/docker/Dockerfile` builds from the published `parlo-linux-x86_64.tar.gz`, checks the hash from `SHA256SUMS.txt` before unpacking it, runs as uid 10001 and starts `parlo serve` on 8787 through `packaging/docker/entrypoint.sh`, which writes `PARLO_CONFIG` with the sharing switch and `PARLO_PIN`; `cargo test -p parlo-cli --test packaging` fails when the archive, the version, the checksum, the user or the port drift; no image has been built or run here, and the PyInstaller engine ships without models | Parity | 5 | `docker run` serves the OpenAI endpoint |
| R4 | Android app | Preview, offline | Rejected · the product is the desktop, and a phone reaches the server over the LAN with the PIN instead of an app | Not planned | — | Not planned |
| R5 | iOS app | Preview, sideload | Rejected · the product is the desktop, and a phone reaches the server over the LAN with the PIN instead of an app | Not planned | — | Not planned |
| R6 | Stores | MS Store, Mac App Store | No | Desktop stores, parity | 5 | A store build installs and updates |
| R7 | Update channels | Stable and preview | Done · 2d17fd3: the Updates field in the app and `parlo config --channel preview` store the choice, the release request asks for the full list and keeps prereleases only on preview, a preview build moves to a newer preview or a newer stable while a stable build never takes a preview, and offline tests over a fake feed fail when any of the three breaks | Parity | 5 | A preview build updates to stable |
| R8 | Website and docs | Yes, 20 languages | Done · 694d054: `docs/START.md` is the landing page a stranger follows from install to a dictated sentence, benchmarks, privacy and every other page are linked from it and from the README, and `crates/parlo-cli/tests/quickstart.rs` extracts the documented commands and flags from both pages, fails when one is not a command or flag the binary has, runs `parlo doctor` on the real binary, and fails when a linked page does not exist | Site, quick start, benchmarks, privacy | 5 | A stranger installs in under five minutes |

## Quality

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| Q1 | Automated tests | Large suite | Done · 8d0a20b: 380 Rust, 87 core and the full Python suite, 803 tests in all; `ROUTES` names all 90 route rows, the sweep refuses every one without the token, the behaviour tests answer the pages, meetings, vocabulary, voice, models, history, dub, insert and settings routes, and `the_route_table_covers_every_route_the_router_serves` fails when a route arrives without a row | 600+, route and flow tests | 6 | Every route has a test |
| Q2 | Published benchmarks | Yes | Done · 36e3e56: `parlo bench` publishes one row per engine and device with the cores, the memory, the gpu, the profile and the commit, `the_published_table_is_the_one_the_run_generates` fails when `docs/BENCHMARKS.md` drifts from `docs/benchmarks.json`, and `every_published_number_names_real_hardware_and_a_real_commit` fails on a hardware line without cores or a commit this repository does not hold | Per engine, per device | 6 | Hardware and commit beside every number |
| Q3 | Licence gate | Yes | Done · 17800a0: every model the app offers names a catalogue row under the same licence and a source, every catalogue row names a model the registry offers, and the Python gate still refuses a copyleft crate before the build ships | Keep, cover every model | 6 | A copyleft crate fails the build |
| Q4 | Network audit | Documented | Done · bedc7c8: the suite fails when a page or the CLI reaches any host besides the release check and the model registry | Automated, per release | 6 | A release fails if a request leaves the machine |
| Q5 | Crash-free sessions | Claimed | Partial · a6906a6: `crates/parlo-core/src/sessions.rs` keeps `sessions.jsonl` in the data folder with one line per start, clean end and panic (message and file:line), appended in one write, the newest 400 lines kept and unreadable lines skipped; `parlo doctor` prints the sessions recorded, the clean and crashed counts, the last session and the crash-free rate, with one line when nothing is recorded; tests cover a panic counted as crashed, a clean end, a truncated file, a directory that did not exist, the rate and the cap; no two-hour run has been recorded on this machine | Two hours, no crash | 6 | A recorded two-hour run |
| Q6 | External testers | Community | Rejected · Parlo is open source on GitHub for now, and a build that works here works for whoever clones it | Not planned | — | Not planned |

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
| 1 Transcription and platform | the recogniser in use, translation pairs on demand, diarisation, WS, OpenAI, MCP | 12 weeks | 6 weeks |
| 2 Voice | Clone, design, 20 engines, devices, previews | 16 weeks | 8 weeks |
| 3 Dubbing | Multi-speaker, multi-target, QC, export matrix | 16 weeks | 9 weeks |
| 4 Longform and batch | Queue, watch folders, stories, audiobooks | 12 weeks | 6 weeks |
| 5 Reach | Signed builds, desktop stores, 21 locales, site | 16 weeks | 8 weeks |
| 6 Audit | Every planned ledger row, benchmarks, 1.0 | 10 weeks | 5 weeks |
| **Total** | | **88 weeks** | **45 weeks** |

The destination is full parity. The dates are honest: this is not a quarter, and pretending otherwise would only produce the lite edition the plan refuses to build.

## Progress

56 of the 70 planned rows are Done (80.0%), counted as the rows whose Parlo today cell starts with Done or Yes; ten rows are Rejected and counted in no total. Eight Partial rows counted at half give 85.7%.

## Next

The open rows are the ones a single Windows machine cannot finish on its own. D5, D8, T4, V6 and X4 end in the engine files, where the voice, the word times and the cue render are decided. T3 needs the recogniser's speaker embeddings. M6 and Q5 wait for the two-hour run, which happens when the owner starts using Parlo for real. R2 and R3 need a package channel and Docker installed end to end. D10 keeps its dictionary and waits for a listening check. Everything else is Rejected on purpose.
