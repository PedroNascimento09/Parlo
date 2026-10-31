# Privacy

Parlo is built so that a privacy claim is something you can check, not something you have to trust.

## What local mode guarantees

- Audio is captured, processed, and written locally. It is never uploaded.
- There is no telemetry, no analytics, no crash reporting, and no account system.
- The app does not phone home to check for updates unless you enable update checks.
- No model weights are bundled or downloaded silently.

## How to verify

The audio core has zero dependencies:

```bash
cargo tree -p parlo-core
```

The only entry is `parlo-core` itself. There is no HTTP client, no analytics crate, and no asynchronous runtime.

Search the source for network usage:

```bash
grep -rn "TcpStream\|reqwest\|http" crates/ engines/
```

The Python sidecar imports only the standard library unless you install the optional model extra, and even then it is used for inference on local audio, not for uploads.

Run Parlo with the network disabled and it keeps working.

## The one exception, and it is your choice

A bring-your-own-key cloud engine is on the roadmap for users who want languages no local model supports. It is off by default, requires your own key, and the UI states clearly that audio leaves the machine while it is enabled. If you never enable it, nothing leaves.

## System audio capture

Two-way translation needs to hear the other participants, which means capturing system audio. When that mode is active:

- The app shows an indicator for the whole time capture is running.
- No recording is written to disk unless you ask for a transcript.
- Nothing is retained after the session ends.

## Voice and consent

Parlo translates the voice of the person speaking live. It does not import, clone, or imitate a voice from a recording, and it will not accept a voice sample as input. Misuse, impersonation, and fraud are violations of the [acceptable use policy](../ACCEPTABLE_USE.md).

Tell the people you are speaking to that your voice is being translated. The app can play a short notice at the start of a call if you want that automated.

## Reporting a privacy issue

Open a security advisory as described in [SECURITY.md](../SECURITY.md). Do not open a public issue for a vulnerability.
