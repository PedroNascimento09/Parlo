import subprocess
import wave
from pathlib import Path

import numpy as np

from dub_model import SAMPLE_RATE, Segment
from parlo_cascade.engine import float_to_pcm16, pcm16_to_float

FRAME_SECONDS = 0.02
PAUSE_SECONDS = 0.28
MIN_SEGMENT_SECONDS = 0.8
MAX_SEGMENT_SECONDS = 15.0
VOICED_RMS = 0.008


def read_wav(path: Path) -> np.ndarray:
    with wave.open(str(path), "rb") as wav:
        if wav.getsampwidth() != 2 or wav.getnchannels() != 1:
            raise SystemExit("ffmpeg produced audio that is not mono pcm16")
        if wav.getframerate() != SAMPLE_RATE:
            raise SystemExit(f"ffmpeg produced {wav.getframerate()} Hz audio, expected {SAMPLE_RATE}")
        return np.asarray(pcm16_to_float(wav.readframes(wav.getnframes())), dtype=np.float32)


def voiced_frames(samples: np.ndarray) -> np.ndarray:
    size = max(int(FRAME_SECONDS * SAMPLE_RATE), 1)
    usable = samples[: len(samples) - (len(samples) % size)]
    if usable.size == 0:
        return np.zeros(0, dtype=bool)
    frames = usable.reshape(-1, size)
    energy = np.sqrt(np.mean(frames * frames, axis=1))
    return energy >= VOICED_RMS


def split_long(samples: np.ndarray, start: int, end: int) -> list[Segment]:
    limit = int(MAX_SEGMENT_SECONDS * SAMPLE_RATE)
    pieces = []
    cursor = start
    while end - cursor > limit:
        window = samples[cursor + int(limit * 0.6) : cursor + limit]
        if window.size == 0:
            break
        size = max(int(FRAME_SECONDS * SAMPLE_RATE), 1)
        count = window.size // size
        if count == 0:
            break
        energy = np.sqrt(np.mean(window[: count * size].reshape(-1, size) ** 2, axis=1))
        offset = int(np.argmin(energy) * size + limit * 0.6)
        cut = cursor + offset
        pieces.append(Segment(cursor, cut))
        cursor = cut
    pieces.append(Segment(cursor, end))
    return pieces


def segment(samples: np.ndarray) -> list[Segment]:
    voiced = voiced_frames(samples)
    if voiced.size == 0:
        return []
    size = max(int(FRAME_SECONDS * SAMPLE_RATE), 1)
    pause = int(PAUSE_SECONDS / FRAME_SECONDS)
    minimum = int(MIN_SEGMENT_SECONDS * SAMPLE_RATE)

    runs = []
    index = 0
    while index < voiced.size:
        if not voiced[index]:
            index += 1
            continue
        begin = index
        silence = 0
        while index < voiced.size:
            if voiced[index]:
                silence = 0
            else:
                silence += 1
                if silence >= pause:
                    break
            index += 1
        end = index - silence
        runs.append((max(begin * size, 0), min(end * size, len(samples))))

    merged = []
    for start, end in runs:
        if merged and start - merged[-1][1] < minimum:
            merged[-1] = (merged[-1][0], end)
        else:
            merged.append((start, end))

    segments = []
    for start, end in merged:
        if end - start < minimum:
            continue
        segments.extend(split_long(samples, start, end))
    return segments


def write_wav(path: Path, samples: np.ndarray) -> None:
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(float_to_pcm16(samples))


def mux(ffmpeg: str, source: Path, track: Path, target: Path, video: bool) -> None:
    command = [ffmpeg, "-nostdin", "-y", "-v", "error", "-i", str(source), "-i", str(track)]
    if video:
        command += ["-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k"]
    else:
        command += ["-map", "1:a:0", "-c:a", "aac", "-b:a", "192k"]
    command += ["-shortest", str(target)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(f"ffmpeg could not write {target.name}: {result.stderr.strip()}")


