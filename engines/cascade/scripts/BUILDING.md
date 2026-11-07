# Building the cascade assets and scripts

This directory contains small standalone Python helpers. They are not part of the engine
server, and they are not required to run Parlo. They exist so the repository can regenerate
its own README and docs images, and so a source checkout has a practical path to dub files
and measure the engine.

## What this file is for

If you are cloning the repository and want the scripts to run, this is the place to look
before you chase a missing dependency. If you are trying to understand why an image or a
dub command is failing, start here.

## What you need

For the asset scripts only:

* Python 3.10 or newer
* Pillow

For the dub and measurement scripts, add the cascade virtual environment and the models it
expects. The asset scripts do not need models and do not download anything.

## First time setup

```bash
python -m venv .venv
.venv/bin/pip install -r engines/cascade/requirements.txt
.venv/bin/pip install Pillow
```

On Windows use `.venv\Scripts\python` and `.venv\Scripts\pip`.

## Running the asset scripts

From the repository root:

```bash
python engines/cascade/scripts/logo_trail.py
python engines/cascade/scripts/book_preview.py
python engines/cascade/scripts/mic_shield.py
python engines/cascade/scripts/voice_preview.py
```

Each script writes its outputs into `assets/` by default. They are standalone: a script
fails early with a plain message if Pillow is missing, and it does not reach for the
network.

## What each script is intended to be

* `logo_trail.py` writes the Parlo wordmark and a sequence of trailer stills. The trailer
  is a small brand clip made from the same colour tokens as the app. It is a stand-in for a
  real rendered trailer until there is one.
* `book_preview.py` writes a cover card and an interior page pin-up. They are used to talk
  about a book-like preview without needing a real printed mock-up.
* `mic_shield.py` writes a small microphone badge in light and dark. It is a line icon, not
  a photo, so it stays clean at small sizes.
* `voice_preview.py` writes a neutral voice preview panel in light and dark. It shows the
  shape of the panel, not a real user waveform.

## Dubbing from a checkout

```bash
python engines/cascade/scripts/dub.py video.mp4 --to en
```

The script needs `ffmpeg` and `ffprobe` on `PATH`. It writes an output file beside the
input and an SRT with the translated lines. If you want the full list of flags, run
`python engines/cascade/scripts/dub.py --help`.

## If something is missing

If a script complains about a missing module, install only what it names. If it complains
about a missing model, that script probably belongs to the engine side and needs the models
to have been set up first. If an image looks wrong, the first suspect is the font fallback:
the asset scripts try a small list of common system fonts and fall back to the default if
none of them are present.
