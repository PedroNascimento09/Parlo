import unittest

from parlo_cascade.engine import CascadeEngine


class Broken:
    def __call__(self, text):
        raise RuntimeError("the translation model is missing")


class Working:
    def __call__(self, text):
        return "Good morning, how are you?"


class WriteKeepsTheWordsTest(unittest.TestCase):
    def engine(self, translator, target="en"):
        engine = CascadeEngine.__new__(CascadeEngine)
        engine.source = "pt"
        engine.target = target
        engine.transcribe = lambda pcm16: "Bom dia, tudo bem?"
        engine._load_writer = lambda language: translator
        engine._touch = lambda name: None
        return engine

    def test_a_broken_translation_stage_keeps_the_words(self):
        text, translation = self.engine(Broken()).write(b"pcm")
        self.assertEqual(text, "Bom dia, tudo bem?")
        self.assertEqual(translation, "")

    def test_a_working_translation_stage_answers_in_the_other_language(self):
        text, translation = self.engine(Working()).write(b"pcm")
        self.assertEqual(text, "Bom dia, tudo bem?")
        self.assertEqual(translation, "Good morning, how are you?")

    def test_writing_the_same_language_needs_no_translator(self):
        def refuse(language):
            raise AssertionError("a translator was loaded for the same language")

        text, translation = self.engine(refuse, target="pt").write(b"pcm")
        self.assertEqual(text, "Bom dia, tudo bem?")
        self.assertEqual(translation, text)

    def test_silence_stays_empty(self):
        engine = self.engine(Working())
        engine.transcribe = lambda pcm16: ""
        self.assertEqual(engine.write(b"pcm"), ("", ""))


if __name__ == "__main__":
    unittest.main()
