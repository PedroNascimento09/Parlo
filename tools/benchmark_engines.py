import argparse
import base64
import collections
import json
import os
import statistics
import subprocess
import sys
import threading
import time
import wave
from pathlib import Path

import psutil

ROOT = Path(__file__).resolve().parent.parent


def default_python() -> str:
    local = ROOT / ".venv" / "Scripts" / "python.exe"
    return str(local) if local.is_file() else sys.executable


def audio_payload(path: Path) -> tuple[str, float]:
    with wave.open(str(path), "rb") as handle:
        if handle.getframerate() != 16_000 or handle.getnchannels() != 1 or handle.getsampwidth() != 2:
            raise SystemExit("the clip must be 16 kHz mono 16 bit PCM")
        seconds = handle.getnframes() / handle.getframerate()
        return base64.b64encode(handle.readframes(handle.getnframes())).decode(), seconds


def tree_rss(pid: int) -> float:
    try:
        parent = psutil.Process(pid)
    except psutil.Error:
        return 0.0
    total = parent.memory_info().rss
    for child in parent.children(recursive=True):
        try:
            total += child.memory_info().rss
        except psutil.Error:
            pass
    return total / 1_048_576


class Session:
    def __init__(self, command: list[str], cwd: Path, environment: dict[str, str]):
        self.started = time.perf_counter()
        merged = os.environ.copy()
        merged.update(environment)
        self.process = subprocess.Popen(
            command,
            cwd=str(cwd),
            env=merged,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            bufsize=1,
        )
        self.stderr: collections.deque[str] = collections.deque(maxlen=8)
        threading.Thread(target=self._drain, daemon=True).start()

    def _drain(self) -> None:
        stream = self.process.stderr
        if stream is None:
            return
        for line in stream:
            text = line.strip()
            if text:
                self.stderr.append(text)

    def reason(self) -> str:
        return f": {self.stderr[-1]}" if self.stderr else ""

    def call(self, request: dict, timeout: int = 240) -> tuple[dict, float]:
        begin = time.perf_counter()
        try:
            self.process.stdin.write(json.dumps(request) + "\n")
            self.process.stdin.flush()
        except OSError as error:
            raise RuntimeError(f"the engine stopped: {error}" + self.reason()) from error
        timer = threading.Timer(timeout, self.process.kill)
        timer.start()
        line = self.process.stdout.readline()
        timer.cancel()
        if not line:
            raise RuntimeError("the engine closed its output" + self.reason())
        return json.loads(line), (time.perf_counter() - begin) * 1000

    def hello(self) -> tuple[dict, float]:
        answer, _ = self.call({"op": "hello"})
        return answer, (time.perf_counter() - self.started) * 1000

    def close(self) -> None:
        try:
            if self.process.poll() is None:
                self.process.terminate()
                self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()


def answer_text(answer: dict) -> str:
    return str(answer.get("text") or "")


def answer_translation(answer: dict) -> str:
    return str(answer.get("translation") or "")


def audio_seconds(answer: dict, fallback_rate: int) -> tuple[float, int]:
    payload = answer.get("pcm16") or ""
    if not payload:
        return 0.0, fallback_rate
    pcm16 = base64.b64decode(payload)
    rate = int(answer.get("sample_rate") or fallback_rate)
    return len(pcm16) / 2 / rate, rate


def measure(
    name: str,
    command: list[str],
    cwd: Path,
    environment: dict[str, str],
    audio: str,
    source: str,
    target: str,
    switch_to: str | None,
    runs: int,
    speak_text: str | None,
) -> dict:
    session = Session(command, cwd, environment)
    result = {
        "name": name,
        "hello_ms": None,
        "rss_hello_mb": None,
        "rss_loaded_mb": None,
        "dictate_ms": [],
        "median_ms": None,
        "text": "",
        "same_text": False,
        "write_first_ms": None,
        "write_ms": [],
        "write_median_ms": None,
        "write_translation": "",
        "switch_target": None,
        "switch_ms": None,
        "switch_write_first_ms": None,
        "switch_write_second_ms": None,
        "switch_translation": "",
        "switch_back_ms": None,
        "switch_back_write_ms": None,
        "translate_first_ms": None,
        "translate_ms": [],
        "translate_median_ms": None,
        "translate_audio_seconds": None,
        "translate_rate": None,
        "translate_text": "",
        "speak_first_ms": None,
        "speak_ms": [],
        "speak_median_ms": None,
        "speak_seconds": None,
        "errors": [],
    }
    try:
        hello, waited = session.hello()
        rate = int(hello.get("sample_rate") or 16_000)
        result["hello_ms"] = round(waited, 1)
        result["rss_hello_mb"] = round(tree_rss(session.process.pid), 1)
        texts = []
        for index in range(runs):
            try:
                answer, elapsed = session.call({"op": "dictate", "seq": index + 1, "pcm16": audio})
            except (RuntimeError, ValueError) as error:
                result["errors"].append(f"dictate: {error}")
                break
            if answer.get("op") == "error":
                result["errors"].append(f"dictate: {answer.get('error')}")
                break
            result["dictate_ms"].append(round(elapsed, 1))
            texts.append(answer_text(answer))
            result["rss_loaded_mb"] = round(tree_rss(session.process.pid), 1)
        if result["dictate_ms"]:
            result["median_ms"] = round(statistics.median(result["dictate_ms"]), 1)
            result["text"] = texts[0]
            result["same_text"] = len(set(texts)) == 1
        else:
            result["text"] = ""

        try:
            answer, elapsed = session.call({"op": "write", "seq": 1, "pcm16": audio})
            if answer.get("op") == "error":
                raise RuntimeError(str(answer.get("error")))
            result["write_first_ms"] = round(elapsed, 1)
            result["write_translation"] = answer_translation(answer)
            for index in range(runs):
                answer, elapsed = session.call({"op": "write", "seq": index + 2, "pcm16": audio})
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["write_ms"].append(round(elapsed, 1))
                result["write_translation"] = answer_translation(answer)
            result["write_median_ms"] = round(statistics.median(result["write_ms"]), 1)
        except (RuntimeError, ValueError) as error:
            result["errors"].append(f"write: {error}")

        if switch_to and result["write_first_ms"] is not None:
            try:
                answer, elapsed = session.call(
                    {"op": "pair", "seq": 1, "from": source, "to": switch_to}
                )
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["switch_target"] = switch_to
                result["switch_ms"] = round(elapsed, 1)
                for field, seq in (
                    ("switch_write_first_ms", 2),
                    ("switch_write_second_ms", 3),
                ):
                    answer, elapsed = session.call(
                        {"op": "write", "seq": seq, "pcm16": audio}
                    )
                    if answer.get("op") == "error":
                        raise RuntimeError(str(answer.get("error")))
                    result[field] = round(elapsed, 1)
                    result["switch_translation"] = answer_translation(answer)
                answer, elapsed = session.call(
                    {"op": "pair", "seq": 4, "from": source, "to": target}
                )
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["switch_back_ms"] = round(elapsed, 1)
                answer, elapsed = session.call({"op": "write", "seq": 5, "pcm16": audio})
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["switch_back_write_ms"] = round(elapsed, 1)
            except (RuntimeError, ValueError) as error:
                result["errors"].append(f"switch: {error}")

        if speak_text:
            try:
                answer, elapsed = session.call(
                    {"op": "speak", "seq": 1, "text": speak_text}
                )
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["speak_first_ms"] = round(elapsed, 1)
                spoken, _ = audio_seconds(answer, rate)
                result["speak_seconds"] = round(spoken, 3)
                for index in range(runs):
                    answer, elapsed = session.call(
                        {"op": "speak", "seq": index + 2, "text": speak_text}
                    )
                    if answer.get("op") == "error":
                        raise RuntimeError(str(answer.get("error")))
                    result["speak_ms"].append(round(elapsed, 1))
                    spoken, _ = audio_seconds(answer, rate)
                    result["speak_seconds"] = round(spoken, 3)
                result["speak_median_ms"] = round(
                    statistics.median(result["speak_ms"]), 1
                )
            except (RuntimeError, ValueError) as error:
                result["errors"].append(f"speak: {error}")

        try:
            answer, elapsed = session.call({"op": "translate", "seq": 1, "pcm16": audio})
            if answer.get("op") == "error":
                raise RuntimeError(str(answer.get("error")))
            result["translate_first_ms"] = round(elapsed, 1)
            result["translate_text"] = answer_text(answer)
            seconds, output_rate = audio_seconds(answer, rate)
            result["translate_audio_seconds"] = round(seconds, 3)
            result["translate_rate"] = output_rate
            for index in range(runs):
                answer, elapsed = session.call({"op": "translate", "seq": index + 2, "pcm16": audio})
                if answer.get("op") == "error":
                    raise RuntimeError(str(answer.get("error")))
                result["translate_ms"].append(round(elapsed, 1))
                result["translate_text"] = answer_text(answer)
                seconds, output_rate = audio_seconds(answer, rate)
                result["translate_audio_seconds"] = round(seconds, 3)
                result["translate_rate"] = output_rate
            result["translate_median_ms"] = round(statistics.median(result["translate_ms"]), 1)
        except (RuntimeError, ValueError) as error:
            result["errors"].append(f"translate: {error}")
    finally:
        session.close()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Measure the Python and Rust engines on one clip")
    parser.add_argument("--wav", type=Path, required=True)
    parser.add_argument("--models", type=Path, required=True)
    parser.add_argument("--voice", type=Path)
    parser.add_argument("--espeak-data", type=Path)
    parser.add_argument("--cascade-dir", type=Path)
    parser.add_argument("--python", default=default_python())
    parser.add_argument("--bundled", type=Path)
    parser.add_argument("--rust", type=Path)
    parser.add_argument("--from", dest="source", default="pt")
    parser.add_argument("--to", dest="target", default="en")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--switch-to", dest="switch_to", default=None)
    parser.add_argument("--warmup", action="store_true")
    parser.add_argument(
        "--speak", dest="speak_text", default=None, help="text to say aloud after the translation calls"
    )
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()

    if not args.cascade_dir and not args.bundled and not args.rust:
        parser.error("pass at least one of --cascade-dir, --bundled or --rust")
    if args.voice and not args.voice.is_file():
        parser.error(f"{args.voice} does not exist")
    if args.espeak_data and not args.espeak_data.is_dir():
        parser.error(f"{args.espeak_data} does not exist")
    if args.switch_to:
        if len(args.switch_to) != 2 or not args.switch_to.isalpha():
            parser.error("--switch-to needs a two letter language code")
        if args.switch_to == args.target:
            parser.error("--switch-to must differ from --to")
    audio, seconds = audio_payload(args.wav)
    models = str(args.models.resolve())
    voice = str(args.voice.resolve()) if args.voice else None
    espeak = str(args.espeak_data.resolve()) if args.espeak_data else None
    results = []

    python_environment = {
        "PARLO_MODELS_DIR": models,
        "PARLO_CASCADE_SOURCE": args.source,
        "PARLO_CASCADE_TARGET": args.target,
        "PARLO_CASCADE_THREADS": str(args.threads),
        "PARLO_CASCADE_WARMUP": "1" if args.warmup else "0",
    }
    if voice:
        python_environment["PARLO_CASCADE_VOICE"] = voice

    if args.cascade_dir:
        script = args.cascade_dir / "serve.py"
        if not script.is_file():
            parser.error(f"{script} does not exist")
        results.append(
            measure(
                "python source",
                [args.python, "serve.py"],
                args.cascade_dir,
                python_environment,
                audio,
                args.source,
                args.target,
                args.switch_to,
                args.runs,
                args.speak_text,
            )
        )

    if args.bundled:
        results.append(
            measure(
                "python bundled",
                [str(args.bundled)],
                args.bundled.parent,
                python_environment,
                audio,
                args.source,
                args.target,
                args.switch_to,
                args.runs,
                args.speak_text,
            )
        )

    if args.rust:
        rust_environment = {
            "PARLO_MODELS_DIR": models,
            "PARLO_FROM": args.source,
            "PARLO_TO": args.target,
            "PARLO_CASCADE_THREADS": str(args.threads),
        }
        if voice:
            rust_environment["PARLO_VOICE_PATH"] = voice
        if espeak:
            rust_environment["PARLO_ESPEAK_DATA"] = espeak
        results.append(
            measure(
                "rust",
                [str(args.rust)],
                args.rust.parent,
                rust_environment,
                audio,
                args.source,
                args.target,
                args.switch_to,
                args.runs,
                args.speak_text,
            )
        )

    for result in results:
        stage = f"hello {result['hello_ms']} ms" if result["hello_ms"] is not None else "no hello"
        memory = f"rss {result['rss_hello_mb']} to {result['rss_loaded_mb']} MB"
        speech = f"dictate {result['dictate_ms']} ms" if result["dictate_ms"] else "no dictation"
        print(f"{result['name']}: {stage}, {memory}, {speech}")
        if result["write_median_ms"] is not None:
            print(
                f"  write {result['write_first_ms']} ms first, "
                f"{result['write_median_ms']} ms median; {result['write_translation']}"
            )
        if result["switch_ms"] is not None:
            print(
                f"  switch to {result['switch_target']} {result['switch_ms']} ms; "
                f"write {result['switch_write_first_ms']} ms first, "
                f"{result['switch_write_second_ms']} ms second; "
                f"back {result['switch_back_ms']} ms, "
                f"write {result['switch_back_write_ms']} ms; "
                f"{result['switch_translation']}"
            )
        if result["translate_median_ms"] is not None:
            print(
                f"  translate {result['translate_first_ms']} ms first, "
                f"{result['translate_median_ms']} ms median; "
                f"{result['translate_audio_seconds']} s at {result['translate_rate']} Hz; "
                f"{result['translate_text']}"
            )
        if result["speak_median_ms"] is not None:
            print(
                f"  speak {result['speak_first_ms']} ms first, "
                f"{result['speak_median_ms']} ms median; "
                f"{result['speak_seconds']} s of audio"
            )
        for error in result["errors"]:
            print(f"  {error}")
        if result["text"]:
            print(f"  {result['text']}")

    if args.json:
        args.json.write_text(
            json.dumps(
                {
                    "clip_seconds": round(seconds, 3),
                    "source": args.source,
                    "target": args.target,
                    "threads": args.threads,
                    "runs": args.runs,
                    "engines": results,
                },
                indent=2,
            ),
            encoding="utf-8",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
