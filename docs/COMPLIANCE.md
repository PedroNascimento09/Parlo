# Compliance checklist

Internal record of what Parlo verifies before a release. It is not legal advice and does not claim legal compliance; it separates what is verified, what is pending, and what needs interpretation. The project is personal, free, open source and non-commercial, which reduces exposure but does not remove obligations.

Status legend: **verified** means the evidence exists in the repository; **pending** means the work is known and not done; **interpretation** means the answer depends on a legal reading that documentation alone does not settle.

## 1. Licences and attribution

| Item | Requirement | Evidence | Status |
|---|---|---|---|
| Original code | Apache-2.0 text and notice | `LICENSE`, `NOTICE` | verified |
| Releases up to v0.1.7 | MIT text preserved, no retroactive change | `licenses/MIT-Parlo-0.1.x.txt` | verified |
| Third-party components | Notices and attribution | `THIRD_PARTY_NOTICES.md` | verified |
| Dependency inventory | Machine-readable SBOM per release | CycloneDX artefact in `release.yml` | verified |
| Licence gate | Build fails on missing, denied or unpinned licence | `tools/licenses.py`, `tools/license-policy.json` | verified |
| Per-module index | Code, weights and voice licences separated | `LICENSES.md`, `docs/MODEL_LICENSES.md` | verified |
| Model artefacts | Licence, hash and state per checkpoint | `catalog/models.json` | verified, one artefact pending |
| Argos pack | Licence of the packaged translation data | catalogue entry | pending: removed from core, not catalogued as installable |
| Copyleft boundary | No GPL or AGPL code inside the core | `engines/tts-piper` external adapter only | verified |
| Combined-work question | Whether shipping a GPL adapter beside the core creates a combined work | analysis in plan section 80 | interpretation |

## 2. Voices and consent

| Item | Requirement | Evidence | Status |
|---|---|---|---|
| Consent screens before creation | Explanation of cloning, storage, purpose, deletion | plan section 5.1 | verified as design, pending as code |
| Consent before profile creation | Explicit declaration with purpose and scope | plan section 5.2 to 5.4 | pending: not implemented |
| Declared origin | Own voice or authorised third-party voice, recorded per profile | plan section 5.2 | pending: not implemented |
| Contextual notices | Seven defined moments, no needless repetition | plan section 5.5 | pending: not implemented |
| Revocation | Future use revoked and profile deleted with derived data | plan section 5.7 | pending: not implemented |
| Consent record | Local, dated, tied to profile and notice version | plan section 5.2 | pending: not implemented |
| Third-party voices | Authorisation required, deception forbidden | `ACCEPTABLE_USE.md`, plan section 25 | verified as policy, pending as UI |
| Deletion | Sample, embeddings, prompts and caches removed together | plan section 5.4 | pending: not implemented |
| Bundled voices | No third-party voice profiles shipped | repository contains none | verified |
| Example audio in docs | Only synthetic or author's own | repository contains none | verified |

## 3. Privacy and data protection

| Item | Requirement | Evidence | Status |
|---|---|---|---|
| Local by default | No audio, transcript or profile leaves the device | `docs/PRIVACY.md` | verified |
| Telemetry | None, with no opt-in path that collects by default | code and docs | verified |
| Network calls | Only the release and catalogue check, carrying no content or identifiers | plan sections 37 and 46 | verified for updates, pending for catalogue client |
| Retention | User-controlled, deletable per artefact | plan section 37 | pending: not implemented |
| Voice representations | Treated as personal data; biometric qualification depends on purpose | plan section 38 | interpretation |
| Training | No use of user audio for training | docs and code | verified |

## 4. European AI Act, Article 50

Transparency duties apply from 2 August 2026. The project documents them and does not assume that being open source removes them.

| Item | Who it applies to | What Parlo does | Status |
|---|---|---|---|
| Inform that output is synthetic | Provider | Interface labels synthesised speech | pending: no synthesis path shipped yet |
| Machine-readable marking | Provider of generative systems | Metadata plan per export format | pending: format work not done |
| Deepfake disclosure | Deployer who publishes the content | Warning and reminder at export | interpretation: exact form for each medium |
| Watermarking | Provider, where technically available | Only when the engine offers it, verified, never simulated | pending: engine survey |
| Provenance metadata | Provider of generative systems | Engine, version and date written into exported files | pending: format work |
| Limitations disclosure | Provider | Documented per mode and language | verified |

## 5. Distribution

| Item | Requirement | Evidence | Status |
|---|---|---|---|
| Installer without weights | Model files never inside release archives | `release.yml`, plan section 45 | verified |
| Release hashes | SHA-256 published per artefact | release workflow | verified |
| Windows signing | Authenticode not available | documented limitation | pending: no certificate |
| Manifest signing | Ed25519, public key in the application | plan section 53 | pending: not implemented |
| Downloads | Explicit consent, provenance and integrity checked | plan section 46 | verified as rule, pending as code |

## 6. Review before each release

- Run the licence gate, the notices generator and the index check; all three must pass.
- Confirm no new dependency entered without a licence in the allow list for its scope.
- Confirm no weights, voices or private audio entered the repository or the archive.
- Confirm the installer size is within 100 MB, or record the deviation and its cause.
- Confirm the catalogue is signed and the client rejects an unsigned manifest.
- Confirm every model listed as installable has licence, hash and state.
- Confirm privacy documentation matches the behaviour actually shipped.
- Confirm no voice profile can be created without a recorded consent and a declared origin.
- Confirm every exported synthetic audio carries provenance metadata, or says why it does not.
- Confirm no document claims watermarking or detection that the shipped engines do not provide.
- Record new risks or interpretation questions here instead of in a release note.

## 7. When to seek advice

Only when a concrete and material question cannot be answered from the official licence text, the applicable documentation or reasonable technical analysis. Current candidates: distributing a GPL adapter in the same download as the core, the disclosure form required for generated speech, and the biometric qualification of voice embeddings for a specific feature. None of them blocks development; each is recorded above with the reasoning that exists today.
