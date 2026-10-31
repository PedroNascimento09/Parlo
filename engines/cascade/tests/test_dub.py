import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import dub

HAS_NUMPY = importlib.util.find_spec("numpy") is not None


class TimestampTest(unittest.TestCase):
    def test_zero(self):
        self.assertEqual(dub.timestamp(0.0), "00:00:00,000")

    def test_rounds_to_the_nearest_millisecond(self):
        self.assertEqual(dub.timestamp(1.2346), "00:00:01,235")
        self.assertEqual(dub.timestamp(0.9996), "00:00:01,000")

    def test_minutes_and_hours(self):
        self.assertEqual(dub.timestamp(61.5), "00:01:01,500")
        self.assertEqual(dub.timestamp(3661.001), "01:01:01,001")

    def test_negative_time_is_clamped_to_zero(self):
        self.assertEqual(dub.timestamp(-2.5), "00:00:00,000")


class SrtTest(unittest.TestCase):
    def test_writes_numbered_blocks_with_a_blank_line_between_them(self):
        lines = [
            dub.Line(0.0, 1.25, "bom dia", "good morning"),
            dub.Line(61.5, 63.0, "obrigado", "thank you"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "out.srt"
            dub.write_srt(path, lines)
            text = path.read_text(encoding="utf-8")

        self.assertEqual(
            text,
            "1\n"
            "00:00:00,000 --> 00:00:01,250\n"
            "good morning\n"
            "\n"
            "2\n"
            "00:01:01,500 --> 00:01:03,000\n"
            "thank you\n",
        )

    def test_writes_only_the_translation(self):
        lines = [dub.Line(0.0, 1.0, "texto de origem", "target text")]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "out.srt"
            dub.write_srt(path, lines)
            text = path.read_text(encoding="utf-8")

        self.assertIn("target text", text)
        self.assertNotIn("texto de origem", text)


@unittest.skipUnless(HAS_NUMPY, "numpy is required for the segmentation helpers")
class FrameTest(unittest.TestCase):
    def test_silence_is_unvoiced(self):
        import numpy as np

        frames = dub.voiced_frames(np.zeros(16_000, dtype=np.float32))
        self.assertEqual(frames.size, 50)
        self.assertFalse(frames.any())

    def test_a_loud_tone_is_voiced(self):
        import numpy as np

        tone = (0.5 * np.sin(np.linspace(0, 400 * np.pi, 16_000))).astype(np.float32)
        frames = dub.voiced_frames(tone)
        self.assertEqual(frames.size, 50)
        self.assertTrue(frames.all())

    def test_returns_nothing_below_one_frame(self):
        import numpy as np

        frames = dub.voiced_frames(np.zeros(100, dtype=np.float32))
        self.assertEqual(frames.size, 0)


@unittest.skipUnless(HAS_NUMPY, "numpy is required for the segmentation helpers")
class SegmentTest(unittest.TestCase):
    def tone(self, seconds):
        import numpy as np

        count = int(seconds * dub.SAMPLE_RATE)
        return (0.5 * np.sin(np.linspace(0, 200 * np.pi, count))).astype(np.float32)

    def silence(self, seconds):
        import numpy as np

        return np.zeros(int(seconds * dub.SAMPLE_RATE), dtype=np.float32)

    def test_pure_silence_has_no_segments(self):
        import numpy as np

        self.assertEqual(dub.segment(np.zeros(16_000 * 3, dtype=np.float32)), [])

    def test_a_continuous_tone_is_one_segment(self):
        samples = self.tone(2.0)
        segments = dub.segment(samples)
        self.assertEqual(len(segments), 1)
        self.assertEqual(segments[0].start, 0)
        self.assertAlmostEqual(segments[0].seconds(), 2.0, delta=0.1)

    def test_a_long_pause_starts_a_new_segment(self):
        import numpy as np

        samples = np.concatenate([self.tone(1.5), self.silence(1.0), self.tone(1.5)])
        segments = dub.segment(samples)
        self.assertEqual(len(segments), 2)
        self.assertLess(segments[0].end, segments[1].start)
        self.assertAlmostEqual(segments[0].seconds(), 1.5, delta=0.1)
        self.assertAlmostEqual(segments[1].seconds(), 1.5, delta=0.1)

    def test_a_short_pause_stays_inside_one_segment(self):
        import numpy as np

        samples = np.concatenate([self.tone(1.5), self.silence(0.1), self.tone(1.5)])
        self.assertEqual(len(dub.segment(samples)), 1)


@unittest.skipUnless(HAS_NUMPY, "numpy is required for the segmentation helpers")
class SplitLongTest(unittest.TestCase):
    def test_covers_the_range_without_gaps_and_respects_the_cap(self):
        import numpy as np

        samples = (0.5 * np.sin(np.linspace(0, 4000 * np.pi, dub.SAMPLE_RATE * 40))).astype(
            np.float32
        )
        pieces = dub.split_long(samples, 0, len(samples))
        limit = int(dub.MAX_SEGMENT_SECONDS * dub.SAMPLE_RATE)

        self.assertGreater(len(pieces), 1)
        self.assertEqual(pieces[0].start, 0)
        self.assertEqual(pieces[-1].end, len(samples))
        for piece, following in zip(pieces, pieces[1:]):
            self.assertEqual(piece.end, following.start)
        for piece in pieces:
            self.assertLessEqual(piece.end - piece.start, limit)

    def test_leaves_a_short_range_alone(self):
        import numpy as np

        samples = np.zeros(dub.SAMPLE_RATE * 3, dtype=np.float32)
        pieces = dub.split_long(samples, 0, len(samples))
        self.assertEqual(pieces, [dub.Segment(0, len(samples))])


if __name__ == "__main__":
    unittest.main()
