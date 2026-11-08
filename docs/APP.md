# The app

Parlo opens as a window of its own. The window, `parlo-app`, is a thin native shell around a page that the `parlo` binary serves from `127.0.0.1`; there is no address bar because there is no browser, nothing is installed, and no account is involved.

## Start it

From a release, unzip the archive and double-click `parlo-app`, which is `parlo-app.exe` on Windows. The archive carries three things: `parlo`, the command line; `parlo-app`, the window; and a folder named `parlo-engine`, the self-contained translation engine with its libraries, about 900 MB once unpacked. The engine folder must stay next to the other two, because the window looks for it beside itself, and no Python is involved. From a terminal, `parlo app` opens the window, which has to sit beside `parlo`; without it the command says so instead of opening a page. Models are not in the archive: the Models panel downloads them on first use into `%APPDATA%\parlo\models` on Windows, `~/Library/Application Support/parlo/models` on macOS, or `$XDG_DATA_HOME/parlo/models` elsewhere, and `PARLO_MODELS_DIR` moves them. That first download is roughly 800 MB, so the first launch needs a network connection once, and everything works offline afterwards. The config file sits beside that folder.

To keep the release somewhere permanent, `parlo install` copies both programs and the engine folder into `%LOCALAPPDATA%\Programs\Parlo` on Windows, `~/Applications/Parlo` on macOS or `~/.local/opt/parlo` on Linux, and writes one menu entry that opens the window rather than a console. `--dir` installs somewhere else and `--no-shortcut` skips the entry. `parlo uninstall` removes both and keeps your models, because redownloading 800 MB on a whim is not a kindness; `--purge` removes them as well.

From a source checkout, double-click `scripts/parlo-app.cmd` on Windows, or run `scripts/parlo-app.sh` on Linux and macOS: the first run builds the programs, then every run opens the window. From a terminal the same thing is:

```bash
cargo build --release -p parlo-cli -p parlo-app
target/release/parlo-app
```

The window starts the server behind it and stops it on the way out, so no terminal has to stay open. Both programs have to be built, because the window looks for `parlo` beside itself.

The language pair and the recogniser are chosen inside the page. `parlo app --from en --to pt --stt whisper` writes those values to the config file and opens the window, which reads them at start.

## The window

`parlo-app` does one thing before the page appears: it starts `parlo serve` on a free loopback port with a fresh token, waits for the local server to answer `/health`, and only then loads the page with the token already in the address. The page, the endpoints and the behaviour described below are the whole app.

While the engine loads, the window shows a quiet starting screen with a pulsing mark. A cold start measured about 18 seconds on the reference machine, and the window waits up to four minutes before giving up, which leaves room for a first run that is still fetching models.

If the engine never answers, a failure screen replaces it: close Parlo and open it again, and if it repeats, the reason is in `parlo-app.log` and `parlo-engine.log` in the system temporary folder, `%TEMP%` on Windows.

Closing the window stops the engine. If the window's own process is killed outright, the engine sees the pipe to its parent close and exits by itself, so no copy is left holding a port.

The window needs the system webview: WebView2 on Windows, which Windows 10 and 11 already have through Edge, and WebKitGTK on Linux. It is not packaged as a `.msi` or a `.dmg` yet, so a release is still an archive that is unzipped and opened.

## What the screen does

The page is a single column built around the one thing you do most:

| Part of the screen | What happens |
|---|---|
| Top strip | The running version, and the update banner when a newer release exists, with the notes linked. |
| Record card | The large control at the centre with the input level inside it. Hold it to speak, or click once to start and again to stop; it pulses while it listens, and the clip goes to the local server. |
| Live toggle | Keeps listening instead of recording one clip, translating a window at a time and speaking each one in order. Details below. |
| Result card | The translated sentence, selectable so a name or a word can be copied, and the translated voice playing back under it. |
| Settings | Language pair and recogniser, loaded from the server and applied with one button. Changing either reloads the models. |
| Status group | The engine in use, how long the app has been running, and whether the microphone opened. |
| Models panel | One row per model with its size on disk or `Missing`, and a **Download models** button when something is absent. It never blocks the record card, the first run of a released archive starts here, and while the download runs the panel says it is working instead of looking stuck. |
| Dub panel | Drop a video or audio file on the page, or pick one, and the local server dubs it into the other language. The translated lines appear while it works and the new file downloads from the same panel. |

## Live mode

The `Live` toggle beside the record control turns the page into an interpreter that never stops listening. It cuts the microphone audio into windows of about 2.5 seconds, sends one window at a time to the local server, and plays each translated window in order, without ever overlapping the previous one. The translated sentences stack up in the page, so one that scrolled past can still be read, and toggling it off stops the microphone and the playback.

Every window costs its own 2.5 seconds plus the processing time published in [BENCHMARKS.md](BENCHMARKS.md), so the wait per sentence is about three seconds. Live mode cannot remove that window: it is how the recogniser knows a sentence ended. It refuses to start while a file is dubbing, and it needs the models installed, because there is nothing to translate with until they are there.

Live mode is not the way into a call. Neither a window nor a page can present itself as a microphone, so until the virtual microphone ships the practical route on Windows and macOS is a virtual cable with `parlo listen --play --speaker N`, described in the README.

## Languages and the recogniser

The settings row offers the two languages whose models ship, Portuguese and English, in either direction, and the two recognisers: **Parakeet**, the default, which measured faster and more accurate on this project's test audio, and **Whisper**, smaller, for a machine that cannot spare the 640 MB. The list of languages is deliberately short: offering a pair without its Argos package and its voice would be a button that fails when pressed.

Applying a change unloads the models and loads the new ones, which takes about twenty seconds on the reference machine, and the page says so while it happens rather than looking frozen. Everything else on the screen keeps working through the wait.

The same choice from a terminal, without opening the page:

```bash
parlo config --from en --to pt --stt whisper
```

## The dictation shortcut

The island opens and closes on **Ctrl+Alt+D**. To use other keys, write them once and restart Parlo:

```
parlo config --shortcut "Ctrl+Alt+Shift+D"
```

The island settles at the bottom of the monitor where the cursor is, so on a two-screen desk it follows the screen you are working on instead of the middle of another one. The key has two gestures: a tap opens the island with its choices, and holding it starts the dictation at once, with the microphone open from the moment the key goes down, so the first word is not lost.

Three shortcuts live beside each other: the dictation key, `Ctrl+Alt+Shift+D` for hands free and `Ctrl+Alt+C` for captions. Each can be typed in its settings row, which validates it, refuses a combination another of the three already uses, refuses one Windows reports as taken by another program, and stores it. Ctrl, Alt, Shift or Win plus a letter, a digit, a function key or Space is accepted, and a bare key is refused because a global shortcut without a modifier would swallow typing everywhere. The new keys are registered the next time the window starts; until then the old ones still work, and the window log names whatever it registered, including the case where another program already owns the combination.

## What dictation leaves out

People correct themselves while they talk, and the dictated text should be the sentence they meant rather than every word they said. Three modes decide how much of that is done, and the engine takes one of them from `PARLO_CASCADE_CLEAN`: `original` writes what the recogniser heard, `light` is the default and removes self-corrections, abandoned sentence starts, hesitations and accidental repetitions, and `intelligent` adds tidier spacing and a capital letter at the start.

A correction is applied when the words after «quer dizer», «afinal», «não, espera», «ou melhor» or «corrigindo» replace what came before: «Quero comprar 2 bilhetes para o cinema, quer dizer, 3 bilhetes para o cinema» becomes «Quero comprar 3 bilhetes para o cinema». When the second half adds something instead of replacing it, as in «Gosto de cinema, quer dizer, gosto especialmente de filmes de ficção científica», both halves stay and only the marker goes, because dropping an idea is worse than leaving a hesitation. The sentence that was heard is always kept beside the cleaned one, so nothing is destroyed, and `original` brings all of it back.

The cleanup runs on text alone, after recognition and before anything is inserted: no model, no network, microseconds. It only knows the markers it was taught, in both supported languages, so a correction said without a marker ("two tickets… three tickets") is left alone.

## Dub a file

The dub panel is for recordings, not for a call: drop a video or an audio file on the page, choose the target language, and press the dubbing control. The request goes to the local server on `127.0.0.1`, which runs the same offline pipeline as `parlo dub`, so the file never leaves the machine. It takes minutes on a long file, the panel keeps counting while it works, and the finished media and its `.srt` download from the page when it ends.

`ffmpeg` and `ffprobe` must be on `PATH`, and the models must be installed; the panel says so instead of failing halfway. What file mode can and cannot do, with measured numbers: [DUB.md](DUB.md).

## Updates

Releases are tags `vX.Y.Z`, and only a tagged release reaches a user: nothing is published per commit, so the version in the banner is a release and never a change that happened to land on `main`.

The interface checks the latest published release when it starts and shows a slim banner with its version and a link to the notes. The same banner carries an **Install update** button: it asks the local server, which downloads the build, stages it and replaces the `parlo` binary a couple of seconds after the engine stops. Nothing is installed without that click, and nothing has to be uninstalled either.

That replacement is not silent, to be plain about it: the window loses its connection to the server on purpose, so close Parlo and open it again and the new version runs. The window program itself is not replaced by that button yet, only the binary behind the page, so a change to the interface arrives without a reinstall while a change to the window needs the newer archive.

To install that update over the running copy, without uninstalling anything:

```bash
parlo update --check
parlo update
```

The check prints the version it found; the update downloads the build for your system, stages it, and replaces the binary about two seconds after the old process exits. It keeps a copy of the previous binary next to it as `parlo.previous`, so a bad release can be put back by renaming the file.

## First run

A released archive starts with no models. The page loads, the record card says the microphone is ready, and the Models panel lists what is missing with one **Download models** button: about 800 MB into the user data folder, once, after which the machine can be offline for good. Nothing needs to be installed, and the uploads never happen.

If dubbing is what you came for, `ffmpeg` must also be on `PATH`; the dub panel says so in a sentence instead of failing halfway through a file.

## Privacy

The page talks to two places only: the local server on `127.0.0.1`, and the GitHub releases API for the update check. Your audio goes to the local server and nowhere else. Nothing is uploaded, there is no telemetry, and no account is involved.

## Limitations

- Windows is the only platform validated end to end so far, the window included; macOS and Linux are compiled and tested by CI, but `parlo-app` has not been run there.
- Portuguese and English only.
- Machine translation quality, not human quality.
- The first translation arrives a few seconds after you start speaking, and live mode keeps that wait for every window.
- Live translation exists in the page, but a call app still sees the system microphone: routing the translated voice into Zoom, Meet or Teams needs the virtual cable today.
