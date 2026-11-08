import json
import math
import shutil
import subprocess
from pathlib import Path

from dub_model import SAMPLE_RATE, Cue


def require(tool: str) -> str:
    path = shutil.which(tool)
    if path is None:
        raise SystemExit(f"{tool} is not installed; install ffmpeg and add it to PATH")
    return path


def load_cues(path: Path) -> list[Cue]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise SystemExit(f"cannot read imported subtitles: {error}") from error
    except json.JSONDecodeError as error:
        raise SystemExit(f"the imported subtitles are not valid JSON: {error}") from error
    items = document.get("cues") if isinstance(document, dict) else None
    if not isinstance(items, list) or not items:
        raise SystemExit("the imported subtitles have no cues")
    cues: list[Cue] = []
    for number, item in enumerate(items, start=1):
        if not isinstance(item, dict):
            raise SystemExit(f"cue {number} is not an object")
        identifier = item.get("id")
        start = item.get("start_ms")
        end = item.get("end_ms")
        source = item.get("source")
        if not isinstance(identifier, str) or not identifier:
            raise SystemExit(f"cue {number} has no id")
        if (
            not isinstance(start, (int, float))
            or isinstance(start, bool)
            or not math.isfinite(start)
            or start < 0
        ):
            raise SystemExit(f"cue {number} has no valid start")
        if (
            not isinstance(end, (int, float))
            or isinstance(end, bool)
            or not math.isfinite(end)
            or end <= start
        ):
            raise SystemExit(f"cue {number} has no valid end")
        if not isinstance(source, str) or not source.strip():
            raise SystemExit(f"cue {number} has no text")
        voice = item.get("voice")
        if voice is not None and (not isinstance(voice, str) or not voice.strip()):
            raise SystemExit(f"cue {number} has an invalid voice")
        pace = item.get("pace", 1.0)
        if (
            not isinstance(pace, (int, float))
            or isinstance(pace, bool)
            or not math.isfinite(pace)
            or not 0.8 <= pace <= 1.25
        ):
            raise SystemExit(f"cue {number} has an invalid pace")
        cues.append(Cue(identifier, start / 1000.0, end / 1000.0, source, voice, float(pace)))
    cues.sort(key=lambda cue: cue.start)
    identifiers = [cue.id for cue in cues]
    duplicate = next((name for name in identifiers if identifiers.count(name) > 1), None)
    if duplicate is not None:
        raise SystemExit(f"the imported subtitles repeat the cue id {duplicate}")
    return cues


def extract_audio(ffmpeg: str, source: Path, target: Path) -> None:
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
        raise SystemExit(f"ffmpeg could not read {source.name}: {result.stderr.strip()}")


def has_video(ffprobe: str, source: Path) -> bool:
    result = subprocess.run(
        [
            ffprobe,
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "csv=p=0",
            str(source),
        ],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0 and "video" in result.stdout
