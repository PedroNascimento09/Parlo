import argparse
import os
import subprocess
import time
from pathlib import Path

import psutil

ROOT = Path(__file__).resolve().parents[3]


def peak_of(process: psutil.Process, peaks: dict[int, int]) -> None:
    try:
        peaks[process.pid] = max(peaks.get(process.pid, 0), process.memory_info().rss)
        for child in process.children(recursive=True):
            peaks[child.pid] = max(peaks.get(child.pid, 0), child.memory_info().rss)
    except psutil.Error:
        pass


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("--output", default=".git/memory.wav")
    parser.add_argument("--threads", type=int, default=4)
    args = parser.parse_args()

    environment = {**os.environ, "PARLO_CASCADE_THREADS": str(args.threads)}
    binary = ROOT / "target" / "debug" / ("parlo.exe" if os.name == "nt" else "parlo")
    launch = [str(binary)] if binary.is_file() else ["cargo", "run", "-q", "-p", "parlo-cli", "--"]
    command = [
        *launch,
        "translate",
        args.input,
        args.output,
        "--engine",
        "cascade",
    ]
    process = subprocess.Popen(command, cwd=ROOT, env=environment)

    peaks: dict[int, int] = {}
    root = psutil.Process(process.pid)
    while process.poll() is None:
        peak_of(root, peaks)
        time.sleep(0.2)
    peak_of(root, peaks)

    total = sum(peaks.values()) / 1024 / 1024
    print(f"peak resident memory: {total:.0f} MB across {len(peaks)} processes")
    print(f"exit code: {process.returncode}")
    return process.returncode


if __name__ == "__main__":
    raise SystemExit(main())
