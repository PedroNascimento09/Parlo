<p align="center">
  <img src="assets/parlo-wordmark.png" width="280" alt="Parlo">
</p>

<p align="center">
  <b>Speech to text on your computer. Speak, and the words appear where you were typing.</b><br>
  Dictation, live translation, captions, meetings and file dubbing share one local engine.<br>
  Windows first. macOS and Linux build. No account, no telemetry, no audio sent.
</p>

<p align="center">
  <a href="https://github.com/PedroNascimento09/Parlo/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/PedroNascimento09/Parlo?style=flat-square&color=b03c1e"></a>
  <a href="https://github.com/PedroNascimento09/Parlo/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/PedroNascimento09/Parlo/ci.yml?style=flat-square"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-1e4f4a?style=flat-square"></a>
</p>

Parlo is a desktop program that turns speech into text. Press a shortcut, speak, and the phrase appears in the window you were typing in, cleaned up and optionally translated. Recognition runs on your computer, with no account and no telemetry.

<p align="center">
  <img src="docs/demo.gif" width="760" alt="The Parlo window: the home screen, the history with a Portuguese sentence and its English translation, and the counters for what was dictated">
</p>

<p align="center">
  <sub>The window after two Portuguese sentences were dictated and translated into English on this machine: home, the history with both sentences beside their translation, and the counters. Portuguese and English are the shipped pair; the translation packs are listed in <a href="docs/LANGUAGES.md">languages</a>.</sub>
</p>

## Install

**Windows, from the terminal:**

```powershell
irm https://github.com/PedroNascimento09/Parlo/raw/main/scripts/install.ps1|iex
```

It downloads the release, unpacks it into `%LOCALAPPDATA%\Programs\Parlo`, adds a Start menu entry and opens Parlo.

If Windows says **Windows protected your PC**, choose **More info**, then **Run anyway**. The installer is unsigned, so this may appear once.

**macOS and Linux:**

```bash
curl -fsSL https://github.com/PedroNascimento09/Parlo/raw/main/scripts/install.sh|bash
```

**Download from a browser, if you prefer:**

1. [Download the latest release](https://github.com/PedroNascimento09/Parlo/releases/latest/download/Parlo-Setup.exe).
2. Double-click `Parlo-Setup.exe`.
3. If Windows says **Windows protected your PC**, choose **More info**, then **Run anyway**.

**Package managers.** winget, Homebrew, AUR, Snap, DEB, AppImage and Nix have manifests in [packaging](packaging/README.md). They are recipes, not tested installs: none has been run end to end, and each is marked partial there with what is missing.

The installer is small because the models are separate. The program installs in a fraction of a second. The recogniser is a download of about 640 MB, offered the first time you open the app. Translation, read-aloud and captions each name their model and size before anything moves. After that, Parlo works offline. `parlo install` and `parlo uninstall` do the same from a terminal; uninstall keeps your models unless you pass `--purge`.

Each release includes a `SHA256SUMS.txt` and a build provenance record that names the workflow, commit and repository that built it.

To check the installer on Windows:

```bat
certutil -hashfile Parlo-Setup.exe SHA256
```

Compare the result with the `Parlo-Setup.exe` line in `SHA256SUMS.txt`. With the GitHub CLI logged in, this also checks where the file came from:

```bash
gh attestation verify Parlo-Setup.exe --repo PedroNascimento09/Parlo
```

Neither check replaces code signing. Windows still treats the installer as unsigned, and SmartScreen may ask once.

## First run

The installer carries the programs, not the models. Open Parlo and the Models panel lists what is missing, with the size of each item. Press **Download models** once; after that Parlo works with the network off.

The first start can take about twenty seconds while the engine loads. The window says so.

## What you can do

**Dictate into any program.** `Ctrl+Alt+D` opens a small bar at the top of the screen, on the monitor with the cursor. Speak, and the sentence is typed into the window that had focus as it is recognised, in the language you were writing. With Polish on, Parlo waits and types the cleaned sentence when you stop. `Ctrl+Alt+Shift+D` keeps listening without holding a key, and `Escape` discards the phrase.

**Read what you say while you say it.** Live text opens two columns: your speech on the left and the translation on the right, in windows of about 2.5 seconds. Captions show the same stream in a thin bar over a call or a video. `Ctrl+Alt+C` opens captions from anywhere, and `Escape` stops them.

**Keep a meeting.** Parlo records the room and writes a transcript with a time and a speaker on each line. Rename a speaker once and every line follows. You can import an existing transcript, search the meeting, ask what was decided, and jump from each answer to the line, speaker and time behind it. Copy everything as Markdown.

**Dub a file.** Drop in a video or audio file to get the same content in the other language, with subtitles synced to the original. You can keep the original background audio under the new voice.

**Hear the translation.** The translated sentence is read aloud in a European Portuguese voice, in the same window.

**Fix the words the recogniser keeps missing.** A local vocabulary corrects names one word at a time, before any model reads the sentence. The history keeps what was said, what was written and the program it arrived in. One click copies any line back to the clipboard.

## One program, six places

The window is for setup and review. Daily use happens through the shortcut and the tray.

- **Home** says whether it is ready, which engine it found, the profile of the machine it found, and the last three entries with the program each one arrived in.
- **Library** keeps the history, the models and the vocabulary.
- **Meetings** keeps recordings, transcripts and speakers.
- **Voice** records a sample, seals it with your Windows account if you ask, plays it back and exports it marked as synthetic. Cloning a voice from that sample is not in the build yet.
- **Stats** counts words, minutes and programs behind the things dictated, and the last four weeks, from what stays on this computer.
- **Settings** has five real groups of fields: You, Dictation, Languages, Shortcuts and Advanced, which store cleanup, provider, privacy and about. Each field remembers where it lives, and the language pair has its own Apply.

The tray lists the three modes above a separator and the five destinations below it. A left click opens the bar. First-run setup has five steps: your name, the language pair, the microphone (with the Windows setting to allow it), the writing style, and the shortcut. Only the name is required. The other steps can be skipped, and setup does not return once finished.

## Shortcuts

| Keys | What happens |
|---|---|
| `Ctrl+Alt+D` | Opens the bar and dictates into the window that had the focus |
| `Ctrl+Alt+Shift+D` | Keeps listening without holding any key |
| `Ctrl+Alt+C` | Opens or closes captions |
| `Escape` | Discards the phrase, or stops live text, captions or a meeting in progress |
| One click on the tray | The bar, at any time |

You can change each key in Settings. Parlo checks it on save, refuses a key that repeats another one, and gives the reason when Windows says another program already uses it.

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

These numbers come from the one machine above. No request to a cloud provider was made with a real key, so no provider latency or price is claimed.

## Privacy

Local mode makes no network calls. Audio is not written to disk unless you pass an output path. The local API listens on loopback and uses a token the window generates at each start. The update check is the only external call, and it sends no audio and no identifier. A cloud provider is contacted only when you enable it, with your own key, over HTTPS or loopback. History, meetings and vocabulary stay in your per-user data folder and can be deleted from the interface. A voice sample can be sealed with your Windows account. [docs/PRIVACY.md](docs/PRIVACY.md) explains how to verify this.

## What is not here yet

- The dictation shortcut is only on Windows. On macOS and Linux the text goes to the clipboard.
- Windows is validated end to end, window included. macOS and Linux are compiled and tested by CI, but the window has not been opened there.
- Portuguese and English are the only pair. PT→DE runs through English and is published as **not validated by a native speaker**.
- Dictation is tested, not yet validated with recordings from people other than the author.
- The build is unsigned, and there is no signed `.msi`, `.dmg` or `.deb`.
- Meetings record the microphone instead of another app’s audio, there is no meeting summary yet, and speakers are labelled by the recogniser instead of by a model trained to do that.
- Captions and live text read the microphone, not system audio, and reaching a calling app still needs a virtual cable today.
- Read-aloud uses a neutral European Portuguese voice that no listener panel has judged, and no voice is cloned from a recording.

## About the Windows warning

The Windows build is unsigned, so SmartScreen may warn once on first download. This is expected for a new unsigned installer. It does not mean the file is malicious. Check the SHA-256 against the release, and read [Windows code-signing reality](docs/WINDOWS_SIGNING_REALITY.md) for the full explanation.

If Parlo is useful to you, a [star on the repository](https://github.com/PedroNascimento09/Parlo) helps others find it.

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