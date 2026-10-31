import json
import sys

from parlo_hibiki import HibikiEngine
from parlo_hibiki.protocol import handle_line


def main() -> int:
    engine = HibikiEngine()
    for line in sys.stdin:
        if not line.strip():
            continue
        response = handle_line(line, engine)
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
