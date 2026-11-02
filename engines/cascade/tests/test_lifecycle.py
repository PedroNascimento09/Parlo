import json
import os
import tempfile
import time
import unittest
from pathlib import Path

from parlo_cascade.engine import CascadeEngine, configured_unload


class UnloadPolicyTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.config = Path(self.folder.name) / "config.json"
        previous = {name: os.environ.get(name) for name in ("PARLO_CONFIG", "PARLO_UNLOAD_MINUTES")}
        os.environ["PARLO_CONFIG"] = str(self.config)
        os.environ.pop("PARLO_UNLOAD_MINUTES", None)

        def restore():
            for name, value in previous.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value

        self.addCleanup(restore)

    def write(self, payload):
        self.config.write_text(json.dumps(payload), encoding="utf-8")

    def engine_with_models(self):
        engine = CascadeEngine(source="pt", target="en")
        engine._stt = object()
        engine._tts = object()
        engine._translator = lambda text: text
        engine._writers["en"] = lambda text: text
        return engine

    def test_what_is_loaded_is_reported(self):
        engine = self.engine_with_models()
        self.assertEqual(
            engine.loaded_units(),
            ("recognition", "translation", "dictation translation", "voice"),
        )
        engine._stt = None
        engine._tts = None
        self.assertEqual(engine.loaded_units(), ("translation", "dictation translation"))

    def test_a_missing_setting_means_thirty_minutes(self):
        self.assertEqual(configured_unload(), 30)

    def test_the_setting_is_read_from_the_config_file(self):
        self.write({"unload": 10})
        self.assertEqual(configured_unload(), 10)

    def test_the_environment_wins_when_the_file_says_nothing(self):
        os.environ["PARLO_UNLOAD_MINUTES"] = "15"
        self.assertEqual(configured_unload(), 15)

    def test_a_nonsense_value_falls_back(self):
        self.write({"unload": "soon"})
        self.assertEqual(configured_unload(), 30)
        self.write({"unload": -5})
        self.assertEqual(configured_unload(), 0)
        self.write({"unload": 100_000})
        self.assertEqual(configured_unload(), 1_440)

    def test_a_model_in_use_is_not_dropped(self):
        self.write({"unload": 10})
        engine = self.engine_with_models()
        engine.used_at = time.time()
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)

    def test_an_idle_engine_drops_every_model(self):
        self.write({"unload": 10})
        engine = self.engine_with_models()
        engine.used_at = time.time() - 11 * 60
        dropped = engine.unload_if_idle()
        self.assertEqual(dropped, ("recognition", "translation", "dictation translation", "voice"))
        self.assertIsNone(engine._stt)
        self.assertIsNone(engine._tts)
        self.assertIsNone(engine._translator)
        self.assertEqual(engine._writers, {})

    def test_never_means_never(self):
        self.write({"unload": 0})
        engine = self.engine_with_models()
        engine.used_at = time.time() - 365 * 24 * 60 * 60
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)


if __name__ == "__main__":
    unittest.main()
