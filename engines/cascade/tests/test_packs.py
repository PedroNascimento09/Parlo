import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import packs


def build_archive(path, names=("model/model.bin", "sentencepiece.model")):
    with zipfile.ZipFile(path, "w") as bundle:
        for name in names:
            bundle.writestr(name, b"x")
    return path


def build_index(folder, archive, source="pt", target="en", version="1.9"):
    index = folder / "index.json"
    index.write_text(
        json.dumps(
            [
                {
                    "package_version": version,
                    "from_code": source,
                    "to_code": target,
                    "links": [archive.as_uri()],
                }
            ]
        ),
        encoding="utf-8",
    )
    return index.as_uri()


class ChoiceTests(unittest.TestCase):
    def test_finds_the_entry_and_its_link(self):
        entries = [{"from_code": "en", "to_code": "pt", "links": ["u"], "package_version": "1.9"}]
        entry, url = packs.choose(entries, "en", "pt")
        self.assertEqual(entry["package_version"], "1.9")
        self.assertEqual(url, "u")

    def test_an_unlisted_pair_is_an_error(self):
        with self.assertRaises(packs.PackError):
            packs.choose([], "pt", "de")

    def test_an_entry_without_a_link_is_an_error(self):
        with self.assertRaises(packs.PackError):
            packs.choose([{"from_code": "pt", "to_code": "en", "links": []}], "pt", "en")

    def test_the_folder_carries_the_version(self):
        self.assertEqual(packs.folder_name("pt", "en", "1.9"), "translate-pt_en-1_9")


class ExtractTests(unittest.TestCase):
    def test_a_normal_archive_unpacks(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "ok.argosmodel")
            target = root / "out"
            target.mkdir()
            packs.extract(archive, target)
            self.assertTrue((target / "model" / "model.bin").is_file())

    def test_an_archive_that_escapes_the_folder_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "bad.argosmodel", ("../evil.txt", "model/model.bin"))
            target = root / "out"
            target.mkdir()
            with self.assertRaises(packs.PackError):
                packs.extract(archive, target)
            self.assertFalse((root / "evil.txt").exists())

    def test_an_archive_with_one_top_folder_unpacks_at_the_root(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = root / "nested.argosmodel"
            with zipfile.ZipFile(archive, "w") as bundle:
                bundle.writestr("translate-en_de-1_3/model/model.bin", b"x")
                bundle.writestr("translate-en_de-1_3/sentencepiece.model", b"x")
            target = root / "out"
            target.mkdir()
            packs.extract(archive, target)
            self.assertTrue((target / "model" / "model.bin").is_file())
            self.assertTrue((target / "sentencepiece.model").is_file())

    def test_a_nested_archive_cannot_escape_either(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = root / "bad.argosmodel"
            with zipfile.ZipFile(archive, "w") as bundle:
                bundle.writestr("nested/../../evil.txt", b"x")
            target = root / "out"
            target.mkdir()
            with self.assertRaises(packs.PackError):
                packs.extract(archive, target)
            self.assertFalse((root / "evil.txt").exists())

    def test_an_archive_without_the_model_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "thin.argosmodel", ("sentencepiece.model",))
            target = root / "out"
            target.mkdir()
            with self.assertRaises(packs.PackError):
                packs.extract(archive, target)


class DownloadTests(unittest.TestCase):
    def test_every_request_names_a_client(self):
        seen = []

        class Answer:
            def __init__(self, payload):
                self.payload = payload

            def __enter__(self):
                return self

            def __exit__(self, *_):
                return False

            def read(self, _=0):
                payload, self.payload = self.payload, b""
                return payload

        def opener(url, timeout=0):
            seen.append(url)
            return Answer(b"abc")

        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "file.bin"
            packs.download("https://example.invalid/file", target, opener=opener)
            self.assertEqual(target.read_bytes(), b"abc")
        self.assertEqual(seen[0].get_header("User-agent"), packs.USER_AGENT)


class InstallTests(unittest.TestCase):
    def test_a_package_is_downloaded_verified_and_recorded(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "pt_en.argosmodel")
            index = build_index(root, archive)
            installed = packs.install("pt", "en", root=root / "packs", index_url=index)
            self.assertEqual(installed.name, "translate-pt_en-1_9")
            self.assertTrue((installed / "model" / "model.bin").is_file())
            record = json.loads((installed / "parlo.json").read_text(encoding="utf-8"))
            self.assertEqual(record["version"], "1.9")
            self.assertEqual(len(record["sha256"]), 64)
            self.assertEqual(record["source"], archive.as_uri())

    def test_installing_twice_keeps_the_first_copy(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "pt_en.argosmodel")
            index = build_index(root, archive)
            first = packs.install("pt", "en", root=root / "packs", index_url=index)
            (first / "parlo.json").unlink()
            second = packs.install("pt", "en", root=root / "packs", index_url=index)
            self.assertEqual(first, second)
            self.assertFalse((second / "parlo.json").exists())

    def test_a_missing_pair_installs_nothing(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = build_archive(root / "pt_en.argosmodel")
            index = build_index(root, archive)
            with self.assertRaises(packs.PackError):
                packs.install("pt", "de", root=root / "packs", index_url=index)
            self.assertFalse((root / "packs").exists())


if __name__ == "__main__":
    unittest.main()
