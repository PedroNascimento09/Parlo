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


def measure(name: str, command: list[str], cwd: Path, environment: dict[str, str], audio: str, runs: int) -> dict:
    session = Session(command, cwd, environment)
    result = {"name": name, "hello_ms": None, "rss_hello_mb": None, "rss_loaded_mb": None, "dictate_ms": [], "median_ms": None, "text": "", "same_text": False}
    try:
        _, waited = session.hello()
        result["hello_ms"] = round(waited, 1)
        result["rss_hello_mb"] = round(tree_rss(session.process.pid), 1)
        texts = []
        for index in range(runs):
            try:
                answer, elapsed = session.call({"op": "dictate", "seq": index + 1, "pcm16": audio})
            except (RuntimeError, ValueError) as error:
                result["error"] = str(error)
                break
            if answer.get("op") == "error":
                result["error"] = answer.get("error")
                break
            result["dictate_ms"].append(round(elapsed, 1))
            texts.append(answer.get("text", ""))
            result["rss_loaded_mb"] = round(tree_rss(session.process.pid), 1)
        if result["dictate_ms"]:
            result["median_ms"] = round(statistics.median(result["dictate_ms"]), 1)
            result["text"] = texts[0]
            result["same_text"] = len(set(texts)) == 1
    finally:
        session.close()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Measure the Python and Rust engines on one clip")
    parser.add_argument("--wav", type=Path, required=True)
    parser.add_argument("--models", type=Path, required=True)
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
    audio, seconds = audio_payload(args.wav)
    models = str(args.models.resolve())
    results = []

    if args.cascade_dir:
        script = args.cascade_dir / "serve.py"
        if not script.is_file():
            parser.error(f"{script} does not exist")
        results.append(
            measure(
                "python source",
                [args.python, "serve.py"],
                args.cascade_dir,
                {"PARLO_MODELS_DIR": models, "PARLO_CASCADE_SOURCE": args.source, "PARLO_CASCADE_TARGET": args.target, "PARLO_CASCADE_WARMUP": "0"},
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
                {"PARLO_MODELS_DIR": models, "PARLO_CASCADE_SOURCE": args.source, "PARLO_CASCADE_TARGET": args.target, "PARLO_CASCADE_WARMUP": "0"},
                audio,
                args.runs,
            )
        )

    if args.rust:
        results.append(
            measure(
                "rust",
                [str(args.rust)],
                args.rust.parent,
                {"PARLO_MODELS_DIR": models, "PARLO_FROM": args.source, "PARLO_TO": args.target, "PARLO_CASCADE_THREADS": str(args.threads)},
                audio,
                args.runs,
            )
        )

    for result in results:
        stage = f"hello {result['hello_ms']} ms" if result["hello_ms"] is not None else "no hello"
        memory = f"rss {result['rss_hello_mb']} to {result['rss_loaded_mb']} MB"
        speech = f"dictate {result['dictate_ms']} ms" if result["dictate_ms"] else "no dictation"
        print(f"{result['name']}: {stage}, {memory}, {speech}")
        if result.get("error"):
            print(f"  {result['error']}")
        if result["text"]:
            print(f"  {result['text']}")

    if args.json:
        args.json.write_text(
            json.dumps({"clip_seconds": round(seconds, 3), "source": args.source, "target": args.target, "runs": args.runs, "engines": results}, indent=2),
            encoding="utf-8",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
