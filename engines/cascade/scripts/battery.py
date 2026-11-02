import argparse
import json
import os
import sys
import unicodedata
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from parlo_cascade import CascadeEngine, pcm16_to_float

SENTENCES = (
    "O meu nome é Jorge e faço programas de computador.",
    "Marquei uma reunião para amanhã às dez da manhã.",
    "Preciso de trezentos e vinte euros para o bilhete.",
    "A Maria telefona ao Pedro todas as sextas-feiras.",
    "O projeto abre o microfone e traduz a frase.",
    "Envia o relatório até quinta-feira, por favor.",
    "Estou a falar de Lisboa e não do Porto.",
    "A reunião com o cliente começa às catorze e trinta.",
    "Compra pão, leite e café quando saíres do trabalho.",
    "O senhor Silva trabalha no terceiro andar do edifício.",
    "Preciso de ajuda com esta tradução para inglês.",
    "O comboio para o Porto parte às dezanove e quarenta.",
    "A temperatura em Coimbra está nos vinte e cinco graus.",
    "Vou ligar ao João depois do almoço de domingo.",
    "Podes confirmar o endereço da rua das Flores?",
    "O ficheiro está na pasta de documentos partilhados.",
    "Temos de enviar as faturas antes do fim do mês.",
    "A Ana disse que chegava por volta das oito.",
    "Este programa funciona sem internet e sem enviar dados.",
    "Quero traduzir esta conversa em tempo real.",
    "O número do cliente é vinte e três, quarenta e cinco.",
    "Amanhã de manhã tenho uma chamada com Londres.",
    "Obrigado pela ajuda, falamos depois do jantar.",
    "O sistema precisa de mais memória para correr bem.",
)
CONDITIONS = (("clean", None), ("noise20", 20.0), ("noise10", 10.0))


def normalise(text: str) -> str:
    plain = unicodedata.normalize("NFD", text.lower())
    kept = "".join(
        character
        for character in plain
        if not unicodedata.combining(character) and (character.isalnum() or character.isspace())
    )
    return " ".join(kept.split())


def word_error_rate(reference: str, hypothesis: str) -> float:
    words = normalise(reference).split()
    heard = normalise(hypothesis).split()
    if not words:
        return 0.0
    previous = list(range(len(heard) + 1))
    for row, expected in enumerate(words, 1):
        current = [row]
        for column, actual in enumerate(heard, 1):
            substitution = previous[column - 1] + (0 if expected == actual else 1)
            current.append(
                min(previous[column] + 1, current[column - 1] + 1, substitution)
            )
        previous = current
    return previous[-1] / len(words)


def add_noise(samples, snr_db: float, seed: int):
    generator = np.random.default_rng(seed)
    power = float(np.mean(samples * samples))
    if power <= 0.0:
        return samples
    noise_power = power / (10.0 ** (snr_db / 10.0))
    noise = generator.normal(0.0, np.sqrt(noise_power), samples.shape)
    return np.clip(samples + noise.astype("float32"), -1.0, 1.0)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stt", choices=["parakeet", "whisper"], default="parakeet")
    parser.add_argument("--model", default="base")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--out", default="")
    parser.add_argument("--voice", default="pt_PT-tugão-medium.onnx")
    args = parser.parse_args()

    os.environ["PARLO_CASCADE_STT"] = args.stt
    os.environ["PARLO_CASCADE_THREADS"] = str(args.threads)
    if args.stt == "whisper":
        os.environ["PARLO_CASCADE_MODEL"] = args.model

    sentences = SENTENCES[: args.limit] if args.limit > 0 else SENTENCES
    voice = Path(__file__).resolve().parent.parent / "models" / args.voice
    engine = CascadeEngine(source="pt", target="en", voice=voice)
    if not engine.available:
        raise SystemExit("install the cascade requirements first")
    if not engine.voice_path.exists():
        raise SystemExit("missing voice model, run scripts/setup_models.py")

    engine._load_stt()
    rows = []
    for index, sentence in enumerate(sentences):
        clean = np.asarray(
            pcm16_to_float(engine._synthesize(sentence)), dtype="float32"
        )
        row = {"text": sentence, "conditions": {}}
        for name, snr in CONDITIONS:
            samples = clean if snr is None else add_noise(clean, snr, args.seed + index)
            heard = engine._transcribe(samples)
            row["conditions"][name] = {
                "wer": round(word_error_rate(sentence, heard), 3),
                "transcript": heard,
            }
        rows.append(row)

    summary = {
        "sentences": len(rows),
        "seed": args.seed,
        "stt": args.stt,
        "model": args.model if args.stt == "whisper" else "",
        "threads": args.threads,
        "voice": args.voice,
        "conditions": {},
    }
    print(f"\n{args.stt} | {len(rows)} sentences | seed {args.seed}")
    for name, _ in CONDITIONS:
        scores = [row["conditions"][name]["wer"] for row in rows]
        mean = round(float(np.mean(scores)), 3)
        summary["conditions"][name] = {"mean_wer": mean, "errors": scores}
        label = "clean" if name == "clean" else f"{name[5:]} dB SNR"
        print(f"  {label:<16} mean WER {mean:>5.3f}")
    summary["rows"] = rows
    print("\n" + json.dumps({key: summary[key] for key in summary if key != "rows"}))
    if args.out:
        Path(args.out).write_text(json.dumps(summary, indent=2), encoding="utf-8")
        print(f"written {args.out}")
    return 1 if summary["conditions"]["clean"]["mean_wer"] > 0.5 else 0


if __name__ == "__main__":
    raise SystemExit(main())
