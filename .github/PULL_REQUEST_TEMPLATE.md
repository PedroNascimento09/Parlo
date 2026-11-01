## What this changes

<!-- One paragraph. The problem, the fix, and anything a reviewer should look at first. -->

## Why

<!-- The user impact. Link the issue if there is one. -->

## Checks

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Python tests, if the sidecar changed
- [ ] `python tools/licenses.py check`, if a dependency or a model changed
- [ ] Documentation updated, if behaviour changed
- [ ] Every commit is signed off (`git commit -s`)

## Risk

<!-- What could break, who is affected, and how to roll it back. -->
