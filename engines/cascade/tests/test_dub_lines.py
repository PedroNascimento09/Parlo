import sys
import unittest
from pathlib import Path
from types import SimpleNamespace

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import dub


class LineCacheKeyTest(unittest.TestCase):
    def test_an_old_single_voice_line_keeps_its_exact_key(self):
        self.assertEqual(dub.line_file("line-1"), "line-1.wav")
        self.assertEqual(dub.line_file("line-1", None, 1.0), "line-1.wav")

    def test_voice_and_pace_change_the_key(self):
        base = dub.line_file("line-1")
        self.assertNotEqual(base, dub.line_file("line-1", "pt_BR-cadu-medium", 1.0))
        self.assertNotEqual(
            dub.line_file("line-1", "pt_BR-cadu-medium", 1.0),
            dub.line_file("line-1", "pt_BR-cadu-medium", 1.1),
        )

    def test_a_report_keeps_the_voice_and_the_cache_flag_after_a_restart(self):
        line = dub.line_from_document(
            {
                "id": "l1",
                "source": "Bom dia.",
                "translated": "Bom dia.",
                "voice": "pt_BR-cadu-medium",
                "cached": True,
            }
        )
        self.assertEqual(line.voice, "pt_BR-cadu-medium")
        self.assertTrue(line.cached)

    def test_an_old_report_without_a_voice_still_reads(self):
        line = dub.line_from_document({"id": "l1", "source": "Bom dia."})
        self.assertIsNone(line.voice)
        self.assertFalse(line.cached)

    def test_selecting_the_voice_already_loaded_does_not_reload_it(self):
        voice = "pt_BR-cadu-medium"
        model = dub.MODEL_DIR / f"{voice}.onnx"
        if not model.is_file():
            self.skipTest("the story voice models are not installed")
        releases = []
        engine = SimpleNamespace(
            voice_path=Path("another-voice.onnx"),
            _release_voice=lambda: releases.append(True),
        )
        dub.select_voice(engine, voice, None)
        self.assertEqual(engine.voice_path, model)
        self.assertEqual(len(releases), 1)
        dub.select_voice(engine, voice, None)
        self.assertEqual(len(releases), 1)


if __name__ == "__main__":
    unittest.main()
