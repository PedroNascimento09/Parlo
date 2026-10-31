# Benchmarks

Published numbers are the difference between a project people trust and a demo people stop believing. Parlo publishes what it measures and nothing else.

## Published results

No speech model is integrated yet, so there are no translation quality numbers to publish. The table stays empty until there is a real engine, rather than being filled with estimates.

| Hardware | Engine | Latency | Voice similarity | Quality | Version |
|---|---|---|---|---|---|
| pending | pending | pending | pending | pending | 0.1.0 |

## What gets measured

| Metric | Method |
|---|---|
| Latency | Timestamp the first captured audio frame and the first translated output frame, subtract, repeat at least 20 times, report median and 95th percentile. |
| Translation quality | Fixed test set, standard BLEU and COMET. Report the set, not just the score. |
| Voice similarity | Speaker embedding cosine similarity between input and output. |
| Resources | Peak resident memory, GPU memory, average CPU during a 5 minute call, and battery drain on a laptop. |
| Throughput | Frames processed per second with the echo engine, to prove the pipeline is not the bottleneck. |

## Hardware profiles

| Profile | Typical hardware | Expected mode |
|---|---|---|
| High | Apple Silicon with 16 GB or more, or a recent NVIDIA GPU | Hibiki-Zero, about 2 seconds |
| Medium | Modern laptop CPU, integrated GPU | Cascade with small models, about 4 seconds |
| Basic | Older CPU, no GPU | Cascade, noticeably slower, warn the user before install |

Detection happens on first run, and the user can override the profile. Reporting a high profile on basic hardware is the fastest way to lose trust.

## Reproduce the pipeline benchmark

The echo engine needs no model and no microphone:

```bash
cargo run --release -p parlo-cli -- demo out/demo.wav
```

On the reference machine used for development this processes 125 frames of 20 ms audio in well under a second, which confirms the pipeline itself adds no meaningful delay. The number that matters is the engine's, and that is what the table above will record.

## Submitting a result

Open a hardware report issue with your `parlo doctor` output, the command you ran, and the raw timings. Results are added here with the contributor's hardware named, because a benchmark without its hardware is a rumour.
