# parlo-cascade

The default Parlo engine: streaming speech recognition, translation, and speech synthesis, all on CPU, all under permissive licences.

| Stage | Component | Licence |
|---|---|---|
| Voice detection | Parlo pipeline gate (RMS with hangover) | MIT |
| Speech recognition | Parakeet TDT 0.6B v3 int8 via sherpa-onnx | CC-BY-4.0 model, Apache-2.0 runtime |
| Translation | Argos Translate (OpenNMT/CTranslate2) | MIT |
| Speech synthesis | Piper (`pt_BR-faber-medium`, `en_US-lessac-medium`) | **GPL-3.0-or-later** |

Piper is not MIT. The PyPI package `piper-tts` is GPL-3.0-or-later, and each voice model carries its own terms. Parlo's own code stays MIT, and Piper is treated as a separate program: the user installs it, runs it as a sidecar, and the licence is shown in the README and by `parlo engines`. It is never bundled in an installer.

If Parlo later ships a one-click installer, the synthesiser has to change or the whole product becomes GPL. Kokoro (Apache-2.0) is the first candidate to evaluate; that decision belongs to v0.2, not to a prototype.

No model here is non-commercial, which is why this is the default and Hibiki-Zero is not.

## Install

```bash
python -m venv .venv
.venv/Scripts/python -m pip install -r engines/cascade/requirements.txt
.venv/Scripts/python engines/cascade/scripts/setup_models.py
```

On Linux and macOS replace `.venv/Scripts/python` with `.venv/bin/python`.

The setup script downloads the Piper voices, the Parakeet v3 model, and the Argos Portuguese to English packages. Add `--stt whisper` to cache faster-whisper instead, or `--stt both`. Everything lands in `engines/cascade/models` and the user's model cache, both outside Git.

## Run

```bash
parlo translate in.wav out.wav --engine cascade
```

The CLI looks for `.venv` next to the engine, in `engines/`, or at the project root, and falls back to `python` on the path. `PARLO_PYTHON` overrides that search.

| Variable | Default | Purpose |
|---|---|---|
| `PARLO_PYTHON` | detected `.venv`, else `python` | Interpreter with the cascade requirements installed |
| `PARLO_CASCADE_DIR` | `engines/cascade` | Directory containing `serve.py` |

## How it behaves

Audio arrives in 20 ms frames. The engine buffers them until it has at least 2.5 seconds of speech, then transcribes, translates, synthesises, and returns the translated audio in one response. Frames that only fill the buffer return empty audio, which the pipeline treats as "nothing to play yet".

`PARLO_CASCADE_SEGMENT` changes that window. It is the single most important setting for keeping up with a conversation, because the recogniser pays a fixed cost per call:

| Window | Real time factor on a 12 core CPU |
|---|---|
| 1.2 s | 2.66, backlog grows |
| 2.5 s | 0.96, roughly keeps up |
| 4.0 s | 1.15, one slow call |

The recogniser matters more than the window. With 4 threads, which approximates an ordinary laptop, on 4.95 s of audio:

| Recogniser | RTF, 2.5 s window | RTF, whole utterance |
|---|---|---|
| parakeet v3 (default) | **0.113** | 0.092 |
| whisper base | 0.309 | 0.153 |

Parakeet is the default because it is both faster and more accurate. `PARLO_CASCADE_STT=whisper` switches back, and only then does `PARLO_CASCADE_MODEL` (`tiny`, `base`, `small`) matter. Translation costs under 0.05 s and synthesis 0.28 s on a five second block; recognition was the whole bill and no longer is.

This is the "fast consecutive" mode from the plan, not simultaneous speech.

## Warm-up and glossary

Models load at startup, so the first sentence is not the one that waits: startup takes 12 to 21 seconds and prints progress on stderr. `PARLO_CASCADE_WARMUP=0` disables it.

`glossary.txt` maps names the recogniser gets wrong, one `wrong=right` per line, applied to the transcript before translation. The default already fixes `Palo` and `Paul's` into `Parlo`.

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
