import argparse
import io
import json
import shutil
import subprocess
import sys
import tempfile
import wave
from dataclasses import dataclass
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import CascadeEngine
from parlo_cascade.engine import SAMPLE_RATE, float_to_pcm16, pcm16_to_float

FRAME_SECONDS = 0.02
MIN_SEGMENT_SECONDS = 0.8
MAX_SEGMENT_SECONDS = 15.0
PAUSE_SECONDS = 0.28
VOICED_RMS = 0.008
SLOT_GROWTH = 1.15
SLOT_MAX_GROWTH_SECONDS = 3.0
FASTEST_LENGTH_SCALE = 0.82
BACKGROUND_GAIN = 0.18
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


def require(tool: str) -> str:
    path = shutil.which(tool)
    if path is None:
        raise SystemExit(f"{tool} is not installed; install ffmpeg and add it to PATH")
    return path


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


def speak(engine: CascadeEngine, text: str, slot_seconds: float) -> np.ndarray:
    audio = engine._synthesize(text)
    samples = np.asarray(pcm16_to_float(audio), dtype=np.float32)
    if samples.size == 0:
        return np.zeros(0, dtype=np.float32)
    length = len(samples) / SAMPLE_RATE
    if length <= slot_seconds:
        return samples
    scale = max(FASTEST_LENGTH_SCALE, slot_seconds / length)
    from piper import PiperVoice, SynthesisConfig

    if engine._tts is None:
        engine._tts = PiperVoice.load(str(engine.voice_path))
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as wav:
        engine._tts.synthesize_wav(text, wav, syn_config=SynthesisConfig(length_scale=scale))
    buffer.seek(0)
    with wave.open(buffer, "rb") as wav:
        faster = np.asarray(pcm16_to_float(wav.readframes(wav.getnframes())), dtype=np.float32)
    return faster if faster.size > 0 else samples


def build(engine: CascadeEngine, samples: np.ndarray, progress) -> tuple[np.ndarray, list[Line]]:
    segments = segment(samples)
    if not segments:
        raise SystemExit("no speech was found in the file")

    track = np.zeros(len(samples), dtype=np.float32)
    lines: list[Line] = []
    for number, piece in enumerate(segments, start=1):
        source = engine._transcribe(samples[piece.start : piece.end])
        if not source:
            progress(f"[{number}/{len(segments)}] skipped, nothing recognised")
            continue
        translated = engine._translate(source)
        if not translated:
            progress(f"[{number}/{len(segments)}] skipped, nothing translated")
            continue
        following = segments[number].start if number < len(segments) else len(samples)
        gap = (following - piece.end) / SAMPLE_RATE
        slot = piece.seconds() * SLOT_GROWTH + min(gap, SLOT_MAX_GROWTH_SECONDS)
        audio = speak(engine, translated, slot)
        end = min(piece.start + len(audio), len(track))
        track[piece.start : end] += audio[: end - piece.start]
        lines.append(
            Line(
                start=piece.start / SAMPLE_RATE,
                end=(piece.start + len(audio)) / SAMPLE_RATE,
                source=source,
                translated=translated,
            )
        )
        progress(
            f"[{number}/{len(segments)}] {piece.start / SAMPLE_RATE:7.1f}s  {translated}"
        )
    return np.clip(track, -1.0, 1.0), lines


def timestamp(seconds: float) -> str:
    milliseconds = max(int(round(seconds * 1000)), 0)
    hours, rest = divmod(milliseconds, 3_600_000)
    minutes, rest = divmod(rest, 60_000)
    whole, millis = divmod(rest, 1000)
    return f"{hours:02d}:{minutes:02d}:{whole:02d},{millis:03d}"


def write_srt(path: Path, lines: list[Line]) -> None:
    blocks = []
    for number, line in enumerate(lines, start=1):
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


def main() -> int:
    parser = argparse.ArgumentParser(description="Dub a media file offline")
    parser.add_argument("source", type=Path)
    parser.add_argument("--to", default="en")
    parser.add_argument("--from", dest="source_language", default="pt")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--srt", type=Path)
    parser.add_argument("--keep-background", action="store_true")
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()

    if not args.source.is_file():
        raise SystemExit(f"{args.source} does not exist")
    if args.source_language == args.to:
        raise SystemExit("the source and target language must differ")

    ffmpeg = require("ffmpeg")
    ffprobe = require("ffprobe")
    video = has_video(ffprobe, args.source)
    video = video and args.source.suffix.lower() in VIDEO_SUFFIXES

    stem = args.source.with_suffix("")
    suffix = args.source.suffix or ".wav"
    out = args.out or stem.with_name(f"{stem.name}.{args.to}{suffix}")
    srt = args.srt or out.with_suffix(".srt")

    engine = CascadeEngine(source=args.source_language, target=args.to)
    with tempfile.TemporaryDirectory(prefix="parlo-dub-") as directory:
        work = Path(directory)
        extracted = work / "source.wav"
        extract_audio(ffmpeg, args.source, extracted)
        samples = read_wav(extracted)
        if samples.size == 0:
            raise SystemExit("the file has no audio to dub")
        print(
            f"dubbing {args.source.name} ({len(samples) / SAMPLE_RATE:.1f} s, "
            f"{args.source_language} to {args.to})"
        )
        track, lines = build(engine, samples, print)
        if not lines:
            raise SystemExit("nothing was translated")
        if args.keep_background:
            track = np.clip(track + samples * BACKGROUND_GAIN, -1.0, 1.0)
        track_path = work / "dub.wav"
        write_wav(track_path, track)
        mux(ffmpeg, args.source, track_path, out, video)

    write_srt(srt, lines)
    if args.json:
        args.json.write_text(
            json.dumps(
                {
                    "source": str(args.source),
                    "out": str(out),
                    "srt": str(srt),
                    "from": args.source_language,
                    "to": args.to,
                    "seconds": round(len(samples) / SAMPLE_RATE, 2),
                    "lines": [
                        {
                            "start": round(line.start, 2),
                            "end": round(line.end, 2),
                            "source": line.source,
                            "translated": line.translated,
                        }
                        for line in lines
                    ],
                },
                ensure_ascii=False,
                indent=2,
            ),
            encoding="utf-8",
        )
    print(f"wrote {out}")
    print(f"wrote {srt}")
    print(f"  lines        {len(lines)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
