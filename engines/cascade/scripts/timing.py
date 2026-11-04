import math
import subprocess
from dataclasses import dataclass
from pathlib import Path

SAMPLE_RATE = 16_000
MODES = ("natural", "concise", "smart-fit", "stretch", "lip-sync")


class TimingError(ValueError):
    pass


@dataclass(frozen=True)
class Placement:
    tempo: float
    start: float
    end: float


def check_mode(mode):
    if mode not in MODES:
        raise TimingError("timing must be one of " + ", ".join(MODES))


def check_limits(fastest, slowest):
    if not math.isfinite(fastest) or not 1.0 <= fastest <= 4.0:
        raise TimingError("fastest must be between 1 and 4")
    if not math.isfinite(slowest) or not 0.25 <= slowest <= 1.0:
        raise TimingError("slowest must be between 0.25 and 1")


def plan(mode, start, slot_end, next_start, duration, fastest=1.25, slowest=0.9, video=True):
    check_mode(mode)
    check_limits(fastest, slowest)
    slot = max(slot_end - start, 0.0)
    room = max(next_start - start, slot)
    if duration <= 0 or slot <= 0:
        return Placement(1.0, start, start + max(duration, 0.0))
    if mode == "natural":
        return Placement(1.0, start, start + duration)
    if mode == "concise":
        return Placement(1.0, start, start + duration)
    if mode == "smart-fit":
        tempo = min(max(duration / slot, slowest), fastest)
        return Placement(tempo, start, start + duration / tempo)
    if mode == "stretch":
        if duration <= room:
            return Placement(1.0, start, start + duration)
        tempo = duration / room
        return Placement(tempo, start, start + room)
    if not video:
        raise TimingError("lip sync needs a video file with a picture")
    tempo = duration / slot
    return Placement(tempo, start, slot_end)


def atempo_chain(tempo):
    if not math.isfinite(tempo) or tempo <= 0:
        raise TimingError("speed must be a positive number")
    filters = []
    while tempo < 0.5:
        filters.append(0.5)
        tempo /= 0.5
    while tempo > 2.0:
        filters.append(2.0)
        tempo /= 2.0
    filters.append(tempo)
    return ",".join(f"atempo={value:.6f}" for value in filters)


def stretch_wav(ffmpeg, source, target, tempo):
    if abs(tempo - 1.0) < 0.01:
        Path(target).write_bytes(Path(source).read_bytes())
        return
    result = subprocess.run(
        [
            ffmpeg,
            "-nostdin",
            "-y",
            "-v",
            "error",
            "-i",
            str(source),
            "-filter:a",
            atempo_chain(tempo),
            "-ac",
            "1",
            "-ar",
            str(SAMPLE_RATE),
            "-c:a",
            "pcm_s16le",
            str(target),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise TimingError(f"ffmpeg could not change the line speed: {result.stderr.strip()}")
