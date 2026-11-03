# Model policy

What may enter Parlo, how it is checked, and what the product promises about it. `docs/MODEL_LICENSES.md` holds the per-artefact table, `docs/MODEL_MANAGER.md` the behaviour a person sees, and `tools/licenses.py` the automated gate that fails a build when this document and the repository disagree.

## Rules

- No model is downloaded without a click that names it, its size and its licence. The first start recommends, it never fetches on its own.
- Nothing leaves the machine. A model runs locally; the only network use is the download itself and the optional cloud provider the user configures in Settings, Privacy.
- A model is a file with a hash. The published sha256 and size are checked before the file is used, a mismatch leaves nothing behind, and a repaired file is fetched again rather than trusted.
- A model can be removed completely, and a feature that needs it says so in a sentence instead of failing later.
- Removing a model never removes a setting, a transcript or a recording.
- Every model states its licence and what it does. A size, memory, speed or quality number appears only where it was measured here, and the row says so when it was not. The library is grouped by purpose and the choice of what to install belongs to the person; nothing is tested for its own sake, only for a claim that is published or a feature that ships.

## Licences

| Scope | Allowed without review | Needs the owner's decision |
|---|---|---|
| Code | Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unlicense, Zlib | Anything copyleft in the core; anything unknown |
| Weights | Apache-2.0, MIT, CC-BY-4.0, CC0-1.0, OpenRAIL-M with acceptable use terms | Custom model licences, non-commercial terms, unclear redistribution |
| Voice | Own recordings, CC0, and models whose licence permits commercial use and whose voices carry consent | A voice cloned from a third party, whatever the licence |
| Translation packages | CC-BY-4.0 with attribution recorded | Packages whose licence is absent from the index |

`r-efi` is the single tracked exception in the Rust closure: it reaches a release only through the UEFI target of `getrandom`, and the gate asks cargo for every release target and fails if it appears in one. `OpenMDW-1.1` is **not** approved yet: it is an open model licence with its own terms and needs a reading before a Nemotron checkpoint can ship, even though the weights are downloadable today.

## Lifecycle states

A model is `Not installed`, `Installed · unloaded`, `Loaded` or `In use`. `Unload` frees memory and keeps the files; `Remove` deletes the files and leaves the feature saying what it now needs. The engine records the last real use per stage, and a model is unloaded after the delay that applies to it, or kept while a task is running.

## Quality claims

A quality number is published only with its corpus, its hardware, its commit and the policy that produced it, as in `docs/BENCHMARKS.md`. A number measured on synthetic speech is labelled synthetic. A number measured on another language is not a number for Portuguese. Where a model card claims a language the project has not measured, the row says claimed, not measured.

## Adding a model

1. Find the artefact, its licence and its redistribution terms.
2. Record it in `catalog/models.json` with hash, size, licence and source.
3. Run the licence gate with `--strict --strict-models`.
4. Measure it when it is wired into a feature, on the corpora that feature uses, on the reference machine, and write the row into `docs/BENCHMARKS.md`. Listing a model before it is measured is allowed: the row then says the number is not measured, which is not the same as a number.
5. Wire it into a feature only after the gate passes. A measured number is required before a quality claim is published, not before a model is listed.
