import importlib.util
import unittest
from pathlib import Path

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

    def synthesize(self, text: str):
        yield from self.chunks


class SynthesizeTest(unittest.TestCase):
    def test_text_without_phonemes_answers_with_silence(self):
        audio, rate = load_adapter().synthesize(Voice([]), "...")
        self.assertEqual(audio, b"")
        self.assertEqual(rate, 22050)

    def test_chunks_are_joined_in_order(self):
        audio, rate = load_adapter().synthesize(
            Voice([Chunk(b"\x01\x00", 22050), Chunk(b"\x02\x00", 22050)]), "Bom dia."
        )
        self.assertEqual(audio, b"\x01\x00\x02\x00")
        self.assertEqual(rate, 22050)


if __name__ == "__main__":
    unittest.main()
