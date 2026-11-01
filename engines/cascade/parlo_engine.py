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
    arguments = sys.argv[sys.argv.index("--download-models") + 1 :]
    return module.main(arguments)


def intelligence_admin() -> int:
    from parlo_cascade import admin

    return admin.main()


def main() -> int:
    if "--download-models" in sys.argv:
        return download_models()
    if "--intelligence" in sys.argv:
        return intelligence_admin()
    import serve

    return serve.main()


if __name__ == "__main__":
    raise SystemExit(main())
