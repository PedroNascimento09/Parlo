import os
import re
from pathlib import Path

from . import packs

LANGUAGE_TOKEN = re.compile(r"\s*>>[A-Za-z_]{2,8}<<\s*$")
TRAILING_SEPARATOR = re.compile(r"\s+[,.;:]+$")
SOURCE_CLOCK = re.compile(r"\b(1[3-9]|2[0-3])\s?[h:.]\s?([0-5]\d)?\b")
TARGET_CLOCK = re.compile(r"\b(\d{1,2})\s?([:.])\s?([0-5]\d)\b")
MARKED_CLOCK = re.compile(r"\b\d{1,2}\s?[:.]\s?[0-5]\d\s?(?:am|pm)\b|\b\d{1,2}\s?(?:am|pm)\b", re.I)


class MissingModel(RuntimeError):
    pass


def restore_meridiem(source, translation):
    if not description_has_afternoon_clock(source):
        return translation
    result = translation
    for hour_text, minute_text in SOURCE_CLOCK.findall(source):
        hour = int(hour_text)
        minute = int(minute_text) if minute_text else 0
        expected = hour - 12
        for match in TARGET_CLOCK.finditer(result):
            if int(match.group(1)) != expected or int(match.group(3)) != minute:
                continue
            if MARKED_CLOCK.search(result[max(0, match.start() - 4) : match.end() + 4]):
                continue
            result = f"{result[:match.end()]} pm{result[match.end():]}"
            break
    return result


def description_has_afternoon_clock(text):
    return bool(SOURCE_CLOCK.search(text))


def clean(text):
    return " ".join(text.replace("\u2581", " ").split())


def strip_language_token(text):
    previous = None
    cleaned = clean(text)
    while cleaned != previous:
        previous = cleaned
        cleaned = clean(LANGUAGE_TOKEN.sub("", cleaned))
    if LANGUAGE_TOKEN.search(text):
        cleaned = TRAILING_SEPARATOR.sub("", cleaned)
    return cleaned.strip(" .,") if cleaned.endswith("<<") else cleaned


def pack_roots():
    override = os.environ.get("PARLO_MT_PACKS_DIR")
    if override:
        return [Path(override)]
    root = [packs.packs_dir()]
    appdata = os.environ.get("APPDATA")
    if appdata:
        root.append(Path(appdata) / "argos-translate" / "packages")
    root.append(Path.home() / ".local" / "share" / "argos-translate" / "packages")
    return root


def _find_in(folder, source, target):
    prefix = f"translate-{source}_{target}"
    if not folder.is_dir():
        return None
    matches = [
        entry
        for entry in sorted(folder.iterdir())
        if entry.is_dir() and (entry.name == prefix or entry.name.startswith(prefix + "-"))
    ]
    return matches[-1] if matches else None


def find_pack(source, target, root=None):
    if root is not None:
        return _find_in(Path(root), source, target)
    for folder in pack_roots():
        found = _find_in(folder, source, target)
        if found is not None:
            return found
    return None


def find_vocabulary(folder):
    folder = Path(folder)
    for base in (folder, folder.parent, folder.parent / "hf"):
        if (base / "source.spm").is_file() and (base / "target.spm").is_file():
            return base / "source.spm", base / "target.spm"
    for base in (folder, folder.parent):
        for name in ("sentencepiece.model", "sentencepiece.bpe.model"):
            single = base / name
            if single.is_file():
                return single, single
    raise MissingModel(f"no sentencepiece model beside {folder}")


def find_weights(folder):
    folder = Path(folder)
    for candidate in (folder / "ct2", folder, folder / "model"):
        if (candidate / "model.bin").is_file():
            return candidate
    raise MissingModel(f"no ctranslate2 model in {folder}")


class MarianTranslator:
    def __init__(
        self,
        folder,
        prefix=None,
        threads=0,
        beam=4,
        repetition_penalty=1.2,
        no_repeat_ngram_size=3,
        language=None,
        source_token=None,
        target_token=None,
    ):
        import ctranslate2
        import sentencepiece as spm

        weights = find_weights(folder)
        source, target = find_vocabulary(weights)
        self.prefix = prefix
        self.language = language
        self.source_token = source_token
        self.target_token = target_token
        self.beam = beam
        self.repetition_penalty = repetition_penalty
        self.no_repeat_ngram_size = no_repeat_ngram_size
        self.source_piece = spm.SentencePieceProcessor(model_file=str(source))
        self.target_piece = spm.SentencePieceProcessor(model_file=str(target))
        self.translator = ctranslate2.Translator(
            str(weights),
            device="cpu",
            compute_type="int8",
            intra_threads=threads or (os.cpu_count() or 1),
        )

    def __call__(self, text):
        if not text.strip():
            return ""
        prepared = text.strip()
        if self.prefix:
            prepared = f"{prepared} {self.prefix}"
        pieces = self.source_piece.encode(prepared, out_type=str)
        if self.source_token:
            pieces = [self.source_token] + pieces
        options = {"beam_size": self.beam, "max_batch_size": 1}
        if self.repetition_penalty:
            options["repetition_penalty"] = self.repetition_penalty
        if self.no_repeat_ngram_size:
            options["no_repeat_ngram_size"] = self.no_repeat_ngram_size
        if self.target_token:
            options["target_prefix"] = [[self.target_token]]
        result = self.translator.translate_batch([pieces], **options)[0]
        hypothesis = result.hypotheses[0]
        if self.target_token and hypothesis and hypothesis[0] == self.target_token:
            hypothesis = hypothesis[1:]
        translated = strip_language_token(self.target_piece.decode(hypothesis))
        if self.language == "en":
            translated = restore_meridiem(text, translated)
        return translated


def from_environment(source=None, target=None):
    folder = os.environ.get("PARLO_CASCADE_MT_DIR")
    if not folder and source and target:
        pack = find_pack(source, target)
        folder = str(pack) if pack else None
    if not folder:
        return None
    return MarianTranslator(
        folder,
        prefix=os.environ.get("PARLO_CASCADE_MT_PREFIX") or None,
        threads=int(os.environ.get("PARLO_CASCADE_THREADS") or 0),
        language=target,
        source_token=os.environ.get("PARLO_CASCADE_MT_SOURCE_TOKEN") or None,
        target_token=os.environ.get("PARLO_CASCADE_MT_TARGET_TOKEN") or None,
    )
