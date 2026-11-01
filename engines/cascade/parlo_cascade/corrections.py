import re
from dataclasses import dataclass

CLEAN_MODES = ("original", "light", "intelligent")

REPLACEMENT_MARKERS = {
    "pt": (
        "desculpa, queria dizer",
        "desculpa queria dizer",
        "queria dizer",
        "não, espera",
        "não espera",
        "não, melhor",
        "não melhor",
        "corrigindo",
        "ou melhor",
        "melhor dizendo",
        "afinal",
    ),
    "en": (
        "sorry, i meant",
        "sorry i meant",
        "i meant",
        "no, wait",
        "no wait",
        "no, better",
        "no better",
        "correction",
        "or rather",
        "rather",
        "actually",
    ),
}

CLARIFICATION_MARKERS = {
    "pt": ("quer dizer", "isto é", "aliás"),
    "en": ("i mean", "that is"),
}

FILLERS = {
    "pt": ("hmm", "hum", "ah", "eh", "ãh", "ahn", "é pá"),
    "en": ("hmm", "hum", "uh", "um", "er"),
}

STOPWORDS = {
    "o", "a", "os", "as", "um", "uma", "uns", "umas", "de", "do", "da", "dos", "das",
    "para", "por", "com", "em", "no", "na", "nos", "nas", "e", "ou", "que", "se",
    "me", "te", "lhe", "nos", "eu", "tu", "ele", "ela", "nós", "eles", "elas",
    "the", "a", "an", "of", "to", "for", "with", "in", "on", "and", "or", "that",
    "i", "you", "he", "she", "we", "they", "it", "is", "are", "was", "were",
}

CORRECTION_OVERLAP = 0.2
TERMINAL = (".", "!", "?")
CONNECTORS = {
    "a", "ao", "à", "às", "aos", "o", "os", "as", "um", "uma",
    "de", "do", "da", "dos", "das", "em", "no", "na", "nos", "nas",
    "para", "por", "com",
}

SENTENCE = re.compile(r"[^.!?…]+[.!?…]*")
TOKEN = re.compile(r"[\wÀ-ÿ'-]+|[^\wÀ-ÿ\s]+")


@dataclass(frozen=True)
class CleanResult:
    original: str
    cleaned: str
    changes: tuple[str, ...]

    def as_dict(self) -> dict:
        return {
            "original": self.original,
            "cleaned": self.cleaned,
            "changes": list(self.changes),
        }


def clean(text: str, mode: str = "light", language: str = "pt") -> CleanResult:
    if mode not in CLEAN_MODES:
        raise ValueError(f"unknown cleanup mode {mode}, expected {CLEAN_MODES}")
    original = text
    if mode == "original" or not text.strip():
        return CleanResult(original, text.strip(), ())

    changes = []
    working = text.strip()
    working, dropped = _drop_false_starts(working)
    changes.extend(dropped)
    for _ in range(4):
        working, dropped = _merge_corrections(working, language)
        changes.extend(dropped)
        if not dropped:
            break
    working, dropped = _drop_fillers(working, language)
    changes.extend(dropped)
    working, dropped = _collapse_repeats(working)
    changes.extend(dropped)
    if mode == "intelligent":
        working, dropped = _tidy(working)
        changes.extend(dropped)
    return CleanResult(original, working, tuple(changes))


def _content_words(text: str) -> set[str]:
    words = {
        token.lower()
        for token in TOKEN.findall(text)
        if token[:1].isalnum() and token.lower() not in STOPWORDS and len(token) > 1
    }
    return words


def _overlap(left: str, right: str) -> float:
    first = _content_words(left)
    second = _content_words(right)
    if not first or not second:
        return 0.0
    return len(first & second) / max(len(first), len(second))


def _markers(language: str) -> tuple[tuple[str, ...], tuple[str, ...]]:
    key = "pt" if language.lower().startswith("pt") else "en"
    return REPLACEMENT_MARKERS[key], CLARIFICATION_MARKERS[key]


def _find_marker(text: str, phrases: tuple[str, ...]) -> tuple[int, str] | None:
    lowered = text.lower()
    best = None
    for phrase in phrases:
        index = lowered.find(phrase)
        if index < 0:
            continue
        if best is None or index < best[0] or (index == best[0] and len(phrase) > len(best[1])):
            best = (index, phrase)
    return best


def _is_fragment(text: str) -> bool:
    return len(TOKEN.findall(text)) <= 4


def _is_correction(before: str, after: str, explicit: bool) -> bool:
    if not before:
        return True
    overlap = _overlap(before, after)
    if overlap < CORRECTION_OVERLAP:
        return explicit and _is_fragment(after)
    if overlap >= 0.5:
        return True
    return len(_content_words(after)) <= len(_content_words(before)) or explicit


def _merge_corrections(text: str, language: str) -> tuple[str, list[str]]:
    replacements, clarifications = _markers(language)
    sentences = [part.strip() for part in SENTENCE.findall(text) if part.strip()]
    if not sentences:
        return text, []
    changes = []
    merged: list[str] = []
    for sentence in sentences:
        marker = _find_marker(sentence, replacements + clarifications)
        if marker is None:
            merged.append(sentence)
            continue
        index, phrase = marker
        explicit = phrase in replacements
        before = sentence[:index].strip(" ,;:")
        after = sentence[index + len(phrase) :].strip(" ,;:")
        if not after:
            merged.append(sentence)
            continue
        if _is_correction(before, after, explicit):
            if merged:
                merged[-1] = _replace_tail(merged[-1], after)
            elif before:
                merged.append(_replace_tail(before, after))
            else:
                merged.append(after)
            changes.append(f"corrected: {phrase}")
            continue
        changes.append(f"kept both sides of {phrase}")
        merged.append(f"{before}, {after}" if before else after)
    result = " ".join(part for part in merged if part)
    return result, changes


def _replace_tail(previous: str, fragment: str) -> str:
    words = previous.split()
    fresh = fragment.split()
    if not fresh:
        return previous
    first = fresh[0].strip(".,!?").lower()
    index = None
    for position, word in enumerate(words):
        if word.strip(".,!?").lower() == first:
            index = position
    if index is not None:
        return " ".join(words[:index] + fresh)
    shared = _shared_suffix(words, fresh)
    if not shared:
        return fragment
    cut = len(words) - shared - 1
    while cut > 0 and _bare(words[cut - 1]) in CONNECTORS and _bare(fresh[0]) in CONNECTORS:
        cut -= 1
    return " ".join(words[:cut] + fresh)


def _bare(word: str) -> str:
    return word.strip(".,!?").lower()


def _shared_suffix(words: list[str], fresh: list[str]) -> int:
    count = 0
    while count < len(words) and count < len(fresh):
        if words[-1 - count].strip(".,!?").lower() != fresh[-1 - count].strip(".,!?").lower():
            break
        count += 1
    return count


def _drop_false_starts(text: str) -> tuple[str, list[str]]:
    sentences = [part for part in SENTENCE.findall(text) if part.strip()]
    kept = []
    changes = []
    for sentence in sentences:
        opened = sentence.rstrip()
        if opened.endswith("...") or opened.endswith("…"):
            changes.append("dropped a false start")
            continue
        kept.append(opened.strip())
    return " ".join(kept), changes


def _drop_fillers(text: str, language: str) -> tuple[str, list[str]]:
    key = "pt" if language.lower().startswith("pt") else "en"
    phrases = FILLERS[key]
    working = text
    changes = []
    for phrase in phrases:
        pattern = re.compile(rf"(?<![\wÀ-ÿ]){re.escape(phrase)}(?![\wÀ-ÿ])\s*,?\s*", re.I)
        if pattern.search(working):
            working = pattern.sub("", working)
            changes.append(f"removed filler {phrase}")
    working = re.sub(r"\s{2,}", " ", working)
    working = re.sub(r"\s+([,.;:!?])", r"\1", working)
    working = re.sub(r",\s*,", ",", working)
    return working.strip(" ,;:"), changes


def _collapse_repeats(text: str) -> tuple[str, list[str]]:
    tokens = TOKEN.findall(text)
    kept: list[str] = []
    changes = []
    dropped = False
    for token in tokens:
        if (
            kept
            and token[:1].isalnum()
            and kept[-1].lower() == token.lower()
            and len(token) > 2
        ):
            dropped = True
            continue
        kept.append(token)
    if not dropped:
        return text, []
    return _detokenize(kept), ["collapsed a repeated word"]


def _detokenize(tokens: list[str]) -> str:
    parts: list[str] = []
    for token in tokens:
        if token in TERMINAL or token in {",", ";", ":", "!"}:
            if parts:
                parts[-1] = parts[-1] + token
            else:
                parts.append(token)
            continue
        parts.append(token)
    return " ".join(parts)


def _tidy(text: str) -> tuple[str, list[str]]:
    working = re.sub(r"\s{2,}", " ", text).strip()
    changes = []
    if working and working[0].islower():
        working = working[0].upper() + working[1:]
        changes.append("capitalised the first word")
    if working and working[-1] not in TERMINAL:
        working = working + "."
        changes.append("closed the sentence")
    return working, changes
