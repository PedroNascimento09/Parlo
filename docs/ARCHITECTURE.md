# Architecture

Parlo moves audio in fixed, bounded steps. Every number in this document is either enforced in code or checked by a test.

## Data path

```text
capture -> frame buffer -> voice gate -> resampler -> engine -> output buffer -> virtual mic
```

| Stage | Type | Responsibility |
|---|---|---|
| Capture | host | Reads the real microphone at 48 kHz mono. |
| Frame buffer | `Pipeline::pending` | Accumulates samples until a full frame exists. |
| Voice gate | `VoiceGate` | Drops silence so the engine does not hallucinate speech from noise. |
| Resampler | `Resampler` | Streams 48 kHz to the 16 kHz the engine expects, continuously across blocks. |
| Engine | `dyn Engine` | Returns translated audio and optional text. |
| Output | host | Writes frames to the virtual microphone. |

Frames are 20 ms by default, which is 960 samples at 48 kHz and 320 samples at 16 kHz. A smaller frame lowers latency and raises the risk of running dry; a larger frame is more forgiving but adds delay. 20 ms is the default because it stays well under the 100 ms capture budget even on a loaded machine.

## Why these bounds

The pipeline never grows without limit:

- `RingBuffer` keeps the newest samples and reports how many it evicted, so a slow consumer causes dropped audio instead of unbounded memory.
- `Pipeline::push` only emits complete frames; no partial frame ever reaches the engine.
- `Pipeline::new` rejects zero-length frames, frames above 200 ms, and non-mono audio. Failing at construction is cheaper than failing mid-call.

## Latency model

`LatencyBudget` is built when the pipeline is constructed:

```text
capture = frame_ms            (time to fill one frame)
engine  = engine.latency_ms() (declared by the adapter)
output  = 5 ms                (fixed write allowance)
```

Targets:

| Stage | Target |
|---|---|
| Capture | under 100 ms |
| Engine first audio | 1,500 to 3,000 ms |
| Output | under 100 ms |
| Total perceived | about 2 to 3 seconds |

`is_within_target()` uses 3,000 ms; `is_within_ceiling()` uses 5,000 ms, the point where a conversation stops feeling usable. The engine number is **declared by the adapter**, not measured by the budget. Measuring real latency end to end is a separate benchmark, described in [BENCHMARKS.md](BENCHMARKS.md).

## Design decisions

**Rust for the audio core.** Fixed frame sizes, no garbage collector pauses, and a binary with no runtime to install. The core crate has zero dependencies, which keeps the supply chain and the build time tiny.

**Engine as a trait plus a process boundary.** The desktop app will spawn engines as sidecars speaking JSON lines over stdio. A crashing model cannot take the app down, and an engine can be updated without releasing the app.

**Mono first.** Microphone capture is mono, and mono removes a whole class of channel-alignment bugs. System audio capture is stereo and will be downmixed at the boundary when two-way translation lands.

**Voice gate before the engine.** Silence sent to a speech model produces invented speech. Gating keeps silent frames out of the engine and lets the output side stay quiet, which also prevents the feedback loop described in [VIRTUAL_MIC.md](VIRTUAL_MIC.md).

## Known limitations

These are real and should be fixed before a 1.0 release:

- The resampler is linear interpolation. It is transparent enough for speech at 16 kHz but aliases above 8 kHz; a band-limited resampler belongs in the shipping path. Tracked in [ROADMAP.md](../ROADMAP.md).
- Frames are accumulated in a `Vec` and drained per frame. This is predictable and allocation-light for 20 ms frames, but a lock-free audio thread will replace it when capture is wired up.
- The engine latency is a declaration, not a measurement.
- Only mono PCM16 WAV is accepted by the CLI today.

## Testing strategy

Unit tests live next to the code they cover and focus on the boundaries that break in production: resampler continuity across blocks, ring buffer eviction order, voice gate hangover, WAV header validation, and pipeline frame accounting. CLI behaviour is covered by running `doctor`, `demo`, and `translate` in CI.
