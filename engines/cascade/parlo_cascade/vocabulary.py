import json
import os
import re
from datetime import datetime
from pathlib import Path

from .models import data_root

KINDS = ("term", "abbreviation", "snippet")
FILE = "vocabulary.json"
LIMIT = 64
SNIPPET_LIMIT = 240


def path() -> Path:
    return data_root() / FILE


def load() -> dict:
    try:
        payload = json.loads(path().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return _empty()
    if not isinstance(payload, dict):
        return _empty()
    profile = _empty()
    for kind in KINDS:
        profile[_key(kind)] = _pairs(payload.get(_key(kind)))
    return profile


def _empty() -> dict:
    return {_key(kind): {} for kind in KINDS}


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
    limit = SNIPPET_LIMIT if kind == "snippet" else LIMIT
    profile[_key(kind)][_side(wrong)] = _side(right, limit)
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
    return {"term": "terms", "abbreviation": "abbreviations", "snippet": "snippets"}[kind]


def _side(value: str, limit: int = LIMIT) -> str:
    if not isinstance(value, str):
        raise ValueError("a word is written as text")
    text = " ".join(value.split())
    if not text:
        raise ValueError("a word cannot be empty")
    if len(text) > limit:
        raise ValueError(f"a word cannot be longer than {limit} characters")
    return text


def export_text(profile: dict | None = None) -> str:
    profile = profile if profile is not None else load()
    lines = ["kind\twhat it hears\twhat it writes"]
    for kind in KINDS:
        for wrong, right in sorted(profile.get(_key(kind), {}).items()):
            lines.append(f"{kind}\t{wrong}\t{right}")
    return "\n".join(lines) + "\n"


def import_text(text) -> dict:
    if not isinstance(text, str) or not text.strip():
        raise ValueError("an import needs at least one line")
    profile = load()
    added = 0
    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip() or line.startswith("#") or line.lower().startswith("kind\t"):
            continue
        parts = [part.strip() for part in line.split("\t")]
        if len(parts) != 3:
            raise ValueError(
                f"line {number} needs a kind, what it hears and what it writes, separated by tabs"
            )
        kind, wrong, right = parts
        limit = SNIPPET_LIMIT if kind == "snippet" else LIMIT
        profile[_key(kind)][_side(wrong)] = _side(right, limit)
        added += 1
    if not added:
        raise ValueError("an import needs at least one line")
    save(profile)
    return {**profile, "added": added}


def display_name() -> str:
    configured = os.environ.get("PARLO_CONFIG")
    target = Path(configured) if configured else data_root() / "config.json"
    try:
        payload = json.loads(target.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return ""
    name = payload.get("name") if isinstance(payload, dict) else None
    return name.strip() if isinstance(name, str) else ""


def _fill(right: str, name: str) -> str:
    if "{{" not in right:
        return right
    moment = datetime.now()
    return (
        right.replace("{{date}}", f"{moment.day} {moment.strftime('%B %Y')}")
        .replace("{{time}}", moment.strftime("%H:%M"))
        .replace("{{name}}", name)
    )


def apply(text: str, profile: dict | None = None) -> tuple[str, tuple[str, ...]]:
    if not text:
        return text, ()
    profile = profile if profile is not None else load()
    changes = []
    name = display_name() if profile.get("snippets") else ""
    for kind in KINDS:
        for wrong, right in profile.get(_key(kind), {}).items():
            pattern = re.compile(rf"\b{re.escape(wrong)}\b", re.IGNORECASE)
            if not pattern.search(text):
                continue
            filled = _fill(right, name) if kind == "snippet" else right
            text = pattern.sub(lambda match, value=filled: _match_case(match.group(0), value), text)
            changes.append(f"{kind}: {wrong} → {right}")
    return text, tuple(changes)


def _match_case(spoken: str, right: str) -> str:
    if len(spoken) > 3 and spoken.isupper():
        return right.upper()
    if spoken[:1].isupper():
        return right[:1].upper() + right[1:]
    return right
