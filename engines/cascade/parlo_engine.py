import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))


def download_models() -> int:
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "parlo_setup_models", ROOT / "scripts" / "setup_models.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.main([])


def main() -> int:
    if "--download-models" in sys.argv:
        return download_models()
    import serve

    return serve.main()


if __name__ == "__main__":
    raise SystemExit(main())
