import os
import re
from pathlib import Path

LANGUAGE_TOKEN = re.compile(r"\s*>>[A-Za-z_]{2,8}<<\s*$")
TRAILING_SEPARATOR = re.compile(r"\s+[,.;:]+$")


class MissingModel(RuntimeError):
    pass


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


def packages_root():
    override = os.environ.get("PARLO_MT_PACKS_DIR")
    if override:
        return Path(override)
    candidates = []
    appdata = os.environ.get("APPDATA")
    if appdata:
        candidates.append(Path(appdata) / "argos-translate" / "packages")
    candidates.append(Path.home() / ".local" / "share" / "argos-translate" / "packages")
    for candidate in candidates:
        if candidate.is_dir():
            return candidate
    return candidates[-1]


def find_pack(source, target, root=None):
    folder = Path(root) if root else packages_root()
    prefix = f"translate-{source}_{target}"
    if not folder.is_dir():
        return None
    matches = [
        entry
        for entry in sorted(folder.iterdir())
        if entry.is_dir() and (entry.name == prefix or entry.name.startswith(prefix + "-"))
    ]
    return matches[-1] if matches else None


def find_vocabulary(folder):
    folder = Path(folder)
    for base in (folder, folder.parent, folder.parent / "hf"):
        if (base / "source.spm").is_file() and (base / "target.spm").is_file():
            return base / "source.spm", base / "target.spm"
    for base in (folder, folder.parent):
        single = base / "sentencepiece.model"
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
    def __init__(self, folder, prefix=None, threads=0, beam=4, repetition_penalty=1.2, no_repeat_ngram_size=3):
        import ctranslate2
        import sentencepiece as spm

        weights = find_weights(folder)
        source, target = find_vocabulary(weights)
        self.prefix = prefix
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
        options = {"beam_size": self.beam, "max_batch_size": 1}
        if self.repetition_penalty:
            options["repetition_penalty"] = self.repetition_penalty
        if self.no_repeat_ngram_size:
            options["no_repeat_ngram_size"] = self.no_repeat_ngram_size
        result = self.translator.translate_batch([pieces], **options)[0]
        return strip_language_token(self.target_piece.decode(result.hypotheses[0]))


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
    )
