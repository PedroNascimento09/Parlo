# Model licences

Weights, voices and translation packages carry their own licences. None of them follows the licence of the code that loads it, and none of them is shipped in a release archive: the user downloads them from the original source.

The single source of truth is [`catalog/models.json`](../catalog/models.json). This page explains it and records what is still unverified.

## What a catalogue entry says

| Field | Meaning |
|---|---|
| `id` | Stable name used by the code and the notices |
| `kind` | `asr`, `tts`, `mt` or `voice` |
| `version` | The artefact version, or `unversioned` |
| `license` | The licence of the artefact, or `unverified` |
| `status` | `verified` once the licence and the source were read, `unverified` until then |
| `source` | The official distribution URL, never a mirror |
| `attribution` | The sentence the licence requires, empty when none is required |

`python tools/licenses.py check` refuses an entry marked `verified` that lacks a licence or an attribution, and `--strict-models` fails while any entry remains `unverified`.

## The artefacts

| Artefact | Kind | Licence | State | Attribution |
|---|---|---|---|---|
| Parakeet TDT 0.6B v3 | asr | CC-BY-4.0 | Verified | Required: NVIDIA Parakeet TDT 0.6B v3, CC BY 4.0 |
| Whisper large-v3 | asr | MIT | Verified | Not required |
| Kokoro-82M | tts | Apache-2.0 | Verified | Notices only |
| Piper voices (`pt_PT-tugão`, `pt_BR-faber`, `en_US-lessac`) | voice | MIT, as declared by the `rhasspy/piper-voices` repository | Verified | Voice from `rhasspy/piper-voices`, repository declared MIT |
| OPUS-MT `translate-pt_en-1_9` and `translate-en_pt-1_9` | mt | CC-BY-4.0, as the package declares | Verified | Required: Jörg Tiedemann and Santhosh Thottingal, OPUS-MT, CC BY 4.0 |
| Qwen3-ASR 0.6B | asr | Apache-2.0 | Verified, experimental in Parlo | Notices only |
| Hibiki-Zero | tts | CC BY-NC-SA 4.0 | Verified, non commercial | Required, and never the default |

## Rules the code follows

1. No weights, voices or language packages are inside a release archive.
2. Every download comes from the official source and is checked before it is used: against the digest the source publishes when there is one, and otherwise against a structure check plus a SHA-256 recorded beside the installed copy, with the index it came from written into `parlo.json`.
3. An artefact whose licence is not confirmed is not installed silently: the setup script needs an explicit opt in and prints what is unknown.
4. A model that forbids commercial use is never the default engine and is described as such wherever it is offered.

## What is still open

- Packages beyond the two this project installs. The index carries no licence field, so each package's own README is read before it is added to the catalogue; the two here declare CC BY 4.0 and are recorded with their authors.
- The per-voice cards of `rhasspy/piper-voices`. The repository declares MIT and the three voices used here are recorded against that declaration; if one of them ever becomes a shipped default, its own card is read first.
- The training data licences for Parakeet and Whisper are documented by their publishers, not audited here. Redistributing the weights is allowed by the weight licence; the data is a separate question only if the weights are retrained or redistributed as a dataset.

This page is not legal advice.
