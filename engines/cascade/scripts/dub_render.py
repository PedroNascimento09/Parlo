import json
from dataclasses import replace
from pathlib import Path

import numpy as np

from dub_audio import read_wav
from dub_model import SAMPLE_RATE, Line, line_document

DUCKING_FLOOR = 0.18


def inspect(line: Line, media: float) -> Line:
    if line.failed:
        return replace(line, issues=("failed", "unverified"))
    if line.skipped:
        return replace(line, issues=("skipped", "unverified"))
    issues = []
    slot = line.slot_end if line.slot_end is not None else line.end
    if line.end > slot + 0.02:
        issues.append("overlong")
    if line.end > media + 0.02:
        issues.append("clipped")
    return replace(line, issues=tuple(issues))


def rebuild(lines: list[Line], length: int, directory: Path) -> np.ndarray:
    track = np.zeros(length, dtype=np.float32)
    for line in lines:
        if line.failed or line.skipped or not line.file:
            continue
        path = directory / line.file
        if not path.is_file():
            label = line.id or f"{line.start:.2f}s"
            raise SystemExit(f"the audio for line {label} is missing; dub the file again")
        audio = read_wav(path)
        start = max(int(round(line.start * SAMPLE_RATE)), 0)
        end = min(start + len(audio), length)
        if end > start:
            track[start:end] += audio[: end - start]
    return np.clip(track, -1.0, 1.0)


def ducking(length: int, lines: list[Line], fade_seconds: float = 0.15) -> np.ndarray:
    envelope = np.ones(length, dtype=np.float32)
    fade = max(int(fade_seconds * SAMPLE_RATE), 1)
    for line in lines:
        start = max(int(line.start * SAMPLE_RATE) - fade, 0)
        end = min(int(line.end * SAMPLE_RATE) + fade, length)
        if end > start:
            envelope[start:end] = DUCKING_FLOOR
    kernel = np.ones(fade * 2 + 1, dtype=np.float32) / (fade * 2 + 1)
    smoothed = np.convolve(envelope, kernel, mode="same")
    return np.clip(smoothed, DUCKING_FLOOR, 1.0).astype(np.float32)


def write_report(
    path: Path,
    args,
    out: Path,
    srt: Path,
    samples: np.ndarray,
    lines: list[Line],
    written: list[dict],
    separation_mode: str,
    separation_note: str | None,
) -> None:
    path.write_text(
        json.dumps(
            {
                "source": str(args.source),
                "out": str(out),
                "srt": str(srt),
                "from": args.source_language,
                "to": args.to,
                "timing": args.timing,
                "fastest": args.fastest,
                "slowest": args.slowest,
                "seconds": round(len(samples) / SAMPLE_RATE, 2),
                "lines": [line_document(line) for line in lines],
                "exports": written,
                "separation": separation_mode,
                "separation_note": separation_note,
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )


