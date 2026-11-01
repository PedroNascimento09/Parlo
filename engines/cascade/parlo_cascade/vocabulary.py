import json
import re
from pathlib import Path

from .models import data_root

KINDS = ("term", "abbreviation")
FILE = "vocabulary.json"
LIMIT = 64


def path() -> Path:
    return data_root() / FILE


def load() -> dict:
    try:
        payload = json.loads(path().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {"terms": {}, "abbreviations": {}}
    if not isinstance(payload, dict):
        return {"terms": {}, "abbreviations": {}}
    return {
        "terms": _pairs(payload.get("terms")),
        "abbreviations": _pairs(payload.get("abbreviations")),
    }


def _pairs(value) -> dict:
    if not isinstance(value, dict):
        return {}
    return {
        wrong.strip(): right.strip()
        for wrong, right in value.items()
        if isinstance(wrong, str)
        and isinstance(right, str)
        and wrong.strip()
        and right.strip()
    }


def save(profile: dict) -> None:
    target = path()
    target.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(profile, ensure_ascii=False, indent=2, sort_keys=True)
    target.write_text(text + "\n", encoding="utf-8")


def add(kind: str, wrong: str, right: str) -> dict:
    profile = load()
    profile[_key(kind)][_side(wrong)] = _side(right)
    save(profile)
    return profile


def discard(kind: str, wrong: str) -> dict:
    profile = load()
    profile[_key(kind)].pop(_side(wrong), None)
    save(profile)
    return profile


def _key(kind: str) -> str:
    if not isinstance(kind, str) or kind not in KINDS:
        raise ValueError(f"unknown kind {kind}, expected one of {KINDS}")
    return "terms" if kind == "term" else "abbreviations"


def _side(value: str) -> str:
    if not isinstance(value, str):
        raise ValueError("a word is written as text")
    text = " ".join(value.split())
    if not text:
        raise ValueError("a word cannot be empty")
    if len(text) > LIMIT:
        raise ValueError(f"a word cannot be longer than {LIMIT} characters")
    return text


def apply(text: str, profile: dict | None = None) -> tuple[str, tuple[str, ...]]:
    if not text:
        return text, ()
    profile = profile if profile is not None else load()
    changes = []
    for kind in KINDS:
        for wrong, right in profile[_key(kind)].items():
            pattern = re.compile(rf"\b{re.escape(wrong)}\b", re.IGNORECASE)
            if not pattern.search(text):
                continue
            text = pattern.sub(lambda match: _match_case(match.group(0), right), text)
            changes.append(f"{kind}: {wrong} → {right}")
    return text, tuple(changes)


def _match_case(spoken: str, right: str) -> str:
    if len(spoken) > 3 and spoken.isupper():
        return right.upper()
    if spoken[:1].isupper():
        return right[:1].upper() + right[1:]
    return right
