import subprocess
import wave
from pathlib import Path

import numpy as np

SAMPLE_RATE = 16_000
MIN_SIDE_RMS = 0.004


def write_wav(path: Path, samples: np.ndarray) -> None:
    pcm = (np.clip(samples, -1.0, 1.0) * 32767).astype("<i2")
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(SAMPLE_RATE)
        handle.writeframes(pcm.tobytes())


def read_stereo(path: Path) -> np.ndarray | None:
    with wave.open(str(path), "rb") as handle:
        if handle.getnchannels() != 2 or handle.getsampwidth() != 2:
            return None
        raw = handle.readframes(handle.getnframes())
    return np.frombuffer(raw, dtype="<i2").astype(np.float32).reshape(-1, 2) / 32768.0


def center_side(stereo: np.ndarray) -> np.ndarray:
    return np.clip((stereo[:, 0] - stereo[:, 1]) * 0.5, -1.0, 1.0)


def separate(ffmpeg: str, source: Path, target: Path, work: Path) -> tuple[Path | None, str, str | None]:
    if not source.is_file():
        return None, "ducking", "the source file is missing, so the background was ducked"
    stereo_path = work / "source-stereo.wav"
    try:
        result = subprocess.run(
            [
                ffmpeg,
                "-nostdin",
                "-y",
                "-v",
                "error",
                "-i",
                str(source),
                "-vn",
                "-ac",
                "2",
                "-ar",
                str(SAMPLE_RATE),
                "-c:a",
                "pcm_s16le",
                str(stereo_path),
            ],
            capture_output=True,
            text=True,
        )
    except OSError as error:
        return None, "ducking", f"the separator could not start: {error}"
    if result.returncode != 0:
        return None, "ducking", "the separator could not read this file, so the background was ducked"
    try:
        stereo = read_stereo(stereo_path)
    except (OSError, wave.Error):
        return None, "ducking", "the separator could not inspect this file, so the background was ducked"
    if stereo is None or len(stereo) == 0:
        return None, "ducking", "the source is mono, so the original speech could not be separated"
    side = center_side(stereo)
    mid = (stereo[:, 0] + stereo[:, 1]) * 0.5
    side_rms = float(np.sqrt(np.mean(side * side)))
    mid_rms = float(np.sqrt(np.mean(mid * mid)))
    if side_rms < max(MIN_SIDE_RMS, mid_rms * 0.05):
        return (
            None,
            "ducking",
            "the stereo channels carry the same signal, so separation would remove the music too",
        )
    write_wav(target, np.clip(side * 2.0, -1.0, 1.0))
    return target, "center", "Center speech was removed; the stereo music is kept."
