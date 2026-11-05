# parlo-cascade

The default Parlo engine: streaming speech recognition, translation, and speech synthesis, all on CPU, all under permissive licences.

| Stage | Component | Licence |
|---|---|---|
| Voice detection | Parlo pipeline gate (RMS with hangover) | MIT |
| Speech recognition | Parakeet TDT 0.6B v3 int8 via sherpa-onnx | CC-BY-4.0 model, Apache-2.0 runtime |
| Translation | Argos Translate (OpenNMT/CTranslate2) | MIT |
| Speech synthesis | Piper (`pt_PT-tugão-medium`, `en_US-lessac-medium`) | **GPL-3.0-or-later** |
| Media page download | yt-dlp | Unlicense |

Piper is not MIT. The PyPI package `piper-tts` is GPL-3.0-or-later, and each voice model carries its own terms. Parlo's own code is Apache-2.0. The engine imports no Piper: synthesis is the optional adapter in [engines/tts-piper](../tts-piper), started as a separate process and driven over JSON lines, so `parlo-engine` contains no GPL code and the adapter is where the GPL applies. [docs/LICENSES.md](../../docs/LICENSES.md) states the split and its limits.

A permissive synthesiser would let the whole distribution stay MIT. Kokoro (Apache-2.0) is the first candidate to evaluate.

No model here is non-commercial, which is why this is the default and Hibiki-Zero is not.

## Install

```bash
python -m venv .venv
.venv/Scripts/python -m pip install -r engines/cascade/requirements.txt
.venv/Scripts/python engines/cascade/scripts/setup_models.py
```

On Linux and macOS replace `.venv/Scripts/python` with `.venv/bin/python`.

The setup script downloads the Piper voices, the Parakeet v3 model, and the Argos Portuguese to English packages. Add `--stt whisper` to cache faster-whisper instead, or `--stt both`. Everything lands in `engines/cascade/models` and the user's model cache, both outside Git.

```bash
python engines/cascade/scripts/setup_models.py --stt both --whisper small
python engines/cascade/scripts/setup_models.py --stt whisper --whisper medium
```

The first installs Parakeet and caches Whisper `small` so the two can be compared; the second caches only `medium`.

## Run

```bash
parlo translate in.wav out.wav --engine cascade
```

The CLI looks for `.venv` next to the engine, in `engines/`, or at the project root, and falls back to `python` on the path. `PARLO_PYTHON` overrides that search.

| Variable | Default | Purpose |
|---|---|---|
| `PARLO_PYTHON` | detected `.venv`, else `python` | Interpreter with the cascade requirements installed |
| `PARLO_CASCADE_DIR` | `engines/cascade` | Directory containing `serve.py` |
| `PARLO_CASCADE_STT` | `parakeet` | Recogniser backend, `parakeet` or `whisper` |
| `PARLO_CASCADE_MODEL` | `base` | Whisper size when `PARLO_CASCADE_STT=whisper`: `tiny`, `base`, `small`, `medium`, `large-v3` |
| `PARLO_CASCADE_THREADS` | `4` | Threads used by the recogniser |
| `PARLO_CASCADE_SEGMENT` | `2.5` | Seconds of speech buffered before a segment is translated |

## How it behaves

Audio arrives in 20 ms frames. The engine buffers them until it has at least 2.5 seconds of speech, then transcribes, translates, synthesises, and returns the translated audio in one response. Frames that only fill the buffer return empty audio, which the pipeline treats as "nothing to play yet".

`PARLO_CASCADE_SEGMENT` changes that window. It is the single most important setting for keeping up with a conversation, because the recogniser pays a fixed cost per call:

| Window | Real time factor on a 12 core CPU |
|---|---|
| 1.2 s | 2.66, backlog grows |
| 2.5 s | 0.96, roughly keeps up |
| 4.0 s | 1.15, one slow call |

With Parakeet the window is a latency choice, not a speed one. Same 4.95 s clip, 4 threads:

| Window | RTF | Cost |
|---|---|---|
| 1.0 s | 0.131 | loses words |
| 1.5 s | 0.111 | loses content words |
| 2.0 s | 0.108 | usable, drops a filler |
| 2.5 s | 0.094 | default, keeps the whole sentence |

The recogniser matters more than the window. With 4 threads, which approximates an ordinary laptop, on 4.95 s of audio:

| Recogniser | RTF, 2.5 s window | RTF, whole utterance |
|---|---|---|
| parakeet v3 (default) | **0.113** | 0.092 |
| whisper base | 0.309 | 0.153 |

Peak resident memory for a run is about 1.3 GB, of which 640 MB is the recogniser. Parakeet is the default because it is both faster and more accurate. `PARLO_CASCADE_STT=whisper` switches back, and only then does `PARLO_CASCADE_MODEL` matter: `tiny`, `base`, `small`, `medium`, or `large-v3`, all int8 on CPU through faster-whisper. An unknown size stops the server at startup with the list of supported ones instead of downloading something unexpected. Translation costs under 0.05 s and synthesis 0.28 s on a five second block; recognition was the whole bill and no longer is.

That comparison covers recognition alone. On this project's own recordings the whole cascade, translation and synthesis included, measured mean RTF 0.45 with Parakeet and 1.27 with Whisper base over ten 6 second clips from a laptop microphone, Windows 11, 4 threads.


This is the "fast consecutive" mode from the plan, not simultaneous speech.

## Warm-up and glossary

Models load at startup, so the first sentence is not the one that waits: startup takes 12 to 21 seconds and prints progress on stderr. `PARLO_CASCADE_WARMUP=0` disables it.

`glossary.txt` maps names the recogniser gets wrong, one `wrong=right` per line, applied to the transcript before translation. The default already fixes `Palo` and `Paul's` into `Parlo`.

## Limits

- One direction per process, set by `PARLO_CASCADE_SOURCE` and `PARLO_CASCADE_TARGET`: Portuguese to English by default, English to Portuguese when reversed.
- The first request after start is slow because the models load lazily; warm-up lands in v0.2.
- The translated voice is neutral, not yours. Levels 2 to 4 are separate work.

## Reference battery

`scripts/battery.py` keeps recognition honest with a number instead of an impression. It holds twenty-four Portuguese sentences with names, numbers and dates, synthesises each one with the local Piper voice, and recognises it in three conditions: clean, 20 dB signal to noise, and 10 dB. Noise is drawn from a seeded generator, so a run is reproducible, and the score is the word error rate against the sentence text.

```bash
python engines/cascade/scripts/battery.py --out out/battery.json
python engines/cascade/scripts/battery.py --limit 5
python engines/cascade/scripts/battery.py --stt whisper --model small
```

The number covers recognition only: translation quality and the generated voice are not scored, and `--stt whisper` is the way to compare recognisers on the same sentences. A clean mean above 0.5 makes the script exit non-zero, so it can gate a build later.

Synthetic speech is a fair stand-in for a regression test, not for a user: it shares one voice, one accent and no room, no microphone and no human hesitation. A recorded set from several speakers and microphones is still required before v1.0, and this battery does not replace it.

## Dubbing

`scripts/dub.py` uses the same recogniser, translator and voice on a whole file instead of a live stream: ffmpeg decodes to 16 kHz mono, the audio is split on pauses with a 15 s ceiling, every segment is recognised, translated and spoken, and each line is placed at the time its sentence started. A line longer than its slot is spoken again up to about 18 percent faster, and overlaps the next one if that is not enough. An imported cue can carry a pace from 0.8 to 1.25; the renderer passes its inverse to Piper as `length_scale`, and the project keeps the chosen pace.

```bash
python engines/cascade/scripts/dub.py video.mp4 --to en
python engines/cascade/scripts/dub.py talk.wav --from en --to pt --keep-background
```

It writes the media file plus an `.srt`, and `--json` adds the times and both texts per line. `ffmpeg` and `ffprobe` must be on `PATH`.

A public video page is fetched with yt-dlp after the same public-address check as a direct file. Private, age-gated and members-only videos are refused, and no cookies are read. The page's own media redirects are followed by yt-dlp, not re-checked by Parlo. Details, measured output and limits: [../../docs/DUB.md](../../docs/DUB.md).

## Tests

```bash
cd engines/cascade
python -m unittest discover -s tests
```

The tests cover the numeric helpers and the engine identity; model quality is measured separately, never asserted in unit tests.
