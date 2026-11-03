import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

import mt_gate


class OnnxGateTest(unittest.TestCase):
    def test_the_tag_for_a_language(self):
        self.assertEqual(mt_gate.onnx_tag("de"), ">>deu<<")
        self.assertEqual(mt_gate.onnx_tag("en"), ">>eng<<")
        self.assertIsNone(mt_gate.onnx_tag("xx"))

    def test_a_folder_without_the_model_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            args = type("Args", (), {"tag": ""})()
            with self.assertRaises(SystemExit) as error:
                mt_gate.onnx_translator(args, Path(folder), "de")
            self.assertIn("encoder_model.onnx", str(error.exception))

    def test_an_unknown_target_asks_for_a_tag(self):
        with tempfile.TemporaryDirectory() as folder:
            for name in ("encoder_model.onnx", "decoder_model.onnx", "source.spm", "target.spm"):
                (Path(folder) / name).write_bytes(b"")
            args = type("Args", (), {"tag": ""})()
            with self.assertRaises(SystemExit) as error:
                mt_gate.onnx_translator(args, Path(folder), "xx")
            self.assertIn("--tag", str(error.exception))


if __name__ == "__main__":
    unittest.main()
