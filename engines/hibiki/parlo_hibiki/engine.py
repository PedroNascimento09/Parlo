import importlib.util
from dataclasses import dataclass

SAMPLE_RATE = 16_000
TARGET_LANGUAGES = ("en",)
MODEL_ID = "kyutai/hibiki-zero"


class EngineUnavailable(RuntimeError):
    pass


@dataclass(frozen=True)
class Translation:
    pcm16: bytes
    text: str | None
    latency_ms: int


def torch_available() -> bool:
    return importlib.util.find_spec("torch") is not None


class HibikiEngine:
    id = "hibiki"
    sample_rate = SAMPLE_RATE
    target_languages = TARGET_LANGUAGES

    def __init__(self) -> None:
        self._available = torch_available()

    @property
    def available(self) -> bool:
        return self._available

    def translate(self, pcm16: bytes) -> Translation:
        if not self._available:
            raise EngineUnavailable(
                "torch is not installed; install the model extra from engines/hibiki"
            )
        raise EngineUnavailable(
            f"{MODEL_ID} weights are not wired up yet; see docs/ENGINES.md"
        )
