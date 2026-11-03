import argparse
import gc
import importlib.util
import json
import os
import sys
import time
import wave
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

CLIP = Path("out/falar/clips/falar-000.wav")
SENTENCE = "o prazo de entrega é dia vinte e três de março"


def memory_mb() -> float:
    if importlib.util.find_spec("psutil"):
        import psutil

        return round(psutil.Process().memory_info().rss / (1024 * 1024), 1)
    import ctypes
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [
            ("cb", wintypes.DWORD),
            ("PageFaultCount", wintypes.DWORD),
            ("PeakWorkingSetSize", ctypes.c_size_t),
            ("WorkingSetSize", ctypes.c_size_t),
            ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
            ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
            ("PagefileUsage", ctypes.c_size_t),
            ("PeakPagefileUsage", ctypes.c_size_t),
        ]

    counters = Counters()
    counters.cb = ctypes.sizeof(Counters)
    process = ctypes.windll.kernel32.GetCurrentProcess()
    ctypes.windll.psapi.GetProcessMemoryInfo(
        process, ctypes.byref(counters), ctypes.sizeof(Counters)
    )
    return round(counters.WorkingSetSize / (1024 * 1024), 1)


def read_clip(path: Path) -> np.ndarray:
    with wave.open(str(path), "rb") as handle:
        frames = handle.readframes(handle.getnframes())
    return np.frombuffer(frames, dtype="<i2").astype("float32") / 32768.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--unit", required=True)
    parser.add_argument("--out", default="")
    args = parser.parse_args()

    from parlo_cascade import CascadeEngine

    before = memory_mb()
    started = time.perf_counter()
    result = {"unit": args.unit, "memory_before_mb": before}

    if args.unit in ("parakeet", "whisper"):
        os.environ["PARLO_CASCADE_STT"] = args.unit
        if args.unit == "whisper":
            os.environ.setdefault("PARLO_CASCADE_MODEL", "base")
        engine = CascadeEngine(source="pt", target="en")
        model = engine._load_stt()
        if args.unit == "parakeet":
            warm = model.create_stream()
            warm.accept_waveform(16_000, np.zeros(16_000, dtype="float32"))
            model.decode_stream(warm)
        else:
            list(model.transcribe(np.zeros(16_000, dtype="float32"), language="pt")[0])
        loaded = time.perf_counter()
        audio = read_clip(CLIP)
        if args.unit == "parakeet":
            stream = model.create_stream()
            stream.accept_waveform(16_000, audio)
            model.decode_stream(stream)
            text = stream.result.text.strip()
        else:
            segments, _ = model.transcribe(audio, language="pt", beam_size=1, vad_filter=False)
            text = " ".join(segment.text.strip() for segment in segments).strip()
        elapsed = time.perf_counter() - loaded
        seconds = len(audio) / 16_000
        result.update(
            {
                "load_seconds": round(loaded - started, 2),
                "seconds_audio": round(seconds, 2),
                "seconds_work": round(elapsed, 2),
                "realtime_factor": round(elapsed / seconds, 3),
                "text": text,
            }
        )
    elif args.unit.startswith("voice"):
        engine = CascadeEngine(source="pt", target="en")
        name = "en_US-lessac-medium.onnx" if args.unit == "voice-en" else "pt_PT-tugão-medium.onnx"
        chosen = ROOT / "models" / name
        if chosen.is_file():
            engine.voice_path = chosen
        engine._load_voice()
        engine._synthesize("teste")
        loaded = time.perf_counter()
        audio = engine._synthesize(SENTENCE)
        elapsed = time.perf_counter() - loaded
        seconds = len(audio) / 2 / 16_000
        result.update(
            {
                "load_seconds": round(loaded - started, 2),
                "seconds_audio": round(seconds, 2),
                "seconds_work": round(elapsed, 2),
                "realtime_factor": round(elapsed / max(seconds, 0.01), 3),
            }
        )
    elif args.unit.startswith("mt"):
        source, target = ("en", "pt") if args.unit == "mt-en-pt" else ("pt", "en")
        engine = CascadeEngine(source=source, target=target)
        translator = engine._load_translation()
        translator("teste")
        loaded = time.perf_counter()
        translated = translator(SENTENCE)
        elapsed = time.perf_counter() - loaded
        result.update(
            {
                "load_seconds": round(loaded - started, 2),
                "sentence_ms": round(elapsed * 1000, 1),
                "text": translated,
            }
        )
    else:
        raise SystemExit(f"unknown unit {args.unit}")

    gc.collect()
    result["memory_after_mb"] = memory_mb()
    result["memory_delta_mb"] = round(result["memory_after_mb"] - before, 1)
    print(json.dumps(result, ensure_ascii=False))
    if args.out:
        Path(args.out).write_text(
            json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
