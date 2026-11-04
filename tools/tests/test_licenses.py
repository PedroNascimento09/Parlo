import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from licenses import Component, licence_problems, not_shipped_problems, policy, rust_components


class NotShippedTests(unittest.TestCase):
    def test_an_absent_crate_passes_every_target(self):
        graphs = {"x86_64-pc-windows-msvc": {"tauri", "getrandom"}, "x86_64-unknown-linux-gnu": {"tauri"}}
        self.assertEqual(not_shipped_problems({"r-efi": "reason"}, graphs), [])

    def test_a_crate_that_reaches_one_target_fails(self):
        graphs = {
            "x86_64-pc-windows-msvc": {"tauri"},
            "aarch64-apple-darwin": {"tauri", "r-efi"},
        }
        problems = not_shipped_problems({"r-efi": "reason"}, graphs)
        self.assertEqual(len(problems), 1)
        self.assertIn("aarch64-apple-darwin", problems[0])
        self.assertIn("no longer true", problems[0])

    def test_the_check_names_every_target_that_carries_it(self):
        graphs = {"a": {"r-efi"}, "b": {"r-efi"}, "c": set()}
        self.assertEqual(len(not_shipped_problems({"r-efi": "reason"}, graphs)), 2)

    def test_nothing_exempt_means_no_problems(self):
        self.assertEqual(not_shipped_problems({}, {"a": {"r-efi"}}), [])



class SqliteLicenceTests(unittest.TestCase):
    def test_the_sqlite_dependencies_are_accepted(self):
        components = [
            component
            for component in rust_components()
            if component.name in {"rusqlite", "libsqlite3-sys"}
        ]
        self.assertTrue(any(component.name == "rusqlite" for component in components))
        problems, _, _ = licence_problems(components, policy())
        self.assertEqual(problems, [])

    def test_a_copyleft_crate_is_rejected(self):
        component = Component(
            name="copyleft-fixture",
            version="1.0.0",
            license="GPL-3.0-only",
            scope="core",
            kind="rust",
            url="",
        )
        problems, _, _ = licence_problems([component], policy())
        self.assertEqual(len(problems), 1)
        self.assertIn("not allowed", problems[0])

    def test_a_crate_without_a_licence_is_rejected(self):
        component = Component(
            name="unknown-fixture",
            version="1.0.0",
            license="",
            scope="core",
            kind="rust",
            url="",
        )
        problems, _, _ = licence_problems([component], policy())
        self.assertEqual(len(problems), 1)
        self.assertIn("no licence metadata", problems[0])


if __name__ == "__main__":
    unittest.main()
