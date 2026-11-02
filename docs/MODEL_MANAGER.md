# The model manager

What a person should experience, written down before it is built. Every number below is a **target** unless it says measured, and nothing here may be quoted as a capability of the current release.

## The problem this solves

The archive is 220 MB and unpacks to 631 MB; the first start then asks for about 800 MB of models before anything works. That is a slow first impression and it asks the person to care about models before they have dictated a single sentence.

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
| State | `Not installed`, `Installed · unloaded`, `Loaded`, `In use` |

States are the four the plan already fixes, and they are visible: a model that is downloaded but not loaded says so, and so does one that a task is currently using.

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

**Partly built.** The app lists the recogniser and the two voices, each with its own size and its own button: **Download** fetches that model alone and **Remove** deletes it, refusing while a download is running and saying why. The engine accepts `--only parakeet`, `--only voice-pt` and the rest, so a machine can be given the recogniser by itself.

The first-start screen now asks for the recogniser alone, about 640 MB, and says that translation and read aloud are separate models added later; the screen leaves once dictation works. Not built yet: no recommendation from the machine profile, no feature switches, no relocation of the model folder, no cancellation and resume of a download, and the translation pairs are not in the list because the app cannot yet verify their files. The plan keeps this work at **v0.7 Modelos**.
