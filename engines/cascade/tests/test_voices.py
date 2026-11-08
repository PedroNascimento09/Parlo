import unittest

from parlo_cascade.engine import VOICES
from parlo_cascade.tts import PiperVoice


class VoicePerLanguageTest(unittest.TestCase):
    def test_each_translated_language_has_a_voice(self):
        self.assertEqual(
            set(VOICES), {"pt", "en", "es", "fr", "de", "it", "nl"}
        )

    def test_the_file_name_matches_the_model_it_downloads(self):
        self.assertEqual(VOICES["de"], "de_DE-thorsten-medium.onnx")
        self.assertEqual(VOICES["fr"], "fr_FR-siwis-medium.onnx")

    def test_a_missing_voice_is_reported_by_name(self):
        voice = PiperVoice("C:/models/does-not-exist.onnx")
        self.assertFalse(voice.installed())


if __name__ == "__main__":
    unittest.main()
