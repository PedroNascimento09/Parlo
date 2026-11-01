import importlib.util
import unittest

from parlo_cascade import (
    CascadeEngine,
    apply_glossary,
    float_to_pcm16,
    load_glossary,
    pcm16_to_float,
    resample,
    rms,
)

HAS_NUMPY = importlib.util.find_spec("numpy") is not None


@unittest.skipUnless(HAS_NUMPY, "numpy is required for the numeric helpers")
class HelpersTest(unittest.TestCase):
    def test_pcm_roundtrip_preserves_levels(self):
        decoded = pcm16_to_float(float_to_pcm16([0.0, 0.5, -0.5]))
        self.assertEqual(len(decoded), 3)
        self.assertAlmostEqual(decoded[1], 0.5, places=3)
        self.assertAlmostEqual(decoded[2], -0.5, places=3)

    def test_pcm_ignores_a_dangling_byte(self):
        self.assertEqual(len(pcm16_to_float(b"\x00\x01\x02")), 1)

    def test_resample_preserves_duration(self):
        source = [0.0] * 1000
        self.assertEqual(len(resample(source, 16_000, 8_000)), 500)
        self.assertEqual(len(resample(source, 22_050, 16_000)), 726)

    def test_resample_is_a_no_op_at_the_same_rate(self):
        result = resample([0.1, 0.2, 0.3], 16_000, 16_000).tolist()
        for actual, expected in zip(result, [0.1, 0.2, 0.3]):
            self.assertAlmostEqual(actual, expected, places=6)

    def test_rms_detects_silence_and_level(self):
        self.assertEqual(rms([0.0] * 100), 0.0)
        self.assertAlmostEqual(rms([0.5] * 100), 0.5, places=5)


class GlossaryTest(unittest.TestCase):
    def test_replaces_names_without_case_sensitivity(self):
        self.assertEqual(apply_glossary("palo falou", {"Palo": "Parlo"}), "Parlo falou")

    def test_leaves_unrelated_words_alone(self):
        self.assertEqual(apply_glossary("o palacio", {"Palo": "Parlo"}), "o palacio")

    def test_parses_pairs_and_ignores_noise(self):
        import tempfile
        from pathlib import Path

        with tempfile.NamedTemporaryFile(
            "w", suffix=".txt", delete=False, encoding="utf-8"
        ) as handle:
            handle.write("# comment\nPalo=Parlo\nbroken\n\n")
            path = Path(handle.name)
        try:
            self.assertEqual(load_glossary(path), {"Palo": "Parlo"})
        finally:
            path.unlink()

    def test_missing_file_is_an_empty_glossary(self):
        from pathlib import Path

        self.assertEqual(load_glossary(Path("does-not-exist.txt")), {})


class EngineTest(unittest.TestCase):
    def test_exposes_the_expected_identity(self):
        engine = CascadeEngine()
        self.assertEqual(engine.id, "cascade")
        self.assertEqual(engine.sample_rate, 16_000)
        self.assertEqual(engine.target_languages, ("en",))

    def test_reports_availability_as_a_boolean(self):
        self.assertIsInstance(CascadeEngine().available, bool)

    def test_parakeet_is_the_default_recogniser(self):
        self.assertEqual(CascadeEngine().stt, "parakeet")

    def test_recogniser_can_be_switched_to_whisper(self):
        import os

        os.environ["PARLO_CASCADE_STT"] = "whisper"
        try:
            self.assertEqual(CascadeEngine().stt, "whisper")
        finally:
            os.environ.pop("PARLO_CASCADE_STT", None)

    def test_unknown_recogniser_is_rejected(self):
        import os

        os.environ["PARLO_CASCADE_STT"] = "nope"
        try:
            with self.assertRaises(ValueError):
                CascadeEngine()
        finally:
            os.environ.pop("PARLO_CASCADE_STT", None)

    def test_unknown_whisper_model_is_rejected(self):
        import os

        os.environ["PARLO_CASCADE_STT"] = "whisper"
        os.environ["PARLO_CASCADE_MODEL"] = "enormous"
        try:
            with self.assertRaises(ValueError) as raised:
                CascadeEngine()
            self.assertIn("enormous", str(raised.exception))
        finally:
            os.environ.pop("PARLO_CASCADE_STT", None)
            os.environ.pop("PARLO_CASCADE_MODEL", None)

    def test_known_whisper_model_is_accepted(self):
        import os

        os.environ["PARLO_CASCADE_STT"] = "whisper"
        os.environ["PARLO_CASCADE_MODEL"] = "small"
        try:
            self.assertEqual(CascadeEngine().model_name, "small")
        finally:
            os.environ.pop("PARLO_CASCADE_STT", None)
            os.environ.pop("PARLO_CASCADE_MODEL", None)

    def test_parakeet_ignores_the_model_name(self):
        import os

        os.environ["PARLO_CASCADE_MODEL"] = "enormous"
        try:
            self.assertEqual(CascadeEngine().stt, "parakeet")
        finally:
            os.environ.pop("PARLO_CASCADE_MODEL", None)

    def test_segment_window_can_be_configured(self):
        import os

        os.environ["PARLO_CASCADE_SEGMENT"] = "3.5"
        try:
            self.assertEqual(CascadeEngine().min_segment, 3.5)
        finally:
            os.environ.pop("PARLO_CASCADE_SEGMENT", None)

    def test_dictation_is_off_by_default(self):
        self.assertFalse(CascadeEngine().dictate)

    def test_dictation_can_be_enabled(self):
        import os

        os.environ["PARLO_CASCADE_DICTATE"] = "1"
        try:
            self.assertTrue(CascadeEngine().dictate)
        finally:
            os.environ.pop("PARLO_CASCADE_DICTATE", None)

    def test_dictation_and_captions_are_separate_modes(self):
        import os

        os.environ["PARLO_CASCADE_DICTATE"] = "1"
        try:
            engine = CascadeEngine()
            self.assertTrue(engine.dictate)
            self.assertFalse(engine.captions)
        finally:
            os.environ.pop("PARLO_CASCADE_DICTATE", None)

    def test_cleanup_mode_defaults_to_light(self):
        self.assertEqual(CascadeEngine().clean_mode, "light")

    def test_an_unknown_cleanup_mode_is_refused(self):
        import os

        os.environ["PARLO_CASCADE_CLEAN"] = "clever"
        try:
            with self.assertRaises(ValueError):
                CascadeEngine()
        finally:
            os.environ.pop("PARLO_CASCADE_CLEAN", None)


if __name__ == "__main__":
    unittest.main()
