# Licences

The index every user, company and contributor should read first is [LICENSES.md](../LICENSES.md) at the root of the repository: one table per section, code, dependencies, weights and voices, with the licence of each and what it obliges you to do. This page explains the reasoning behind it and the questions it leaves open.

## The short version

| Piece | Licence | Where it is |
|---|---|---|
| Parlo code: `parlo-core`, `parlo-cli`, `parlo-app`, the page, the cascade engine, the scripts, the tests | **Apache-2.0** | [`LICENSE`](../LICENSE), `crates/`, `engines/cascade`, `engines/hibiki` |
| Releases up to v0.1.7 | **MIT**, unchanged and not retroactive | [`licenses/MIT-Parlo-0.1.x.txt`](../licenses/MIT-Parlo-0.1.x.txt) |
| Piper speech synthesiser | **GPL-3.0-or-later** | [engines/tts-piper](../engines/tts-piper), installed and licensed separately, never in a release archive |
| Model weights and voices | Their own licences, one per artefact | [`catalog/models.json`](../catalog/models.json), downloaded by the user, never shipped |

## Why Apache-2.0

Apache-2.0 keeps everything MIT allowed, adds an express patent grant and an explicit statement about contributions, and stays compatible with closed products. The copyright holders of this repository confirmed that every commit in the history belongs to them, so the whole original code was relicensed in one step. Earlier releases keep the MIT text they were published with.

## Why the core is permissive and the voice is not shipped

Piper is GPL-3.0-or-later and carries the espeak-ng data it loads. A release that put Piper in the same program as the engine would distribute one combined work under the GPL.

Since this change the engine program contains no Piper code: `engines/cascade/parlo-engine.spec` no longer bundles `piper`, `piper_phonemize` or `espeak-ng-data`, and `engines/cascade/requirements.txt` no longer lists `piper-tts`. The synthesiser is an optional adapter in [engines/tts-piper](../engines/tts-piper) that the user installs with its own command, and the engine talks to it over a JSON line protocol on stdin and stdout.

**A process boundary is not a legal conclusion.** Running two programs side by side and exchanging data on a pipe is the arrangement the project implements and describes; whether a given distribution is one work or two is a question for a lawyer.

Without the adapter Parlo still translates, still recognises speech, still dubs files that need no voice, and says in a sentence that synthesis is unavailable and why.

## What a release contains

Each archive carries the two programs, the packaged engine, the icons, `README.md`, `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md` and `sbom.cdx.json`. It carries no Piper, no espeak-ng data and no model weights.

The Python closure inside the packaged engine is recorded in [`tools/python-packages.json`](../tools/python-packages.json) and reproduced in [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md). The SBOM is CycloneDX 1.5. Full licence texts live in [`licenses/`](../licenses).

## Models and voices

Weights and voices keep their own licences, which never follow the licence of the code that loads them:

| Artefact | Licence | State |
|---|---|---|
| Parakeet TDT 0.6B v3 | CC-BY-4.0 | Verified, attribution required |
| Whisper large-v3 | MIT | Verified |
| Kokoro-82M | Apache-2.0 | Verified |
| Piper voices | Per voice, not yet confirmed | **Unverified** |
| Argos language packages | Per package, not yet confirmed | **Unverified** |
| Qwen3-ASR | Not confirmed | **Unverified** |
| Hibiki-Zero | CC BY-NC-SA 4.0 | Verified, non commercial, never the default |

Details per artefact and the rules the code follows: [MODEL_LICENSES.md](MODEL_LICENSES.md).

## The gate

```bash
python tools/licenses.py check             # every component, every scope
python tools/licenses.py check --strict    # a release: tracked issues fail too
python tools/licenses.py index             # regenerate LICENSES.md
```

The gate reads the Rust closure from `cargo metadata`, the Python closures from the two requirements files, and the catalogue. It fails when a licence is missing, denied, outside the allow list for that scope, or attached to an unpinned requirement, and CI fails when the committed `LICENSES.md` or `THIRD_PARTY_NOTICES.md` is not the one the tool generates. GPL is allowed for the optional components and refused in the core, because that is where the isolation is real.

`tools/license-policy.json` holds the allow lists, the per component overrides, and the tracked issues with the reason each one is still open.

## Open questions

1. **`minisbd` is AGPL-3.0-only** and reaches the engine through `argostranslate`, so the gate tracks it and no release passes while it is there. The replacement named in the master plan is OPUS-MT on CTranslate2. The attempt on 1 November 2026 did not produce a shippable model and nothing was published:

   - `Helsinki-NLP/opus-mt-pt-en` and `Helsinki-NLP/opus-mt-en-pt` answer 401 and cannot be downloaded.
   - `Helsinki-NLP/opus-mt-ROMANCE-en` works português→inglês with the `>>por<<` source prefix, beam 4, `no_repeat_ngram_size=3` and `repetition_penalty=1.2`: 69 to 130 ms per sentence on four threads, correct output on three of three samples.
   - `Helsinki-NLP/opus-mt-tc-big-en-pt` and `Helsinki-NLP/opus-mt-en-ROMANCE` degenerate into repeated fragments inglês→português with every target prefix tried (`>>por<<`, `>>pt<<`, none).

   The engine still calls Argos, so `G-LIC-MT` stays open and releases stay blocked until a working permissive pair is chosen and measured with the same sentences.
2. **`r-efi` is LGPL-2.1-or-later**, reachable only through the UEFI target of `getrandom`. It is not in any shipped binary today; the position needs a lawyer if a UEFI build is ever published.
3. **The process boundary with Piper** needs a legal view before the repository claims that the two are separate works, not before.

Nothing on this page is legal advice.
