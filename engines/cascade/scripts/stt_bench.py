import argparse
import statistics
import time
import wave
from pathlib import Path

import numpy as np
import sherpa_onnx
from faster_whisper import WhisperModel

SAMPLE_RATE = 16_000


def load(path: Path) -> np.ndarray:
    with wave.open(str(path)) as wav:
        rate, channels = wav.getframerate(), wav.getnchannels()
        raw = wav.readframes(wav.getnframes())
    audio = np.frombuffer(raw, dtype="<i2").astype("float32") / 32_768.0
    if channels > 1:
        audio = audio.reshape(-1, channels).mean(axis=1)
    if rate != SAMPLE_RATE:
        count = int(len(audio) * SAMPLE_RATE / rate)
        audio = np.interp(
            np.linspace(0, len(audio) - 1, count),
            np.arange(len(audio)),
            audio,
        ).astype("float32")
    return audio


def parakeet(directory: Path, threads: int):
    return sherpa_onnx.OfflineRecognizer.from_transducer(
        encoder=str(directory / "encoder.int8.onnx"),
        decoder=str(directory / "decoder.int8.onnx"),
        joiner=str(directory / "joiner.int8.onnx"),
        tokens=str(directory / "tokens.txt"),
        num_threads=threads,
        model_type="nemo_transducer",
    )


def transcribe_parakeet(recognizer, audio: np.ndarray) -> str:
    stream = recognizer.create_stream()
    stream.accept_waveform(SAMPLE_RATE, audio)
    recognizer.decode_stream(stream)
    return stream.result.text.strip()


def whisper(model: str, threads: int):
    return WhisperModel(
        model,
        device="cpu",
        compute_type="int8",
        cpu_threads=threads,
    )


def transcribe_whisper(model, audio: np.ndarray, language: str) -> str:
    segments, _ = model.transcribe(
        audio,
        beam_size=1,
        vad_filter=False,
        language=language,
    )
    return " ".join(segment.text.strip() for segment in segments)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("wav")
    parser.add_argument("--backend", choices=["whisper", "parakeet"], required=True)
    parser.add_argument("--model", default="base")
    parser.add_argument("--dir", default="models/parakeet")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--language", default="pt")
    parser.add_argument("--chunk", type=float, default=0.0)
    args = parser.parse_args()

    audio = load(Path(args.wav))
    duration = len(audio) / SAMPLE_RATE
    if args.chunk:
        step = int(args.chunk * SAMPLE_RATE)
        chunks = [audio[index : index + step] for index in range(0, len(audio), step)]
    else:
        chunks = [audio]

    if args.backend == "whisper":
        engine = whisper(args.model, args.threads)
        run = lambda chunk: transcribe_whisper(engine, chunk, args.language)
        label = f"whisper {args.model}"
    else:
        engine = parakeet(Path(args.dir), args.threads)
        run = lambda chunk: transcribe_parakeet(engine, chunk)
        label = "parakeet v3"

    def run_all() -> str:
        return " ".join(run(chunk) for chunk in chunks).strip()

    text = run_all()
    timings = []
    for _ in range(args.runs):
        started = time.perf_counter()
        text = run_all()
        timings.append(time.perf_counter() - started)

    mean = statistics.mean(timings)
    window = args.chunk or duration
    label = f"{label} | threads {args.threads} | window {window:.1f} s"
    print(f"{label} | audio {duration:.2f} s | chunks {len(chunks)}")
    print(f"  runs    {' '.join(f'{value:.2f}' for value in timings)}")
    print(f"  mean    {mean:.2f} s")
    print(f"  rtf     {mean / duration:.3f}")
    print(f"  text    {text}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
