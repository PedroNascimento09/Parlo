import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade.corrections import CLEAN_MODES, clean


class ModeTests(unittest.TestCase):
    def test_original_keeps_every_word(self):
        text = "Quero comprar 2 bilhetes, quer dizer, 3 bilhetes."
        self.assertEqual(clean(text, "original").cleaned, text)

    def test_an_unknown_mode_is_refused(self):
        with self.assertRaises(ValueError):
            clean("texto", "magic")

    def test_the_three_modes_are_named(self):
        self.assertEqual(CLEAN_MODES, ("original", "light", "intelligent"))

    def test_the_original_survives_any_mode(self):
        text = "Marca para terça. Não, espera, para quarta."
        for mode in CLEAN_MODES:
            self.assertEqual(clean(text, mode).original, text)


class CorrectionTests(unittest.TestCase):
    def test_a_number_is_corrected(self):
        text = "Quero comprar 2 bilhetes para o cinema, quer dizer, 3 bilhetes para o cinema."
        self.assertEqual(clean(text).cleaned, "Quero comprar 3 bilhetes para o cinema.")

    def test_a_week_day_is_corrected_across_sentences(self):
        text = "Marca a reunião para terça-feira. Não, espera, para quarta-feira."
        self.assertEqual(clean(text).cleaned, "Marca a reunião para quarta-feira.")

    def test_a_name_is_corrected(self):
        text = "Envia ao João, afinal, ao Pedro."
        self.assertEqual(clean(text).cleaned, "Envia ao Pedro.")

    def test_a_destination_is_corrected(self):
        text = "Vou a Lisboa amanhã, ou melhor, ao Porto amanhã."
        self.assertEqual(clean(text).cleaned, "Vou ao Porto amanhã.")

    def test_a_correction_is_recorded_as_a_change(self):
        result = clean("Marca para terça. Não, espera, para quarta.")
        self.assertTrue(any(change.startswith("corrected") for change in result.changes))


class MeaningTests(unittest.TestCase):
    def test_a_clarification_keeps_both_ideas(self):
        text = "Gosto de cinema, quer dizer, gosto especialmente de filmes de ficção científica."
        cleaned = clean(text).cleaned
        self.assertIn("Gosto de cinema", cleaned)
        self.assertIn("ficção científica", cleaned)

    def test_a_new_topic_is_not_deleted(self):
        text = "Falamos do orçamento, aliás, tenho de te mostrar o novo escritório."
        cleaned = clean(text).cleaned
        self.assertIn("orçamento", cleaned)
        self.assertIn("escritório", cleaned)

    def test_a_change_of_intention_removes_only_the_marker(self):
        text = "Precisamos de mais tempo, quer dizer, precisamos de mais duas semanas."
        cleaned = clean(text).cleaned
        self.assertNotIn("quer dizer", cleaned)
        self.assertIn("duas semanas", cleaned)


class FalseStartTests(unittest.TestCase):
    def test_an_abandoned_start_is_dropped(self):
        text = "Amanhã vou... afinal, na sexta-feira vou ao escritório."
        self.assertEqual(clean(text).cleaned, "na sexta-feira vou ao escritório.")

    def test_a_truncated_sentence_is_reported(self):
        result = clean("Amanhã vou... afinal, sexta-feira.")
        self.assertTrue(any("false start" in change for change in result.changes))


class FillerTests(unittest.TestCase):
    def test_hesitations_are_removed(self):
        text = "Hmm, preciso de enviar o relatório, eh, hoje."
        self.assertEqual(clean(text).cleaned, "preciso de enviar o relatório, hoje.")

    def test_a_repeated_word_is_collapsed(self):
        text = "Vou vou enviar o relatório."
        self.assertEqual(clean(text).cleaned, "Vou enviar o relatório.")

    def test_a_meaningful_word_is_kept(self):
        text = "O pronto-socorro fica ali."
        self.assertIn("pronto-socorro", clean(text).cleaned)


class IntelligentTests(unittest.TestCase):
    def test_the_first_letter_is_capitalised_and_the_sentence_closed(self):
        self.assertEqual(
            clean("preciso de enviar o relatório", "intelligent").cleaned,
            "Preciso de enviar o relatório.",
        )

    def test_light_does_not_rewrite_the_text(self):
        text = "preciso de enviar o relatório"
        self.assertEqual(clean(text, "light").cleaned, text)

    def test_several_corrections_in_one_long_sentence(self):
        text = (
            "Amanhã vou... afinal, na sexta vou ao escritório, quer dizer, "
            "na sexta vou ao escritório do Porto."
        )
        self.assertEqual(clean(text).cleaned, "na sexta vou ao escritório do Porto.")


class EnglishTests(unittest.TestCase):
    def test_an_english_correction_is_handled(self):
        text = "Book two tickets, I mean, three tickets for the cinema."
        self.assertEqual(clean(text, language="en").cleaned, "three tickets for the cinema.")

    def test_an_english_filler_is_removed(self):
        self.assertEqual(clean("uh, send the report", language="en").cleaned, "send the report")


if __name__ == "__main__":
    unittest.main()
