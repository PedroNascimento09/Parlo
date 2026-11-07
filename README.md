<p align="center">
  <img src="assets/parlo-wordmark.png" width="280" alt="Parlo">
</p>

<p align="center">
  <b>Parlo turns speech into text on your computer. Speak, and the words appear where you were already typing.</b><br>
  Dictation, live translation, captions, meetings and file dubbing share one local engine.<br>
  Windows first. macOS and Linux build. No account, no telemetry, no audio sent.
</p>

<p align="center">
  <a href="https://github.com/PedroNascimento09/Parlo/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/PedroNascimento09/Parlo/ci.yml?style=flat-square"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-1e4f4a?style=flat-square"></a>
</p>

Parlo is a desktop program that turns speech into text without sending anything out. Press a shortcut, speak, and the phrase appears in the field you were already typing in, cleaned up and, if you ask, translated into another language. Everything stays on your computer. No account, no telemetry, no audio sent.

<p align="center">
  <img src="docs/demo.gif" width="760" alt="The Parlo bar while it writes a translated sentence">
</p>

<p align="center">
  <sub>This machine's bar, captured while it wrote a translated sentence. Spanish needs its own translation pack; the pairs the build ships are in <a href="docs/LANGUAGES.md">languages</a>.</sub>
</p>

## Install

**Windows, from the terminal:**

```powershell
irm https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.ps1 | iex
```

That downloads the same content as the release, unpacks it into `%LOCALAPPDATA%\Programs\Parlo`, adds the Start menu entry, and opens Parlo when it finishes. Same app, one command.

If Windows says ***Windows protected your PC***, choose **More info**, then **Run anyway**. The installer is unsigned, so this may appear once.

**macOS and Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

**Download from a browser, if you prefer:**

1. [Download the latest release](https://github.com/PedroNascimento09/Parlo/releases/latest/download/Parlo-Setup.exe).
2. Double-click `Parlo-Setup.exe`.
3. If Windows says ***Windows protected your PC***, choose **More info**, then **Run anyway**.

**Package managers.** winget, Homebrew, AUR, Snap, DEB, AppImage and Nix each have a manifest in [packaging](packaging/README.md) with the install command. Those are recipes, not fully tested installs: no channel has been run end to end by the machine that wrote the manifests, and each one is marked partial there with what is still missing.

The installer is light because it keeps the program separate from the heavy part. The program installs in a fraction of a second; the recogniser is a separate download of about 640 MB, offered the first time you use the app, and nothing else is downloaded until you press its button. Translation, read-aloud and captions are separate keys, and each one names the model and its size before anything moves. After that, the machine can be offline. `parlo install` and `parlo uninstall` do the same thing from a terminal, and uninstall keeps your models unless you add `--purge`.

Each release carries a `SHA256SUMS.txt` with the hash of every file and a build provenance record that names the workflow, commit and repository that produced it.

Windows:

```bat
certutil -hashfile Parlo-Setup.exe SHA256
```

Compare that line with the one for the installer in `SHA256SUMS.txt`. With the GitHub CLI authenticated, this also checks where the file came from:

```bash
gh attestation verify Parlo-Setup.exe --repo PedroNascimento09/Parlo
```

Neither of those replaces code signing. Windows still treats the installer as unsigned, and SmartScreen may still ask once.

## The first run

The installer carries the programs, not the models. Open Parlo, and the Models panel lists what is missing, with the size of each one, before anything moves. Press **Download models** once and it fetches those; after that Parlo works with the network switched off.

The first start can take around twenty seconds while the engine loads. The window says so instead of looking stuck.

## What you can do

**Dictate into any program.** `Ctrl+Alt+D` opens a small bar at the top of the screen, on the monitor where the cursor is. Speak, and the sentence is written into the window that had the focus, in the language you were writing in. `Ctrl+Alt+Shift+D` keeps listening open while you need it. When a program cannot accept typed text, the sentence goes to the clipboard, and `Escape` discards it.

**Read what you say while you say it.** Live text opens a window with two columns: the microphone on the left and the translation on the right, in windows of about 2.5 seconds. Captions put the same stream in a thin bar over a call or a video, with a button that changes the size. `Ctrl+Alt+C` opens captions from anywhere, and `Escape` stops any of them.

**Keep a meeting.** It records the room and gives you a transcript with a time and a speaker on every line. Rename one speaker once and every line follows. Import a transcript you already have, search the whole meeting, ask what was decided, and get back the line, the speaker and the time behind the answer. Copy it all as Markdown.

**Dub a file.** Drag in a video or audio file and you get the same content in the other language, with subtitles synced to the original and the original background audio under the new voice if you want it.

**Hear the translation.** The translated sentence is read aloud in a European Portuguese voice, in the same window, without leaving for another service.

**Fix the words the recogniser keeps missing.** A local vocabulary corrects the names the recogniser keeps getting wrong, one word at a time, before any model reads the sentence. The history keeps what was said, what was written and the program it arrived in, and one click puts a line back on the clipboard.

## One program, six places

The window is for setting up and for reviewing. The work happens through the shortcut and the tray.

- **Home** says whether it is ready, which engine it found, the profile of the machine it found, and the last three entries with the program each one arrived in.
- **Library** keeps the history, the models and the vocabulary.
- **Meetings** keeps recordings, transcripts and speakers.
- **Voice** records a sample, seals it with your Windows account if you ask, plays it back and exports it marked as synthetic. Cloning a voice from that sample is not in the build yet.
- **Stats** counts words, minutes and programs behind the things dictated, and the last four weeks, from what stays on this computer.
- **Settings** has five real groups of fields: You, Dictation, Languages, Shortcuts and Advanced, which store cleanup, provider, privacy and about. Each field remembers where it lives, and the language pair has its own Apply.

The tray lists the three modes above a separator and the five destinations below it; a left click opens the bar. The first time is five steps: your name, the language pair, the microphone with the exact Windows place to allow it, the writing style and the shortcut with what `Escape` does. Only the name is required; everything else can be skipped, and the first step does not come back once it is done.

## Shortcuts

| Keys | What happens |
|---|---|
| `Ctrl+Alt+D` | Opens the bar and dictates into the window that had the focus |
| `Ctrl+Alt+Shift+D` | Keeps listening without holding any key |
| `Ctrl+Alt+C` | Opens or closes captions |
| `Escape` | Discards the phrase, or stops live text, captions or a meeting in progress |
| One click on the tray | The bar, at any time |

Each key changes in Settings, is validated when you save it, is refused when it repeats one of the other three, and is refused with a reason when Windows says another program already owns it.

## What is inside

| | |
|---|---|
| **A model manager that tells the truth** | One line per model: size on disk, `Installed · not loaded`, `Loaded`, `In use` or `Not installed`, a measured quality number or the honest “no measured number”. Each line has `Load`, `Download now` and `Remove`, and its delay before being released, from 10 minutes to 4 hours or `Keep loaded`. Downloads can be stopped, and the whole models folder can move to another disk with a byte-by-byte comparison before the old copy is deleted. |
| **Two recognisers, chosen by machine** | Parakeet TDT 0.6B v3 int8 is the default; Whisper `tiny` to `large-v3` is selectable where 640 MB is too much. The list names the cores, the memory and whether AVX2 is present, and why this machine got the recommendation it got. |
| **Translation with a pivot** | Portuguese and English, in both directions. 100 original sentences are checked for numbers, clocks, negations, names and completeness before a release is called good. PT→DE runs through English and is published as not validated by a native speaker. |
| **Cleanup that respects what you said** | Three levels, named in Settings as `Keep my words as they came`, `Light, fix the obvious` and `Intelligent, tidy the sentence`. The first writes what the recogniser heard, the second removes self-corrections and hesitations, the third adds spacing and capitals. The heard phrase always stays beside it. |
| **Words Parlo should learn** | A local vocabulary corrects the names the recogniser keeps getting wrong, one word at a time, before any model reads the sentence. |
| **A history you can reuse** | Each dictation keeps what was said, what was written and the program it arrived in, and one click puts a line back on the clipboard. |
| **A voice panel with a real sample** | 10 to 30 seconds are recorded in the window or refused with the measured length; a silent recording is also refused; the sample can be sealed as DPAPI encryption, played back, exported as a file marked synthetic, or deleted with the profile. |
| **A provider only when you ask** | The cloud model sits behind a key, your own key and a test phrase that proves it answers. With the key off, nothing leaves the machine, and the Privacy section says that in one sentence. |
| **Updates inside the app** | The latest version arrives with its notes and a button, and replaces the copy that is running instead of asking you to uninstall. It keeps `parlo.previous` beside the binary, and a version without a build for your system offers the notes instead of a dead button. |
| **An interface built to be read** | Each pair of texts passes 4.5:1 measured on the real colours, both themes are drawn from the same fifteen tokens, a focus ring reaches everything that accepts keyboard, and no font, script or image comes from a CDN. |

## From the terminal

The window is not the only door. The same engine answers on the command line, for scripts and for a machine without a graphical environment:

```bash
parlo dictate                 # one phrase to the clipboard
parlo stream in.wav           # partial, stable and final text as it arrives
parlo listen --play --speaker 3   # translates the microphone into another output
parlo dub talk.mp4 --srt talk.srt # a dubbed file and its subtitles
parlo serve                   # the local authenticated API, loopback only
parlo profile                 # what this machine can do, and what to load
parlo doctor                  # whether capture and engine are healthy
```

Every program start leaves one record in `sessions.jsonl` in the data folder: the moment it began, the moment a clean finish ended it, or the message and file and line of a panic, keeping the most recent 400. `parlo doctor` reads that file back and prints the recorded sessions, how many were clean, how many failed, when the last one ended and the failure-free rate, and says in one line when it still has nothing recorded.

## Measured

Windows 11 laptop, no GPU, 4 threads. Conditions and each raw line are in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| What | Result |
|---|---|
| Cleanup rules, 10 sentences × 1,000 corrections | **61.9 µs** on average |
| One cascade segment, recognition included | **420.7 ms** on average, 485 ms p95 |
| Real-time factor, Parakeet v3 int8 | **0.11** |
| Captions, speech to screen in a 1.2 s window | **about 1.6 s** |
| Stream, first partial to finished sentence | **849 ms** to **3,355 ms** |
| Recogniser, 20 European Portuguese sentences | **0.24** word error rate |
| Cold start, engine included | **18.1 s** |

Nothing here describes a machine nobody ran, and no latency or provider price is claimed: no request to a cloud provider was made with a real key.

## Privacy

Local mode makes no network calls. Audio is never written to disk unless you pass an output path. The local API listens on loopback with a token the window generates on each start, and the update check is the only external call, carrying no audio and no identifier. A provider is only reached when you allow it, keep your own key and choose, by HTTPS or loopback, directly from your machine. History, meetings and vocabulary stay in your per-user data folder, and each one can be deleted from the interface, a voice sample can be sealed with your Windows account, and [docs/PRIVACY.md](docs/PRIVACY.md) shows how to verify all of that.

## What is not here yet

- The dictation shortcut is only on Windows. On macOS and Linux the text goes to the clipboard.
- Windows is validated end to end, window included. macOS and Linux are compiled and tested by CI, but the window has not been opened there.
- Portuguese and English are the only pair. PT→DE runs through English and is published as **not validated by a native speaker**.
- Dictation is tested, not yet validated with recordings from people other than the author.
- The build is unsigned, and there is no signed `.msi`, `.dmg` or `.deb`.
- Meetings record the microphone instead of another app’s audio, there is no meeting summary yet, and speakers are labelled by the recogniser instead of by a model trained to do that.
- Captions and live text read the microphone, not system audio, and reaching a calling app still needs a virtual cable today.
- Read-aloud uses a neutral European Portuguese voice that no listener panel has judged, and no voice is cloned from a recording.

## The honest state of the download on Windows

The Windows build is unsigned. SmartScreen may ask once on first download. That is the expected behaviour for a new, unsigned installer on Windows, and it is not a claim that the file is malicious. You can verify the file you downloaded with the release SHA-256, and if you want the longer version, see [Windows code-signing reality](docs/WINDOWS_SIGNING_REALITY.md).

If the app makes sense to you, the simplest way to support it is to [star the repository](https://github.com/PedroNascimento09/Parlo). That is it: no message, no account, and it is what helps the most.

## Where to read next

- [Start here](docs/START.md)
- [App guide](docs/APP.md)
- [Interface](docs/UX.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Engines](docs/ENGINES.md)
- [Models](docs/MODEL_MANAGER.md)
- [Model policy](docs/MODEL_POLICY.md)
- [Plugins](docs/PLUGIN_DEVELOPMENT.md)
- [Dubbing](docs/DUB.md)
- [Benchmarks](docs/BENCHMARKS.md)
- [Install](docs/INSTALL.md)
- [Privacy](docs/PRIVACY.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)

## License

The code is **Apache-2.0**. Model weights keep the terms of their own publishers: Parakeet is CC-BY-4.0, the OPUS-MT packages are CC BY 4.0, and the optional Piper adapter is GPL-3.0-or-later, installed separately and never embedded in an archive. [LICENSES.md](LICENSES.md) is the index and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) carries the notices.