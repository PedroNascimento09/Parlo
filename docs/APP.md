# The app

Parlo's interface is served by the same `parlo` binary you already built. Nothing is installed, there is no driver and no account, and the page runs on `127.0.0.1` in your usual browser.

## Start it

Double-click `scripts/parlo-app.cmd` on Windows, or run `scripts/parlo-app.sh` on Linux and macOS: the first run builds the binary, then every run opens the interface for you. From a terminal the same thing is:

```bash
cargo run --release -p parlo-cli -- app
```

The command prints the token and the address, then opens the browser with the token already in the URL. Keep that terminal window open while you use the interface: closing it stops the app.

Add `--from en --to pt` to translate the other way round; the cascade runs one direction per process, so the pair is fixed when the app starts. `parlo translate` and `parlo listen` take the same two flags.

## What the screen does

The page is a single column built around the one thing you do most:

| Part of the screen | What happens |
|---|---|
| Top strip | The running version, and the update banner when a newer release exists, with the notes linked. |
| Record card | The large control at the centre with the input level inside it. Hold it to speak, or click once to start and again to stop; it pulses while it listens, and the clip goes to the local server. |
| Result card | The translated sentence, selectable so a name or a word can be copied, and the translated voice playing back under it. |
| Status group | The engine in use, how long the app has been running, and whether the microphone opened. |
| Models panel | One row per model with its size on disk or `Missing`, and a **Download models** button when something is absent. It never blocks the record card. |

## Updates

Releases are tags `vX.Y.Z`, and only a tagged release reaches a user: nothing is published per commit, so the version in the banner is a release and never a change that happened to land on `main`.

The interface checks the latest published release when it starts and shows a slim banner with its version and a link to the notes. The same banner carries an **Install update** button: it asks the local server, which downloads the build, stages it and replaces the binary a couple of seconds after the server stops, so the page loses its connection for a moment on purpose. Start `parlo app` again and the new version runs. Nothing is installed without that click.

To install that update over the running copy, without uninstalling anything:

```bash
parlo update --check
parlo update
```

The check prints the version it found; the update downloads the build for your system, stages it, and replaces the binary about two seconds after the old process exits. It keeps a copy of the previous binary next to it as `parlo.previous`, so a bad release can be put back by renaming the file.

## Privacy

The page talks to two places only: the local server on `127.0.0.1`, and the GitHub releases API for the update check. Your audio goes to the local server and nowhere else. Nothing is uploaded, there is no telemetry, and no account is involved.

## Limitations

- Windows is the only platform validated end to end so far; Linux and macOS are covered by unit tests alone.
- Portuguese and English only.
- Machine translation quality, not human quality.
- The first translation arrives a few seconds after you start speaking.
