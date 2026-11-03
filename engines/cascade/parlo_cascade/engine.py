import importlib.util
import json
import os
import sys
import re
import time
from dataclasses import dataclass
from pathlib import Path

from .corrections import CLEAN_MODES, clean
from .intelligence import Intelligence, config_path, settings_from_config
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
    "pt": "pt_PT-tugão-medium.onnx",
    "en": "en_US-lessac-medium.onnx",
    "de": "de_DE-thorsten-medium.onnx",
    "fr": "fr_FR-siwis-medium.onnx",
}
GLOSSARY_FILE = Path(__file__).resolve().parent.parent / "glossary.txt"
UNLOAD_DEFAULT_MINUTES = 30
UNLOAD_LIMIT_MINUTES = 1_440
UNITS = ("recognition", "translation", "dictation translation", "voice")


class EngineUnavailable(RuntimeError):
    pass


@dataclass(frozen=True)
class Translation:
    pcm16: bytes
    text: str | None
    latency_ms: int


def available() -> bool:
    return all(importlib.util.find_spec(name) is not None for name in REQUIRED_MODULES)


def configured_unload() -> int:
    minutes = None
    try:
        payload = json.loads(config_path().read_text(encoding="utf-8"))
        minutes = payload.get("unload") if isinstance(payload, dict) else None
    except (OSError, ValueError):
        minutes = None
    if isinstance(minutes, bool) or not isinstance(minutes, int):
        from_environment = os.environ.get("PARLO_UNLOAD_MINUTES")
        try:
            minutes = int(from_environment) if from_environment else UNLOAD_DEFAULT_MINUTES
        except ValueError:
            minutes = UNLOAD_DEFAULT_MINUTES
    return max(0, min(minutes, UNLOAD_LIMIT_MINUTES))


def policy_table() -> tuple[int, dict[str, int], bool]:
    policies: dict[str, int] = {}
    manage = True
    try:
        payload = json.loads(config_path().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        payload = {}
    if isinstance(payload, dict):
        stored = payload.get("policies")
        if isinstance(stored, dict):
            for unit, minutes in stored.items():
                if (
                    unit in UNITS
                    and isinstance(minutes, int)
                    and not isinstance(minutes, bool)
                ):
                    policies[unit] = max(0, min(minutes, UNLOAD_LIMIT_MINUTES))
        if isinstance(payload.get("manage_memory"), bool):
            manage = payload["manage_memory"]
    return configured_unload(), policies, manage


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
        self.clean_mode = os.environ.get("PARLO_CASCADE_CLEAN", "light").lower()
        if self.clean_mode not in CLEAN_MODES:
            raise ValueError(
                f"unknown cleanup mode {self.clean_mode}, expected {CLEAN_MODES}"
            )
        self.last_original = ""
        self.last_engine = ""
        self.touched: dict[str, float] = {}
        self.intelligence = Intelligence(settings_from_config(self.clean_mode), source)
        self.glossary = load_glossary(GLOSSARY_FILE)
        self.voice_path = voice or MODEL_DIR / VOICES.get(target, VOICES["pt"])
        self._buffer = bytearray()
        self._ready = available()
        self._stt = None
        self._tts = None
        self._translator = None
        self._writers: dict[str, object] = {}
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

    def loaded_units(self) -> tuple[str, ...]:
        units = []
        if self._stt is not None:
            units.append("recognition")
        if self._translator is not None:
            units.append("translation")
        if self._writers:
            units.append("dictation translation")
        if self._tts is not None:
            units.append("voice")
        return tuple(units)

    def _touch(self, unit: str) -> None:
        self.touched[unit] = time.time()

    def policy(self, unit: str) -> tuple[int, str]:
        default, policies, _ = policy_table()
        if unit in policies:
            return policies[unit], "model"
        return default, "global"

    def lifecycle(self, now: float | None = None) -> dict:
        moment = time.time() if now is None else now
        _, _, manage = policy_table()
        loaded = set(self.loaded_units())
        units = []
        for unit in UNITS:
            minutes, origin = self.policy(unit)
            used = self.touched.get(unit)
            units.append(
                {
                    "unit": unit,
                    "state": "loaded" if unit in loaded else "unloaded",
                    "idle_seconds": None if used is None else max(0, int(moment - used)),
                    "policy": minutes,
                    "policy_origin": origin,
                }
            )
        return {"manage": manage, "units": units}

    def load_unit(self, unit: str) -> None:
        if unit == "recognition":
            self._load_stt()
        elif unit == "translation":
            self._load_translation()
        elif unit == "dictation translation":
            self._load_writer(self.target)
        elif unit == "voice":
            self._load_voice()
        else:
            raise ValueError(f"unknown unit {unit}")
        self._touch(unit)

    def unload_unit(self, unit: str) -> bool:
        if unit == "recognition":
            held = self._stt is not None
            self._stt = None
        elif unit == "translation":
            held = self._translator is not None
            self._translator = None
        elif unit == "dictation translation":
            held = bool(self._writers)
            self._writers.clear()
        elif unit == "voice":
            held = self._tts is not None
            self._tts = None
        else:
            raise ValueError(f"unknown unit {unit}")
        self.touched.pop(unit, None)
        return held

    def unload_if_idle(self, now: float | None = None) -> tuple[str, ...]:
        _, _, manage = policy_table()
        if not manage:
            return ()
        moment = time.time() if now is None else now
        dropped = []
        for unit in self.loaded_units():
            minutes, _ = self.policy(unit)
            if minutes <= 0:
                continue
            used = self.touched.get(unit)
            if used is None or moment - used < minutes * 60:
                continue
            if self.unload_unit(unit):
                dropped.append(unit)
        return tuple(dropped)

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
            self.intelligence = Intelligence(
                settings_from_config(self.clean_mode), self.source
            )
            edit = self.intelligence.edit(text)
            self.last_original = edit.original
            self.last_engine = edit.engine
            elapsed = int((time.perf_counter() - started) * 1000)
            return Translation(pcm16=b"", text=edit.text, latency_ms=elapsed)
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

    def write(
        self, pcm16: bytes, target: str | None = None, clean_mode: str | None = None
    ) -> tuple[str, str]:
        self.last_warning = ""
        text = self.transcribe(pcm16)
        if not text:
            return "", ""
        mode = clean_mode if clean_mode in CLEAN_MODES else self.clean_mode
        if mode in CLEAN_MODES:
            self.intelligence = Intelligence(settings_from_config(mode), self.source)
            edit = self.intelligence.edit(text)
            self.last_original = edit.original
            self.last_engine = edit.engine
            text = edit.text
            if not text:
                return "", ""
        language = (target or self.target).lower()
        if language == self.source:
            return text, text
        try:
            translator = self._load_writer(language)
            self._touch("dictation translation")
            return text, translator(text)
        except Exception as failure:
            print(f"translation failed, the words were kept: {failure}", flush=True)
            return text, ""

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

    def _load_translation(self):
        from . import mt

        if self._translator is None:
            translator = mt.from_environment(self.source, self.target)
            if translator is None:
                raise EngineUnavailable(
                    f"no translation model for {self.source}->{self.target}; "
                    "run engines/cascade/scripts/setup_models.py"
                )
            self._translator = translator
        return self._translator

    def _load_writer(self, language: str):
        from . import mt

        translator = self._writers.get(language)
        if translator is None:
            translator = mt.from_environment(self.source, language)
            if translator is None:
                raise EngineUnavailable(
                    f"no translation model for {self.source}->{language}; "
                    "run engines/cascade/scripts/setup_models.py"
                )
            self._writers[language] = translator
        return translator

    def _load_voice(self):
        if self._tts is None:
            if not self.voice_path.is_file():
                raise EngineUnavailable(f"the voice {self.voice_path.name} is not installed")
            self._tts = PiperVoice(self.voice_path)
        return self._tts

    def _transcribe(self, samples) -> str:
        import numpy as np

        model = self._load_stt()
        self._touch("recognition")
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
        self._touch("translation")
        return self._load_translation()(text)

    def _synthesize(self, text: str) -> bytes:
        self._touch("voice")
        pcm16, rate = self._load_voice().synthesize(text)
        return float_to_pcm16(resample(pcm16_to_float(pcm16), rate, SAMPLE_RATE))
