# Languages

Parlo will be honest about this even when it costs adoption.

## Today

No speech model is wired up yet, so no language pair works end to end. The only engine that runs is `echo`, which returns the audio it receives.

## v1.0: into English

Hibiki-Zero translates speech into **English only**. That is a property of the model, not a configuration mistake.

| Source | Target | State |
|---|---|---|
| Portuguese | English | Planned |
| Spanish | English | Planned |
| French | English | Planned |
| German | English | Planned |
| Italian | English | Planned |
| English | English | Not a use case |

If you need English into Portuguese, or Portuguese into Spanish, that is not v1.0. It is the cascade engine in v1.5, listed below.

## v1.5: any direction, lower fidelity

A cascade (speech recognition, then machine translation, then speech synthesis with voice cloning) removes the direction limit at the cost of latency and voice similarity.

| Source | Target | State |
|---|---|---|
| English | Portuguese, Spanish, French, German, Italian | Planned |
| Any supported pair | Any supported pair | Planned |

Expect roughly one to two extra seconds of latency and a voice that is close but not identical. The two-way mode, where you hear the other side in your language, also belongs to this release.

## Interface languages

The app interface is a few hundred strings. The v1.0 release plan covers English, Portuguese, Spanish, French, German, Hindi, Japanese, and Chinese. This does not change the audio languages: the interface can be in Japanese while the engine still translates into English.

## Adding a language

1. Check that the engine supports the direction, not just the language.
2. Add the pair to the engine's `target_languages` and to this table.
3. Measure quality on a fixed sample set and publish the result in [BENCHMARKS.md](BENCHMARKS.md).
4. Only then call it supported.

The rule: a language is supported when a test proves it, never because a model card mentions it.
