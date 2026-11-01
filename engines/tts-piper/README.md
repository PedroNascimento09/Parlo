# parlo-tts-piper

The optional speech synthesis adapter. It is a separate program from `parlo-engine`: the engine never imports `piper`, it starts this process and exchanges one JSON object per line with it.

## Licence

This directory is **GPL-3.0-or-later**, because it depends on `piper-tts` and the eSpeak NG data that package carries. That is the licence of this component, not of the engine.

The engine is distributed under its own licence and does not contain any code from this directory. Nothing here is legal advice: whether a specific distribution of two cooperating programs forms one combined work depends on how they are shipped, installed and coupled. See [docs/LICENSES.md](../../docs/LICENSES.md).

## Install

```bash
python -m venv .venv-tts
.venv-tts/bin/python -m pip install -r engines/tts-piper/requirements.txt
PYTHONPATH=. python engines/tts-piper/serve.py
```

On Windows use `.venv-tts/Scripts/python`.

The engine looks for this directory in `engines/tts-piper`, or wherever `PARLO_PIPER_DIR` points. It uses the interpreter that runs the engine, the one in `PARLO_PIPER_PYTHON`, or a virtualenv it can find. Without this adapter the engine still recognises, translates and streams text; only the spoken voice is unavailable, and the engine says so.

## Voices

Voices are downloaded, never shipped:

```bash
python engines/cascade/scripts/setup_models.py --voice pt_pt
```

Each voice has its own terms, published by its author. The catalogue records the licence of each voice separately from the licence of this code.

## Protocol

One JSON object per line on stdin and stdout.

```json
{"op": "hello"}
{"op": "hello", "engine": "piper", "available": true}

{"op": "synthesize", "text": "bom dia"}
{"op": "synthesized", "pcm16": "<base64 little endian pcm16>", "sample_rate": 22050}
```

An error answers `{"op": "error", "error": "..."}` and the process stays alive.
