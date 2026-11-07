# Security policy

## Supported versions

Parlo is pre-release. Only the latest commit on `main` receives security fixes.

| Version | Supported |
|---|---|
| main | Yes |
| 0.1.x | Yes |
| older | No |

If you are reporting something that could affect people who install from a release, prefer the release version line rather than the development line.

## Reporting a vulnerability

Report privately through GitHub Security Advisories on the repository: **Security** tab, then **Report a vulnerability**.

Do not open a public issue, and do not include vulnerability details in a pull request description.

Please include:

- What the issue allows an attacker to do.
- The version or commit you can reproduce it on.
- Steps to reproduce, with the smallest proof you can build.
- Your operating system.

You can expect an acknowledgement within 72 hours and an assessment within 7 days. Fixes are released as soon as they are ready, and reporters are credited unless they ask otherwise.

## What Parlo is trying to protect

- The common case is a local, offline translation and dictation tool that should never send audio or transcripts anywhere unless the user explicitly enables a cloud path.
- If a change would make audio, transcripts, or voice samples leave the machine without the user asking, treat it as a serious issue.
- If a change would let one person’s files or settings be read or written by another person’s input, treat it as a serious issue.
- If a change would let an untrusted model file or untrusted input start running code, treat it as a serious issue.

## Scope

In scope:

- Arbitrary code execution or memory unsafety in the Rust crates.
- Path traversal or file overwrite through the CLI.
- Silent network transmission of audio.
- Privilege escalation in the virtual microphone installer.
- Malicious model files causing code execution.
- Vulnerabilities in the Python sidecar, including protocol parsing.

Out of scope:

- Denial of service by supplying enormous audio files to a local CLI.
- Attacks that require access to the user's unlocked machine.
- Issues in third-party models, which belong to their maintainers.
- Social engineering.

## Design commitments

- No audio leaves the machine in local mode.
- No telemetry and no hidden network calls.
- Model downloads happen only when the user asks, from the original source.
- Drivers are signed before distribution. An unsigned driver build is never released.

## Dependency policy

The audio core has zero dependencies by design. Every addition is reviewed for maintenance status, license compatibility, and the amount of code it brings. Regressions in that policy are treated as security issues.
