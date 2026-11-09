import re

from .corrections import CLARIFICATION_MARKERS, FILLERS, REPLACEMENT_MARKERS

TOKEN = re.compile(r"[^\W\d_]+")


def _phrases(code: str) -> tuple[str, ...]:
    return (
        *REPLACEMENT_MARKERS.get(code, ()),
        *CLARIFICATION_MARKERS.get(code, ()),
        *FILLERS.get(code, ()),
    )


def guess(text: str, fallback: str) -> str:
    padded = f" {' '.join(TOKEN.findall(text.lower()))} "
    scores = {
        code: sum(padded.count(f" {phrase} ") for phrase in _phrases(code))
        for code in FILLERS
    }
    best = max(scores.values())
    if best == 0:
        return fallback
    leaders = [code for code, hits in scores.items() if hits == best]
    return leaders[0] if len(leaders) == 1 else fallback
