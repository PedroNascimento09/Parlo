from dataclasses import dataclass
from pathlib import Path

SAMPLE_RATE = 16_000


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


