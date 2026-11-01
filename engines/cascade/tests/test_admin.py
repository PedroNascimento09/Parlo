import json
import os
import tempfile
import unittest
from pathlib import Path

from parlo_cascade import admin
from parlo_cascade.intelligence import PROVIDERS, Settings, choose, settings_from_config

VARIABLES = (
    "PARLO_CONFIG",
    "PARLO_INTELLIGENCE_MODE",
    "PARLO_INTELLIGENCE_PROVIDER",
    "PARLO_INTELLIGENCE_CLOUD",
    "PARLO_INTELLIGENCE_LOCAL_URL",
    "PARLO_CASCADE_CLEAN",
)


class SettingsFileTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.path = Path(self.folder.name) / "config.json"
        previous = {name: os.environ.get(name) for name in VARIABLES}
        for name in VARIABLES:
            os.environ.pop(name, None)
        os.environ["PARLO_CONFIG"] = str(self.path)

        def restore():
            for name, value in previous.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value

        self.addCleanup(restore)

    def write(self, payload):
        self.path.write_text(json.dumps(payload), encoding="utf-8")

    def test_a_missing_file_gives_the_documented_defaults(self):
        settings = settings_from_config()
        self.assertEqual(settings.mode, "auto")
        self.assertEqual(settings.clean, "light")
        self.assertFalse(settings.allow_cloud)
        self.assertEqual(settings.provider, "")

    def test_the_saved_section_is_read(self):
        self.write(
            {
                "intelligence": {
                    "mode": "cloud",
                    "clean": "intelligent",
                    "allow_cloud": True,
                    "provider": "anthropic",
                    "model": "claude-3-5-haiku-latest",
                }
            }
        )
        settings = settings_from_config()
        self.assertEqual(settings.mode, "cloud")
        self.assertEqual(settings.clean, "intelligent")
        self.assertTrue(settings.allow_cloud)
        self.assertEqual(settings.provider, "anthropic")
        self.assertEqual(settings.model, "claude-3-5-haiku-latest")

    def test_the_environment_wins_over_the_file(self):
        self.write({"intelligence": {"mode": "off", "provider": "openai"}})
        os.environ["PARLO_INTELLIGENCE_MODE"] = "local"
        os.environ["PARLO_INTELLIGENCE_PROVIDER"] = "gemini"
        settings = settings_from_config()
        self.assertEqual(settings.mode, "local")
        self.assertEqual(settings.provider, "gemini")

    def test_a_hand_edited_value_falls_back_instead_of_raising(self):
        self.write(
            {
                "intelligence": {
                    "mode": "turbo",
                    "clean": "aggressive",
                    "provider": "someone",
                    "model": 7,
                }
            }
        )
        settings = settings_from_config()
        self.assertEqual(settings.mode, "auto")
        self.assertEqual(settings.clean, "light")
        self.assertEqual(settings.provider, "")
        self.assertEqual(settings.model, "")

    def test_a_broken_file_is_treated_as_missing(self):
        self.path.write_text("not json", encoding="utf-8")
        self.assertEqual(settings_from_config().mode, "auto")

    def test_a_compatible_provider_without_an_address_stays_deterministic(self):
        settings = Settings(mode="cloud", allow_cloud=True, provider="compatible")
        route = choose(settings, key_present=True)
        self.assertEqual(route.engine, "deterministic")
        self.assertIn("base URL", route.reason)


class AdminTests(unittest.TestCase):
    def test_status_lists_every_provider(self):
        reply = admin.answer({"action": "status"})
        self.assertIn("store", reply)
        names = [provider["name"] for provider in reply["providers"]]
        self.assertEqual(names, list(PROVIDERS))
        self.assertTrue(all(isinstance(provider["key"], bool) for provider in reply["providers"]))

    def test_an_unknown_action_is_refused_without_a_trace(self):
        reply = admin.answer({"action": "forget"})
        self.assertIn("unknown action", reply["error"])

    def test_a_key_needs_a_provider_pardo_can_use(self):
        reply = admin.answer({"action": "store", "provider": "someone", "key": "secret"})
        self.assertIn("unknown provider", reply["error"])

    def test_an_empty_key_is_refused(self):
        reply = admin.answer({"action": "store", "provider": "openai", "key": "  "})
        self.assertIn("key is needed", reply["error"])


if __name__ == "__main__":
    unittest.main()
