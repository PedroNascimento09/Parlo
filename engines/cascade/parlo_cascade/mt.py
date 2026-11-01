import os
import re
from pathlib import Path

LANGUAGE_TOKEN = re.compile(r"\s*>>[A-Za-z_]{2,8}<<\s*$")


class MissingModel(RuntimeError):
    pass


def strip_language_token(text):
    previous = None
    cleaned = text
    while cleaned != previous:
        previous = cleaned
        cleaned = LANGUAGE_TOKEN.sub("", cleaned).strip()
    return cleaned.strip(" .,") if cleaned.endswith("<<") else cleaned


def find_vocabulary(folder):
    folder = Path(folder)
    direct = folder / "source.spm"
    if direct.is_file() and (folder / "target.spm").is_file():
        return folder / "source.spm", folder / "target.spm"
    parent = folder.parent / "hf"
    if (parent / "source.spm").is_file() and (parent / "target.spm").is_file():
        return parent / "source.spm", parent / "target.spm"
    single = folder / "sentencepiece.model"
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
        result = self.translator.translate_batch(
            [pieces],
            beam_size=self.beam,
            repetition_penalty=self.repetition_penalty,
            no_repeat_ngram_size=self.no_repeat_ngram_size,
            max_batch_size=1,
        )[0]
        return strip_language_token(self.target_piece.decode(result.hypotheses[0]))


def from_environment():
    folder = os.environ.get("PARLO_CASCADE_MT_DIR")
    if not folder:
        return None
    return MarianTranslator(
        folder,
        prefix=os.environ.get("PARLO_CASCADE_MT_PREFIX") or None,
        threads=int(os.environ.get("PARLO_CASCADE_THREADS") or 0),
    )
