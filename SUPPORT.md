# Support

Parlo is a small open-source project without a support contract. This page says what you can reasonably expect, and what you cannot.

## Where to ask

| You want | Where |
|---|---|
| To report a bug or a wrong translation | [Issues](https://github.com/PedroNascimento09/parlo/issues) |
| To ask a question or compare hardware | [Discussions](https://github.com/PedroNascimento09/parlo/discussions) |
| To report a security problem | [SECURITY.md](SECURITY.md), privately |
| To contribute a fix | [CONTRIBUTING.md](CONTRIBUTING.md) |

## Before you open an issue

Attach the output of `parlo doctor` and `parlo profile`, name your operating system and version, and say which engine and recogniser you used. A short recording helps enormously, but never upload a recording of another person without their consent.

If something is flaky, say whether it failed once or every time, and whether it failed on a cold start or a warm one. That distinction matters more than it looks like it should.

Known limits are documented: no GPU is required but slower machines run a smaller window, dubbing needs `ffmpeg`, and the virtual microphone needs Linux and PipeWire.

## What you will not get

- A response time promise. This is maintained in spare time.
- Support for private builds, custom models or commercial deployments.
- Help with a modified copy of the code where the modification is the cause. The issue tracker is for the code in this repository.

## Versions

Only the latest release and `main` receive fixes. An older release is answered with "please update" unless the problem is a security issue.

If a bug report is missing the basics above, it is not ignored on purpose; it is just harder to act on. Say what you did, what happened, and what you expected, and the issue is already most of the way there.
