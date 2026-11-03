# Troubleshooting

## `cargo` is not installed

Install Rust from [rustup.rs](https://rustup.rs). Parlo needs 1.83 or newer:

```bash
rustup show
```

## The engine was not found after unpacking the archive

A release archive holds three things: `parlo` (or `parlo.exe`), a directory `parlo-engine`, and `README.md`. The translator lives in that directory, so running `parlo` on its own, or moving the binary somewhere else, leaves it with nothing to translate with. The error is `cannot start ... in engines/cascade`.

Keep the three items together in the same folder and start from there:

```bash
parlo app
```

If the engine has to live elsewhere, point at it:

```bash
PARLO_ENGINE=/path/to/parlo-engine/parlo-engine parlo translate in.wav out.wav --engine cascade
```

On Windows the same thing is `set PARLO_ENGINE=C:\path\parlo-engine\parlo-engine.exe` before the command. From a source checkout there is no `parlo-engine` directory and Parlo uses `python serve.py` instead, which needs the venv from [the cascade guide](../engines/cascade/README.md).

## The app says the models are missing

The archive does not carry the models; they are about 800 MB and are downloaded once into the per-user data directory:

| System | Models |
|---|---|
| Windows | `%APPDATA%\parlo\models` |
| macOS | `~/Library/Application Support/parlo/models` |
| Linux | `$XDG_DATA_HOME/parlo/models`, or `~/.local/share/parlo/models` |

The **Models** panel in `parlo app` starts the download, and the panel keeps saying so while it runs. To fetch them without the app, run the engine by hand:

```bash
parlo-engine/parlo-engine.exe --download-models
```

`PARLO_MODELS_DIR` points both the app and the engine at another folder, which is useful when the models live on a different drive.

## An update left two versions of the engine

`parlo update` replaces the running binary in place. If the archive was unpacked by hand rather than installed by the updater, replace the whole folder, not only `parlo`: an old `parlo-engine` next to a new `parlo` works, but it runs the older engine, and the version it reports is the one printed at startup.

Delete the old archive folder, unpack the new one, and keep `parlo`, `parlo-engine` and `README.md` together.

## `parlo install` copied a binary without an engine

`install` copies the binary that is running, so from a source checkout it installs `target/release/parlo`, which has no `parlo-engine` folder beside it and says so: `engine none beside this binary, install from a release archive to get one`. The installed copy then falls back to `python serve.py` in the checkout, which only works while that checkout exists.

Install from an unpacked release archive instead, where the folder is next to the binary, or copy `parlo-engine` into the installed directory by hand and run the installed `parlo` again.

## Uninstalling kept a folder with every model

That is on purpose. `parlo uninstall` removes the program and its menu entry and leaves the data directory, models included, because downloading 800 MB again after changing your mind is worse than a folder you can delete yourself. Nothing else reads it once the program is gone.

To remove the models and the config in the same step:

```bash
parlo uninstall --purge
```

## The data directory is not writable

Without a writable data directory there is nowhere to put the config file or the models, and the download fails at once. Check the folder named in the table above, or choose another one:

```bash
PARLO_DATA_DIR=/path/with/write/access parlo app
```

`PARLO_DATA_DIR` moves the config and the models together; `PARLO_MODELS_DIR` moves only the models.

## The engine prints `warmup deferred`

A line like `warmup deferred for voice: ...` on stderr means a model could not be loaded while the engine started. The engine keeps running and loads it on the first request, which then reports its own error if the model is genuinely unusable. It is not fatal by itself, and it is expected in some packaged builds.

The two lines worth reading in a failure are the `error:` line the CLI prints, which says what failed, and the line right before it on stderr, which comes from the engine and usually names the missing file or the device. `parlo doctor` reports the platform, the profile and the audio setup, and belongs in a bug report next to both lines.

## `parlo doctor` says "engine: echo (no model installed)"

Expected. The prototype ships with the echo engine so latency and framing can be tested without downloading weights. Model integration is tracked in [ROADMAP.md](../ROADMAP.md).

## `error: input must be mono pcm16 audio`

The CLI accepts mono 16 bit WAV. Convert with ffmpeg:

```bash
ffmpeg -i input.wav -ac 1 -ar 48000 -c:a pcm_s16le mono.wav
```

## `error: unsupported wave encoding: format 3, 32 bits`

The file is IEEE float instead of PCM16. Convert it with the command above.

## The microphone sounds loud in Discord but the recording is quiet

Chat apps apply automatic gain control, so the same microphone sounds loud there and quiet in a raw file. `parlo record` captures what the operating system delivers and then normalises towards -1 dBFS with a +24 dB ceiling, printing the peak and the gain it applied, for example `gain +24.0 dB`. `parlo listen` applies automatic gain inside the pipeline, after the voice gate, so the engine receives a usable level whatever the device delivers.

A laptop Realtek microphone measured raw speech between -24.7 and -32.2 dBFS peak, with speech in 100 ms windows between -39.7 and -46.2 dBFS, against a room noise floor between -63.9 and -67.0 dBFS. The voice gate opens at -55 dBFS, which sits above that noise floor and below normal speech.

No gain line means the signal already reached full scale and nothing needed boosting. If the gate never opens, the input is muted, `record --list` shows a different device than the one you speak into, or Windows is denying desktop apps access to the microphone.

Check the input device and its volume in Settings > System > Sound > Input, allow desktop apps in Settings > Privacy > Microphone, and compare the printed gain line rather than trusting the speaker volume, because the speaker hides a quiet capture.

## The recording stops because the device went away

Unplugging a headset or a USB interface ends the audio stream the system gave Parlo. The command then fails with `the input device stopped: ...` instead of writing a short or silent file, and the app reports the same sentence. Plug the device back in, run `parlo record --list` to see the index that came back, and start again; a device that returns often gets a new index, which is why `parlo config --device N` exists.

## `record --system` writes silence while sound is playing

The command records the default speakers, so it hears nothing when the audio goes to another output, when that output is muted, or when the program you are watching owns the device in exclusive mode, which leaves the shared loopback stream empty. Check the output in Settings > System > Sound, unmute it, and run `parlo record --system out/system.wav --seconds 5` again while something plays. The printed peak should read about -20 dBFS or higher, and -120 dBFS means the shared stream heard nothing. Capturing the speakers works on Windows only, because it uses WASAPI loopback; macOS and Linux answer that the platform is missing.

## The output file is silent

The voice gate closed because the input was below the threshold. Check the level:

```bash
ffmpeg -i input.wav -af volumedetect -f null -
```

If the mean volume is below about -55 dB, raise the recording level or lower `gate_threshold_db` in `PipelineConfig`.

## `demo` reports fewer voiced frames than expected

Synthetic speech has deliberate pauses. Only frames above the gate threshold are counted as voiced, which is the behaviour the pipeline uses in a real call.

## Audio sounds metallic or aliased

The current resampler uses linear interpolation. Speech at 16 kHz is usually acceptable, but content with strong high frequencies above 8 kHz will alias. A band-limited resampler is on the roadmap.

## Latency is higher than the printed budget

The budget prints the declared engine latency plus one frame of capture. Real end-to-end latency also includes the model's startup, the audio device buffers, and the operating system scheduler. Measure it as described in [BENCHMARKS.md](BENCHMARKS.md).

## The virtual microphone does not appear in Zoom, Meet, or Discord

It is not shipped yet, and on Linux it has to be created manually. See [VIRTUAL_MIC.md](VIRTUAL_MIC.md). On macOS and Windows the current path requires an existing virtual audio device.

## The other side hears an echo or a double translation

The translated voice is re-entering your microphone. Use headphones, and keep the translated-only output mode. See [VIRTUAL_MIC.md](VIRTUAL_MIC.md#feedback-loops).

## Build fails with a linker error on Windows

Install the MSVC build tools and use the `stable-x86_64-pc-windows-msvc` toolchain. The GNU toolchain works too, but MSVC matches the release builds.

## Nothing here solved it

Open an issue with the output of `parlo doctor`, your operating system, and the exact command you ran.
