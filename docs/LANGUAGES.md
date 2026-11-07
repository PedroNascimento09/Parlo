# Languages

## Today

Portuguese to English and English to Portuguese run end to end through the cascade: Parakeet hears, an OPUS-MT package translates, and Piper reads the result. Every other pair is listed and blocked, as the table below says.

| Source | Target | State |
|---|---|---|
| Portuguese | English | Ships, measured |
| English | Portuguese | Ships, measured |
| Portuguese | German | Measured directly, no published weights, cannot be installed |
| Portuguese | Spanish, French, Italian, Dutch, Russian | Registered, no download manifest, cannot be installed |
| English | Spanish, French, Italian | Registered, no download manifest, cannot be installed |
| German, French, Spanish | English | Registered, no download manifest, cannot be installed |

The registry offers thirteen translation pairs and `catalog/manifests/` carries a download for two. A pair appears in the app only when its weights, its licence and its measured quality are all present.

## Why a cascade

The default engine is the universal cascade, not a single end-to-end model, so direction is a policy choice rather than a model limit. Cascades cost roughly one to two seconds of extra latency and use a voice that is close but not identical. They also run on any modern CPU, which is the point.

## Into English only, on a GPU

Hibiki-Zero translates **into English only**, needs an NVIDIA GPU with 8 to 12 GB of VRAM, and its weights are CC BY-NC-SA 4.0, which is non-commercial. It is never the default and never the only way to use Parlo.

## Adding a language

1. Confirm the engine supports the direction, not just the language.
2. Add the pair to the engine and to the registry, with a download manifest.
3. Check the licence and record it in `catalog/models.json`.
4. Measure quality on a fixed sample set and publish it in [BENCHMARKS.md](BENCHMARKS.md).
5. Only then call the pair supported.

A language is supported when a test proves it, never because a model card mentions it.

## Interface languages

`crates/parlo-cli/web/locales.json` carries the interface strings and a test fails when a locale leaves a key out or empty. Interface language and audio language are independent.
