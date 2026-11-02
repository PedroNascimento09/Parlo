# Parlo as a voice layer

The product is not a window with a record button. It is a shortcut, an overlay, and a machine that writes what you said into whatever field you were already in. The window exists to configure that machine and to read back what it did.

This document audits what the build does today, then states the architecture the build should have. It is not a wish list: every section names what changes in the code, and the last section says which changes come first.

## What the build does today

Facts, read from the repository, not remembered.

| Surface | File | What it does |
| --- | --- | --- |
| Main window | `crates/parlo-cli/web/index.html`, 370 lines | One scrolling page: hero with a hold-to-speak button, a Live row, a Dictate row, four status lines, then languages, cleanup, vocabulary, meetings, model library, history, voice, models, dubbing, licences |
| Global shortcut | `crates/parlo-app/src/island.rs` | `RegisterHotKey` on the configured combination, then toggles the island window visible or hidden |
| Island | `crates/parlo-cli/web/island.html`, 444 lines | Separate window, opens the microphone itself on focus, records on a timer, posts the WAV to `/dictate`, posts the text to `/insert`, copies to the clipboard when insertion fails |
| Tray | `crates/parlo-app/src/presence.rs` | Dictate, Live, Open Parlo, Settings, Quit; a left click toggles the island |
| Capture and insertion | `crates/parlo-cli/src/serve.rs`, `insert.rs` | `SendInput` types the text into the focused field on Windows |
| History | `crates/parlo-cli/src/history.rs` | Append-only JSONL, 200 entries, fields `at`, `text`, `translation`, `from`, `to` |

### What is wrong with it

1. **The main window is the recorder.** The largest element on the page is "Hold to speak", and every other capability sits below it in one scroll. A person who opens Parlo sees a recorder, not the state of a voice layer.
2. **The shortcut is a window toggle, not push-to-talk.** Key down shows the island, key up does nothing. There is no distinction between holding and pressing, so the mental model the overlay teaches ("press and speak") is false.
3. **Escape inserts instead of cancelling.** In `island.html`, `Escape` calls `finish()`, which transcribes and inserts. The one key a person presses to abandon something is the key that commits it.
4. **Losing focus ends the dictation.** `visibilitychange` calls `finish()`, so clicking into the target application — the normal thing to do — submits a partial sentence.
5. **Nothing is remembered about where the text went.** History stores the sentence but not the application, the mode, or the duration, so it cannot answer "what did I dictate into Slack".
6. **The island does not know whether it can insert.** It only discovers that after the fact, by getting a failure back and falling back to the clipboard.
7. **Configuration lives on the daily surface.** Model library, licences and dubbing share the page with the thing a person does twenty times a day.
8. **No cancellation state exists anywhere.** There is Ready, Listening, Writing, Inserted, Copied, Failed. "Stop" and "cancel" are the same code path.
9. **The overlay only knows the primary monitor.** `place()` uses `primary_monitor()`, so on a second screen it appears on the wrong one.
10. **The tray duplicates the window.** Dictate opens the island, Live and Settings open the window; three entries and one duplicate action.

None of that is a styling problem. It is an architecture problem, and the fix is to make the shortcut the product.

## A. Product philosophy

1. The shortcut is the product. Everything else is support for it.
2. The window is for configuration and review. It never asks a person to dictate from inside it.
3. One key starts a sentence, the same key ends it, Escape throws it away. Stop and cancel are different verbs and different keys.
4. The interface answers "is it listening" without being asked: through the overlay, the tray icon and the window, always the same three states.
5. Text goes to the cursor. The user never chooses a field, because the field was already chosen by working in it.
6. Local by default. Cloud sits behind an explicit switch that names what leaves the machine.
7. A feature that cannot justify a permanent surface does not get one: it is contextual, automatic, in Advanced, or it does not ship.
8. Nothing is announced as done before it is done. "Inserted" means the field received the text.

## B. Information architecture

The window becomes five destinations, each answering one question.

```
Parlo
├── Home          Is it ready, and what did it just do?
├── Library       What have I said, and what is installed?
│   ├── History
│   ├── Models
│   └── Vocabulary
├── Meetings      What was said in a meeting, and what did we decide?
├── Voice         What does my translated voice sound like?
└── Settings      How does it behave?
    ├── General
    ├── Dictation
    ├── Shortcuts
    ├── Audio
    ├── Languages
    ├── Appearance
    ├── Privacy
    └── Advanced
```

Changes from the current tree, stated plainly:

- **Delete the hero recorder.** It survives only inside onboarding as "Try it once".
- **Delete Dictate and Live as pages.** They are modes of the shortcut and entries in the tray, not destinations.
- **Merge Models and History into Library**, with Vocabulary beside them, because all three answer "what do I have".
- **Merge Meetings and Meeting Memory** into one page with two tabs, since a meeting and its memory are the same object at different times.
- **Move Vocabulary out of Settings.** It is content, not configuration.
- **Move cleanup and cloud provider into Settings → Dictation**, and keep the API key under Privacy, where the sentence about what leaves the machine belongs.
- **Move model library, licences and dubbing off the daily surface.** Licences go to Settings → Advanced. Dubbing becomes a tool under Library, opened on purpose.
- **Tray loses nothing but stops duplicating**: it lists modes and destinations, and the window's own copy of the same action disappears.

## C. Interaction architecture

```
                    ┌──────────────────────────────┐
   shortcut down ──▶│  Island (always the truth)   │──▶ inserted text
   shortcut up   ──▶│  Ready / Listening / …       │
                    └──────────────────────────────┘
                                  ▲
   tray ──────────────────────────┤
   window ────────────────────────┘

   Window  = configure, review, choose models, read history
   Tray    = start a mode, reach a destination, quit
   Island  = say what is happening, and take one click: stop, cancel
   Shortcut= the only way a person dictates in normal work
```

Every surface reads its state from one place, so they cannot disagree. The state lives in the engine process, not in each view: `serve.rs` exposes it, the island, tray and window render it.

## D. User flows

### Push to talk

1. Cursor is in a field, in any application.
2. Shortcut down: microphone opens, island appears at the bottom of the screen holding the cursor, first audio is captured in under 100 ms.
3. Speak. The island shows a level meter and a clock; the target application keeps focus.
4. Shortcut up: capture stops, island shows "Processing".
5. Transcription, cleanup, insertion into the remembered field.
6. Island shows "Inserted" for 1.6 s, then returns to Ready. Focus never moved.

### Hands free

1. Shortcut down and up quickly (a tap, under 250 ms): the session stays open, island shows "Listening, tap to stop".
2. Speak, pause, continue. Silence of 2.5 s with no new speech offers to finish; the island counts down 3 s and the session ends by itself unless the user keeps talking.
3. A tap stops and processes. Escape cancels.

### Cancel

Escape at any moment while listening, processing, before insertion. Capture is discarded, no text is written, no history entry is created, island returns to Ready. The target field is never touched. This is the flow that today inserts text instead.

### Dictation versus translation

The mode decides what happens after transcription: dictation inserts the words, translation inserts the words and plays the other language. One setting, one word in the island ("Listening · dictating" or "Listening · translating"), no separate pages.

### Captions

Started from tray or window, never from a page visit. A floating captions window, movable and resizable, with four controls: language, size, position, close. It keeps its own state and can run while dictation is used elsewhere.

### Live

Started from tray, window or a second shortcut. Translation runs on segments and plays through the output device; the island shows the running translation as a single line, and the window shows the log when opened. Stopping keeps the log in History with mode `live`.

### Meeting

Before: title, microphone or system audio, language, model. During: transcript with speakers and timestamps, pause and resume, notes. After: summary, decisions, action items, export, searchable memory. A meeting is a session with a lifecycle, not a long dictation.

### Voice

Configure only: create a profile, record samples, hear a test sentence, set quality, revoke, delete. Nothing here is on the path of daily speech.

### History and Settings

History is a searchable list with copy, insert again, edit, favourite, delete, filtered by application and mode. Settings is eight short sections, each one screen of fields, no scrolling page of everything.

## E. State machine

One machine, shared by island, tray and window.

| State | Entered by | Leaves to | Shows |
| --- | --- | --- | --- |
| `IDLE` | start, finish, cancel, error timeout | `ARMED` on shortcut down | dot quiet, "Parlo ready" |
| `ARMED` | shortcut down | `LISTENING` when the first audio block arrives (≤100 ms), `IDLE` on immediate failure | "Opening the microphone" |
| `LISTENING` | audio flowing | `PROCESSING` on stop or timeout, `IDLE` on cancel | level meter, clock, mode |
| `PROCESSING` | stop | `INSERTING` on text, `ERROR` on failure, `IDLE` on cancel | "Processing", elapsed ms |
| `INSERTING` | text ready | `SUCCESS` on acceptance, `ERROR` on refusal | "Inserting" |
| `SUCCESS` | insertion accepted | `IDLE` after 1.6 s | the text, briefly |
| `ERROR` | any failure | `IDLE` on dismiss, retry, or 8 s | one sentence and one action |
| `CANCELLED` | Escape | `IDLE` at once | nothing, no history entry |

Rules: a state transition never depends on the pointer. Every state is reachable by keyboard alone. `CANCELLED` exists so the code has one place that means "throw it away", which today it does not.

## F. Screen by screen

### Home

Goal: answer "is it ready" in one glance. Content: readiness line with the reason when not ready, the shortcut in a keycap, mode (dictating or translating), language pair, active model, voice profile, microphone level of the last check, and the last three results with the application they went to. Primary action: none, because the product is the shortcut; one secondary: "Copy the shortcut" or "Change it". Empty: first run shows onboarding, not an empty Home. Loading: skeleton lines, never a spinner over the whole page. Error: readiness line turns into the fault and one button that fixes it. Permission: microphone denied shows the exact Windows path to allow it.

### Library

History: search field, filters by application and mode, rows with time, application, mode, first line, duration. Actions per row: copy, insert again, edit, favourite, delete. Empty: "Nothing dictated yet. Press Ctrl+Alt+D and speak." Loading: three skeleton rows. Error: "History could not be read" plus the folder it tried.

Models: installed, available, active, size on disk, measured latency of the last run, offline yes. Download and update with size and disk left. Vocabulary: current pairs, add, remove, with the sentence about staying on this machine.

### Meetings

Tabs: Sessions, Memory. Sessions list with title, date, duration, speakers. Detail: transcript with timestamps, summary, decisions, actions, export. Memory: search across every transcript. Before-start panel: sources, language, model, title. Errors name the source that refused.

### Voice

Profile list, quality, samples, test sentence, revoke, delete. Explicit statement that the profile stays in a folder on this computer. When the voice engine is absent, the page says so instead of offering a dead button, which is what it does today with a note under a working form.

### Settings

Eight sections, each with a title, one sentence of purpose, fields, and Apply only when something changed. Shortcuts shows conflicts before saving, with the program that owns the combination when Windows reports it. Privacy states what leaves the machine and what does not, and holds the cloud key. Advanced holds licences, logs, the engine path and the diagnostics export.

## G. Overlay specification

Geometry: one row, 560 × 56 logical pixels, centred horizontally on the monitor holding the cursor, 72 px above the bottom edge, above normal windows, never focusable, never in the taskbar, none of it in screen recordings unless the user enables that.

| State | Dot | Line one | Line two | Duration |
| --- | --- | --- | --- | --- |
| Ready | outline | Parlo ready | Ctrl+Alt+D | while idle, dims to 55% after 3 s |
| Listening | accent, pulsing | Listening · dictating | 0.0 s, level meter | until stop |
| Processing | accent, spinning | Processing | elapsed ms after 600 ms | until text |
| Inserting | accent, solid | Inserting | — | until accepted |
| Inserted | green | Inserted | first 48 characters | 1.6 s |
| Error | amber | one sentence | one action, e.g. Retry | 8 s, then Ready |
| Cancelled | none | — | — | closes immediately |

Motion: 180 ms settle on appearance, 120 ms state cross-fade, 240 ms exit. Reduced motion: no pulse, no settle, cross-fade only. No shadow blur above 40 px, because it costs a frame on integrated graphics.

Multi-monitor: follow the cursor, not the primary. Fullscreen and games: if the foreground window is exclusive fullscreen, the island stays hidden and state is reported in the tray tooltip; during presentations, an option pins it to the presenting display or keeps it out of the captured output entirely.

## H. Shortcut system

Defaults: `Ctrl+Alt+D` push to talk, `Ctrl+Alt+Shift+D` hands free, `Ctrl+Alt+C` captions, Escape cancel. All four rebindable, all four stored as parsed key plus modifiers, never as a string to be interpreted at run time.

Down and up are both handled, with a repeat filter so a held key does not restart a session. A tap under 250 ms means hands free, a hold means push to talk, and the overlay states which one is happening. Conflicts are detected by asking Windows to register the combination before saving, and the error names the combination, not an error code. Accessibility: every action reachable from the tray menu without a shortcut, and a "no shortcut" mode for dictation started from the tray.

## I. Error system

One sentence, plain, and the next action.

| Situation | Message | Action |
| --- | --- | --- |
| Microphone blocked | Parlo cannot reach the microphone. Windows blocked it. | Open the microphone setting |
| No microphone | No microphone is connected. | Choose another input |
| Microphone busy | Another program is using the microphone. | Try again |
| No speech | Nothing was heard. | Speak, then stop |
| Too short | That was too short to write down. | Try again |
| Transcription failed | The words could not be recognised. | Retry, keep audio |
| Model missing | The dictation model is not downloaded. | Download it, with the size |
| Insertion refused | The field did not accept the text. It is on the clipboard. | Paste by hand |
| Engine down | The engine stopped answering. | Restart Parlo |
| Cloud refused | The cloud provider refused the request. | Check the key |

Every message states what happened and what to do, never a status code alone, and never "something went wrong".

## J. Onboarding

Under sixty seconds, five steps, no feature tour.

1. Language pair, two selects, remembered.
2. Microphone permission, requested once, with the exact place to allow it if refused.
3. Shortcut, prefilled, with a keycap that can be pressed to test.
4. One sentence spoken: "Say a sentence, then press Enter." The text appears in the onboarding field itself, so the insertion path is proven, not described.
5. Done: Home shows Ready, and one line says where the shortcut is and what Escape does.

Nothing about meetings, voice cloning, models or cloud in onboarding. Those are discovered later, on the pages that own them.

## K. Product rules

1. Never require the window to dictate.
2. Never move focus to the island.
3. Stop inserts, Escape discards. Two keys, two verbs.
4. If insertion is not certain, say where the text is before saying it was inserted.
5. Show the same three states on every surface.
6. One screen, one question.
7. Configuration that a person touches once lives in Settings, not on Home.
8. Every error names a next action.
9. Every measured claim in the app carries its conditions.
10. No feature gains a permanent surface without a daily use.

## L. What changes, in order

Ordered so each step ships something a person can feel, smallest first.

1. **Escape cancels, losing focus does not submit.** Island only. Small, and it removes a way to lose a sentence.
2. **Push to talk on key up.** Handle the hotkey release, distinguish tap from hold. This is the change that makes the product what it claims.
3. **One state machine in the engine**, read by island, tray and window. Ends the three surfaces disagreeing.
4. **Island knows the insertion target before it speaks** and reports readiness to insert; clipboard becomes the documented fallback.
5. **Home becomes status**, the recorder moves to onboarding, sections stop sharing one scroll.
6. **Library**: History gains application, mode and duration; Models and Vocabulary move beside it.
7. **Tray**: modes and destinations, no duplicates, state in the tooltip.
8. **Settings split into eight sections**, clouds and keys under Privacy.
9. **Captions as its own floating window**, then Live, then Meetings with a lifecycle.
10. **Shortcut system**: rebinding with conflict detection, four defaults, accessibility path.

## Comparison

Honest, by what exists in this repository today.

| Capability | Handy | Wispr Flow | Parlo today | Target |
| --- | --- | --- | --- | --- |
| Push to talk | Yes | Yes | Toggle, not hold | Yes, step 2 |
| Hands free | Partial | Yes | No | Yes, step 10 |
| Global shortcut | Yes | Yes | Yes | Yes |
| Contextual insertion | Yes | Yes | `SendInput`, no target check | Yes, step 4 |
| Floating UI | Minimal | Flow Bar | Island exists, wrong trigger | Yes, step 1–3 |
| Cancel without inserting | Likely | Yes | No, Escape inserts | Yes, step 1 |
| Smart formatting | Basic | Strong | Rules only, cloud optional | Yes, rules first |
| Context awareness | Little | Strong | None | Later |
| Meetings | No | Yes | Import text, record mic | Yes, step 9 |
| Captions | No | Partial | No | Yes, step 9 |
| Live | No | Partial | Yes, page-bound | Yes, step 3 |
| Voice profile | No | No | Form only, engine missing | Yes |
| History | Basic | Yes | Yes, no application or mode | Yes, step 6 |
| Model choice | Basic | No | Yes, richer | Keep |
| Offline | Yes | No | Yes | Keep |
| Privacy | Local | Cloud | Local, cloud opt-in | Keep, state it |
| Customization | Settings | Settings | One long page | Yes, step 8 |
| AI actions | No | Partial | Cleanup only | Later |
| Cross-platform | Yes | mac, Windows | Windows primary | Later |

Where Parlo is better today: offline translation with a chosen model, no account, no telemetry, and an engine that can be swapped. Where it is worse: the shortcut does not behave like a dictation key, Escape commits, and the window is a recorder. Those three are the whole of step 1 and 2, and they matter more than any page below them.

## Measured, and targets

Measured on the reference machine, same models: dictation of a 6.33 s clip in 647 ms; live mode confirms its first stable words after 1.0 s of audio and costs 4.7 s of work for that clip.

Targets, not yet measured: shortcut to audible capture under 100 ms, overlay visible under 80 ms, stop to inserted text under 1.2 s for a 10 s sentence, no frame over 16 ms while the island animates.
