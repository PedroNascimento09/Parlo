import argparse
import json
import re
import subprocess
from dataclasses import dataclass, replace
from importlib import metadata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TOOLS = Path(__file__).resolve().parent
POLICY_PATH = TOOLS / "license-policy.json"
MANIFEST_PATH = TOOLS / "python-packages.json"
CATALOG_PATH = ROOT / "catalog" / "models.json"
CARGO_MANIFEST = ROOT / "Cargo.toml"

REQUIREMENTS = {
    "core": ROOT / "engines" / "cascade" / "requirements.txt",
    "optional": ROOT / "engines" / "tts-piper" / "requirements.txt",
}

ALIASES = {
    "mit": "MIT",
    "apache": "Apache-2.0",
    "apache 2.0": "Apache-2.0",
    "apache license 2.0": "Apache-2.0",
    "apache software": "Apache-2.0",
    "apache software license": "Apache-2.0",
    "3-clause bsd": "BSD-3-Clause",
    "bsd 3-clause": "BSD-3-Clause",
    "python software foundation license": "PSF-2.0",
    "mozilla public license 2.0 (mpl 2.0)": "MPL-2.0",
    "isc license (iscl)": "ISC",
    "the unlicense (unlicense)": "Unlicense",
    "gnu general public license v3 (gplv3)": "GPL-3.0-only",
    "gnu general public license v3 or later (gplv3+)": "GPL-3.0-or-later",
    "gnu lesser general public license v3 (lgplv3)": "LGPL-3.0-only",
    "gnu lesser general public license v3 or later (lgplv3+)": "LGPL-3.0-or-later",
    "gnu affero general public license v3": "AGPL-3.0-only",
}

CLEANUPS = (
    (r",?\s*as found in the licen[cs]e file\.?", ""),
    (r"\s+licen[cs]ed$", ""),
    (r"\s+licen[cs]e$", ""),
    (r"\s*\+\s*file\s+\S+$", ""),
)

SPLIT = re.compile(r"\s+(?:OR|AND)\s+", re.IGNORECASE)


@dataclass(frozen=True)
class Component:
    name: str
    version: str
    license: str
    scope: str
    kind: str
    url: str

    def as_dict(self) -> dict:
        return {
            "name": self.name,
            "version": self.version,
            "license": self.license,
            "scope": self.scope,
            "kind": self.kind,
            "url": self.url,
        }


def canonical(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).strip().lower()


def normalise(raw: str) -> str:
    text = " ".join((raw or "").split())
    for pattern, replacement in CLEANUPS:
        text = re.sub(pattern, replacement, text, flags=re.IGNORECASE)
    text = text.strip()
    if not text:
        return "UNKNOWN"
    resolved = ALIASES.get(text.lower())
    if resolved:
        return resolved
    if len(text) > 96:
        return "UNKNOWN"
    return text


def tokens(expression: str) -> list[str]:
    text = re.sub(r"\s+WITH\s+\S+", "", expression, flags=re.IGNORECASE)
    text = text.replace("(", " ").replace(")", " ")
    parts: list[str] = []
    for chunk in SPLIT.split(text):
        for token in chunk.split("/"):
            if token.strip():
                parts.append(token.strip())
    return parts


def policy() -> dict:
    return json.loads(POLICY_PATH.read_text(encoding="utf-8"))


def installed_index() -> dict[str, metadata.Distribution]:
    index: dict[str, metadata.Distribution] = {}
    for distribution in metadata.distributions():
        name = (distribution.metadata.get("Name") or "").strip()
        if name:
            index[canonical(name)] = distribution
    return index


def distribution_license(distribution: metadata.Distribution) -> str:
    declared = normalise(
        distribution.metadata.get("License-Expression")
        or distribution.metadata.get("License")
        or ""
    )
    if declared != "UNKNOWN":
        return declared
    for classifier in distribution.metadata.get_all("Classifier") or []:
        if classifier.startswith("License :: OSI Approved ::"):
            label = classifier.rsplit("::", 1)[-1].strip()
            return normalise(label)
    return "UNKNOWN"


def from_distribution(distribution: metadata.Distribution, scope: str) -> Component:
    name = (distribution.metadata.get("Name") or "").strip()
    url = distribution.metadata.get("Home-page") or ""
    if not url:
        for entry in distribution.metadata.get_all("Project-URL") or []:
            if entry.lower().startswith("homepage"):
                url = entry.split(",", 1)[-1].strip()
                break
    return Component(
        name=name,
        version=distribution.version,
        license=distribution_license(distribution),
        scope=scope,
        kind="python",
        url=url,
    )


def requirement_lines(scope: str) -> list[str]:
    path = REQUIREMENTS[scope]
    if not path.is_file():
        return []
    lines = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.split("#")[0].strip()
        if line:
            lines.append(line)
    return lines


def requirement_pins(scope: str) -> tuple[dict[str, str], list[str]]:
    pins: dict[str, str] = {}
    problems: list[str] = []
    for line in requirement_lines(scope):
        match = re.match(r"([A-Za-z0-9][A-Za-z0-9._-]*)\s*==\s*([^\s;]+)$", line)
        if match:
            pins[canonical(match.group(1))] = match.group(2)
        else:
            problems.append(f"{REQUIREMENTS[scope].name}: {line} is not pinned to an exact version")
    return pins, problems


def python_components() -> tuple[list[Component], list[str]]:
    index = installed_index()
    resolved: dict[str, Component] = {}
    skipped: list[str] = []
    queue = [
        (name, scope, True)
        for scope in REQUIREMENTS
        for name in (
            re.split(r"[<>=!~\[; ]", line, maxsplit=1)[0] for line in requirement_lines(scope)
        )
        if name
    ]
    while queue:
        name, scope, direct = queue.pop(0)
        key = canonical(name)
        if key in resolved:
            if scope == "core" and resolved[key].scope != "core":
                resolved[key] = replace(resolved[key], scope="core")
            continue
        distribution = index.get(key)
        if distribution is None:
            if direct:
                raise SystemExit(
                    f"{name} is required by the {scope} components but is not installed; "
                    f"install it from {REQUIREMENTS[scope]}"
                )
            skipped.append(name)
            continue
        resolved[key] = from_distribution(distribution, scope)
        for requirement in distribution.requires or []:
            match = re.match(r"\s*([A-Za-z0-9][A-Za-z0-9._-]*)", requirement)
            if match:
                queue.append((match.group(1), scope, False))
    return sorted(resolved.values(), key=lambda component: component.name.lower()), skipped


def rust_components() -> list[Component]:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--all-features"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(f"cargo metadata failed:\n{completed.stderr.strip()}")
    data = json.loads(completed.stdout)
    members = set(data["workspace_members"])
    packages = {package["id"]: package for package in data["packages"]}
    components = []
    for node in data["resolve"]["nodes"]:
        if node["id"] in members:
            continue
        package = packages[node["id"]]
        components.append(
            Component(
                name=package["name"],
                version=package["version"],
                license=normalise(package.get("license") or ""),
                scope="core",
                kind="rust",
                url=package.get("repository") or f"https://crates.io/crates/{package['name']}",
            )
        )
    return sorted(components, key=lambda component: component.name.lower())


def licence_problems(
    components: list[Component], rules: dict
) -> tuple[list[str], list[str], list[str]]:
    allowed = {scope: set(values) for scope, values in rules["allowed"].items()}
    denied = set(rules.get("denied", []))
    overrides = {canonical(name): value for name, value in rules.get("overrides", {}).items()}
    known_issues = {canonical(name): value for name, value in rules.get("known_issues", {}).items()}
    not_shipped = {canonical(name): value for name, value in rules.get("not_shipped", {}).items()}
    problems: list[str] = []
    known: list[str] = []
    exempted: list[str] = []
    for component in components:
        expression = overrides.get(canonical(component.name), component.license)
        found = tokens(expression)
        if not found or expression == "UNKNOWN":
            problems.append(f"{component.name} {component.version}: no licence metadata")
            continue
        reason = known_issues.get(canonical(component.name))
        exemption = not_shipped.get(canonical(component.name))
        for token in found:
            if exemption and token.split("-")[0] in ("AGPL", "LGPL", "GPL", "SSPL", "BUSL", "Elastic"):
                exempted.append(f"{component.name} {component.version}: {token}, {exemption}")
                continue
            if reason and token.split("-")[0] in ("AGPL", "LGPL", "GPL", "SSPL", "BUSL", "Elastic"):
                known.append(f"{component.name} {component.version}: {token}, {reason}")
                continue
            if token in denied:
                problems.append(f"{component.name} {component.version}: {token} is denied")
            elif token not in allowed.get(component.scope, set()):
                problems.append(
                    f"{component.name} {component.version}: {token} is not allowed for {component.scope}"
                )
    return problems, known, exempted


def target_graph(target: str) -> set[str]:
    completed = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", target],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(f"cargo metadata for {target} failed:\n{completed.stderr.strip()}")
    data = json.loads(completed.stdout)
    selected = {node["id"] for node in data["resolve"]["nodes"]}
    return {package["name"] for package in data["packages"] if package["id"] in selected}


def not_shipped_problems(exempt: dict, graphs: dict) -> list[str]:
    problems = []
    for target, names in sorted(graphs.items()):
        for name in sorted(exempt):
            if canonical(name) in names:
                problems.append(
                    f"{name} reaches {target}: the not shipped exemption is no longer true"
                )
    return problems


def pin_problems(components: list[Component]) -> list[str]:
    problems: list[str] = []
    for scope in REQUIREMENTS:
        pins, found = requirement_pins(scope)
        problems.extend(found)
        for component in components:
            if component.kind != "python" or component.scope != scope:
                continue
            key = canonical(component.name)
            if key in pins and pins[key] != component.version:
                problems.append(
                    f"{component.name}: requirements pin {pins[key]} but the recorded version is {component.version}"
                )
    return problems


def installed_problems(components: list[Component]) -> list[str]:
    index = installed_index()
    problems: list[str] = []
    for component in components:
        if component.kind != "python":
            continue
        distribution = index.get(canonical(component.name))
        if distribution is None:
            problems.append(f"{component.name} {component.version} is recorded but not installed")
        elif distribution.version != component.version:
            problems.append(
                f"{component.name}: installed {distribution.version} but the record says {component.version}; run refresh"
            )
    return problems


def catalog() -> dict:
    return json.loads(CATALOG_PATH.read_text(encoding="utf-8"))


def catalog_problems(data: dict) -> list[str]:
    problems: list[str] = []
    for artifact in data.get("artifacts", []):
        identifier = artifact.get("id") or "?"
        for field in ("id", "kind", "license", "source", "status"):
            if not artifact.get(field):
                problems.append(f"catalogue {identifier}: missing {field}")
        status = artifact.get("status")
        if status not in ("verified", "unverified"):
            problems.append(f"catalogue {identifier}: unknown status {status}")
        if status == "verified" and not artifact.get("attribution"):
            problems.append(f"catalogue {identifier}: verified without attribution")
    return problems


def run_check(verify_installed: bool, strict_models: bool, strict: bool) -> int:
    rules = policy()
    python, skipped = python_components()
    components = python + rust_components()
    problems, known, exempted = licence_problems(components, rules)
    problems += pin_problems(components)
    graphs = {
        target: target_graph(target) for target in rules.get("release_targets", [])
    }
    problems += not_shipped_problems(rules.get("not_shipped", {}), graphs)
    if verify_installed:
        problems += installed_problems(python)
    data = catalog()
    problems += catalog_problems(data)
    unverified = [entry["id"] for entry in data.get("artifacts", []) if entry.get("status") == "unverified"]
    if strict_models and unverified:
        problems.append("catalogue still has unverified artifacts: " + ", ".join(unverified))
    for problem in problems:
        print(f"FAIL {problem}")
    for entry in known:
        print(f"KNOWN {entry}")
    for entry in exempted:
        print(f"NOT SHIPPED {entry}")
    if strict and known:
        problems.append("known issues still open: " + str(len(known)))
    if unverified:
        print("NOTE unverified catalogue artifacts: " + ", ".join(unverified))
    if skipped:
        print("NOTE declared but not installed, left out: " + ", ".join(sorted(set(skipped))))
    print(f"{len(components)} components checked, {len(problems)} problems")
    return 1 if problems else 0


def run_refresh() -> int:
    components, skipped = python_components()
    payload = {"packages": [component.as_dict() for component in components]}
    MANIFEST_PATH.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"recorded {len(components)} python packages in {MANIFEST_PATH.name}")
    if skipped:
        print("left out, declared but not installed: " + ", ".join(sorted(set(skipped))))
    return 0


def recorded() -> list[Component]:
    if not MANIFEST_PATH.is_file():
        return []
    data = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    return [Component(**entry) for entry in data["packages"]]


OBLIGATIONS = {
    "Apache-2.0": "Keep this notice and state what changed",
    "BSD-2-Clause": "Reproduce the notice, no endorsement",
    "BSD-3-Clause": "Reproduce the notice, no endorsement",
    "CC-BY-4.0": "Credit the author",
    "CC-BY-NC-SA-4.0": "Credit the author, non commercial, share alike",
    "GPL-3.0-only": "Offer the corresponding source under the GPL",
    "GPL-3.0-or-later": "Offer the corresponding source under the GPL",
    "AGPL-3.0-only": "Offer the source, including to network users",
    "ISC": "Reproduce the notice",
    "LGPL-3.0-only": "Offer the library source and allow relinking",
    "LGPL-2.1-or-later": "Offer the library source and allow relinking",
    "MPL-2.0": "Publish changes to the covered files",
    "PSF-2.0": "Reproduce the notice",
}

ORIGINAL = (
    ("`parlo-core`", "`crates/parlo-core`"),
    ("`parlo-cli`", "`crates/parlo-cli`"),
    ("`parlo-app`", "`crates/parlo-app`"),
    ("Interface page", "`crates/parlo-cli/web`"),
    ("Cascade engine", "`engines/cascade`"),
    ("Licence gate and generators", "`tools`"),
    ("Reference scripts", "`scripts`"),
)

ADAPTERS = (
    (
        "Piper adapter",
        "`engines/tts-piper`",
        "GPL-3.0-or-later",
        "none",
        "OHF-voice, wrapper by Parlo contributors",
        "Optional",
        "Confirmed",
        "Offer the corresponding source under the GPL",
        "[GPL-3.0](licenses/GPL-3.0-or-later.txt)",
    ),
    (
        "Hibiki reference adapter",
        "`engines/hibiki`",
        "Apache-2.0",
        "CC BY-NC-SA 4.0, not confirmed per checkpoint",
        "Kyutai, adapter by Parlo contributors",
        "Optional",
        "Code confirmed, weights not verified",
        "Credit the author, non commercial",
        "https://github.com/kyutai-labs/hibiki",
    ),
)

COPYLEFT = (
    (
        "Piper adapter",
        "`engines/tts-piper`",
        "GPL-3.0-or-later",
        "none",
        "OHF-voice",
        "Optional",
        "Confirmed",
        "Offer the corresponding source under the GPL",
        "[GPL-3.0](licenses/GPL-3.0-or-later.txt)",
    ),
    (
        "r-efi",
        "dependency of `getrandom`, UEFI target only",
        "LGPL-2.1-or-later",
        "none",
        "Rust UEFI project",
        "Not shipped",
        "Absent from every release target, and the gate fails if it appears in one",
        "Offer the library source and allow relinking",
        "https://github.com/rust-osdev/r-efi",
    ),
)

HEADER = (
    "| Component | Location | Code licence | Weights, voices or data | Holder | Required | Confirmed | Obligations | Licence text |",
    "|---|---|---|---|---|---|---|---|---|",
)


def row(*cells: str) -> str:
    return "| " + " | ".join((cell or "-").replace("|", "\\|") for cell in cells) + " |"


def obligation(expression: str) -> str:
    return OBLIGATIONS.get(expression, "See the licence text")


def index_document(components: list[Component]) -> str:
    artifacts = catalog().get("artifacts", [])
    lines = [
        "# Licences",
        "",
        "Every licence Parlo carries, on one page: the original code, the adapters, the third",
        "party dependencies, the model weights and the voices.",
        "",
        "The rule behind the tables is that a licence of code never covers the weights, the voices",
        "or the training data of the thing that code loads. Each row states what it covers.",
        "",
        "This file is generated by `python tools/licenses.py index`. CI regenerates it and fails",
        "when the committed copy differs, so it cannot drift away from the dependencies.",
        "",
        "## 1. Original Parlo code",
        "",
        "**Apache License 2.0**, from [`LICENSE`](LICENSE). Releases up to v0.1.7 were MIT and stay",
        "MIT: their text is kept in [`licenses/MIT-Parlo-0.1.x.txt`](licenses/MIT-Parlo-0.1.x.txt).",
        "",
        *HEADER,
    ]
    for name, location in ORIGINAL:
        lines.append(
            row(
                name,
                location,
                "Apache-2.0",
                "None",
                "Parlo contributors",
                "Required",
                "Confirmed",
                obligation("Apache-2.0"),
                "[Apache-2.0](licenses/Apache-2.0.txt)",
            )
        )
    lines += [
        "",
        "## 2. Adapters and integrations",
        "",
        "Code that connects the engine to a model living outside this repository. Each adapter may",
        "carry a different licence from the core.",
        "",
        *HEADER,
    ]
    for entry in ADAPTERS:
        lines.append(row(*entry))
    lines += [
        "",
        "## 3. Third party dependencies",
        "",
        "The licences here are the licences of the code. Versions, and the full closure with every",
        "transitive package, are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and in the",
        "SBOM attached to each release.",
        "",
        "### 3.1 Required by the release",
        "",
        *HEADER,
    ]
    for component in (item for item in components if item.scope == "core"):
        location = "`parlo-engine`" if component.kind == "python" else "Rust programs"
        lines.append(
            row(
                component.name,
                location,
                component.license,
                "None",
                "See the project",
                "Required",
                "Confirmed",
                obligation(component.license),
                component.url or "-",
            )
        )
    lines += ["", "### 3.2 Optional, installed by the user", "", *HEADER]
    optional = [item for item in components if item.scope == "optional"]
    if optional:
        for component in optional:
            lines.append(
                row(
                    component.name,
                    "`engines/tts-piper`",
                    component.license,
                    "None",
                    "See the project",
                    "Optional",
                    "Confirmed",
                    obligation(component.license),
                    component.url or "-",
                )
            )
    else:
        lines.append("| None | - | - | - | - | - | - | - | - |")
    weights = [item for item in artifacts if item.get("kind") in ("asr", "tts", "mt")]
    lines += [
        "",
        "## 4. Models and weights",
        "",
        "Never inside a release archive. The user downloads them from the publisher named in the",
        "source column, and they keep their own licence. A model marked unverified is not offered",
        "as if its terms were known.",
        "",
        *HEADER,
    ]
    for artifact in weights:
        lines.append(
            row(
                artifact["id"],
                "Downloaded, never shipped",
                "See the runtime dependencies",
                artifact.get("license") or "Unverified",
                artifact.get("attribution") or "See the source",
                "External",
                "Confirmed" if artifact.get("status") == "verified" else "Not verified",
                obligation(artifact.get("license") or ""),
                artifact.get("source") or "-",
            )
        )
    voices = [item for item in artifacts if item.get("kind") == "voice"]
    lines += [
        "",
        "## 5. Voices and datasets",
        "",
        "A voice carries the terms of the dataset it was trained on, which is often not the licence",
        "of the synthesizer that reads it. No dataset is redistributed by Parlo.",
        "",
        *HEADER,
    ]
    for artifact in voices:
        lines.append(
            row(
                artifact["id"],
                "Downloaded, never shipped",
                "See the runtime dependencies",
                artifact.get("license") or "Unverified",
                artifact.get("attribution") or "See the source",
                "External",
                "Confirmed" if artifact.get("status") == "verified" else "Not verified",
                obligation(artifact.get("license") or ""),
                artifact.get("source") or "-",
            )
        )
    lines += [
        "",
        "## 6. Optional components under copyleft",
        "",
        "These are the only copyleft pieces the project touches. GPL is allowed in an optional",
        "component and refused in the core by `python tools/licenses.py check`.",
        "",
        *HEADER,
    ]
    for entry in COPYLEFT:
        lines.append(row(*entry))
    lines += [
        "",
        "## What this means in practice",
        "",
        "- Using Parlo locally, for your own calls and recordings, obliges you to nothing.",
        "- Shipping a product that contains the Parlo code means keeping `LICENSE` and `NOTICE` and",
        "  stating what you changed. Apache-2.0 adds no source obligation beyond that.",
        "- Shipping the optional Piper adapter means shipping it under the GPL, with its source.",
        "- Redistributing a model or a voice means following that artefact's own licence, not this",
        "  repository's.",
        "",
        "Open questions and the parts that need a lawyer are in [`docs/LICENSES.md`](docs/LICENSES.md).",
        "",
    ]
    return "\n".join(lines)


def effective(components: list[Component]) -> list[Component]:
    overrides = {canonical(name): value for name, value in policy().get("overrides", {}).items()}
    return [
        replace(component, license=overrides.get(canonical(component.name), component.license))
        for component in components
    ]


def notices(components: list[Component]) -> str:
    lines = [
        "# Third party notices",
        "",
        "Parlo is licensed under the Apache License 2.0, from `LICENSE`. Releases up to v0.1.7",
        "were MIT and keep their text; see `LICENSES.md` and `docs/LICENSES.md`.",
        "",
        "This file is generated by `python tools/licenses.py notices`. Do not edit it by hand.",
        "Regenerate it whenever a dependency or a catalogue entry changes.",
        "",
    ]
    titles = {
        "core": "## Required components",
        "optional": "## Optional components, installed separately",
    }
    for scope in ("core", "optional"):
        rows = [component for component in components if component.scope == scope]
        lines.append(titles[scope])
        lines.append("")
        if not rows:
            lines.append("None.")
            lines.append("")
            continue
        lines.append("| Component | Version | Licence | Source |")
        lines.append("|---|---|---|---|")
        for component in rows:
            lines.append(
                f"| {component.name} | {component.version} | {component.license} | {component.url or '-'} |"
            )
        lines.append("")
    data = catalog()
    lines.append("## Model artifacts")
    lines.append("")
    lines.append(
        "Model weights and voices are never shipped with Parlo. They are downloaded by the user"
    )
    lines.append("from the original source and each one keeps its own licence.")
    lines.append("")
    lines.append("| Artifact | Kind | Licence | Status | Source |")
    lines.append("|---|---|---|---|---|")
    for artifact in data.get("artifacts", []):
        lines.append(
            f"| {artifact['id']} | {artifact.get('kind', '-')} | {artifact.get('license', '-')} | "
            f"{artifact.get('status', '-')} | {artifact.get('source', '-')} |"
        )
    lines.append("")
    return "\n".join(lines)


def project_version() -> str:
    text = CARGO_MANIFEST.read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1) if match else "0.0.0"


def bom(components: list[Component]) -> dict:
    entries = []
    for component in components:
        entry = {
            "type": "library",
            "name": component.name,
            "version": component.version,
            "licenses": [{"expression": component.license}],
            "properties": [{"name": "parlo:scope", "value": component.scope}],
        }
        if component.url:
            entry["externalReferences"] = [{"type": "website", "url": component.url}]
        entries.append(entry)
    for artifact in catalog().get("artifacts", []):
        entries.append(
            {
                "type": "data",
                "name": artifact["id"],
                "version": artifact.get("version") or "unversioned",
                "licenses": [{"expression": artifact.get("license") or "NOASSERTION"}],
                "properties": [
                    {"name": "parlo:scope", "value": "downloadable"},
                    {"name": "parlo:status", "value": artifact.get("status", "unverified")},
                ],
                "externalReferences": [{"type": "distribution", "url": artifact.get("source") or ""}],
            }
        )
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "component": {"type": "application", "name": "parlo", "version": project_version()}
        },
        "components": entries,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Parlo licence inventory and compliance gate")
    subcommands = parser.add_subparsers(dest="command", required=True)
    subcommands.add_parser("refresh", help="record the installed python dependency closure")
    check = subcommands.add_parser("check", help="verify every component against the policy")
    check.add_argument("--verify-installed", action="store_true")
    check.add_argument("--strict-models", action="store_true")
    check.add_argument("--strict", action="store_true", help="treat tracked known issues as failures")
    notice = subcommands.add_parser("notices", help="write THIRD_PARTY_NOTICES")
    notice.add_argument("--out", type=Path)
    sbom = subcommands.add_parser("sbom", help="write a CycloneDX SBOM")
    sbom.add_argument("--out", type=Path)
    index = subcommands.add_parser("index", help="write the LICENSES.md index")
    index.add_argument("--out", type=Path, default=ROOT / "LICENSES.md")
    arguments = parser.parse_args()

    if arguments.command == "refresh":
        return run_refresh()
    if arguments.command == "check":
        return run_check(arguments.verify_installed, arguments.strict_models, arguments.strict)

    components = recorded()
    if not components:
        components = python_components()[0]
    components += rust_components()
    components = effective(components)
    if arguments.command == "index":
        arguments.out.write_text(index_document(components), encoding="utf-8")
        print(f"wrote {arguments.out}")
        return 0
    if arguments.command == "notices":
        text = notices(components)
        if arguments.out:
            arguments.out.write_text(text, encoding="utf-8")
            print(f"wrote {arguments.out}")
        else:
            print(text)
        return 0

    payload = json.dumps(bom(components), indent=2) + "\n"
    if arguments.out:
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(payload, encoding="utf-8")
        print(f"wrote {arguments.out}")
    else:
        print(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
