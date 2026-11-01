# Contributing

Thanks for helping. Parlo is early, so the most valuable contributions right now are bug reports, hardware reports, and documentation fixes.

## Setup

```bash
git clone https://github.com/PedroNascimento09/parlo
cd parlo
cargo build --workspace
```

Requirements: Rust 1.83 or newer. Python 3.10 or newer only if you work on the engine sidecar.

## Before you open a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p parlo-cli -- doctor
cargo run -p parlo-cli -- demo out/demo.wav

cd engines/hibiki && python -m unittest discover -s tests

python tools/licenses.py check
```

CI runs the same checks on Linux, macOS, and Windows. A pull request that fails them will not be merged.

## What makes a good pull request

- One concern per pull request. A bug fix does not carry a refactor.
- The change is the smallest one that fully solves the problem.
- New non-trivial logic comes with a test.
- Public behaviour changes come with a documentation update.
- No new dependency unless the alternative is meaningfully worse. Explain it in the description.

## Code style

- English identifiers, English messages, no comments: the code should explain itself, and the `docs/` folder explains the design.
- Keep the audio path allocation-light and fallible at the edges.
- Errors are typed, displayed as full sentences, and never swallowed.

## Commit messages

Short imperative sentences in English, describing the change and why it matters. One logical change per commit.

## Licensing a contribution

Every commit carries a Developer Certificate of Origin sign-off:

```bash
git commit -s -m "your message"
```

The sign-off line, `Signed-off-by: Your Name <you@example.com>`, states that you wrote the patch or otherwise have the right to submit it under the licence of this repository. CI checks every commit in a pull request for it.

You keep the copyright of what you write. No assignment is asked for.

Never paste code from a source you cannot cite. If a change follows another project closely, name it and its licence in the pull request. A dependency under a copyleft licence belongs in an optional component, never in the core, and `python tools/licenses.py check` refuses it there.

## Good first issues

- Add a hardware report for a machine you own.
- Translate the README into your language.
- Test Parlo with a calling app we have not covered.
- Improve an error message that confused you.

## Reporting bugs

Include the output of `parlo doctor`, your operating system and version, the exact command, and what you expected. Audio issues are much easier to fix with a short recording, but never upload a recording of another person without their consent.

## Security issues

Do not open a public issue. Follow [SECURITY.md](SECURITY.md).

## Code of conduct

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
