# Cascade scripts

Small Python helpers that run beside the cascade engine. They are not part of the engine
server itself, but they are the practical entry points for the things a person does with
a source checkout: build assets, prepare models, run the benchmark scripts, and dub files.

## What lives here

* `logo_trail.py` — render the Parlo wordmark and a short brand trailer as PNG stills
* `book_preview.py` — render a cover card and an interior page pin-up for the README
* `mic_shield.py` — render a clean microphone-shield icon in light and dark
* `voice_preview.py` — render a neutral voice preview panel in light and dark
* `dub.py` — dub a media file offline and write the exports, subtitles and report
* `exports.py` — subtitle and media export helpers used by the dub script
* `setup_models.py` — download and verify the models the cascade needs
* `mt_gate.py` — score a translation package against a fixed sentence corpus
* `benchmark.py`, `stt_bench.py`, `stream_bench.py`, `mt_bench.py`, `mt_hf_bench.py` — measurement scripts
* `battery.py`, `compare.py`, `soak.py`, `falar_clips.py`, `ingest.py`, `memory.py`, `model_memory.py`, `timing.py`, `separation.py` — supporting scripts

## Requirements

These scripts expect the cascade virtual environment to exist and the models it needs to
have been set up. The asset scripts are the exception: they only need Pillow and no model
downloads.

```bash
python -m pip install Pillow
```

The asset scripts are intentionally offline and small. They are not a substitute for real
screenshots from the running app; they exist so the README and docs can talk about the
interface without pretending a placeholder is a live capture.

## Common commands

Render the README assets:

```bash
python engines/cascade/scripts/logo_trail.py
python engines/cascade/scripts/book_preview.py
python engines/cascade/scripts/mic_shield.py
python engines/cascade/scripts/voice_preview.py
```

Each script prints the files it wrote. The outputs live in `assets/` by default, and each
script accepts `--assets` if you need them elsewhere.

Dub a file from a checkout:

```bash
python engines/cascade/scripts/dub.py video.mp4 --to en
```

The script writes an MP4 beside the input and an SRT with the translated lines. Use
`--help` for the full flag list. The dub script needs `ffmpeg` and `ffprobe` on `PATH`.

## Adding a script here

Keep them small and honest. If a script touches models, mark which ones it needs. If it
produces images, make it clear whether the output is a real capture or a stand-in. If it
scores something, record the conditions next to the number rather than describing the
result as if it were a product claim.
