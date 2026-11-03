<p align="center">
  <img src="assets/parlo-wordmark.png" width="280" alt="Parlo">
</p>

<p align="center">
  <b>Speech in. Text out. In any app, in two languages, on your own computer.</b><br>
  Dictation, live text, captions, meetings and dubbing share one engine.<br>
  Windows first, with macOS and Linux building from source. No account, no telemetry, no audio upload.
</p>

<p align="center">
  <a href="https://github.com/PedroNascimento09/Parlo/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/PedroNascimento09/Parlo/ci.yml?style=flat-square"></a>
  <a href="LICENSE"><img alt="Licence" src="https://img.shields.io/badge/licence-Apache--2.0-1e4f4a?style=flat-square"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
</p>

Parlo is a desktop app that turns speech into writing without sending it anywhere. Press a shortcut, speak, and the sentence appears in the field you were already typing in, cleaned up, translated if you asked for another language. The same engine powers live text, captions, meetings and dubbing, so a model installed once serves every surface: the window configures it, the shortcut and the tray are how you use it.

## What you can do with it

**Dictate into any program.** `Ctrl+Alt+D` opens a small island at the bottom of the screen, on the monitor the cursor is on. Speak, and the sentence is typed into the window that had the focus, in the language you are writing. `Ctrl+Alt+Shift+D` keeps listening for as long as you need it. Where a program refuses typed text, the sentence lands on the clipboard instead, and Escape throws it away.

**Read what you say while you say it.** Live opens a window with two columns, the microphone on the left and the translation on the right, in windows of about 2.5 seconds. Captions put the same stream in a thin strip over a call or a video, with one button that changes the size. `Ctrl+Alt+C` opens captions from anywhere, and Escape stops either of them.

**Keep a meeting.** Record the room and get a transcript with a time and a speaker on every line. Rename a speaker once and every line follows, import a transcript you already have, search the whole meeting, ask what was decided and get the line, the speaker and the time behind the answer, and copy it all out as Markdown.

**Dub a recording.** Drop a video or an audio file and get it back in the other language, with subtitles timed to the source beside it and the original background under the new voice if you want it.

**Hear the translation.** The translated sentence is read aloud in a European Portuguese voice, in the same window, without leaving for another service.

**Get the words right.** A vocabulary of names fixes the words the recogniser keeps missing, on whole words only, before any model reads the sentence. The history keeps what was said, what was written and the program it landed in, and one click puts a line back on the clipboard.

## One window, five places

The window is for configuration and review. The work happens from the shortcut and the tray.

- **Home** says whether it is ready, which engine and machine profile it found, and the three most recent results with the program each one landed in.
- **Library** holds the history, the model list and the vocabulary.
- **Meetings** holds recordings, transcripts and speakers.
- **Voice** records a sample, seals it with your Windows account if you ask, plays it back and exports it marked as synthetic. Cloning a voice from that sample is not in the build yet.
- **Settings** is eight sections of real fields: Dictation, Languages, Shortcuts, Cleanup, Provider, Privacy, About and Advanced. Each has its own Apply, so nothing waits behind a tab.

The tray lists the three modes above a separator and the five destinations below it, and a left click opens the island. The first run is five steps: the language pair, the microphone with the exact Windows place to allow it, the shortcut captured from the keys you press, one real sentence read back with the program it landed in, and a line saying where the shortcut is and what Escape does.

## Shortcuts

| Keys | What happens |
|---|---|
| `Ctrl+Alt+D` | Opens the island and dictates into the window that had the focus |
| `Ctrl+Alt+Shift+D` | Stays listening without holding anything |
| `Ctrl+Alt+C` | Opens or closes captions |
| `Escape` | Throws the sentence away, or stops Live, Captions or a running meeting |
| A left click on the tray | The island, at any time |

Every key is changed in Settings, validated when it is saved, refused when it repeats another of the three, and refused with the reason when Windows says another program already owns it.

## What is inside

| | |
|---|---|
| **A model manager that tells the truth** | One row per model: size on disk, `Installed · unloaded`, `Loaded`, `In use` or `Not installed`, a measured quality number or an honest "no number measured". Each row has `Load`, `Unload now` and `Remove`, and its own delay before it is freed, from ten minutes to four hours or `Keep loaded`. Downloads can be stopped, and the whole model folder can move to another drive with a byte comparison before the old copy is deleted. |
| **Two recognisers, chosen per machine** | Parakeet TDT 0.6B v3 int8 is the default; Whisper `tiny` to `large-v3` stays selectable where 640 MB is too much. The list names the cores, the memory and whether AVX2 is there, and why that machine got the recommendation it got. |
| **Translation with a gate** | Portuguese and English, both directions. 100 original sentences are checked for numbers, clocks, negations, names and completeness before a release is called good. PT→DE runs through English and is published as not validated by a native speaker. |
| **Cleanup that respects what you said** | Three levels: what the recogniser heard, the sentence you meant with self-corrections and hesitations removed, and a tidier version with spacing and capitals. The heard sentence is always kept beside it. |
| **Words Parlo should learn** | A local vocabulary fixes the names the recogniser keeps missing, on whole words only, before any model reads the sentence. |
| **A history you can use again** | Every dictation keeps what was said, what was written and the program it landed in, and one click puts a line back on the clipboard. |
| **A voice panel with a real sample** | Ten to thirty seconds are recorded in the window or refused with the measured length, a silent take is refused too, and the sample can be sealed as DPAPI ciphertext, heard back, exported as one file marked synthetic, or deleted with the profile. |
| **A provider only when you ask** | The cloud model sits behind a switch, your own key and a test sentence that proves it answers. With the switch off, nothing leaves the machine, and the Privacy section says so in one sentence. |
| **Updates inside the app** | The latest release arrives with its notes and one button, and replaces the running copy instead of asking you to uninstall. It keeps `parlo.previous` beside the binary, and a release without a build for your system offers the notes instead of a dead button. |
| **An interface built to be read** | Every text pair clears 4.5:1 measured on the real colours, both themes are drawn from the same fifteen tokens, one focus ring reaches everything that takes the keyboard, and no font, script or image comes from a CDN. |

## Install

**Windows.** [Download Parlo-Setup.exe](https://github.com/PedroNascimento09/Parlo/releases/latest/download/Parlo-Setup.exe) and double click it. It installs for your user, adds a Start menu entry and opens Parlo when it finishes, so one download is the whole install. The build is unsigned, so SmartScreen asks once: **More info**, then **Run anyway**.

**macOS and Linux**, in any terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

**From source**, with Rust 1.85 or newer and Python 3.12 for the engine:

```bash
cargo build --release -p parlo-cli -p parlo-app
target/release/parlo-app
```

The installer is 44.8 MB. On first start the model list offers the recogniser alone, about 640 MB, and nothing else downloads until you press its button. Translation, read aloud and captions are separate switches, and each names the model and its size before anything moves. After that the machine can stay offline. `parlo install` and `parlo uninstall` do the same from a terminal, and uninstalling keeps your models unless you add `--purge`.

## From the terminal

The window is not the only door. The same engine answers on the command line, for scripts and for a machine without a desktop:

```bash
parlo dictate                 # one sentence into the clipboard
parlo stream in.wav           # partial, stable and final text as it arrives
parlo listen --play --speaker 3   # translate the microphone into another output
parlo dub talk.mp4 --srt talk.srt # a dubbed file and its subtitles
parlo serve                   # the authenticated local api, on loopback only
parlo profile                 # what this machine can do, and what to load
parlo doctor                  # whether capture and the engine are healthy
```

## Measured

Windows 11 laptop, no GPU, 4 threads. Conditions and every raw row are in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| What | Result |
|---|---|
| Cleanup rules, 10 sentences × 1 000 runs | **61.9 µs** mean |
| One segment through the cascade, recognition included | **420.7 ms** mean, 485 ms p95 |
| Real time factor, Parakeet v3 int8 | **0.11** |
| Captions, speech to screen at a 1.2 s window | **about 1.6 s** |
| Streaming, first partial to finished sentence | **849 ms** to **3 355 ms** |
| Recogniser, 20 European Portuguese sentences | **0.24** word error rate |
| Cold start, engine included | **18.1 s** |

Nothing here describes a machine nobody ran, and no provider latency or price is claimed: no request to a cloud provider has been made with a real key.

## Privacy

Local mode makes no network call. Audio is never written to disk unless you pass an output path. The local API listens on loopback with a token the window generates at every start, and the update check is the only outside request, carrying no audio and no identifiers. A provider is reached only when you allow it, store your own key and pick it, over HTTPS or loopback, straight from your machine. History, meetings and vocabulary stay in your user data folder and each can be deleted from the interface, a voice sample can be sealed with your Windows account, and [docs/PRIVACY.md](docs/PRIVACY.md) shows how to check all of it.

## What is not there yet

- The dictation shortcut is Windows only. On macOS and Linux the text goes to the clipboard.
- Windows is validated end to end, window included. macOS and Linux are compiled and tested by CI, but the window has not been run there.
- Portuguese and English are the only pair. PT→DE runs through English and is published as **not validated by a native speaker**.
- Dictation is tested, not yet validated with recordings from people other than the author.
- The build is unsigned, and there is no signed `.msi`, `.dmg` or `.deb`.
- Meetings record the microphone rather than the sound of another application, there is no meeting summary yet, and speakers are labelled by the recogniser rather than separated by a model trained for it.
- Captions and Live read the microphone, not the system audio, and reaching a call app still needs a virtual cable today.
- Read aloud uses a neutral European Portuguese voice that no panel of listeners has judged, and no voice is cloned from a recording.

## Docs

[App guide](docs/APP.md) · [Interface](docs/UX.md) · [Architecture](docs/ARCHITECTURE.md) · [Engines](docs/ENGINES.md) · [Models](docs/MODEL_MANAGER.md) · [Model policy](docs/MODEL_POLICY.md) · [Plugins](docs/PLUGIN_DEVELOPMENT.md) · [Dubbing](docs/DUB.md) · [Benchmarks](docs/BENCHMARKS.md) · [Install](docs/INSTALL.md) · [Privacy](docs/PRIVACY.md) · [Troubleshooting](docs/TROUBLESHOOTING.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

## Licence

The code is **Apache-2.0**. Model weights keep their publishers' terms: Parakeet is CC-BY-4.0, the OPUS-MT packages are CC BY 4.0, and the optional Piper adapter is GPL-3.0-or-later, installed separately and never shipped in an archive. [LICENSES.md](LICENSES.md) is the index and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) carries the notices.
