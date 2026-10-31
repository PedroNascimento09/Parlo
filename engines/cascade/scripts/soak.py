import argparse
import json
import statistics
import sys
import time
import wave
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import CascadeEngine, float_to_pcm16, resample


def load(path: Path) -> bytes:
    with wave.open(str(path)) as wav:
        rate, channels = wav.getframerate(), wav.getnchannels()
        raw = wav.readframes(wav.getnframes())
    samples = np.frombuffer(raw, dtype="<i2").astype("float32") / 32_768.0
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)
    return float_to_pcm16(resample(samples, rate, 16_000))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("wav")
    parser.add_argument("--minutes", type=float, default=30.0)
    parser.add_argument("--threads", type=int, default=4)
    args = parser.parse_args()

    engine = CascadeEngine()
    engine.warmup()
    pcm = load(Path(args.wav))

    deadline = time.perf_counter() + args.minutes * 60
    latencies = []
    segments = 0
    errors = 0
    cycle = 0
    started = time.perf_counter()

    while time.perf_counter() < deadline:
        cycle += 1
        for index in range(0, len(pcm), 640):
            result = engine.translate(pcm[index : index + 640])
            if result.text:
                segments += 1
                latencies.append(result.latency_ms)
                if result.latency_ms <= 0:
                    errors += 1
        elapsed = time.perf_counter() - started
        print(
            f"  {elapsed:6.0f} s  cycles {cycle:3d}  segments {segments:4d}  "
            f"mean {statistics.mean(latencies):5.0f} ms"
            if latencies
            else f"  {elapsed:6.0f} s  cycles {cycle:3d}  no speech",
            flush=True,
        )

    summary = {
        "cycles": cycle,
        "segments": segments,
        "minutes": round((time.perf_counter() - started) / 60, 1),
        "mean_ms": round(statistics.mean(latencies), 1) if latencies else None,
        "median_ms": round(statistics.median(latencies), 1) if latencies else None,
        "p95_ms": round(sorted(latencies)[int(len(latencies) * 0.95)], 1) if latencies else None,
        "first_ten_ms": round(statistics.mean(latencies[:10]), 1) if len(latencies) >= 10 else None,
        "last_ten_ms": round(statistics.mean(latencies[-10:]), 1) if len(latencies) >= 10 else None,
        "errors": errors,
    }
    print(json.dumps(summary))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
