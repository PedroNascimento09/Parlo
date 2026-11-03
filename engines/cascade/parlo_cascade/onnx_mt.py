import json
import os
from pathlib import Path

from .mt import MissingModel, clean

TAGS = {
    "de": ">>deu<<",
    "en": ">>eng<<",
    "fr": ">>fra<<",
    "pt": ">>por<<",
    "nl": ">>nld<<",
    "es": ">>spa<<",
    "it": ">>ita<<",
}
REQUIRED = (
    "encoder_model.onnx",
    "decoder_model.onnx",
    "decoder_with_past_model.onnx",
    "config.json",
    "vocab.json",
    "source.spm",
    "target.spm",
)


def greedy(step, start, eos, limit=64):
    tokens = [start]
    for _ in range(limit):
        chosen = step(tokens)
        if chosen == eos:
            break
        tokens.append(chosen)
    return tokens[1:]


def read_ids(text, piece, tag_id):
    return [tag_id] + piece.encode(text.strip(), out_type=int)


class OnnxTranslator:
    def __init__(self, folder, target=None, tag=None, threads=0, limit=64):
        import onnxruntime as ort
        import sentencepiece as spm

        self.folder = Path(folder)
        for name in REQUIRED:
            if not (self.folder / name).is_file():
                raise MissingModel(f"{self.folder} is missing {name}")
        self.config = json.loads((self.folder / "config.json").read_text(encoding="utf-8"))
        self.vocab = json.loads((self.folder / "vocab.json").read_text(encoding="utf-8"))
        self.tag = tag or TAGS.get(target or "")
        if not self.tag:
            raise MissingModel("no language tag is known for this pair; pass one")
        if self.tag not in self.vocab:
            raise MissingModel(f"{self.tag} is not in the vocabulary of {self.folder}")
        self.tag_id = int(self.vocab[self.tag])
        self.source = spm.SentencePieceProcessor(model_file=str(self.folder / "source.spm"))
        self.target = spm.SentencePieceProcessor(model_file=str(self.folder / "target.spm"))
        options = ort.SessionOptions()
        options.intra_op_num_threads = threads or (os.cpu_count() or 1)
        self.sessions = {
            name: ort.InferenceSession(
                str(self.folder / name), options, providers=["CPUExecutionProvider"]
            )
            for name in ("encoder_model.onnx", "decoder_model.onnx", "decoder_with_past_model.onnx")
        }
        self.start = int(self.config.get("decoder_start_token_id", 0))
        self.eos = int(self.config.get("eos_token_id", 1))
        self.limit = limit

    def _encode(self, ids):
        import numpy as np

        input_ids = np.array([ids], dtype=np.int64)
        mask = np.ones_like(input_ids)
        hidden = self.sessions["encoder_model.onnx"].run(
            None, {"input_ids": input_ids, "attention_mask": mask}
        )[0]
        return mask, hidden

    def __call__(self, text):
        if not text.strip():
            return ""
        mask, hidden = self._encode(read_ids(text, self.source, self.tag_id))
        state = {}

        def step(tokens):
            import numpy as np

            if "past" not in state:
                feeds = {
                    "input_ids": np.array([tokens], dtype=np.int64),
                    "encoder_attention_mask": mask,
                    "encoder_hidden_states": hidden,
                }
                outputs = self.sessions["decoder_model.onnx"].run(None, feeds)
            else:
                feeds = {
                    "input_ids": np.array([[tokens[-1]]], dtype=np.int64),
                    "encoder_attention_mask": mask,
                    **state["past"],
                }
                outputs = self.sessions["decoder_with_past_model.onnx"].run(None, feeds)
            logits = outputs[0][0][-1]
            chosen = int(max(range(len(logits)), key=lambda index: logits[index]))
            state["past"] = {
                name: value
                for name, value in zip(self._present_names(), outputs[1:])
                if name.startswith("present")
            }
            return chosen

        produced = greedy(step, self.start, self.eos, self.limit)
        return clean(self.target.decode(produced))

    def _present_names(self):
        names = [output.name for output in self.sessions["decoder_model.onnx"].get_outputs()][1:]
        return [
            name.replace("present", "past_key_values", 1) if name.startswith("present") else name
            for name in names
        ]
