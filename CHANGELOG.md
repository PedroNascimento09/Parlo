# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org).

## [Unreleased]

- A meeting keeps its decisions, and a later one replaces an earlier one on the same topic. A line that closes a decision ("combinado", "decidimos", "aprovado", "agreed") is kept with its speaker; when a later line shares a content word with it, the later line is the decision and the page says which one it replaced, so a meeting that changes its mind reads as one decision instead of two. A hedge ("talvez", "acho que", "maybe") is not a decision. **Status: deterministic rules with tests; it judges wording, not meaning.**
- A meeting names its action items. Every line of a transcript is read for a promise ("vou", "fica de", "envio", "I'll", "will send" and the rest of a fixed table), the owner is the speaker or the person named right before the verb, and the due phrase is the one the line used ("amanhã", "sexta", "até dia 5", "tomorrow"), shown beside the meeting with no date invented. A denial ("não vou") is not a promise, and the same sentence is kept once. **Status: deterministic rules with tests, no model and no reading of the meeting; it is not on the level of a human summary.**

- A moved model folder is adopted only when every file matches byte for byte. The comparison used names and sizes, so a file of the same length but different content counted as a copy; it now reads both files in 64 KB blocks and refuses at the first difference, and an empty file never counts as a match, so a truncated copy is copied again instead of adopted.

- A translation pair exported to a folder now installs from the command line. `parlo pair <folder> --from pt --to de` checks the seven files the direct export needs, names the ones that are missing instead of failing inside the decoder, copies them into `models/mt/pt-de-onnx` and refuses a destination that already holds a pair. **Status: the check and the copy are covered by tests; no export is published, so nothing here was run against real weights.**

- An interrupted model download resumes where it stopped. The bytes already on disk stay in the `.part` file and the next run asks the server for the rest with `Range: bytes=<n>-`, hashes the whole file again and only then installs it; a server that ignores the range starts the file over, a part larger than the published size is dropped, and a part that already holds every byte is checked without a request. A downloaded file that ends short of the published size is still refused and deleted, so a wrong total never looks like a resume.

- The island shows the words while they are being said. Every 800 ms it sends the audio captured since the last window to `POST /listen/stream`, which feeds the engine's streaming session and answers with `partial`, `stable` and `final` pieces, so the line under **Listening** grows instead of staying empty; stopping or cancelling closes the session through `/listen/stream-end`. **Status: wired end to end, not run against a microphone here.**

- Dictation can write into another window on Linux and macOS. On X11 the text goes through `xdotool type` after checking that the same window is still in front, on macOS through System Events after checking that the same program is still frontmost, and Wayland refuses with a sentence that puts the text on the clipboard instead of pretending. **Status: implemented, not verified on either system here; the clipboard fallback is unchanged.**

- A failed translation no longer loses the sentence. When the translation stage cannot run, the engine keeps the words it heard and answers with them as the text and an empty translation, so the island inserts what was said instead of reporting an error and dropping it; the reason goes to the engine log. **Status: covered by a test, not run against a live translation failure here.**

- The window is reachable from the keyboard alone: **Alt+1** to **Alt+5** open Home, Library, Meetings, Voice and Settings, **Alt+Left** goes back within the app, the rail names each shortcut to a screen reader, and the section someone was on is remembered, so reopening the window returns there instead of starting at Home.

- Every dictation in **Library, History** can be acted on. **Favourite** marks an entry and a **Favourites only** filter keeps the marked ones, **Correct** replaces the sentence Parlo wrote down, and **Delete** removes that entry alone after asking; each entry carries an id the local server knows, including lines written before ids existed, each row also names how long the audio was, and every change stays in the history file on this computer. Home lists the last five results with a link to the rest, and names how many models are installed, their size, the space left on the disk that holds them and a **manage** link.

- Recording a meeting captures audio. The page fed the recogniser only while a dictation or a voice sample ran, so a meeting kept no lines at all; it now captures for as long as the meeting runs, and the same screen gained the middle of its lifecycle: **Pause** stops what is written down without ending the transcript, **Resume** carries on with the same meeting, every line's timestamp leaves the paused time out, and a **Note** typed while recording is kept at that moment, listed under the meeting and carried into search, questions and the Markdown copy. **Status: implemented, not yet run against a microphone on this machine.**

- Read aloud speaks German and French: the German and French voices are in the library with their own Download button, the engine picks the voice by the language it is reading, and the download fetches that voice alone. A German translation is now spoken in German instead of falling back to the Portuguese voice. **Status: wired end to end, audio not heard here.** The files, units and mapping are tested; nobody has listened to the two voices on this machine yet.
- `docs/MODEL_MANAGER.md` lays out the folder an exported pair needs, its tag table and its refusals, and points at `engines/cascade/scripts/export_pair.py`, so the German pair is reachable on a machine that builds it. The status is written beside it: implemented, not verified in an installed build.
- The engine can load a direct ONNX pair. A folder named `<source>-<target>-onnx` under `models/mt` with the exported encoder, decoder, decoder-with-past, vocabulary and sentencepiece files is used before any translation package, with the language tag read from a table; the decoding loop is tested, and the runtime path is **implemented, not verified** because no weights are installed here.

- Portuguese to German is no longer pivot-only on paper: an int8 ONNX build of a Romance-to-German checkpoint passes the gate's 20 sentences directly — numbers, clocks and negations 1.000, names 0.850, 601.6 ms median against 0.79 s for the two-stage route — and three sentences the pivot broke come back right. Nothing ships from it yet: the engine has no ONNX path, so the pair is measured, not installable.

- `parlo doctor` ends with a **first session** check: every model on disk, the recogniser Settings chose, the shortcut, the settings file and a writable data folder, each pass or fail in one line, and it exits 1 while anything is missing.

- The plan stops waiting for users it does not have. G2 is now the author's own first session on a clean install — models, one sentence, one dictation, reopen — and the external-user round starts when there is a first user; every release still states plainly what has not been validated.

- Captions can be saved. A session keeps the lines it showed, and **Save** writes an SRT with timing into the data folder and names the file it wrote, so a meeting can be read afterwards without another tool.

- A model keeps the copy it replaced. When a model is downloaded again, the build in place is set aside beside it, and a **Restore** row puts it back if the new copy is worse; nothing is deleted until the new download finishes.
- Moving the models says what it costs before it copies. The first press answers with the megabytes that will move and whether the target already holds an identical copy, and only a second press copies anything; a target that already matches is adopted instead of copied again.
- The first start names the recogniser it recommends with the measured speed behind the choice, instead of memory alone.

- The model library is a catalogue by purpose, not a checklist. Rows are grouped into **Speech to text**, **Read text aloud**, **Translation** and **Speech synthesis**, the catalogue names more recognisers and voices than this build installs, and a model that cannot run here says why instead of offering a button. Only what ships or is claimed is measured, so a new row no longer waits for a benchmark of its own.

- The first start ends with the next step instead of a silent change of screen. When a recogniser arrives, the page says `Dictation is ready.` and names the shortcut to hold, taken from the settings, so nobody is left looking at a window with no instruction.

- Whisper can be removed like anything else: the row deletes the weights from the Hugging Face cache, and a download still blocks removal. The first-start choice also stops resetting itself every time the model list refreshes, so a tick someone made stays ticked.

- Whisper base joins the library as a second way to dictate: 145 MB on disk, 148 MB in memory, 0.24 of real time and 0.377 word error on the same recordings, downloaded and removed like any other model. It counts as a dictation model only when Settings selects it, so readiness never points at a model that is not there, and the first start offers it when Parakeet's 746 MB would leave less than 700 MB of headroom on this machine.

- The first start explains itself with measured numbers. Instead of a bare suggestion, each feature states the memory and speed of the model behind it on this machine, so the line reads like `671 MB to download · suggested here · Parakeet TDT 0.6B v3: 746 MB in memory, 0.21 realtime here`.

- The library shows what each model costs on this machine. A new `engines/cascade/scripts/model_memory.py` loads one unit, warms it, then reports the memory it added and how fast it ran: Parakeet 746 MB at 0.21 of real time, Whisper base 148 MB at 0.24, each voice about 2 MB, each translation pair about 112 MB at 278 and 510 ms per sentence. Rows carry those numbers beside the size on disk, measured here rather than quoted from a model card.

- The first start asks before it downloads. Each feature is offered with the size of the models behind it, the ones this machine's memory suggests are already ticked, and one button fetches only what is ticked through `POST /models/download` with the units it was given. A machine with 8 GB is offered dictation, 8 GB or more adds the translation pair, 16 GB adds the voices, and nothing is fetched until someone presses the button.

- The model library shows what exists before anything is installed. Every component the build knows carries its name, what it does, the languages it covers, the size measured on this machine or a plain **size not measured**, its licence and its measured quality where one exists, and the two translation pairs can now be downloaded from that list. A row fetches one model and nothing else, and a component this build cannot install says so instead of offering a button that would fail.

- Streaming keeps its windowed pipeline, and the decision is written down. On recorded speech Parakeet holds a real time factor of 1.023 with flat drift across the run, so the current design meets the latency target on the reference machine. Both native streaming candidates are blocked: `nvidia/nemotron-3.5-asr-streaming-0.6b` carries OpenMDW-1.1, which the licence gate does not approve, and its RNNT weights have no decode path in this repository's transformers build, while `Qwen/Qwen3-ASR-0.6B` streams only through vLLM and its Portuguese tag does not separate Portugal from Brazil. `docs/ENGINES.md` records what would change the decision.

- The last two documents the plan makes mandatory exist. `docs/MODEL_POLICY.md` states which licences may enter the product, what a quality claim needs before it is published, and how a model is added; `docs/PLUGIN_DEVELOPMENT.md` records the local API as the extension point and the contract a plugin will have to meet, and says plainly that no plugin runtime ships today.

- Streaming is measured on recorded speech, not only on a synthetic voice. `stream_bench.py --audio-dir` reads a folder of 16 kHz recordings with a manifest, and `scripts/falar_clips.py` builds that folder from a FalAR shard, keeping the segments the corpus scores at or below 0.05 character error rate. On ten segments of Portuguese parliamentary speech (66.6 s, four speakers), Parakeet reaches 0.243 final word error rate with 84.4% of the sentence confirmed when the segment closes and a real time factor of **1.023**; Whisper base reaches 0.377 and **3.274**. The harness had been ignoring `--stt`, `--model` and `--threads`, so both recognisers were measured as the default one; it now applies all three before it builds the engine.

- Meetings answer questions from the transcript alone. **Ask this meeting** and **Ask every meeting** post to `/meeting/ask` and `/memory/ask`, and the answer is assembled only from stored lines: every answer carries the meeting, the speaker and the time behind each line it used, and a question the transcript does not answer gets one sentence saying so instead of a guess. Deterministic rules, with no model and no network, and a decision is only called a decision when the line closes it.
- The release downloads like a program. The Windows installer is one fixed file, `Parlo-Setup.exe`, so `releases/latest/download/Parlo-Setup.exe` always fetches the current build, and setup opens Parlo as soon as it finishes.
- `parlo app` opens the Parlo window and nothing else. The browser is no longer a way in, so the local address and the microphone prompt attached to it are gone; the window answers the microphone permission itself. Options passed to `parlo app` are saved and the window uses them, and `--port` now points at `parlo serve`, which keeps the local api.
- Escape stops a surface. Live, Captions and a running meeting all end the capture on Escape instead of leaving it running behind another window, and the button that used to say **Pause** says **Stop**, because it ends the capture rather than holding it. The header of each window says the same in a sentence: `Live · listening, every 2.5 seconds · Escape stops`.
- A model can be loaded and unloaded by hand. Every row of the model list has **Load** or **Unload now**, so a model can be brought into memory before a meeting or freed before another program needs the RAM. Unloading refuses with a sentence while a task is running, and it never deletes anything: the files stay on disk.
- Each model decides when it is freed. Its row carries a delay from ten minutes to four hours, or **Keep loaded**, and says whether that delay is its own or the global one. The setting lives in `config.json` as `policies`, which the engine reads at every check, so nothing has to be reloaded. Settings gained the switch that turns automatic memory management off, and the delay is greyed out while it is off.
- The engine measures inactivity per stage instead of for the whole process, so a recogniser that ran for an hour no longer keeps the voice in memory. `GET /models` carries `idle_seconds`, `policy` and `policy_origin`, taken from a new `lifecycle` op on the sidecar protocol, and the row reads `Loaded`, `Installed · unloaded`, `In use` or `Not installed` in the words the plan fixes.
- `POST /models/load/<id>`, `POST /models/unload/<id>` and `POST /models/policy` are the routes. A model whose files are missing is refused with its name, and an unknown id with 404.

## [0.2.2] - 2 November 2026

- Two more keys are settings of their own: `Ctrl+Alt+Shift+D` stays listening without holding anything, and `Ctrl+Alt+C` opens or closes the captions window, beside the existing dictation key. All three are validated when they are saved, refused when they repeat one another, and refused when Windows says another program already owns the combination, with a sentence that names the keys instead of an error code.

- The dictation island opens on the screen the cursor is on, not on the primary one. Its place is the bottom centre of that monitor, and a machine that reports no monitor under the cursor still falls back to the primary one.

- The window stopped starting modes. Home kept its own Live and Dictate buttons, which the tray already offered, so both are gone with the code behind them: Live is the tray entry and the window served from /live, Dictate is the island the shortcut opens. The hold-to-speak button stays as the first-run test and translates one clip.

- The tray lists every destination instead of one door. Home, Library, Meetings, Voice and Settings each open the window on their own screen, so Open Parlo is gone: it did what Home now does. The three modes stay above the separator, the tooltip still repeats the shared state, and the left click still shows the island.

- Onboarding asks for the three things it needs before it proves anything: the pair, the microphone, with the exact Windows place to allow it when it is blocked, and the shortcut, captured from the keys the person presses. Then it waits for one real sentence and reads it back with the program it landed in, and the last line says where the shortcut is and what Escape does. Five steps, every one of them saving a setting that exists, and skipping still stops it asking again.

- Settings is eight sections instead of four. **Dictation** holds the three feature switches, the recogniser and the idle timer, **Languages** the pair and what dictation writes, **Shortcuts** the keys, **Cleanup** the engine and how far it may go, **Provider** the cloud or local model it may call, **Privacy** the key and the sentence about what leaves the machine, **About** the version on this copy and the check for a newer release, and **Advanced** the licences. One Apply sits under every section, so a change is never left waiting behind a tab.

- The cloud key has its own section. **Settings, Privacy** says in one sentence what leaves the machine and when, and holds the API key, the button that removes it and the test sentence that proves the provider answers, all three moved out of the cleanup section where they had been left.

- Live has a surface of its own. The tray item opens a window with two columns, what the microphone heard on the left and the sentence in the other language on the right, always on top and away from the taskbar, instead of raising the main window and starting live inside it. The page is served at `/live` and refuses with a sentence when translation is switched off in Settings. Captions and Live now share one toggle, so the two windows cannot drift apart.

## [0.2.1.5] - 2026-11-02

- The model list says **In use** while the engine is working, without asking it anything. The session keeps the last answer the engine gave about what it holds and marks the recogniser row as busy for as long as a transcription runs, so the page reads the list in a millisecond instead of waiting for the engine it is describing. Only the recogniser is marked: every transcription and every translation goes through it, while the translation pair depends on what the request asked for. Measured with the real engine: a ten second silence sent to `POST /dictate` came back in **2.47 s**, and `GET /models` answered `in use` at the first poll, 0.2 s into that request, then `loaded` once it finished.

- The local server answers one connection per thread, so a request no longer waits behind every earlier one: a health check came back in 3 ms while another request was in flight, and a static file in 3.6 ms. The chosen languages and shortcut sit behind a read lock and the engine behind a mutex, with a test that fails if the session ever stops being safe to share between threads. The engine still transcribes one request at a time, so `In use` stays absent: the model list would wait for the engine it is trying to describe.

- Home answers what happened last: the three most recent dictations sit under the status with their time, their language pair and the program they landed in, so the first screen says what the app did instead of only what it is doing.


## [0.2.1.4] - 2026-11-02

- The model list says whether a model sits in memory. `GET /models` answers with `state` on every row, `not installed`, `installed` or `loaded`, by asking the engine what it holds through a new `loaded` op in the sidecar protocol, which the bundled engine and the Python one both answer. The page shows `In memory` beside a model that is loaded and `On disk` beside one that is downloaded but not loaded, so the two no longer read the same. A translation pair reads as loaded only for the direction the engine is set to, because the engine holds one direction at a time. `In use` is not shown: the local server answers one request at a time, so no page can read the list while a model works.
- Every model row states the quality number measured for it, or says there is none. The recogniser carries 0.24 word error on 20 European Portuguese sentences, Windows 11, 4 threads; each translation pair carries its gate result for numbers, clocks, negations and names; the voices say no number was measured, because none was. `docs/MODEL_MANAGER.md` records the same, and a test fails if a row ever comes back without a sentence.
- The README names the version people can download, says what the model rows show, and states two limits beside the rest: read aloud is a neutral voice no panel of listeners has judged, and `In use` does not exist because the server answers one request at a time.

## [0.2.1.3] - 2026-11-02

- Read aloud speaks European Portuguese. The voice Parlo installs is `pt_PT-tugão-medium` instead of the Brazilian `pt_BR-faber-medium`, because on this project's own 20 sentences the European voice scored 0.24 word error rate against 0.67 for the Brazilian one, measured with Parakeet v3 on Windows 11 and 4 threads. Models already on disk are not moved: the list offers the European voice as a separate download, and `setup_models.py --voice pt_br` still fetches the Brazilian one.
- The reference battery takes the voice as an option, `--voice`, and records it in the report beside the recogniser and the thread count, so a number is never read against the wrong voice.
- Removing a voice or a translation pair switches the feature that needed it off and says so where the person did it: the list names the switch in Settings and the model to download again, and `POST /models/remove/<id>` carries the same sentence. The recogniser has no switch to turn off, because dictation exists only while its files are on disk.
- Every text pair in the window clears 4.5:1, measured on the real colours. The accent is no longer set as text: it measured 4.29 on `surface` and 3.98 on `canvas`, so links and the hover words on a button use `accent-deep`, which measures 6.68 and 6.20; the accent keeps the fills and the borders it had. The button ink goes to `#120a05`, 4.57 on the accent, from 4.33. The warning colour is defined in both themes and used by nothing in the interface, so the 3.79 it measures on `surface` has no reader.

## [0.2.1.2] - 2026-11-02

- The type follows section 85: one interface family and one mono family as tokens, the body at 14/21 instead of 16/1.6, and digits set tabular so timings do not dance while recording. The two large headings drop to the display and title scale. Colour pairs are unchanged, so the ratios already measured still hold, and every body size now sits in the stricter 4.5:1 band that `ink-soft` at 11.84 and `muted` at 5.98 meet.
- The interface takes the tokens from section 85 instead of its own palette: the same fifteen colours in both themes, `surface-sunken` and `tech` included, and one focus ring drawn with the accent on everything that can take the keyboard. Measured on the real pairs: ink on canvas 16.65, ink-soft on surface 11.84, muted on surface 5.98, muted on canvas 5.55, error on canvas 6.66, ok 4.85, listen 5.50, tech 11.01; dark reads 16.44, 11.22, 6.47, 5.80, 8.31, 7.95 and 8.64. The one pair below 4.5 is the plan's own `warn` on the light canvas, at 3.51, and it is used only for large text, icons and fills.
- Translation, read aloud and captions can be switched off without deleting a single model. The switches live in Settings and are saved in `config.json`; with captions off the captions page refuses with a sentence, and with translation off dubbing refuses instead of quietly doing half the work. `GET` and `POST /features` are the routes.
- The engine says what this machine can do and why. `GET /engine` reports the profile, the voice that was asked for, the voice that is used instead, the chain of steps, and a sentence that names the cores, the memory and AVX2; every change is appended to `logs/engines.jsonl`, and the Home screen shows that sentence under the engine. Nothing in it claims a latency that was not measured.

## [0.2.1.1] - 2026-11-02

- The update box only offers **Install update** when the release carries a build for this platform. `/update` now reports whether an archive exists, and a release without one shows the sentence and the link to what changed instead of a button that did nothing.
- The models can live on another drive. **Keep the models in this folder** copies the whole tree, compares the copied bytes with the original, and only then deletes the old folder and saves the new path; a copy that does not match leaves both in place and says so, and a relative path or a folder inside the old one is refused before anything moves.

## [0.2.1] - 2026-11-02

- The translation pairs are in the model list. **Portuguese to English** and **English to Portuguese** each show their size on disk and their own Download and Remove buttons, detected from both layouts the engine uses (`mt/en-pt/ct2` and `mt/translate-pt_en-1_9/model`), so what the list shows is the file that is actually there.
- A model download can be stopped. **Stop the download** appears in the list while one runs and ends the process, the list reports **downloading** from the process itself rather than from a clock, and what already arrived stays on disk; starting again continues from the files the download tool can reuse, and the page says only that much.
- The model list says why each model is the one offered, with the machine it measured: cores, memory and whether AVX2 is there, read at the moment the page asks. Nothing about speed is claimed, because nothing was measured for the new recommendation.
- The sample can be sealed with the Windows account that made it. A profile created with **Keep the sample sealed with my Windows account** stores it as DPAPI ciphertext, the list marks it as sealed, and playback and export unseal it on demand; a sample another account wrote is refused with a sentence instead of failing silently.
- A voice profile can carry a real sample. Ten to thirty seconds are recorded in the window or refused with the measured length, the silent take is refused too, and the profile can then be heard back, exported as one file that marks the voice as synthetic, or deleted with its sample; the engine that turns the sample into speech is still not in the build, and the panel says so.
- The first start offers the recogniser alone, about 640 MB, instead of asking for the whole set of models at once; translation and read aloud are added later, one model at a time.

## [0.2.0] - 2026-11-01

### Changed

- The download is a third of what it was. The engine bundle carried a 310 MB `torch_cpu.dll`, pulled in because the licence bench installs torch in the same environment, and a 47 MB `transformers` copy that faster-whisper lists only as an optional extra; neither is imported anywhere on the dictation, translation or voice path, and both are now excluded from the package and filtered out of what the packaging step collects. The Windows installer went from **231 MB to 44.8 MB** and from **655 MB to 175 MB** installed, with the same features.

### Added

- Models install and leave one at a time. Every row of the list names the feature it serves, its size on disk and its own **Download** or **Remove** button, so a machine can carry the recogniser alone and nothing else; removing frees the space at once, refuses while a download is running, and leaves history, meetings, vocabulary and settings untouched. `POST /models/download/<id>` and `POST /models/remove/<id>` are the routes, and the engine accepts `--only parakeet` so the same split works without the window.
- Dictation can be the first thing installed. **Only what dictation needs** downloads the recogniser by itself and the page follows that one download rather than waiting for the whole set, so speaking works minutes after opening Parlo for the first time while translation and voices stay available but optional. `POST /models/dictation` is the route and `dictation` in `/models` tells the page when it is done.
- The model list says what is missing and who needs it: each gap is written with the feature that depends on it, so a missing voice reads as «Read aloud needs the Portuguese voice» instead of a list of files.
- Models are freed when nothing uses them. A watchdog in the engine asks every minute whether the loaded recogniser, translation, dictation translation and voice have been idle past the delay, drops them when they have, and says in the log what it dropped; the next dictation loads them again. The delay is a setting — 10, 15, 30 minutes, one, two or four hours, or **Never** — written to `config.json` as `unload`, read at every check rather than at startup, so changing it needs no restart, and a machine with the file absent keeps them for 30 minutes. The watchdog holds the same lock as the requests, so a model is never freed in the middle of a dictation.
- Parlo learns the words a person actually uses. **Words Parlo should know** keeps two local lists — names and terms the recogniser keeps getting wrong, and short forms that should expand — and the intelligence engine applies them to the cleaned sentence, on whole words only, before any model is asked and before anything leaves the machine. Each substitution is recorded in the changes beside the cleanup ones, a regular expression never matches, and the words live in `vocabulary.json` in the data folder: no account, no server, nothing sent. The two sides are refused when empty, longer than 64 characters, or of a kind Parlo does not know.
- A meeting can be recorded instead of only imported: **Record a meeting** takes the name from the same field, keeps the transcript in `meetings/` with the moment each line was said, and appends line by line as the recogniser closes them, so a crash costs the last window and not the hour. The microphone is what Parlo hears, so the sound of another application is not captured yet and every line is filed under `Me`; naming a speaker still works on top of that. `POST /meeting/start` and `POST /meeting/segment` are the two routes behind it, and a meeting with no title is called `Meeting`.

## [0.1.9] - 2026-11-01

### Fixed

- Dictation in the app was asking the engine for the other language. `/dictate` passed the request as it arrived, so an empty «Dictation writes» reached the engine as `pt` to `en` and came back translated: the island inserted the English instead of what was said, and a machine without the translation package failed outright. The endpoint now resolves the target first — the language spoken unless another one was asked for — and the change is covered by a test on that resolution.

### Added

- The dictation island now asks for the latest release when it opens, through the same `/update` route the window uses, and says «v0.1.9 is available, you are on v0.1.8. Open Parlo to update.» in its hint. The island still installs nothing on its own: installing stays a click in the window, and a failed or rate limited lookup shows nothing rather than a wrong version.

## [0.1.8] - 2026-11-01

### Documentation

- `PARLO_GLOSSARY.md` names the parts of the product once, with the rule that keeps the names honest: PARLO is the root, each engine carries a two-to-four-letter sigla, and a sigla is never reused. **PAVE** (audio and voice), **PSTE** (speech to text), **PTE** (translation), **PIE** (intelligence), **PIMO** (model orchestration) and **PME** (memory), each with its responsibility, dependencies, the code it lives in today and whether it is implemented, tested or validated. Surfaces keep plain names, and no code is renamed for the sake of the name.
- The master plan gains **Part XVIII** (identity) and **Part XIX** (the intelligence engine): the naming rules, the routing order of `auto`, the deterministic-first rule, the optional loopback model, the key storage per system, the validation that refuses an edit changing meaning, and the list of what is still missing.
- The master plan gains **Part XVI**, sections 114 to 122: model memory and lifecycle. Four states (`Not installed`, `Installed · Unloaded`, `Loaded`, `In use`), an unload policy measured from the last real use with `Never`, 10 and 15 minutes, 30 minutes, 1, 2 and 4 hours and `Automatically`, a global setting with 30 minutes as the recommendation, a per-model policy that overrides it, a rule that a running task suspends the unload, the Resource Manager as the only component that loads and unloads, the Model Hub states and actions, and the separation of `Unload` from `Remove`. The principle is stated once: Parlo manages, the user decides.
- The master plan moves to **v12.5** with a new Part XV: model selection that recommends without blocking, Parlo Meetings as a pillar with transcript, speakers and retroactive naming, a post-meeting surface with decisions kept distinct from proposals, Ask Parlo with citations, meeting memory between sessions, privacy per stored item, four support states per call platform, and three implementation layers. The appendices now come after Part XIV, where the index always said they were.

### Added

- The app has a **How dictation is cleaned up** panel: the cleanup engine (rules only, a model on this machine, a cloud provider, or automatic), how much may change, the provider and model, an optional local model address, the allow-cloud switch, the key field, and a button that sends one sentence to prove the setup works. The settings live in the config file and are read on the next dictation, so changing them costs no model reload; a plain `http://` address is accepted only for a server on this machine, and the compatible provider needs its own address instead of borrowing the local one.
- A key entered in the app goes straight to the operating system's credential store through the engine and is never written to the config file, never logged and never shown again; the panel says which providers have a key and lets one be removed. The engine gained `parlo-engine --intelligence` for this, one request per action, reading from standard input and answering with no secret in the reply.
- PIE, the intelligence engine behind dictation, now decides where a sentence is edited instead of always using the rules: `off` runs the deterministic cleanup alone, `local` adds a model on loopback, `cloud` adds the provider the user authorised, and `auto` tries local, then an authorised provider with a key, then the rules. Cloud is never reached without `PARLO_INTELLIGENCE_CLOUD=1` and a stored key, so an unconfigured machine behaves exactly as before. OpenAI, Anthropic, Google Gemini and any OpenAI-compatible server are prepared, each with an injectable transport so the tests never touch the network.
- API keys are stored in the operating system's credential store and nowhere else: DPAPI on Windows, the keychain on macOS, the secret service on Linux. With none of them Parlo refuses to store the key rather than writing it in clear, an environment variable is read first and never written, a key is never shown again, never logged, and never put in an error message. Requests go over HTTPS or loopback only, directly from the machine to the provider, with no Parlo server in between.
- A model's answer cannot be used until it passes validation: JSON with a `text` field, no number that was not said, no invented proper name, no growth past 1.6×, no shrinkage below 0.4×, no commentary. Anything else falls back to the deterministic text with the reason recorded, so an invalid key, a rate limit or a timeout costs a nicer sentence, never the sentence.
- Dictation now writes the sentence the person meant instead of every word they said: a correction after «quer dizer», «afinal», «não, espera», «não, melhor», «desculpa, queria dizer», «corrigindo» or «ou melhor» replaces what came before it, an abandoned sentence start disappears, hesitations and accidental repetitions are dropped, and nothing else is touched. It runs after recognition and before insertion, on rules alone — no model, no network, microseconds — and the engine takes `original`, `light` or `intelligent` from `PARLO_CASCADE_CLEAN`, with `light` as the default. What came before is decided by overlap, not by the marker: when the second half adds content («Gosto de cinema, quer dizer, gosto especialmente de filmes de ficção científica») both halves stay and only the marker goes, so a clarification is never mistaken for a correction. The sentence that was heard is kept beside the cleaned one. Tested on the PT-PT and English cases in `engines/cascade/tests/test_corrections.py`; not validated with real recordings, and the switch between the two texts is not in the interface yet.
- The dictation shortcut is a setting instead of a constant: `parlo config --shortcut "Ctrl+Alt+Shift+D"` writes it, the window registers whatever the file names and reports a shortcut another program owns, the island shows the same keys in its hint, and the settings page validates and stores it. A restart is what makes the new shortcut live, and Parlo says so rather than pretending the change took effect.
- **Meetings**, the first layer of it: a transcript can be brought in as plain text with a speaker at the start of each line and an optional `[00:12]` timestamp, and Parlo keeps it in `meetings/` beside the configuration, split by speaker. A speaker can be named once and the name applies to every line that speaker had said, while the original marker is kept underneath for audit; an empty name goes back to the marker, and a name without the sentence that supports it is refused. Deleting a meeting removes its file.
- Dictating in another language now shows both sides on the island: the inserted line is the translation, and the hint keeps the sentence that was actually heard, so a mistranslation is visible instead of silent.
- The Hub now shows the **model library** the build actually ships with: every component with its identifier, version, function, licence and whether that licence was checked, served by `GET /catalog` from the catalogue embedded in the binary. It is read only, and the panel says so — installing from there is not in this build.
- A past dictation can be used again: every line in **Recent dictations** is a button that puts that text back on the clipboard, so a sentence said yesterday does not have to be said again. The entry keeps the translation that was inserted, not the raw speech.
- Every dictation is now kept in a local history: what was said, its translation and the pair of languages, in `history.jsonl` beside the configuration. The Hub lists the last twenty under **Recent dictations** with a clear button, `GET /history` serves them behind the token, `POST /history/clear` removes the file, and the store stops at two hundred entries so it cannot grow without a bound. Nothing leaves the machine.
- Dictation can now be written in another language. **Dictation writes** in the Hub takes Portuguese or English, or the language spoken; the island asks the engine to translate the recognised sentence and inserts that instead, so a person who speaks Portuguese can dictate English into any field. The engine translates through the same local models, the choice is stored in `config.json`, and a missing translation model is reported as a sentence rather than a wrong insertion.
- The tray menu now lands where it says it does: **Live** opens the window and starts live translation, and **Settings** opens the window with the languages and recogniser row in view and in focus. The page reads the `#live` and `#settings` fragments, so the menu no longer drops the user on the Home screen.
- Parlo now has one presence: a tray icon whose menu offers Dictate, Live, Open Parlo, Settings and Quit, and whose left click shows or hides the dictation island. The shortcut stays as an accelerator, not as the only way in. Everything the menu names exists; nothing in the menu opens a screen the product does not have.
- Dictation writes into the field it came from. When the island starts listening it asks the engine which window has the focus and remembers it; when the text is ready, `POST /insert` types it with the keyboard only if that same window is still in front, and otherwise the island falls back to the clipboard and says why.
- Dictation has a floating island: a borderless pill at the bottom of the screen, shown and hidden with **Ctrl+Alt+D**. Showing it starts listening, hiding it stops the recording, writes the text and copies it, so one shortcut pressed twice does the whole thing. It is served by the local API at `/island` and uses the same `POST /dictate`, it never takes the keyboard focus from the application you are writing in, and the shortcut is registered with the Win32 hotkey API, so no plugin is added to the window; elsewhere the log says the shortcut is not available yet.
- The window can dictate. A **Dictate** button beside **Live** turns the record button into transcription: hold it, speak, let go, and the text appears in the window and in the clipboard, with no translation and no voice. It goes through a new `POST /dictate` on the local API, which answers with the text and its latency. The request travels the same `dictate` path as the one behind the terminal: `Engine` gained a dictation method, the pipeline forwards it, and the sidecar answers it on the recogniser it already has loaded, so dictating costs no second copy of the model.
- `parlo dictate` turns the microphone into a clipboard: it transcribes in Portuguese or English, prints each segment as it closes, and copies the joined text with `clip`, `pbcopy`, `wl-copy` or `xclip`. The engine gained a dictation mode that skips translation and voice, so a machine without the OPUS-MT packages can still dictate.
- The translation corpus grew to 100 original sentences, 50 in each direction, with numbers, negations, names, dates and times; the first 24 are unchanged.
- `engines/cascade/scripts/mt_gate.py` checks a package against that corpus for numbers, clocks, negations, names and completeness, and exits non-zero below its thresholds.

### Changed

- `stream_bench.py` answers the half of G0 that can be answered on synthetic audio: it reports whether decode time and the real time factor grow from the first third of a run to the last, and fails when the factor grows past 1.25. On 20 PT-PT sentences with the European voice both ratios came out below one (x0.988 and x0.971), so the reprocessing pipeline does not accumulate delay on this machine.
- Portuguese to German is measurable, through the English pivot the plan allows but requires to be declared: 20 business e-mail sentences joined the corpus, `mt_gate.py` grew `pt-de`, German negation and numeral patterns, and `--pivot` composes PT→EN with EN→DE and prints the route. Numbers, clocks, negations and completeness score 1.000 and names 0.950. The pair is not validated: the index carries no `pt` to `de` package, the German references have not been read by a native speaker, and rows the counts cannot see are visibly wrong (`Das Büro in Porto ist 5`).
- The package downloader sends a user agent, because the mirror answers 403 without one, and it reads packages whose contents sit inside a single top folder instead of refusing them (the EN→DE package is built that way).
- A direct PT→DE model was attempted and is not usable yet: `facebook/m2m100_418M`, MIT, converted to CTranslate2 int8 in 471 MB, answers but degenerates into repeated tokens (`Ich habe das Tre Tre Tre …`), with and without the repetition penalties the Marian path uses. The cause is the converter, not the model or the quantisation: `ctranslate2` 4.8.4 against `transformers` 4.44.2 fails with `M2M100ForConditionalGeneration.__init__() got an unexpected keyword argument 'dtype'`, and against `transformers` 5.22.1 it converts weights that repeat themselves. The translator understands a source token and a target token and the gate can read a converted folder, so the route is ready for a converter that works, but PT→DE keeps running only through English.
- The translation gate searched the Argos application's package folders before the folder Parlo installs into, so the published PT↔EN numbers described the Argos copy rather than the one the product writes. `mt.pack_roots` reads Parlo's own folder first now, and with the decoder fix the folders `packs.install` creates clear the gate in both directions. The `ct2/` + `hf/` folder left in the checkout is not what the product installs and is ignored by name.
- `MarianTranslator` joined a hypothesis's pieces with spaces before decoding them, which split numbers in the output: `2026` came out as `20 26` and `1.250,50` as `1.2 50 , 50`. The pieces are decoded as a list now, and on the copy Parlo installs both directions are clean on numbers: PT→EN numbers, clocks and negations 1.000 with names 0.980, EN→PT numbers and negations 1.000 with clocks and names 0.980. The two runs taken before the fix read 0.840 and 0.880, and those numbers were this bug, not the model.
- On the 100 sentence gate both directions pass every criterion: numbers, clocks, negations, names and completeness. PT→EN used to lose the half of the day on afternoon times; the backend now restores the marker when the source states a 24 hour time, which is a rule with tests rather than a better model. One PT→EN row still translates a name (pt-en-003), and three of the four once reported were the gate ignoring the first word of a sentence, so the pair is not declared validated.



## [0.1.7] - 2026-11-01


### Added

- `parlo-app`, a native window built on Tauri 2, so Parlo no longer opens in a browser. It starts `parlo serve` on a free loopback port with a fresh token, waits for `/health`, and shows the page in its own window with a starting screen while the engine loads and a plain failure screen, naming both log files, after four minutes without an answer.
- Closing the window stops the engine, and `PARLO_APP_GUARD` makes the engine exit when its parent's pipe closes, so a crashed window leaves no process holding the port.
- `parlo app` opens the window when `parlo-app` sits beside `parlo`, and falls back to the browser otherwise.
- The release archive carries `parlo-app` next to `parlo` and `parlo-engine`.
- A licence gate, `tools/licenses.py`, that reads the Rust closure from `cargo metadata`, the Python closure from the pinned requirements and the model catalogue, and fails when a licence is missing, denied, outside the allow list for that scope, or attached to an unpinned requirement.
- `THIRD_PARTY_NOTICES.md` and a CycloneDX SBOM generated from the same inventory, both included in every release archive next to `LICENSE` and `NOTICE`.
- `docs/MODEL_LICENSES.md`, `SUPPORT.md`, `GOVERNANCE.md` and a model catalogue at `catalog/models.json`, so each downloadable artefact carries its own licence instead of inheriting the one of the code.
- Text states for streaming recognition: `partial`, `stable`, `final` and `corrected`, with stable words confirmed by local agreement between the last two hypotheses, a segment that closes after 0.45 s of silence, and the mode reported as `reprocess` rather than native streaming.
- `parlo stream <file.wav>` prints those states as they arrive, over a new `stream` operation on the sidecar protocol.
- `engines/cascade/scripts/stream_bench.py` and a 20 sentence European Portuguese corpus measure the first partial, the first stable words, the final text and the word error rate of each stage.
- `setup_models.py --voice pt_pt` fetches the Piper `pt_PT-tugão-medium` voice.
- `parlo_cascade/mt.py`, a translator that runs an OPUS-MT model through CTranslate2 with its own SentencePiece vocabulary, so a model translates without the Argos runtime; it uses the folder in `PARLO_CASCADE_MT_DIR` when set, otherwise the package installed for the pair, and reports a missing model instead of guessing.
- The model catalogue carries the two OPUS-MT packages as CC BY 4.0 with their authors named, read from the licence line the package itself ships.
- `parlo_cascade/packs.py`, a downloader for those packages: it reads the published index, streams the archive with a 512 MB ceiling, refuses entries that escape the folder, checks the files a model needs, writes the SHA-256 and the index it came from next to the copy, and leaves an existing install untouched.
- **Voice profiles in the app.** Create a profile for your own voice, or for someone
  else's with their permission, and Parlo records the consent locally with the purpose,
  the notice version and the time. Revoke or delete it from the same panel; deleting
  removes the folder and everything in it. Recording the sample needs the voice engine,
  which is not in this build yet, so a profile can be created but not used.

### Changed

- `parlo install` copies both programs and points the Start Menu or desktop entry at the window instead of a console.
- The engine log is written to `parlo-engine.log` in the temporary folder when the window starts it, because a double-clicked app has no console.
- The minimum Rust version is 1.85.
- `scripts/parlo-app.cmd` and `scripts/parlo-app.sh` build both programs and open the window.
- Captions reach the screen about 1.6 s after speech starts at a 1.2 s window, 444 ms of processing over 20 sentences, inside the two second target the earlier 2.2 s run missed; the translation stage is the part that changed.
- A two minute soak of the current program turns a segment around in 699 ms mean, p95 940 ms, 66 segments and no errors, voice synthesis included; the run and its conditions are in `docs/BENCHMARKS.md`.
- The cascade translates through that backend and nothing else: `argostranslate` left the engine, the requirements, the PyInstaller spec and the licence closure, taking the AGPL `minisbd` sentence splitter with it. The SentencePiece word marker is cleaned out of the output.
- `setup_models.py` installs the packages itself instead of calling the Argos tooling, and warms the pair through the same backend the engine uses.
- Piper left the engine program: `parlo-engine` no longer bundles `piper`, `piper_phonemize` or the espeak-ng data, so the packaged engine carries no GPL code. The synthesiser is an optional adapter in `engines/tts-piper`, installed separately by the user and driven over JSON lines on a pipe; without it Parlo translates and recognises as before and says the voice is unavailable.
- `ctranslate2` and `sentencepiece` are declared and pinned in the engine requirements, because the translation backend imports them directly now that they no longer arrive through Argos.
- The licence gate no longer treats `r-efi` as an open issue it can only report: it asks cargo for the dependency graph of each release target and fails when a crate marked not shipped appears in one. `r-efi` is absent from all three.
- Engine requirements are pinned to exact versions, every commit in a pull request must carry a DCO sign-off, and the release workflow refuses to build while the licence gate reports an open issue.
- The original code moved to the Apache License 2.0 once the copyright holders confirmed the whole history; releases up to v0.1.7 keep the MIT text they were published with, in `licenses/MIT-Parlo-0.1.x.txt`.
- `LICENSES.md` at the root indexes every licence the project carries, by section, with the code licence, the model or voice licence, the holder, whether it is required or optional, whether it was confirmed and what it obliges; CI regenerates it and fails when it drifts.
- On the European Portuguese voice the cascade reaches the first partial at 849 ms and the final text at 3.36 s, with 0.231 final word error rate for Parakeet and 0.279 for Whisper base, and 81% of the sentence confirmed before the segment closes. The Brazilian voice reading the same European sentences scored about 0.66 for both, so the corpus now uses the PT-PT voice.

## [0.1.6] - 2026-10-31

### Changed

- The release archive now carries the engine, and models, settings and downloads moved to the per-user data directory: `%APPDATA%\parlo` on Windows, `~/Library/Application Support/parlo` on macOS, `$XDG_DATA_HOME/parlo` elsewhere, overridable with `PARLO_MODELS_DIR` and `PARLO_DATA_DIR`.
- The local API refuses a `Host` header that is not the local address and a write whose `Origin` is not the page, which closes DNS rebinding and cross-site writes against the loopback port.
- The hardware profiles follow the plan again: `ultra`, `high`, `medium` and `basic`, with GPU memory read from `nvidia-smi` when it exists and the high and ultra profiles reserved for a GPU the machine actually has.
- On the same 24 sentence battery, Whisper `small` scored 0.217 clean against 0.215 for Parakeet, and degraded faster under noise (0.309 against 0.197 at 10 dB), so Parakeet stays the default on accuracy as well as speed.
- The cascade uses Parakeet TDT 0.6B v3 through sherpa-onnx for recognition: 0.113 real time factor on 4 threads with a 2.5 second window, against 0.309 for Whisper `base`, with fewer name errors. `PARLO_CASCADE_STT=whisper` keeps the old recogniser, and `PARLO_CASCADE_MODEL` applies only to it.
- The setup script downloads Parakeet by default, and caches Whisper only when asked with `--stt whisper`.
- A window sweep with Parakeet confirms the 2.5 second default: shorter windows save about half a second and lose words, so the next latency gain has to come from a sentence boundary, not a smaller cut.
- Models warm up at startup, moving the 12 to 21 second load time away from the first sentence.
- A glossary file corrects names the recogniser mishears, starting with `Parlo`.
- Piper is GPL-3.0-or-later, not MIT. The packaged engine now contains Piper and the espeak-ng data, so `parlo-engine` is distributed under the GPL-3.0-or-later while the `parlo` command line program stays MIT; `docs/LICENSES.md` states the split.
- The cascade segment window defaults to 2.5 seconds and is configurable with `PARLO_CASCADE_SEGMENT`; at 1.2 seconds the real time factor was 2.66.
- Speech recognition runs without beam search and without a second voice activity filter, since the pipeline already gates silence.
- The default engine is the universal CPU cascade, MIT apart from the GPL synthesiser it bundles in `parlo-engine`; Hibiki-Zero becomes an optional premium engine because its weights are CC BY-NC-SA 4.0.
- The recorder peak normalises every clip towards -1 dBFS with a +24 dB ceiling instead of writing the raw signal, after a laptop Realtek microphone measured between -24.7 and -32.2 dBFS peak.
- The voice gate default moved from -45 to -55 dBFS RMS, because that microphone's speech sits between -39.7 and -46.2 dBFS in 100 millisecond windows against a room noise floor of -63.9 to -67.0 dBFS.
- Live listening and the local API apply automatic gain inside the pipeline after the voice gate, up to +24 dB, so a quiet microphone still reaches the cascade engine.

### Added

- CPU cascade engine: faster-whisper, Argos, and Piper translate Portuguese and English in both directions, offline, at about 2.6 seconds per 1.2 second segment on a 12 core CPU. The output voice follows the target language.
- `parlo translate --engine cascade` streams audio through the cascade sidecar.
- `parlo record` captures the microphone to a WAV for benchmarks, and `scripts/compare.py` runs two recognisers over the same recordings.
- `scripts/stt_bench.py` measures a recogniser alone, with a fixed thread count and window.
- `scripts/memory.py` measures peak memory of a real run: 1302 MB with Parakeet, including the 640 MB model.
- `parlo listen` captures the microphone continuously, translates it through the cascade, and reports the time until the first translated audio.
- `parlo serve` exposes an authenticated local HTTP API on `127.0.0.1` with `/health` and `/translate` for 16 kHz mono WAV audio.
- `scripts/soak.py` loops audio through the engine and reports latency drift: two minutes and 86 segments showed no drift and no errors.
- `parlo engines` lists available engines with licenses, hardware, and voice levels.
- `parlo profile` recommends a hardware profile and voice level from the detected CPU.
- `parlo mic setup`, `parlo mic status`, and `parlo mic remove` manage a PipeWire null sink on Linux.
- `parlo translate --engine sidecar` drives the Python sidecar over the JSON protocol instead of the echo engine.
- Engines report their target languages as owned strings, so adapters are no longer limited to static language lists.
- Offline dubbing: `parlo dub video.mp4 --to en` extracts the audio, recognises each spoken segment, translates it, speaks it in the target voice at its original start time, and writes a video with a replaced audio track plus a `.srt`; `--keep-background` mixes music under the dub at 0.18 and `--json` writes the lines.
- A dub panel in the app: drop a video or audio file on the page, watch elapsed time while the local server works, then download the dubbed file and read the translated lines.
- Captions mode: `parlo captions` translates the microphone to text with no speech synthesis, using a 1.2 s window by default. Measured warm on Parakeet at 958 to 1266 ms of processing per segment, so a caption lands about 2.2 s after the words start.
- Language and recogniser settings inside the page, with `GET /settings` and `POST /settings` on the local API. Applying a change reloads the models in about twenty seconds and is written to the config file.
- `--stt parakeet|whisper` on `config`, `app`, `serve`, `listen` and `captions`, and an `stt` field in the config file.
- `parlo speakers` lists output devices, and `parlo listen --play --speaker N` plays the translated voice to one of them through a bounded queue and rate matched resampling.
- `parlo-engine`: the cascade packaged with PyInstaller into one self-contained executable, preferred by the CLI when it sits beside the binary, so a release user installs no Python. The release workflow builds it per platform and ships it inside the archives.
- `parlo config` reads and writes `config.json` (engine, recogniser, language pair, port, device), and `app`, `serve`, `listen` and `translate` start from it.
- `POST /dub` on the local API, uploads limited to 512 MiB, and `GET /dub/<id>` serving the produced file back only from an allowlist of files the server wrote.
- `docs/THREAT-MODEL.md`, an initial threat model written against the code, with what is protected, who could attack and what is still open.
- A reference battery of 24 Portuguese sentences with seeded noise and automatic word error rate scoring, so a recognition regression shows up as a number without anyone recording anything.

### Fixed

- A self-correction replaces the words it corrects instead of the whole sentence: «Quero comprar 2 bilhetes para o cinema, quer dizer, 3 bilhetes para o cinema» now gives «Quero comprar 3 bilhetes para o cinema», and «Vou a Lisboa amanhã, ou melhor, ao Porto amanhã» gives «Vou ao Porto amanhã» rather than losing the beginning or repeating the preposition.

- The live test produced no translated audio at all from 20 seconds of speech on a quiet microphone, because so few frames cleared the gate that the cascade never completed a 2.5 second segment.
- The input stream error was swallowed, so a device that disappeared mid recording ended as a short silent file; capture now ends with `the input device stopped` instead.

### Planned

- Hibiki-Zero inference behind the engine boundary.
- Measured latency and quality benchmarks.
- Linux virtual microphone automation.

## [0.1.5] - 2026-10-31

### Added

- `--from` and `--to` on `app`, `serve`, `listen` and `translate`, validated and passed to the cascade sidecar; English to Portuguese was verified end to end, 6.33 s of input giving 4.21 s of output.
- `scripts/parlo-app.cmd` and `scripts/parlo-app.sh` build once and open the interface, so the app starts without a terminal command.

### Changed

- The app documentation describes both launchers and the one-direction-per-process rule.

## [0.1.4] - 2026-10-31

### Added

- A models panel in the app lists the recogniser and both voices with their size on disk or `Missing`, and `Download models` starts the fetch from the page and refreshes itself until the translator is ready.
- `GET /models` and `POST /models` expose the same state on the local API.

### Changed

- The first start after a boot is published as 18.1 s on Windows 11, from process start to a translated file, with the sidecar and every model included.

## [0.1.3] - 2026-10-31

### Added

- The interface shows the translated sentence as selectable text, returned by the local endpoint in a base64 header next to the audio.
- The update banner installs the release on the spot: a token bearing request to the local server stages the new binary and lets the page explain that the connection closing is the update.

### Changed

- The screen is redesigned around one record card that holds the input level and pulses while listening, a result card with the sentence and the playback, and a three fact status group.
- Updates follow tagged releases only. The `nightly` channel is removed, so nobody is asked to install a commit.
- A thirty minute soak closed the audio half of the v0.2 criterion: 652 cycles, 1095 segments, 420.7 ms mean, p95 485 ms, zero errors and no drift.

## [0.1.2] - 2026-10-31

### Added

- `parlo update` reads the release feed, downloads the build for the platform, and replaces the running binary about two seconds after it exits, keeping the old one beside it as `parlo.previous`; `--check` reports the two versions and changes nothing.

### Changed

- Release titles are the version alone, `vX.Y.Z`, with no codename after it.

## [0.1.1] - 2026-10-31

### Added

- `parlo app` serves one page from the same binary and opens it in the browser with the session token, so the translator needs no command in front of you: hold the control to speak, hear the translated voice, watch the input level, and see the engine and version.
- The page offers a newer release when the GitHub feed has one, with the notes linked, and never installs anything without a click.
- Every push to `main` publishes the `nightly` prerelease for the three build targets.

### Changed

- The roadmap gains a `v0.4 App` phase and the later phases move up one number, since an interface comes before dubbing a video and a native shell.

## [0.1.0] - 2026-10-31

First tagged release of the command line prototype. Everything under `[Unreleased]` above describes this same code, and the release notes live in `docs/releases/v0.1.0.md`.

### Added

- Streaming pipeline with 20 ms frames, voice gate, resampling from 48 kHz to 16 kHz, and an engine boundary.
- `Engine` trait plus an echo engine for testing without a model.
- Latency budget with a 3 second target and a 5 second ceiling.
- Bounded ring buffer that keeps the newest audio and reports evictions.
- PCM16 WAV reader and writer with header validation.
- `parlo` CLI: `doctor`, `demo`, and `translate`.
- Reference Hibiki sidecar with a JSON lines protocol, honest unavailability errors, and tests.
- Documentation for architecture, engines, languages, privacy, virtual microphones, benchmarks, and troubleshooting.
- CI for Linux, macOS, and Windows, release builds on tags, CodeQL, and dependency updates.

[Unreleased]: https://github.com/PedroNascimento09/parlo/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/PedroNascimento09/parlo/releases/tag/v0.1.0
