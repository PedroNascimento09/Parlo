import base64
import binascii
import json

from .engine import EngineUnavailable


def hello(engine) -> dict:
    return {
        "op": "hello",
        "engine": engine.id,
        "available": engine.available,
        "sample_rate": engine.sample_rate,
        "target_languages": list(engine.target_languages),
    }


def handle_line(line: str, engine) -> dict:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        return {"op": "error", "error": "invalid json"}

    if not isinstance(request, dict):
        return {"op": "error", "error": "request must be an object"}

    op = request.get("op")
    if op == "hello":
        return hello(engine)
    if op == "translate":
        return translate(request, engine)
    return {"op": "error", "error": f"unknown op: {op}"}


def translate(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    payload = request.get("pcm16")
    if not isinstance(payload, str):
        return {"op": "error", "seq": seq, "error": "pcm16 must be base64 text"}

    try:
        pcm16 = base64.b64decode(payload, validate=True)
    except (binascii.Error, ValueError):
        return {"op": "error", "seq": seq, "error": "pcm16 is not valid base64"}

    if len(pcm16) % 2 != 0:
        return {"op": "error", "seq": seq, "error": "pcm16 must hold 16 bit samples"}

    try:
        result = engine.translate(pcm16)
    except EngineUnavailable as error:
        return {"op": "error", "seq": seq, "error": str(error)}

    return {
        "op": "translated",
        "seq": seq,
        "pcm16": base64.b64encode(result.pcm16).decode("ascii"),
        "text": result.text,
        "latency_ms": result.latency_ms,
    }
