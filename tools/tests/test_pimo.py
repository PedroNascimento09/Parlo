import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from pimo import compat, hardware, manifest, session, signing


def model(**overrides):
    entry = {
        "manifest_version": 1,
        "id": "parakeet-tdt-0.6b-v3",
        "name": "Parakeet TDT 0.6B v3",
        "version": "3",
        "task": "asr",
        "state": "recommended",
        "evidence": "validated",
        "origin": "NVIDIA",
        "url": "https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3",
        "checksum": "sha256:" + "a" * 64,
        "license": {
            "code": "Apache-2.0",
            "weights": "CC-BY-4.0",
            "weights_verified": True,
            "attribution": "NVIDIA Parakeet TDT 0.6B v3",
        },
        "languages": {"declared": ["pt-PT", "en"], "validated": ["pt-PT", "en"]},
        "runtime": {"engine": "sherpa-onnx", "min_version": "1.12", "formats": ["onnx"]},
        "platforms": {"os": ["windows", "macos", "linux"], "arch": ["x86_64", "arm64"], "backends": ["cpu"]},
        "resources": {"ram_gb": 2.0, "disk_gb": 0.7, "vram_gb": 0},
        "api": {"min_schema": 1, "max_schema": 2},
        "capabilities": {"streaming": False},
        "limits": ["reprocesses audio in blocks instead of streaming"],
        "benchmarks": {},
        "maintenance": "active",
    }
    entry.update(overrides)
    return entry


def device(**overrides):
    data = {
        "os": "windows",
        "arch": "x86_64",
        "ram_available_gb": 8.0,
        "disk_free_gb": 50.0,
        "vram_gb": 0.0,
        "backends": ["cpu"],
    }
    data.update(overrides)
    return data


class ManifestTests(unittest.TestCase):
    def test_valid_manifest_passes(self):
        self.assertEqual(manifest.validate(model()), [])

    def test_automatic_state_requires_validated_evidence(self):
        entry = model(evidence="documented")
        self.assertTrue(manifest.validate(entry))
        self.assertFalse(manifest.is_auto_eligible(entry))

    def test_missing_field_is_reported(self):
        entry = model()
        del entry["checksum"]
        self.assertIn("checksum: missing", manifest.validate(entry))

    def test_checksum_must_be_sha256(self):
        self.assertTrue(manifest.validate(model(checksum="sha256:abc")))

    def test_validated_language_must_be_declared(self):
        entry = model(languages={"declared": ["pt-PT"], "validated": ["pt-BR"]})
        self.assertTrue(any("not declared" in problem for problem in manifest.validate(entry)))

    def test_unverified_weights_block_automatic_use(self):
        entry = model()
        entry["license"] = dict(entry["license"], weights_verified=False)
        self.assertFalse(manifest.is_auto_eligible(entry))


class CompatTests(unittest.TestCase):
    def test_admits_compatible_model(self):
        self.assertEqual(compat.admit(model(), device()), (True, None))

    def test_rejects_other_system(self):
        entry = model(platforms={"os": ["windows"], "arch": ["x86_64"], "backends": ["cpu"]})
        self.assertEqual(compat.admit(entry, device(os="linux")), (False, "os"))

    def test_rejects_insufficient_memory(self):
        entry = model(resources={"ram_gb": 32.0, "disk_gb": 1.0, "vram_gb": 0})
        self.assertEqual(compat.admit(entry, device()), (False, "ram"))

    def test_rejects_schema_outside_range(self):
        entry = model(api={"min_schema": 2, "max_schema": 3})
        self.assertEqual(compat.admit(entry, device(), schema=1), (False, "api"))

    def test_rejects_unvalidated_locale(self):
        entry = model(languages={"declared": ["pt-PT", "pt-BR"], "validated": ["pt-PT"]})
        self.assertEqual(compat.admit(entry, device(), locale="pt-BR"), (False, "locale"))

    def test_rejects_experimental_state(self):
        entry = model(state="experimental", evidence="experimental")
        self.assertEqual(compat.admit(entry, device()), (False, "state:experimental"))

    def test_rejects_unknown_backend(self):
        entry = model(platforms={"os": ["windows"], "arch": ["x86_64"], "backends": ["cuda"]})
        self.assertEqual(compat.admit(entry, device()), (False, "backend"))

    def test_rank_splits_candidates(self):
        catalogue = [
            model(),
            model(id="heavy-model", resources={"ram_gb": 64.0, "disk_gb": 1.0, "vram_gb": 0}),
        ]
        result = compat.rank(catalogue, device())
        self.assertEqual(result["admitted"], ["parakeet-tdt-0.6b-v3"])
        self.assertEqual(result["rejected"], [{"id": "heavy-model", "reason": "ram"}])


class SigningTests(unittest.TestCase):
    def test_signature_verifies_only_the_signed_payload(self):
        with tempfile.TemporaryDirectory() as folder:
            key = Path(folder) / "key.txt"
            public = signing.generate(key)
            signature = signing.sign({"models": []}, key)
            self.assertTrue(signing.verify({"models": []}, signature, public))
            self.assertFalse(signing.verify({"models": [model()]}, signature, public))

    def test_file_round_trip_detects_tampering(self):
        with tempfile.TemporaryDirectory() as folder:
            key = Path(folder) / "key.txt"
            public = signing.generate(key)
            path = Path(folder) / "catalogue.json"
            path.write_text(json.dumps({"models": [model()]}), encoding="utf-8")
            target = signing.sign_file(path, key)
            self.assertTrue(target.exists())
            self.assertTrue(signing.verify_file(path, public))
            path.write_text(json.dumps({"models": [model(limits=["changed after signing"])]}), encoding="utf-8")
            self.assertFalse(signing.verify_file(path, public))

    def test_rejects_another_key(self):
        with tempfile.TemporaryDirectory() as folder:
            key = Path(folder) / "key.txt"
            signing.generate(key)
            other = Path(folder) / "other.txt"
            other_public = signing.generate(other)
            signature = signing.sign({"models": []}, key)
            self.assertFalse(signing.verify({"models": []}, signature, other_public))

    def test_private_key_is_not_world_readable(self):
        with tempfile.TemporaryDirectory() as folder:
            key = Path(folder) / "key.txt"
            signing.generate(key)
            mode = key.stat().st_mode & 0o777
            self.assertIn(mode, (0o600, 0o666))


class CatalogTests(unittest.TestCase):
    def test_shipped_manifests_are_valid(self):
        root = Path(__file__).resolve().parents[2]
        paths = sorted((root / "catalog" / "manifests").glob("*.json"))
        self.assertTrue(paths)
        for path in paths:
            with self.subTest(manifest=path.name):
                self.assertEqual(manifest.validate(manifest.load(path)), [])

    def test_every_shipped_checksum_matches_a_file_entry(self):
        root = Path(__file__).resolve().parents[2]
        for path in sorted((root / "catalog" / "manifests").glob("*.json")):
            entry = manifest.load(path)
            if "files" in entry:
                with self.subTest(manifest=path.name):
                    self.assertIn(entry["checksum"][7:], [item["sha256"] for item in entry["files"]])

    def test_bad_file_hash_is_reported(self):
        entry = model(files=[{"name": "encoder.onnx", "sha256": "abc"}])
        self.assertTrue(any(problem.startswith("files.sha256") for problem in manifest.validate(entry)))

    def test_single_manifest_loads_as_one_entry(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "model.json"
            path.write_text(json.dumps(model()), encoding="utf-8")
            self.assertEqual(len(manifest.entries(manifest.load(path))), 1)

    def test_licence_index_is_not_a_manifest(self):
        with self.assertRaises(manifest.ManifestError):
            manifest.entries({"schema": 1, "artifacts": []})


class LiveTests(unittest.TestCase):
    def catalogue(self):
        return [
            model(id="mt-en", task="mt", languages={"declared": ["en"], "validated": ["en"]}),
            model(
                id="clone-en",
                task="clone",
                languages={"declared": ["en"], "validated": ["en"]},
                benchmarks={"identity": 0.81},
            ),
        ]

    def speakers(self):
        return [
            {"id": "a", "locale": "pt-PT", "profile": True, "authorised": True},
            {"id": "b", "locale": "en"},
        ]

    def test_resolves_both_directions(self):
        result = session.live(self.catalogue(), device(), self.speakers())
        forward = result["directions"][0]
        self.assertEqual((forward["from"], forward["to"]), ("a", "b"))
        self.assertEqual(forward["translation"]["id"], "mt-en")
        self.assertEqual(forward["voice"]["id"], "clone-en")
        self.assertEqual(forward["identity"], "measured")

    def test_missing_clone_refuses_to_promise_identity(self):
        catalogue = [model(id="mt-en", task="mt", languages={"declared": ["en"], "validated": ["en"]})]
        result = session.live(catalogue, device(), self.speakers())
        forward = result["directions"][0]
        self.assertIsNone(forward["voice"]["id"])
        self.assertEqual(forward["identity"], "unavailable")
        self.assertIn("cannot be promised", result["notice"])

    def test_unauthorised_profile_is_not_used(self):
        speakers = [
            {"id": "a", "locale": "pt-PT", "profile": True, "authorised": False},
            {"id": "b", "locale": "en"},
        ]
        forward = session.live(self.catalogue(), device(), speakers)["directions"][0]
        self.assertIsNone(forward["voice"]["id"])
        self.assertIn("not authorised", forward["voice"]["reason"])

    def test_same_locale_needs_no_translation(self):
        speakers = [
            {"id": "a", "locale": "pt-PT"},
            {"id": "b", "locale": "pt-PT"},
        ]
        result = session.live(self.catalogue(), device(), speakers)
        self.assertEqual(result["directions"][0]["identity"], "not applicable")
        self.assertIn("same locale", result["notice"])

    def test_two_candidates_are_listed_not_chosen(self):
        catalogue = self.catalogue() + [
            model(id="mt-en-b", task="mt", languages={"declared": ["en"], "validated": ["en"]})
        ]
        translation = session.live(catalogue, device(), self.speakers())["directions"][0]["translation"]
        self.assertIsNone(translation["id"])
        self.assertEqual(translation["candidates"], ["mt-en", "mt-en-b"])

    def test_unmeasured_clone_is_reported_as_such(self):
        catalogue = [
            model(id="mt-en", task="mt", languages={"declared": ["en"], "validated": ["en"]}),
            model(id="clone-en", task="clone", languages={"declared": ["en"], "validated": ["en"]}),
        ]
        result = session.live(catalogue, device(), self.speakers())
        self.assertEqual(result["directions"][0]["identity"], "unmeasured")
        self.assertIn("not measured", result["notice"])

    def test_one_speaker_is_rejected(self):
        with self.assertRaises(session.LiveError):
            session.live(self.catalogue(), device(), [{"id": "a", "locale": "pt-PT"}])

    def test_duplicate_speaker_is_rejected(self):
        speakers = [
            {"id": "a", "locale": "pt-PT"},
            {"id": "a", "locale": "en"},
        ]
        with self.assertRaises(session.LiveError):
            session.live(self.catalogue(), device(), speakers)


class HardwareTests(unittest.TestCase):
    def test_profile_reports_measurements(self):
        data = hardware.profile()
        for field in ("os", "arch", "ram_total_gb", "ram_available_gb", "disk_free_gb", "backends"):
            self.assertIn(field, data)
        self.assertGreaterEqual(data["ram_total_gb"], data["ram_available_gb"])
        self.assertIn("cpu", data["backends"])


if __name__ == "__main__":
    unittest.main()
