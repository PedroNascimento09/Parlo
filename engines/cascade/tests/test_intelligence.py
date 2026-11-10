import os
import tempfile
import unittest

from parlo_cascade import vocabulary
from parlo_cascade.intelligence import (
    CLOUD,
    DEADLINE,
    DETERMINISTIC,
    LOCAL,
    Intelligence,
    IntelligenceError,
    Settings,
    _json_text,
    choose,
    is_loopback,
    requires_secure_transport,
    settings_from_environment,
    validate,
)


def openai_reply(text):
    return {"choices": [{"message": {"content": '{"text": "' + text + '"}'}}]}


def unreachable(*args):
    raise AssertionError("the network was used")


class RoutingTests(unittest.TestCase):
    def test_off_never_leaves_the_machine(self):
        route = choose(Settings(mode="off", allow_cloud=True, provider="openai"), True)
        self.assertEqual(route.engine, DETERMINISTIC)

    def test_cloud_without_authorisation_is_refused(self):
        route = choose(Settings(mode="cloud", provider="openai"), True)
        self.assertEqual(route.engine, DETERMINISTIC)
        self.assertIn("not allowed", route.reason)

    def test_cloud_without_a_key_is_refused(self):
        route = choose(Settings(mode="cloud", allow_cloud=True, provider="openai"), False)
        self.assertEqual(route.engine, DETERMINISTIC)
        self.assertIn("API key", route.reason)

    def test_auto_uses_cloud_when_authorised_and_configured(self):
        route = choose(Settings(mode="auto", allow_cloud=True, provider="openai"), True)
        self.assertEqual(route.engine, CLOUD)
        self.assertEqual(route.model, "gpt-4o-mini")

    def test_a_local_model_wins_over_cloud(self):
        settings = Settings(
            mode="auto",
            allow_cloud=True,
            provider="openai",
            local_url="http://127.0.0.1:8080/v1",
        )
        self.assertEqual(choose(settings, True).engine, LOCAL)

    def test_a_plain_remote_connection_is_refused(self):
        self.assertTrue(is_loopback("http://127.0.0.1:8080/v1"))
        self.assertFalse(requires_secure_transport("https://api.openai.com/v1/chat/completions"))
        self.assertFalse(requires_secure_transport("http://localhost:8080/v1/chat/completions"))
        self.assertTrue(requires_secure_transport("http://models.example.com/v1"))

    def test_an_unknown_mode_is_refused(self):
        with self.assertRaises(ValueError):
            Settings(mode="best")

    def test_an_unknown_provider_is_refused(self):
        with self.assertRaises(ValueError):
            Settings(provider="someone")


class ValidationTests(unittest.TestCase):
    def test_a_conservative_edit_passes(self):
        self.assertTrue(validate("hmm, quero comprar 2 bilhetes.", "Quero comprar 2 bilhetes.").ok)

    def test_a_number_that_was_not_said_is_refused(self):
        verdict = validate("Custa vinte euros.", "Custa 20 euros, 25 com portes.")
        self.assertFalse(verdict.ok)
        self.assertIn("number", verdict.reason)

    def test_a_removed_or_repeated_number_is_refused(self):
        original = "Envia 3 livros e 2 revistas para o Pedro."
        for edited in (
            "Envia livros e 2 revistas para o Pedro.",
            "Envia 3 livros, 3 livros e 2 revistas para o Pedro.",
            "Envia 3 livros e 3 revistas para o Pedro.",
        ):
            with self.subTest(edited=edited):
                self.assertFalse(validate(original, edited).ok)
        self.assertTrue(validate(original, "Envia 2 revistas e 3 livros para o Pedro.").ok)

    def test_an_invented_name_is_refused(self):
        self.assertFalse(validate("Envia ao Pedro.", "Envia ao Pedro e ao João.").ok)

    def test_a_dropped_sentence_is_refused(self):
        self.assertFalse(validate("Levo o relatório e a apresentação para a reunião.", "Levo o relatório.").ok)

    def test_text_that_grew_is_refused(self):
        self.assertFalse(validate("Chego às nove.", "Chego às nove da manhã, como combinámos ontem à noite.").ok)

    def test_an_empty_answer_is_refused(self):
        self.assertFalse(validate("Olá.", "   ").ok)

    def test_a_free_form_answer_is_refused(self):
        with self.assertRaises(IntelligenceError):
            _json_text("Aqui está o texto: Olá.")

    def test_a_fenced_answer_is_refused(self):
        with self.assertRaises(IntelligenceError):
            _json_text('```json\n{"text": "Olá."}\n```')


class EditTests(unittest.TestCase):
    def test_off_still_cleans_the_text_locally(self):
        engine = Intelligence(Settings(mode="off"), "pt", transport=unreachable, key_lookup=lambda name: "")
        text = "Quero comprar 2 bilhetes para o cinema, quer dizer, 3 bilhetes para o cinema."
        edit = engine.edit(text)
        self.assertEqual(edit.text, "Quero comprar 3 bilhetes para o cinema.")
        self.assertEqual(edit.original, text)
        self.assertEqual(edit.engine, DETERMINISTIC)

    def test_a_personal_term_survives_the_cleanup_pass(self):
        with tempfile.TemporaryDirectory() as folder:
            previous = os.environ.get("PARLO_DATA_DIR")
            os.environ["PARLO_DATA_DIR"] = folder
            try:
                vocabulary.add("term", "parlo", "Parlo")
                engine = Intelligence(
                    Settings(mode="off"), "pt", transport=unreachable, key_lookup=lambda name: ""
                )
                edit = engine.edit("o parlo abriu hoje, quer dizer, ontem")
            finally:
                if previous is None:
                    os.environ.pop("PARLO_DATA_DIR", None)
                else:
                    os.environ["PARLO_DATA_DIR"] = previous
        self.assertIn("Parlo", edit.text)
        self.assertIn("term: parlo → Parlo", edit.changes)

    def test_cloud_edits_when_authorised(self):
        calls = []

        def transport(url, headers, payload, timeout):
            calls.append((url, headers, payload))
            return openai_reply("Quero comprar 3 bilhetes para o cinema.")

        settings = Settings(mode="cloud", allow_cloud=True, provider="openai")
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "sk-test")
        edit = engine.edit("quero comprar 3 bilhetes para o cinema")
        self.assertEqual(edit.engine, CLOUD)
        self.assertEqual(edit.text, "Quero comprar 3 bilhetes para o cinema.")
        self.assertEqual(edit.provider, "openai")
        self.assertEqual(calls[0][1]["Authorization"], "Bearer sk-test")
        self.assertGreaterEqual(edit.latency_ms, 0)

    def test_cloud_is_not_called_without_authorisation(self):
        settings = Settings(mode="auto", provider="openai", allow_cloud=False)
        engine = Intelligence(settings, "pt", transport=unreachable, key_lookup=lambda name: "sk-test")
        edit = engine.edit("preciso de enviar o relatório")
        self.assertEqual(edit.engine, DETERMINISTIC)

    def test_a_refused_key_falls_back_silently(self):
        def transport(*args):
            raise IntelligenceError("401 the provider refused the API key")

        settings = Settings(mode="cloud", allow_cloud=True, provider="openai")
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "sk-test")
        edit = engine.edit("preciso de enviar o relatório")
        self.assertTrue(edit.fallback)
        self.assertEqual(edit.engine, DETERMINISTIC)
        self.assertEqual(edit.text, "preciso de enviar o relatório")
        self.assertNotIn("sk-test", edit.reason)

    def test_an_invented_fact_is_not_accepted(self):
        def transport(*args):
            return openai_reply("Marca a reunião para quinta-feira, 12 de dezembro.")

        settings = Settings(mode="cloud", allow_cloud=True, provider="openai")
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "sk-test")
        edit = engine.edit("marca a reunião para quinta-feira")
        self.assertTrue(edit.fallback)
        self.assertIn("number", edit.reason)

    def test_a_dropped_number_keeps_the_deterministic_text(self):
        def transport(*args):
            return openai_reply("Envia os livros e 2 revistas para o Pedro.")

        settings = Settings(mode="cloud", allow_cloud=True, provider="openai")
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "sk-test")
        edit = engine.edit("Envia 3 livros e 2 revistas para o Pedro.")
        self.assertTrue(edit.fallback)
        self.assertEqual(edit.text, "Envia 3 livros e 2 revistas para o Pedro.")
        self.assertIn("number", edit.reason)

    def test_a_local_model_is_used_over_loopback(self):
        seen = []

        def transport(url, headers, payload, timeout):
            seen.append(url)
            return openai_reply("Olá.")

        settings = Settings(mode="auto", local_url="http://127.0.0.1:8080/v1/chat/completions")
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "")
        edit = engine.edit("olá")
        self.assertEqual(edit.engine, LOCAL)
        self.assertEqual(seen, ["http://127.0.0.1:8080/v1/chat/completions"])

    def test_a_slow_model_is_asked_within_the_deadline(self):
        waited = []

        def transport(url, headers, payload, timeout):
            waited.append(timeout)
            raise IntelligenceError("the provider took too long")

        settings = Settings(mode="cloud", allow_cloud=True, provider="openai", deadline=0.8)
        engine = Intelligence(settings, "pt", transport=transport, key_lookup=lambda name: "sk-test")
        edit = engine.edit("preciso de enviar o relatório")
        self.assertEqual(waited, [0.8])
        self.assertTrue(edit.fallback)
        self.assertEqual(edit.engine, DETERMINISTIC)

    def test_a_local_model_outside_loopback_is_refused(self):
        for url in ("http://models.example.com/v1", "https://models.example.com/v1"):
            with self.subTest(url=url):
                settings = Settings(mode="auto", local_url=url)
                engine = Intelligence(settings, "pt", transport=unreachable, key_lookup=lambda name: "")
                edit = engine.edit("olá")
                self.assertTrue(edit.fallback)
                self.assertIn("loopback", edit.reason)


class SettingsTests(unittest.TestCase):
    def test_environment_defaults_to_automatic_without_cloud(self):
        for name in ("PARLO_INTELLIGENCE_MODE", "PARLO_INTELLIGENCE_CLOUD", "PARLO_INTELLIGENCE_PROVIDER"):
            os.environ.pop(name, None)
        settings = settings_from_environment()
        self.assertEqual(settings.mode, "auto")
        self.assertFalse(settings.allow_cloud)
        self.assertEqual(settings.provider, "")

    def test_environment_can_authorise_cloud(self):
        os.environ["PARLO_INTELLIGENCE_MODE"] = "cloud"
        os.environ["PARLO_INTELLIGENCE_CLOUD"] = "1"
        os.environ["PARLO_INTELLIGENCE_PROVIDER"] = "anthropic"
        try:
            settings = settings_from_environment("intelligent")
            self.assertEqual(settings.mode, "cloud")
            self.assertTrue(settings.allow_cloud)
            self.assertEqual(settings.provider, "anthropic")
            self.assertEqual(settings.clean, "intelligent")
        finally:
            for name in ("PARLO_INTELLIGENCE_MODE", "PARLO_INTELLIGENCE_CLOUD", "PARLO_INTELLIGENCE_PROVIDER"):
                os.environ.pop(name, None)


    def test_the_deadline_is_read_from_the_environment(self):
        os.environ["PARLO_INTELLIGENCE_DEADLINE_MS"] = "900"
        try:
            self.assertEqual(settings_from_environment().deadline, 0.9)
        finally:
            os.environ.pop("PARLO_INTELLIGENCE_DEADLINE_MS", None)

    def test_a_nonsense_deadline_falls_back(self):
        os.environ["PARLO_INTELLIGENCE_DEADLINE_MS"] = "soon"
        try:
            self.assertEqual(settings_from_environment().deadline, DEADLINE)
        finally:
            os.environ.pop("PARLO_INTELLIGENCE_DEADLINE_MS", None)

    def test_a_deadline_outside_the_floor_and_ceiling_is_clamped(self):
        os.environ["PARLO_INTELLIGENCE_DEADLINE_MS"] = "60000"
        try:
            self.assertEqual(settings_from_environment().deadline, 5.0)
        finally:
            os.environ.pop("PARLO_INTELLIGENCE_DEADLINE_MS", None)


if __name__ == "__main__":
    unittest.main()
