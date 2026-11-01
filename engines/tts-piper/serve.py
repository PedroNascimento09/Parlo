import base64
import io
import json
import os
import sys
import wave


def reply(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload) + "\n")
    sys.stdout.flush()


def synthesize(voice, text: str) -> tuple[bytes, int]:
    import numpy as np

    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as wav:
        voice.synthesize_wav(text, wav)
    buffer.seek(0)
    with wave.open(buffer, "rb") as wav:
        rate = wav.getframerate()
        channels = wav.getnchannels()
        frames = wav.readframes(wav.getnframes())

    samples = np.frombuffer(frames, dtype="<i2")
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)
    return samples.astype("<i2").tobytes(), rate


def main() -> int:
    voice_path = os.environ.get("PARLO_PIPER_VOICE")
    if not voice_path:
        reply({"op": "hello", "available": False, "error": "PARLO_PIPER_VOICE is not set"})
        return 2

    try:
        from piper import PiperVoice
    except Exception as error:
        reply(
            {
                "op": "hello",
                "available": False,
                "error": f"piper is not installed: {error}",
            }
        )
        return 2

    try:
        voice = PiperVoice.load(str(voice_path))
    except Exception as error:
        reply(
            {
                "op": "hello",
                "available": False,
                "error": f"cannot load {voice_path}: {error}",
            }
        )
        return 2

    reply({"op": "hello", "engine": "piper", "available": True})

    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            reply({"op": "error", "error": "invalid json"})
            continue
        if not isinstance(request, dict):
            reply({"op": "error", "error": "request must be an object"})
            continue
        if request.get("op") != "synthesize":
            reply({"op": "error", "error": f"unknown op: {request.get('op')}"})
            continue
        text = request.get("text")
        if not isinstance(text, str) or not text.strip():
            reply({"op": "error", "error": "text must be a non empty string"})
            continue
        try:
            pcm16, rate = synthesize(voice, text)
        except Exception as error:
            reply({"op": "error", "error": str(error)})
            continue
        reply(
            {
                "op": "synthesized",
                "pcm16": base64.b64encode(pcm16).decode("ascii"),
                "sample_rate": rate,
            }
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
