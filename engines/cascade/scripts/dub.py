import argparse
import contextlib
import hashlib
import json
import math
import shutil
import subprocess
import sys
import tempfile
import threading
import wave

import exports
import separation
import timing
from dataclasses import dataclass, replace
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import CascadeEngine
from parlo_cascade.engine import (
    MODEL_DIR,
    SAMPLE_RATE,
    VOICES,
    float_to_pcm16,
    pcm16_to_float,
)
from parlo_cascade.tts import VoiceUnavailable

FRAME_SECONDS = 0.02
MIN_SEGMENT_SECONDS = 0.8
MAX_SEGMENT_SECONDS = 15.0
PAUSE_SECONDS = 0.28
VOICED_RMS = 0.008
BACKGROUND_GAIN = 0.55
DUCKING_FLOOR = 0.18
VIDEO_SUFFIXES = {".mp4", ".mkv", ".mov", ".webm", ".avi", ".m4v"}


@dataclass(frozen=True)
class Segment:
    start: int
    end: int

    def seconds(self) -> float:
        return (self.end - self.start) / SAMPLE_RATE


@dataclass(frozen=True)
class Line:
    start: float
    end: float
    source: str
    translated: str
    id: str | None = None
    voice: str | None = None
    slot_end: float | None = None
    file: str | None = None
    issues: tuple[str, ...] = ()
    failed: bool = False
    failure: str | None = None
    skipped: bool = False
    cached: bool = False


@dataclass(frozen=True)
class Cue:
    id: str
    start: float
    end: float
    source: str
    voice: str | None = None
    pace: float = 1.0


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


def speak(engine: CascadeEngine, text: str, pace: float = 1.0) -> np.ndarray:
    if pace == 1.0:
        audio = engine._synthesize(text)
    else:
        audio = engine._synthesize(text, length_scale=1.0 / pace)
    return np.asarray(pcm16_to_float(audio), dtype=np.float32)


def warm_voice(engine: CascadeEngine) -> threading.Thread | None:
    load = getattr(engine, "_load_voice", None)
    voice = getattr(engine, "voice_path", None)
    if load is None or voice is None or not Path(voice).is_file():
        return None

    def boot() -> None:
        try:
            load().start()
        except VoiceUnavailable:
            pass

    thread = threading.Thread(target=boot, daemon=True)
    thread.start()
    return thread


def select_voice(
    engine: CascadeEngine, voice: str | None, default: Path | None
) -> Path | None:
    if not voice:
        if default is None:
            return None
        path = default
    else:
        name = VOICES.get(voice.removeprefix("voice-"))
        if name is None and (MODEL_DIR / voice).is_file():
            name = voice
        if name is None and (MODEL_DIR / f"{voice}.onnx").is_file():
            name = f"{voice}.onnx"
        if name is None:
            raise ValueError(f"{voice} is not a voice this engine installs")
        path = MODEL_DIR / name
        if not path.is_file():
            raise ValueError(f"the voice {path.name} is not installed")
    if Path(engine.voice_path) != path:
        engine._release_voice()
        engine.voice_path = path
    return path


def cue_pieces(cues: list[Cue], length: int) -> list[tuple[Segment, Cue]]:
    pieces = []
    for cue in cues:
        start = max(int(round(cue.start * SAMPLE_RATE)), 0)
        end = min(int(round(cue.end * SAMPLE_RATE)), length)
        if end > start:
            pieces.append((Segment(start, end), cue))
    if not pieces:
        raise SystemExit("none of the imported cues fall inside the audio")
    return pieces


def build(
    engine: CascadeEngine,
    samples: np.ndarray,
    progress,
    cues: list[Cue] | None = None,
    mode: str = "natural",
    fastest: float = 1.25,
    slowest: float = 0.9,
    video: bool = True,
    ffmpeg: str | None = None,
    workdir: Path | None = None,
    lines_dir: Path | None = None,
) -> tuple[np.ndarray, list[Line]]:
    timing.check_mode(mode)
    timing.check_limits(fastest, slowest)
    if cues is None:
        segments = segment(samples)
        if not segments:
            raise SystemExit("no speech was found in the file")
        pieces = [(piece, None) for piece in segments]
    else:
        pieces = cue_pieces(cues, len(samples))

    engine_voice = getattr(engine, "voice_path", None)
    default_voice = Path(engine_voice) if engine_voice else None
    warming = warm_voice(engine)
    track = np.zeros(len(samples), dtype=np.float32)
    lines: list[Line] = []
    media = len(samples) / SAMPLE_RATE
    lines_dir = lines_dir or workdir
    for number, (piece, cue) in enumerate(pieces, start=1):
        voice = cue.voice if cue is not None else None
        source = cue.source if cue is not None else ""
        translated = ""
        start = piece.start / SAMPLE_RATE
        next_start = (
            pieces[number][0].start / SAMPLE_RATE
            if number < len(pieces)
            else len(samples) / SAMPLE_RATE
        )
        identifier = cue.id if cue is not None else f"auto-{number}"
        try:
            line, audio = render_line(
                engine,
                samples,
                piece,
                cue,
                number,
                next_start,
                source,
                mode,
                fastest,
                slowest,
                video,
                ffmpeg,
                workdir,
                lines_dir,
            )
        except Exception as error:
            progress(f"[{number}/{len(pieces)}] failed: {error}")
            lines.append(
                inspect(
                    Line(
                        start=start,
                        end=start,
                        source=source,
                        translated=translated,
                        id=identifier,
                        voice=voice,
                        slot_end=piece.end / SAMPLE_RATE,
                        failure=str(error),
                        failed=True,
                    ),
                    media,
                )
            )
            continue
        end = min(piece.start + len(audio), len(track))
        if end > piece.start:
            track[piece.start : end] += audio[: end - piece.start]
        line = inspect(line, media)
        lines.append(line)
        progress(
            f"[{number}/{len(pieces)}] {line.start:7.1f}s  {line.translated}"
        )
    if warming is not None:
        warming.join()
    return np.clip(track, -1.0, 1.0), lines


def line_file(identifier: str, voice: str | None = None, pace: float = 1.0) -> str:
    safe = "".join(
        character if character.isalnum() or character in "-_" else "_"
        for character in identifier
    )
    stem = safe[:80] or "line"
    if voice is None and pace == 1.0:
        return f"{stem}.wav"
    key = hashlib.sha1(f"{voice or ''}|{pace:.4f}".encode("utf-8")).hexdigest()[:10]
    return f"{stem}-{key}.wav"


def render_line(
    engine: CascadeEngine,
    samples: np.ndarray,
    piece: Segment,
    cue: Cue | None,
    number: int,
    next_start: float,
    source: str,
    mode: str,
    fastest: float,
    slowest: float,
    video: bool,
    ffmpeg: str | None,
    workdir: Path | None,
    lines_dir: Path | None,
    spoken: str | None = None,
) -> tuple[Line, np.ndarray]:
    voice = cue.voice if cue is not None else None
    engine_voice = getattr(engine, "voice_path", None)
    select_voice(engine, voice, Path(engine_voice) if engine_voice else None)
    if spoken is None:
        if not source:
            source = engine._transcribe(samples[piece.start : piece.end])
        if not source:
            raise ValueError("nothing was recognised")
        translated = engine._translate(source)
    else:
        source = spoken
        translated = spoken
    if not translated:
        raise ValueError("nothing was translated")
    audio = speak(engine, translated, cue.pace if cue is not None else 1.0)
    if mode == "concise":
        words = translated.split()
        while len(words) > 1 and len(audio) / SAMPLE_RATE > piece.seconds():
            words.pop()
            translated = " ".join(words)
            audio = speak(engine, translated, cue.pace if cue is not None else 1.0)
    start = piece.start / SAMPLE_RATE
    placed = timing.plan(
        mode,
        start,
        piece.end / SAMPLE_RATE,
        next_start,
        len(audio) / SAMPLE_RATE,
        fastest,
        slowest,
        video,
    )
    if abs(placed.tempo - 1.0) >= 0.01:
        if ffmpeg is None or workdir is None:
            raise ValueError("changing the line speed needs ffmpeg and a work directory")
        raw = workdir / f"line-{number}.wav"
        fitted = workdir / f"line-{number}-fit.wav"
        write_wav(raw, audio)
        timing.stretch_wav(ffmpeg, raw, fitted, placed.tempo)
        audio = read_wav(fitted)
    identifier = cue.id if cue is not None else f"auto-{number}"
    name = None
    if lines_dir is not None:
        name = line_file(identifier, voice, cue.pace if cue is not None else 1.0)
        write_wav(lines_dir / name, audio)
    return (
        Line(
            start=start,
            end=(piece.start + len(audio)) / SAMPLE_RATE,
            source=source,
            translated=translated,
            id=identifier,
            voice=voice,
            slot_end=piece.end / SAMPLE_RATE,
            file=name,
        ),
        audio,
    )


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


def timestamp(seconds: float) -> str:
    milliseconds = max(int(round(seconds * 1000)), 0)
    hours, rest = divmod(milliseconds, 3_600_000)
    minutes, rest = divmod(rest, 60_000)
    whole, millis = divmod(rest, 1000)
    return f"{hours:02d}:{minutes:02d}:{whole:02d},{millis:03d}"


def write_srt(path: Path, lines: list[Line]) -> None:
    blocks = []
    number = 0
    for line in lines:
        if line.failed or line.skipped:
            continue
        number += 1
        blocks.append(
            f"{number}\n{timestamp(line.start)} --> {timestamp(line.end)}\n{line.translated}\n"
        )
    path.write_text("\n".join(blocks), encoding="utf-8")


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


def line_document(line: Line) -> dict:
    return {
        "start": round(line.start, 2),
        "end": round(line.end, 2),
        "source": line.source,
        "translated": line.translated,
        "id": line.id,
        "voice": line.voice,
        "slot_end": round(line.slot_end, 2) if line.slot_end is not None else None,
        "file": line.file,
        "issues": list(line.issues),
        "failed": line.failed,
        "failure": line.failure,
        "skipped": line.skipped,
        "cached": line.cached,
    }


def line_from_document(item: dict) -> Line:
    identifier = item.get("id")
    if not isinstance(identifier, str) or not identifier:
        raise SystemExit("the previous dub report has a line without an id")
    return Line(
        start=float(item.get("start", 0.0)),
        end=float(item.get("end", 0.0)),
        source=str(item.get("source", "")),
        translated=str(item.get("translated", "")),
        id=identifier,
        voice=item.get("voice"),
        slot_end=item.get("slot_end"),
        file=item.get("file"),
        failed=bool(item.get("failed")),
        failure=item.get("failure"),
        skipped=bool(item.get("skipped")),
        cached=bool(item.get("cached")),
    )


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


def retry_line(
    lines: list[Line],
    index: int,
    cues: list[Cue],
    samples: np.ndarray,
    args,
    engine: CascadeEngine,
    ffmpeg: str,
    video: bool,
    work: Path,
    lines_dir: Path,
) -> Line:
    pieces = cue_pieces(cues, len(samples))
    position = next((number for number, (_, cue) in enumerate(pieces) if cue.id == args.retry), None)
    if position is None:
        raise SystemExit(f"line {args.retry} is not inside the audio")
    piece, cue = pieces[position]
    next_start = (
        pieces[position + 1][0].start / SAMPLE_RATE
        if position + 1 < len(pieces)
        else len(samples) / SAMPLE_RATE
    )
    previous = lines[index]
    try:
        line, _ = render_line(
            engine,
            samples,
            piece,
            cue,
            index + 1,
            next_start,
            cue.source,
            args.timing,
            args.fastest,
            args.slowest,
            video,
            ffmpeg,
            work,
            lines_dir,
        )
        return inspect(line, len(samples) / SAMPLE_RATE)
    except Exception as error:
        if previous.file:
            (lines_dir / previous.file).unlink(missing_ok=True)
        return inspect(
            Line(
                start=previous.start,
                end=previous.start,
                source=previous.source,
                translated=previous.translated,
                id=previous.id,
                voice=previous.voice,
                slot_end=piece.end / SAMPLE_RATE,
                failed=True,
                failure=str(error),
            ),
            len(samples) / SAMPLE_RATE,
        )


def render_story(args) -> int:
    try:
        document = json.loads(args.render_lines.read_text(encoding="utf-8"))
    except OSError as error:
        raise SystemExit(f"cannot read the story request: {error}") from error
    except json.JSONDecodeError as error:
        raise SystemExit(f"the story request is not valid JSON: {error}") from error
    if not isinstance(document, dict):
        raise SystemExit("the story request must be an object")
    language = document.get("language")
    items = document.get("lines")
    if not isinstance(language, str) or not language.strip():
        raise SystemExit("the story request has no language")
    if not isinstance(items, list) or not items:
        raise SystemExit("the story request has no lines")
    out = args.out or Path("story.wav")
    srt = args.srt or out.with_suffix(".srt")
    lines_dir = args.lines_dir or out.parent / "lines"
    out.parent.mkdir(parents=True, exist_ok=True)
    lines_dir.mkdir(parents=True, exist_ok=True)
    engine = CascadeEngine(source=language, target=language)
    lines: list[Line] = []
    chunks: list[np.ndarray] = []
    cursor = 0.0
    rendered = 0
    cached = 0
    failed = 0
    for number, item in enumerate(items, start=1):
        if not isinstance(item, dict):
            raise SystemExit(f"story line {number} is not an object")
        identifier = item.get("id")
        text = item.get("text")
        voice = item.get("voice")
        pace = item.get("pace", 1.0)
        if not isinstance(identifier, str) or not identifier:
            raise SystemExit(f"story line {number} has no id")
        if not isinstance(text, str) or not text.strip():
            raise SystemExit(f"story line {number} has no text")
        if voice is not None and (not isinstance(voice, str) or not voice.strip()):
            raise SystemExit(f"story line {number} has an invalid voice")
        if (
            not isinstance(pace, (int, float))
            or isinstance(pace, bool)
            or not math.isfinite(pace)
            or not 0.8 <= pace <= 1.25
        ):
            raise SystemExit(f"story line {number} has an invalid pace")
        name = line_file(identifier, voice, float(pace))
        audio = None
        if (lines_dir / name).is_file():
            try:
                audio = read_wav(lines_dir / name)
            except (SystemExit, OSError, EOFError):
                (lines_dir / name).unlink(missing_ok=True)
        if audio is None:
            start = int(cursor * SAMPLE_RATE)
            piece = Segment(start, start + SAMPLE_RATE)
            try:
                line, audio = render_line(
                    engine,
                    np.zeros(1, dtype=np.float32),
                    piece,
                    Cue(identifier, cursor, cursor + 1.0, text, voice, float(pace)),
                    number,
                    cursor + 1.0,
                    text,
                    "natural",
                    1.25,
                    0.9,
                    False,
                    None,
                    None,
                    lines_dir,
                    spoken=text.strip(),
                )
                rendered += 1
            except Exception as error:
                failed += 1
                lines.append(
                    inspect(
                        Line(
                            start=cursor,
                            end=cursor,
                            source=text,
                            translated=text,
                            id=identifier,
                            voice=voice,
                            slot_end=cursor,
                            failed=True,
                            failure=str(error),
                        ),
                        cursor,
                    )
                )
                print(f"[{number}/{len(items)}] failed: {error}")
                continue
            line = replace(
                line,
                start=cursor,
                end=cursor + len(audio) / SAMPLE_RATE,
                slot_end=cursor + len(audio) / SAMPLE_RATE,
                cached=False,
            )
        else:
            cached += 1
            line = Line(
                start=cursor,
                end=cursor + len(audio) / SAMPLE_RATE,
                source=text,
                translated=text,
                id=identifier,
                voice=voice,
                slot_end=cursor + len(audio) / SAMPLE_RATE,
                file=name,
                cached=True,
            )
        lines.append(line)
        chunks.append(audio)
        cursor = line.end
        if number < len(items):
            chunks.append(np.zeros(int(PAUSE_SECONDS * SAMPLE_RATE), dtype=np.float32))
            cursor += PAUSE_SECONDS
        print(f"[{number}/{len(items)}] {line.start:7.1f}s  {text}")
    track = np.concatenate(chunks) if chunks else np.zeros(0, dtype=np.float32)
    write_wav(out, np.clip(track, -1.0, 1.0))
    write_srt(srt, lines)
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(
            json.dumps(
                {
                    "language": language,
                    "out": str(out),
                    "srt": str(srt),
                    "seconds": round(len(track) / SAMPLE_RATE, 2),
                    "rendered": rendered,
                    "cached": cached,
                    "failed": failed,
                    "lines": [line_document(line) for line in lines],
                },
                ensure_ascii=False,
                indent=2,
            ),
            encoding="utf-8",
        )
    print(f"wrote {out}")
    print(f"  rendered {rendered}  cached {cached}  failed {failed}")
    return 1 if failed else 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Dub a media file offline")
    parser.add_argument("source", type=Path, nargs="?")
    parser.add_argument("--to", default="en")
    parser.add_argument("--from", dest="source_language", default="pt")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--srt", type=Path)
    parser.add_argument("--keep-background", action="store_true")
    parser.add_argument("--cues", type=Path)
    parser.add_argument("--timing", default="natural")
    parser.add_argument("--fastest", type=float, default=1.25)
    parser.add_argument("--slowest", type=float, default=0.9)
    parser.add_argument("--mp3-bitrate", type=int, default=192)
    parser.add_argument("--json", type=Path)
    parser.add_argument("--track", type=Path)
    parser.add_argument("--lines-dir", type=Path)
    parser.add_argument("--previous", type=Path)
    parser.add_argument("--retry")
    parser.add_argument("--skip")
    parser.add_argument("--work", type=Path)
    parser.add_argument("--render-lines", type=Path)
    args = parser.parse_args()

    if args.render_lines:
        return render_story(args)
    if args.source is None:
        raise SystemExit("a source file or --render-lines is required")

    if args.retry and args.skip:
        raise SystemExit("retry and skip cannot be used together")
    resume = bool(args.retry or args.skip)
    if resume and (args.previous is None or args.lines_dir is None or args.track is None):
        raise SystemExit("retrying or skipping a line needs --previous, --lines-dir and --track")
    if resume and args.json is None:
        raise SystemExit("retrying or skipping a line needs --json to update the report")
    if not args.source.is_file():
        raise SystemExit(f"{args.source} does not exist")
    if args.source_language == args.to:
        raise SystemExit("the source and target language must differ")
    try:
        timing.check_mode(args.timing)
        timing.check_limits(args.fastest, args.slowest)
    except timing.TimingError as error:
        raise SystemExit(str(error)) from error
    if not 64 <= args.mp3_bitrate <= 320:
        raise SystemExit("the MP3 bitrate must be between 64 and 320")

    cues = load_cues(args.cues) if args.cues else None
    if resume and cues is None:
        raise SystemExit("retrying or skipping a line needs imported subtitles")

    ffmpeg = require("ffmpeg")
    ffprobe = require("ffprobe")
    video = has_video(ffprobe, args.source)
    video = video and args.source.suffix.lower() in VIDEO_SUFFIXES
    if args.timing == "lip-sync" and not video:
        raise SystemExit("lip sync needs a video file with a picture")

    stem = args.source.with_suffix("")
    suffix = args.source.suffix or ".wav"
    out = args.out or stem.with_name(f"{stem.name}.{args.to}{suffix}")
    srt = args.srt or out.with_suffix(".srt")

    if args.work:
        workspace = contextlib.nullcontext(args.work.resolve())
        args.work.mkdir(parents=True, exist_ok=True)
    else:
        workspace = tempfile.TemporaryDirectory(prefix="parlo-dub-")
    with workspace as directory:
        work = Path(directory)
        extracted = work / "source.wav"
        extract_audio(ffmpeg, args.source, extracted)
        samples = read_wav(extracted)
        if samples.size == 0:
            raise SystemExit("the file has no audio to dub")
        lines_dir = args.lines_dir or work / "lines"
        lines_dir.mkdir(parents=True, exist_ok=True)
        track_path = args.track or work / "dub.wav"
        media = len(samples) / SAMPLE_RATE
        print(
            f"dubbing {args.source.name} ({media:.1f} s, "
            f"{args.source_language} to {args.to})"
        )
        if resume:
            try:
                previous = json.loads(args.previous.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as error:
                raise SystemExit(f"cannot read the previous dub report: {error}") from error
            lines = [line_from_document(item) for item in previous.get("lines", [])]
            identifier = args.retry or args.skip
            matches = [number for number, line in enumerate(lines) if line.id == identifier]
            if len(matches) != 1:
                raise SystemExit(f"line {identifier} is not part of this render")
            index = matches[0]
            engine = CascadeEngine(source=args.source_language, target=args.to)
            if args.retry:
                lines[index] = retry_line(
                    lines, index, cues, samples, args, engine, ffmpeg, video, work, lines_dir
                )
            else:
                previous_line = lines[index]
                if previous_line.file:
                    (lines_dir / previous_line.file).unlink(missing_ok=True)
                lines[index] = inspect(
                    replace(
                        previous_line,
                        skipped=True,
                        failed=False,
                        failure=None,
                        issues=(),
                    ),
                    media,
                )
            track = rebuild(lines, len(samples), lines_dir)
        else:
            engine = CascadeEngine(source=args.source_language, target=args.to)
            track, lines = build(
                engine,
                samples,
                print,
                cues,
                mode=args.timing,
                fastest=args.fastest,
                slowest=args.slowest,
                video=video,
                ffmpeg=ffmpeg,
                workdir=work,
                lines_dir=lines_dir,
            )
        if not lines:
            raise SystemExit("nothing was translated")
        active = [line for line in lines if not line.failed and not line.skipped]
        voice = track.copy()
        background = None
        separation_mode = "off"
        separation_note = "Background was not requested."
        if args.keep_background:
            background, separation_mode, separation_note = separation.separate(
                ffmpeg, args.source, work / "background.wav", work
            )
            if background is not None:
                track = np.clip(track + read_wav(background) * BACKGROUND_GAIN, -1.0, 1.0)
            else:
                envelope = ducking(len(samples), active)
                track = np.clip(track + samples * BACKGROUND_GAIN * envelope, -1.0, 1.0)
        write_wav(track_path, track)
        mux(ffmpeg, args.source, track_path, out, video)
        write_srt(srt, lines)
        write_wav(work / "voice.wav", voice)
        write_wav(work / "original.wav", samples)
        write_wav(work / "mix.wav", track)
        written = exports.export_all(
            ffmpeg,
            args.source,
            out,
            srt,
            track_path,
            work / "voice.wav",
            work / "original.wav",
            work / "mix.wav",
            lines,
            video,
            args.mp3_bitrate,
            work,
            background,
        )

    if args.json:
        write_report(
            args.json,
            args,
            out,
            srt,
            samples,
            lines,
            written,
            separation_mode,
            separation_note,
        )
    print(f"wrote {out}")
    print(f"wrote {srt}")
    print(f"  lines        {len(active)}")
    print(f"  exports      {len(written)}")
    print(f"  separation   {separation_mode}")
    failed = sum(1 for line in lines if line.failed)
    if failed:
        print(f"  failed       {failed} (retry them from the app)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
