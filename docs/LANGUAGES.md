# Languages

Parlo will be honest about this even when it costs adoption.

## Today

No speech model is integrated, so no pair works end to end. The only engine that runs is `echo`, which returns the audio it receives.

## v0.3: Portuguese and English, both directions

The default engine is the universal cascade, not a single end-to-end model, so direction is a policy choice rather than a model limit.

| Source | Target | Engine | State |
|---|---|---|---|
| Portuguese | English | Cascade | Planned for v0.3 |
| English | Portuguese | Cascade | Planned for v0.3 |

Cascades cost roughly one to two seconds of extra latency and a voice that is close but not identical. They also run on any modern CPU, which is the whole point.

## Premium add-on: into English only

Hibiki-Zero translates speech **into English only**, needs an NVIDIA GPU with 8 to 12 GB of VRAM, and its weights are CC BY-NC-SA 4.0 (non-commercial).

| Source | Target | State |
|---|---|---|
| Portuguese, Spanish, French, German, Italian | English | Optional, after v0.3 |

It is the highest voice fidelity available today and the wrong default: it excludes most machines and most professional use. The app therefore offers it as a download with a visible license warning, and never as the only way to use Parlo.

## v1.3 and later: more languages

| Milestone | Scope |
|---|---|
| v1.3 Anything | 20+ input languages, 10+ output voices, model chosen per language |
| v1.5 Two-way | Both directions live, in-person mode |
| v2.x Languages | Community language programme with native speaker review |

A language is supported when a test proves it, never because a model card mentions it. Each new language needs a native speaker review and a published benchmark before it is called supported.

## Interface languages

The app interface is a few hundred strings. v0.8 covers English, Portuguese, Spanish, French, German, Hindi, Japanese, and Chinese. Interface language and audio language are independent.

## Add a language

1. Confirm the engine supports the direction, not just the language.
2. Add the pair to the engine's `target_languages` and to this table.
3. Measure quality on a fixed sample set and publish it in [BENCHMARKS.md](BENCHMARKS.md).
4. Have a native speaker review it.
5. Only then call it supported.
