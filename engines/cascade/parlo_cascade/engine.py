import importlib.util
import io
import os
import time
import wave
from dataclasses import dataclass
from pathlib import Path

SAMPLE_RATE = 16_000
MIN_SEGMENT_SECONDS = 2.5
SILENCE_RMS = 0.01
MODEL_DIR = Path(__file__).resolve().parent.parent / "models"
REQUIRED_MODULES = ("faster_whisper", "argostranslate", "piper", "numpy")
VOICES = {
    "pt": "pt_BR-faber-medium.onnx",
    "en": "en_US-lessac-medium.onnx",
}


class EngineUnavailable(RuntimeError):
    pass


@dataclass(frozen=True)
class Translation:
    pcm16: bytes
    text: str | None
    latency_ms: int


def available() -> bool:
    return all(importlib.util.find_spec(name) is not None for name in REQUIRED_MODULES)


def pcm16_to_float(pcm16: bytes) -> list[float]:
    usable = len(pcm16) - (len(pcm16) % 2)
    return [
        int.from_bytes(pcm16[index : index + 2], "little", signed=True) / 32_768.0
        for index in range(0, usable, 2)
    ]


def float_to_pcm16(samples) -> bytes:
    import numpy as np

    clipped = np.clip(np.asarray(samples, dtype="float32"), -1.0, 1.0)
    return (clipped * 32_767).astype("<i2").tobytes()


def rms(samples) -> float:
    if len(samples) == 0:
        return 0.0
    import numpy as np

    values = np.asarray(samples, dtype="float32")
    return float(np.sqrt(np.mean(values * values)))


def resample(samples, from_rate: int, to_rate: int):
    import numpy as np

    values = np.asarray(samples, dtype="float32")
    if from_rate == to_rate or values.size == 0:
        return values
    count = max(int(round(values.size * to_rate / from_rate)), 1)
    positions = np.linspace(0, values.size - 1, count)
    return np.interp(positions, np.arange(values.size), values).astype("float32")


class CascadeEngine:
    id = "cascade"
    sample_rate = SAMPLE_RATE

    def __init__(self, source: str = "pt", target: str = "en", voice: Path | None = None):
        self.source = source
        self.target = target
        self.target_languages = (target,)
        self.min_segment = float(os.environ.get("PARLO_CASCADE_SEGMENT", MIN_SEGMENT_SECONDS))
        self.voice_path = voice or MODEL_DIR / VOICES.get(target, VOICES["pt"])
        self._buffer = bytearray()
        self._ready = available()
        self._stt = None
        self._tts = None

    @property
    def available(self) -> bool:
        return self._ready

    def translate(self, pcm16: bytes) -> Translation:
        if not self._ready:
            raise EngineUnavailable(
                "install the cascade requirements: pip install -r engines/cascade/requirements.txt"
            )
        if not self.voice_path.exists():
            raise EngineUnavailable(
                f"missing voice model {self.voice_path.name}; run engines/cascade/scripts/setup_models.py"
            )

        self._buffer.extend(pcm16)
        buffered = pcm16_to_float(bytes(self._buffer))
        if len(buffered) < int(self.min_segment * SAMPLE_RATE):
            return Translation(b"", None, 0)

        self._buffer.clear()
        if rms(buffered) < SILENCE_RMS:
            return Translation(b"", None, 0)

        started = time.perf_counter()
        text = self._transcribe(buffered)
        if not text:
            return Translation(b"", None, 0)
        translated = self._translate(text)
        audio = self._synthesize(translated)
        elapsed = int((time.perf_counter() - started) * 1000)
        return Translation(pcm16=audio, text=translated, latency_ms=elapsed)

    def _transcribe(self, samples) -> str:
        import numpy as np
        from faster_whisper import WhisperModel

        if self._stt is None:
            self._stt = WhisperModel("small", device="cpu", compute_type="int8")
        segments, _ = self._stt.transcribe(
            np.asarray(samples, dtype="float32"),
            language=self.source,
            vad_filter=False,
            beam_size=1,
            condition_on_previous_text=False,
        )
        return " ".join(segment.text.strip() for segment in segments).strip()

    def _translate(self, text: str) -> str:
        import argostranslate.translate

        return argostranslate.translate.translate(text, self.source, self.target)

    def _synthesize(self, text: str) -> bytes:
        from piper import PiperVoice

        if self._tts is None:
            self._tts = PiperVoice.load(str(self.voice_path))

        buffer = io.BytesIO()
        with wave.open(buffer, "wb") as wav:
            self._tts.synthesize_wav(text, wav)
        buffer.seek(0)
        with wave.open(buffer, "rb") as wav:
            rate = wav.getframerate()
            channels = wav.getnchannels()
            frames = wav.readframes(wav.getnframes())

        import numpy as np

        samples = np.frombuffer(frames, dtype="<i2").astype("float32") / 32_768.0
        if channels > 1:
            samples = samples.reshape(-1, channels).mean(axis=1)
        return float_to_pcm16(resample(samples, rate, SAMPLE_RATE))
