# Release candidate proposal

Status: **proposal, nothing published.** No tag, no GitHub release, no announcement exists for any of this. Prepared on 1 November 2026.

## 1. Recommended version

**v0.1.8**, one step from the last published tag `v0.1.7`. The batch is a patch: it fixes what the product did wrong, finishes dictation into something a person can use daily, and adds no capability that changes the shape of the application. A minor bump would advertise a feature line the release does not contain.

## 2. What the batch contains

Everything below is on `main` since `v0.1.7`, 48 commits, and is reachable from the app or the CLI.

### Dictation, now the finished surface

- **A floating island**, shown and hidden with one shortcut, which starts listening when it appears and writes the text when it hides. It never takes the keyboard focus from the field being written in.
- **Insertion into the field it came from.** The window in front is remembered when listening starts, and the text is typed with the keyboard only if that same window is still in front; otherwise the clipboard is used and the reason is said.
- **Dictation in another language**, chosen in the Hub, translated through the same local models. The island shows both sides: the inserted translation and the sentence that was heard.
- **A local history** of the last two hundred dictations, listed in the Hub, reusable with one click, clearable, served behind the token. Nothing leaves the machine.
- **The shortcut is a setting**: `parlo config --shortcut "Ctrl+Alt+Shift+D"` or the settings row, validated, with the registration result logged and the restart named as the moment it applies.
- **Dictate Intelligence** (`PARLO_CASCADE_CLEAN` = `original`, `light`, `intelligent`; `light` by default): self-corrections, abandoned sentence starts, hesitations and accidental repetitions are removed after recognition, on rules alone — no model, no network — and the heard sentence is kept beside the cleaned one. A second half that adds content is preserved, so a clarification is never mistaken for a correction.

### Presence

- **One presence**: a tray icon with the three modes and the five destinations, whose left click shows the island, and whose menu lands on the screen it names. Every item in it exists.

### Meetings

- **A transcript can be brought in** as plain text with a speaker per line and optional `[00:12]` timestamps, kept per meeting, with retroactive speaker naming that preserves the original marker for audit and refuses a name nothing supports.

### Translation

- **The gate runs on 100 original sentences** in each direction: PT→EN numbers, clocks, negations and completeness 1.000 with names 0.980; EN→PT numbers and negations 1.000 with clocks and names 0.980.
- **A decoder bug is fixed** that split numbers (`2026` → `20 26`). The two runs taken before the fix read 0.840 and 0.880; those numbers were the bug.
- **The gate now measures the copy Parlo installs**, not the Argos application's folder.
- **PT→DE is measurable through the English pivot**, declared as such: 1.000 on numbers, clocks, negations and completeness, 0.950 on names, and visibly wrong rows the counts cannot see (`Das Büro in Porto ist 5`).
- **M2M100 is ruled out with a cause**: `ctranslate2` 4.8.4 against `transformers` 4.44.2 fails on `dtype`, and against 5.x converts weights that repeat themselves. Recorded so nobody repeats the experiment blindly.

### Measurement

- **Streaming drift is measured**: decode time ×0.988 and real time factor ×0.971 from the first third of a run to the last, so the pipeline does not accumulate delay.

## 3. Complete, partial, and the honest middle

| Capability | Implemented | Tested | Validated |
|---|---|---|---|
| Island, shortcut, insertion | Yes | Yes | Windows only, by the author |
| Dictation in another language | Yes | Yes | Windows only |
| Dictation history | Yes | Yes | Windows only |
| Dictate Intelligence engine | Yes | Yes, 22 cases PT-PT and EN | **No** — no real recordings |
| Dictate Intelligence switch in the app | **No** — `PARLO_CASCADE_CLEAN` only | — | — |
| Meetings transcript import and naming | Yes | Yes | Windows only |
| Tray presence | Yes | Partly | Windows only |
| PT↔EN translation | Yes | Gate on 100 sentences | Not by native reviewers |
| PT→DE | Yes, through English | Gate on 20 sentences | **No** — no native reader |
| Live into a call | No | — | — |
| macOS, Linux window | Compiled, CI | No | **No** |

## 4. Known bugs and limitations

- `Das Büro in Porto ist 5` survives the PT→DE gate: the counts do not see a sentence that translates into nonsense.
- One PT→EN row still translates a name (`pt-en-003`).
- Dictate Intelligence can collapse a deliberate repetition (`muito, muito bom`) and cannot catch a correction said without a marker.
- The update button replaces the binary behind the page, not the window program.
- Live translation still needs a virtual cable to reach a call application.
- Installers are archives, unsigned (section 6).

## 5. Test state

- Python engine: `163 tests, OK` (`engines/cascade`, this batch).
- Rust workspace: `cargo test --workspace` green (4 test binaries, 0 failures) on this batch.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` belong to the publication checklist below and were not run for this proposal.
- The translation gate is a script, not a test suite: it is evidence, and its numbers are in `docs/BENCHMARKS.md`.
- No regression was observed. Nothing was validated with people outside this machine.

## 6. Models, licences, installers

- **Licences are unchanged by this batch.** The embedded catalogue still serves identifier, version, function, licence and whether that licence was checked (`GET /catalog`). A licence scanner without alerts proves declared metadata is coherent; it is not legal approval.
- **Piper stays the stable voice**, used as an external adapter with GPL obligations respected; Chatterbox remains experimental and is not announced.
- **Installed models are 800 MB, downloaded on first use.** None was added or removed by this batch. The plan sources are no longer published.
- **Installers are unsigned.** There is no certificate for this project. Windows SmartScreen shows *Windows protected your PC* and requires **More info → Run anyway**; macOS Gatekeeper refuses a downloaded archive from an unidentified developer until the user allows it in System Settings → Privacy & Security → Open Anyway, and with quarantine attributes an app can need `xattr -d com.apple.quarantine`. Signing later needs an Authenticode certificate (an OV certificate from roughly a few hundred euros a year, with an EV certificate or cloud signing needed for immediate SmartScreen trust) and an Apple Developer account (99 USD a year plus notarisation). Until then, integrity is verified by comparing the archive's SHA-256 with the one published next to the release:
  ```bash
  sha256sum parlo-0.1.8-windows-x86_64.zip
  ```
  and the app and the documentation say the build is unsigned rather than hiding it.

## 7. Publication checklist

- [ ] Approval for `v0.1.8` from the author.
- [ ] `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` green on the release commit.
- [ ] Python engine tests green in the build that ships.
- [ ] Version in every manifest moved to 0.1.8 together.
- [ ] `docs/releases/v0.1.8.md` written: what it is, highlights as single-line bullets, measured numbers with conditions from `docs/BENCHMARKS.md`, install command, known limitations, licences if changed.
- [ ] Archives built for Windows, macOS and Linux, checksums published next to them, the unsigned status stated in the notes.
- [ ] Third-party notices regenerated.
- [ ] Annotated tag `v0.1.8` on `main` and a GitHub release whose title is exactly `v0.1.8`, body from the notes file.
- [ ] The app's update check points at the new tag, verified by starting the previous release and reading the banner.

## 8. Risks

- **Trust risk, medium:** a release that claims dictation intelligence the product has only tested on written cases invites a complaint the first time it removes a word somebody wanted. The release notes must say it is not validated with recordings.
- **Support risk, medium:** no external tester means the first bug report after publication is also the first test outside this machine.
- **PT→DE risk, low:** published as *not validated by a native speaker*, which is what it is; the risk is a reader taking the gate numbers for quality.
- **Process risk, low:** unsigned installers trigger warnings that read as malware to someone who did not read the notes.
