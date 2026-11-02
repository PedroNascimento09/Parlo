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

Built so far: a rail with Home, Library, Meetings, Voice and Settings, a hash router that hides the sections outside the chosen one and marks the current link, and Home carrying the shared state, the shortcut, the language pair and the mode. The hold-to-speak button is the first-run test: it sits in the welcome block, which Home shows until the models are on disk, and it goes away once dictation works. Home lists the last three results, each with its time, its language pair and the program it landed in.

Estimate: 5 hours.

## Phase 5: Library

1. Merge Models, History and Vocabulary into one destination with three tabs.
2. Add filters by application and mode, plus search across the text.
3. Vocabulary moves out of Settings, where it never belonged.

Check: find a dictation from last week by the application it went into.

Built so far: the three parts share the Library destination, and the history keeps every entry in memory, lists the programs it has seen, and filters by program, mode and a word of the text. A row now reads the time, the language pair, the program and the mode. Models, Vocabulary, Files and History now sit behind four tabs over one Library destination.

Estimate: 4 hours.

## Phase 6: the remaining surfaces

1. Settings split into eight sections, cloud key under Privacy with the sentence about what leaves the machine.
2. Tray: modes and destinations, state in the tooltip, no action duplicated from the window.
3. Onboarding: five steps under sixty seconds, ending with a real insertion.
4. Captions as its own floating window, then Live, then Meetings with a lifecycle.

Built so far: Settings carries eight tabs, one screen each: Dictation with the feature switches, the recogniser and the idle timer, Languages with the pair, Shortcuts with the keys, Cleanup with the engine, Provider with the model it may call, Privacy with the cloud key and the sentence about what leaves the machine, About with the version and the check for a newer release, and Advanced with the licences; one Apply sits under every section, and the tray tooltip now reads the shared state, so the three surfaces repeat the same word. The onboarding is three steps and proves the loop instead of describing it: it waits for the shared state to say listening, then for a new history entry, then reads back the sentence and the program it landed in. Skipping sets a local flag and never asks again. Captions live in their own always-on-top window served from /captions, started from the tray, with a pause button and a size button; the audio is cut into two and a half second windows and each one is sent through /dictate. Live has its own window too, served from /live, with what was heard and what it became side by side. Meetings keep each line with its speaker, its timestamp and an optional real name, and the window now copies every meeting as Markdown. A summary with decisions and action items still needs a model this build does not run offline, so it is not invented. A meeting can be summarised by a model the reader runs on this machine: the button posts to /meeting/summary, which answers with a sentence instead of inventing when no local address is configured, and the client only speaks http:// to loopback.

Estimate: 12 hours across several sittings.

## Order of evidence

Every phase ends with the same three checks: the crate's tests, clippy with warnings as errors, and one hand check of the flow it changed. A phase that cannot be hand checked on this machine says so in its commit message rather than claiming it works.
