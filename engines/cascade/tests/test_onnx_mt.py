import os
import tempfile
import unittest
from pathlib import Path

from parlo_cascade import mt
from parlo_cascade.mt import MissingModel
from parlo_cascade.onnx_mt import OnnxTranslator, greedy, read_ids


class FakePiece:
    def encode(self, text, out_type=int):
        return [11, 12]


class GreedyTest(unittest.TestCase):
    def test_it_stops_at_the_end_token(self):
        script = iter([7, 8, 2, 9])
        tokens = greedy(lambda _tokens: next(script), start=5, eos=2)
        self.assertEqual(tokens, [7, 8])

    def test_it_respects_the_limit(self):
        tokens = greedy(lambda tokens: len(tokens) + 10, start=0, eos=99, limit=3)
        self.assertEqual(tokens, [11, 12, 13])

    def test_the_tag_comes_first(self):
        self.assertEqual(read_ids("ola", FakePiece(), 692), [692, 11, 12])


class OnnxPairTest(unittest.TestCase):
    def test_a_folder_without_the_export_names_the_missing_file(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaises(MissingModel) as error:
                OnnxTranslator(folder, target="de")
            self.assertIn("encoder_model.onnx", str(error.exception))

    def test_an_unknown_target_needs_a_tag(self):
        with tempfile.TemporaryDirectory() as folder:
            for name in (
                "encoder_model.onnx",
                "decoder_model.onnx",
                "decoder_with_past_model.onnx",
                "source.spm",
                "target.spm",
            ):
                (Path(folder) / name).write_bytes(b"")
            (Path(folder) / "config.json").write_text("{}", encoding="utf-8")
            (Path(folder) / "vocab.json").write_text("{}", encoding="utf-8")
            with self.assertRaises(MissingModel) as error:
                OnnxTranslator(folder, target="xx")
            self.assertIn("tag", str(error.exception))

    def test_a_tag_missing_from_the_vocabulary_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            for name in (
                "encoder_model.onnx",
                "decoder_model.onnx",
                "decoder_with_past_model.onnx",
                "source.spm",
                "target.spm",
            ):
                (Path(folder) / name).write_bytes(b"")
            (Path(folder) / "config.json").write_text("{}", encoding="utf-8")
            (Path(folder) / "vocab.json").write_text("{}", encoding="utf-8")
            with self.assertRaises(MissingModel) as error:
                OnnxTranslator(folder, target="de")
            self.assertIn(">>deu<<", str(error.exception))

    def test_a_pair_is_found_beside_the_other_models(self):
        with tempfile.TemporaryDirectory() as folder:
            pair = Path(folder) / "mt" / "pt-de-onnx"
            pair.mkdir(parents=True)
            (pair / "encoder_model.onnx").write_bytes(b"")
            previous = os.environ.get("PARLO_MODELS_DIR")
            os.environ["PARLO_MODELS_DIR"] = folder
            try:
                self.assertEqual(mt.find_onnx_pair("pt", "de"), pair)
                self.assertIsNone(mt.find_onnx_pair("pt", "fr"))
                self.assertTrue(mt.is_onnx_folder(pair))
            finally:
                if previous is None:
                    os.environ.pop("PARLO_MODELS_DIR", None)
                else:
                    os.environ["PARLO_MODELS_DIR"] = previous


if __name__ == "__main__":
    unittest.main()
