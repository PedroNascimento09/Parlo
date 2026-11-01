import base64
import json
import os
import subprocess
import sys
from pathlib import Path


class VoiceUnavailable(RuntimeError):
    pass


def adapter_dir() -> Path:
    override = os.environ.get("PARLO_PIPER_DIR")
    if override:
        return Path(override)
    return Path(__file__).resolve().parent.parent.parent / "tts-piper"


class PiperVoice:
    engine = "piper"

    def __init__(
        self,
        voice: Path,
        python: str | None = None,
        directory: Path | None = None,
    ):
        self.voice = Path(voice).expanduser().absolute()
        self.directory = Path(directory) if directory else adapter_dir()
        self.python = python or os.environ.get("PARLO_PIPER_PYTHON") or sys.executable
        self._process: subprocess.Popen | None = None

    @property
    def script(self) -> Path:
        return self.directory / "serve.py"

    def installed(self) -> bool:
        return self.script.is_file() and self.voice.is_file()

    def start(self) -> subprocess.Popen:
        if self._process is not None:
            return self._process
        if not self.script.is_file():
            raise VoiceUnavailable(
                f"the piper adapter is not installed at {self.directory}; "
                "install it with pip install -r engines/tts-piper/requirements.txt"
            )
        if not self.voice.is_file():
            raise VoiceUnavailable(
                f"missing voice model {self.voice.name}; "
                "run engines/cascade/scripts/setup_models.py"
            )
        environment = dict(os.environ)
        environment["PARLO_PIPER_VOICE"] = str(self.voice)
        try:
            process = subprocess.Popen(
                [self.python, str(self.script)],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                cwd=str(self.directory),
                env=environment,
                text=True,
                encoding="utf-8",
                bufsize=1,
            )
        except OSError as error:
            raise VoiceUnavailable(f"cannot start the piper adapter: {error}") from error
        self._process = process
        try:
            response = self._read()
        except VoiceUnavailable:
            self.close()
            raise
        if not response.get("available", False):
            self.close()
            raise VoiceUnavailable(
                str(response.get("error") or "the piper adapter is not usable")
            )
        return process

    def synthesize(self, text: str) -> tuple[bytes, int]:
        self.start()
        self._send({"op": "synthesize", "text": text})
        response = self._read()
        if "error" in response:
            raise VoiceUnavailable(str(response["error"]))
        payload = response.get("pcm16")
        if not isinstance(payload, str):
            raise VoiceUnavailable("the piper adapter returned no audio")
        return base64.b64decode(payload), int(response["sample_rate"])

    def close(self) -> None:
        if self._process is None:
            return
        process, self._process = self._process, None
        if process.stdin is not None:
            process.stdin.close()
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            process.kill()

    def _send(self, request: dict) -> None:
        process = self._process
        if process is None or process.stdin is None:
            raise VoiceUnavailable("the piper adapter is not running")
        try:
            process.stdin.write(json.dumps(request) + "\n")
            process.stdin.flush()
        except OSError as error:
            raise VoiceUnavailable(f"the piper adapter stopped: {error}") from error

    def _read(self) -> dict:
        process = self._process
        if process is None or process.stdout is None:
            raise VoiceUnavailable("the piper adapter is not running")
        try:
            line = process.stdout.readline()
        except OSError as error:
            raise VoiceUnavailable(f"the piper adapter stopped: {error}") from error
        if not line:
            raise VoiceUnavailable("the piper adapter stopped without answering")
        try:
            return json.loads(line)
        except json.JSONDecodeError as error:
            raise VoiceUnavailable(f"invalid answer from the piper adapter: {error}") from error
