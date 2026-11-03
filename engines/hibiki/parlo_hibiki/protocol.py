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


def loaded(engine) -> dict:
    units = getattr(engine, "loaded_units", None)
    return {"op": "loaded", "units": list(units()) if callable(units) else []}


def lifecycle(engine) -> dict:
    report = getattr(engine, "lifecycle", None)
    if not callable(report):
        return {"op": "lifecycle", "manage": True, "units": []}
    state = report()
    return {"op": "lifecycle", "manage": state["manage"], "units": state["units"]}


def switch_unit(request: dict, engine) -> dict:
    unit = request.get("unit")
    if not isinstance(unit, str) or not unit:
        return {"op": "error", "error": "unit must be text"}
    loading = request.get("op") == "load"
    method = getattr(engine, "load_unit" if loading else "unload_unit", None)
    if not callable(method):
        return {"op": "error", "error": "this engine cannot change what it holds"}
    try:
        method(unit)
    except (EngineUnavailable, ValueError) as failure:
        return {"op": "error", "error": str(failure)}
    units = getattr(engine, "loaded_units", None)
    held = callable(units) and unit in units()
    return {"op": "model", "unit": unit, "state": "loaded" if held else "unloaded"}


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
    if op == "loaded":
        return loaded(engine)
    if op == "lifecycle":
        return lifecycle(engine)
    if op in ("load", "unload"):
        return switch_unit(request, engine)
    if op == "translate":
        return translate(request, engine)
    if op == "dictate":
        return dictate(request, engine)
    if op == "write":
        return write(request, engine)
    if op == "stream":
        return stream(request, engine)
    if op == "stream_close":
        return close_stream(request, engine)
    return {"op": "error", "error": f"unknown op: {op}"}


def decode_audio(request: dict):
    seq = request.get("seq", 0)
    payload = request.get("pcm16")
    if not isinstance(payload, str):
        return None, {"op": "error", "seq": seq, "error": "pcm16 must be base64 text"}

    try:
        pcm16 = base64.b64decode(payload, validate=True)
    except (binascii.Error, ValueError):
        return None, {"op": "error", "seq": seq, "error": "pcm16 is not valid base64"}

    if len(pcm16) % 2 != 0:
        return None, {"op": "error", "seq": seq, "error": "pcm16 must hold 16 bit samples"}
    return pcm16, None


def translate(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    pcm16, error = decode_audio(request)
    if error is not None:
        return error

    try:
        result = engine.translate(pcm16)
    except EngineUnavailable as failure:
        return {"op": "error", "seq": seq, "error": str(failure)}

    return {
        "op": "translated",
        "seq": seq,
        "pcm16": base64.b64encode(result.pcm16).decode("ascii"),
        "text": result.text,
        "latency_ms": result.latency_ms,
    }


def dictate(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    if not hasattr(engine, "transcribe"):
        return {"op": "error", "seq": seq, "error": "engine cannot transcribe on its own"}
    pcm16, error = decode_audio(request)
    if error is not None:
        return error
    try:
        text = engine.transcribe(pcm16)
    except EngineUnavailable as failure:
        return {"op": "error", "seq": seq, "error": str(failure)}
    return {"op": "dictated", "seq": seq, "text": text}


def write(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    if not hasattr(engine, "write"):
        return {
            "op": "error",
            "seq": seq,
            "error": "engine cannot write in another language",
        }
    pcm16, error = decode_audio(request)
    if error is not None:
        return error
    target = request.get("target")
    clean = request.get("clean")
    try:
        text, translation = engine.write(
            pcm16,
            target if isinstance(target, str) else None,
            clean if isinstance(clean, str) else None,
        )
    except EngineUnavailable as failure:
        return {"op": "error", "seq": seq, "error": str(failure)}
    return {
        "op": "written",
        "seq": seq,
        "text": text,
        "translation": translation,
    }


def stream(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    if not hasattr(engine, "stream"):
        return {"op": "error", "seq": seq, "error": "engine does not stream"}
    pcm16, error = decode_audio(request)
    if error is not None:
        return error
    try:
        events = engine.stream(pcm16)
    except EngineUnavailable as failure:
        return {"op": "error", "seq": seq, "error": str(failure)}
    return stream_response(engine, seq, events)


def close_stream(request: dict, engine) -> dict:
    seq = request.get("seq", 0)
    if not hasattr(engine, "close_stream"):
        return {"op": "error", "seq": seq, "error": "engine does not stream"}
    try:
        events = engine.close_stream()
    except EngineUnavailable as failure:
        return {"op": "error", "seq": seq, "error": str(failure)}
    return stream_response(engine, seq, events)


def stream_response(engine, seq, events) -> dict:
    return {
        "op": "stream",
        "seq": seq,
        "mode": getattr(engine, "stream_mode", "reprocess"),
        "events": [event.as_dict() for event in events],
    }
