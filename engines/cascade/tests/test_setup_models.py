import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock
from urllib.error import URLError

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import setup_models

PAYLOAD = b"parlo model payload" * 64
DIGEST = hashlib.sha256(PAYLOAD).hexdigest()


class Response:
    def __init__(self, body: bytes):
        self.body = body

    def read(self, size: int = -1) -> bytes:
        if size is None or size < 0:
            body, self.body = self.body, b""
            return body
        body, self.body = self.body[:size], self.body[size:]
        return body

    def __enter__(self):
        return self

    def __exit__(self, *arguments):
        return False


class DownloadTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.source = self.root / "source.bin"
        self.source.write_bytes(PAYLOAD)
        self.target = self.root / "models" / "model.bin"

    def tearDown(self):
        self.directory.cleanup()

    def part(self) -> Path:
        return self.target.with_suffix(self.target.suffix + ".part")

    def test_a_matching_file_is_installed(self):
        setup_models.download(self.source.as_uri(), self.target, DIGEST, len(PAYLOAD))
        self.assertEqual(self.target.read_bytes(), PAYLOAD)
        self.assertFalse(self.part().exists())

    def test_the_digest_alone_is_enough(self):
        setup_models.download(self.source.as_uri(), self.target, DIGEST, None)
        self.assertEqual(self.target.read_bytes(), PAYLOAD)

    def test_the_size_alone_is_enough(self):
        setup_models.download(self.source.as_uri(), self.target, None, len(PAYLOAD))
        self.assertEqual(self.target.read_bytes(), PAYLOAD)

    def test_a_wrong_digest_installs_nothing(self):
        with self.assertRaises(SystemExit) as raised:
            setup_models.download(self.source.as_uri(), self.target, "0" * 64, len(PAYLOAD))
        self.assertIn("digest", str(raised.exception))
        self.assertFalse(self.target.exists())
        self.assertFalse(self.part().exists())

    def test_a_wrong_size_installs_nothing(self):
        with self.assertRaises(SystemExit) as raised:
            setup_models.download(self.source.as_uri(), self.target, DIGEST, len(PAYLOAD) + 1)
        self.assertIn("bytes", str(raised.exception))
        self.assertFalse(self.target.exists())
        self.assertFalse(self.part().exists())

    def test_a_missing_source_leaves_no_half_file(self):
        with self.assertRaises(URLError):
            setup_models.download((self.root / "absent.bin").as_uri(), self.target, DIGEST, 10)
        self.assertFalse(self.target.exists())
        self.assertFalse(self.part().exists())

    def test_an_existing_file_of_the_published_size_is_not_downloaded_again(self):
        self.target.parent.mkdir(parents=True)
        self.target.write_bytes(b"already here")
        setup_models.download(self.source.as_uri(), self.target, None, 12)
        self.assertEqual(self.target.read_bytes(), b"already here")

    def test_an_existing_file_of_the_wrong_size_is_replaced(self):
        self.target.parent.mkdir(parents=True)
        self.target.write_bytes(b"truncated")
        expected = hashlib.sha256(self.source.read_bytes()).hexdigest()
        setup_models.download(
            self.source.as_uri(), self.target, expected, self.source.stat().st_size
        )
        self.assertEqual(self.target.read_bytes(), self.source.read_bytes())
        self.assertFalse(self.target.with_suffix(self.target.suffix + ".part").exists())


class MetadataTest(unittest.TestCase):
    def tree(self, entries):
        body = json.dumps(entries).encode("utf-8")
        return mock.patch.object(
            setup_models.urllib.request, "urlopen", return_value=Response(body)
        )

    def test_reads_the_published_digest_of_a_large_file(self):
        entries = [
            {
                "type": "file",
                "path": "encoder.int8.onnx",
                "size": 652184281,
                "lfs": {"oid": DIGEST, "size": 652184281},
            }
        ]
        with self.tree(entries):
            self.assertEqual(
                setup_models.metadata("repo", "encoder.int8.onnx"), (DIGEST, 652184281)
            )

    def test_reads_the_size_of_a_file_without_a_digest(self):
        entries = [{"type": "file", "path": "tokens.txt", "size": 93939}]
        with self.tree(entries):
            self.assertEqual(setup_models.metadata("repo", "tokens.txt"), (None, 93939))

    def test_a_directory_entry_is_skipped(self):
        entries = [
            {"type": "directory", "path": "test_wavs", "size": 0},
            {"type": "file", "path": "tokens.txt", "size": 12},
        ]
        with self.tree(entries):
            self.assertEqual(setup_models.metadata("repo", "tokens.txt"), (None, 12))

    def test_an_unpublished_file_is_reported(self):
        with self.tree([{"type": "file", "path": "tokens.txt", "size": 12}]):
            with self.assertRaises(LookupError):
                setup_models.metadata("repo", "missing.onnx")

    def test_a_nested_path_lists_its_own_directory(self):
        captured = []

        def urlopen(url, timeout=0):
            captured.append(url)
            return Response(b"[]")

        with mock.patch.object(setup_models.urllib.request, "urlopen", urlopen):
            with self.assertRaises(LookupError):
                setup_models.metadata("rhasspy/piper-voices", "pt/pt_BR/faber/model.onnx")
        self.assertTrue(captured[0].endswith("/tree/main/pt/pt_BR/faber"))

    def test_a_failed_lookup_degrades_to_no_expectation(self):
        with mock.patch.object(
            setup_models.urllib.request, "urlopen", side_effect=URLError("offline")
        ):
            self.assertEqual(
                setup_models.expected("repo", "encoder.int8.onnx", Path("encoder.int8.onnx")),
                (None, None),
            )


if __name__ == "__main__":
    unittest.main()
