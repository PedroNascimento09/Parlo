import argparse
import json
import statistics
import sys
import time
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from battery import add_noise, word_error_rate
from parlo_cascade import CascadeEngine, float_to_pcm16, pcm16_to_float

CHUNK_MS = 100
DEFAULT_CORPUS = ROOT / "tests" / "corpus" / "pt_pt.txt"


def load_corpus(path: Path) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    return [
        line.strip()
        for line in lines
        if line.strip() and not line.strip().startswith("#")
    ]


def percentile(values: list[float], share: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = min(int(round(len(ordered) * share)), len(ordered) - 1)
    return round(ordered[index], 1)


def run(
    engine: CascadeEngine,
    samples: np.ndarray,
    chunk: int,
    lead: int,
    tail: int,
    snr: float | None,
    seed: int,
) -> list[tuple[float, object]]:
    audio = np.concatenate(
        [np.zeros(lead, dtype="float32"), samples, np.zeros(tail, dtype="float32")]
    )
    if snr is not None:
        audio = add_noise(audio, snr, seed)
    started = time.perf_counter()
    events = []
    for index in range(0, len(audio), chunk):
        pcm = float_to_pcm16(audio[index : index + chunk])
        for event in engine.stream(pcm):
            events.append((time.perf_counter() - started, event))
    return events


def summarise(reference: str, events: list[tuple[float, object]]) -> dict:
    partials = [(wall, event) for wall, event in events if event.state == "partial"]
    stables = [(wall, event) for wall, event in events if event.state == "stable"]
    finals = [(wall, event) for wall, event in events if event.state == "final"]
    corrected = [event for _, event in events if event.state == "corrected"]
    partial_text = partials[-1][1].text if partials else ""
    stable_text = " ".join(event.text for _, event in stables).strip()
    final_text = finals[-1][1].text if finals else ""
    latencies = [event.latency_ms for _, event in events]

    def moment(event):
        return event.audio_ms + event.latency_ms

    return {
        "first_partial_ms": moment(partials[0][1]) if partials else None,
        "first_stable_ms": moment(stables[0][1]) if stables else None,
        "final_ms": moment(finals[0][1]) if finals else None,
        "process_ms": round(events[-1][0] * 1000, 1) if events else None,
        "decode_ms": round(statistics.mean(latencies), 1) if latencies else None,
        "corrections": len(corrected),
        "partial": partial_text,
        "stable": stable_text,
        "final": final_text,
        "partial_wer": round(word_error_rate(reference, partial_text), 3),
        "stable_wer": round(word_error_rate(reference, stable_text), 3),
        "stable_coverage": round(
            len(stable_text.split()) / max(len(reference.split()), 1), 3
        ),
        "final_wer": round(word_error_rate(reference, final_text), 3),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    parser.add_argument("--stt", choices=["parakeet", "whisper"], default="parakeet")
    parser.add_argument("--model", default="base")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--decode-every", type=float, default=0.4)
    parser.add_argument("--rounds", type=int, default=1)
    parser.add_argument("--segment", type=float, default=12.0)
    parser.add_argument("--snr", type=float, default=None)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--lead", type=float, default=0.0)
    parser.add_argument("--tail", type=float, default=0.8)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--events", action="store_true")
    parser.add_argument("--voice", default="")
    parser.add_argument("--out", default="")
    args = parser.parse_args()

    sentences = load_corpus(Path(args.corpus))
    if args.limit > 0:
        sentences = sentences[: args.limit]
    if not sentences:
        raise SystemExit(f"no sentences in {args.corpus}")

    engine = CascadeEngine(source="pt", target="en")
    if args.voice:
        chosen = Path(args.voice)
        engine.voice_path = chosen if chosen.exists() else ROOT / "models" / args.voice
    if not engine.available:
        raise SystemExit("install the cascade requirements first")
    if not engine.voice_path.exists():
        raise SystemExit("missing voice model, run scripts/setup_models.py")
    from parlo_cascade.streaming import StreamingSession

    engine._stream = StreamingSession(
        engine._transcribe,
        decode_every=args.decode_every,
        segment_seconds=args.segment,
        rounds=args.rounds,
    )
    engine._load_stt()

    chunk = int(CHUNK_MS * 16_000 / 1000)
    lead = int(args.lead * 16_000)
    tail = int(args.tail * 16_000)
    rows = []
    print(
        f"\n{args.stt} | rounds {args.rounds} | decode every {args.decode_every:.2f} s"
        f" | voice {engine.voice_path.name} | corpus {Path(args.corpus).name}"
        f" | {len(sentences)} sentences"
    )
    for index, sentence in enumerate(sentences):
        samples = np.asarray(
            pcm16_to_float(engine._synthesize(sentence)), dtype="float32"
        )
        audio_seconds = len(samples) / 16_000
        events = run(engine, samples, chunk, lead, tail, args.snr, args.seed + index)
        row = summarise(sentence, events)
        row["sentence"] = sentence
        row["audio_seconds"] = round(audio_seconds, 2)
        rows.append(row)
        if args.events:
            for _, event in events:
                print(
                    f"      {event.audio_ms:>6d} ms audio  "
                    f"{event.latency_ms:>5d} ms decode  "
                    f"{event.state:<9} {event.text}"
                )
        print(
            f"  {row['first_partial_ms'] or 0:>6.0f} ms  "
            f"{row['first_stable_ms'] or 0:>6.0f} ms  "
            f"{row['final_ms'] or 0:>6.0f} ms  "
            f"WER {row['partial_wer']:.2f} / {row['stable_wer']:.2f} / "
            f"{row['final_wer']:.2f}  {row['final'][:64]}"
        )

    partials = [row["first_partial_ms"] for row in rows if row["first_partial_ms"]]
    stables = [row["first_stable_ms"] for row in rows if row["first_stable_ms"]]
    finals = [row["final_ms"] for row in rows if row["final_ms"]]
    summary = {
        "stt": args.stt,
        "model": args.model if args.stt == "whisper" else "",
        "threads": args.threads,
        "decode_every": args.decode_every,
        "rounds": args.rounds,
        "segment": args.segment,
        "snr": args.snr,
        "seed": args.seed,
        "sentences": len(rows),
        "first_partial_ms": round(statistics.mean(partials), 1) if partials else None,
        "first_stable_ms": round(statistics.mean(stables), 1) if stables else None,
        "final_ms": round(statistics.mean(finals), 1) if finals else None,
        "p95_final_ms": percentile(finals, 0.95),
        "mean_partial_wer": round(statistics.mean(row["partial_wer"] for row in rows), 3),
        "mean_stable_wer": round(statistics.mean(row["stable_wer"] for row in rows), 3),
        "mean_stable_coverage": round(
            statistics.mean(row["stable_coverage"] for row in rows), 3
        ),
        "mean_final_wer": round(statistics.mean(row["final_wer"] for row in rows), 3),
        "corrections": sum(row["corrections"] for row in rows),
    }
    print("\n" + json.dumps(summary, indent=2))
    if args.out:
        Path(args.out).write_text(
            json.dumps({"summary": summary, "rows": rows}, indent=2), encoding="utf-8"
        )
        print(f"written {args.out}")
    return 1 if summary["mean_final_wer"] > 0.35 else 0


if __name__ == "__main__":
    raise SystemExit(main())
