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


class CleanTests(unittest.TestCase):
    def test_the_sentencepiece_marker_becomes_a_space(self):
        self.assertEqual(mt.clean("\u2581It\u2581costs EUR 249,90\u2581with\u2581VAT"), "It costs EUR 249,90 with VAT")

    def test_a_language_token_is_removed_without_touching_the_sentence(self):
        self.assertEqual(mt.strip_language_token("\u2581It costs 249.90 euros. \u2581>>per<<"), "It costs 249.90 euros.")

    def test_a_separator_left_by_a_removed_token_goes_away(self):
        self.assertEqual(mt.strip_language_token("\u2581Hello there , >>pt<<"), "Hello there")


class MeridiemTests(unittest.TestCase):
    def test_an_afternoon_hour_gets_its_marker_back(self):
        self.assertEqual(mt.restore_meridiem("às 15h30", "at 3:30"), "at 3:30 pm")

    def test_a_whole_afternoon_hour_gets_its_marker_back(self):
        self.assertEqual(mt.restore_meridiem("até às 18h", "by 6:00."), "by 6:00 pm.")

    def test_a_marked_time_is_left_alone(self):
        self.assertEqual(mt.restore_meridiem("às 17h", "at 5:00 pm"), "at 5:00 pm")

    def test_a_morning_time_is_left_alone(self):
        self.assertEqual(mt.restore_meridiem("às 9h30", "at 9:30"), "at 9:30")

    def test_a_different_hour_is_left_alone(self):
        self.assertEqual(mt.restore_meridiem("às 18h", "at 4:00"), "at 4:00")

    def test_only_the_afternoon_clock_of_a_sentence_is_touched(self):
        self.assertEqual(
            mt.restore_meridiem("às 9h e às 18h", "at 9 and at 6:00"),
            "at 9 and at 6:00 pm",
        )


class DiscoveryTests(unittest.TestCase):
    def test_finds_the_newest_pack_for_a_pair(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ("translate-pt_en-1_8", "translate-pt_en-1_9", "translate-en_pt-1_9"):
                (root / name).mkdir()
            self.assertEqual(mt.find_pack("pt", "en", root).name, "translate-pt_en-1_9")
            self.assertEqual(mt.find_pack("en", "pt", root).name, "translate-en_pt-1_9")
            self.assertIsNone(mt.find_pack("pt", "de", root))

    def test_a_missing_pack_folder_is_not_an_error(self):
        with tempfile.TemporaryDirectory() as folder:
            self.assertIsNone(mt.find_pack("pt", "en", Path(folder) / "absent"))

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

    def test_finds_the_vocabulary_of_a_pack_that_keeps_it_at_the_root(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            model = root / "model"
            model.mkdir()
            (model / "model.bin").write_bytes(b"")
            (root / "sentencepiece.model").write_bytes(b"")
            self.assertEqual(mt.find_vocabulary(model), (root / "sentencepiece.model", root / "sentencepiece.model"))

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
