# Governance

Parlo is maintained by its copyright holders and grows through community contributions. This page says who decides what, so nobody has to guess.

Parlo is a personal project with a public process. That means the rules are written down, decisions are visible, and the project tries to say no in the open when it says no.

## Roles

- **Maintainers** merge changes, cut releases and speak for the project. Today that is the repository owner.
- **Contributors** send patches under the [Developer Certificate of Origin](https://developercertificate.org/) and keep the copyright of what they write.
- **Users** report bugs, measure hardware and translate the documentation.

A contributor who lands several accepted changes is invited to review pull requests in the area they touched. Review is not authority: a maintainer merges.

## Decisions

| Kind of decision | How it is taken |
|---|---|
| A bug fix, a documentation fix, a measurement | A maintainer merges when the checks pass |
| A new capability inside the roadmap | An issue with the design first, then a pull request |
| A new dependency | Justified in the pull request: what it replaces, its licence, its maintenance state, what it costs in bytes |
| A change to the licence of original code | Maintainers, after the copyright holders are confirmed, announced ahead of the release |
| A change to the public protocol, the config format or the release process | Documented in the pull request and in `CHANGELOG.md` |

Silence is not consent. A proposal that gets no review in fourteen days is not accepted by default; ping a maintainer.

## Licence and contributions

The project uses the DCO: every commit states that the author has the right to submit it. No copyright assignment is asked for. Contributions are accepted under the licence the repository carries at the moment of the merge, and the Apache-2.0 transition applies to the code as a whole once the holders confirm it.

Third-party code is never accepted without a traceable origin and a licence that is compatible with the component it lands in. `python tools/licenses.py check` enforces the result.

## Releases

A release happens when the accumulated work is worth a download, not per commit. The version advances one step from the last published tag, the notes live in `docs/releases/`, and a release only publishes when the licence gate passes in strict mode.

The project does not treat every commit as a release, and it does not bump a version just because it can. If a new version is not noticeably useful to someone downloading it, it is not a release yet.

## Conduct and security

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) and [SECURITY.md](SECURITY.md) apply to everyone, maintainers included.
