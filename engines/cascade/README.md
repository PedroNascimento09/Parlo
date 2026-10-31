# parlo-cascade

The default Parlo engine: streaming speech recognition, translation, and speech synthesis, all on CPU, all under permissive licences.

| Stage | Component | Licence |
|---|---|---|
| Voice detection | faster-whisper built-in VAD | MIT |
| Speech recognition | faster-whisper (`small`, int8) | MIT |
| Translation | Argos Translate (OpenNMT/CTranslate2) | MIT |
| Speech synthesis | Piper (`pt_BR-faber-medium`) | MIT |

No model here is non-commercial, which is why this is the default and Hibiki-Zero is not.

## Install

```bash
python -m venv .venv
.venv/Scripts/python -m pip install -r engines/cascade/requirements.txt
.venv/Scripts/python engines/cascade/scripts/setup_models.py
```

On Linux and macOS replace `.venv/Scripts/python` with `.venv/bin/python`.

The setup script downloads the Piper voice, installs the Argos Portuguese to English package, and caches the Whisper `small` weights. Everything lands in `engines/cascade/models` and the user's model cache, both outside Git.

## Run

```bash
PARLO_PYTHON=.venv/Scripts/python.exe parlo translate in.wav out.wav --engine cascade
```

| Variable | Default | Purpose |
|---|---|---|
| `PARLO_PYTHON` | `python` | Interpreter with the cascade requirements installed |
| `PARLO_CASCADE_DIR` | `engines/cascade` | Directory containing `serve.py` |

## How it behaves

Audio arrives in 20 ms frames. The engine buffers them until it has at least 1.2 seconds of speech, then transcribes, translates, synthesises, and returns the translated audio in one response. Frames that only fill the buffer return empty audio, which the pipeline treats as "nothing to play yet".

This is the "fast consecutive" mode from the plan: usable, honest, and about 3 to 5 seconds behind on a medium CPU. Simultaneous speech with wait-k policies arrives in v0.3.

## Limits

- One direction per process, set by `PARLO_CASCADE_SOURCE` and `PARLO_CASCADE_TARGET`: Portuguese to English by default, English to Portuguese when reversed.
- The first request after start is slow because the models load lazily; warm-up lands in v0.2.
- The translated voice is neutral, not yours. Levels 2 to 4 are separate work.

## Tests

```bash
cd engines/cascade
python -m unittest discover -s tests
```

The tests cover the numeric helpers and the engine identity; model quality is measured separately, never asserted in unit tests.
