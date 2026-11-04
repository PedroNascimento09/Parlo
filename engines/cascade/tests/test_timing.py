import shutil
import subprocess
import sys
import tempfile
import unittest
import wave
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import timing

FFMPEG = shutil.which("ffmpeg")


class PlanTest(unittest.TestCase):
    def test_natural_keeps_the_spoken_length(self):
        placed = timing.plan("natural", 1.0, 2.5, 4.0, 1.7)
        self.assertEqual(placed.tempo, 1.0)
        self.assertAlmostEqual(placed.end, 2.7)

    def test_concise_keeps_the_trimmed_length(self):
        placed = timing.plan("concise", 1.0, 2.5, 4.0, 1.0)
        self.assertEqual(placed.tempo, 1.0)
        self.assertAlmostEqual(placed.end, 2.0)

    def test_smart_fit_speeds_up_a_long_line_to_the_slot(self):
        placed = timing.plan("smart-fit", 1.0, 2.5, 4.0, 1.7, fastest=1.25, slowest=0.9)
        self.assertAlmostEqual(placed.tempo, 1.7 / 1.5)
        self.assertAlmostEqual(placed.end, 2.5, places=5)

    def test_smart_fit_respects_the_fastest_limit(self):
        placed = timing.plan("smart-fit", 1.0, 2.0, 4.0, 1.7, fastest=1.25, slowest=0.9)
        self.assertEqual(placed.tempo, 1.25)
        self.assertAlmostEqual(placed.end, 1.0 + 1.7 / 1.25, places=5)

    def test_smart_fit_slows_a_short_line_within_the_slowest_limit(self):
        placed = timing.plan("smart-fit", 1.0, 3.0, 4.0, 1.0, fastest=1.5, slowest=0.9)
        self.assertEqual(placed.tempo, 0.9)
        self.assertAlmostEqual(placed.end, 1.0 + 1.0 / 0.9, places=5)

    def test_stretch_uses_the_gap_before_the_next_line(self):
        placed = timing.plan("stretch", 1.0, 2.5, 4.0, 2.0)
        self.assertEqual(placed.tempo, 1.0)
        self.assertAlmostEqual(placed.end, 3.0)

    def test_stretch_speeds_up_when_the_gap_is_too_small(self):
        placed = timing.plan("stretch", 1.0, 2.5, 2.5, 2.0)
        self.assertAlmostEqual(placed.tempo, 2.0 / 1.5, places=5)
        self.assertAlmostEqual(placed.end, 2.5)

    def test_lip_sync_fills_the_slot_exactly(self):
        placed = timing.plan("lip-sync", 1.0, 2.5, 4.0, 1.7)
        self.assertAlmostEqual(placed.tempo, 1.7 / 1.5)
        self.assertAlmostEqual(placed.end, 2.5)

    def test_lip_sync_refuses_audio_only(self):
        with self.assertRaisesRegex(timing.TimingError, "video"):
            timing.plan("lip-sync", 1.0, 2.5, 4.0, 1.7, video=False)

    def test_an_unknown_mode_is_refused(self):
        with self.assertRaisesRegex(timing.TimingError, "timing must be"):
            timing.plan("fastest", 1.0, 2.5, 4.0, 1.7)

    def test_limits_are_validated(self):
        with self.assertRaisesRegex(timing.TimingError, "fastest"):
            timing.check_limits(0.5, 0.9)
        with self.assertRaisesRegex(timing.TimingError, "slowest"):
            timing.check_limits(1.25, 0.1)
        with self.assertRaisesRegex(timing.TimingError, "slowest"):
            timing.check_limits(1.25, 1.5)


class SpeedTest(unittest.TestCase):
    def test_short_speeds_are_chained_inside_the_filter_range(self):
        self.assertEqual(timing.atempo_chain(1.0), "atempo=1.000000")
        self.assertEqual(timing.atempo_chain(4.0), "atempo=2.000000,atempo=2.000000")
        self.assertEqual(timing.atempo_chain(0.25), "atempo=0.500000,atempo=0.500000")

    def test_an_invalid_speed_is_refused(self):
        with self.assertRaises(timing.TimingError):
            timing.atempo_chain(0)

    @unittest.skipUnless(FFMPEG, "ffmpeg is required to change a line's speed")
    def test_ffmpeg_changes_the_wav_duration(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "line.wav"
            target = Path(directory) / "fit.wav"
            with wave.open(str(source), "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(timing.SAMPLE_RATE)
                wav.writeframes(b"\x00\x00" * timing.SAMPLE_RATE * 2)
            timing.stretch_wav(FFMPEG, source, target, 2.0)
            with wave.open(str(target), "rb") as wav:
                seconds = wav.getnframes() / wav.getframerate()
        self.assertAlmostEqual(seconds, 1.0, delta=0.05)

    @unittest.skipUnless(FFMPEG, "ffmpeg is required to change a line's speed")
    def test_the_same_speed_copies_the_file(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "line.wav"
            target = Path(directory) / "fit.wav"
            source.write_bytes(b"unchanged")
            timing.stretch_wav(FFMPEG, source, target, 1.0)
            self.assertEqual(target.read_bytes(), b"unchanged")


if __name__ == "__main__":
    unittest.main()
