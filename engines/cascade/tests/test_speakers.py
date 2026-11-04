import tempfile
import unittest
from pathlib import Path

from parlo_cascade.speakers import MODEL, Embedder, MissingSpeakerModel, installed, model_path


class SpeakerModelTest(unittest.TestCase):
    def test_a_missing_model_is_named_before_anything_loads(self):
        with tempfile.TemporaryDirectory() as folder:
            self.assertFalse(installed(folder))
            with self.assertRaises(MissingSpeakerModel) as raised:
                Embedder(folder)
            self.assertIn("Model library", str(raised.exception))

    def test_the_model_sits_where_the_download_puts_it(self):
        root = Path("C:/models")
        self.assertEqual(model_path(root), root / MODEL)

    def test_an_installed_file_is_seen(self):
        with tempfile.TemporaryDirectory() as folder:
            (Path(folder) / MODEL).write_bytes(b"weights")
            self.assertTrue(installed(folder))


if __name__ == "__main__":
    unittest.main()
