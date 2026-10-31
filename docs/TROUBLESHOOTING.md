# Troubleshooting

## `cargo` is not installed

Install Rust from [rustup.rs](https://rustup.rs). Parlo needs 1.83 or newer:

```bash
rustup show
```

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
