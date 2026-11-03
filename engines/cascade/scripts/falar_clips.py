import argparse
import io
import json
import wave
from pathlib import Path

import pyarrow.parquet as pq


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--parquet", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--count", type=int, default=20)
    parser.add_argument("--max-cer", type=float, default=0.05)
    parser.add_argument("--min-seconds", type=float, default=2.0)
    parser.add_argument("--max-seconds", type=float, default=12.0)
    args = parser.parse_args()

    table = pq.read_table(
        args.parquet,
        columns=["wav", "transcription", "cer", "duration", "speaker_id"],
    )
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    manifest = []
    for row in table.to_pylist():
        if len(manifest) >= args.count:
            break
        if row["cer"] is None or row["cer"] > args.max_cer:
            continue
        seconds = row["duration"] or 0.0
        if seconds < args.min_seconds or seconds > args.max_seconds:
            continue
        text = (row["transcription"] or "").strip()
        data = (row["wav"] or {}).get("bytes") or b""
        if not text or not data:
            continue
        with wave.open(io.BytesIO(data), "rb") as handle:
            mono = handle.getnchannels() == 1
            rate = handle.getframerate() == 16_000
            width = handle.getsampwidth() == 2
        if not (mono and rate and width):
            continue
        name = f"falar-{len(manifest):03d}.wav"
        (out / name).write_bytes(data)
        manifest.append(
            {
                "audio": name,
                "text": text,
                "cer": round(row["cer"], 4),
                "seconds": round(seconds, 2),
                "speaker": int(row["speaker_id"] or 0),
            }
        )

    if not manifest:
        raise SystemExit(f"no clip matched the filters in {args.parquet}")
    lines = [json.dumps(row, ensure_ascii=False) for row in manifest]
    (out / "reference.jsonl").write_text("\n".join(lines) + "\n", encoding="utf-8")
    total = sum(row["seconds"] for row in manifest)
    speakers = len({row["speaker"] for row in manifest})
    print(
        f"{len(manifest)} clips, {total:.1f} s, {speakers} speakers, "
        f"max cer {args.max_cer}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
