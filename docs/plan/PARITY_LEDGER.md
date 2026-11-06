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
| D9 | Snippets and vocabulary | Yes | Done · f9a304c: snippets fill `{{date}}`, `{{time}}` and `{{name}}`, and a personal term is applied after the cleanup pass | Keep | 0 | `{{date}}` fills; a term survives cleanup |
| D10 | Pronunciation dictionary | Respelling, IPA, CMU | Partial · 5b7e517: fourteen names live in `crates/parlo-core/assets/pronounce.json` with a respelling, an IPA form and a CMU form each, `Pipeline::speak` respells the whole sentence before any engine hears it, `parlo pronounce [word] [--text TEXT]` shows the table and the substitution, and tests prove a word inside a longer word, a word with a digit after it, a change of case, punctuation and an empty string all behave, with a recorder engine hearing `PAR-loh reads JAY-son`; what is missing is that nobody heard it, because no text to speech model is installed on this machine, so the listening check was not run | Parity | 2 | A respelled name is spoken as written |

## Transcription

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| T1 | Recogniser breadth | 11 engines | 2 | 12 engines | 1 | Each adapter has a WER row on the FalAR corpus |
| T2 | Language breadth | 646 | 2 | 646 class, permissive weights only | 1 | Published count equals the measured count; NLLB excluded by licence |
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
| M3 | Decisions and action items | LLM summary | Rule-based, cited | Keep as an edge | 0 | Each item links to speaker and minute |
| M4 | Ask with citations | Not documented | Yes | Keep as an edge | 0 | An answer with no match says so |
| M5 | Local summary model | Yes | Done · c0e8ce5: the meeting summary asks the local model, falls back to the local rules when it cannot answer, and the screen names which one wrote it with the reason for the fallback | Local model, rule fallback | 4 | The screen says which produced the summary |
| M6 | Two-hour reliability | Claimed | Partial · c5deb53: `parlo soak --seconds N --interval S` starts the server as a child process, samples its working set every 15 s with a named cap of 480 samples, stops it and reads `sessions.jsonl` back, and a real run on this machine measured 184 s with 11.8 MB first, 11.7 MB last and 11.8 MB peak over 12 samples, one session and no crash; the two-hour window with the app beside the server has not been run here, and a test exercises a four second run on every `cargo test` | 2 hours, no crash, flat memory | 6 | A two-hour run with a memory chart |

## Voice

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| V1 | Clone from 5–75 s | Yes | No | Parity | 2 | A 15-second sample previews in under 4 s |
| V2 | Design a voice from words | Yes | No | Parity | 2 | Same seed and description give the same voice |
| V3 | Consent, revoke, identity lock | Yes | Done · 1b85e5a: revoking deletes `sample.wav` from disk and drops it from the profile's artefacts, the profile and its revocation stay on record, a revoked profile refuses a new sample, playback of a revoked voice answers with the default voice instead of failing, the app offers that fallback and says why, and a profile written before revocation existed reads exactly as before | Parity | 2 | Revoking deletes the sample and the next render uses the default |
| V4 | TTS engine breadth | 17 | 1 (Piper) | 20 | 2 | Every engine has a licence row and a measured speed |
| V5 | Device selection | CUDA, ROCm, MPS, Vulkan | CPU only | Parity, per engine | 2 | A GPU row exists only where measured |
| V6 | Expressive controls | Emotion, direction, cues | Partial: per-cue pace reaches Piper and is saved; no emotion or pitch | Parity | 3 | A cue changes the render and is saved |
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
| P6 | LAN sharing with a PIN | Yes | Partial · 4bf91c0: `parlo config --share on --pin 4821` and the Sharing field store the switch, PIN and port, the server binds `0.0.0.0` only when sharing is on with a four-to-eight digit PIN, and tests over a real socket prove a request without the PIN and a wrong PIN get 401 on `/state` and `/v1/models` while the right PIN gets 200 and a closed share leaves the port refusing connections; a phone on a real LAN could not be tried from this machine | Parity, off by default | 5 | A phone on the LAN transcribes with the PIN |
| P7 | Remote backend | Yes | Partial · d2a7191: `parlo serve --tls-cert --tls-key` binds the network and answers plain HTTP with 426, `parlo config --remote-url https://host:8788 --remote-token …` stores the address and `parlo remote file.wav` posts to it over rustls with native roots or `--remote-ca`, and tests prove a certificate the client does not trust is refused, a trusted one reaches the API, a wrong token gets 401 and plain HTTP gets 426; a laptop on another network could not be tried from this machine | Parity, HTTPS only | 5 | A laptop drives a desktop GPU |
| P8 | Keys and tickets | API key, WS ticket | Done · daa2b0b: `POST /ticket` issues a single-use ticket that opens one WebSocket and is refused on replay, and the token still works alone | Parity | 5 | A short-lived ticket cannot be replayed |
| P9 | SDK and examples | Python client | Done · 273b302: `examples/openai_client.py` calls models, transcriptions, translations and speech on a running server and prints one line per endpoint, `docs/API.md` carries the request and the real response for each, and a test fails when a `/v1` route has no example | Parity | 5 | One example per endpoint runs |

## Models

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| X1 | Model catalogue | Yes | Done · dfb8b40: 52 entries, and every card carries a licence, a source and its languages | Keep, generated from the registry | 0 | A card cannot exist without a licence |
| X2 | Per-engine benchmarks | Yes | Partial · afefed4: `parlo bench --json` writes one row per engine and device with the hardware, the commit and a state, the run of this machine is stored verbatim in `docs/benchmarks/machine-2026-11-06.json` and quoted in `docs/BENCHMARKS.md`, and `cargo test -p parlo-cli --test benchmarks` fails when a measured row loses its real time factor, its seconds, its frames or its commit, or when a row claims a run it did not make; on 2026-11-06 the echo row measured 0.0026 real time and 6.4 ms for the 2.5 s sentence while the cascade row has no number because Parakeet v3 and the python stack are not on this machine | One row per engine and device | 6 | No card claims a number without a row |
| X3 | GPU auto-detect | Yes | Done · 62bb7ff: the profile follows the detected graphics card, `/engine` names the device and the reason, and Settings can force the processor | Parity | 2 | The chosen device is shown and overridable |
| X4 | Download resume and repair | Yes | Blocked · the byte-level resume lives in `engines/cascade`, off limits in this campaign; `crates/parlo-cli/src/models.rs` only reports `download_remaining` | Parity | 1 | A killed download resumes at the byte |
| X5 | Model packs | Yes | Done · ccc21c0: importing a pack downloads the voices it names through `install_blocking` before the manifest is written, so a valid pack is ready in one action, a pack whose voices cannot be installed leaves no file or entry behind, an invalid pack never asks for an install, importing the same pack twice keeps one entry and one file, and a store without packs lists nothing and creates no folder | Parity | 5 | A pack installs a working set in one action |
| X6 | Licence inventory in the UI | Yes | Done · 7f1d035: every model card carries the licence from the registry in `/models` and in the app, before the download | Parity | 1 | Each model shows its licence before download |

## Packaging and reach

| ID | Capability | VoiceStudio | Parlo today | Target | Phase | Acceptance |
|---|---|---|---|---|---|---|
| R1 | Signed installer | Yes | No | Authenticode and notarised macOS | 5 | SmartScreen does not warn |
| R2 | Package managers | 9 channels | Partial · fe2bd56: seven manifests under `packaging/` repack the published v0.2.4 downloads — winget, Homebrew, AUR (`parlo-bin`), Snap, the DEB rules, an AppImage recipe and a Nix derivation — carrying the published checksums, `packaging/README.md` gives each install command and marks all seven Partial, and `cargo test -p parlo-cli --test packaging` fails when a manifest drifts from `Cargo.toml`, drops its checksum or names an archive `release.yml` does not attach; no channel has been installed end to end from this machine, and the nine this row counts have no second list beyond the seven the acceptance column names | winget, brew, AUR, Snap, DEB, AppImage, Nix | 5 | One command installs on each |
| R3 | Docker | CPU, CUDA, ROCm | Partial · f94a456: `packaging/docker/Dockerfile` builds from the published `parlo-linux-x86_64.tar.gz`, checks the hash from `SHA256SUMS.txt` before unpacking it, runs as uid 10001 and starts `parlo serve` on 8787 through `packaging/docker/entrypoint.sh`, which writes `PARLO_CONFIG` with the sharing switch and `PARLO_PIN`; `cargo test -p parlo-cli --test packaging` fails when the archive, the version, the checksum, the user or the port drift; no image has been built or run here, and the PyInstaller engine ships without models | Parity | 5 | `docker run` serves the OpenAI endpoint |
| R4 | Android app | Preview, offline | No | Native, offline after download | 5 | A phone transcribes a file in airplane mode |
| R5 | iOS app | Preview, sideload | No | Native, offline | 5 | The same file transcribes on an iPhone |
| R6 | Stores | MS Store, Mac App Store, F-Droid | No | Parity | 5 | A store build installs and updates |
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

53 of 80 rows are Done (66.3%), counted as the rows whose Parlo today cell starts with Done or Yes. Eleven Partial rows counted at half give 73.1%.

## Next

D5, T4 and X4 stay parked: the first three on the engine files, X4 on the byte-level resume that lives there too. X2 now measures the cascade whenever its models are on disk, and waits only for Parakeet 0.6B v3 here. The cheapest unblocked row left is D8, where a program already owns a style and a written language and only the voice is still global; after it comes T1, one more recogniser with a WER row, and the rest of the open rows need a model, a graphics card, a signing certificate, Docker, a phone or two hours of running.
