import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parlo_cascade.streaming import (
    StabilityPolicy,
    StreamingSession,
    TextEvent,
    agreement,
    common_prefix,
)


class FakeClock:
    def __init__(self):
        self.now = 0.0

    def __call__(self):
        self.now += 0.01
        return self.now


class Scripted:
    def __init__(self, replies):
        self.replies = list(replies)

    def __call__(self, samples):
        if not self.replies:
            return ""
        return self.replies.pop(0)


class AgreementTest(unittest.TestCase):
    def test_counts_the_shared_start(self):
        self.assertEqual(common_prefix(["a", "b", "c"], ["a", "b", "d"]), 2)
        self.assertEqual(agreement([["a", "b"], ["a", "b", "c"]]), 2)

    def test_a_single_hypothesis_agrees_with_itself(self):
        self.assertEqual(agreement([["a", "b"]]), 2)

    def test_disagreement_stops_the_count(self):
        self.assertEqual(agreement([["a", "b"], ["a", "c"]]), 1)
        self.assertEqual(agreement([["a"], ["b"]]), 0)

    def test_no_hypotheses(self):
        self.assertEqual(agreement([]), 0)


class TextEventTest(unittest.TestCase):
    def test_rejects_an_unknown_state(self):
        with self.assertRaises(ValueError):
            TextEvent("guess", "text", 0, 0)

    def test_serialises(self):
        event = TextEvent("partial", "bom dia", 1200, 310)
        self.assertEqual(
            event.as_dict(),
            {"state": "partial", "text": "bom dia", "audio_ms": 1200, "latency_ms": 310},
        )


class StabilityPolicyTest(unittest.TestCase):
    def test_rejects_a_window_of_one(self):
        with self.assertRaises(ValueError):
            StabilityPolicy(rounds=1)

    def test_names_the_policy(self):
        self.assertEqual(StabilityPolicy().policy, "local-agreement")

    def test_reports_every_changed_hypothesis_as_partial(self):
        policy = StabilityPolicy()
        events = policy.observe(["bom"], 500, 40)
        self.assertEqual([event.state for event in events], ["partial"])

    def test_repeats_the_same_text_without_a_new_partial(self):
        policy = StabilityPolicy()
        policy.observe(["bom"], 500, 40)
        partials = [
            event for event in policy.observe(["bom"], 900, 40) if event.state == "partial"
        ]
        self.assertEqual(partials, [])

    def test_two_agreeing_hypotheses_confirm_the_prefix(self):
        policy = StabilityPolicy(rounds=2)
        self.assertEqual(
            policy.observe(["bom", "dia"], 500, 40),
            [TextEvent("partial", "bom dia", 500, 40)],
        )
        events = policy.observe(["bom", "dia"], 900, 40)
        self.assertEqual([event.text for event in events if event.state == "stable"], ["bom dia"])
        self.assertEqual(policy.confirmed, 2)

    def test_stable_text_is_emitted_once(self):
        policy = StabilityPolicy()
        policy.observe(["bom"], 500, 40)
        policy.observe(["bom"], 900, 40)
        self.assertEqual(policy.observe(["bom"], 1300, 40), [])

    def test_a_three_window_needs_three_hypotheses(self):
        policy = StabilityPolicy(rounds=3)
        policy.observe(["bom", "dia"], 500, 40)
        self.assertEqual([event.state for event in policy.observe(["bom", "dia"], 900, 40)], [])
        events = policy.observe(["bom", "dia"], 1300, 40)
        self.assertEqual([event.text for event in events if event.state == "stable"], ["bom dia"])

    def test_a_revision_does_not_confirm_new_words(self):
        policy = StabilityPolicy(rounds=2)
        policy.observe(["bom", "dia"], 500, 40)
        policy.observe(["bom", "dia"], 900, 40)
        events = policy.observe(["bom", "noite"], 1300, 40)
        self.assertEqual([event.state for event in events], ["partial"])
        self.assertEqual(policy.confirmed, 2)

    def test_growth_confirms_one_step_at_a_time(self):
        policy = StabilityPolicy(rounds=2)
        policy.observe(["bom"], 500, 40)
        first = policy.observe(["bom", "dia"], 900, 40)
        second = policy.observe(["bom", "dia"], 1300, 40)
        self.assertEqual([event.text for event in first if event.state == "stable"], ["bom"])
        self.assertEqual([event.text for event in second if event.state == "stable"], ["dia"])

    def test_final_is_always_reported(self):
        policy = StabilityPolicy()
        events = policy.finalize(["bom", "dia"], 2000, 60)
        self.assertEqual([event.state for event in events], ["final"])
        self.assertEqual(events[0].text, "bom dia")

    def test_final_reports_a_correction_when_confirmed_text_changed(self):
        policy = StabilityPolicy()
        policy.observe(["bom", "dia"], 500, 40)
        policy.observe(["bom", "dia"], 900, 40)
        events = policy.finalize(["boa", "noite"], 2000, 60)
        self.assertEqual([event.state for event in events], ["corrected", "final"])

    def test_a_final_matching_the_confirmed_text_is_not_a_correction(self):
        policy = StabilityPolicy()
        policy.observe(["bom", "dia"], 500, 40)
        policy.observe(["bom", "dia"], 900, 40)
        events = policy.finalize(["bom", "dia"], 2000, 60)
        self.assertEqual([event.state for event in events], ["final"])

    def test_silent_segment_produces_no_event(self):
        self.assertEqual(StabilityPolicy().finalize([], 2000, 60), [])

    def test_reset_clears_the_state(self):
        policy = StabilityPolicy()
        policy.observe(["bom"], 500, 40)
        policy.observe(["bom"], 900, 40)
        policy.reset()
        self.assertEqual(policy.confirmed, 0)
        self.assertEqual([event.state for event in policy.observe(["bom"], 500, 40)], ["partial"])


class StreamingSessionTest(unittest.TestCase):
    def session(self, replies, **kwargs):
        kwargs.setdefault("sample_rate", 1000)
        kwargs.setdefault("decode_every", 0.5)
        kwargs.setdefault("min_audio_seconds", 0.5)
        kwargs.setdefault("segment_seconds", 5.0)
        return StreamingSession(Scripted(replies), clock=FakeClock(), **kwargs)

    def test_waits_for_the_minimum_audio(self):
        session = self.session(["bom dia"])
        self.assertEqual(session.feed([0.0] * 300), [])

    def test_decodes_once_the_cadence_is_reached(self):
        session = self.session(["bom dia"])
        events = session.feed([0.0] * 500)
        self.assertEqual([event.state for event in events], ["partial"])

    def test_does_not_decode_twice_within_the_same_cadence(self):
        session = self.session(["bom dia", "bom dia"])
        session.feed([0.0] * 500)
        self.assertEqual(session.feed([0.0] * 400), [])
        self.assertEqual(session.audio_ms, 900)

    def test_close_returns_a_final_and_clears_the_segment(self):
        session = self.session(["bom dia", "bom dia"])
        session.feed([0.0] * 500)
        events = session.close()
        self.assertEqual([event.state for event in events], ["final"])
        self.assertEqual(session.audio_seconds, 0.0)

    def test_close_before_the_minimum_audio_returns_nothing(self):
        session = self.session(["bom dia"])
        session.feed([0.0] * 200)
        self.assertEqual(session.close(), [])
        self.assertEqual(session.audio_seconds, 0.2)

    def test_a_long_segment_closes_itself(self):
        session = self.session(["bom dia", "bom dia", "bom dia"], segment_seconds=1.0)
        events = session.feed([0.0] * 500)
        events += session.feed([0.0] * 500)
        self.assertIn("final", [event.state for event in events])
        self.assertEqual(session.audio_seconds, 0.0)

    def test_declares_reprocessing_and_the_policy(self):
        session = self.session(["bom dia"])
        self.assertEqual(session.mode, "stream")
        self.assertEqual(session.policy, "local-agreement")


if __name__ == "__main__":
    unittest.main()
