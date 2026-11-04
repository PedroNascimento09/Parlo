import subprocess
import sys
import tempfile
import textwrap
import threading
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade import tts as tts_module
from parlo_cascade.tts import PiperVoice, VoiceUnavailable

ADAPTER = textwrap.dedent(
    """
    import base64, json, sys
    print(json.dumps({"op": "hello", "engine": "piper", "available": True}), flush=True)
    for line in sys.stdin:
        request = json.loads(line)
        if request.get("op") == "synthesize":
            audio = base64.b64encode(bytes([1, 0]) * 4).decode("ascii")
            print(json.dumps({"op": "synthesized", "pcm16": audio, "sample_rate": 22050}), flush=True)
    """
)

ONCE = textwrap.dedent(
    """
    import base64, json, sys
    print(json.dumps({"op": "hello", "engine": "piper", "available": True}), flush=True)
    for line in sys.stdin:
        request = json.loads(line)
        if request.get("op") == "synthesize":
            audio = base64.b64encode(bytes([1, 0]) * 4).decode("ascii")
            print(json.dumps({"op": "synthesized", "pcm16": audio, "sample_rate": 22050}), flush=True)
            sys.exit(1)
    """
)

CRASHING = textwrap.dedent(
    """
    import json, sys
    print(json.dumps({"op": "hello", "engine": "piper", "available": True}), flush=True)
    for line in sys.stdin:
        sys.stderr.write("onnxruntime: bad allocation\\n")
        sys.stderr.flush()
        sys.exit(3)
    """
)

UNAVAILABLE = textwrap.dedent(
    """
    import json
    print(json.dumps({"op": "hello", "available": False, "error": "piper is not installed"}), flush=True)
    """
)


class AdapterTest(unittest.TestCase):
    def make(self, script, voice=True):
        directory = Path(tempfile.mkdtemp())
        (directory / "serve.py").write_text(script, encoding="utf-8")
        model = directory / "voice.onnx"
        if voice:
            model.write_bytes(b"")
        return directory, model

    def test_missing_adapter_is_reported(self):
        directory, model = self.make(ADAPTER)
        voice = PiperVoice(model, directory=directory / "absent")
        with self.assertRaises(VoiceUnavailable) as raised:
            voice.synthesize("bom dia")
        self.assertIn("piper adapter is not installed", str(raised.exception))

    def test_missing_voice_is_reported(self):
        directory, model = self.make(ADAPTER, voice=False)
        voice = PiperVoice(model, directory=directory)
        with self.assertRaises(VoiceUnavailable) as raised:
            voice.synthesize("bom dia")
        self.assertIn("missing voice model", str(raised.exception))

    def test_unavailable_adapter_is_reported(self):
        directory, model = self.make(UNAVAILABLE)
        voice = PiperVoice(model, directory=directory)
        with self.assertRaises(VoiceUnavailable) as raised:
            voice.synthesize("bom dia")
        self.assertIn("piper is not installed", str(raised.exception))

    def test_synthesis_roundtrip(self):
        directory, model = self.make(ADAPTER)
        voice = PiperVoice(model, directory=directory)
        pcm16, rate = voice.synthesize("bom dia")
        self.assertEqual(rate, 22050)
        self.assertEqual(pcm16, bytes([1, 0]) * 4)
        voice.close()

    def test_a_dead_adapter_is_replaced_for_the_next_sentence(self):
        directory, model = self.make(ONCE)
        voice = PiperVoice(model, directory=directory)
        first, _ = voice.synthesize("bom dia")
        second, _ = voice.synthesize("boa tarde")
        self.assertEqual(first, bytes([1, 0]) * 4)
        self.assertEqual(second, bytes([1, 0]) * 4)
        voice.close()

    def test_a_dead_adapter_reports_its_last_words(self):
        directory, model = self.make(CRASHING)
        voice = PiperVoice(model, directory=directory)
        with self.assertRaises(VoiceUnavailable) as raised:
            voice.synthesize("bom dia")
        self.assertIn("bad allocation", str(raised.exception))

    def test_installed_requires_both_files(self):
        directory, model = self.make(ADAPTER)
        self.assertTrue(PiperVoice(model, directory=directory).installed())
        self.assertFalse(PiperVoice(model, directory=directory / "absent").installed())


class ConcurrentStartTest(unittest.TestCase):
    def test_two_threads_start_one_adapter(self):
        directory = Path(tempfile.mkdtemp())
        (directory / "serve.py").write_text(ADAPTER, encoding="utf-8")
        model = directory / "voice.onnx"
        model.write_bytes(b"")
        voice = PiperVoice(model, directory=directory)
        self.addCleanup(voice.close)

        real = subprocess.Popen
        spawned = []
        ready = threading.Barrier(2)

        def slow(*arguments, **keywords):
            spawned.append(arguments)
            time.sleep(0.2)
            return real(*arguments, **keywords)

        def boot():
            ready.wait()
            voice.start()

        with mock.patch.object(tts_module.subprocess, "Popen", slow):
            threads = [threading.Thread(target=boot) for _ in range(2)]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()

        self.assertEqual(
            len(spawned),
            1,
            f"the piper adapter started {len(spawned)} processes for one voice",
        )


if __name__ == "__main__":
    unittest.main()
