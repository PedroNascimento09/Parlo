import platform
import shutil
import subprocess
from pathlib import Path

import psutil

SYSTEMS = {"Windows": "windows", "Darwin": "macos", "Linux": "linux"}
ARCHITECTURES = {"amd64": "x86_64", "x86_64": "x86_64", "arm64": "arm64", "aarch64": "arm64"}


class ProfileError(RuntimeError):
    pass


def os_name():
    return SYSTEMS.get(platform.system(), platform.system().lower())


def arch_name():
    return ARCHITECTURES.get(platform.machine().lower(), platform.machine().lower())


def _gib(value):
    try:
        return round(float(value.split()[0]) / 1024, 2)
    except (AttributeError, IndexError, ValueError):
        return 0.0


def gpus():
    try:
        result = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=name,memory.total,memory.free",
                "--format=csv,noheader,nounits",
            ],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError):
        return []
    if result.returncode != 0:
        return []
    cards = []
    for line in result.stdout.splitlines():
        fields = [field.strip() for field in line.split(",")]
        if len(fields) != 3:
            continue
        cards.append(
            {
                "name": fields[0],
                "vram_gb": _gib(fields[1]),
                "vram_free_gb": _gib(fields[2]),
                "backend": "cuda",
            }
        )
    return cards


def profile(root=None):
    memory = psutil.virtual_memory()
    target = Path(root) if root else Path.home()
    try:
        free = shutil.disk_usage(target).free
    except OSError as error:
        raise ProfileError(f"cannot read free space for {target}: {error}") from error
    cards = gpus()
    backends = ["cpu"]
    if cards:
        backends.append("cuda")
    if os_name() == "macos" and arch_name() == "arm64":
        backends.append("metal")
    return {
        "os": os_name(),
        "arch": arch_name(),
        "cpu_physical": psutil.cpu_count(logical=False) or 1,
        "cpu_logical": psutil.cpu_count() or 1,
        "ram_total_gb": round(memory.total / 1e9, 1),
        "ram_available_gb": round(memory.available / 1e9, 1),
        "disk_free_gb": round(free / 1e9, 1),
        "gpus": cards,
        "vram_gb": max((card["vram_free_gb"] for card in cards), default=0.0),
        "backends": backends,
    }
