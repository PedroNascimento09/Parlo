import json
import os
import tempfile
import unittest
from pathlib import Path

from parlo_cascade import mt, packs
from parlo_cascade.engine import CascadeEngine

TABLE = Path(__file__).resolve().parents[3] / "crates" / "parlo-cli" / "web" / "languages.json"


class WritePathTest(unittest.TestCase):
    def setUp(self):
        self.table = json.loads(TABLE.read_text(encoding="utf-8"))
        self.pairs = [tuple(pair) for pair in self.table["pairs"]]
        self.root = Path(tempfile.mkdtemp())
        os.environ["PARLO_MT_PACKS_DIR"] = str(self.root)
        for source, target in self.pairs:
            pack = self.root / packs.folder_name(source, target, "1.9")
            pack.mkdir()
            (pack / "source.spm").write_bytes(b"")
            (pack / "target.spm").write_bytes(b"")

    def tearDown(self):
        os.environ.pop("PARLO_MT_PACKS_DIR", None)

    def writer(self, source, target):
        pack = mt.find_pack(source, target)
        if pack is None:
            raise LookupError(f"no pack for {source}->{target}")
        prefix = f"translate-{source}_{target}-"
        if not pack.name.startswith(prefix):
            raise LookupError(f"{pack.name} is not a {prefix} pack")
        pack_id = f"mt-{source}-{target}"
        print(f"{source}->{target} {pack_id} {pack.name}")
        return lambda text: f"{pack_id}: {text}"

    def engine(self, source, target="en"):
        engine = CascadeEngine.__new__(CascadeEngine)
        engine.source = source
        engine.target = target
        engine.transcribe = lambda pcm16: "Bom dia"
        engine.clean_mode = "light"
        engine._touch = lambda name: None
        engine._load_writer = lambda language: self.writer(source, language)
        return engine

    def test_every_offered_language_writes_with_its_registry_pack(self):
        for source, target in self.pairs:
            with self.subTest(source=source, target=target):
                pack_id = f"mt-{source}-{target}"
                text, translation = self.engine(source).write(b"pcm16", target)
                self.assertEqual(text, "Bom dia")
                self.assertEqual(translation, f"{pack_id}: Bom dia")

    def test_each_language_picks_its_own_pack(self):
        for source in sorted({pair[0] for pair in self.pairs}):
            targets = sorted({pair[1] for pair in self.pairs if pair[0] == source})
            found = {
                target: mt.find_pack(source, target).name
                for target in targets
            }
            print(f"{source} -> {found}")
            self.assertEqual(len(found), len(set(found.values())))

    def test_an_unknown_language_keeps_the_words(self):
        text, translation = self.engine("pt").write(b"pcm16", "xx")
        self.assertEqual(text, "Bom dia")
        self.assertEqual(translation, "")

    def test_the_spoken_language_needs_no_pack(self):
        def refuse(language):
            raise AssertionError(f"a pack was loaded for {language}")

        engine = self.engine("pt", target="pt")
        engine._load_writer = refuse
        text, translation = engine.write(b"pcm16", "pt")
        self.assertEqual(text, "Bom dia")
        self.assertEqual(translation, text)


if __name__ == "__main__":
    unittest.main()
