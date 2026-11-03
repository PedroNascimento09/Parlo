import argparse
import sys
from pathlib import Path

DIRECT = "Helsinki-NLP/opus-mt-tc-bible-big-roa-deu_eng_fra_por_spa"
TASK = "text2text-generation-with-past"


def export(repo, output):
    try:
        from optimum.exporters.onnx import main_export
    except ImportError:
        print(
            "this needs torch, optimum[onnxruntime] and transformers in the environment; "
            "see docs/MODEL_MANAGER.md for the pinned versions",
            file=sys.stderr,
        )
        return 2
    main_export(repo, output=str(output), task=TASK)
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description="Export a direct translation pair to ONNX.")
    parser.add_argument("--repo", default=DIRECT)
    parser.add_argument("--output", default="models/mt/pt-de-onnx")
    arguments = parser.parse_args(argv)
    output = Path(arguments.output)
    if output.exists() and any(output.iterdir()):
        print(f"{output} already has files; remove them or pick another folder", file=sys.stderr)
        return 1
    code = export(arguments.repo, output)
    if code:
        return code
    missing = [
        name
        for name in (
            "encoder_model.onnx",
            "decoder_model.onnx",
            "decoder_with_past_model.onnx",
            "config.json",
            "vocab.json",
            "source.spm",
            "target.spm",
        )
        if not (output / name).is_file()
    ]
    if missing:
        print(f"the export left out {', '.join(missing)}; copy them in before the engine can load {output}", file=sys.stderr)
        return 1
    print(f"ready     {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
