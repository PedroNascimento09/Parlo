import time
from dataclasses import dataclass

SAMPLE_RATE = 16_000
TEXT_STATES = ("partial", "stable", "final", "corrected")
POLICY = "local-agreement"

FRAME_MS = 20
FRAME_SAMPLES = SAMPLE_RATE * FRAME_MS // 1000
SILENCE_RMS = 0.01


def rms(samples) -> float:
    if not samples:
        return 0.0
    total = 0.0
    for value in samples:
        total += value * value
    return (total / len(samples)) ** 0.5


def common_prefix(left, right) -> int:
    count = 0
    for first, second in zip(left, right):
        if first != second:
            break
        count += 1
    return count


def agreement(hypotheses: list[list[str]]) -> int:
    if not hypotheses:
        return 0
    count = 0
    for words in zip(*hypotheses):
        if len(set(words)) != 1:
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
    policy = POLICY

    def __init__(self, rounds: int = 2):
        if rounds < 2:
            raise ValueError("local agreement needs at least two hypotheses")
        self.rounds = rounds
        self.confirmed = 0
        self._confirmed_words: list[str] = []
        self._history: list[list[str]] = []

    def reset(self) -> None:
        self.confirmed = 0
        self._confirmed_words = []
        self._history = []

    def observe(self, words: list[str], audio_ms: int, latency_ms: int) -> list[TextEvent]:
        events = []
        previous = self._history[-1] if self._history else []
        text = " ".join(words)
        if text and text != " ".join(previous):
            events.append(TextEvent("partial", text, audio_ms, latency_ms))

        self._history.append(list(words))
        if len(self._history) > self.rounds:
            self._history.pop(0)

        shared = agreement(self._history)
        if len(self._history) == self.rounds and shared > self.confirmed:
            fresh = " ".join(self._history[-1][self.confirmed : shared])
            self.confirmed = shared
            self._confirmed_words = list(self._history[-1][:shared])
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


class VoiceActivity:
    """Deteta segmentos de fala por nível RMS com histerese."""

    def __init__(
        self,
        sample_rate: int = SAMPLE_RATE,
        threshold: float = 0.01,
        hangover_seconds: float = 0.4,
        pre_roll_seconds: float = 0.25,
        silence_seconds: float = 0.45,
    ):
        self.sample_rate = sample_rate
        self.threshold = threshold
        self.hangover_frames = int(hangover_seconds * 1000 / FRAME_MS)
        self.pre_roll_frames = int(pre_roll_seconds * 1000 / FRAME_MS)
        self.silence_seconds = silence_seconds
        self._ring: list[float] = []
        self._remaining = 0
        self._voiced = False

    def feed(self, samples) -> tuple[bool, list[float]]:
        """Devolve (is_voiced_now, pre_roll_samples) quando o segmento fecha."""
        self._ring.extend(samples)
        if len(self._ring) > FRAME_SAMPLES * (self.hangover_frames + self.pre_roll_frames):
            self._ring = self._ring[-FRAME_SAMPLES * (self.hangover_frames + self.pre_roll_frames) :]

        level = rms(samples)
        is_voiced = level >= self.threshold

        if is_voiced:
            self._remaining = self.hangover_frames
            self._voiced = True
        elif self._remaining > 0:
            self._remaining -= 1
        else:
            self._voiced = False

        return self._voiced, list(self._ring[: self.pre_roll_frames * FRAME_SAMPLES])


class StreamingSession:
    mode = "stream"

    def __init__(
        self,
        transcribe,
        sample_rate: int = SAMPLE_RATE,
        decode_every: float = 0.4,
        segment_seconds: float = 6.0,
        min_audio_seconds: float = 0.4,
        silence_seconds: float = 0.45,
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
        self.silence_seconds = silence_seconds
        self._silence_seconds = 0.0
        self._tail: list[float] = []
        self._speech = False
        self._fresh_speech = False
        self._last_words: list[str] = []
        self._last_decode_time = 0.0
        self._clock = clock
        self._vad = VoiceActivity(
            sample_rate=sample_rate,
            silence_seconds=silence_seconds,
        )

    @property
    def policy(self) -> str:
        return self._policy.policy

    @property
    def audio_seconds(self) -> float:
        return len(self._samples) / self.sample_rate

    @property
    def audio_ms(self) -> int:
        return int(round(self.audio_seconds * 1000))

    def feed(self, samples) -> list[TextEvent]:
        self._samples.extend(samples)
        self._track(samples)
        events = []

        if self.audio_seconds - self._decoded_seconds >= self.decode_every:
            events.extend(self._decode())
        if self.audio_seconds >= self.segment_seconds:
            events.extend(self._finalize())
        elif (
            self._speech
            and self.audio_seconds >= self.min_audio_seconds
            and self._silence_seconds >= self.silence_seconds
        ):
            events.extend(self._finalize())
        return events

    def _track(self, samples) -> None:
        frame_samples = self.sample_rate * FRAME_MS // 1000
        self._tail.extend(samples)
        frames = len(self._tail) // frame_samples
        for index in range(frames):
            frame = self._tail[index * frame_samples : (index + 1) * frame_samples]
            if rms(frame) >= SILENCE_RMS:
                self._speech = True
                self._fresh_speech = True
                self._silence_seconds = 0.0
            else:
                self._silence_seconds += FRAME_MS / 1000
        if frames:
            del self._tail[: frames * frame_samples]

    def close(self) -> list[TextEvent]:
        if self.audio_seconds < self.min_audio_seconds:
            return []
        return self._finalize()

    def reset(self) -> None:
        self._samples = []
        self._decoded_seconds = 0.0
        self._silence_seconds = 0.0
        self._tail = []
        self._speech = False
        self._fresh_speech = False
        self._last_words = []
        self._policy.reset()

    def _decode(self) -> list[TextEvent]:
        words, latency = self._listen()
        return self._policy.observe(words, self.audio_ms, latency)

    def _finalize(self) -> list[TextEvent]:
        if self._last_words and not self._fresh_speech:
            events = self._policy.finalize(self._last_words, self.audio_ms, 0)
        else:
            words, latency = self._listen()
            events = self._policy.finalize(words, self.audio_ms, latency)
        self.reset()
        return events

    def _listen(self) -> tuple[list[str], int]:
        started = self._clock()
        words = self.transcribe(list(self._samples)).split()
        latency = int((self._clock() - started) * 1000)
        self._decoded_seconds = self.audio_seconds
        self._fresh_speech = False
        self._last_words = words
        return words, latency
