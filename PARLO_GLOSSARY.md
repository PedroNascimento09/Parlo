# PARLO glossary

The official names of the parts of Parlo. A name here means a component with one responsibility, and every name in this file is used the same way in code, documentation, diagrams and the roadmap. The interface does not need these names to work: a person who never reads this file can use Parlo completely.

Naming rules:

- **PARLO** is the root. Every engine carries it, so a log line, a benchmark or a bug report says where it came from without a second lookup.
- A sigla is two to four letters, pronounceable as a word, and never reused for another responsibility.
- An engine exists to own one responsibility; a mode of an engine does not get a sigla of its own.
- No engine is named before there is a plan to implement it, and no code path is renamed for the sake of the name: the mapping below is the contract, and a rename needs a migration with its tests.

## Engines

| Sigla | Name | Responsibility | Depends on | State |
|---|---|---|---|---|
| **PAVE** | Parlo Audio & Voice Engine | Opens the device, converts and resamples the stream, detects speech, routes playback and speaks. Owns the microphone, the speakers and synthesis through the Piper adapter. | device APIs, Piper | **Implemented** (Windows validated) |
| **PSTE** | Parlo Speech Transcription Engine | Turns audio into text: Parakeet TDT and Whisper, the segment window, and the stability policy that decides when a phrase is final. | PAVE | **Implemented** |
| **PTE** | Parlo Translation Engine | Translates text between languages, including the English pivot for a pair with no direct package, and owns the gate that measures it. | PSTE output, Argos packages | **Implemented**; PT→DE **not validated** by a native speaker |
| **PIE** | Parlo Intelligence Engine | Turns a transcript into the text the person meant: self-corrections, false starts, hesitations, repetitions, punctuation, and the validation that refuses an edit which changes meaning. Also where personal vocabulary will live. | PSTE, PTE (when translating) | **Implemented** core, tested; Local/Cloud routing implemented; settings **not in the interface** |
| **PIMO** | Parlo Intelligence Model Orchestrator | Chooses which model runs, loads and unloads it, and knows what it costs in memory and time. Owns the catalogue, installation and the lifecycle states. | catalogue, PAVE/PSTE/PTE/PIE | **Partially implemented**: catalogue, install and the intelligence route exist; the full lifecycle of Part XVI is planned |
| **PME** | Parlo Memory Engine | Remembers what was said: dictation history, imported meetings, speakers, and the retrieval that answers a question about a past session. | PIE, PSTE | **Partially implemented**: history and meetings exist; cross-session retrieval is planned |

## Surfaces

Surfaces are how a person meets the engines, and they keep plain names rather than siglas: **Parlo Island** (the dictation pill), **Parlo Window** (the page in its own window), **Parlo Hub** (models, library and history), **Parlo Tray** (the one presence with the menu).

## How they fit

```
PARLO
├── PAVE   audio in, audio out, speech detection, synthesis
├── PSTE   audio → text
├── PTE    text → text, in another language
├── PIE    text → the text the person meant
│   └── Personal  vocabulary, abbreviations, style, context (planned)
├── PIMO   which model runs, when it loads, when it leaves
└── PME    what is remembered, and what can be asked of it
```

A dictation walks PAVE → PSTE → PIE → the field being written in. A live session walks PAVE → PSTE → PTE → PAVE. A meeting will walk PAVE → PSTE → PME, and a question about it walks PME → PIE. No engine calls the same work twice: the segment window belongs to PSTE, the correction rules to PIE, the unload policy to PIMO.

## Code mapping

| Engine | Where it lives today |
|---|---|
| PAVE | `crates/parlo-core/src/{audio,vad,resample,ring,voice,wav,gain}.rs`, `engines/cascade/parlo_cascade/tts.py` |
| PSTE | `engines/cascade/parlo_cascade/{engine,streaming}.py` |
| PTE | `engines/cascade/parlo_cascade/mt.py`, `engines/cascade/scripts/mt_gate.py` |
| PIE | `engines/cascade/parlo_cascade/{intelligence,corrections,credentials}.py` |
| PIMO | `crates/parlo-cli/src/catalog.rs`, `catalog/models.json`, `engines/cascade/parlo_cascade/{models,packs}.py`, `intelligence.choose` |
| PME | `history.jsonl`, `meetings/` under the data folder, `crates/parlo-cli/src/serve.rs` |

## States

**Implemented** means the code exists and its tests pass. **Tested** means a test exercises it. **Validated** means a person used it in the situation it was built for. **Planned** means specified and not written. A component is never described by a state it has not reached, and the glossary is updated in the same change that moves one.
