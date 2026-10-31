# Benchmarks

Published numbers are the difference between a project people trust and a demo people stop believing. Parlo publishes what it measures and nothing else.

## Published results

First measured run of the CPU cascade, on the reference machine: a 12 core desktop CPU with no GPU, Windows 11, Python 3.12.

| Hardware | Engine | Latency | Voice similarity | Quality | Version |
|---|---|---|---|---|---|
| 12 core CPU, no GPU | cascade: whisper small int8, Argos pt-en, Piper medium | 2.6 to 2.7 s per 1.2 s segment; 11.5 s for 5 s of audio, warm | not measured | not measured, BLEU and COMET pending | 0.1.0 |

Cold start adds roughly 10 seconds for the first segment because the models load lazily. Voice similarity and translation quality are deliberately blank: nobody has measured them yet, and a guess would be worse than a gap.

The test audio was synthetic speech from Piper, which is harder for recognition than a real microphone. Real recordings will be added with the v0.2 test corpus.

## What gets measured

| Metric | Method |
|---|---|
| Latency | Timestamp the first captured audio frame and the first translated output frame, subtract, repeat at least 20 times, report median and 95th percentile. |
| Translation quality | Fixed test set, standard BLEU and COMET. Report the set, not just the score. |
| Voice similarity | Speaker embedding cosine similarity between input and output. |
| Resources | Peak resident memory, GPU memory, average CPU during a 5 minute call, and battery drain on a laptop. |
| Throughput | Frames processed per second with the echo engine, to prove the pipeline is not the bottleneck. |

## Hardware profiles

Design targets, measured before publishing. The reference machine for the project is an ordinary PC without an NVIDIA GPU, on purpose: if Parlo is good there, it is good for most people.

| Profile | Reference hardware | Mode | Latency target |
|---|---|---|---|
| Ultra | NVIDIA 12 GB or more, or recent Apple Silicon | Cascade level 3, or Hibiki-Zero | 2 to 3 s |
| High | Any GPU with 6 GB or more, or Apple Silicon | Cascade level 2 to 3 | 2.5 to 4 s |
| Medium | 8+ core CPU, 16 GB RAM, weak or no GPU | Cascade on CPU, level 1 to 2 | 3 to 5 s |
| Basic | 4 core CPU, 8 GB RAM, older laptop | Captions or light neutral voice | Captions under 2 s, voice 5 to 8 s |

Detection happens on first run. The current prototype reads CPU cores only, and says so in `parlo profile`; GPU and memory detection arrive with v0.2. Reporting a high profile on basic hardware is the fastest way to lose trust.

## Reproduce the pipeline benchmark

The echo engine needs no model and no microphone:

```bash
cargo run --release -p parlo-cli -- demo out/demo.wav
```

On the reference machine used for development this processes 125 frames of 20 ms audio in well under a second, which confirms the pipeline itself adds no meaningful delay. The number that matters is the engine's, and that is what the table above will record.

## Submitting a result

Open a hardware report issue with your `parlo doctor` output, the command you ran, and the raw timings. Results are added here with the contributor's hardware named, because a benchmark without its hardware is a rumour.
