# Dubbing a file

Live mode translates while you speak. File mode does the same thing without a clock: it reads a recording, translates every sentence, and writes a new file with a voice in the other language. It needs no virtual microphone, no driver and no NVIDIA card, which makes it the first part of Parlo most people can actually use on a call recording or a video.

Use it when you have the file already, when the result can take minutes instead of seconds, or when you want subtitles as well. Use [the app](APP.md) or `parlo listen` when you need the translation while the conversation happens.

## Commands

```bash
parlo dub video.mp4 --to en
```

That writes `video.en.mp4` next to the input and `video.en.srt` with the translated lines. From a source checkout:

```bash
.venv/bin/python engines/cascade/scripts/dub.py video.mp4 --to en
```

On Windows use `.venv/Scripts/python`. The other direction is `--to pt --from en`.

| Option | Default | Purpose |
|---|---|---|
| `--to` | `en` | Language of the new voice |
| `--from` | `pt` | Language spoken in the file |
| `--out` | the input name with the target language appended | Where the dubbed file goes |
| `--srt` | the output name with `.srt` | Where the subtitles go |
| `--keep-background` | off | Keep the original music and effects, ducked under each translated line |
| `--json` | off | Write every line with its times, source text and translation |

The input can be a video (`mp4`, `mkv`, `mov`, `webm`, `avi`, `m4v`) or an audio file. A file without a video stream produces audio only.

## Requirements

- `ffmpeg` and `ffprobe` on `PATH`. They do the decoding, the encoding and the muxing; Parlo does not link a media library.
- The cascade models, installed once:

```bash
.venv/bin/python engines/cascade/scripts/setup_models.py
```

The recognition backend is the same one the live path uses: Parakeet by default, Whisper with `PARLO_CASCADE_STT=whisper` and `PARLO_CASCADE_MODEL`. See [../engines/cascade/README.md](../engines/cascade/README.md).

## How it works

1. ffmpeg decodes the file to 16 kHz mono, the only format the cascade accepts.
2. The audio is split into sentences on pauses of about 0.28 s, with a 15 s ceiling so a long monologue still fits one recognition call.
3. Each segment is recognised, translated with the OPUS-MT package of the pair, and synthesised with the Piper voice of the target language.
4. Every line is placed at the moment the original sentence started. If the translation does not fit its slot, it is synthesised again up to about 18 percent faster.
5. With `--keep-background` the original audio is mixed under the new track, following a ducking envelope built from the line times: 0.55 while nobody speaks, about 0.10 under a translated line, with a 0.15 s fade on each side.
6. ffmpeg copies the video stream unchanged, replaces the audio with the new track, and the translated lines are written as `.srt`.

## Measured

Windows 11, Parakeet TDT 0.6B v3 on 4 threads, one 6.6 s Portuguese clip dubbed to English:

| What | Result |
|---|---|
| Lines produced | 1 |
| Output | MP4, H.264 copied, AAC 192 kbit/s |
| Peak level of the new track | -0.5 dBFS |
| Subtitle | one cue, 00:00:00,040 to 00:00:06,727 |
| Ducking envelope | 1.0 far from a line, 0.18 inside it, 0.317 one fade edge away |

No timing quality metric exists yet: the line starts where the sentence started, and nothing measures whether the dub stays in sync over a long file. No end-to-end duration on a long file has been measured either. The clip above is a synthetic voice, not a person, and one file is not a benchmark.

## Known limitations

- Timing is approximate. Each line starts at the right second, but it is a fresh recording of a different sentence length, so it drifts inside the sentence.
- Fast speech overlaps the next line. If a translation is still longer than its slot after the speed-up, the two lines are audible at once.
- `--keep-background` keeps music and effects, and the original speech with them: the original sits at 0.55 between lines and about 0.10 while the dub speaks, so the music comes back up between sentences but the original voice is audible there too. No source separation is performed.
- Everything runs on the CPU. A long file takes minutes, and the terminal prints one line per segment while it works.
- Machine translation quality, not human quality, and the voice is a neutral one, not the speaker's.
- Portuguese and English only.
- The result is written as AAC in an MP4 container; other containers follow what ffmpeg derives from the output extension.

## Troubleshooting

**`ffmpeg is not installed`**: install it and make sure `ffmpeg -version` works in the same terminal. Nothing is bundled, so a build of Parlo does not carry ffmpeg inside it.

**`the file has no audio to dub`**: the input has no audio stream, or ffmpeg could not decode it. Check with `ffprobe video.mp4` and try the file in a player first.

**`nothing was translated` or `no speech was found in the file`**: the file is music, silence, or too quiet for the segmenter at `0.008` RMS. `ffmpeg -i input.mp4 -af volumedetect -f null -` prints the level; if the mean is very low, raise the volume of the file first.

**A line is cut in the middle of a sentence**: the pause threshold found a gap inside the sentence. `--json` shows where every line starts and ends, which is enough to see where the split happened.
