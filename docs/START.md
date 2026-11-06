# Start here

Five minutes from a computer that has never run Parlo to a sentence you dictated. Everything here happens on your machine: no account, no upload, no terminal tricks.

## 1. Install Parlo

**Windows.** [Download Parlo-Setup.exe](https://github.com/PedroNascimento09/Parlo/releases/latest/download/Parlo-Setup.exe) and double click it. The build is unsigned, so SmartScreen asks once: choose **More info**, then **Run anyway**. Parlo installs for your user and the window opens when it finishes.

**macOS and Linux.**

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

The archive, the Windows script without an installer and building from source are all in [Install](INSTALL.md).

## 2. Open the window and download the models once

```bash
parlo app
```

The program carries the engine, not the weights. The Models panel lists what is missing; press **Download models** and leave it alone for a few minutes, about 800 MB on a connection you already have. It happens once, and after it Parlo works with the network switched off.

If `parlo` is not on your PATH, the **Parlo** entry in the Start menu opens the same window.

## 3. Check what the machine can do

```bash
parlo doctor
```

It prints the platform, the capture and engine formats, the latency budget, the profile it recommends, where the models and the config live, and whether the engine binary answers. Nothing in it is sent anywhere.

## 4. Dictate your first sentence

```bash
parlo dictate
```

Speak, wait for the line to settle, and the sentence is on the clipboard. Paste it wherever you were writing. `--seconds 10` listens longer, `--no-copy` leaves it on screen only, and `--from en` tells it you are speaking English.

The same engine answers the shortcut: `Ctrl+Alt+D` opens a small island over whatever you are typing in, and the sentence lands in that window instead of the clipboard.

## What you have now

| You want | Where it is |
|---|---|
| Translate what you say, live, in two columns | `parlo listen --play` |
| Captions over a call or a video | `Ctrl+Alt+C`, or `parlo captions` |
| A dubbed video with subtitles beside it | `parlo dub talk.mp4 --srt talk.srt` |
| A voice you recorded, its sample and its consent record | the Voice panel in `parlo app` |
| Meetings, transcripts and the history | the Meetings and Library panels |

## Where to read next

- [Benchmarks](BENCHMARKS.md) has every number this page does not claim, with the machine and the conditions beside it.
- [Privacy](PRIVACY.md) shows how to check, command by command, that no audio and no telemetry leaves the machine.
- [Troubleshooting](TROUBLESHOOTING.md) covers the microphone Windows blocks, a missing engine and a model that will not download.
- [App guide](APP.md) walks the five panels of the window, and [Interface](UX.md) explains the design decisions behind them.

If a step here does not work on your machine, that is a bug worth reporting: [open an issue](https://github.com/PedroNascimento09/Parlo/issues) with the output of `parlo doctor`.
