import shutil
import sys
import tempfile
import unittest
import wave
from pathlib import Path

import numpy as np

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import separation


class SeparationTest(unittest.TestCase):
    def test_center_side_removes_center_speech_and_keeps_sides(self):
        time = np.arange(separation.SAMPLE_RATE) / separation.SAMPLE_RATE
        voice = 0.35 * np.sin(2 * np.pi * 500 * time)
        left = voice + 0.4 * np.sin(2 * np.pi * 220 * time)
        right = voice + 0.4 * np.sin(2 * np.pi * 330 * time)
        stereo = np.stack([left, right], axis=1)
        side = separation.center_side(stereo)
        left_side = side + voice
        self.assertLess(np.max(np.abs(side - (stereo[:, 0] - stereo[:, 1]) * 0.5)), 1e-6)
        self.assertGreater(np.max(np.abs(left_side)), 0.2)

    @unittest.skipUnless(shutil.which("ffmpeg"), "ffmpeg is required")
    def test_mono_source_falls_back_to_ducking(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            source = path / "mono.wav"
            with wave.open(str(source), "wb") as handle:
                handle.setnchannels(1)
                handle.setsampwidth(2)
                handle.setframerate(separation.SAMPLE_RATE)
                handle.writeframes(b"\x00\x01" * separation.SAMPLE_RATE)
            background, mode, note = separation.separate(
                shutil.which("ffmpeg"), source, path / "music.wav", path
            )
            self.assertIsNone(background)
            self.assertEqual(mode, "ducking")
            self.assertIn("stereo channels", note)
            self.assertFalse((path / "music.wav").exists())


if __name__ == "__main__":
    unittest.main()
