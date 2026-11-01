import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

from parlo_cascade import credentials


@unittest.skipUnless(credentials.available(), "this system has no credential store")
class CredentialTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.mkdtemp()
        os.environ["PARLO_CREDENTIALS_DIR"] = self.folder
        self.name = "parlo-test-provider"

    def tearDown(self):
        os.environ.pop("PARLO_CREDENTIALS_DIR", None)
        credentials.delete(self.name)
        shutil.rmtree(self.folder, ignore_errors=True)

    def test_a_secret_round_trips(self):
        credentials.store(self.name, "sk-not-a-real-key")
        self.assertEqual(credentials.load(self.name), "sk-not-a-real-key")
        self.assertTrue(credentials.present(self.name))

    def test_deleting_removes_the_secret(self):
        credentials.store(self.name, "sk-not-a-real-key")
        credentials.delete(self.name)
        self.assertFalse(credentials.present(self.name))

    def test_a_missing_secret_is_not_an_error(self):
        self.assertFalse(credentials.present("parlo-nothing-stored"))

    def test_an_empty_secret_is_refused(self):
        with self.assertRaises(ValueError):
            credentials.store(self.name, "   ")

    @unittest.skipUnless(sys.platform == "win32", "the file layout is Windows only")
    def test_the_windows_file_does_not_carry_the_secret_in_clear(self):
        credentials.store(self.name, "sk-not-a-real-key")
        blob = (Path(self.folder) / f"{self.name}.bin").read_bytes()
        self.assertNotIn(b"sk-not-a-real-key", blob)
        self.assertTrue(blob)


if __name__ == "__main__":
    unittest.main()
