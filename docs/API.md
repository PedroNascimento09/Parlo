# API

Parlo serves a local HTTP API on `127.0.0.1` with a token and answers nowhere else. Every example below is the request `examples/openai_client.py` sends, and the response shown is the shape the server returns.

```bash
python examples/openai_client.py --port 8787 --token <the token parlo serve prints>
```

The client prints one line per endpoint: `models`, `transcribe`, `translate`, `speech` and `refused`.

## GET /v1/models

```bash
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8787/v1/models
```

```json
{"object":"list","data":[{"id":"parakeet","object":"model","owned_by":"parlo","kind":"stt","feature":"Dictation","installed":true,"licence":"CC-BY-4.0","languages":["pt","en"],"default":true}]}
```

Every card carries its licence and its languages, and a model that is not on disk is listed with `"installed": false`, so a client can ask before downloading. No token means `401` with `{"error":"wrong token"}`.

## POST /v1/audio/transcriptions

Multipart with `model` and `file`; the audio is decoded locally and recognised by the dictation pipeline.

```bash
curl -s -H "Authorization: Bearer $TOKEN" -F model=parakeet -F file=@sample.wav http://127.0.0.1:8787/v1/audio/transcriptions
```

```json
{"text":"Bom dia, tudo bem?"}
```

`response_format` accepts `json`, `text`, `verbose_json`, `srt` and `vtt`. A model outside `whisper-1`, `whisper` and `parakeet` is refused with `400` and an `invalid_request_error` naming `model`, which is the `refused: 400` line the client prints.

## POST /v1/audio/translations

The same multipart shape; the answer is the translation of what was said.

```bash
curl -s -H "Authorization: Bearer $TOKEN" -F model=parakeet -F file=@sample.wav http://127.0.0.1:8787/v1/audio/translations
```

```json
{"text":"Good morning, how are you?"}
```

## POST /v1/audio/speech

```bash
curl -s -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"input":"Bom dia","response_format":"wav"}' http://127.0.0.1:8787/v1/audio/speech -o speech.wav
```

The reply is `audio/wav` and starts with `RIFF`; `response_format` also takes `pcm` and `mp3`. Empty input, more than 4096 characters, a speed other than 1 and an unknown format are each refused with `400`.

## POST /v1/audio/transcriptions/stream

A WebSocket upgrade. Send 16 kHz mono PCM16 frames and read `partial`, `stable` and `final` events, each with the session id; close the socket to finish.

```python
import json
import websocket

socket = websocket.create_connection(
    f"ws://127.0.0.1:{port}/v1/audio/transcriptions/stream",
    header=[f"Authorization: Bearer {token}"],
)
socket.send_binary(frame)
print(json.loads(socket.recv()))
```

## POST /v1/jobs and GET /v1/jobs/{id}

Dubbing jobs queued through the API. `POST /v1/jobs` with the project takes the work, `GET /v1/jobs/` lists it and `GET /v1/jobs/{id}` reports one.

```bash
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8787/v1/jobs/
```

```json
{"object":"list","data":[{"id":"j-1","kind":"dub","state":"running","stage":"translate","progress":0.4}]}
```

`POST /v1/jobs/{id}/cancel` stops one that has not finished. The API is bound to `127.0.0.1`, so every one of these calls stays on this computer.
