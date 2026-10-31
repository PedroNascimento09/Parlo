import os
import sys
from pathlib import Path


def frozen() -> bool:
    return bool(getattr(sys, "frozen", False))


def data_root() -> Path:
    override = os.environ.get("PARLO_DATA_DIR")
    if override:
        return Path(override)
    if sys.platform == "win32":
        base = os.environ.get("APPDATA") or Path.home() / "AppData" / "Roaming"
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    else:
        base = os.environ.get("XDG_DATA_HOME") or Path.home() / ".local" / "share"
    return Path(base) / "parlo"


def models_dir() -> Path:
    override = os.environ.get("PARLO_MODELS_DIR")
    if override:
        return Path(override)
    if frozen():
        return data_root() / "models"
    return Path(__file__).resolve().parent.parent / "models"
