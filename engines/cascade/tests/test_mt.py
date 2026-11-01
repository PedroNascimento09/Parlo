import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import mt


class StripTests(unittest.TestCase):
    def test_removes_a_trailing_language_token(self):
        self.assertEqual(mt.strip_language_token("It costs 249.90 euros. >>per<<"), "It costs 249.90 euros.")
        self.assertEqual(mt.strip_language_token("He said he couldn't attend. >>by<<"), "He said he couldn't attend.")

    def test_leaves_text_without_a_token_alone(self):
        self.assertEqual(mt.strip_language_token("Não temos orçamento para isso."), "Não temos orçamento para isso.")

    def test_removes_repeated_tokens(self):
        self.assertEqual(mt.strip_language_token("We'll meet at the door. >>by<< >>for<<"), "We'll meet at the door.")


class DiscoveryTests(unittest.TestCase):
    def test_a_folder_without_a_model_says_so(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaises(mt.MissingModel):
                mt.find_weights(folder)
            with self.assertRaises(mt.MissingModel):
                mt.find_vocabulary(folder)

    def test_finds_a_sibling_hf_vocabulary(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            weights = root / "ct2"
            weights.mkdir()
            (weights / "model.bin").write_bytes(b"")
            hf = root / "hf"
            hf.mkdir()
            (hf / "source.spm").write_bytes(b"")
            (hf / "target.spm").write_bytes(b"")
            self.assertEqual(mt.find_weights(root), weights)
            self.assertEqual(mt.find_vocabulary(weights), (hf / "source.spm", hf / "target.spm"))

    def test_finds_an_argos_style_pack(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            model = root / "model"
            model.mkdir()
            (model / "model.bin").write_bytes(b"")
            (model / "sentencepiece.model").write_bytes(b"")
            self.assertEqual(mt.find_weights(root), model)
            self.assertEqual(mt.find_vocabulary(model), (model / "sentencepiece.model", model / "sentencepiece.model"))

    def test_no_model_in_the_environment_means_no_backend(self):
        saved = sys.modules.get("os")
        import os

        previous = os.environ.pop("PARLO_CASCADE_MT_DIR", None)
        try:
            self.assertIsNone(mt.from_environment())
        finally:
            if previous is not None:
                os.environ["PARLO_CASCADE_MT_DIR"] = previous
            self.assertIs(saved, sys.modules.get("os"))


if __name__ == "__main__":
    unittest.main()
