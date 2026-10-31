# Benchmarks

Published numbers are the difference between a project people trust and a demo people stop believing. Parlo publishes what it measures and nothing else.

## Published results

Reference machine: 12 core desktop CPU, no GPU, Windows 11, Python 3.12. Engine: Parakeet TDT 0.6B v3 int8 via sherpa-onnx, Argos Portuguese to English, Piper medium. Test audio: 4.95 s of Portuguese synthetic speech.

### Recogniser choice on modest hardware

To approximate an ordinary laptop, the recogniser was limited to 4 threads on the same 12 core machine. 4.95 s of Portuguese audio, 2.5 s segment window, five runs each, mean reported.

| Recogniser | Size | RTF | Transcript |
|---|---|---|---|
| **Parakeet v3** | 640 MB int8 | **0.113** | "Este é um teste do palo. Só a falar em... português para ver se a tradução funciona bem." |
| Whisper base | 145 MB int8 | 0.309 | "Oh, este é um teste do palo. Só vou falar em... Poto guias para ver se a tradução funciona bem." |

Whole utterance, without the 2.5 s window: Parakeet 0.092, Whisper base 0.153. Parakeet is both faster and more accurate here, and it does not invent the "Oh" that Whisper adds to a clipped first syllable.

With a fast recogniser the window becomes a latency choice, not a speed one. Sweep with Parakeet on the same 4.95 s, 4 threads:

| Window | RTF | Transcript |
|---|---|---|
| 1.0 s | 0.131 | "Uh. do Palo. a falar em português funciona bem." |
| 1.5 s | 0.111 | "Este é um teste do pá. Só a falar em português..." — loses a content word |
| 2.0 s | 0.108 | "Este é um teste do palo. a falar em português para ver se a tradução funciona bem." |
| 2.5 s | 0.094 | "Este é um teste do palo. Só a falar em... português para ver se a tradução funciona bem." |

**Decision taken:** the 2.5 s window stays. Cutting it to 1.5 s saves about 0.5 s of waiting and loses words, which is the wrong trade for a translator. The remaining latency is the window itself, so the fix is a sentence boundary (push to translate), not a smaller window.

Earlier measurements on the superseded Whisper-only engine, kept for comparison:

| Model | Threads | RTF | Segment | Transcript |
|---|---|---|---|---|
| tiny | 4 | **0.50** | 748 ms | "Oh, and full of Parlo's test..." |
| base | 4 | 0.56 | 1092 ms | "Oh, this is a Parlo test..." |
| small | 4 | 0.98 | 3042 ms | "Hello, this is Paul's test..." |

Segment window sweep with Whisper small on 12 threads:

| Window | Wall time for 4.95 s | RTF | Calls |
|---|---|---|---|
| 1.2 s | 13.14 s | 2.66 | 4 |
| 2.5 s | 4.76 s | 0.96 | 1 |
| 4.0 s | 5.66 s | 1.15 | 1 |

Per stage on a 4.95 s block: recognition 3.39 s, translation under 0.05 s, synthesis 0.28 s. Recognition is the cost; translation is not the problem.

**Decision taken:** Parakeet v3 is the default. It reaches 0.113 on 4 threads, which is 4.4 times under the 0.5 comfort target and 2.7 times faster than Whisper base at the same window, and its transcript keeps more words. `PARLO_CASCADE_STT=whisper` keeps faster-whisper as a fallback for machines where the 640 MB model is too much. The recogniser is no longer the bottleneck; the remaining latency is the 2.5 second window plus synthesis, which is why the next measurement has to be a live microphone.

Warm-up now loads every model at startup: 12 to 21 seconds before the first request, instead of the same wait on the first sentence.

What this means, plainly: **the real time gate is met with margin on the recogniser.** 0.113 leaves room for a second stream or a slower laptop. This says nothing yet about perceived latency, which also contains the window, capture, buffers and synthesis.

### Latency, corrected

An earlier version of this document estimated 3.6 seconds for the first translated word by adding the 2.5 second window to the processing time. **That is a floor, not an expectation.** It ignores microphone capture buffering, the operating system audio buffer, the time the synthesiser itself takes to speak, and whatever the calling app adds. The only way to know is a live capture, and the tool for it exists now:

```bash
parlo record out/voice-01.wav --seconds 6
```

### Measurement validity

Everything above uses synthetic Piper speech on the reference 12 core desktop. The comparison of Parakeet and Whisper proves a direction, not a final ranking. Slower machines are untested: limiting threads approximates core count, not clock speed, memory bandwidth, or thermal throttling, so a weak laptop is a known limit rather than a promise. The project targets the reference machine and does not gate on four core hardware.

The decisive test is ten recordings of a real voice, run through both recognisers:

```bash
python engines/cascade/scripts/compare.py out/voice --recognisers parakeet whisper --threads 4
```

### Memory

Peak resident memory for one Portuguese to English run with Parakeet, 4 threads: **1302 MB across three processes** (the CLI, the sidecar, and its worker), including the 640 MB int8 recogniser. The recogniser is now the largest single allocation, which matters for the 8 GB machines this is meant to run on.

```bash
python engines/cascade/scripts/memory.py out/pt.wav --threads 4
```

### Soak of continuous audio

Two minutes of looped speech through the real cascade, 4 threads: **86 segments, 522 ms mean, 618 ms p95, zero errors**. The first ten segments averaged 536 ms and the last ten 491 ms, so there is no upward drift; if anything the run warms up. That was the warm-up, and the thirty minute criterion needed the full run.

The full run: Windows 11, 4 threads, the cascade engine with Parakeet TDT 0.6B v3 int8, Argos and Piper, one 6 s Portuguese recording from a laptop Realtek microphone, looped without a break for thirty minutes. **30.0 minutes, 652 cycles, 1095 segments, 420.7 ms mean per segment, 417 ms median, 485 ms p95, zero errors.** The first ten cycles averaged 457.5 ms and the last ten 399.4 ms.

Reading: latency did not drift across half an hour. The last ten cycles were 58 ms faster than the first ten, and no cycle raised an error. This covers the audio continuity half of the v0.2 criterion; cold start, and a second person reproducing the same run, are still open.

```bash
python engines/cascade/scripts/soak.py out/voice/07.wav --minutes 30 --threads 4
```

Not measured yet, and blank on purpose: VRAM (there is no NVIDIA GPU here), perceived latency through a live microphone, BLEU and COMET, voice similarity. The test audio is synthetic speech from Piper, which the recogniser reads worse than a real voice; `Parlo` came out as `Palo`, which is what the glossary is for.

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
