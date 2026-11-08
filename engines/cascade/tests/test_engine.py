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
    trim_silence,
)

HAS_NUMPY = importlib.util.find_spec("numpy") is not None


class CleanTextTest(unittest.TestCase):
    def engine(self):
        engine = CascadeEngine.__new__(CascadeEngine)
        engine.source = "pt"
        engine.clean_mode = "light"
        engine.last_original = ""
        engine.last_engine = ""
        return engine

    def test_cleaning_keeps_the_sentence_words(self):
        engine = self.engine()

        self.assertEqual(engine.clean_text("   "), "")

        cleaned = engine.clean_text("Bom dia, isto é uma prova.")

        self.assertIn("prova", cleaned)
        self.assertTrue(cleaned[0].isupper())

    def test_cleaning_keeps_the_spoken_language_for_the_sentence(self):
        engine = self.engine()

        cleaned = engine.clean_text("Podemos começar amanhã.", "original")

        self.assertEqual(cleaned.strip(), "Podemos começar amanhã.")


class FlushTest(unittest.TestCase):
    def engine(self):
        engine = CascadeEngine.__new__(CascadeEngine)
        engine._ready = True
        engine._buffer = bytearray()
        engine.dictate = False
        engine.captions = False
        engine.min_segment = 2.5
        engine._transcribe = lambda samples: "bom dia"
        engine._translate = lambda text: "good morning"
        engine._synthesize = lambda text: b"\x01\x00" * 10
        return engine

    def test_translate_holds_the_tail_until_the_stream_ends(self):
        engine = self.engine()

        self.assertIsNone(engine.translate(float_to_pcm16([0.25] * 4000)).text)

        flushed = engine.flush()

        self.assertEqual(flushed.text, "good morning")
        self.assertEqual(flushed.pcm16, b"\x01\x00" * 10)

    def test_file_translation_drops_the_recogniser_before_translating(self):
        events = []
        engine = CascadeEngine.__new__(CascadeEngine)
        engine._ready = True
        engine.dictate = False
        engine.captions = False
        engine.min_segment = 2.5
        engine._transcribe = lambda samples: (events.append("transcribe") or "bom dia")
        engine._translate = lambda text: (events.append("translate") or "good morning")
        engine._synthesize = lambda text: (events.append("speak") or b"\x01\x00" * 10)
        engine.unload_unit = lambda unit: (events.append("unload " + unit) or True)

        result = engine.translate_file(float_to_pcm16([0.25] * 48000))

        self.assertEqual(events[:3], ["transcribe", "transcribe", "unload recognition"])
        self.assertEqual(events[3:], ["translate", "translate", "speak", "speak"])
        self.assertEqual(result.text, "good morning good morning")
        self.assertEqual(result.pcm16, b"\x01\x00" * 20)

    def test_flush_without_a_tail_speaks_nothing(self):
        engine = self.engine()

        flushed = engine.flush()

        self.assertIsNone(flushed.text)
        self.assertEqual(flushed.pcm16, b"")


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


@unittest.skipUnless(HAS_NUMPY, "numpy is required for the numeric helpers")
class TrimTest(unittest.TestCase):
    def test_a_long_silence_is_trimmed_down_to_the_speech(self):
        silence = [0.0] * 16_000
        speech = [0.2, -0.2] * 12_800

        trimmed = trim_silence(silence + speech + silence)

        self.assertLess(len(trimmed), 57_600)
        self.assertGreaterEqual(len(trimmed), 25_600)
        self.assertLessEqual(len(trimmed), 34_600)

    def test_a_quiet_recording_is_left_alone(self):
        quiet = [0.001, -0.001] * 40_000

        self.assertEqual(len(trim_silence(quiet)), 80_000)

    def test_speech_that_fills_the_clip_is_left_alone(self):
        loud = [0.3, -0.3] * 40_000

        self.assertEqual(len(trim_silence(loud)), 80_000)

    def test_a_clip_shorter_than_the_frame_floor_is_left_alone(self):
        self.assertEqual(len(trim_silence([0.0] * 500)), 500)


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


class PairSwitchTest(unittest.TestCase):
    def test_a_switch_keeps_the_recogniser_and_the_known_languages(self):
        engine = CascadeEngine()
        recogniser = object()
        writer = lambda text: text
        engine._stt = recogniser
        engine._translator = writer
        engine._writers["en"] = writer
        engine._tts = object()
        engine._buffer.extend(b"\x00\x00")
        engine.set_pair("pt", "de")
        self.assertEqual(engine.target, "de")
        self.assertEqual(engine.target_languages, ("de",))
        self.assertIs(engine._stt, recogniser)
        self.assertIsNone(engine._translator)
        self.assertIs(engine._writers["en"], writer)
        self.assertIsNone(engine._tts)
        self.assertEqual(engine._buffer, bytearray())

    def test_a_new_source_drops_the_languages_it_can_no_longer_write(self):
        engine = CascadeEngine()
        engine._writers["en"] = lambda text: text
        engine.set_pair("en", "pt")
        self.assertEqual(engine._writers, {})

    def test_the_same_pair_keeps_what_is_loaded(self):
        engine = CascadeEngine()
        translator = lambda text: text
        engine._translator = translator
        engine.set_pair("pt", "en")
        self.assertIs(engine._translator, translator)

    def test_a_pair_needs_two_letter_codes(self):
        engine = CascadeEngine()
        with self.assertRaises(ValueError):
            engine.set_pair("pt", "english")
        with self.assertRaises(ValueError):
            engine.set_pair("de", "de")

    def test_a_language_that_was_written_before_is_not_loaded_again(self):
        from unittest import mock

        engine = CascadeEngine()
        with mock.patch(
            "parlo_cascade.mt.from_environment", side_effect=lambda source, target: (source, target)
        ) as loader:
            engine.set_pair("pt", "es")
            self.assertEqual(engine._load_writer("es"), ("pt", "es"))
            engine.set_pair("pt", "en")
            self.assertEqual(engine._load_writer("en"), ("pt", "en"))
            engine.set_pair("pt", "es")
            self.assertEqual(engine._load_writer("es"), ("pt", "es"))
        self.assertEqual(loader.call_count, 2)

    def test_the_writer_cache_holds_two_languages(self):
        from unittest import mock

        engine = CascadeEngine()
        engine._writers["pt"] = "oldest"
        engine._writers["en"] = "newer"
        with mock.patch("parlo_cascade.mt.from_environment", side_effect=lambda source, target: target):
            engine._load_writer("es")
        self.assertEqual(list(engine._writers), ["en", "es"])

    def test_the_voice_follows_the_new_target(self):
        engine = CascadeEngine()
        engine.set_pair("pt", "de")
        self.assertEqual(engine.voice_path.name, "de_DE-thorsten-medium.onnx")


if __name__ == "__main__":
    unittest.main()
