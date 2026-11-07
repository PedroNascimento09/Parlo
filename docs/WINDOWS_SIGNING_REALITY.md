# Windows code-signing reality for Parlo

This project is free, open-source software. It does not send audio anywhere in local mode, and release integrity is verified by the published SHA-256 checksums and the GitHub Actions build provenance next to each release.

This document explains, in plain terms, what Windows will and will not trust, and what is free, what is conditional, and what is not free.

## What Windows is actually reacting to

For downloaded Windows installers, the relevant signal is not principally “this file looks strange” or “this installer is big”. It is:

- whether the executable is Authenticode-signed,
- who the publisher is,
- whether that publisher has built reputation with Microsoft SmartScreen over time.

A freshly built, unsigned, publicly released installer from a new project is exactly the shape that triggers SmartScreen “Windows protected your PC” and similar warnings. That is why the Handy installer can look normal to you on a transfer while Parlo still warns: Handy is arriving through a distribution path and publisher identity the system already has context for; Parlo currently ships unsigned with a new hash every release and has no established publisher reputation.

This is a distribution and trust-engineering problem, not a cosmetic one.

## What is free right now

These things are free and genuinely useful:

- keep the installer small and the first-run experience simple;
- publish the exact SHA-256 for each release artifact;
- publish build provenance from GitHub Actions;
- tell users the honest state: unsigned installer, SmartScreen may warn once, here is how to verify the file, here is how to bypass once if you trust the source;
- when a real user is blocked, submit the exact blocked file to Microsoft as a false positive through the Microsoft Security Intelligence file submission portal, as a software developer, marked clean;
- keep the project free, public, and documented.

These reduce harm and improve trust on a per-installation basis for people who will check, but they do not make every future download stop warning.

## What is free only with conditions

There is a free code-signing route for qualifying open-source projects: SignPath Foundation. It provides OV-level code signing through a managed pipeline, with the private key held in their HSM, and the certificate is issued to SignPath Foundation rather than to you personally.

Important conditions and limits of that route:

- it is intended for qualifying open-source projects;
- the project must usually be public, active, already released, and documented;
- the project should use an OSI-approved open-source license across its components and contain no proprietary or adware/PUA-style components;
- each release may require manual approval or a controlled pipeline policy;
- acceptance is discretionary; applying does not guarantee approval;
- the visible publisher can be SignPath Foundation, depending on the certificate and policy, not necessarily you.

So this is genuinely free for some projects, but it is not “configure and forget”, and it is not guaranteed for this project until the project applies and is approved.

## What is not free

If you want Windows to treat the project as a trusted publisher of your own name or brand, the durable route is a trusted code-signing certificate, and that usually costs money and carries operational requirements. That is the ordinary path behind “my app no longer warns because I am a known publisher”.

Also: a self-signed certificate is not a public-trust solution. It is useful for local development and for managed devices where you control trust, but it does not help random users downloading the installer.

## What does not change the equation

None of the following removes the need for signing or a trusted distribution channel:

- making the installer smaller;
- changing the icon;
- renaming the setup file;
- adding more screenshots or nicer copy;
- making the installer look more like another app.

Those can improve human perception of the release, which matters, but they do not persuade Windows that the file is from a trusted publisher.

## What this project currently does

Right now, Parlo ships unsigned on Windows. The release workflow has a signing hook that stays quiet until signing credentials are configured, but no certificate is configured for this project yet. The honest state is written next to the download: unsigned installer, SmartScreen may warn once, verify with the checksum and provenance.

## If you want a durable fix later

If, at some point, you want to reduce the warning for future users in a lasting way, the realistic free-with-conditions route is to prepare the project to apply for a program like SignPath Foundation, and/or evaluate Microsoft Store distribution as an MSIX path where Microsoft re-signs the package.

If instead you decide to pay for a certificate and run your own signing pipeline, that is a separate decision from what this document is for. This document exists so the project does not promise a free path that does not exist.

## For maintainers

Before considering signing, make sure:

- the repository is public;
- the release process is reproducible from public source;
- the license state of all shipped components is clear and consistent with the project’s distribution model;
- there are no proprietary or unwanted bundled components in the distributed artifacts;
- the project’s functionality is documented on the download page or store entry.

If and when you apply to a signing provider, record the outcome in the release notes or a closed issue so repeat flags have context.
