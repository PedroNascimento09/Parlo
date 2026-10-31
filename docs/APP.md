# The app

Parlo's interface is served by the same `parlo` binary you already built. Nothing is installed, there is no driver and no account, and the page runs on `127.0.0.1` in your usual browser.

## Start it

```bash
cargo run --release -p parlo-cli -- app
```

The command prints the token and the address, then opens the browser with the token already in the URL. Keep that terminal window open while you use the interface: closing it stops the app.

## What the screen does

| Control | What happens |
|---|---|
| Main control | Hold it to speak, or click once to start and again to stop. The clip goes to the local server and comes back translated. |
| Input level | Shows the microphone while you speak, so a muted or wrong device is visible before you record. |
| Playback | The translated voice plays back on the same screen as soon as the engine returns it. |
| Status | The engine in use and how long the app has been running. |

## Updates

Stable releases are tags `vX.Y.Z`. Every push to `main` also becomes the `nightly` prerelease, so a change is downloadable within minutes of being committed.

The interface checks the release feed when it starts and shows a banner when a newer release exists, with its notes linked. It never installs anything without a click. Installing that update from inside the app, without uninstalling the old version first, is the next step and does not work yet.

## Privacy

The page talks to two places only: the local server on `127.0.0.1`, and the GitHub releases API for the update check. Your audio goes to the local server and nowhere else. Nothing is uploaded, there is no telemetry, and no account is involved.

## Limitations

- Windows is the only platform validated end to end so far; Linux and macOS are covered by unit tests alone.
- Portuguese and English only.
- Machine translation quality, not human quality.
- The first translation arrives a few seconds after you start speaking.
