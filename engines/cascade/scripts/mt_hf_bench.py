import argparse
import json
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent.parent
CORPUS = ROOT / "benchmarks" / "corpus" / "translation.jsonl"

DIRECTIONS = {
    "en-pt": ("en", "pt-PT"),
    "pt-en": ("pt-PT", "en"),
}


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
    parser.add_argument("--out", default=None)
    args = parser.parse_args(argv)

    import torch
    from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

    torch.set_num_threads(args.threads)
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    model = AutoModelForSeq2SeqLM.from_pretrained(args.model)
    model.eval()

    results = []
    with torch.no_grad():
        for entry in rows(args.direction):
            text = entry["text"] + (f" {args.prefix}" if args.prefix else "")
            started = time.perf_counter()
            batch = tokenizer([text], return_tensors="pt", padding=True, truncation=True)
            output = model.generate(**batch, num_beams=args.beam, max_length=128)
            elapsed = (time.perf_counter() - started) * 1000
            translated = tokenizer.decode(output[0], skip_special_tokens=True)
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

    print(f"{args.direction} | {args.model} | prefix {args.prefix or 'none'} | beam {args.beam}")
    for row in results:
        print(f"\n[{row['id']}] {row['ms']} ms  {row['tags']}")
        print(f"  src  {row['source']}")
        print(f"  ref  {row['reference']}")
        print(f"  out  {row['output']}")

    times = sorted(row["ms"] for row in results)
    summary = {
        "direction": args.direction,
        "model": args.model,
        "prefix": args.prefix or None,
        "beam": args.beam,
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
