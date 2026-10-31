# Threat model

An initial model, written against the code as it is, not as it might be. It names what is protected, who could reach it, what the code already enforces, and what it does not.

## What this protects

- **The microphone and the machine's audio.** Captured on the machine, processed there, written there, and never uploaded.
- **The local API.** Anything that can reach `127.0.0.1` on the port the server chose.
- **The models and the binary.** Both are replaced from files downloaded over the network, so a wrong archive is code execution on the next start.
- **The session token.** It is the only thing standing between a local request and the translation and update endpoints.

## Who could attack, and how

- **A web page in the user's browser.** It can address `127.0.0.1` on any port, so it can send requests to the local server whether or not the user meant it to.
- **Another process running as the same user.** It can read the process command line, the terminal output and files such as `parlo.previous`.
- **A compromised release.** `parlo update` downloads an archive and replaces the running binary with it, so whoever controls the release feed controls the next start.
- **A hostile model download.** The recogniser and voice files come from a remote host and are loaded by a Python process with the user's rights.
- **A corrupted audio file.** `parlo translate` reads a file the user picked, and the API reads a body the caller sent.

## What is in place

- The listener is bound to `127.0.0.1` only (`serve::run`), so nothing on the network can reach it directly.
- `POST /translate`, `POST /update` and `POST /models` require the token, sent as `Authorization: Bearer` or `x-parlo-token`, compared with a constant-time comparison that does not return early on the first different byte (`serve::authorised`, `serve::constant_time_eq`).
- The token is 24 hex characters derived from the clock and the process id (`serve::new_token`), printed to the terminal, passed into the page URL by `parlo app`, and sent back by the page as a header (`app.js`).
- Requests are bounded: 16 KiB of headers, 20 MiB of body, 10 second read timeout and 30 second write timeout per connection (`serve::MAX_HEAD`, `serve::MAX_BODY`).
- Audio is validated before use: RIFF/WAVE header, PCM16 only, chunk sizes checked against the buffer, even data length (`parlo_core::wav::read_pcm16`), and the API additionally requires 16 kHz mono.
- JSON bodies are escaped before they are written (`serve::json_string`), so a Windows path in an error message cannot break the response.
- The page talks to two places only: the local server, and `https://api.github.com/repos/PedroNascimento09/Parlo/releases/latest` for the update check. It loads no font, script or image from anywhere else.
- There is no telemetry, no analytics and no HTTP client in the Rust core. Audio is not part of any request except the loopback one that translates it.
- Model downloads are verified. `setup_models.py` asks the Hugging Face API for the sha256 it publishes for every file, streams the download into a `.part` file while hashing it, compares the size as well, and only renames it into place when both match. A mismatch deletes the part file and stops the install. When the metadata call itself fails, it warns once and falls back to the size check, so an offline-ish install still works.

## What is not in place yet

- **No `Origin` or `Host` validation.** A page the user visits can still fire a request at the loopback server; the token is what stops it from succeeding, not the request's origin. DNS rebinding is possible in the same way, and neither is mitigated in code today.
- **No checksum or signature on release archives.** The model files are checked against the digest the Hugging Face API publishes, but the release archive is not: `softprops`-built releases carry no signatures and `update::install` extracts and swaps whatever arrived, so a compromised release feed is still code execution on the next start.
- **The model integrity check trusts the transport and the API.** The sha256 comes from the same host over HTTPS, so an attacker who can lie about the file can usually lie about the digest too. It catches a corrupted or truncated download, a mirror serving the wrong bytes, and a file replaced on disk; it does not catch a compromised Hugging Face account. A digest pinned in the repository, published with each release, would close that gap.
- **Page and API share one origin.** The interface is served by the same server as the API, so there is no browser-enforced separation between them.
- **No rate limiting.** The token comparison is cheap, but nothing slows down repeated attempts to guess one, and nothing limits repeated translation requests.
- **The token lives in a URL.** It appears in the browser address bar, in history, and in anything that logs URLs. It is not rotated, and every start of `parlo app` produces a new one only because the process restarts.
- **The sidecar trusts the Python on disk.** `SidecarEngine::spawn` runs `serve.py` with the interpreter it finds (`PARLO_PYTHON`, then `.venv`, then `python`), so a modified script or a shadowed interpreter runs with the user's rights.
- **`parlo.previous` is written beside the binary** without checking the directory's permissions, so on a shared or oddly permissioned install it could be readable or writable by someone else.

## What you can do

- Keep the token to yourself: do not paste the app URL into a chat, a screenshot or an issue.
- Close the terminal, or stop the process, when you are done. The server does not outlive your session by design.
- Refuse an update you did not expect, and check the version and notes in the banner before pressing **Install update**.
- Point `PARLO_PYTHON` at an interpreter you trust if the machine has several.
- Treat the repository and the release page as the source of truth: anything claiming to be a Parlo build from somewhere else is not one.

## Reporting a problem

Follow [SECURITY.md](../SECURITY.md): private GitHub security advisories, never a public issue, with what the issue allows, the version or commit, the smallest reproduction, and your operating system. Acknowledgement within 72 hours, assessment within 7 days.
