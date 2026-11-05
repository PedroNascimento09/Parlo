import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

ADAPTER = Path(__file__).resolve().parents[2] / "tts-piper" / "serve.py"


def load_adapter():
    spec = importlib.util.spec_from_file_location("parlo_piper_adapter", ADAPTER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Chunk:
    def __init__(self, audio: bytes, rate: int):
        self.audio_int16_bytes = audio
        self.sample_rate = rate


class Voice:
    def __init__(self, chunks, rate: int = 22050):
        self.chunks = chunks
        self.config = type("Config", (), {"sample_rate": rate})()
        self.syn_configs = []

    def synthesize(self, text: str, syn_config=None):
        self.syn_configs.append(syn_config)
        yield from self.chunks


class SynthesizeTest(unittest.TestCase):
    def test_text_without_phonemes_answers_with_silence(self):
        audio, rate = load_adapter().synthesize(Voice([]), "...")
        self.assertEqual(audio, b"")
        self.assertEqual(rate, 22050)

    def test_chunks_are_joined_in_order(self):
        voice = Voice([Chunk(b"\x01\x00", 22050), Chunk(b"\x02\x00", 22050)])
        audio, rate = load_adapter().synthesize(voice, "Bom dia.")
        self.assertEqual(audio, b"\x01\x00\x02\x00")
        self.assertEqual(rate, 22050)
        self.assertEqual(voice.syn_configs, [None])

    def test_length_scale_builds_a_piper_synthesis_config(self):
        class SynthesisConfig:
            def __init__(self, **values):
                self.values = values

        fake = type(sys)("piper")
        fake.SynthesisConfig = SynthesisConfig
        voice = Voice([Chunk(b"\x01\x00", 22050)])
        with mock.patch.dict(sys.modules, {"piper": fake}):
            audio, _ = load_adapter().synthesize(voice, "Bom dia.", 0.8)

        self.assertEqual(audio, b"\x01\x00")
        self.assertEqual(voice.syn_configs[0].values, {"length_scale": 0.8})

    def test_length_scale_outside_the_range_is_refused(self):
        with self.assertRaisesRegex(ValueError, "length_scale"):
            load_adapter().synthesize(Voice([]), "Bom dia.", 2.5)


if __name__ == "__main__":
    unittest.main()
