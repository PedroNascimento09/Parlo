#!/usr/bin/env python3
"""Run every OpenAI-compatible endpoint against a local Parlo server."""

import argparse
import io
import json
import math
import struct
import urllib.error
import urllib.request
import wave


def wav_bytes(seconds=1.0, rate=16_000):
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(rate)
        frames = b"".join(
            struct.pack("<h", int(8_000 * math.sin(2 * math.pi * 440 * index / rate)))
            for index in range(int(rate * seconds))
        )
        handle.writeframes(frames)
    return buffer.getvalue()


def multipart(audio, model="parakeet"):
    boundary = "parloexampleboundary"
    head = (
        f"--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\n{model}\r\n"
        f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample.wav\"\r\n"
        "Content-Type: audio/wav\r\n\r\n"
    ).encode()
    tail = f"\r\n--{boundary}--\r\n".encode()
    return head + audio + tail, f"multipart/form-data; boundary={boundary}"


def post(base, token, path, body, content_type):
    try:
        return send(base, token, path, body, content_type)
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", "replace")
        raise SystemExit(f"{path}: {error.code} {detail}")


def send(base, token, path, body, content_type):
    request = urllib.request.Request(
        base + path,
        data=body,
        method="POST",
        headers={"Authorization": f"Bearer {token}", "Content-Type": content_type},
    )
    return urllib.request.urlopen(request, timeout=120)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8787)
    parser.add_argument("--token", default="")
    arguments = parser.parse_args()
    base = f"http://127.0.0.1:{arguments.port}"

    models = json.load(urllib.request.urlopen(base + "/v1/models", timeout=30))
    print("models:", len(models["data"]), "cards")

    audio = wav_bytes()
    body, content_type = multipart(audio)
    transcribed = json.load(
        post(base, arguments.token, "/v1/audio/transcriptions", body, content_type)
    )
    print("transcribe:", transcribed["text"])

    body, content_type = multipart(audio)
    translated = json.load(
        post(base, arguments.token, "/v1/audio/translations", body, content_type)
    )
    print("translate:", translated["text"])

    spoken = post(
        base,
        arguments.token,
        "/v1/audio/speech",
        json.dumps({"input": "Bom dia", "response_format": "wav"}).encode(),
        "application/json",
    ).read()
    if spoken[:4] != b"RIFF":
        raise SystemExit("speech did not return a WAV file")
    print("speech:", len(spoken), "bytes")

    body, content_type = multipart(audio, model="not-a-model")
    try:
        send(base, arguments.token, "/v1/audio/transcriptions", body, content_type)
    except urllib.error.HTTPError as error:
        print("refused:", error.code)
    else:
        raise SystemExit("an unsupported model was accepted")


if __name__ == "__main__":
    main()
