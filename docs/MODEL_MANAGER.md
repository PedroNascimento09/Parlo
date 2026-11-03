# The model manager

What a person should experience, written down before it is built. Every number below is a **target** unless it says measured, and nothing here may be quoted as a capability of the current release.

## The problem this solves

The first builds shipped a 231 MB installer, unpacked to 655 MB, and then asked for about 800 MB of models before anything worked. That is a slow first impression and it asks the person to care about models before they have dictated a single sentence.

The target is the opposite: a small download, a first start that looks at the machine, one clear recommendation, and a download that only brings what the person will actually use.

## First start

1. **The window opens** with a short, honest line: what Parlo does, and that nothing leaves the computer.
2. **The machine is read once**, locally, with no identifier and nothing sent: operating system, architecture, CPU features (AVX2, AVX-512), total and free memory, free disk, GPU and VRAM if any, and the power state. This is the same profile the plan defines for hardware classes.
3. **A recommendation appears**, in one sentence per feature, with the reason next to it: *"Dictation with Parakeet int8 — this machine has 16 GB and AVX2, about 0.11 of real time."*
4. **One button downloads the recommended set.** The person can also open the switches first, which is the point below.

No account, no email, no telemetry, and the profile is never written to a log.

## Feature switches

Each capability is a switch, off until it is turned on, and each switch names the models it needs and their size before anything downloads.

| Switch | What it needs | Roughly |
|---|---|---|
| Dictate | one recogniser | the smallest set; dictation works with this alone |
| Write in another language | a translation pair for the chosen direction | added only when the switch is on |
| Live | recogniser, a pair, a voice | the heaviest; off by default on 8 GB machines |
| Translate a file | a pair, plus `ffmpeg` on `PATH` | says so before downloading |
| Meetings | recogniser, optional voice | recording works without translation |
| Read aloud | one voice per language | off unless asked |

Turning a switch off never deletes anything by itself: it stops offering the feature and says which models are now unused. Deleting is a separate, explicit action.

## The model list

One row per model, and every row answers the same questions in the same order, in words a person can compare:

| Column | Example |
|---|---|
| Name and what it does | `Parakeet 0.6B v3 · speech to text` |
| Size on disk | `631 MB` |
| Memory when loaded | `1.2 GB` |
| Speed on this machine | `0.11 real time`, measured here, not a brochure number |
| Quality note | `word error 0.231 on this project's corpus` |
| Licence | `CC-BY-4.0` |
- **In use** marks the recogniser while a transcription or a translation runs. The list keeps the last answer the engine gave about what it holds and does not ask again while the engine is busy, so the row is honest and the page never waits. The translation pair is not marked: whether a request translated depends on the language it wrote into.

States are the four the plan already fixes, and they are visible: a model that is downloaded but not loaded says so, and so does one that a task is currently using.

## The library, by what a model does

The library is a broad catalogue grouped into sections by purpose: **Speech to text**, **Read text aloud**, **Translation** and **Speech synthesis**. A person opens the section that matches what they want and reads the rows; nothing is installed by opening it.

Each row carries the name, the languages it covers, its size where one was measured, its licence, and its measured memory, speed or quality where a measurement exists. A row with no measurement says so instead of borrowing a number from a sibling. A row whose model this build cannot run carries no button and a sentence saying why; a row that can be installed carries **Download** and brings that model alone.

The product does not test every model it lists. It tests what it ships and what it claims: a quality number is published only with its corpus and conditions, and a feature is wired only after the licence gate passes. Beyond that the catalogue is a choice, not a curriculum, and the client decides which model to take.

## Uninstall, and what happens next

- Any model can be removed with one action, from its own row, with the size it frees shown first.
- **Removing a model that a feature needs disables that feature and says it in a sentence**, on the feature and on the row: *"Live needs the English voice you removed. Download it again to turn Live back on."* The app never waits until the person presses a button to report a missing file.
- Nothing in use can be removed; the row says which task is holding it and offers to stop it first.
- Removing keeps the transcript, the vocabulary and the settings. `Remove` and `Unload` stay different words with different effects.

## Moving models

- One setting says where models live, and it can point at another disk, including an external one.
- Moving is a copy with verification, then the old copy goes; a move that fails leaves the working copy untouched and says why.
- A folder that already contains models is adopted after checking each file's digest, so a person can carry them on a drive between machines of the same platform.
- The app says how much space a location needs before it copies anything.

## Delivery

- The downloaded app is a small setup executable (a few megabytes) or the archive, and neither carries models.
- A person who only wants dictation downloads the recogniser and nothing else.
- Every download resumes where it stopped, is checked against a published digest, and can be cancelled without leaving a half-installed model behind.

## The rule for every feature, now and later

This is the part that must survive every future feature, not only the ones listed above.

- **A feature is off until the model that serves it is on disk.** Being able to reach a provider, a runtime or a code path is not the same as having the weights; the app checks the files before it offers the action.
- **Installing is always a choice, and the choice is per model.** A person can add the recogniser and stop there, or add translation, or add a voice, in any order, and the app downloads nothing else on its own.
- **What was installed is what may run.** A feature whose model is absent says which model and offers the one action that fixes it, in the place the person already is — never a silent failure, never a download in the background.
- **Removing is honest both ways.** Removing a model frees the space immediately, disables exactly the features that depended on it, and leaves everything else untouched: history, meetings, vocabulary and settings stay.
- **A recommendation is advice.** It sets the default offer for a machine, never a restriction; any compatible model stays reachable, with the reason stated when it is a poorer choice.
- **No feature may quietly download.** If something needs bytes, the person sees the size and presses the button.

Applied to the current surfaces: dictation needs a recogniser; write-in-another-language needs a recogniser and one translation pair; Live needs a recogniser, a pair and a voice; translate-a-file needs a pair and `ffmpeg`; meetings need a recogniser and optionally a voice; read-aloud needs a voice. Every one of those checks lives on the same list, so adding a feature means adding a row, not a special case.

## Rules that do not move

- No model downloads without a switch the person turned on or a button they pressed.
- No hardware detail leaves the machine, and none of it is needed for the app to work.
- A recommendation is never a block: any compatible model remains reachable, with a warning when it is worse, and the reason stated.
- Every row shows a measured number with its conditions, or says the number is not measured yet.

## Status

Built: the model library lists every component this build knows, with what it does, the languages it covers, the size measured on this machine or a plain "size not measured", its licence and its quality number where one exists, and a row whose model the app can fetch carries its own Download button, the translation pairs included. The list holds the recogniser, the two voices and the two translation pairs, and each row shows its size on disk and the feature it serves. Download fetches that model alone and Remove deletes it, refusing while a download runs and saying which. A running download can be stopped and what already arrived stays on disk. Each row also says why it is the model offered, from the cores, memory and AVX2 read on this machine when the page asks, and claims no speed, because no speed was measured for the new recommendation. The folder that holds the models can move to another disk: the app copies the tree, compares the bytes, and only then deletes the old folder and saves the new path. Dictation, translation, read aloud and captions are separate switches saved in `config.json`, and captions or dubbing refuse with a sentence when the model behind them is switched off. Removing a voice or a pair switches off the feature that needed it and says which switch went off and which model to download again. Whisper base is a second recogniser with its own row: 148 MB in memory against 746, and 0.24 of real time against 0.21, so it is the one offered when the larger model would eat the machine's headroom. Whether it is installed is read from the weights in the Hugging Face cache, never from a folder that merely exists. Dictation counts as ready only when the recogniser Settings chose is the one on disk, so a machine with Whisper alone is not told it can dictate while the engine would still ask for Parakeet. Removing it deletes those weights, and the first-start list keeps whatever ticks the person made while the model list refreshes underneath. The first-start screen offers each feature with the size of the models behind it, ticks the ones this machine's memory suggests, states the measured memory and speed of the model behind each one, and downloads only what the person leaves ticked. The engine accepts `--only parakeet`, `--only voice-pt` and the rest, so a machine without the window gets exactly the units it asked for.

A row says **In memory** when the engine answered that it holds that model, and `On disk` when the files are there and nothing loaded them yet. `GET /models` carries the same values in `state`.

A row also states the quality number measured for that model with its conditions, and a voice says that no number was measured rather than borrowing one from a sibling.

Rows carry the lifecycle of Part XVI. A model is loaded or unloaded by hand, every row has its own delay from ten minutes to four hours or **Keep loaded**, and the row says whether that delay is its own or the global one. Settings turns automatic management off, and while it is off nothing is freed and each model still loads when a task needs it. Idle time is measured per stage, so a recogniser that ran for an hour no longer keeps the voice in memory. A row says `In use` while a request is in flight, and unloading refuses with a sentence until that request finishes. `Keep loaded` is the delay that prevents an automatic unload, so no second pin control is offered.

Built since: moving the models answers with the megabytes it will copy and whether the target already holds an identical tree, and only a second press copies anything; a target whose files match in name and size is adopted instead of copied again. A model downloaded over an existing copy sets that copy aside beside it, and **Restore** puts it back. The first start names the recogniser it recommends with the measured realtime factor behind the choice, not memory alone.

Still missing: the file-by-file digest check on adoption, resuming an interrupted download instead of starting it again, memory pressure, preloading and a limit on total memory. Those stay planned, and the plan keeps the rest at **v0.7 Modelos**.

## A translation pair the engine loads from a folder

The engine reads two kinds of translation model. A package produced by `setup_models.py` speaks CTranslate2. A folder of ONNX files speaks the direct export, which is how Portuguese to German runs at 601.6 ms median where the pivot needs about 790 ms.

The folder is `models/mt/<source>-<target>-onnx` (or `translate-<source>_<target>-onnx`), and it holds seven files:

| File | What it is |
|---|---|
| `encoder_model.onnx` | the encoder |
| `decoder_model.onnx` | the decoder for the first token |
| `decoder_with_past_model.onnx` | the decoder that reuses past key values |
| `config.json` | carries `decoder_start_token_id` and `eos_token_id` |
| `vocab.json` | the piece table, where the language tag is an id |
| `source.spm`, `target.spm` | the sentencepiece models |

`parlo` looks for the folder before any other pair for that direction, so an exported pair wins over a converted one. `PARLO_CASCADE_MT_DIR` accepts either shape and is recognised by the presence of `encoder_model.onnx`.

The language tag comes from a table (`>>deu<<`, `>>eng<<`, `>>fra<<`, `>>por<<`, `>>nld<<`, `>>spa<<`, `>>ita<<`). A folder whose tag is unknown, or whose tag is not in `vocab.json`, is refused with the file or the tag named, before any weight is loaded.

No such export is published for the German pair, so the folder is made on the machine that has the toolchain:

```bash
python -m venv out/onnx
out/onnx/Scripts/pip install torch==2.4.1 "optimum[onnxruntime]==1.24.0" transformers==4.46.3
out/onnx/Scripts/python engines/cascade/scripts/export_pair.py --output out/pt-de-onnx
```

`export_pair.py` calls `main_export` with `text2text-generation-with-past`, which is the task that writes `decoder_with_past_model.onnx`; the plain `translation` task does not. Copy the folder to `models/mt/pt-de-onnx` and the engine picks it up on the next start.

**Status: implemented, not verified in an installed build.** The decoding loop and the folder refusals are covered by tests that need no weights; the runtime path has not been run since the export measured in `docs/BENCHMARKS.md` was deleted with the other artefacts, so it is not claimed as working.

The export writes `config.json` and `vocab.json` itself, so the copy step is a real step rather than a formality: a missing file is named in the refusal instead of failing somewhere inside the decoder.
