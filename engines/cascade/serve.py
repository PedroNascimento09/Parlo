import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
sys.path.insert(0, str(ROOT.parent / "hibiki"))

from parlo_cascade import CascadeEngine
from parlo_hibiki.protocol import handle_line


def main() -> int:
    engine = CascadeEngine(
        source=os.environ.get("PARLO_CASCADE_SOURCE", "pt"),
        target=os.environ.get("PARLO_CASCADE_TARGET", "en"),
    )
    for line in sys.stdin:
        if not line.strip():
            continue
        response = handle_line(line, engine)
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
