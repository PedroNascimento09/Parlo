# Evaluation corpus

Original test text written for Parlo, released under CC0-1.0. It measures translation between European Portuguese and English in both directions, with material that automated metrics alone would miss: numbers, times, dates, names, negation, clitics, reported speech and idioms. It also carries a first block of Portuguese to German business e-mail sentences, which the plan makes the central gate of v0.5.

## What it contains

`translation.jsonl`, one object per line:

| Field | Meaning |
|---|---|
| `id` | Stable identifier, `source-target-NNN`: `pt-en-`, `en-pt-` or `pt-de-` |
| `source` | Source language: `pt-PT` or `en` |
| `target` | Target language: `en` or `pt-PT` |
| `text` | Source sentence as written |
| `reference` | Human reference translation, not a model output |
| `tags` | What the sentence stresses: `negation`, `numbers`, `date`, `idiom`, `clitic`, and so on |

Portuguese European forms are deliberate: *telemóvel*, *casa de banho*, *comboio*, *correio eletrónico*, *está avariado*, the `a fazer` construction and the `15h30` time format. A model that only handles Brazilian Portuguese will visibly fail here, which is the point.

## Portuguese to German

`pt-de-001` to `pt-de-020` are business e-mail sentences: appointments, deadlines, invoices, quantities, refusals and one complaint. They exist because PT→DE is the central quality gate of v0.5 and a gate needs a corpus before it needs a model.

**State of the pair, stated plainly:** the German references were written by the project and have not been read by a native speaker, and no PT→DE model is installed, because the package index Parlo installs from carries no `pt` to `de` entry. The gate therefore checks the corpus, not a translation: `mt_gate.py --direction pt-de --pack` stops with `no package installed for pt-de`. The pair is not validated and must not be described as working.

## What it does not measure

This corpus measures translation. It says nothing about recognition quality: an ASR evaluation needs authorised recordings and a separate reference transcript, and the two must never be reported as if they were the same result.

## Rules

- No private messages, personal data or third-party text. Every line is written for the project.
- A new line needs an id, a reference and at least one tag.
- References are human-written; a model output is never accepted as its own reference.
- Language codes distinguish `pt-PT` from `pt-BR`. Lumping them together hides the failure the corpus exists to catch.
- The corpus is CC0-1.0 so it can be copied into other evaluation harnesses without negotiation.
