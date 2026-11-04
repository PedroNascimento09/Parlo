import argparse
import base64
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
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            bufsize=1,
        )

    def call(self, request: dict, timeout: int = 240) -> tuple[dict, float]:
        begin = time.perf_counter()
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()
        timer = threading.Timer(timeout, self.process.kill)
        timer.start()
        line = self.process.stdout.readline()
        timer.cancel()
        if not line:
            raise RuntimeError("the engine closed its output")
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
    runs: int,
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
        "translate_first_ms": None,
        "translate_ms": [],
        "translate_median_ms": None,
        "translate_audio_seconds": None,
        "translate_rate": None,
        "translate_text": "",
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
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()

    if not args.cascade_dir and not args.bundled and not args.rust:
        parser.error("pass at least one of --cascade-dir, --bundled or --rust")
    if args.voice and not args.voice.is_file():
        parser.error(f"{args.voice} does not exist")
    if args.espeak_data and not args.espeak_data.is_dir():
        parser.error(f"{args.espeak_data} does not exist")
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
        "PARLO_CASCADE_WARMUP": "0",
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
                args.runs,
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
                args.runs,
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
                args.runs,
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
        if result["translate_median_ms"] is not None:
            print(
                f"  translate {result['translate_first_ms']} ms first, "
                f"{result['translate_median_ms']} ms median; "
                f"{result['translate_audio_seconds']} s at {result['translate_rate']} Hz; "
                f"{result['translate_text']}"
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
