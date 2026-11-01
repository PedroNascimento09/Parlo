import time
from dataclasses import dataclass

SAMPLE_RATE = 16_000
TEXT_STATES = ("partial", "stable", "final", "corrected")


def common_prefix(left, right) -> int:
    count = 0
    for first, second in zip(left, right):
        if first != second:
            break
        count += 1
    return count


@dataclass(frozen=True)
class TextEvent:
    state: str
    text: str
    audio_ms: int
    latency_ms: int

    def __post_init__(self):
        if self.state not in TEXT_STATES:
            raise ValueError(f"unknown text state {self.state}, expected {TEXT_STATES}")

    def as_dict(self) -> dict:
        return {
            "state": self.state,
            "text": self.text,
            "audio_ms": self.audio_ms,
            "latency_ms": self.latency_ms,
        }


class StabilityPolicy:
    def __init__(self, rounds: int = 2):
        if rounds < 1:
            raise ValueError("rounds must be at least 1")
        self.rounds = rounds
        self.confirmed = 0
        self._confirmed_words: list[str] = []
        self._previous: list[str] = []
        self._candidate = 0
        self._candidate_rounds = 0

    def reset(self) -> None:
        self.confirmed = 0
        self._confirmed_words = []
        self._previous = []
        self._candidate = 0
        self._candidate_rounds = 0

    def observe(self, words: list[str], audio_ms: int, latency_ms: int) -> list[TextEvent]:
        text = " ".join(words)
        events = []
        if text and text != " ".join(self._previous):
            events.append(TextEvent("partial", text, audio_ms, latency_ms))
        overlap = common_prefix(self._previous, words)
        self._previous = list(words)

        if self.confirmed and words[: self.confirmed] != self._confirmed_words:
            self._confirmed_words = words[: self.confirmed]
            self._candidate = self.confirmed
            self._candidate_rounds = 0

        if overlap <= self.confirmed:
            self._candidate = self.confirmed
            self._candidate_rounds = 0
            return events

        if overlap > self._candidate:
            self._candidate = overlap
            self._candidate_rounds = 1
        elif overlap == self._candidate:
            self._candidate_rounds += 1
        else:
            self._candidate = overlap
            self._candidate_rounds = 1

        if self._candidate_rounds >= self.rounds:
            fresh = " ".join(words[self.confirmed : self._candidate])
            self.confirmed = self._candidate
            self._confirmed_words = list(words[: self.confirmed])
            self._candidate_rounds = 0
            events.append(TextEvent("stable", fresh, audio_ms, latency_ms))
        return events

    def finalize(self, words: list[str], audio_ms: int, latency_ms: int) -> list[TextEvent]:
        text = " ".join(words).strip()
        if not text:
            return []
        events = []
        if self.confirmed and words[: self.confirmed] != self._confirmed_words:
            events.append(TextEvent("corrected", text, audio_ms, latency_ms))
        events.append(TextEvent("final", text, audio_ms, latency_ms))
        return events


class StreamingSession:
    mode = "reprocess"

    def __init__(
        self,
        transcribe,
        sample_rate: int = SAMPLE_RATE,
        decode_every: float = 0.4,
        segment_seconds: float = 12.0,
        min_audio_seconds: float = 0.6,
        rounds: int = 2,
        clock=time.perf_counter,
    ):
        if decode_every <= 0:
            raise ValueError("decode_every must be positive")
        if segment_seconds <= 0:
            raise ValueError("segment_seconds must be positive")
        self.transcribe = transcribe
        self.sample_rate = sample_rate
        self.decode_every = decode_every
        self.segment_seconds = segment_seconds
        self.min_audio_seconds = min_audio_seconds
        self._policy = StabilityPolicy(rounds)
        self._samples: list[float] = []
        self._decoded_seconds = 0.0
        self._clock = clock

    @property
    def audio_seconds(self) -> float:
        return len(self._samples) / self.sample_rate

    @property
    def audio_ms(self) -> int:
        return int(round(self.audio_seconds * 1000))

    def feed(self, samples) -> list[TextEvent]:
        self._samples.extend(samples)
        events = []
        if (
            self.audio_seconds >= self.min_audio_seconds
            and self.audio_seconds - self._decoded_seconds >= self.decode_every
        ):
            events.extend(self._decode())
        if self.audio_seconds >= self.segment_seconds:
            events.extend(self._finalize())
        return events

    def close(self) -> list[TextEvent]:
        if self.audio_seconds < self.min_audio_seconds:
            return []
        return self._finalize()

    def reset(self) -> None:
        self._samples = []
        self._decoded_seconds = 0.0
        self._policy.reset()

    def _decode(self) -> list[TextEvent]:
        started = self._clock()
        words = self.transcribe(list(self._samples)).split()
        latency = int((self._clock() - started) * 1000)
        self._decoded_seconds = self.audio_seconds
        return self._policy.observe(words, self.audio_ms, latency)

    def _finalize(self) -> list[TextEvent]:
        started = self._clock()
        words = self.transcribe(list(self._samples)).split()
        latency = int((self._clock() - started) * 1000)
        events = self._policy.finalize(words, self.audio_ms, latency)
        self.reset()
        return events
