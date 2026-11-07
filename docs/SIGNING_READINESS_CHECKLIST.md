# Signing readiness checklist

This checklist is for the project’s own use if it ever decides to pursue a free open-source signing route for Windows distribution. It is not a promise that any provider will accept the project.

## Public identity of the project

- [ ] the repository is public;
- [ ] there is at least one public release;
- [ ] the project is actively maintained enough to look alive, not abandoned;
- [ ] there is a download page, release page, or store entry that describes what the software does.

## License fit

- [ ] the project uses an OSI-approved open-source license;
- [ ] the license posture is consistent across the components actually shipped;
- [ ] there is no commercial dual-licensing scheme that would conflict with a free open-source signing route.

## Distributed contents

- [ ] the distributed installer/archive is built from public source;
- [ ] the distribution does not include malware or unwanted/PUA-style behavior;
- [ ] the distribution does not include proprietary bundled code that would disqualify an open-source signing route;
- [ ] third-party dependencies in the shipped artifacts have licenses that fit the project’s distribution model.

## Build and provenance

- [ ] the release workflow builds the artifact from the public repository;
- [ ] the release workflow is usable from a GitHub-hosted or equivalent trusted build system;
- [ ] the project can tell a signing provider what source repo, branch, commit, and build job produced the artifact;
- [ ] checksums and provenance are published with releases.

## Signing operational readiness

- [ ] someone is responsible for release signing approval if manual approval is required;
- [ ] the project has a documented code-signing policy;
- [ ] the project has a privacy statement or privacy documentation where users can read what the software does and does not transmit;
- [ ] the project can follow a provider’s signing policy without breaking the build/release process.

## Honest limits

Even if all of the above is true:

- acceptance is still up to the provider;
- the visible publisher may be the provider rather than you;
- signing may not remove every warning immediately for every user;
- signing does not replace honest documentation or real verification of release integrity.

Use this checklist to prepare, not to assume.

## Local note for this project

This project currently has public releases and an Apache-2.0 code license for the project code. Before any signing route is pursued, the maintainer should confirm the current state of:

- repository visibility;
- third-party components in the actual distributed artifacts, not only in development;
- any documentation gap between what the project says and what the signing provider will want to see;
- whether the project is comfortable with the publisher name that any chosen route would show.
