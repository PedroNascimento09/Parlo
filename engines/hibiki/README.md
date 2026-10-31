# parlo-hibiki

Reference adapter that connects [Hibiki-Zero](https://github.com/kyutai-labs/hibiki) to Parlo.

It speaks newline delimited JSON over stdio, so the desktop app can spawn it as a sidecar and restart it without restarting Parlo.

## Run

```bash
cd engines/hibiki
python serve.py
```

Install the model extra before running real inference:

```bash
pip install -e ".[model]"
```

## Protocol

Request:

```json
{"op": "translate", "seq": 1, "pcm16": "<base64 little endian pcm>"}
```

Response:

```json
{"op": "translated", "seq": 1, "pcm16": "<base64>", "text": "hello", "latency_ms": 1400}
```

Errors are always reported, never swallowed:

```json
{"op": "error", "seq": 1, "error": "torch is not installed"}
```

Send `{"op": "hello"}` first to read the engine id, target languages, and whether the model is available.

## License warning

Hibiki-Zero weights are **CC BY-NC-SA 4.0**: non-commercial, share alike. They are downloaded by the user, never bundled, and never part of the default engine. Commercial use of this engine is not permitted by its license. See [../../docs/ENGINES.md](../../docs/ENGINES.md).

## Status

Audio is passed through the protocol and validated, but the Hibiki weights are not wired to the inference loop yet. Until then the adapter answers `hello` and refuses translation with a clear error. See [../../docs/ENGINES.md](../../docs/ENGINES.md).

## Tests

```bash
cd engines/hibiki
python -m unittest discover -s tests
```
