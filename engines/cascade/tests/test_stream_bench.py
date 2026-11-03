import json
import sys
import tempfile
import unittest
import wave
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

import stream_bench


def write_wav(path: Path, rate: int = 16_000) -> None:
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(rate)
        handle.writeframes(b"\x00\x00" * rate)


class Clips(unittest.TestCase):
    def test_manifest_lists_clips_in_order(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            rows = [
                {"audio": "a.wav", "text": "bom dia"},
                {"audio": "b.wav", "text": "até logo"},
            ]
            (root / "reference.jsonl").write_text(
                "\n".join(json.dumps(row) for row in rows), encoding="utf-8"
            )
            clips = stream_bench.load_clips(root)
            self.assertEqual([path.name for path, _ in clips], ["a.wav", "b.wav"])
            self.assertEqual(clips[1][1], "até logo")

    def test_read_wav_returns_one_second_of_samples(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "clip.wav"
            write_wav(path)
            self.assertEqual(len(stream_bench.read_wav(path)), 16_000)

    def test_read_wav_refuses_another_rate(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "clip.wav"
            write_wav(path, rate=8_000)
            with self.assertRaises(SystemExit):
                stream_bench.read_wav(path)


if __name__ == "__main__":
    unittest.main()
