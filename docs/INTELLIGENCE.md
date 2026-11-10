# The intelligence engine

PIE, the Parlo Intelligence Engine, is the layer between a transcript and the text that is written. It applies self-corrections, removes false starts, hesitations and accidental repetitions, and fixes punctuation. Model edits are instructed to preserve what the person said and are checked before use, but these checks cannot detect every change in meaning.

It runs after recognition and before insertion:

```
PAVE → PSTE → PIE → the field being written in
```

## Modes

| Mode | What runs |
|---|---|
| `off` | The deterministic rules only: corrections, false starts, hesitations, repetitions. No model, no network. |
| `local` | The deterministic rules, then a local model if one is configured at a loopback address. |
| `cloud` | The deterministic rules, then the configured provider, if cloud processing is authorised and a key is stored. |
| `auto` | The same as the above in order: a configured local model, then an authorised provider with a key, then the rules alone. This is the default. |

A mode never overrides the user's permission: `auto` will not reach a provider unless cloud processing was allowed and the provider was configured. From a terminal:

```bash
PARLO_INTELLIGENCE_MODE=auto
PARLO_INTELLIGENCE_CLOUD=1
PARLO_INTELLIGENCE_PROVIDER=openai
PARLO_CASCADE_CLEAN=light
```

The interface has the same choices in **How dictation is cleaned up**, and writes them to the config file, where the engine reads them on the next dictation; nothing has to be reloaded. From a terminal the same file is read and the variables above override it, so a person testing a provider can do it without touching the app.

## Deterministic first

The rules in `corrections.py` run in every mode, including `off`, because they cost microseconds and need nothing installed. A model is only asked to work on text the rules already cleaned, which keeps the prompt small and the answer easier to check.

A local model is never loaded just to punctuate a sentence: it is only used when one is configured, and Parlo ships no weights for it. The supported shape is an OpenAI-compatible server on loopback (for example a `llama.cpp` server), so a person who has one can point Parlo at it without a new runtime in the product.

## Providers

| Provider | Endpoint | Models offered | Key variable |
|---|---|---|---|
| OpenAI | `https://api.openai.com/v1/chat/completions` | `gpt-4o-mini`, `gpt-4o` | `PARLO_OPENAI_KEY` |
| Anthropic | `https://api.anthropic.com/v1/messages` | `claude-3-5-haiku-latest`, `claude-3-5-sonnet-latest` | `PARLO_ANTHROPIC_KEY` |
| Google Gemini | `https://generativelanguage.googleapis.com/v1beta/models` | `gemini-2.0-flash`, `gemini-1.5-flash` | `PARLO_GEMINI_KEY` |
| OpenAI compatible | base URL you configure | whatever the server serves | `PARLO_COMPATIBLE_KEY` |

The key belongs to the user. Parlo has no central API and never pays for inference, so nothing in the product stops working when a key is absent: the deterministic engine stays in place.

## Keys

A key is stored in the operating system's credential store, never in `config.json`:

- **Windows:** DPAPI through `CryptProtectData`, in `credentials/<provider>.bin` under the Parlo data folder. The blob is bound to the Windows account that wrote it, so another account reading the file gets nothing.
- **macOS:** the login keychain, through `security add-generic-password`.
- **Linux:** the secret service, through `secret-tool`.

When none of these exists, Parlo refuses to store the key and says so instead of writing it in clear. An environment variable is read first and never written anywhere, so a machine without a credential store can still work for a session. Keys are never shown again after being stored, never appear in logs or error messages, and every request goes over HTTPS or to loopback — a plain remote connection is refused before anything is sent.

Requests go from the machine directly to the provider. There is no Parlo proxy and no server in the middle.

## Personal vocabulary

**Words Parlo should know** keeps two lists in `vocabulary.json` beside the configuration: names and terms the recogniser keeps getting wrong, and short forms that should expand. The engine applies them to the cleaned sentence with word boundaries only, before any model is called, and records each substitution in the changes. Nothing about the list leaves the machine, no account is involved, and an empty or unreadable file means the feature does nothing rather than breaking a dictation. A pattern such as `a.b` is matched literally, never as an expression.

## What protects the text

A model is an editor, not an author, and the answer is checked before it can be used:

- The answer must be JSON with a `text` field. Free-form prose, a code fence or commentary is rejected.
- Digit sequences must be preserved with the same occurrences: an added, removed or repeated number is rejected.
- It cannot introduce a capitalised name that was not there.
- It cannot grow past 1.6 times the original length or shrink below 0.4 of it.
- On any failure — an invalid key, a rate limit, a timeout, a rejected answer — the deterministic text is used and the reason is recorded. Nothing is lost and nothing is retried behind the user's back.

These checks do not prove that names, negations or other facts were preserved. Review the result when accuracy matters. The original transcript is kept beside the cleaned text.

## Cost and latency

One request per dictation, on the finished text, never per audio window and never on partial transcripts. A dictation that the rules can handle is never sent anywhere. There is no retry loop, no background polling and no cache of other people's text.

A model has 1.5 s to answer, set by `PARLO_INTELLIGENCE_DEADLINE_MS` and clamped between 200 ms and 5 s. Past that the deterministic text is used and the reason is recorded, so the worst case a person feels is a correction that did not happen, never a dictation waiting on a provider.

An estimate of what a request costs requires the provider's current prices, which Parlo does not bundle; the honest answer today is that Parlo cannot show a price it has not measured, so it does not invent one. Spending limits and per-model price display are planned, not implemented.

## What is not built yet

- **Cost control**: usage estimates, spending caps and a cheaper-model preference. The screen shows no price, because Parlo has measured none.
- **A validated provider call**: the screen can save a key and send a test sentence, but no real provider has been exercised from this machine yet, so the panel is tested, not validated.
- **Style preferences and context profiles**: the vocabulary list exists, the rest of Personal Intelligence does not.
- **A comparison with Wispr Flow.** None has been run, and no claim of equivalence is made.

## Tests

`engines/cascade/tests/test_intelligence.py` covers routing (off, local, cloud, auto, an unauthorised cloud, a missing key), the transport rule, the validation rules, fallback on a refused key, fallback on an invented fact, the deadline a slow model is given, and the loopback-only local model. `test_admin.py` covers the settings the engine reads from the config file, the environment winning over it, a hand-edited value falling back instead of breaking dictation, and the refusals of the key entry. `test_credentials.py` covers the OS store round trip and checks that the Windows file does not carry the secret in clear. The transport is injected, so the tests never touch the network. On the Rust side, `intelligence.rs` tests the settings that reach the engine: modes, provider names, model names, and which plain `http://` addresses are refused.

That is **tested**, not **validated**: no real provider call was made with a real key from this machine, and no person has compared `original`, `light` and `intelligent` on their own dictation.

## Where the key never goes

The screen accepts a key, hands it to the engine once, and clears the field; the server never writes it to the config file, never logs it and never sends it back. Removing a key deletes it from the operating system's store, and the next dictation falls back to the rules with a reason.
