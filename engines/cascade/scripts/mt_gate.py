import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent.parent
CORPUS = ROOT / "benchmarks" / "corpus" / "translation.jsonl"
DIRECTIONS = {
    "pt-en": ("pt-PT", "en"),
    "en-pt": ("en", "pt-PT"),
}
THRESHOLDS = {
    "numbers": 0.95,
    "clock": 0.95,
    "negation": 0.90,
    "names": 0.80,
    "complete": 1.0,
}
NUMBER = re.compile(r"\d+(?:[.,\s]\d{3})*(?:[.,]\d+)?")
TIME = re.compile(
    r"\b(?:[01]?\d|2[0-3])\.([0-5]\d)(?!\d)"
    r"|\b(?:[01]?\d|2[0-3])\s?[h:]\s?([0-5]\d)?\b"
    r"|\b(?:[01]?\d|2[0-3])\s?(am|pm)\b",
    re.IGNORECASE,
)
WORD_NUMBERS = {
    "zero": 0, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
    "seven": 7, "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12,
    "thirteen": 13, "fourteen": 14, "fifteen": 15, "sixteen": 16, "seventeen": 17,
    "eighteen": 18, "nineteen": 19, "twenty": 20, "thirty": 30, "forty": 40,
    "fifty": 50, "sixty": 60, "seventy": 70, "eighty": 80, "ninety": 90,
    "um": 1, "uma": 1, "dois": 2, "duas": 2, "três": 3, "tres": 3, "quatro": 4,
    "cinco": 5, "seis": 6, "sete": 7, "oito": 8, "nove": 9, "dez": 10, "onze": 11,
    "doze": 12, "treze": 13, "catorze": 14, "quatorze": 14, "quinze": 15,
    "dezasseis": 16, "dezassete": 17, "dezessete": 17, "dezoito": 18,
    "dezanove": 19, "dezenove": 19, "vinte": 20,
}
NEGATION = {
    "en": re.compile(r"\b(not|no|never|none|nobody|nothing|without|neither|nor)\b|n't", re.I),
    "pt": re.compile(
        r"\b(não|nunca|nenhum|nenhuma|nenhuns|nenhumas|ninguém|ninguem|nada|nem|sem)\b", re.I
    ),
}
NOT_NAMES = {
    "segunda", "terça", "terca", "quarta", "quinta", "sexta", "sábado", "sabado", "domingo",
    "janeiro", "fevereiro", "março", "marco", "abril", "maio", "junho", "julho", "agosto",
    "setembro", "outubro", "novembro", "dezembro", "monday", "tuesday", "wednesday",
    "thursday", "friday", "saturday", "sunday", "january", "february", "march", "april",
    "may", "june", "july", "august", "september", "october", "november", "december",
    "i", "eu", "the", "o", "a", "os", "as", "um", "uma",
}


def values(text: str) -> set[str]:
    found = set()
    for raw in NUMBER.findall(text):
        digits = raw.replace(" ", "")
        if re.fullmatch(r"\d{1,3}(?:\.\d{3})+", digits):
            digits = digits.replace(".", "")
        elif re.fullmatch(r"\d{1,3}(?:,\d{3})+", digits):
            digits = digits.replace(",", "")
        elif "," in digits and "." in digits:
            separator = "," if digits.rfind(",") > digits.rfind(".") else "."
            thousands = "." if separator == "," else ","
            digits = digits.replace(thousands, "").replace(separator, ".")
        elif "," in digits:
            head, _, tail = digits.rpartition(",")
            digits = digits.replace(",", "") if len(tail) == 3 and head else digits.replace(",", ".")
        elif digits.count(".") > 1:
            digits = digits.replace(".", "")
        try:
            found.add(f"{float(digits):.4f}")
        except ValueError:
            continue
    for word in re.findall(r"[^\W\d_]+", text, re.UNICODE):
        value = WORD_NUMBERS.get(word.lower())
        if value is not None:
            found.add(f"{float(value):.4f}")
    return found


def minutes(text: str) -> set[int]:
    clocks = set()
    for raw in TIME.finditer(text):
        span = raw.group(0).lower().replace(" ", "")
        marker = raw.group(3)
        if marker is None:
            trailing = text[raw.end() : raw.end() + 3].strip().lower()
            marker = "pm" if trailing.startswith("pm") else "am" if trailing.startswith("am") else ""
        cleaned = span.replace("am", "").replace("pm", "")
        separator = next((char for char in "h:." if char in cleaned), None)
        if separator is None:
            hour_text, minute_text = cleaned, "00"
        else:
            hour_text, _, minute_text = cleaned.partition(separator)
            minute_text = minute_text or "00"
        if not hour_text.isdigit() or not minute_text.isdigit():
            continue
        hour = int(hour_text)
        minute = int(minute_text)
        if marker and marker.lower() == "pm" and hour < 12:
            hour += 12
        if marker and marker.lower() == "am" and hour == 12:
            hour = 0
        if hour < 24 and minute < 60:
            clocks.add(hour * 60 + minute)
    return clocks


def clock_ok(source: str, output: str) -> bool:
    source_times = minutes(source)
    output_times = minutes(output)
    if source_times <= output_times:
        return True
    printed = {float(value) for value in values(TIME.sub(" ", output))}
    for missing in source_times - output_times:
        hour, minute = divmod(missing, 60)
        if minute:
            return False
        if hour not in printed and (hour % 12 or 12) not in printed:
            return False
    return True


def names(text: str) -> set[str]:
    found = set()
    for index, word in enumerate(text.split()):
        bare = re.sub(r"^[^\wÀ-ÿ]+|[^\wÀ-ÿ]+$", "", word)
        if index == 0 or len(bare) < 3 or not bare[:1].isupper() or bare.lower() in NOT_NAMES:
            continue
        if bare.isupper():
            continue
        found.add(bare.lower())
    return found


def check(row: dict, output: str) -> dict:
    source = row["text"]
    reference = row["reference"]
    source_language = "pt" if row["source"].startswith("pt") else "en"
    target_language = "en" if source_language == "pt" else "pt"
    source_numbers = values(TIME.sub(" ", source))
    source_names = names(source)
    return {
        "numbers": source_numbers <= values(TIME.sub(" ", output)),
        "negation": not NEGATION[source_language].search(source)
        or bool(NEGATION[target_language].search(output)),
        "names": source_names <= names(output),
        "complete": bool(output.strip()) and output.strip()[-1] in ".!?",
        "clock": clock_ok(source, output),
        "reference_numbers": source_numbers <= values(TIME.sub(" ", reference)),
        "reference_names": source_names <= names(reference),
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--direction", required=True, choices=sorted(DIRECTIONS))
    parser.add_argument("--pack", action="store_true")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--show", action="store_true")
    parser.add_argument("--out", default="")
    args = parser.parse_args(argv)
    if not args.pack:
        raise SystemExit("only --pack is wired; a converted checkpoint is not used for a release gate")

    source, target = DIRECTIONS[args.direction]
    rows = [
        json.loads(line)
        for line in CORPUS.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    rows = [row for row in rows if row["source"] == source and row["target"] == target]
    if not rows:
        raise SystemExit(f"no rows for {args.direction} in {CORPUS}")

    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from parlo_cascade import mt

    pack = mt.find_pack(source.split("-")[0], target.split("-")[0])
    if pack is None:
        raise SystemExit(f"no package installed for {args.direction}")
    translate = mt.MarianTranslator(pack, threads=args.threads, language=target.split("-")[0])

    results = []
    for row in rows:
        output = translate(row["text"])
        results.append({**row, **check(row, output), "output": output})

    totals = {
        key: round(sum(1 for result in results if result[key]) / len(results), 3)
        for key in ("numbers", "clock", "negation", "names", "complete")
    }
    if args.show:
        for result in results:
            flags = "".join(
                key[0].upper() if result[key] else key[0]
                for key in ("numbers", "negation", "names", "complete")
            )
            print(f"{result['id']}  {flags}  {result['text']}")
            print(f"           out  {result['output']}")

    reference = {
        key: round(sum(1 for result in results if result[f"reference_{key}"]) / len(results), 3)
        for key in ("numbers", "names")
    }
    print(f"\n{args.direction} | {len(results)} rows | {args.threads} threads | pack {pack.name}")
    for key, threshold in THRESHOLDS.items():
        mark = "ok" if totals[key] >= threshold else "FAIL"
        print(f"  {key:<9} {totals[key]:.3f}  threshold {threshold:.2f}  {mark}")
    print(f"  reference numbers {reference['numbers']:.3f}, names {reference['names']:.3f}")

    failures = [result["id"] for result in results if not result["numbers"]]
    if failures:
        print(f"  rows that lost a number: {', '.join(failures)}")
    failures = [result["id"] for result in results if not result["negation"]]
    if failures:
        print(f"  rows that lost a negation: {', '.join(failures)}")
    failures = [result["id"] for result in results if not result["names"]]
    if failures:
        print(f"  rows that lost a name: {', '.join(failures)}")

    if args.out:
        summary = {"direction": args.direction, "rows": len(results), "totals": totals, "reference": reference}
        Path(args.out).write_text(
            json.dumps({"summary": summary, "rows": results}, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        print(f"written {args.out}")
    return 0 if all(totals[key] >= value for key, value in THRESHOLDS.items()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
