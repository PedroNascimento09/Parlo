import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from types import SimpleNamespace

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import exports


def line(start, end, text):
    return SimpleNamespace(
        start=start,
        end=end,
        translated=text,
        failed=False,
        skipped=False,
    )


class SidecarTest(unittest.TestCase):
    def test_vtt_and_ass_keep_the_line_times(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            exports.write_vtt(path / "lines.vtt", [line(1.0, 2.7, "Bom dia")])
            exports.write_ass(path / "lines.ass", [line(1.0, 2.7, "Bom dia")])

            vtt = (path / "lines.vtt").read_text(encoding="utf-8")
            ass = (path / "lines.ass").read_text(encoding="utf-8")

        self.assertIn("00:00:01.000 --> 00:00:02.700", vtt)
        self.assertIn("Bom dia", vtt)
        self.assertIn("0:00:01.00,0:00:02.70", ass)
        self.assertIn("Bom dia", ass)

    def test_karaoke_marks_the_line_duration(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "karaoke.ass"
            exports.write_ass(path, [line(1.0, 2.7, "Bom dia")], karaoke=True)
            text = path.read_text(encoding="utf-8")

        self.assertIn("{\\k170}Bom dia", text)

    def test_stems_zip_contains_every_stem(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            for name in ("voice.wav", "original.wav", "mix.wav", "lines.srt"):
                (path / name).write_bytes(b"data")
            archive = exports.stems(
                path / "stems.zip",
                path / "voice.wav",
                path / "original.wav",
                path / "mix.wav",
                path / "lines.srt",
            )

            with zipfile.ZipFile(archive) as opened:
                self.assertEqual(
                    opened.namelist(),
                    ["voice.wav", "original.wav", "mix.wav", "lines.srt"],
                )

    def test_stems_include_the_separated_music(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            for name in ("voice.wav", "music.wav", "original.wav", "mix.wav", "lines.srt"):
                (path / name).write_bytes(b"data")
            archive = exports.stems(
                path / "stems.zip",
                path / "voice.wav",
                path / "original.wav",
                path / "mix.wav",
                path / "lines.srt",
                path / "music.wav",
            )
            with zipfile.ZipFile(archive) as opened:
                self.assertIn("music.wav", opened.namelist())


if __name__ == "__main__":
    unittest.main()
