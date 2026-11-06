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


FIXTURES = Path(__file__).resolve().parent / "fixtures" / "separation"
FFMPEG = shutil.which("ffmpeg")


def correlation(first, second):
    length = min(len(first), len(second))
    left = first[:length] - np.mean(first[:length])
    right = second[:length] - np.mean(second[:length])
    return float(np.dot(left, right) / (np.linalg.norm(left) * np.linalg.norm(right)))


def read_mono(path):
    with wave.open(str(path), "rb") as handle:
        raw = handle.readframes(handle.getnframes())
    return np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32768.0


class CenterSeparationTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.work = Path(self.directory.name)
        self.target = self.work / "music.wav"

    def tearDown(self):
        self.directory.cleanup()

    @unittest.skipUnless(FFMPEG, "ffmpeg is required to separate a stereo file")
    def test_the_committed_fixture_comes_back_as_center_with_the_music_kept(self):
        fixture = FIXTURES / "centered-speech.wav"
        background, mode, note = separation.separate(
            FFMPEG, fixture, self.target, self.work
        )
        self.assertEqual(mode, "center", note)
        self.assertEqual(background, self.target)
        self.assertIn("Center speech was removed", note)
        samples = read_mono(self.target)
        self.assertGreater(float(np.sqrt(np.mean(samples * samples))), 0.05)
        stereo = separation.read_stereo(fixture)
        music = (stereo[:, 0] - stereo[:, 1]) * 0.5
        self.assertGreater(correlation(samples, music), 0.9)
        voice = read_mono(
            Path(__file__).resolve().parent / "fixtures" / "timing" / "source.wav"
        )
        self.assertLess(abs(correlation(samples, voice)), 0.1)

    @unittest.skipUnless(FFMPEG, "ffmpeg is required to separate a stereo file")
    def test_a_centred_only_source_still_falls_back_to_ducking(self):
        voice = read_mono(
            Path(__file__).resolve().parent / "fixtures" / "timing" / "source.wav"
        )
        source = self.work / "centred.wav"
        with wave.open(str(source), "wb") as handle:
            handle.setnchannels(2)
            handle.setsampwidth(2)
            handle.setframerate(separation.SAMPLE_RATE)
            handle.writeframes(
                (np.stack([voice, voice], axis=1) * 32767).astype("<i2").tobytes()
            )
        background, mode, note = separation.separate(
            FFMPEG, source, self.target, self.work
        )
        self.assertIsNone(background)
        self.assertEqual(mode, "ducking")
        self.assertIn("same signal", note)
        self.assertFalse(self.target.exists())


if __name__ == "__main__":
    unittest.main()
