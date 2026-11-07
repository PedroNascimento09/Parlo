# The build plan

`docs/UX.md` says what the product should be. This file says what to change, in what order, and how to know each step worked.

Six phases. Each one leaves the app usable, and each one is finished before the next starts. The order is chosen so the shortest phase fixes the worst defect.

## Phase 1: the dictation key behaves

Why first: Escape inserts and focus loss submits. Both lose a sentence.

1. Split `finish()` in `crates/parlo-cli/web/island.html` into `stop()` and `cancel()`. `stop()` transcribes, inserts, writes history. `cancel()` closes the microphone, drops the audio, writes nothing, and returns to Ready.
2. Bind `Escape` to `cancel()`, and say so in the overlay hint while listening.
3. Delete the `visibilitychange → finish()` handler. Losing focus keeps the session open, because clicking the target application is the normal step, not an abandonment.
4. Register the shortcut with a low level keyboard hook in `crates/parlo-app/src/island.rs`, because `RegisterHotKey` reports a press and never a release. Down starts capture, up stops it. Keep `RegisterHotKey` as the fallback and say which mode is active in the log.
5. A tap shorter than 250 ms means hands free: capture stays open until the next press.

Check: hold the key, speak, release; the text lands in Notepad. Press the key, speak, press Escape; nothing is inserted and no history entry appears. Click into another window mid-sentence; the session survives and the text still lands.

Tests: `cargo test -p parlo-app -p parlo-cli`, `cargo clippy -- -D warnings`. The key release path is Windows only, so it is checked by hand on this machine and recorded in the release notes as such.

Estimate: 2 hours.

Built: `island.html` separates `stop()` from `cancel()`, Escape cancels, losing focus no longer submits, and `island.rs` reads the shortcut with a low level keyboard hook so a release stops the capture. A tap shorter than 250 ms stays listening for hands free. The island reports its busy state on `/island/busy` and the app hides it once it has been idle for a second and a half.

## Phase 2: one state machine

1. Add `crates/parlo-cli/src/live_state.rs` with the eight states from `docs/UX.md` and the legal transitions, plus a unit test per transition.
2. Expose it as `GET /state` and push it to the island as `POST /state/stream` or a short poll, whichever the island renders without a frame drop.
3. Render the same state in three places: island line, tray tooltip, window readiness line. No surface invents its own wording.

Check: start a dictation and watch the island, the tray and the window agree at every step.

Built: `parlo_core::state` holds the eight states with the legal transitions and seven tests over them. The engine serves `GET /state` and accepts `POST /state`, refusing a step that skips a state with 409 and naming both states in the message. The island reports listening, processing, success, error and cancelled, and the app hides it once it reports a quiet state for three polls.

Estimate: 3 hours.

## Phase 3: insertion certainty

1. Add `GET /target` returning the foreground application name and whether it accepts typed input.
2. The island asks before it opens the microphone. The hint reads "Text goes into Notepad", or "No text field here, the text goes to the clipboard".
3. History gains `app` and `mode`. Entries written before this change read as `unknown` instead of failing to parse.

Check: dictate into two different applications and confirm each history row names the right one.

Built: `insert::foreground` names the program that owns the focused window through `QueryFullProcessImageNameW`, `GET /target` reports it with whether Parlo can type there, the island says the name before it starts listening, and a history entry now carries `app` and `mode`. Entries written before this change still load, reading as empty strings.

Estimate: 3 hours.

## Phase 4: Home becomes status

1. Add a navigation rail to `index.html` with five destinations: Home, Library, Meetings, Voice, Settings.
2. Move the sections into those destinations without changing their ids, so `app.js` keeps working while the router is added.
3. Home shows readiness, shortcut, mode, languages, active model, and the last three results. The hold-to-speak button becomes the onboarding test only.
4. Licences move to Settings → Advanced, dubbing to Library.

Check: a person who never used Parlo answers "is it ready, and what does it do" from Home without scrolling.

Built so far: a rail with Home, Library, Meetings, Voice and Settings, a hash router that hides the sections outside the chosen one and marks the current link, and Home carrying the shared state, the shortcut, the language pair and the mode. The hold-to-speak button is the first-run test: it sits in the welcome block, which Home shows until the models are on disk, and it goes away once dictation works. Home lists the last three results, each with its time, its language pair and the program it landed in. No mode starts from the window: the Live and Dictate rows are gone with the code behind them, because the tray and the shortcut are where a mode begins, and the one button left translates a single clip.

Estimate: 5 hours.

## Phase 5: Library

1. Merge Models, History and Vocabulary into one destination with three tabs.
2. Add filters by application and mode, plus search across the text.
3. Vocabulary moves out of Settings, where it never belonged.

Check: find a dictation from last week by the application it went into.

Built so far: the three parts share the Library destination, and the history keeps every entry in memory, lists the programs it has seen, and filters by program, mode and a word of the text. A row now reads the time, the language pair, the program and the mode. Models, Vocabulary, Files and History now sit behind four tabs over one Library destination. A history row carries the sentence, the program and the mode, and its own actions: **Favourite**, with a **Favourites only** filter that keeps the marked ones, **Correct**, which replaces the sentence Parlo wrote down, and **Delete**, which removes that one entry after asking; every entry has an id the local server knows, so a line written before ids existed is still found. Home lists the last five results with a link to the Library, and shows how many models are installed, their size and a manage link.

Estimate: 4 hours.

## Phase 6: the remaining surfaces

1. Settings split into eight sections, cloud key under Privacy with the sentence about what leaves the machine.
2. Tray: modes and destinations, state in the tooltip, no action duplicated from the window.
3. Onboarding: five steps under sixty seconds, ending with a real insertion.
4. Captions as its own floating window, then Live, then Meetings with a lifecycle.

Built so far: Settings carries eight tabs, one screen each: Dictation with the feature switches, the recogniser and the idle timer, Languages with the pair, Shortcuts with the keys, Cleanup with the engine, Provider with the model it may call, Privacy with the cloud key and the sentence about what leaves the machine, About with the version and the check for a newer release, and Advanced with the licences; one Apply sits under every section, and the tray tooltip now reads the shared state, so the three surfaces repeat the same word. The tray menu lists the three modes above a separator and the five destinations below it, each opening the window on its own screen, and Open Parlo is gone because Home does the same thing. The onboarding is five steps and proves the loop instead of describing it: the pair, the microphone with the exact place to allow it, the shortcut captured from the keys pressed, then one real sentence it waits for, shared state listening first and a new history entry after, read back with the program it landed in, and a last line saying where the shortcut is and what Escape does. Skipping sets a local flag and never asks again. Captions live in their own always-on-top window served from /captions, started from the tray, with a pause button and a size button; the audio is cut into two and a half second windows and each one is sent through /dictate. Live has its own window too, served from /live, with what was heard and what it became side by side. Meetings keep each line with its speaker, its timestamp and an optional real name, and the window now copies every meeting as Markdown. A summary with decisions and action items still needs a model this build does not run offline, so it is not invented. A meeting can be summarised by a model the reader runs on this machine: the button posts to /meeting/summary, which answers with a sentence instead of inventing when no local address is configured, and the client only speaks http:// to loopback. A meeting now ...

Estimate: 12 hours across several sittings.

## Order of evidence

Every phase ends with the same three checks: the crate's tests, clippy with warnings as errors, and one hand check of the flow it changed. A phase that cannot be hand checked on this machine says so in its commit message rather than claiming it works.

## Workstreams 2–4 audit (2026-11-07)

Each item is `DONE` with an observed result or `BLOCKED` with the exact missing resource.

### Workstream 2

- **DONE — `/models`.** The shadowing block is removed; the endpoint returns the full lifecycle document, 26 rows and `dictation: true`.
- **DONE — tools on real audio.** Transcription, `POST /dictate` PT→EN, `parlo translate`, `parlo stream`, `POST /v1/audio/speech`, `POST /listen/stream`, `parlo dub`, meeting import, vocabulary and history all returned observed output. The exact transcripts and timings are in `PROGRESS.md`.
- **DONE — clean `voice-pt` install.** With an empty `PARLO_MODELS_DIR`, `POST /models/download/voice-pt` wrote `pt_PT-tugão-medium.onnx` (63,201,294 bytes) and its JSON (5,026 bytes); `/models` reported 63,206,320 bytes and `installed: true`. `POST /v1/audio/speech` then returned a RIFF WAV, 16 kHz mono, 51,314 bytes and 1.60 s. No manual copy was used. `voice_size` now requires both files, tested by `a_voice_without_its_settings_is_not_installed`.
- **DONE — streaming and the remaining tools.** First partial 600 ms, 7 partials, 2 stable, final 3611 ms and 442 ms decode; live partial/stable/final; 36,414-byte dub WAV plus SRT; two meeting segments with an action; vocabulary and three history entries.

### Workstream 3

- **DONE — gate and locales.** Portuguese gate at 1440×900, `PASSO 1 DE 5`, no `Alex` placeholder, 24 `gate.*` keys in all 21 locales; the locale completeness test and `check-languages.mjs` pass.
- **DONE — visual fit.** Home was reviewed at 1440×900; the clipped Settings link ends at 1352 px after the masthead gap and rail padding changes.
- **DONE — routes.** `web/routes.js` covers deep links, unknown tabs and reload; 8 `node:test` cases pass, CI runs them, and the server and share serve `/routes.js` instead of returning 404.

### Workstream 4

- **DONE — native startup.** `parlo-app.exe` opened with title `Parlo` and `Responding=True`; only startup, title and responsiveness are claimed because the browser cannot capture the native window content.
- **BLOCKED — fresh bench run.** `the_published_table_matches_a_fresh_run` is ignored because the test model root lacks `parakeet` (`encoder.int8.onnx`, `decoder.int8.onnx`, `joiner.int8.onnx`, `tokens.txt`) and `mt-pt-en`, plus `engines/cascade/tests/fixtures/timing/source.wav`. Restore with `PARLO_MODELS_DIR=<installed models> cargo test -p parlo-cli bench::tests::the_published_table_matches_a_fresh_run -- --ignored`. The static published-table test passes.
- **DONE — checks and hygiene.** Workspace build, workspace clippy with `-D warnings`, `fmt --check`, both `node --check` runs and the 8 route tests pass; the workspace suite is 440 passed and 1 ignored. The proof server is closed, temporary data and audio are deleted, the manual voice copy was removed, and nothing was committed or pushed.

### Known limitations

The 0.24 WER is visible in the observed transcripts, the TTS round-trip (`Hoj esta um diar bonito amlispoa.`) is imperfect, no physical microphone exists on this surface, and native window content is not browser-capturable. These are recorded as limitations, not as passing checks.
