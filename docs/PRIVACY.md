# Privacy

Parlo is built so that a privacy claim is something you can check, not something you have to trust.

## What local mode guarantees

- In local mode, audio is captured and processed on this computer. Dictation history, meeting transcripts and voice samples can be saved in the Parlo data folder.
- There is no telemetry, no analytics, no remote crash reporting, and no account system.
- The app checks the GitHub releases API when it starts. Model downloads and updates also use the network when requested. The update check carries no audio or application-generated identifier.
- Model weights are not bundled and are downloaded only when requested.

## How to verify

Inspect the audio core's dependencies:

```bash
cargo tree -p parlo-core
```

The core uses serialization libraries and does not include an HTTP client or analytics crate.

Search the source for network usage:

```bash
grep -rn "TcpStream\|reqwest\|http" crates/ engines/
```

With the required models already installed, run Parlo with the network disabled to check the local inference path.

## Optional remote services

If you explicitly enable a cloud provider for text cleanup, the configured provider can receive dictated text. A remote Parlo server can receive audio when you configure and use it. Network sharing is also optional and uses unencrypted HTTP with a PIN; anyone who can observe that network can read the traffic. See [Intelligence](INTELLIGENCE.md) and [Threat model](THREAT-MODEL.md).

## System audio capture

Parlo can capture system audio for supported workflows. Meeting transcripts are saved locally, and recording or export commands can write audio files to disk. Check the destination before starting a recording.

## Voice and consent

A voice profile can keep a recorded sample and a consent record on this computer. Samples can be deleted with the profile. This build does not clone a voice from a sample. Misuse, impersonation and fraud are violations of the [acceptable use policy](../ACCEPTABLE_USE.md).

Tell the people you are speaking to when you record or translate their speech.

## Reporting a privacy issue

Open a security advisory as described in [SECURITY.md](../SECURITY.md). Do not open a public issue for a vulnerability.
