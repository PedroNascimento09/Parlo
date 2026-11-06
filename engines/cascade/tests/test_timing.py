import json
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


FIXTURES = Path(__file__).resolve().parent / "fixtures" / "timing"
RENDERED_MODES = ("natural", "concise", "smart-fit", "stretch", "lip-sync")


def fixture_seconds(path):
    with wave.open(str(path), "rb") as handle:
        return handle.getnframes() / handle.getframerate()


class RenderedModeTest(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((FIXTURES / "manifest.json").read_text(encoding="utf-8"))
        self.modes = self.manifest["modes"]

    def test_every_timing_mode_has_a_rendered_fixture(self):
        for mode in RENDERED_MODES:
            self.assertIn(mode, self.modes)
            self.assertTrue((FIXTURES / f"{mode}.wav").is_file(), mode)

    def test_each_fixture_is_the_length_its_mode_promises(self):
        for mode, values in self.modes.items():
            placed = timing.plan(
                values["strategy"],
                values["start"],
                values["slot_end"],
                values["next_start"],
                values["duration"],
                values["fastest"],
                values["slowest"],
                values["video"],
            )
            self.assertAlmostEqual(placed.tempo, values["tempo"], places=5, msg=mode)
            self.assertAlmostEqual(placed.end, values["end"], places=5, msg=mode)
            seconds = fixture_seconds(FIXTURES / f"{mode}.wav")
            self.assertAlmostEqual(seconds, placed.end - placed.start, delta=0.03, msg=mode)
            self.assertAlmostEqual(seconds, values["seconds"], delta=0.001, msg=mode)

    def test_each_mode_cuts_where_it_says_it_does(self):
        natural = self.modes["natural"]
        concise = self.modes["concise"]
        smart = self.modes["smart-fit"]
        stretch = self.modes["stretch"]
        lip = self.modes["lip-sync"]
        self.assertEqual(natural["tempo"], 1.0)
        self.assertEqual(concise["tempo"], 1.0)
        self.assertLess(concise["seconds"], natural["seconds"])
        self.assertAlmostEqual(smart["end"], smart["slot_end"], places=5)
        self.assertAlmostEqual(stretch["end"], stretch["next_start"], places=5)
        self.assertGreater(stretch["end"], stretch["slot_end"])
        self.assertAlmostEqual(lip["end"], lip["slot_end"], places=5)
        self.assertGreater(lip["tempo"], 1.0)

    def test_the_fixtures_are_speech_and_not_silence(self):
        for mode in RENDERED_MODES:
            with wave.open(str(FIXTURES / f"{mode}.wav"), "rb") as handle:
                self.assertEqual(handle.getframerate(), timing.SAMPLE_RATE, mode)
                self.assertEqual(handle.getnchannels(), 1, mode)
                frames = handle.readframes(handle.getnframes())
            samples = memoryview(frames).cast("h")
            self.assertGreater(max(abs(value) for value in samples), 500, mode)

    @unittest.skipUnless(FFMPEG, "ffmpeg is required to render a line")
    def test_rerendering_the_committed_source_reproduces_each_fixture(self):
        for mode, values in self.modes.items():
            with tempfile.TemporaryDirectory() as directory:
                target = Path(directory) / f"{mode}.wav"
                timing.stretch_wav(FFMPEG, FIXTURES / values["source"], target, values["tempo"])
                seconds = fixture_seconds(target)
            self.assertAlmostEqual(seconds, values["seconds"], delta=0.03, msg=mode)


if __name__ == "__main__":
    unittest.main()
