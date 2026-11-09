# Benchmarks

Published numbers are the difference between a project people trust and a demo people stop believing. Parlo publishes what it measures and nothing else.

## Published results

Reference machine: 12 core desktop CPU, no GPU, Windows 11, Python 3.12. Engine: Parakeet TDT 0.6B v3 int8 via sherpa-onnx, the OPUS-MT Portuguese to English package, Piper medium. Test audio: 4.95 s of Portuguese synthetic speech.

### Dictation hot path, audio end to text

`parlo bench dictate` decodes a fixed 5.00 s clip five times on four threads and prints the median of each stage. The default clip is speech: 1.10 s of silence, 2.81 s of recorded Portuguese from `engines/cascade/tests/fixtures/timing/source.wav`, and 1.09 s of silence. It decodes to words ("Bom dia, uma prova dos modos de tempo do Dói."), so post-processing cleans a real sentence and is measured instead of skipped.

| Stage, speech clip | Trimming off | Trimming on |
|---|---|---|
| Load, cold recogniser, once per process | 2604.7 ms | 2604.7 ms |
| Warm-up, first write, not counted | 374.8 ms | 374.8 ms |
| Decode | 567.0 ms | 373.9 ms |
| Post-processing | 0.8 ms | 0.9 ms |
| **Audio end to text** | **569.9 ms** | **375.0 ms** |

The two columns come from one process, five runs each, recorded in `docs/benchmarks/dictate-speech.json` at commit 669ddf7 with the machine. Load and warm-up are paid once and are not part of the per-dictation total. Two further sessions measured 349.5 ms and 361.9 ms against controls of 528.6 ms and 561.6 ms, so the three sessions were 34.2%, 33.9% and 35.6% faster with trimming (median 34.2%). The recorded baseline is 375.0 ms.

What changed: the engine trims leading and trailing silence before the encoder, at a -50 dBFS floor relative to the clip peak, with a 0.25 s margin and 20 ms frames. A clip where no frame clears the floor is left alone, so a quiet recording is not mistaken for silence. Threads were already at their best value: on the synthetic clip 2 threads cost 715 ms, 4 cost 555 ms, 8 cost 743 ms and 12 cost 1518 ms, which says the decode is CPU bound and 4 is the knee.

A machine under load stretches every run in a session, which is why the ruler also times the clip with trimming off. The control and the measured run share the conditions, so the percentage survives a busy laptop even when the absolute number does not.

The synthetic tone is still there with `--clip tone`, with its own baseline in `docs/benchmarks/dictate.json`: 304.3 ms in a later session against the recorded 567.1 ms, and a post stage of 0 ms because the tone decodes to no words.

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

### What each model costs on this machine

Size on disk says nothing about what a model does to the computer when it runs. `engines/cascade/scripts/model_memory.py` loads one unit, makes a warm call, then reports the memory the process gained and how long the work took. Measured on Windows 11, 4 threads, one process per unit, before and after a garbage collection:

| Unit | On disk | In memory | Speed | Measured on |
|---|---|---|---|---|
| Parakeet TDT 0.6B v3 int8 | 671 MB | 746 MB | 0.206 real time | one 6.35 s FalAR parliamentary clip |
| Whisper base int8 | 145 MB | 148 MB | 0.239 real time | the same clip |
| Voice pt-PT (Tugão) | 63 MB | 2 MB | 0.157 real time | one 2.33 s sentence |
| Voice en-US (Lessac) | 63 MB | 2 MB | 0.119 real time | one 3.58 s sentence |
| OPUS-MT pt to en | 83 MB | 112 MB | 278 ms per sentence | one business sentence |
| OPUS-MT en to pt | 83 MB | 111 MB | 510 ms per sentence | the same sentence, other direction |

Reading: the recogniser is the only model that costs real memory, which is why the first-start suggestion treats it as the one that has to be affordable. The voices cost almost nothing to hold and little to run. The translation pairs sit in between, and the en to pt direction is nearly twice as slow as the other.

The measurement is one warm call, so it is an order of magnitude and not a corpus average. Memory is the change in resident set size, not a peak. The clip is a real recording; the sentences are the project's own.

That run also produced a visible failure worth keeping: en to pt turned *the delivery deadline is day 23 of March* into *o prato de entrega é dia vinte e três de março*, where "deadline" became "prato". The pair's gate scores stay where they are; the sentence is a reminder that a count is not a reading.

```bash
python engines/cascade/scripts/model_memory.py --unit parakeet
python engines/cascade/scripts/model_memory.py --unit mt-en-pt
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

The cascade publishes text in four states: partial, stable, final and corrected. The recogniser reads a growing buffer every 0.4 s of audio and confirms a word only when the last two hypotheses agree on it (LocalAgreement, the policy section 4.3 of the plan names for models that cannot stream natively); the buffer closes after 0.45 s of silence or the segment window, 12 s in the tables below and 3 s in the app. The mode is reported as `reprocess`, not native streaming, because every decode re-reads the whole segment.

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

The close of a sentence was measured on its own, because it is the moment a person notices. A close that arrives on silence used to decode the same audio it had already read; two builds of the engine, one before and one after that close reuses its last hypothesis, read the same 20 rendered clips (`--write-clips`, so both processes see identical audio), segment 3 s, decode every 0.4 s, 4 threads, `pt_PT-tugão-medium`:

| Build | Final latency at a silence close | Decodes per sentence | Loop wall time per sentence | Final WER | Finals that changed |
|---|---|---|---|---|---|
| Before (`50aa1ea`) | **285.4 ms** | 7.20 | 1328.2 ms | **0.218** | — |
| After (`be5d415`) | **0 ms** | 6.45 | 1117.4 ms | **0.218** | 1 of 20 |

Reading: a sentence that ends on silence costs no decode at all now, 285.4 ms of recogniser time removed from the close, 10.4% fewer decodes and 210.8 ms less wall time per sentence across the whole stream. The first final of a long sentence is still a segment close, so the p95 first final is unchanged (3401 against 3404 ms); what changed is the moment the sentence actually ends. Quality is level: the same mean WER to three decimals, the same 163 words, and the one final that differs scores the same error rate on both builds. `stream_bench.py` now reports the kind of every close (silence, segment or end of audio) and how many decodes each sentence took, so this is re-run rather than believed.

The island's request cadence was measured the same way, then changed from 600 ms to 300 ms. Two runs of each cadence, interleaved in time so a warm machine cannot favour either side, the same 20 rendered clips, segment 3 s, decode every 0.4 s, 4 threads, `pt_PT-tugão-medium`, and the session closed at the end of every sentence:

| Island request cadence | First partial | First stable | Final | p95 final | Decodes per sentence | Loop wall time per sentence | Final WER | Closes on silence |
|---|---|---|---|---|---|---|---|---|
| 600 ms | 935.0 ms | **1861.8 ms** | 2998.9 ms | 3331 ms | **5.0** | 1089.8 ms | **0.219** | 2 of 20 |
| **300 ms** | **784.4 ms** | 2041.9 ms | **2849.2 ms** | **3223 ms** | 5.3 | **975.0 ms** | 0.233 | 10 of 20 |

Reading: the cadence moves two things in opposite directions. Asking every 300 ms brings the first provisional text 150.6 ms sooner (16.0%) and the finished sentence 149.7 ms sooner (5.0%), and drops the p95 final by 108 ms, because the session now notices silence within 300 ms and ten sentences close on silence instead of two. The confirmed words arrive 180.1 ms later (9.7%), because the finer grid gave one extra decode on average (5.3 against 5.0 per sentence) and stability lands one decode later. The extra decodes cost 6.0% more recogniser calls per sentence, and the whole loop still ended 10.5% sooner (114.8 ms per sentence) because more sentences closed early. Quality is inside one word: 0.233 against 0.219 over 20 sentences, one final reads differently.

**Decision taken:** 300 ms stays. Someone dictating feels the release to text interval and the provisional words, both of which improved; the later first confirmation is the price, and one word in 20 sentences is within the spread of a recogniser this size.

### The latency the app itself measures

The island times the dictation itself: `performance.now() - releasedAt`, from letting go of the key to the sentence being ready, and sends that number with the insertion, where the server prints it as `the dictation landed in N ms after the key was released`. That path is exercised by `cargo test -p parlo-cli --test dictation_latency -- --ignored`, which starts the real `parlo serve` binary and drives it over HTTP the way the island does. Fixture `engines/cascade/tests/fixtures/timing/source.wav`, 2.8 s of European Portuguese speech, cascade engine with Parakeet TDT int8, 4 threads, temp data folder, default configuration (cleanup mode `auto`, `light`, no local polish server and no cloud key, so the cleanup stage runs the deterministic rules only):

| Path | Release to text | The server's own `latency_ms` |
|---|---|---|
| Polish off, streamed sentence | **2 ms** | 1 ms |
| Polish on, 150 ms model, streamed sentence | **2 ms to the raw text, polished text 155 ms later** | — |
| Polish on, insert and replacement recorded by the app's sink | raw write first, the cleaned text replaces it 159 ms later | — |
| Polish off, audio fallback | 362 ms | 352 ms |
| First dictation after start, streamed | **2 ms** | 1 ms |
| Polish on, 150 ms model, audio fallback | 503 ms | — |
| Polish on, 1.2 s model, the polish call itself | 1205 ms, past the 400 ms grace, so the raw text stands | — |

The island no longer transcribes the audio a second time when the stream has already closed the sentence. A sentence that ends on 0.45 s of silence is finalised while the person is still holding the key, so letting go sends that text and asks only for the cleanup: the answer comes back in single digit milliseconds, 319 ms sooner than the audio fallback. The audio path stays as the fallback, and it is still what runs when the stream has no final sentence, when the last words arrived after the close, or when the dictation is being translated, because a text body can only be written in the language it was spoken.

Cleanup never shortens the insert. The two streamed rows are one code path on one clip, and the 1 ms between them is run-to-run noise: with Polish off the insert carries the raw sentence, with Polish on it carries the same raw sentence and the cleaned one follows 154 ms later, which is the model's own time. No row here has the cleaned text arriving before the raw text.

The polish never sits in front of the text. The island writes the raw sentence, then waits up to 400 ms for the cleanup and rewrites the sentence it just wrote when the answer arrives, so a model that answers in 150 ms fills in behind an insertion that took 2 ms. Past the grace window the answer is dropped and the raw text stands, which is what happens with a deliberately slow model (a loopback server that answers after 1.2 s): the call still takes 1205 ms and the sentence in the field never waited for it. The model is warmed at server start, so the first dictation does not pay a load either: the test sees the cleanup stage called 3 times before the first dictation in the fast run.

Two things had to be fixed to get here, and both were real bugs rather than setup. With cleanup off, the intelligence route ignored the mode and still called a configured model, so Polish off waited 1549 ms for a model it was not supposed to use. And the deterministic cleaner read a clarification marker at the start of a sentence as a self-correction and deleted what came before it, so "Bom dia. Isto é uma prova" came back as "uma prova", which made every polished dictation lose its opening.

The replacement itself is exercised, not assumed. Insertion goes through one `Field` the server selects at start: the real one pastes into the captured window, and a recording one appends each write and erase to a file when `PARLO_FIELD_LOG` is set, so the test drives the real `parlo serve` without typing into any window. With the 150 ms model the log holds three events: the raw sentence at 0 ms, then an erase of one character and a write of `!` at 158 ms and 159 ms, and the field rebuilt from them is exactly the cleaned sentence asserted against the raw one. With the 1.2 s model the log holds the raw write alone, because the polish arrives at 1205 ms, past the grace. A sink that refuses the erase ("PARLO_FIELD_REJECT=1") leaves the same single write, so a failed replacement writes nothing extra and nothing twice.

The first dictation no longer pays the engine load where it hurts. The recogniser takes 8671 ms to load on this machine, and it loads while the person is speaking, because the stream reads the audio as it arrives; letting go then costs the same 5 ms as any later dictation. Only a dictation that arrives with the engine cold and cannot use the stream, such as a translated one, still meets the load.

All of it is on a 2.8 s clip followed by 0.6 s of silence, fed in 300 ms chunks with a 3 s stream window, the same cadence the island uses. The slow model is a local test server, not a real model, so its speed is the point and not its English.

Limit: the `/insert` request is not exercised by the test, because inserting would type into whichever window has the focus. The island's measurement ends before `/insert`, so the interval measured here is the one the app prints, minus that last round trip.

Limits: synthetic speech from one voice, no noise, no accents, no real speakers. The next section repeats the measurement on recordings.

### Recorded European Portuguese speech

Every number above replays one synthetic voice. This is the first measurement on people speaking. The recordings are **FalAR** (`inesc-id/FalAR`, CC-BY-4.0), European Portuguese parliamentary sessions: `scripts/falar_clips.py` takes a parquet shard, keeps the segments whose corpus character error rate is at or below 0.05 and whose duration sits between 2 and 12 s, and writes 16 kHz mono clips with a manifest. Ten segments, 66.6 s, four speakers.

Windows 11, 4 threads, decode every 0.4 s, segment window raised to 30 s so a long clip is not cut, Parakeet TDT 0.6B v3 int8 against Whisper base int8:

| Recogniser | First partial | First stable | Final | p95 final | Final WER | Stable coverage | Real time factor | Corrections |
|---|---|---|---|---|---|---|---|---|
| **Parakeet TDT 0.6B v3 int8** | 1077 ms | 2419 ms | 6506 ms | 11 949 ms | **0.243** | 84.4% | **1.023** | 2 |
| Whisper base int8 | 3530 ms | 2995 ms | 6841 ms | 11 818 ms | 0.377 | 78.3% | 3.274 | 7 |

Reading: parliamentary speech is cleaner than an office and harder than a read corpus. One voice at a time and no crosstalk, but long clauses, names, numbers and dates. Parakeet transcribes at about real time on this machine (1.023) and stays usable; Whisper base at 3.274 cannot keep up, and its first partial appears only at 3.5 s, later than its first stable words at 2.4 s, because its first decode outlasts the audio it is reading.

The mean hides a wide spread. Parakeet's per clip rate runs from 0.00 to 0.96 with a median of 0.125, so most segments are near perfect and two carry nearly all the error: a 11.7 s clip came back as one wrong phrase, and Whisper turned the same clip into a single phrase of its own. Whisper's median is 0.455, which summarises its run better than its mean.

Nothing accumulates for the recogniser that keeps up: Parakeet's decode time over the last third of the run is x0.84 of the first and its real time factor x0.80, both below one. Whisper's grew slightly (x1.095 and x1.136), which is the backlog of a recogniser already past real time.

```bash
python engines/cascade/scripts/falar_clips.py --parquet test-00000-of-00024.parquet --out out/falar/clips
python engines/cascade/scripts/stream_bench.py --audio-dir out/falar/clips --segment 30 --out out/falar/real.json
```

Limits: ten segments from one shard, one parliamentary register, one recording setup, no added noise and no office acoustics. The corpus transcripts carry no punctuation, which suits a word error rate and says nothing about capitalisation. The 30 s segment window is not the 12 s the product uses, so these real time factors compare inside this table and not with the synthetic ones above. Nemotron 3.5 ASR and Qwen3-ASR could not be measured here, for the reasons in `docs/ENGINES.md`, which also records the decision that follows.

### Latency across a whole stream

The G0 criterion is not one sentence's latency but that it does not grow. `stream_bench.py` now reports the drift: the mean of the first third of the run against the last third, for the decode time and for the real time factor, and it exits non-zero when the real time factor grows past 1.25 over the run.

20 sentences, Parakeet TDT 0.6B v3 int8, `pt_PT-tugão-medium`, Windows 11, 4 threads, decode every 0.4 s, 12 s segment:

| Measure | Value |
|---|---|
| Decode time, last third against first third | **x0.988** |
| Real time factor, last third against first third | **x0.971** |
| Real time factor, mean over the run | 1.222 |
| Final, first sentence then last | 3164 ms then 3763 ms |
| First partial, final and p95 final | 958.8 ms, 3601.5 ms, 4165 ms |
| Final WER, stable coverage, corrections | 0.222, 72%, 3 |

Reading: both ratios sit below one, so the work per second of audio did not grow as the run went on and nothing accumulated. The first and last final differ by about 600 ms because the sentences are not the same length; that is why the comparison is between the thirds of the run and not between two samples.

Limits: this is audio replayed as fast as the harness can feed it, not a live microphone, so it proves the pipeline does not grow its own backlog and says nothing about a device that drops frames. The audio is still synthetic; the real-speaker part of G0 remains open.

### A segment on the current backend

`python engines/cascade/scripts/soak.py out/voice/01.wav --minutes 2 --threads 4`, Windows 11 laptop, no GPU, 5.99 s of Portuguese synthetic speech at 48 kHz resampled to 16 kHz, recognition and voice synthesis included, 29 passes over the clip:

| Segments | Mean | Median | p95 | First ten | Last ten | Errors |
|---|---|---|---|---|---|---|
| 66 | **699 ms** | 698 ms | 940 ms | 730 ms | 717 ms | 0 |

The mean does not drift between the start and the end of the run. It is higher than the 420.7 ms published before, and those two are not a before and after: they were taken through the same harness around a different translation stage and a different voice arrangement. This is what the current program costs end to end on one clip, and the translation inside it is the 114 ms table above.

### Captions latency

Captions skip the synthesiser, so the wait is the window plus recognition and translation. Measured again on 1 November after translation moved off the Argos runtime, warm Parakeet, the 20 sentence PT-PT corpus, audio fed in 100 ms blocks:

| Window | Sentences | Processing mean | Median | Max | Caption after speech starts |
|---|---|---|---|---|---|
| 1.2 s | 20 | **444 ms** | 433 ms | 578 ms | **about 1.6 s** |
| 2.5 s | 20 | 844 ms | 835 ms | 1122 ms | about 3.3 s |

The window is what dominates: a caption cannot appear before the window closes, and the 1.2 s row puts one on screen at roughly 1.6 s, inside the two second target the earlier run missed. That earlier run measured **2.2 s** with the Argos translation stage taking about a second of it; the translation now costs about a tenth of that.

The number is window plus processing, not a stopwatch in front of a screen: processing is measured, the window is arithmetic, and a real microphone adds the capture buffer on top.

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

### Changing the language pair

A new pair used to cost a restart: the session killed the engine process, started another one and loaded the recogniser again before it could hear the first word. `Session::switch` now asks the running engine for the pair, and rebuilds only when the engine or the recogniser changes.

Windows 11, 4 threads, the OPUS-MT packages read through CTranslate2 int8, warm Python processes, one 7.5 s Portuguese clip synthesised with `pt_PT-tugão-medium`:

| What | Value |
|---|---|
| `pair` round trip, sidecar already running | **0.50 ms**, and **0.38 ms** back |
| Loading a package that was not in memory | **454.8 ms** (`pt` to `en`), **391.2 ms** (`pt` to `es`) |
| First sentence after that load, then warm | 93.5 ms, then 70.8 ms |

Reading: the swap costs half a millisecond because it only changes the pair the engine is told to use, and the recogniser is not touched at either step. What a person waits for is the new package coming off disk, about 0.4 s, and now only the first time that language is used, because the dictation path keeps the language it just left.

The switch was measured end to end on 4 November 2026 at two threads, with one write before it, two after it, and one after switching back. Windows 11 Home, AMD Ryzen 5 3600, 6 cores and 12 threads, 16 GB of RAM, no GPU, Python 3.12.10, CTranslate2 4.8.4, ONNX Runtime 1.31.1, Parakeet TDT 0.6B v3 int8, the Portuguese to English package. The clip is 7.5 s of Portuguese synthesised with `pt_PT-tugão-medium`, 16 kHz mono PCM16. It is not in the repository: `parlo record out/clip.wav --seconds 8` makes an equivalent one from a microphone, which holds the same path without reproducing these numbers to the millisecond.

| What | Value |
|---|---|
| `hello`, sidecar up | 242.5 ms |
| Memory at `hello`, then with Parakeet loaded | 28.2 MB, then 838.0 MB |
| `dictate`, three calls | 4591.4, 1212.5, 1123.6 ms |
| `write` to English, first, then three warm | 2212.3 ms, then 1709.2, 1601.1, 1689.8 ms, median **1689.8 ms** |
| `pair` to Spanish, then back to English | 0.8 ms, then 1.2 ms |
| `write` after the switch to Spanish, first and second | 1963.1 ms, 1542.3 ms |
| `write` after switching back to English | 1636.6 ms |

Writing a sentence in the other language costs 1.7 s median on this clip, and none of that is the pair change. The switch is under a millisecond because it only tells the engine which pair to use; the Spanish package arrives on the first write that needs it, 1963.1 ms against 1542.3 ms on the next call, and once warm the sentence lands within 150 ms of the English path. Dictation pays 4591.4 ms once, for the recogniser, and about 1.2 s a sentence after that.

Limits. One 7.5 s synthetic clip, one machine, three warm calls, two threads, no music and no crosstalk. The transcript is not a quality result: the recogniser misheard words and those mistakes travel into the translation. Speech is not measured here: `translate` stopped with `the piper adapter stopped without answering`, which is the voice path, not the write path. Four threads could not run this on the day: with games and a browser holding most of 16 GB, the same command stopped with `bad allocation` inside ONNX Runtime, and a later attempt closed the sidecar with `Expecting value: line 1 column 1`. The earlier engine comparison put `write` at 992.7 ms median on a 5.085 s clip with four threads, so that row and this one are not comparable: different clip length and thread count.

```bash
.venv/Scripts/python.exe tools/benchmark_engines.py --wav .tmp-clip-pt.wav --models .tmp-models --cascade-dir engines/cascade --from pt --to en --switch-to es --threads 2 --runs 3 --json .tmp-bench-write-switch.json
```

`--switch-to` adds the pair change and the writes around it to any run, and `--warmup` keeps the sidecar's own warm-up so a restart can be measured the way the app used to pay for it.

### Speaking twice in one process, 4 November 2026

The voice answered once and then failed with `the piper adapter stopped without answering`; every later sentence in the same session repeated it, and the sidecar itself closed after the first failure. Four faults made that happen and each one is fixed:

- the client kept the process object of a dead adapter, so a crash was never followed by a restart, and `stderr` was sent to `NUL`, so the reason was thrown away;
- the adapter wrote its audio through a `wave` file whose format piper sets on the first chunk, and text that phonemises to nothing yields no chunk at all: closing the file raised `# channels not specified`, which is the sentence users saw;
- `engines/cascade/serve.py` let engine errors reach the process boundary, because `parlo_cascade.engine.EngineUnavailable` is not the class `parlo_hibiki.protocol` catches, so one failed sentence ended the sidecar and every later request met a closed pipe.

The proof that the voice now repeats is two `speak` calls in one sidecar, `PARLO_CASCADE_WARMUP=0`, two threads, the `pt_PT-tugão-medium` voice:

| Call | Audio returned |
|---|---|
| 1, `Bom dia, este e um teste.` | **2.17 s** |
| 2, `Boa tarde, outra frase.` | **1.93 s** |

The sidecar exited with 0 after both. Before the fix the first of those calls already failed, and `engines/cascade/tests/test_tts.py` now covers it: an adapter that answers once and exits must still answer the next sentence.

The same proof through the product's own binary, with no hand-started sidecar. `parlo serve` on 127.0.0.1:8733 with `PARLO_APP_GUARD=1` and the same two threads answered two `POST /v1/audio/speech` requests, both with HTTP 200 and 16 kHz mono WAV:

| Request | Text | Audio returned | Round trip |
|---|---|---|---|
| 1 | `Bom dia, este e um teste da voz do Parlo.` | **3.37 s** | 414 ms |
| 2 | `Boa tarde, outra frase para medir a voz.` | **3.09 s** | 242 ms |

Both requests ran inside one `parlo` process, against one cascade sidecar (pid 2776) and one piper adapter (pid 20432), and that process exited with 0 when its stdin closed. `parlo dub` covers the batch path the same way: a 15.9 s Portuguese clip (the 7.5 s clip played twice with 0.9 s of silence between), `--from pt --to en`, exited with 0, printed `[1/2]` and `[2/2]`, and wrote 12.61 s of English audio over two subtitle lines.

```bash
PARLO_APP_GUARD=1 PARLO_MODELS_DIR=.tmp-models target/debug/parlo.exe serve --port 8733 --token probe-token
curl -s -X POST http://127.0.0.1:8733/v1/audio/speech -H "Authorization: Bearer probe-token" -H "Content-Type: application/json" -d '{"input": "Bom dia, este e um teste da voz do Parlo.", "response_format": "wav"}' -o .tmp-speech-1.wav
curl -s -X POST http://127.0.0.1:8733/v1/audio/speech -H "Authorization: Bearer probe-token" -H "Content-Type: application/json" -d '{"input": "Boa tarde, outra frase para medir a voz.", "response_format": "wav"}' -o .tmp-speech-2.wav
target/debug/parlo.exe dub .tmp-clip-pt-long.wav --from pt --to en --out .tmp-dub-out.wav --srt .tmp-dub-out.srt
```

The regression test was checked in both directions on the reference commit. With `engines/cascade/parlo_cascade/tts.py` taken from the commit before the fix, `test_a_dead_adapter_is_replaced_for_the_next_sentence` fails with `parlo_cascade.tts.VoiceUnavailable: the piper adapter stopped without answering` and `FAILED (errors=1)`; with the fixed file back it prints `OK`, and the file is unchanged in the working tree afterwards.

### Dubbing three segments with one voice, 4 November 2026

`parlo dub` runs recognition, translation and voice one segment at a time, and the only part worth hiding is the voice's model load: piper spends 5 s loading `en_US-lessac-medium` before it can say anything, while the recogniser is already busy with the first segment. The load now starts in a thread when the batch begins, and one lock in `parlo_cascade/tts.py` keeps two threads from starting two adapters.

Clip: the 7.5 s Portuguese clip played three times with 0.9 s of silence between, 24.3 s in total, two threads.

| Batch | Wall | Adapter seen from | Segments | WAV | Exit |
|---|---|---|---|---|---|
| Before | 16.60 s | 5.78 s | 3 | 19.83 s | 0 |
| After | **11.75 s** | **0.78 s** | 3 | 20.00 s | 0 |

Both runs produced the same three translated lines, which is the only part of the output that can be compared: piper is not deterministic. One English sentence synthesised six times, three in each of two processes, came back as 7.6742 s, 8.0922 s, 7.8716 s, 8.2547 s, 7.8367 s and 7.7787 s, so the WAV moves by half a second between runs of the same text and the subtitle timings move with it.

One adapter means one `engines/tts-piper/serve.py` session for all three segments. On Windows that is two process ids: `.venv\Scripts\python.exe` is a redirector that starts the base interpreter. Both ids stayed alive for the whole batch in both runs and no second session appeared.

```bash
target/debug/parlo.exe dub .tmp-clip-pt-three.wav --from pt --to en --out .tmp-dub.wav --srt .tmp-dub.srt
```

The `translate` number is still not published, and this time memory is the whole reason. Windows 11 Home, AMD Ryzen 5 3600, 16 GB, games and a browser holding most of it, about 2.3 GB free: the recogniser refused to load (`Load model from .tmp-models\parakeet\encoder.int8.onnx failed: bad allocation`), CTranslate2 stopped with `mkl_malloc: failed to allocate memory`, and once inside the voice `onnxruntime` asked for 8 978 432 bytes and did not get them (`BFCArena::AllocateRawInternal Failed to allocate memory`). The same run with Whisper base instead of Parakeet held only 201 MB and still failed in `mkl_malloc`. Nothing here is a property of the fix: with the fix the failure arrives as one error reply and the sidecar stays alive, which is what the protocol tests assert.

When a machine has the memory, these two commands produce the numbers, the second of them over the voice path alone:

```bash
.venv/Scripts/python.exe tools/benchmark_engines.py --wav .tmp-clip-pt.wav --models .tmp-models --cascade-dir engines/cascade --from pt --to en --threads 2 --runs 3 --json .tmp-bench-translate.json
.venv/Scripts/python.exe tools/benchmark_engines.py --wav .tmp-clip-pt.wav --models .tmp-models --cascade-dir engines/cascade --from pt --to en --threads 2 --runs 3 --speak "Bom dia, este e um teste da voz do Parlo." --json .tmp-bench-speak.json
```

`--speak` is new: it says one sentence through the engine and reports the first call and the median of the warm ones, so the voice path can be measured without the recogniser.

### The translate memory wall, 4 November 2026

`parlo translate` reads the whole file, pushes it through the pipeline in 480 sample frames and writes back the audio it receives. At the start of the week it died here with `bad allocation` while 2.3 GB of the machine's 16 GB were free. With 6.7 GB free the same command finishes, and the memory it needs does not depend on the length of the file:

| Clip | Wall | Peak RSS | Exit |
|---|---|---|---|
| 24.30 s, three utterances | 32.37 s | 1128 MB | 0 |
| 49.50 s, six utterances | 56.57 s | 1133 MB | 0 |

The six utterance run holds that peak for 56 s without growing, so nothing is retained per segment. The memory is three models resident at once, and they stay until the process ends:

| Stage | RSS in the sidecar |
|---|---|
| interpreter and imports | 18 MB |
| engine object | 25 MB |
| parakeet int8 loaded | 720 MB |
| parakeet unloaded | 51 MB |
| translation model loaded | 154 MB |
| voice adapter started | 151 MB |
| voice and translator released | 61 MB |

The recogniser costs 695 MB and the translator 130 MB in the same process, and the piper adapter adds 220 MB in its own: 881 MB in the sidecar, 220 MB in the adapter, 18 MB of Rust, which is the 1128 MB measured.

They are resident together because the pipeline is a stream. Every segment is transcribed, translated and spoken before the next one starts, so the recogniser cannot be dropped while the translator and the voice are in use. Two passes over the file would fix it, transcribe everything, drop the recogniser, then translate and speak, and the peak would be about 750 MB instead of 1128 MB: 720 MB for the recogniser alone, then 350 MB for translator and voice.

That needs the offline path to hand the whole file to the engine, and that path is `translate` in `crates/parlo-cli/src/main.rs`, which pushes 480 sample frames one at a time; it now asks the engine for a flush when the loop ends, but the peak stays where it is because the work is the same and the three models still overlap in time. The smaller-looking alternative, unloading the recogniser between segments and reloading it for the next one, costs a reload every 2.5 s of audio and would make the live path unusable, so it was measured out instead of forced in.

The peak is not a threading artefact either: loading parakeet with 1, 2 and 4 threads settles at 713, 714 and 715 MB, so the threads do not buy memory back.

### The tail the pipeline used to drop, 4 November 2026

A 24.30 s clip is nine 2.5 s segments plus 1.80 s. The engine only translates once its buffer reaches `min_segment`, so the last 1.80 s waited in the buffer and died with the process. Feeding every frame of the clip to `CascadeEngine` in one process gives nine segments; calling the new `flush()` afterwards gives ten, the tenth being `Fatil libre works well.`

The `/translate` route now flushes at end of stream, and two runs of the clip on it took 6.09 s and 6.39 s wall with 1129 MB peak, producing 22.67 s and 24.60 s of audio from 24.30 s of input; the output length moves with the voice, which is not deterministic. The peak does not move: the flush drains an 80 KB buffer, while the 1128 MB is the recogniser, translator and voice adapter staying resident, as measured above.

`parlo translate` now drains that buffer too. The same 24.30 s clip takes 14.98 s and 16.40 s wall with 1124 MB and 1125 MB peak and writes 22.60 s and 22.96 s of audio, against the 32.37 s and 1128 MB recorded above; the three megabytes are run noise, not a saving, since the flush moves an 80 KB buffer.

The clearest proof is the tail alone. Its last 2.00 s is shorter than a segment, so before the change the command had nothing to write and failed, and now it exits 0, produces 1.82 s of audio in 9.21 s, and peaks at 1027 MB.

### The two passes that halve the peak, 5 November 2026

The sidecar warmed up recognition, translation and voice before the first request, so a whole-file translation paid for all three at once. `parlo translate` now starts its engine without warmup — `PARLO_CASCADE_WARMUP=0`, set by the sidecar spawn for that command only — and sends the file in one `translate_file` operation. The engine transcribes every 2.5 s segment first, calls `unload_unit("recognition")`, and only then loads the translator and starts the voice for the segments it already has.

| Clip | Passes | Wall | Peak RSS |
| --- | --- | --- | --- |
| 24.30 s, three utterances | one | 14.98–16.40 s | 1124–1125 MB |
| 24.30 s, three utterances | two | 15.55–20.76 s | 802–803 MB |

Two runs of the two-pass path on the same clip wrote 22.83 s and 22.41 s of audio and peaked at 802 MB and 803 MB. The peak is now the first pass alone: the recogniser holds about 720 MB inside the sidecar, and the translator (130 MB) and the voice adapter (220 MB, in its own process) only appear after it has been dropped. Ten segments come out of the clip, the last being `The one for Gafatil Libra works well.`

The live path is unchanged: the same clip posted to `/translate` still returns 22.77 s of audio in 6.24 s, with the tail in the transcript.

The stage split of that run is 6.24 s of recognition, 6.91 s of voice and 1.49 s of translation on a warm machine, both model loads included. The one cost that was ours to remove is the sample conversion: reading the clip's 388,803 samples from pcm16 was 185.1 ms in a Python loop and is now 0.94–1.04 ms through numpy, which also shortens every live segment by about 19 ms.

### The recogniser floor, 5 November 2026

The parakeet weights are `encoder.int8.onnx` at 622.0 MB, `decoder.int8.onnx` at 11.3 MB and `joiner.int8.onnx` at 6.1 MB, 639.4 MB together. Every lever below was measured on the same 24.30 s clip, in a fresh process, with the recogniser loaded on its own.

| Lever | Loaded | Peak RSS | Recognise |
| --- | --- | --- | --- |
| baseline, two threads | 720 MB | 762 MB | 7.08 s |
| intra-op threads 1 | 720 MB | 763 MB | 8.79 s |
| intra-op threads 2 | 720 MB | 763 MB | 6.98 s |
| intra-op threads 4 | 721 MB | 765 MB | 6.25 s |
| encoder session, ORT arena on | 709 MB | — | load 5.80 s |
| encoder session, ORT arena off | 710 MB | — | load 6.19 s |
| encoder re-quantized with `quantize_dynamic` | 732 MB | 765 MB | 5.77 s |

A second decode pass over the same segments leaves the process at 763 MB, so nothing accumulates between segments. Reusing one sherpa stream for every segment is not an option: it keeps the accepted audio, so the text changes and the run takes 15.75 s instead of 3.16 s.

`OfflineRecognizer.from_transducer` exposes `num_threads`, `provider` and `debug`, and no arena or memory-pattern switch; the two arena rows come from `onnxruntime` directly on the encoder and show what the switch is worth, which is one megabyte.

The re-quantization was run rather than assumed. `onnxruntime.quantization.quantize_dynamic` over `encoder.int8.onnx` finished in 9.3 s and wrote 622.1 MB against the 622.0 MB source, because that source already stores 588 of its tensors as uint8 and 612 as float32; the tool warned it could not touch the int4 tensors it found and left the rest alone. Loaded by sherpa in a fresh process it takes 732 MB, peaks at 765 MB and recognises the clip in 5.77 s, against 731 MB, 764 MB and 5.94 s for the shipped file, and the ten lines it returns are identical line by line to the stored baseline. A variant that changes neither the bytes nor the text is not worth a second file on disk, so the shipped encoder stays the default; the recipe is the one line above if a future model needs it.

The floor is the 709 MB of the encoder session, which is its weights plus the runtime around them; the 762 MB peak adds the decoder, the joiner and one segment's buffers. Twenty percent below 762 MB is 610 MB, less than the weights alone, so that bar is out of reach without a smaller recogniser, and a smaller recogniser changes the transcript. Both passes on the clip return the same ten lines, the last one `The one for Gafatil Libra works well.`

The translations below are the engine's own on the 24.3 s clip, chunked the way the command chunks it. This clip is synthetic and the recogniser mishears words on it, which the translations carry:

```
line 1: This is your copyright symbol.
line 2: And he's got to talk about Tilalphuminian Portugal, that's what it is.
line 3: til for free graphil or works well.
```

```bash
target/debug/parlo.exe translate .tmp-clip-pt-three.wav .tmp-translate.wav --engine cascade --from pt --to en
```

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

### The 100 sentence gate

`benchmarks/corpus/translation.jsonl` holds 100 original sentences, 50 in each direction, written for this project with numbers, negations, names, dates and times throughout. `engines/cascade/scripts/mt_gate.py` runs a package over the corpus and fails below a threshold:

```bash
python engines/cascade/scripts/mt_gate.py --pack --direction pt-en
python engines/cascade/scripts/mt_gate.py --pack --direction en-pt
```

Windows 11 laptop, no GPU, 4 threads, beam 4, warm process, the installed `1.9` packages:

| Direction | Numbers | Clock | Negation | Names | Complete | Gate |
|---|---|---|---|---|---|---|
| PT→EN | 1.000 | 1.000 | 1.000 | 0.980 | 1.000 | passes |
| EN→PT | 1.000 | 0.980 | 1.000 | 0.980 | 1.000 | passes |

Numbers, negations and completeness are clean in both directions. Two things are not:

- **PT→EN used to lose the half of the day.** `15h30` → `3:30`, `18h` → `6:00`, `17h` → `5:00` (pt-en-001, 016, 043). The hour was right and the reader could not recover the rest. The backend now restores the marker when the source states a 24 hour afternoon time and the translation carries the same clock without one; that is a rule in `parlo_cascade/mt.py` with six tests behind it, not a change in the model. The table above is measured after it.
- **Names get translated.** `O João` → `John` and `a Maria` → `Mary` in pt-en-003, the only row that still loses a name. An earlier run counted four, and three of them (017, 019, 044) were a defect in the gate, not in the model: it ignored the first word of every sentence, so an output that opened with the name was scored as if the name had vanished. The gate now reads the names found in the source as context, and `engines/cascade/tests/test_mt_gate.py` covers the case.

The check reads value, not typography: `four hours`, `3.000 euros` and `7.05` → `7h05` all count as preserved, because the same quantity arrives. `8.30` → `20h30` (en-pt-035) is a real error and is counted as one. Thresholds were set before the run and left alone: numbers 0.95, clock 0.95, negation 0.90, names 0.80, complete 1.00.

**On the Argos copy the gate passes on both sides; on the copy Parlo installs it fails, as the next subsection records.** The pair is still not declared validated either way. Five automatic properties are not a reading: one PT→EN row still translates a name (`O João` → `John`), and an idiom can pass every count while being wrong. Declaring the pair approved is the competent-speaker step the plan asks for, and it has not been done.

### The same corpus on the copy Parlo installs

`parlo_cascade.mt.find_pack` used to search the Argos application's package folders before the folder Parlo installs into, so the two tables above described the Argos copy. `pack_roots` now reads Parlo's own folder first, and the pair was measured again on the copy `packs.install` produces, downloaded from the index it uses, `translate-pt_en-1_9`, Windows 11, 4 threads, beam 4:

| Direction | Numbers | Clock | Negation | Names | Complete | Gate |
|---|---|---|---|---|---|---|
| PT→EN, Parlo's copy | 1.000 | 1.000 | 1.000 | 0.980 | 1.000 | passes |
| EN→PT, Parlo's copy | 1.000 | 0.980 | 1.000 | 0.980 | 1.000 | passes |

Neither direction loses a number on this copy. PT→EN loses a name in pt-en-003 (`O João` → `John`) and EN→PT in en-pt-017; EN→PT loses the clock on en-pt-035 (`8.30` → `20h30`), which the 0.95 threshold tolerates and which stays a real error.

The first reading of this same copy scored numbers 0.840 and negation 0.880, and the cause was not the model: `MarianTranslator` joined a hypothesis's pieces with spaces before decoding them, which turns `2026` into `20 26` and `1.250,50` into `1.2 50 , 50`. Decoding the piece list instead of a rebuilt string restores every quantity in both directions. The provenance finding stands — the gate was reading the Argos copy and now reads Parlo's — but the two folders carry the same build for `en-pt`, and the product measures the same on its own folder once the decoder stops inserting spaces.

**These are the numbers to quote for the product.** The pair is still not declared validated, because five automatic properties are not a reading and both directions still lose one name.

### Portuguese to German, through English

The plan makes PT→DE the central gate of v0.5, and the index Parlo installs from carries no `pt` to `de` package at all (100 entries, zero for that pair). What can be measured today is the pivot the plan allows but requires to be declared: PT→EN, then EN→DE, which `--pivot` prints as part of the route.

20 Portuguese business e-mail sentences, `translate-pt_en-1_9` then `translate-en_de-1_3`, Windows 11, 4 threads, beam 4:

| Numbers | Clock | Negation | Names | Complete | Gate |
|---|---|---|---|---|---|
| 1.000 | 1.000 | 1.000 | 0.950 | 1.000 | passes |

That table is not a verdict and the pair is **not** declared working:

- The German references were written by the project and have not been read by a native speaker, so the corpus is unverified material checked by unverified material.
- The output is visibly wrong where the counts cannot see it: pt-de-016 turns *esteve em baixo durante 2 horas* into `Der Server ist seit zwei Stunden ausgefallen` (since, not for), and pt-de-018 turns *o escritório do Porto tem 5* into `Das Büro in Porto ist 5`, which is not German.
- pt-de-003 loses both names, the defect PT→EN shows on its own row.

```bash
python engines/cascade/scripts/mt_gate.py --pack --direction pt-de --pivot
```

A direct model was attempted and is not usable yet. Helsinki-NLP publishes `es-de`, `fr-de` and `it-de` but no `pt-de`, so the candidate was `facebook/m2m100_418M` (MIT), which the plan itself names. It converts to CTranslate2 int8 in 471 MB and answers, and every sentence degenerates into repeated tokens: `Ich habe das Tre Tre Tre Tre …`, `12 12 12 Tage bis bis bis zum Term Term Term`. The behaviour is the same with beam 4 and greedy, with the repetition penalty and the no-repeat n-gram rule the Marian path uses set to off, and with or without the source language token. A `float32` conversion of the same checkpoint, 1.9 GB, degenerates in the same way (`beam 1: Ich habe die Konfer Konfer Konfer …`), so the quantisation is not the cause. What is left to test is the converter itself, and it was tested: with `transformers` 4.44.2, `tokenizers` 0.19.1 and `huggingface-hub` 0.24.6 shadowing the environment, `TransformersConverter` stops with `M2M100ForConditionalGeneration.__init__() got an unexpected keyword argument 'dtype'`, because the converter passes an argument only the 5.x convention accepts; under `transformers` 5.22.1 the same call completes and produces the repeating weights. `ctranslate2` 4.8.4 therefore converts M2M100 into a model that cannot translate, and neither installed `transformers` converts it into one that can. Until the converter agrees with a released `transformers`, M2M100 stays out and PT→DE keeps using the pivot.

A second direct attempt was made on 3 November with `Helsinki-NLP/opus-mt-tc-bible-big-mul-deu_eng_nld` (Apache-2.0), and this one separates the model from the tooling.

**The checkpoint is good.** In float32 `transformers`, with the target tag `>>deu<<` (id 692 in a 58,434 piece vocabulary) prepended to the source ids, three of three business sentences come back as correct German: `die Lieferfrist ist der dreiundzwanzigste Tag des März`, `Wir brauchen die Rechnung bis Freitag.` and `Das Büro in Porto hat fünf Personen.`

**The conversion is not.** Three CTranslate2 int8 builds of it produce the same 242,000,347 byte `model.bin` and the same broken text, `die Liefer Liefer Liefer …` and `Rechnung Rechnung …`, at beam 4 on four threads and about 1.3 s per sentence. The builds were: the converter in the installed environment, a second run of the same tool, and a run against `transformers` 4.46.3 with the converter's `dtype=` kwarg translated to `torch_dtype=`, which is the only argument that version rejects. Reconverting changes nothing, so the defect is in how this toolchain exports the Marian weights, not in the weights.

Two details correct an earlier reading of this model. The checkpoint's `source.spm` and `target.spm` hold 32,000 pieces and contain none of the language tags, so the tag has to be passed to the translator as a token of the shared vocabulary (`>>deu<<`, `>>eng<<`, `>>nld<<`) rather than folded into the text and tokenised; doing that is what turns `delivery delivery …` into `die Liefer Liefer …`, which is why the model cannot be dismissed as untranslatable. And the first build was measured with the wrong tokenizer, so its output said nothing about the model at all.

The pair stays out, `mt_gate.py` keeps refusing converted checkpoints, and PT→DE still runs through the pivot. What would change that is a conversion path that survives its own probe — ONNX export or a fixed converter — because the model itself already answers.

A second checkpoint shows the same split. `Helsinki-NLP/opus-mt-tc-bible-big-roa-deu_eng_fra_por_spa` (Apache-2.0, Romance sources including Portuguese, 45,245 pieces, `>>deu<<` at id 710) answers `die Lieferfrist ist der 23. März`, `Wir brauchen die Rechnung bis Freitag.` and `Das Büro des Hafens hat fünf Personen` in float32 `transformers`, and its CTranslate2 int8 build repeats words instead (`die Liefer Liefer Liefer …`, `Freitag Freitag Freitag`) at 0.45 s per sentence. Two checkpoints, five builds, one pattern: the conversion is what fails, and neither model is judged by it.

ONNX does work, with a pinned toolchain. In its own environment — `torch` 2.4.1+cpu, `optimum[onnxruntime]` 1.24.0, `transformers` 4.46.3 — `main_export` writes `encoder_model.onnx` (491,946,979 bytes) and `decoder_model.onnx` (778,438,780 bytes) in 73 s, with no `decoder_with_past`, so inference runs with `use_cache=False`. Dynamic int8 quantisation brings the pair to 123,405,098 and 195,511,116 bytes and keeps the meaning of all four probes, at the cost of casing and some wording: `Liefer die frist ist am 23. märz`, `Wir Wir brauchen die Rechnung bis Freitag`, `büro von port hat fünf personen`, and one clean sentence, `Der Vertrag wurde von der Finanzdirektorin unterzeichnet.` The float32 ONNX build matches `transformers` on three of three.

Latency is the catch. The int8 build turns a sentence around in 1.42 s on four threads, against about 0.79 s for the pivot's two stages and 0.45 s for the CTranslate2 build of the same checkpoint, which is fast but repeats words. A direct pair is therefore reachable — the first one this project can run end to end from Portuguese — but it is not yet better than the pivot. The next steps are a decoder exported with past key values and a quantisation that keeps casing, since the current one is slower and rougher than the route it would replace.

With the decoder exported **with past key values** (`text2text-generation-with-past`), the same int8 build runs the gate's 20 Portuguese to German sentences directly. Windows 11, four threads, beam 4, warm process, `Helsinki-NLP/opus-mt-tc-bible-big-roa-deu_eng_fra_por_spa` as ONNX int8:

| Numbers | Clock | Negation | Names | Complete | Gate |
|---|---|---|---|---|---|
| 1.000 | 1.000 | 1.000 | 0.850 | 1.000 | passes |

Median **601.6 ms**, p95 **750.3 ms** per sentence, against about 0.79 s for the two-stage pivot. Three rows the pivot got wrong come back right: `Das Team hat 8 Personen und das Porto-Büro hat 5.` where the pivot produced `Das Büro in Porto ist 5`, `Die Zahlung erfolgte am 15. Oktober, nicht am 20.` and `Wir haben nie solche Beschwerden erhalten.` The one name miss is the same `John`/`Mary` row the PT→EN gate loses. The counts come from the gate's own metric functions, but this was a probe, not `mt_gate.py --pack`: the gate still refuses converted checkpoints, and the engine speaks CTranslate2, so the pair is measured and not yet installable. What remains is an ONNX path in the engine, or a conversion that stops repeating words.

The run is repeatable from the repository:

```bash
python engines/cascade/scripts/mt_gate.py --direction pt-de --onnx <folder> --tag '>>deu<<'
```

It needs `torch`, `optimum` and `transformers` in a separate environment; without them the gate refuses with one sentence instead of a traceback.

**And the finding that matters more than the German pair.** `mt.find_pack` searched the Argos application's own package folders before the folder Parlo installs into, so the two tables above described the Argos copy. `pack_roots` reads Parlo's folder first now. The stale folder at `engines/cascade/models/mt/pt-en` still decodes the first corpus sentence into `I set the meeting for 3 .30 pm p . m .30 on Thursday Thursday . .` and keeps going; its layout (`ct2/` and `hf/`) is not what `packs.install` writes, and `find_pack` ignores it by name. What it exposed was the decoder, not the model: joining the hypothesis pieces with spaces split `2026` into `20 26`. With that fixed, the folders the product installs pass the gate in both directions.

The mechanical gate scored that output at numbers 0.950, clocks 0.950, negations 0.950, names 0.950, completeness 1.000, which is the clearest demonstration of what the gate is worth: it counts quantities and punctuation, and cannot hear that the sentence is broken. Nothing from that run is published as a result, and PT→DE still runs only through English.

### The Rust engine against the Python engine

The Rust engine already answers dictation. The useful question is what a replacement would cost and buy. One 8.05 s clip went to all three engines through the same JSON line protocol, with Parakeet TDT 0.6B v3 int8 on four threads and three runs each. The clip is Microsoft Zira reading an English sentence, so the comparison is between engines. The text came back word for word identical.

The Python source ran under Python 3.12.10, the bundled engine is the 0.2.4 PyInstaller folder, and the Rust engine is `parlo-engine` 0.2.1+3 with its three DLLs, a prototype outside the workspace.

| | Python, source | Python, bundled | Rust |
|---|---|---|---|
| `hello`, warm | 229 ms | 403 to 439 ms | **33 ms** |
| RSS at `hello` | 28.1 MB | 37.5 MB | **8.9 MB** |
| RSS with Parakeet loaded | 844.8 MB | 854.0 MB | **809.9 MB** |
| `dictate`, warm median | **1090.4 ms** | 1265.3 ms | 1152.1 ms |
| First `dictate`, load included | 5680.5 ms | 6166.5 ms | 6178.9 ms |
| Installed size | 182 MB, 477 files | 182 MB, 477 files | **23 MB, 4 files** |

The first launch of the freshly built bundle took 15.5 s. The next three took 0.40 to 0.44 s, so that first figure is disk and antivirus work on a new executable, not the startup a user meets on later launches. A Python start that warms every model before answering `hello` took 12.08 s and held 993.4 MB.

The warm segment stays between 1.09 and 1.27 s in all three, which is the recogniser's cost. Rust moves startup from 0.40 to 0.44 s down to 33 ms, saves 35 to 45 MB with Parakeet loaded and cuts the bundle from 182 MB to 23 MB, but it does not make a segment faster. The 23 MB bundle carries the recogniser and the dictation path, not translation or speech synthesis, so it is not the size of a complete engine.

```bash
python tools/benchmark_engines.py --wav out/voice.wav --models "$LOCALAPPDATA/parlo/models" --cascade-dir engines/cascade --bundled build/installer/payload/parlo-engine/parlo-engine.exe --rust crates/parlo-engine-rs/target/release/parlo-engine.exe --from en --to pt
```

The Rust executable needs `sherpa-onnx-c-api.dll`, `sherpa-onnx-cxx-api.dll` and `onnxruntime.dll` beside it, as the build notes in [ENGINES.md](ENGINES.md) describe.

### Voice and translation on one Portuguese clip

A 5.085 s Portuguese clip, synthesised with the `pt_PT-tugão-medium` Piper voice, 16 kHz mono PCM16, went to both engines with the Parakeet TDT 0.6B v3 int8 recogniser, the OPUS-MT PT→EN package and the same Piper voice on four threads. The harness ran three calls after the first one, which loads the models; the table reports the median of those warm calls and the first call separately.

| | Python source | Rust |
|---|---:|---:|
| `hello` | 227.3 ms | **34.5 ms** |
| RSS at `hello` | 28.1 MB | **9.2 MB** |
| RSS with recognition loaded | 806.1 MB | **771.2 MB** |
| `dictate`, median | 704.8 ms | **645.5 ms** |
| `write`, first / median | 1478.1 / **992.7** ms | 4096.1 / 3248.1 ms |
| `translate`, first / median | 7866.5 / **1469.4** ms | 7268.5 / 3849.2 ms |
| Audio returned | 6.409 s at 16 kHz | 5.608 s at 22.05 kHz |

The Rust engine still starts faster and uses less memory with the recogniser loaded. It does not win translation or speech: `write` is about 3.3 times slower and `translate` about 2.6 times slower at the median. Both transcripts misheard “Rust” as “rosto” (Python) and “russo” (Rust), so the translated sentences are not a quality result. That is why the Python engine remains the default and the Rust engine stays opt-in.

```bash
.venv/Scripts/python.exe tools/benchmark_engines.py --wav .tmp-clip.wav --models .tmp-models --voice .tmp-models/pt_PT-tugão-medium.onnx --espeak-data .venv/Lib/site-packages/piper/espeak-ng-data --cascade-dir engines/cascade --rust crates/parlo-engine-rs/target/x86_64-pc-windows-msvc/release/parlo-engine.exe --from pt --to en --threads 4 --runs 3 --json .tmp-bench.json
```

## Per-language recognition and translation

The reference machine (12 core CPU, no GPU, Windows 11, Python 3.12, Parakeet TDT 0.6B v3 int8 on 4 threads, Piper medium) has been measured for Portuguese. The other first-class languages do not have installed models on this machine, so their numbers are blank on purpose until someone runs the command against real audio.

| Language | Recogniser | RTF | First partial | Translation direction | Median | p95 |
|---|---|---|---|---|---|---|
| pt (PT-PT) | Parakeet v3 | 0.113 | 849 ms | pt→en | 114 ms | 131 ms |
| en | Parakeet v3 | not measured yet | not measured yet | en→pt | 111 ms | 128 ms |
| es | not installed | - | - | es→en | not installed | - |
| fr | not installed | - | - | fr→en | not installed | - |
| de | not installed | - | - | de→en | not installed | - |
| it | not installed | - | - | it→en | not installed | - |
| nl | not installed | - | - | nl→en | not installed | - |

The numbers above are not promises: they come from a synthetic Piper voice on one machine, so a real speaker will read differently. To measure your own machine:

```bash
# Recognition battery on any language (replace xx):
python engines/cascade/scripts/battery.py --stt parakeet --lang xx --out out/battery-xx.json

# Translation gate on any installed pair:
python engines/cascade/scripts/mt_gate.py --pack --direction xx-en
python engines/cascade/scripts/mt_gate.py --pack --direction en-xx

# Live stream bench:
python engines/cascade/scripts/stream_bench.py --audio-dir out/voice/xx --segment 12 --out out/stream-xx.json
```

The first-class languages are en, es, fr, de, it and nl because each has a Piper voice and a recogniser path; pt is kept because it is the reference pair the gates were built on. None of the six is declared validated until a competent speaker reads the output and the corpus comes back clean.

## Kokoro against the Portuguese voice

Kokoro 82M is the permissive synthesiser the project keeps considering, and it now has a measurement instead of an expectation. Both files read one sentence, `Hoje está um dia bonito em Lisboa e apetece passear à beira do rio.`, on the reference machine above: CPU only, Windows 11, Python 3.12, piper-tts 1.8.0 and kokoro-onnx 0.6.1 on onnxruntime 1.31.1. The recogniser is Parakeet TDT 0.6B v3 int8 on four threads, the same build the cascade uses.

| Voice | Sample rate | Audio | Warm synthesis | Real time factor | Words recovered |
|---|---|---|---|---|---|
| Piper `pt_PT-tugão-medium` | 22 050 Hz | 3.18 s | 0.22 s | 0.070 | 0.710 |
| Kokoro `pf_dora` | 24 000 Hz | 3.84 s | 1.50 s | 0.390 | 0.929 |

"Words recovered" is the difflib ratio between the lowercase word tokens of the sentence and of the recogniser's transcript of the same audio a listener would hear. It says how much of the sentence survives synthesis and recognition, it is not a naturalness score, and nobody listened to the files to rate them. The transcripts are in `benchmarks/piper-pt-pt.json` and `benchmarks/kokoro-pt-br.json`, next to the audio, so both the number and the sound can be checked again.

Kokoro is the more intelligible of the two here and it is still not the voice this product can adopt. Its Portuguese is Brazilian: the model ships `pf_dora`, `pm_alex` and `pm_santa`, and its card lists `pt-BR`. It is also 5.6 times slower on the same CPU, needs a 311 MB model and 27 MB of voices against 63 MB, and `kokoro-onnx` requires `phonemizer`, which is GPL-3.0, and `espeakng-loader`, which carries espeak-ng: the licence the optional Piper adapter exists to keep out of the package. Apache-2.0 weights do not make the deployment permissive.

The gap named in the roadmap is therefore unchanged. The European Portuguese voice the app offers is the only one Piper publishes, and the recogniser recovers 0.710 of the words from it against 0.929 from the Brazilian one. Closing that gap needs a European Portuguese voice under a permissive licence, measured with this harness before it is offered to anyone.

## Every engine on every device

`parlo bench` runs what this machine can run and says what it cannot, one row per engine and device, with the hardware, the profile and the commit beside every number.
The table below is its output, and `docs/benchmarks.json` is the same run as data; regenerate both with `parlo bench` and `parlo bench --json`, and never type a row by hand.

The cascade row is measured by `engines/cascade/scripts/benchmark.py` on a committed 2.8 s recording of a spoken Portuguese sentence, with the models folder this machine uses, so a machine without Parakeet reports that instead of a number.

<!-- bench:start -->

| Engine | Device | Result | RTF | Frames | Hardware | Commit |
|---|---|---|---|---|---|---|
| cascade | cpu | measured on this machine | 0.170 | 1 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| cascade | gpu | no gpu on this machine | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| echo | cpu | measured on this machine | 0.002 | 125 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| echo | gpu | no gpu on this machine | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| hibiki | cpu | not in this build | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| hibiki | gpu | no gpu on this machine | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| cloud | cpu | not in this build | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |
| cloud | gpu | no gpu on this machine | - | 0 | 12 cores, 15 GB of memory, no gpu | d187a7c |

<!-- bench:end -->

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

## Dictation cleanup, rules only

The deterministic layer in `corrections.py` runs before any model and in every mode, so it has to be invisible in the latency budget. Ten sentences in European Portuguese and English, 8 to 18 words each, every one carrying at least one self-correction, hesitation or repetition:

| Metric | Value |
|---|---|
| Mean per sentence | **61.9 µs** |
| Median | 55.6 µs |
| p95 | 102.1 µs |

10 000 runs of `clean(text, "light", "pt")`, Windows 11 laptop, Python 3.12, 4 threads, no GPU, warm process. At 62 µs a sentence the layer is four orders of magnitude below the 420 ms a segment already costs, which is why it is never traded away for speed. The cloud route is not measured here and has no published latency: no request to a real provider was made.

## Reproduce the pipeline benchmark

The echo engine needs no model and no microphone:

```bash
cargo run --release -p parlo-cli -- demo out/demo.wav
```

On the reference machine used for development this processes 125 frames of 20 ms audio in well under a second, which confirms the pipeline itself adds no meaningful delay. The number that matters is the engine's, and that is what the table above will record.

## Submitting a result

Open a hardware report issue with your `parlo doctor` output, the command you ran, and the raw timings. Results are added here with the contributor's hardware named, because a benchmark without its hardware is a rumour.

## Installed voice check

The European Portuguese Piper voice on this machine, asked for as `voice-pt`, with the recogniser reading the same file back, on 2026-11-06:

| What | Result |
|---|---|
| Audio | 16 kHz mono 16-bit wav, **3.67 s**, **58700** frames, kept in [d10-speech.wav](benchmarks/d10-speech.wav) |
| Level | RMS **7171**, peak **32732**, last 20 ms RMS **1389** |
| Spoken | `Hoje está um dia bonito em Lisboa e apetece passear à beira do rio.` by the Portuguese voice, untouched by the dictionary |
| Read back | `Ojista tem um dia vomitÃª no Lisboa e apetece passear a tilbeira do rio.` |

The audio is a plain Portuguese sentence, so anyone can check it by ear: it starts with a RIFF/WAVE header, carries 3.67 s of speech at 16 kHz and decays to a fifth of its level over the last 20 ms. An earlier recording of a name-heavy sentence sounded weak because `/v1/audio/speech` ignored `model` and `voice` and read Portuguese text with the session's English voice; the route now builds the engine for the voice the request names. The recogniser is English only and recovers most of the words, which is enough to show the audio carries speech and not noise; the facts are stored in [d10-speech.json](benchmarks/d10-speech.json) and checked by `cargo test -p parlo-cli --test d10`.

## Server soak

`parlo soak --seconds N --interval S` starts `parlo serve`, samples the working set of that process every interval, stops it and reads `sessions.jsonl` back, so a long run leaves a number instead of a claim. The sample interval is 15 seconds by default and a run stops at 480 samples.

| What | Result |
|---|---|
| Run | 180 s asked, **184 s measured** |
| Memory | **11.8 MB** first, **11.7 MB** last, **11.8 MB** peak |
| Samples | 12, one every 15 s |
| Sessions | 1 recorded, **0 crashed**, 100.0% crash free |

Measured on 2026-11-06 on the Windows 11 reference laptop with a debug build, the echo engine and no models loaded, so the number describes the server loop rather than the recogniser. The two-hour window with the app beside the server has not been run.

## Per-engine run on this machine

`parlo bench --json` writes one row per engine and device. The run on this machine was made on 2026-11-06 and is stored verbatim in [docs/benchmarks/machine-2026-11-06.json](benchmarks/machine-2026-11-06.json), so a row can be checked against the command that produced it.

| Engine | Device | Result |
|---|---|---|
| echo | cpu | **0.0031 real time, 7.9 ms, 125 frames** |
| cascade | cpu | **0.23 real time**, 0.64 s for the 2.5 s sentence, 1 frame |
| cascade, echo, hibiki, cloud | gpu | no number: this machine has no gpu |
| hibiki, cloud | cpu | not in this build |

Conditions: Windows 11 reference laptop, 12 cores, 15 GB of memory, no gpu, build `5b2b7ef`, debug profile, Parakeet 0.6B v3 and the European Portuguese Piper voice on disk. The echo row measures the frame path without a recogniser, so it is not a dictation speed; the cascade row is one 2.5 s sentence through the installed recogniser.
