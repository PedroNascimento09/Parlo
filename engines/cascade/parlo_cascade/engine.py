import importlib.util
import os
import sys
import re
import time
from dataclasses import dataclass
from pathlib import Path

from .models import models_dir
from .streaming import StreamingSession, TextEvent
from .tts import PiperVoice, VoiceUnavailable

SAMPLE_RATE = 16_000
MIN_SEGMENT_SECONDS = 2.5
SILENCE_RMS = 0.01
SILENCE_HOLD_SECONDS = 0.45
MODEL_DIR = models_dir()
REQUIRED_MODULES = ("numpy", "sherpa_onnx")
DEFAULT_THREADS = 4
STT_BACKENDS = ("parakeet", "whisper")
WHISPER_SIZES = ("tiny", "base", "small", "medium", "large-v3")
PARAKEET_DIR = MODEL_DIR / "parakeet"
PARAKEET_FILES = ("encoder.int8.onnx", "decoder.int8.onnx", "joiner.int8.onnx", "tokens.txt")
VOICES = {
    "pt": "pt_BR-faber-medium.onnx",
    "en": "en_US-lessac-medium.onnx",
}
GLOSSARY_FILE = Path(__file__).resolve().parent.parent / "glossary.txt"


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


def load_glossary(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}
    pairs = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        clean = line.strip()
        if not clean or clean.startswith("#") or "=" not in clean:
            continue
        wrong, _, right = clean.partition("=")
        if wrong.strip() and right.strip():
            pairs[wrong.strip()] = right.strip()
    return pairs


def apply_glossary(text: str, glossary: dict[str, str]) -> str:
    for wrong, right in glossary.items():
        text = re.sub(rf"\b{re.escape(wrong)}\b", right, text, flags=re.IGNORECASE)
    return text


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
    stream_mode = "reprocess"
    sample_rate = SAMPLE_RATE

    def __init__(self, source: str = "pt", target: str = "en", voice: Path | None = None):
        self.source = source
        self.target = target
        self.target_languages = (target,)
        self.min_segment = float(os.environ.get("PARLO_CASCADE_SEGMENT", MIN_SEGMENT_SECONDS))
        self.model_name = os.environ.get("PARLO_CASCADE_MODEL", "base")
        self.stt = os.environ.get("PARLO_CASCADE_STT", "parakeet").lower()
        if self.stt not in STT_BACKENDS:
            raise ValueError(f"unknown recogniser {self.stt}, expected {STT_BACKENDS}")
        if self.stt == "whisper" and self.model_name not in WHISPER_SIZES:
            raise ValueError(
                f"unknown whisper model {self.model_name}, expected {WHISPER_SIZES}"
            )
        threads = os.environ.get("PARLO_CASCADE_THREADS")
        self.threads = int(threads) if threads else DEFAULT_THREADS
        self.captions = os.environ.get("PARLO_CASCADE_CAPTIONS") == "1"
        self.dictate = os.environ.get("PARLO_CASCADE_DICTATE") == "1"
        self.glossary = load_glossary(GLOSSARY_FILE)
        self.voice_path = voice or MODEL_DIR / VOICES.get(target, VOICES["pt"])
        self._buffer = bytearray()
        self._ready = available()
        self._stt = None
        self._tts = None
        self._translator = None
        self._stream: StreamingSession | None = None
        self._silent_seconds = 0.0

    @property
    def available(self) -> bool:
        return self._ready

    def warmup(self) -> None:
        if not self._ready:
            return
        stages = [("recognition", self._load_stt)]
        if not self.dictate:
            stages.append(("translation", self._translate_warmup))
            if self.voice_path.exists():
                stages.append(("voice", self._synthesize_warmup))
        for label, stage in stages:
            try:
                stage()
            except Exception as error:
                print(f"warmup deferred for {label}: {error}", file=sys.stderr)

    def _translate_warmup(self) -> None:
        self._translate("warmup")

    def _synthesize_warmup(self) -> None:
        self._synthesize("warmup")

    def _load_stt(self):
        if self._stt is not None:
            return self._stt
        if self.stt == "parakeet":
            import sherpa_onnx

            missing = [
                name for name in PARAKEET_FILES if not (PARAKEET_DIR / name).is_file()
            ]
            if missing:
                raise EngineUnavailable(
                    f"missing parakeet files {missing}; run engines/cascade/scripts/setup_models.py"
                )
            self._stt = sherpa_onnx.OfflineRecognizer.from_transducer(
                encoder=str(PARAKEET_DIR / "encoder.int8.onnx"),
                decoder=str(PARAKEET_DIR / "decoder.int8.onnx"),
                joiner=str(PARAKEET_DIR / "joiner.int8.onnx"),
                tokens=str(PARAKEET_DIR / "tokens.txt"),
                num_threads=self.threads,
                model_type="nemo_transducer",
            )
        else:
            from faster_whisper import WhisperModel

            self._stt = WhisperModel(
                self.model_name,
                device="cpu",
                compute_type="int8",
                cpu_threads=self.threads,
            )
        return self._stt

    def translate(self, pcm16: bytes) -> Translation:
        if not self._ready:
            raise EngineUnavailable(
                "install the cascade requirements: pip install -r engines/cascade/requirements.txt"
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
        if self.dictate:
            elapsed = int((time.perf_counter() - started) * 1000)
            return Translation(pcm16=b"", text=text, latency_ms=elapsed)
        translated = self._translate(text)
        if self.captions:
            elapsed = int((time.perf_counter() - started) * 1000)
            return Translation(pcm16=b"", text=translated, latency_ms=elapsed)
        try:
            audio = self._synthesize(translated)
        except VoiceUnavailable as error:
            raise EngineUnavailable(str(error)) from error
        elapsed = int((time.perf_counter() - started) * 1000)
        return Translation(pcm16=audio, text=translated, latency_ms=elapsed)

    def transcribe(self, pcm16: bytes) -> str:
        if not self._ready:
            raise EngineUnavailable(
                "install the cascade requirements: pip install -r engines/cascade/requirements.txt"
            )
        samples = pcm16_to_float(pcm16)
        if not samples:
            return ""
        return self._transcribe(samples)

    def stream(self, pcm16: bytes) -> list[TextEvent]:
        if not self._ready:
            raise EngineUnavailable(
                "install the cascade requirements: pip install -r engines/cascade/requirements.txt"
            )
        samples = pcm16_to_float(pcm16)
        if self._stream is None:
            self._stream = StreamingSession(self._transcribe)
        events = self._stream.feed(samples)
        if rms(samples) < SILENCE_RMS:
            self._silent_seconds += len(samples) / SAMPLE_RATE
            if self._silent_seconds >= SILENCE_HOLD_SECONDS:
                events.extend(self._stream.close())
                self._silent_seconds = 0.0
        else:
            self._silent_seconds = 0.0
        return events

    def close_stream(self) -> list[TextEvent]:
        if self._stream is None:
            return []
        return self._stream.close()

    def _transcribe(self, samples) -> str:
        import numpy as np

        model = self._load_stt()
        if self.stt == "parakeet":
            stream = model.create_stream()
            stream.accept_waveform(SAMPLE_RATE, np.asarray(samples, dtype="float32"))
            model.decode_stream(stream)
            text = stream.result.text.strip()
        else:
            segments, _ = model.transcribe(
                np.asarray(samples, dtype="float32"),
                language=self.source,
                vad_filter=False,
                beam_size=1,
                condition_on_previous_text=False,
            )
            text = " ".join(segment.text.strip() for segment in segments).strip()
        return apply_glossary(text, self.glossary)

    def _translate(self, text: str) -> str:
        from . import mt

        if self._translator is None:
            self._translator = mt.from_environment(self.source, self.target)
            if self._translator is None:
                raise EngineUnavailable(
                    f"no translation model for {self.source}->{self.target}; "
                    "run engines/cascade/scripts/setup_models.py"
                )
        return self._translator(text)

    def _synthesize(self, text: str) -> bytes:
        if self._tts is None:
            self._tts = PiperVoice(self.voice_path)
        pcm16, rate = self._tts.synthesize(text)
        return float_to_pcm16(resample(pcm16_to_float(pcm16), rate, SAMPLE_RATE))
