import argparse
import json
import os
import platform
import statistics
import subprocess
import sys
import time
import wave
from pathlib import Path

HERE = Path(__file__).resolve().parent
ENGINE = HERE.parent
REPO = HERE.parents[2]
sys.path.insert(0, str(ENGINE))

import numpy as np

from parlo_cascade import CascadeEngine, float_to_pcm16

RATE = 16_000
CLIP_SECONDS = 5.0
TONE_LEAD_SECONDS = 1.0
TONE_TAIL_SECONDS = 1.4
SPEECH_LEAD_SECONDS = 1.1
SPEECH = ENGINE / "tests" / "fixtures" / "timing" / "source.wav"
SPEECH_BASELINE = REPO / "docs" / "benchmarks" / "dictate-speech.json"
TONE_BASELINE = REPO / "docs" / "benchmarks" / "dictate.json"


def tone():
    steps = int(CLIP_SECONDS * RATE)
    moment = np.arange(steps, dtype=np.float32) / RATE
    voiced = (
        (moment >= TONE_LEAD_SECONDS) & (moment <= CLIP_SECONDS - TONE_TAIL_SECONDS)
    ).astype(np.float32)
    burst = (np.sin(2 * np.pi * 0.8 * moment) > -0.2).astype(np.float32)
    carrier = 0.18 * np.sin(2 * np.pi * 180 * moment) + 0.06 * np.sin(
        2 * np.pi * 620 * moment
    )
    return (carrier * voiced * (0.6 + 0.4 * burst)).astype(np.float32)


def speech_seconds():
    with wave.open(str(SPEECH)) as wav:
        return wav.getnframes() / wav.getframerate()


def speech():
    with wave.open(str(SPEECH)) as wav:
        if (wav.getframerate(), wav.getnchannels(), wav.getsampwidth()) != (RATE, 1, 2):
            raise SystemExit(f"{SPEECH} must be 16 kHz mono 16-bit")
        raw = wav.readframes(wav.getnframes())
    samples = np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32_768.0
    total = int(CLIP_SECONDS * RATE)
    start = int(SPEECH_LEAD_SECONDS * RATE)
    if start + samples.size > total:
        raise SystemExit(f"{SPEECH} does not fit in a {CLIP_SECONDS:.2f} s clip")
    padded = np.zeros(total, dtype=np.float32)
    padded[start : start + samples.size] = samples
    return padded


def clip(kind):
    return speech() if kind == "speech" else tone()


def describe(kind):
    if kind == "tone":
        return (
            f"{CLIP_SECONDS:.2f} s synthetic: {TONE_LEAD_SECONDS:.2f} s silence, "
            f"{CLIP_SECONDS - TONE_LEAD_SECONDS - TONE_TAIL_SECONDS:.2f} s voiced, "
            f"{TONE_TAIL_SECONDS:.2f} s silence"
        )
    tail = CLIP_SECONDS - SPEECH_LEAD_SECONDS - speech_seconds()
    return (
        f"{CLIP_SECONDS:.2f} s speech: {SPEECH_LEAD_SECONDS:.2f} s silence, "
        f"{speech_seconds():.2f} s speech from {SPEECH.relative_to(REPO)}, "
        f"{tail:.2f} s silence"
    )


def machine():
    return f"{platform.system()} {platform.machine()}, {os.cpu_count()} threads"


def commit():
    try:
        return subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=REPO,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def load_baseline(path):
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None
    return payload if isinstance(payload, dict) else None


def record_baseline(
    path, kind, summary, control, gain, transcript, runs, threads, load_ms, warmup_ms
):
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "clip_kind": kind,
        "clip": describe(kind),
        "clip_seconds": CLIP_SECONDS,
        "speech_lead_seconds": SPEECH_LEAD_SECONDS,
        "threads": threads,
        "runs": runs,
        "load_ms": load_ms,
        "warmup_ms": warmup_ms,
        "transcript": transcript,
        **summary,
        "control": control,
        "trim_gain_percent": gain,
        "machine": machine(),
        "commit": commit(),
        "recorded_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return payload


def medians(rows):
    return {
        key: round(statistics.median(row[key] for row in rows), 1)
        for key in ("decode_ms", "post_ms", "total_ms", "cpu_ms")
    }


def timed(engine, pcm, runs):
    rows = []
    for _ in range(runs):
        began = time.perf_counter()
        cpu_began = time.process_time()
        text, _ = engine.write(pcm, target="pt", clean_mode="light")
        cpu = round((time.process_time() - cpu_began) * 1000, 1)
        total = round((time.perf_counter() - began) * 1000, 1)
        timings = engine.last_timings or {}
        rows.append(
            {
                "decode_ms": round(float(timings.get("decode_ms", 0.0)), 1),
                "post_ms": round(float(timings.get("post_ms", 0.0)), 1),
                "total_ms": total,
                "cpu_ms": cpu,
                "text": text,
            }
        )
    return rows


def measure(threads, runs, pcm):
    os.environ["PARLO_CASCADE_STT"] = "parakeet"
    os.environ["PARLO_CASCADE_THREADS"] = str(threads)
    engine = CascadeEngine(source="pt", target="pt")
    started = time.perf_counter()
    engine.load_unit("recognition")
    load_ms = round((time.perf_counter() - started) * 1000, 1)
    engine.trim = True
    started = time.perf_counter()
    engine.write(pcm, target="pt", clean_mode="light")
    warmup_ms = round((time.perf_counter() - started) * 1000, 1)
    return engine, load_ms, warmup_ms, timed(engine, pcm, runs)


def change(baseline, summary):
    if not baseline:
        return None
    before = baseline.get("total_ms")
    after = summary.get("total_ms")
    if not isinstance(before, (int, float)) or not isinstance(after, (int, float)):
        return None
    if before <= 0:
        return None
    return round((after - before) / before * 100, 1)


def report(kind, threads, runs, baseline_path, record, compare, as_json):
    engine, load_ms, warmup_ms, rows = measure(threads, runs, float_to_pcm16(clip(kind)))
    summary = medians(rows)
    transcript = rows[0]["text"]
    control = None
    gain = None
    if compare:
        engine.trim = False
        control = medians(timed(engine, float_to_pcm16(clip(kind)), runs))
        engine.trim = True
        gain = round(
            (control["total_ms"] - summary["total_ms"]) / control["total_ms"] * 100,
            1,
        )
    baseline = load_baseline(baseline_path)
    recorded = (
        record_baseline(
            baseline_path,
            kind,
            summary,
            control,
            gain,
            transcript,
            runs,
            threads,
            load_ms,
            warmup_ms,
        )
        if record
        else None
    )
    delta = change(baseline, summary)
    if as_json:
        print(
            json.dumps(
                {
                    "engine": "parakeet",
                    "clip_kind": kind,
                    "clip": describe(kind),
                    "threads": threads,
                    "runs": runs,
                    "clip_seconds": CLIP_SECONDS,
                    "load_ms": load_ms,
                    "warmup_ms": warmup_ms,
                    "transcript": transcript,
                    "runs_detail": rows,
                    "median": summary,
                    "control": control,
                    "trim_gain_percent": gain,
                    "baseline": baseline,
                    "baseline_recorded": recorded,
                    "change_percent": delta,
                    "machine": machine(),
                    "commit": commit(),
                }
            )
        )
        return
    print("parlo dictation ruler")
    print(f"  clip        {describe(kind)}")
    print(f"  engine      parakeet, {threads} threads, {machine()}")
    print(f"  load        {load_ms:.1f} ms (cold recogniser in this process)")
    print(f"  warmup      {warmup_ms:.1f} ms (first write, not counted)")
    print(f"  transcript  {transcript!r}" if transcript else "  transcript  none")
    print()
    print("  run   decode ms   post ms   total ms   cpu ms")
    for index, row in enumerate(rows, start=1):
        print(
            f"  {index:>3}   {row['decode_ms']:>9.1f}   {row['post_ms']:>7.1f}"
            f"   {row['total_ms']:>8.1f}   {row['cpu_ms']:>6.1f}"
        )
    print(
        f"  median{summary['decode_ms']:>9.1f}   {summary['post_ms']:>7.1f}"
        f"   {summary['total_ms']:>8.1f}   {summary['cpu_ms']:>6.1f}"
    )
    print()
    if control:
        print(f"  trim off    {control['total_ms']:.1f} ms total (same process, control)")
        print(f"  gain        {gain:+.1f}% from trimming leading and trailing silence")
    if not transcript:
        print("  note        post is 0.0 ms because this clip decodes to no words")
    print()
    if recorded:
        print(f"  baseline    recorded {summary['total_ms']:.1f} ms in {baseline_path}")
    elif baseline:
        print(
            f"  baseline    {baseline.get('total_ms')} ms, "
            f"{baseline.get('commit')}, {baseline.get('machine')}"
        )
        if delta is not None:
            print(f"  change      {delta:+.1f}% total")
    else:
        print("  baseline    none yet, record it with --record-baseline")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--clip", choices=("speech", "tone"), default="speech")
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--baseline", default="")
    parser.add_argument("--record-baseline", action="store_true")
    parser.add_argument("--no-compare", dest="compare", action="store_false")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.runs < 3:
        print("the ruler needs at least three runs for a median", file=sys.stderr)
        return 2
    baseline = (
        Path(args.baseline)
        if args.baseline
        else (SPEECH_BASELINE if args.clip == "speech" else TONE_BASELINE)
    )
    report(
        args.clip,
        args.threads,
        args.runs,
        baseline,
        args.record_baseline,
        args.compare,
        args.json,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
