import importlib.util
import unittest

from parlo_cascade import CascadeEngine, float_to_pcm16, pcm16_to_float, resample, rms

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


class EngineTest(unittest.TestCase):
    def test_exposes_the_expected_identity(self):
        engine = CascadeEngine()
        self.assertEqual(engine.id, "cascade")
        self.assertEqual(engine.sample_rate, 16_000)
        self.assertEqual(engine.target_languages, ("en",))

    def test_reports_availability_as_a_boolean(self):
        self.assertIsInstance(CascadeEngine().available, bool)


if __name__ == "__main__":
    unittest.main()
