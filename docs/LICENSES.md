# Licences

Two artefacts ship from this repository and they do not carry the same licence. This page says which, and why.

## The short version

`parlo`, the Rust command line program, is **MIT** and comes from [`LICENSE`](../LICENSE).

`parlo-engine`, the packaged Python engine that sits next to it in a release archive, is **GPL-3.0-or-later**, because it contains Piper and the espeak-ng data Piper loads.

## Why the engine is GPL

Piper is the speech synthesiser of the default cascade and it is GPL-3.0-or-later. Until the packaging changed, Parlo invoked a Piper that the user installed separately, which kept the two programs apart. `parlo-engine` ends that separation: [`parlo-engine.spec`](../engines/cascade/parlo-engine.spec) builds one program that contains the `piper` package, the `piper_phonemize` bindings and the espeak-ng data they load at runtime. A program that contains GPL code and links it in the same process is a combined work, so the whole bundle is distributed under the GPL.

The GPL asks for the complete corresponding source of what was shipped. It is this repository: `engines/cascade` plus the spec file that builds it.

## What each piece is

| Piece | Licence | Where it lives |
|---|---|---|
| `parlo` command line program | MIT | [`LICENSE`](../LICENSE), `crates/` |
| `parlo-engine` bundle | GPL-3.0-or-later | `engines/cascade`, built by `parlo-engine.spec` |
| sherpa-onnx runtime | Apache-2.0 | bundled in `parlo-engine` |
| Parakeet TDT 0.6B v3 weights | CC-BY-4.0 | downloaded, not shipped |
| Argos Translate | MIT | bundled in `parlo-engine` |
| faster-whisper | MIT | bundled in `parlo-engine` when installed |
| CTranslate2 | MIT | bundled in `parlo-engine` |
| Piper | GPL-3.0-or-later | bundled in `parlo-engine` |
| espeak-ng and its data | GPL-3.0-or-later | bundled in `parlo-engine` |
| Piper voices for Portuguese and English | CC BY 4.0, as published by the voice authors | downloaded, not shipped |
| Hibiki-Zero weights | CC BY-NC-SA 4.0 | optional, downloaded, never the default |

`parlo engines` prints the same policy from the registry the program is built with, so the table cannot drift away from the code.

## What that means for you

Using, modifying or redistributing `parlo-engine` carries the GPL obligations, including making the corresponding source available; using the MIT `parlo` binary on its own does not. Running either of them locally, for your own calls and recordings, obliges you to nothing.

## Why this is not where it should end

A permissive synthesiser would let the whole distribution stay MIT and remove the split above. That is the reason the project intends to replace Piper with one, and it is a decision about measured quality, not a date.
