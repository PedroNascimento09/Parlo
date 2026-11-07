# Code signing policy

This is the project’s intended code-signing policy for Windows distribution. It is written so the project can be honest with users and, if needed later, ready to apply for a free open-source signing route.

## What we ship today

Today, Parlo ships an unsigned Windows installer. We do not pretend otherwise. The project is free and open-source, built on GitHub Actions from public source, and release integrity is supposed to be verified with the published SHA-256 checksums and the GitHub Actions build provenance alongside each release.

Because the installer is unsigned and has no established publisher reputation, Windows may warn on first download. That is expected behavior for this shape of release, not a claim that the file is malicious.

## Why we are honest about it

We do not promise that Windows will stop warning just because we change the icon, rename the installer, or make the package lighter. Windows trust for downloaded executables is about Authenticode signing and publisher reputation, not cosmetic improvements.

## What we want to achieve

The goal is to reduce unnecessary warning friction for future users while keeping the project free and open.

The realistic ways to do that are:

- a free code-signing route for qualifying open-source projects, if the project qualifies and is accepted;
- Microsoft Store distribution as an MSIX path, where Microsoft re-signs the package;
- or, separately and at cost, a trusted code-signing certificate used in a signing pipeline.

This policy exists to make clear that the project does not assume a free path exists before it exists.

## What is in scope for signing

Only the binaries and installer artifacts the project itself builds and distributes from public source are in scope. We will not sign third-party binaries as if they were ours, and we will not claim a distributed artifact is ours if it is not.

## Integrity expectations

Before any signing is considered, the project should be able to say, honestly:

- the repository is public;
- releases are built from public source in a reproducible-enough way;
- the license state of shipped components is clear;
- the distributed artifacts do not include proprietary or unwanted bundled components that would disqualify the project from an open-source signing route;
- the project’s functionality is documented on the download page or store entry.

## Future signing provider choice

If the project ever uses a free open-source signing service, the policy is to treat the provider’s terms as part of the distribution setup, including:

- the visible publisher name on the certificate;
- whether each release needs manual approval;
- the source-build and pipeline integrity requirements;
- any limits on who can request signing;
- the fact that approval is not guaranteed.

If the project ever uses a paid certificate, the decision and the operational requirements are owned by the person who pays for them.

## What users should read here

If you are a user, read this as: “today the installer is unsigned; if Windows warns, this document explains why and what verification steps exist; if a future release gets signed through a free open-source route or a store channel, that will be reflected in the release notes.”

If you are a maintainer, read this as: “do not advertise a free signing outcome the project has not earned yet; prepare the project so it can qualify; record the real result when it happens.”
