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


def load(path: Path) -> tuple[bytes, float]:
    with wave.open(str(path)) as wav:
        rate, channels = wav.getframerate(), wav.getnchannels()
        raw = wav.readframes(wav.getnframes())
    samples = np.frombuffer(raw, dtype="<i2").astype("float32") / 32_768.0
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)
    pcm = float_to_pcm16(resample(samples, rate, 16_000))
    return pcm, len(pcm) / 2 / 16_000


def run(engine: CascadeEngine, pcm: bytes) -> tuple[float, list[str]]:
    texts = []
    started = time.perf_counter()
    for index in range(0, len(pcm), 640):
        result = engine.translate(pcm[index : index + 640])
        if result.text:
            texts.append(result.text)
    return time.perf_counter() - started, texts


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("folder")
    parser.add_argument("--recognisers", nargs="+", default=["parakeet", "whisper"])
    parser.add_argument("--models", nargs="+", default=["base"])
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument("--segment", type=float, default=2.5)
    parser.add_argument("--source", default="pt")
    parser.add_argument("--target", default="en")
    args = parser.parse_args()

    files = sorted(Path(args.folder).glob("*.wav"))
    if not files:
        raise SystemExit(f"no wav files in {args.folder}")

    os.environ["PARLO_CASCADE_SEGMENT"] = str(args.segment)
    if args.threads:
        os.environ["PARLO_CASCADE_THREADS"] = str(args.threads)

    report = {}
    for recogniser in args.recognisers:
        models = args.models if recogniser == "whisper" else [""]
        for model in models:
            os.environ["PARLO_CASCADE_STT"] = recogniser
            if model:
                os.environ["PARLO_CASCADE_MODEL"] = model
            label = f"{recogniser} {model}".strip()
            engine = CascadeEngine(source=args.source, target=args.target)
            engine.warmup()
            rows = []
            for path in files:
                pcm, duration = load(path)
                wall, texts = run(engine, pcm)
                rows.append(
                    {
                        "file": path.name,
                        "audio_seconds": round(duration, 2),
                        "wall_seconds": round(wall, 2),
                        "rtf": round(wall / duration, 2),
                        "text": " ".join(texts),
                    }
                )
            mean_rtf = round(sum(row["rtf"] for row in rows) / len(rows), 2)
            report[label] = {"mean_rtf": mean_rtf, "files": rows}
            print(f"\n{label} | mean RTF {mean_rtf}")
            for row in rows:
                print(
                    f"  {row['file']:<24} {row['audio_seconds']:>5.2f} s  "
                    f"RTF {row['rtf']:>4.2f}  {row['text'][:90]}"
                )

    print("\n" + json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
