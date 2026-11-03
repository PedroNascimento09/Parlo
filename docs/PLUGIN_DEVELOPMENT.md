# Plugin development

The local API is the extension point. No plugin runtime ships today, and this document records the contract a plugin will have to meet rather than a feature that exists: writing it now keeps the API stable while the surfaces are still moving.

## What exists today

`parlo serve` listens on loopback only, requires the token it prints, checks the `Host` and `Origin` headers, and answers these routes:

| Route | What it does |
|---|---|
| `GET /models`, `POST /models/load/<id>`, `POST /models/unload/<id>`, `POST /models/policy` | List models with their state, load or free one, set its idle delay |
| `POST /dictate`, `POST /dictate/begin`, `POST /insert` | Transcribe a clip, choose the field to write into, type the text there |
| `POST /translate`, `POST /listen`, `POST /stream` | Translate audio, read it aloud, publish partial and final text |
| `POST /meeting/ask`, `POST /memory/ask`, `POST /dub` | Ask a stored meeting, ask every meeting, dub a file |
| `GET /settings`, `POST /settings`, `GET /update` | Read and change the pair, the recogniser, the shortcuts, the cleanup and the update check |

Anything outside the list is not a contract and may change between releases. The token is per launch, and a caller that loses it must be restarted, which is deliberate: nothing on the machine reaches the engine without the file it is stored in.

## What a plugin will have to be

- **Out of process.** A plugin talks to the API, never loads the engine's Python or Rust in its own process. A crash in a plugin cannot take dictation down with it.
- **Declared capabilities.** A manifest names the routes it uses, the files it reads and whether it needs the microphone. Nothing is granted implicitly.
- **Loopback only, with the token.** No plugin receives a way to open a port on the network, and no plugin is handed the user's audio unless it asked for the microphone and the user allowed it.
- **No telemetry.** A plugin that sends anything about the user, the audio or the transcripts is not installable. This is the same rule the application follows.
- **Signed manifests in the registry.** A registry entry carries the manifest, a digest of the archive and the licence. Until signing exists, only local, hand-installed plugins are in scope.
- **Versioned.** A plugin states the API version it targets; a breaking change bumps the version and the old one keeps working for one release.

## What is deliberately absent

Accounts, a marketplace, remote execution, cloud relays and any route that uploads audio. Parlo is a local program and the plugin surface does not change that.

## Compatibility promise

The routes above are stable within a minor version. Adding a field is not a breaking change; removing one, changing a status code or changing what a field means is. `docs/ENGINES.md` describes the sidecar protocol behind them, which is versioned separately and is not a plugin surface.
