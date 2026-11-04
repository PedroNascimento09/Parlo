import json
import os
import tempfile
import unittest
from pathlib import Path

from parlo_cascade import vocabulary


class VocabularyTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        previous = os.environ.get("PARLO_DATA_DIR")
        os.environ["PARLO_DATA_DIR"] = self.folder.name

        def restore():
            if previous is None:
                os.environ.pop("PARLO_DATA_DIR", None)
            else:
                os.environ["PARLO_DATA_DIR"] = previous

        self.addCleanup(restore)

    def test_an_absent_profile_changes_nothing(self):
        text, changes = vocabulary.apply("Vou ao Porto amanhã.")
        self.assertEqual(text, "Vou ao Porto amanhã.")
        self.assertEqual(changes, ())

    def test_a_term_is_kept_and_applied_on_whole_words(self):
        vocabulary.add("term", "parlo", "Parlo")
        text, changes = vocabulary.apply("o parlo abriu e o parlophone tocou")
        self.assertEqual(text, "o Parlo abriu e o parlophone tocou")
        self.assertEqual(changes, ("term: parlo → Parlo",))

    def test_an_abbreviation_expands_and_keeps_the_capital(self):
        vocabulary.add("abbreviation", "qr", "quarta-feira")
        self.assertEqual(vocabulary.apply("a reunião é na QR")[0], "a reunião é na Quarta-feira")
        self.assertEqual(vocabulary.apply("a reunião é na qr")[0], "a reunião é na quarta-feira")

    def test_the_profile_survives_on_disk(self):
        vocabulary.add("term", "jorge", "Jorge Silva")
        vocabulary.add("abbreviation", "vc", "você")
        stored = json.loads(Path(self.folder.name, "vocabulary.json").read_text(encoding="utf-8"))
        self.assertEqual(stored["terms"], {"jorge": "Jorge Silva"})
        self.assertEqual(stored["abbreviations"], {"vc": "você"})
        self.assertEqual(vocabulary.load()["terms"]["jorge"], "Jorge Silva")

    def test_a_word_can_be_removed(self):
        vocabulary.add("term", "parlo", "Parlo")
        vocabulary.discard("term", "parlo")
        self.assertEqual(vocabulary.load()["terms"], {})
        self.assertEqual(vocabulary.apply("o parlo")[0], "o parlo")

    def test_an_empty_word_is_refused(self):
        with self.assertRaises(ValueError):
            vocabulary.add("term", "  ", "Parlo")
        with self.assertRaises(ValueError):
            vocabulary.add("abbreviation", "qr", " ")

    def test_an_unknown_kind_is_refused(self):
        with self.assertRaises(ValueError):
            vocabulary.add("nickname", "x", "y")

    def test_a_broken_file_is_treated_as_an_empty_profile(self):
        Path(self.folder.name, "vocabulary.json").write_text("not json", encoding="utf-8")
        self.assertEqual(vocabulary.load(), {"terms": {}, "abbreviations": {}, "snippets": {}})
        self.assertEqual(vocabulary.apply("nada muda")[0], "nada muda")

    def test_a_snippet_fills_the_date_and_the_name(self):
        Path(self.folder.name, "config.json").write_text('{"name": "Ana"}', encoding="utf-8")
        vocabulary.add("snippet", "my sign off", "Best, {{name}} — {{date}} at {{time}}")
        text = vocabulary.apply("my sign off")[0]
        self.assertIn("Best, Ana — ", text)
        self.assertNotIn("{{", text)

    def test_a_snippet_without_a_name_drops_it(self):
        vocabulary.add("snippet", "my sign off", "Best, {{name}}")
        self.assertEqual(vocabulary.apply("my sign off")[0], "Best, ")

    def test_a_term_keeps_its_braces(self):
        vocabulary.add("term", "parlo", "{{Parlo}}")
        self.assertEqual(vocabulary.apply("o parlo")[0], "o {{Parlo}}")

    def test_a_list_round_trips_through_a_file(self):
        vocabulary.add("term", "jorge", "Jorge Silva")
        vocabulary.add("snippet", "my calendar", "https://cal.com/example")
        text = vocabulary.export_text()
        vocabulary.discard("term", "jorge")
        self.assertIn("term\tjorge\tJorge Silva", text)
        saved = vocabulary.import_text(text)
        self.assertEqual(saved["added"], 2)
        self.assertEqual(vocabulary.load()["terms"]["jorge"], "Jorge Silva")

    def test_an_import_refuses_a_line_without_three_parts(self):
        with self.assertRaises(ValueError):
            vocabulary.import_text("term\tjorge")

    def test_an_empty_import_is_refused(self):
        with self.assertRaises(ValueError):
            vocabulary.import_text("   ")

    def test_a_regular_expression_is_not_a_word(self):
        vocabulary.add("term", "a.b", "AB")
        self.assertEqual(vocabulary.apply("a.b e axb")[0], "AB e axb")


if __name__ == "__main__":
    unittest.main()
