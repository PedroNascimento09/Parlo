<p align="center">
  <img src="assets/parlo-wordmark.png" width="260" alt="Parlo">
</p>

**A free, open source speech to text app that runs on your own computer.**

Press a shortcut, speak, and the sentence appears in whatever field you were already typing in. Parlo cleans up the way you talk, writes the sentence you meant, and can write it in another language. Nothing is uploaded, and once its models are on disk it works with the network unplugged.

<p align="center">
  <img src="assets/dictate.png" alt="A notes window with a dictated sentence written after the cursor, and the island above it" width="100%">
</p>

## How it works

1. **Press** `Ctrl+Alt+D` and speak. Hold the keys and release to stop, or tap once and Parlo keeps listening until the next press.
2. **Speak.** Parakeet recognises the words on this computer. Nothing leaves the machine.
3. **Change your mind.** Say "Friday, I mean Monday" and Parlo writes Monday. Hesitations and false starts disappear, and the transcript you actually spoke is kept beside the result.
4. **Keep typing.** The sentence is typed into the window you were in, and that window never loses the cursor. Escape cancels and inserts nothing.

## Why Parlo

- **Free and open source.** Apache-2.0, no account, no subscription.
- **Private.** Local mode makes no network call at all. A cloud provider is reached only if you allow it and store your own key.
- **Simple.** One shortcut, one job: put the sentence you meant in the field you were in.
- **Another language when you want it.** Dictate Portuguese into an English field, or the other way round, through models on your own machine.
- **Live, dubbing and meetings** are in the same window: text while you speak, a recording dubbed with subtitles, and a transcript with a speaker and a time on every line.

## Install

Download the latest release from the [releases page](https://github.com/PedroNascimento09/Parlo/releases/latest).

**Windows.** Run `Parlo-<version>-Setup.exe`. It installs into your user folder and adds a Start menu entry. The build is unsigned, so SmartScreen asks once: **More info**, then **Run anyway**.

**macOS and Linux**, in any terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

**From source**, with Rust 1.85 or newer and Python 3.12 for the engine:

```bash
cargo build --release -p parlo-cli -p parlo-app
target/release/parlo-app
```

The installer is 44.8 MB. On first start the window offers the recogniser alone, about 640 MB, and nothing else downloads until you ask for it. After that the machine can stay offline.

## Measured

Windows 11 laptop, no GPU, 4 threads: **420 ms** for one segment including recognition, **0.11** real time with Parakeet v3, **18.1 s** cold start with every model. Conditions and raw rows are in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

## Known limitations

- The dictation shortcut is Windows only. On macOS and Linux the text goes to the clipboard.
- Two languages ship today, English and Portuguese. Portuguese to German runs through English and is published as **not validated by a native speaker**.
- Dictation is tested, not yet validated with recordings from people other than the author.
- No meeting summary and no questions over past meetings yet, and meetings record the microphone rather than the sound of another application.
- Read aloud uses a neutral European Portuguese voice that no panel of listeners has judged. No voice is cloned from a recording.

## Docs

[App guide](docs/APP.md) · [Install](docs/INSTALL.md) · [Architecture](docs/ARCHITECTURE.md) · [Benchmarks](docs/BENCHMARKS.md) · [Privacy](docs/PRIVACY.md) · [Troubleshooting](docs/TROUBLESHOOTING.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

## Licence

The code is **Apache-2.0**. Model weights keep their publishers' terms: Parakeet is CC-BY-4.0, the OPUS-MT packages are CC BY 4.0, and the optional Piper adapter is GPL-3.0-or-later, installed separately and never shipped in an archive. [LICENSES.md](LICENSES.md) is the index and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) carries the notices.
