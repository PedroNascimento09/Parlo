import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import serve
from parlo_cascade.engine import EngineUnavailable


class FailingEngine:
    def translate(self, pcm16):
        raise EngineUnavailable("the piper adapter stopped without answering")


class QuietEngine:
    pass


class AnswerTest(unittest.TestCase):
    def test_a_failing_request_answers_with_an_error(self):
        answer = serve.answer(
            '{"op": "translate", "seq": 7, "pcm16": "AAAAAA=="}', FailingEngine()
        )
        self.assertEqual(answer["op"], "error")
        self.assertEqual(answer["seq"], 7)
        self.assertIn("piper adapter", answer["error"])

    def test_a_normal_request_still_goes_through(self):
        answer = serve.answer('{"op": "unknown"}', QuietEngine())
        self.assertEqual(answer["error"], "unknown op: unknown")


if __name__ == "__main__":
    unittest.main()
