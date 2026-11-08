import threading
from pathlib import Path

from dub_model import SAMPLE_RATE, Cue, Segment
from parlo_cascade import CascadeEngine
from parlo_cascade.engine import MODEL_DIR, VOICES
from parlo_cascade.tts import VoiceUnavailable


def warm_voice(engine: CascadeEngine) -> threading.Thread | None:
    load = getattr(engine, "_load_voice", None)
    voice = getattr(engine, "voice_path", None)
    if load is None or voice is None or not Path(voice).is_file():
        return None

    def boot() -> None:
        try:
            load().start()
        except VoiceUnavailable:
            pass

    thread = threading.Thread(target=boot, daemon=True)
    thread.start()
    return thread


def select_voice(
    engine: CascadeEngine, voice: str | None, default: Path | None
) -> Path | None:
    if not voice:
        if default is None:
            return None
        path = default
    else:
        name = VOICES.get(voice.removeprefix("voice-"))
        if name is None and (MODEL_DIR / voice).is_file():
            name = voice
        if name is None and (MODEL_DIR / f"{voice}.onnx").is_file():
            name = f"{voice}.onnx"
        if name is None:
            raise ValueError(f"{voice} is not a voice this engine installs")
        path = MODEL_DIR / name
        if not path.is_file():
            raise ValueError(f"the voice {path.name} is not installed")
    if Path(engine.voice_path) != path:
        engine._release_voice()
        engine.voice_path = path
    return path


def cue_pieces(cues: list[Cue], length: int) -> list[tuple[Segment, Cue]]:
    pieces = []
    for cue in cues:
        start = max(int(round(cue.start * SAMPLE_RATE)), 0)
        end = min(int(round(cue.end * SAMPLE_RATE)), length)
        if end > start:
            pieces.append((Segment(start, end), cue))
    if not pieces:
        raise SystemExit("none of the imported cues fall inside the audio")
    return pieces
