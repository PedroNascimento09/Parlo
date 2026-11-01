import os
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import mt, packs


class PackOrderTests(unittest.TestCase):
    def test_the_folder_parlo_installs_into_is_read_first(self):
        if "PARLO_MT_PACKS_DIR" in os.environ:
            self.skipTest("the override replaces the search order")
        roots = mt.pack_roots()
        self.assertEqual(roots[0], packs.packs_dir())
        self.assertEqual(len(roots), len(set(roots)))

    def test_the_override_replaces_every_other_folder(self):
        os.environ["PARLO_MT_PACKS_DIR"] = "somewhere"
        try:
            self.assertEqual(mt.pack_roots(), [Path("somewhere")])
        finally:
            del os.environ["PARLO_MT_PACKS_DIR"]

    def test_a_folder_that_is_not_a_translation_package_is_ignored(self):
        self.assertIsNone(mt._find_in(packs.packs_dir(), "zz", "zz"))


if __name__ == "__main__":
    unittest.main()
