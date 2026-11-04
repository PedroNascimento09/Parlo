import base64
import json
import unittest

from parlo_hibiki import EngineUnavailable, HibikiEngine, Translation
from parlo_hibiki.protocol import handle_line


class FakeEngine:
    id = "fake"
    available = True
    sample_rate = 16_000
    target_languages = ("en",)

    def translate(self, pcm16):
        return Translation(pcm16=pcm16, text="hello", latency_ms=1_400)


class BrokenEngine(FakeEngine):
    def translate(self, pcm16):
        raise EngineUnavailable("model missing")


class ProtocolTest(unittest.TestCase):
    def test_hello_reports_engine_capabilities(self):
        response = handle_line('{"op": "hello"}', HibikiEngine())
        self.assertEqual(response["op"], "hello")
        self.assertEqual(response["engine"], "hibiki")
        self.assertEqual(response["target_languages"], ["en"])

    def test_unknown_op_is_reported(self):
        response = handle_line('{"op": "explode"}', HibikiEngine())
        self.assertEqual(response["op"], "error")
        self.assertIn("unknown op", response["error"])

    def test_invalid_json_is_reported(self):
        response = handle_line("{nope", HibikiEngine())
        self.assertEqual(response["error"], "invalid json")

    def test_missing_audio_is_reported(self):
        response = handle_line('{"op": "translate"}', FakeEngine())
        self.assertEqual(response["error"], "pcm16 must be base64 text")

    def test_invalid_base64_is_reported(self):
        response = handle_line('{"op": "translate", "pcm16": "***"}', FakeEngine())
        self.assertEqual(response["error"], "pcm16 is not valid base64")

    def test_odd_byte_count_is_reported(self):
        payload = base64.b64encode(b"\x00").decode()
        response = handle_line(f'{{"op": "translate", "pcm16": "{payload}"}}', FakeEngine())
        self.assertEqual(response["error"], "pcm16 must hold 16 bit samples")

    def test_translate_returns_audio(self):
        payload = base64.b64encode(b"\x00\x01" * 320).decode()
        response = handle_line(f'{{"op": "translate", "seq": 7, "pcm16": "{payload}"}}', FakeEngine())
        self.assertEqual(response["op"], "translated")
        self.assertEqual(response["seq"], 7)
        self.assertEqual(response["text"], "hello")
        self.assertEqual(base64.b64decode(response["pcm16"]), b"\x00\x01" * 320)

    def test_unavailable_engine_is_reported(self):
        payload = base64.b64encode(b"\x00\x00" * 320).decode()
        response = handle_line(
            f'{{"op": "translate", "pcm16": "{payload}"}}', BrokenEngine()
        )
        self.assertEqual(response["op"], "error")
        self.assertIn("model missing", response["error"])

    def test_hibiki_without_torch_is_honest(self):
        engine = HibikiEngine()
        if engine.available:
            self.skipTest("torch is installed in this environment")
        payload = base64.b64encode(b"\x00\x00" * 320).decode()
        response = handle_line(f'{{"op": "translate", "pcm16": "{payload}"}}', engine)
        self.assertEqual(response["op"], "error")
        self.assertIn("torch is not installed", response["error"])


class FakeDictationEngine(FakeEngine):
    def __init__(self):
        self.received = None

    def transcribe(self, pcm16):
        self.received = pcm16
        return "bom dia"


class DictationProtocolTest(unittest.TestCase):
    def test_dictate_returns_text_without_audio(self):
        payload = base64.b64encode(b"\x01\x00" * 160).decode()
        engine = FakeDictationEngine()
        response = handle_line(f'{{"op": "dictate", "seq": 9, "pcm16": "{payload}"}}', engine)
        self.assertEqual(response["op"], "dictated")
        self.assertEqual(response["seq"], 9)
        self.assertEqual(response["text"], "bom dia")
        self.assertNotIn("pcm16", response)
        self.assertEqual(engine.received, b"\x01\x00" * 160)

    def test_engine_that_cannot_transcribe_is_reported(self):
        response = handle_line('{"op": "dictate", "pcm16": ""}', FakeEngine())
        self.assertEqual(response["op"], "error")
        self.assertIn("cannot transcribe", response["error"])

    def test_dictate_rejects_a_missing_payload(self):
        response = handle_line('{"op": "dictate"}', FakeDictationEngine())
        self.assertEqual(response["error"], "pcm16 must be base64 text")

    def test_dictate_reports_an_unavailable_engine(self):
        payload = base64.b64encode(b"\x00\x00" * 160).decode()
        response = handle_line(
            f'{{"op": "dictate", "pcm16": "{payload}"}}', BrokenDictationEngine()
        )
        self.assertEqual(response["op"], "error")
        self.assertIn("model missing", response["error"])


class BrokenDictationEngine(FakeDictationEngine):
    def transcribe(self, pcm16):
        raise EngineUnavailable("model missing")


class FakeWritingEngine(FakeDictationEngine):
    def write(self, pcm16, target=None, clean=None):
        return "bom dia", "good morning" if target == "en" else "bom dia"


class WriteProtocolTest(unittest.TestCase):
    def payload(self):
        return base64.b64encode(b"\x00\x00" * 160).decode()

    def test_write_returns_the_sentence_and_its_translation(self):
        response = handle_line(
            f'{{"op": "write", "seq": 4, "pcm16": "{self.payload()}", "target": "en"}}',
            FakeWritingEngine(),
        )
        self.assertEqual(response["op"], "written")
        self.assertEqual(response["seq"], 4)
        self.assertEqual(response["text"], "bom dia")
        self.assertEqual(response["translation"], "good morning")

    def test_write_without_a_target_keeps_the_language_spoken(self):
        response = handle_line(
            f'{{"op": "write", "pcm16": "{self.payload()}"}}', FakeWritingEngine()
        )
        self.assertEqual(response["translation"], "bom dia")

    def test_write_needs_an_engine_that_can_translate(self):
        response = handle_line(
            f'{{"op": "write", "pcm16": "{self.payload()}"}}', FakeEngine()
        )
        self.assertEqual(response["op"], "error")
        self.assertIn("another language", response["error"])

    def test_write_rejects_a_missing_payload(self):
        response = handle_line('{"op": "write"}', FakeWritingEngine())
        self.assertEqual(response["error"], "pcm16 must be base64 text")


class FakeEvent:
    def __init__(self, state, text):
        self.state = state
        self.text = text
        self.audio_ms = 1000
        self.latency_ms = 80

    def as_dict(self):
        return {
            "state": self.state,
            "text": self.text,
            "audio_ms": self.audio_ms,
            "latency_ms": self.latency_ms,
        }


class FakeStreamingEngine(FakeEngine):
    stream_mode = "reprocess"

    def __init__(self):
        self.fed = []

    def stream(self, pcm16):
        self.fed.append(len(pcm16))
        return [FakeEvent("partial", "bom dia")]

    def close_stream(self):
        return [FakeEvent("final", "bom dia")]


class StreamProtocolTest(unittest.TestCase):
    def test_stream_returns_events_with_the_mode(self):
        payload = base64.b64encode(b"\x00\x00" * 160).decode()
        engine = FakeStreamingEngine()
        response = handle_line(f'{{"op": "stream", "seq": 3, "pcm16": "{payload}"}}', engine)
        self.assertEqual(response["op"], "stream")
        self.assertEqual(response["seq"], 3)
        self.assertEqual(response["mode"], "reprocess")
        self.assertEqual(response["events"][0]["state"], "partial")
        self.assertEqual(engine.fed, [320])

    def test_stream_close_returns_the_final_event(self):
        response = handle_line('{"op": "stream_close", "seq": 4}', FakeStreamingEngine())
        self.assertEqual(response["seq"], 4)
        self.assertEqual(response["events"][0]["state"], "final")

    def test_engine_without_streaming_is_reported(self):
        response = handle_line('{"op": "stream", "pcm16": ""}', FakeEngine())
        self.assertEqual(response["op"], "error")
        self.assertIn("does not stream", response["error"])

    def test_stream_rejects_a_missing_payload(self):
        response = handle_line('{"op": "stream"}', FakeStreamingEngine())
        self.assertEqual(response["error"], "pcm16 must be base64 text")


class FakePairEngine(FakeEngine):
    def __init__(self):
        self.pair = ("pt", "en")

    def set_pair(self, source, target):
        self.pair = (source.strip().lower(), target.strip().lower())
        self.target_languages = (self.pair[1],)


class PairProtocolTest(unittest.TestCase):
    def test_pair_switches_and_reports_the_new_target(self):
        engine = FakePairEngine()
        response = handle_line('{"op": "pair", "seq": 5, "from": "pt", "to": "de"}', engine)
        self.assertEqual(response["op"], "switched")
        self.assertEqual(response["seq"], 5)
        self.assertEqual(response["target_languages"], ["de"])
        self.assertEqual(engine.pair, ("pt", "de"))

    def test_pair_rejects_missing_languages(self):
        response = handle_line('{"op": "pair", "from": "pt"}', FakePairEngine())
        self.assertEqual(response["op"], "error")
        self.assertIn("must be text", response["error"])

    def test_an_engine_without_pairs_is_reported(self):
        response = handle_line('{"op": "pair", "from": "pt", "to": "en"}', FakeEngine())
        self.assertEqual(response["op"], "error")
        self.assertIn("one language pair", response["error"])

    def test_a_refusal_from_the_engine_is_reported(self):
        class Refusing(FakePairEngine):
            def set_pair(self, source, target):
                raise ValueError("no translation model")

        response = handle_line('{"op": "pair", "from": "pt", "to": "ja"}', Refusing())
        self.assertEqual(response["op"], "error")
        self.assertIn("no translation model", response["error"])


if __name__ == "__main__":
    unittest.main()
