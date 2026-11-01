import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

import mt_gate


class NameTests(unittest.TestCase):
    def test_a_name_in_the_middle_is_found(self):
        self.assertIn("maria", mt_gate.names("O João e a Maria chegaram."))

    def test_the_first_word_counts_when_the_source_names_it(self):
        source = mt_gate.names("O João e a Maria chegaram.")
        self.assertIn("joão", mt_gate.names("João und Maria kommen zu spät.", source))

    def test_a_first_word_that_is_not_a_source_name_is_ignored(self):
        self.assertEqual(mt_gate.names("Marquei a reunião."), set())

    def test_a_week_day_is_not_a_name(self):
        self.assertNotIn("quinta-feira", mt_gate.names("Marcado para quinta-feira."))

    def test_an_acronym_is_not_a_name(self):
        self.assertNotIn("iva", mt_gate.names("O IVA sobe."))


class NumberTests(unittest.TestCase):
    def test_digits_and_words_share_one_representation(self):
        self.assertEqual(mt_gate.values("12 dias"), mt_gate.values("doze dias"))

    def test_german_numerals_are_read(self):
        self.assertEqual(mt_gate.values("zwei Stunden"), mt_gate.values("2 Stunden"))

    def test_a_thousands_separator_is_not_a_decimal_point(self):
        self.assertIn("1250.0000", mt_gate.values("1250 euros"))
        self.assertIn("1250.0000", mt_gate.values("1.250 euros"))


class ClockTests(unittest.TestCase):
    def test_a_portuguese_clock_matches_a_german_one(self):
        self.assertTrue(mt_gate.clock_ok("às 15h30", "um 15:30 Uhr"))

    def test_a_lost_clock_is_reported(self):
        self.assertFalse(mt_gate.clock_ok("às 15h30", "am Nachmittag"))


class NegationTests(unittest.TestCase):
    def test_german_negation_is_detected(self):
        self.assertTrue(mt_gate.NEGATION["de"].search("Wir haben keine Zeit."))

    def test_a_negation_in_the_source_needs_one_in_the_output(self):
        row = {"text": "Não recebi o relatório.", "reference": "x", "source": "pt-PT", "target": "de"}
        self.assertFalse(mt_gate.check(row, "Ich habe den Bericht erhalten.")["negation"])
        self.assertTrue(mt_gate.check(row, "Ich habe den Bericht nicht erhalten.")["negation"])


if __name__ == "__main__":
    unittest.main()
