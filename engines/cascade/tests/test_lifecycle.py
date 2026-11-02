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
        engine.touched = {unit: time.time() for unit in engine.loaded_units()}
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)

    def test_an_idle_engine_drops_every_model(self):
        self.write({"unload": 10})
        engine = self.engine_with_models()
        engine.touched = {unit: time.time() - 11 * 60 for unit in engine.loaded_units()}
        dropped = engine.unload_if_idle()
        self.assertEqual(dropped, ("recognition", "translation", "dictation translation", "voice"))
        self.assertIsNone(engine._stt)
        self.assertIsNone(engine._tts)
        self.assertIsNone(engine._translator)
        self.assertEqual(engine._writers, {})

    def test_never_means_never(self):
        self.write({"unload": 0})
        engine = self.engine_with_models()
        engine.touched = {
            unit: time.time() - 365 * 24 * 60 * 60 for unit in engine.loaded_units()
        }
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)

    def test_each_model_follows_its_own_delay(self):
        self.write({"unload": 30, "policies": {"recognition": 5}})
        engine = self.engine_with_models()
        moment = time.time()
        engine.touched = {unit: moment - 6 * 60 for unit in engine.loaded_units()}
        self.assertEqual(engine.unload_if_idle(moment), ("recognition",))
        self.assertIsNone(engine._stt)
        self.assertIsNotNone(engine._translator)

    def test_turning_management_off_keeps_everything(self):
        self.write({"unload": 10, "manage_memory": False})
        engine = self.engine_with_models()
        engine.touched = {unit: 0.0 for unit in engine.loaded_units()}
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)

    def test_an_idle_model_without_a_recorded_use_is_left_alone(self):
        self.write({"unload": 10})
        engine = self.engine_with_models()
        self.assertEqual(engine.unload_if_idle(), ())
        self.assertIsNotNone(engine._stt)

    def test_the_report_names_each_model_its_delay_and_where_it_comes_from(self):
        self.write({"unload": 30, "policies": {"voice": 0}})
        engine = self.engine_with_models()
        engine.touched = {"recognition": time.time() - 90}
        report = engine.lifecycle()
        self.assertTrue(report["manage"])
        units = {unit["unit"]: unit for unit in report["units"]}
        self.assertEqual(units["recognition"]["state"], "loaded")
        self.assertEqual(units["recognition"]["idle_seconds"], 90)
        self.assertEqual(units["recognition"]["policy"], 30)
        self.assertEqual(units["recognition"]["policy_origin"], "global")
        self.assertEqual(units["voice"]["policy"], 0)
        self.assertEqual(units["voice"]["policy_origin"], "model")
        self.assertIsNone(units["recognition"].get("memory"))

    def test_unloading_one_model_leaves_the_others(self):
        engine = self.engine_with_models()
        self.assertTrue(engine.unload_unit("voice"))
        self.assertIsNone(engine._tts)
        self.assertIsNotNone(engine._stt)

    def test_an_unknown_model_is_refused(self):
        engine = self.engine_with_models()
        with self.assertRaises(ValueError):
            engine.unload_unit("nonsense")
        with self.assertRaises(ValueError):
            engine.load_unit("nonsense")

    def test_loading_a_model_that_is_held_records_its_use(self):
        engine = self.engine_with_models()
        engine.touched = {}
        engine.load_unit("translation")
        self.assertIn("translation", engine.touched)


if __name__ == "__main__":
    unittest.main()
