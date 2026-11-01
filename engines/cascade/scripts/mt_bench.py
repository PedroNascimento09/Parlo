import argparse
import json
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent.parent
CORPUS = ROOT / "benchmarks" / "corpus" / "translation.jsonl"

DIRECTIONS = {
    "en-pt": ("en", "pt-PT"),
    "pt-en": ("pt-PT", "en"),
}


def spm_dir(model_dir: Path) -> Path:
    if (model_dir / "source.spm").exists():
        return model_dir
    sibling = model_dir.parent / "hf"
    if (sibling / "source.spm").exists():
        return sibling
    raise SystemExit(f"no sentencepiece model beside {model_dir}")


def rows(direction: str) -> list[dict]:
    source, target = DIRECTIONS[direction]
    entries = []
    for line in CORPUS.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        entry = json.loads(line)
        if entry["source"] == source and entry["target"] == target:
            entries.append(entry)
    return entries


def main(argv=None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--direction", required=True, choices=sorted(DIRECTIONS))
    parser.add_argument("--prefix", default="")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--beam", type=int, default=4)
    parser.add_argument("--repetition-penalty", type=float, default=0.0)
    parser.add_argument("--no-repeat-ngram-size", type=int, default=0)
    parser.add_argument("--out", default=None)
    args = parser.parse_args(argv)

    import ctranslate2
    import sentencepiece as spm

    model_dir = Path(args.model)
    tokens = spm_dir(model_dir)
    source_sp = spm.SentencePieceProcessor(model_file=str(tokens / "source.spm"))
    target_sp = spm.SentencePieceProcessor(model_file=str(tokens / "target.spm"))
    translator = ctranslate2.Translator(
        str(model_dir), device="cpu", compute_type="int8", intra_threads=args.threads
    )

    results = []
    for entry in rows(args.direction):
        text = entry["text"] + (f" {args.prefix}" if args.prefix else "")
        pieces = source_sp.encode(text, out_type=str)
        started = time.perf_counter()
        output = translator.translate_batch(
            [pieces],
            beam_size=args.beam,
            repetition_penalty=args.repetition_penalty,
            no_repeat_ngram_size=args.no_repeat_ngram_size,
            max_batch_size=1,
        )[0]
        elapsed = (time.perf_counter() - started) * 1000
        translated = target_sp.decode(output.hypotheses[0])
        results.append(
            {
                "id": entry["id"],
                "source": entry["text"],
                "reference": entry["reference"],
                "output": translated,
                "ms": round(elapsed),
                "tags": entry["tags"],
            }
        )

    print(f"{args.direction} | {model_dir} | prefix {args.prefix or 'none'} | beam {args.beam}")
    for row in results:
        print(f"\n[{row['id']}] {row['ms']} ms  {row['tags']}")
        print(f"  src  {row['source']}")
        print(f"  ref  {row['reference']}")
        print(f"  out  {row['output']}")

    times = sorted(row["ms"] for row in results)
    summary = {
        "direction": args.direction,
        "model": str(model_dir),
        "prefix": args.prefix or None,
        "beam": args.beam,
        "repetition_penalty": args.repetition_penalty,
        "no_repeat_ngram_size": args.no_repeat_ngram_size,
        "threads": args.threads,
        "sentences": len(results),
        "median_ms": times[len(times) // 2] if times else None,
        "p95_ms": times[max(0, int(len(times) * 0.95) - 1)] if times else None,
        "rows": results,
    }
    print(
        f"\n{summary['sentences']} sentences, median {summary['median_ms']} ms, "
        f"p95 {summary['p95_ms']} ms, {args.threads} threads"
    )
    if args.out:
        Path(args.out).write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"written {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
