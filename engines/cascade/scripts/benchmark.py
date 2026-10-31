import argparse
import json
import os
import sys
import time
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

import numpy as np

from parlo_cascade import CascadeEngine, float_to_pcm16, resample


def load_audio(path: Path) -> bytes:
    with wave.open(str(path)) as wav:
        rate, channels = wav.getframerate(), wav.getnchannels()
        raw = wav.readframes(wav.getnframes())
    samples = np.frombuffer(raw, dtype="<i2").astype("float32") / 32_768.0
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)
    return float_to_pcm16(resample(samples, rate, 16_000))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("audio")
    parser.add_argument("--model", default="small")
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument("--segment", type=float, default=2.5)
    parser.add_argument("--source", default="pt")
    parser.add_argument("--target", default="en")
    args = parser.parse_args()

    os.environ["PARLO_CASCADE_MODEL"] = args.model
    os.environ["PARLO_CASCADE_SEGMENT"] = str(args.segment)
    if args.threads:
        os.environ["PARLO_CASCADE_THREADS"] = str(args.threads)
    else:
        os.environ.pop("PARLO_CASCADE_THREADS", None)

    pcm = load_audio(Path(args.audio))
    duration = len(pcm) / 2 / 16_000
    engine = CascadeEngine(source=args.source, target=args.target)

    started = time.perf_counter()
    engine.warmup()
    warmup_seconds = time.perf_counter() - started

    transcripts = []
    latencies = []
    started = time.perf_counter()
    for index in range(0, len(pcm), 640):
        result = engine.translate(pcm[index : index + 640])
        if result.text:
            transcripts.append(result.text)
        if result.latency_ms:
            latencies.append(result.latency_ms)
    wall = time.perf_counter() - started

    print(
        json.dumps(
            {
                "model": args.model,
                "threads": args.threads or None,
                "segment": args.segment,
                "audio_seconds": round(duration, 2),
                "warmup_seconds": round(warmup_seconds, 2),
                "wall_seconds": round(wall, 2),
                "rtf": round(wall / duration, 2),
                "segment_ms": latencies,
                "segments": len(latencies),
                "text": " | ".join(transcripts),
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
