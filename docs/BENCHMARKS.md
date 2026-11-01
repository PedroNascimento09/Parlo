# Benchmarks

Published numbers are the difference between a project people trust and a demo people stop believing. Parlo publishes what it measures and nothing else.

## Published results

Reference machine: 12 core desktop CPU, no GPU, Windows 11, Python 3.12. Engine: Parakeet TDT 0.6B v3 int8 via sherpa-onnx, the OPUS-MT Portuguese to English package, Piper medium. Test audio: 4.95 s of Portuguese synthetic speech.

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

### Recognition battery

The reference battery synthesises 24 Portuguese sentences with the Portuguese Piper voice, adds seeded white noise, and scores recognition only, so a regression shows up as a number without anyone recording anything. Windows 11, 4 threads, Parakeet TDT 0.6B v3 int8, seed 7.

| Condition | Sentences | Mean word error rate |
|---|---|---|
| Clean | 24 | 0.215 |
| 20 dB signal to noise | 24 | 0.199 |
| 10 dB signal to noise | 24 | 0.197 |

The rate counts a digit against a written number as an error even when the recogniser heard it correctly, so the sentences with a number in them carry the worst rows; the number is a regression signal, not a quality grade. Adding noise did not raise the mean here, which is a sign the noise is mild against this recogniser rather than a claim that noise does not matter. A recorded set from several speakers is still needed before v1.0.

### Two recognisers on the same battery

The same battery, the same seed and the same 4 thread limit were run against both engines. Only the flag that chooses the recogniser changed, which is the point: a comparison is fair when the command around the variable is identical.

```bash
python engines/cascade/scripts/battery.py --stt parakeet --out out/battery-parakeet.json
python engines/cascade/scripts/battery.py --stt whisper --model small --out out/battery-whisper.json
```

| Recogniser | Clean | 20 dB signal to noise | 10 dB signal to noise |
|---|---|---|---|
| **Parakeet TDT 0.6B v3 int8** | **0.215** | **0.199** | **0.197** |
| Whisper small int8 | 0.217 | 0.242 | 0.309 |

On clean speech the two are within a fifth of a percent of each other, which is noise rather than a ranking. The difference appears as the audio gets worse: Whisper small climbs with the noise and Parakeet does not, which matches the earlier finding that Whisper invents words at the start of a clipped syllable. Whisper base, measured on six sentences only before this run, sat around 0.329 clean, so the size of the model matters more than the family.

Parakeet stays the default for two reasons: it holds its rate under noise, and it is about three times faster per the real time factor table above. Whisper stays selectable through `--stt whisper` for machines where the 640 MB model is too much.

### Streaming, partial to final

The cascade publishes text in four states: partial, stable, final and corrected. The recogniser reads a growing buffer every 0.4 s of audio and confirms a word only when the last two hypotheses agree on it (LocalAgreement, the policy section 4.3 of the plan names for models that cannot stream natively); the buffer closes after 0.45 s of silence or 12 s of audio. The mode is reported as `reprocess`, not native streaming, because every decode re-reads the whole segment.

Corpus: 20 European Portuguese sentences written for this test (`engines/cascade/tests/corpus/pt_pt.txt`), synthesised with the Piper `pt_PT-tugão-medium` voice. Windows 11, 4 threads, decode every 0.4 s, two agreeing hypotheses before a word is stable.

| Recogniser | First partial | First stable | Final | p95 final | Final WER | Stable coverage |
|---|---|---|---|---|---|---|
| **Parakeet TDT 0.6B v3 int8** | 849 ms | 1978 ms | 3355 ms | 3918 ms | **0.231** | 81.1% |
| Whisper base int8 | 887 ms | 1951 ms | 3379 ms | 3907 ms | **0.279** | 72.8% |

Reading: the first provisional text lands about 0.9 s after speech starts, the first confirmed words after about 2.0 s, and the finished sentence about 3.4 s after it began. Parakeet scored 0.231 against 0.279 for Whisper base, which is the first time the two separate by more than noise on this corpus; at 20 sentences that is a direction, not a ranking, and the real time factor table above favours Parakeet by roughly three times regardless.

The stable text is a prefix of the sentence, so its word error rate counts the words it has not reached yet as errors. Coverage is the honest companion: 81.1% of the sentence is confirmed when the segment closes, which is what someone dictating would see as settled text. The earlier policy, which required a word to repeat across decodes before accepting it, reached 67.3%; local agreement is both simpler and better here.

Agreement window and decode cadence, Parakeet, the same 20 sentences:

| Policy window | Decode every | First partial | First stable | Final | Stable coverage | Final WER |
|---|---|---|---|---|---|---|
| **LocalAgreement 2** | **0.4 s** | **849 ms** | **1978 ms** | **3355 ms** | **81.1%** | **0.231** |
| LocalAgreement 2 | 0.6 s | 921 ms | 2299 ms | 3349 ms | 64.2% | 0.209 |
| LocalAgreement 3 | 0.4 s | 835 ms | 2409 ms | 3367 ms | 42.2% | 0.222 |
| LocalAgreement 3 | 0.6 s | 937 ms | 2732 ms | 3391 ms | 21.2% | 0.268 |

**Decision taken:** two hypotheses and a 0.4 s cadence stay. Waiting for a third hypothesis halves the confirmed text and adds half a second to the first stable words for a final rate inside the noise; decoding less often saves CPU and costs both latency and coverage.

The voice matters more than the recogniser: the same 20 sentences synthesised with the Brazilian `pt_BR-faber-medium` voice scored 0.67 and 0.65 final WER, against 0.24 and 0.22 with the European voice. A recogniser cannot be judged on PT-PT with a PT-BR voice.

The spread inside a recogniser is wide: the best sentence scored 0.0 and the worst 0.8, so a mean of 0.23 is not a per sentence guarantee.

```bash
python engines/cascade/scripts/setup_models.py --voice pt_pt
python engines/cascade/scripts/stream_bench.py --voice pt_PT-tugão-medium.onnx --out out/stream.json
```

Limits: synthetic speech from one voice, no noise, no accents, no real speakers. G0 asks for streaming recognition of real PT-PT speech without accumulating delay; what is measured here is a windowed reprocessing pipeline on synthetic audio. Nemotron 3.5 ASR and Qwen3-ASR are untested, so the plan's streaming-native candidate is still open.

### A segment on the current backend

`python engines/cascade/scripts/soak.py out/voice/01.wav --minutes 2 --threads 4`, Windows 11 laptop, no GPU, 5.99 s of Portuguese synthetic speech at 48 kHz resampled to 16 kHz, recognition and voice synthesis included, 29 passes over the clip:

| Segments | Mean | Median | p95 | First ten | Last ten | Errors |
|---|---|---|---|---|---|---|
| 66 | **699 ms** | 698 ms | 940 ms | 730 ms | 717 ms | 0 |

The mean does not drift between the start and the end of the run. It is higher than the 420.7 ms published before, and those two are not a before and after: they were taken through the same harness around a different translation stage and a different voice arrangement. This is what the current program costs end to end on one clip, and the translation inside it is the 114 ms table above.

### Captions latency

Captions skip the synthesiser, so the wait is the window plus recognition and translation. Warm Parakeet, 1.2 s window, three consecutive segments of the same 6.6 s Portuguese clip: **1266, 1024 and 958 ms of processing**, which puts a caption about **2.2 s** after the speech starts.

The basic profile targets captions under two seconds, and this window does not reach it. The honest fix is a shorter window, bought with accuracy, not a larger claim: the measurement above is what a person will feel, and it is published as it is.

```bash
parlo captions --window 1.2 --seconds 20
```

### Cold start

The first start after a boot, Windows 11, with no model in memory: **18.1 s** from process start to the translated file being written, for one 6 s Portuguese clip from a laptop Realtek microphone.

```bash
time parlo translate out/voice/01.wav out/cold.wav --engine cascade
```

That figure contains spawning the Python sidecar, loading Parakeet TDT 0.6B v3 int8, Argos and Piper, and translating the clip. It is not a per segment number: in the warm steady state a segment costs 420.7 ms, as the soak above shows. The app keeps one warmed process for a whole conversation, so this describes the wait before the first sentence of a session, not how the translator feels once it is running.

Not measured yet, and blank on purpose: VRAM (there is no NVIDIA GPU here), perceived latency through a live microphone, BLEU and COMET, voice similarity. The test audio is synthetic speech from Piper, which the recogniser reads worse than a real voice; `Parlo` came out as `Palo`, which is what the glossary is for.

### Translation on the corpus

`benchmarks/corpus/translation.jsonl` holds 24 sentence pairs written for this project, 12 in each direction, with times, dates, numbers, negations, clitics, reported speech and idioms. `engines/cascade/scripts/mt_bench.py` runs one checkpoint over one direction and prints every row, so any number below arrives with the text that produced it.

```bash
python engines/cascade/scripts/mt_bench.py --model engines/cascade/models/mt/en-pt/ct2-fixed --direction en-pt --repetition-penalty 1.2 --no-repeat-ngram-size 3
python engines/cascade/scripts/mt_bench.py --model engines/cascade/models/mt/pt-en/ct2-fixed --direction pt-en --repetition-penalty 1.2 --no-repeat-ngram-size 3
```

Windows 11, 4 threads, CTranslate2 int8, beam 4.

| Direction | Checkpoint | Build | Prefix | Median | p95 | Leading clause right | Whole output clean |
|---|---|---|---|---|---|---|---|
| EN→PT | `opus-mt-tc-big-en-pt` | CT2 int8, converted here | none | 851 ms | 1665 ms | 11 of 12 | **0 of 12** |
| PT→EN | `opus-mt-ROMANCE-en` | CT2 int8, converted here | none | 1521 ms | 4338 ms | 9 of 12 | **0 of 12** |
| PT→EN | `opus-mt-ROMANCE-en` | CT2 int8, build already in the tree | `>>por<<` | 711 ms | 921 ms | 9 of 12 | **0 of 12** |

Every output begins with a correct or near-correct clause and then keeps going: the model paraphrases the same sentence again and again instead of stopping. That is not a translation quality problem, it is a generation that never reaches an end token, and it makes every row unusable as a sentence.

```
src  I'll send the invoice by Friday.
ref  Envio a fatura até sexta-feira.
out  Vou enviar a fatura até sexta-feira. Eu vou enviar a factura até sexta sexta- sexta.
     I envio a factura na sexta- feira. eu vou enviar até sexta..
```

**The checkpoint is not the problem.** The same weights run through transformers produce one clean sentence in all five probes:

| Probe | Hugging Face weights | CTranslate2 build |
|---|---|---|
| We don't have the budget for that. | Não temos orçamento para isso. | Spanish and repetition |
| The deadline is 31 December 2026. | O prazo é 31 de dezembro de 2026. | Fragments and repetition |
| Não faz sentido adiar outra vez. | It doesn't make sense to postpone it again. | Nine words, then garbage |
| O prazo termina a 31 de dezembro de 2026. | The deadline ends on 31 December 2026. | Correct clause, then garbage |
| A casa de banho fica ao fundo do corredor. | The bathroom is at the bottom of the hall. `>>by<<` | Correct clause, then garbage |

The CT2 builds in the tree were mis-converted: both were 79 MB, which cannot be right for a 300 M parameter model next to a 77 M one, and reconverting the EN→PT checkpoint from its own weights gives 235 MB. Both builds also lose the end token, so decoding runs until the length cap turns into invented paraphrases.

The mechanism, narrowed down on 1 November: the converted config starts decoding on `</s>` where the checkpoint starts on its pad token, and the pad token is missing from the shared vocabulary the converter wrote, although the weights still carry its row. Appending it by hand crashes the translator with a segmentation fault, which says the vocabulary size is baked into the model binary: the fix belongs in the conversion, not in a patched text file.

What this corrects, in the plan and in `docs/LICENSES.md`: EN→PT was recorded as a checkpoint that degenerates with every language prefix. It degenerates **after conversion**. In transformers the same weights answer five of five probes cleanly, which is the first working EN→PT evidence this project has. It is not yet a translated pair: five probes are not a corpus, and nothing is published as a pair until the conversion is fixed and the 12 sentence table comes back clean.

The leaked `>>by<<` on the PT→EN probe is the language prefix coming out the other side, which says the prefix placement is also still wrong.

### The same corpus on the original weights

`engines/cascade/scripts/mt_hf_bench.py` runs the same 24 sentences through the checkpoints with transformers, beam 4, 4 threads, Windows 11. This measures the models, not the conversion.

| Direction | Checkpoint | Prefix | Median | p95 | Complete sentences | Correct meaning, PT-PT wording |
|---|---|---|---|---|---|---|
| EN→PT | `opus-mt-tc-big-en-pt` | none | 1025 ms | 1130 ms | **12 of 12** | 9 of 12 |
| PT→EN | `opus-mt-ROMANCE-en` | `>>por<<` | 878 ms | 1185 ms | **12 of 12** | 12 of 12 |

This is the first time EN→PT produces a finished Portuguese sentence on every row of this corpus:

```
src  We don't have the budget for that.        →  Não temos orçamento para isso.
src  She said she would call back later.       →  Ela disse que ligaria mais tarde.
src  It costs 249.90 euros, VAT included.      →  Custa 249,90 euros, IVA incluído.
src  Can you confirm the meeting on Thursday?  →  Pode confirmar a reunião na quinta-feira?
```

Four things it gets wrong, all of them worth writing down:

- **Brazilian words and literal idioms.** `The train` becomes `O trem` where European Portuguese says `O comboio`, and `The lift is out of order` becomes `O elevador está fora de ordem`, which is the words in order rather than the Portuguese phrase `está avariado`.
- **`ship` as a verb becomes `navio`.** `Let's keep it simple and ship today` came back as `Vamos mantê-lo simples e navio hoje`: the core problem of a pair that is not curated, and the reason a human read is still part of every gate.
- **Names are translated.** `O João e a Maria` became `John and Mary`, which loses the only part of a sentence a person cannot guess.
- **The language prefix leaks on PT→EN.** Five of twelve outputs end in `>>by<<`, `>>per<<` or `>>for<<` after a correct sentence. Stripping the trailing prefix is a fix in the adapter, not in the model.

Reading: the checkpoint is good enough to be the EN→PT candidate and not good enough to be published as a validated pair. Nine of twelve idiomatic on twelve sentences written by the person who built the corpus is a direction, not a grade, and the three defects above are exactly what the plan's gates ask for: European vocabulary, preserved names, and no prefix left in the output.

Latency sets the other limit: about a second per sentence, four threads, beam 4. That is fine for dictation and too slow for Live, where the target is 2.5 seconds for the whole first utterance including recognition and voice.

Beam 1, same corpus, same threads:

| Direction | Beam 4 median | Beam 1 median | Beam 1 p95 | Complete sentences | Rows whose wording got worse |
|---|---|---|---|---|---|
| EN→PT | 1025 ms | **398 ms** | 500 ms | 12 of 12 | 1 of 12 |
| PT→EN | 878 ms | **450 ms** | 539 ms | 12 of 12 | 0 of 12 |

The only row that changed for the worse was `Let's keep it simple and ship today`: beam 4 gave `Vamos mantê-lo simples e navio hoje` and beam 1 gave `Vamos manter a simplicidade e navegar hoje`. Both mistranslate `ship`; beam 1 is more readable while still wrong, which is exactly the kind of improvement that must not be mistaken for correctness. Two and a half times faster for one paragraph worse in twelve is a trade worth taking for a caption and not yet justified for a sentence someone will send to a client.

Rule that survives this: a number from a broken build is worse than no number, because it closes a question that is still open.

### The installed OPUS-MT package, read without the Argos runtime

`parlo_cascade/mt.py` reads a package's CTranslate2 model and its SentencePiece vocabulary directly, so the model runs with no Argos code on the path. Same 24 sentence corpus, Windows 11 laptop with no NVIDIA GPU, 4 threads, beam 4, warm process:

```bash
python engines/cascade/scripts/mt_bench.py --pack --direction pt-en
python engines/cascade/scripts/mt_bench.py --pack --direction en-pt
```

| Direction | Package | Median | p95 | Complete sentences | Wording a person would accept |
|---|---|---|---|---|---|
| PT→EN | `translate-pt_en-1_9` | **114 ms** | 131 ms | 12 of 12 | 9 of 12 |
| EN→PT | `translate-en_pt-1_9` | **111 ms** | 128 ms | 12 of 12 | 10 of 12 |

The same corpus on the OPUS-MT weights through transformers takes 450 and 398 ms at beam 1, and the CTranslate2 builds converted in the tree repeat themselves until the length cap. The package is about four times faster than the weights it was built from and, unlike that conversion, stops at the end of the sentence.

What it still gets wrong, four rows out of twenty four:

- **Names.** `O João e a Maria vão chegar atrasados.` → `John and Maria are gonna be late.`
- **A dropped clause.** `Fica combinado: encontramo-nos à porta às 9h.` → `We'll meet at the door at 9:00.`
- **A guessed pronoun.** `Enviei-lhe o ficheiro por correio eletrónico.` → `I e-mailed him the file.`, where the sentence says `you`.
- **Old spelling.** `I'll send the invoice by Friday.` → `Mando a factura até sexta-feira.`

Reading: about 110 ms per sentence, model alone, is the first translation figure this project has that fits Live, where the whole first utterance with recognition and voice targets 2.5 s. It is not a pipeline number and it says nothing about a cold process. The package licence is CC BY 4.0 with the authors named in `catalog/models.json`; the runtime that ships it is not needed and not used.

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
