import json
import re
from pathlib import Path

EVIDENCE_LEVELS = ("declared", "documented", "validated", "experimental", "unconfirmed")
STATES = ("recommended", "compatible", "experimental", "blocked", "external")
AUTO_STATES = ("recommended", "compatible")
TASKS = ("asr", "mt", "tts", "clone", "vad", "diarization", "align")
SYSTEMS = ("windows", "macos", "linux")
ARCHITECTURES = ("x86_64", "arm64")
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._-]*$")
CHECKSUM = re.compile(r"^sha256:[0-9a-f]{64}$")

TOP_FIELDS = (
    "manifest_version",
    "id",
    "name",
    "version",
    "task",
    "state",
    "evidence",
    "origin",
    "url",
    "checksum",
    "license",
    "languages",
    "runtime",
    "platforms",
    "resources",
    "api",
    "capabilities",
    "limits",
    "benchmarks",
    "maintenance",
)


class ManifestError(ValueError):
    pass


def load(path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ManifestError(f"{path}: {error}") from error


def entries(document):
    models = document.get("models") if isinstance(document, dict) else document
    if not isinstance(models, list):
        raise ManifestError(
            "catalogue must be a list or an object with a models list; "
            "schema 1 licence indexes are not manifests"
        )
    return models


def validate(entry):
    if not isinstance(entry, dict):
        return ["entry: not an object"]
    problems = [f"{field}: missing" for field in TOP_FIELDS if field not in entry]
    if problems:
        return problems
    problems.extend(_identity(entry))
    problems.extend(_licence(entry))
    problems.extend(_language(entry))
    problems.extend(_runtime(entry))
    problems.extend(_platforms(entry))
    problems.extend(_resources(entry))
    problems.extend(_api(entry))
    problems.extend(_honesty(entry))
    return problems


def is_auto_eligible(entry):
    if validate(entry):
        return False
    if entry["state"] not in AUTO_STATES:
        return False
    if entry["evidence"] != "validated":
        return False
    return entry["license"].get("weights_verified") is True


def _identity(entry):
    problems = []
    if not isinstance(entry["manifest_version"], int) or entry["manifest_version"] < 1:
        problems.append("manifest_version: must be an integer of at least 1")
    if not isinstance(entry["id"], str) or not IDENTIFIER.match(entry["id"] or ""):
        problems.append("id: must be a lowercase identifier")
    for field in ("name", "version", "origin"):
        if not isinstance(entry[field], str) or not entry[field].strip():
            problems.append(f"{field}: must be a non-empty string")
    if entry["task"] not in TASKS:
        problems.append(f"task: must be one of {', '.join(TASKS)}")
    if entry["state"] not in STATES:
        problems.append(f"state: must be one of {', '.join(STATES)}")
    if entry["evidence"] not in EVIDENCE_LEVELS:
        problems.append(f"evidence: must be one of {', '.join(EVIDENCE_LEVELS)}")
    if not isinstance(entry["url"], str) or not entry["url"].startswith("https://"):
        problems.append("url: must be an https address")
    if not isinstance(entry["checksum"], str) or not CHECKSUM.match(entry["checksum"]):
        problems.append("checksum: must be sha256 followed by 64 lowercase hex digits")
    if entry["state"] in AUTO_STATES and entry["evidence"] != "validated":
        problems.append("state: automatic states require validated evidence")
    return problems


def _licence(entry):
    problems = []
    licence = entry["license"]
    if not isinstance(licence, dict):
        return ["license: must be an object"]
    for field in ("code", "weights", "attribution"):
        if not isinstance(licence.get(field), str) or not licence[field].strip():
            problems.append(f"license.{field}: must be a non-empty string")
    if not isinstance(licence.get("weights_verified"), bool):
        problems.append("license.weights_verified: must be true or false")
    data = licence.get("data")
    if data is not None and (not isinstance(data, str) or not data.strip()):
        problems.append("license.data: must be a string when present")
    return problems


def _language(entry):
    problems = []
    languages = entry["languages"]
    if not isinstance(languages, dict):
        return ["languages: must be an object"]
    declared = languages.get("declared")
    validated = languages.get("validated")
    if not isinstance(declared, list) or not declared or not all(isinstance(item, str) for item in declared):
        problems.append("languages.declared: must be a non-empty list of strings")
    if not isinstance(validated, list) or not all(isinstance(item, str) for item in validated):
        problems.append("languages.validated: must be a list of strings")
    if isinstance(declared, list) and isinstance(validated, list):
        extra = [item for item in validated if item not in declared]
        if extra:
            problems.append(f"languages.validated: not declared: {', '.join(extra)}")
    return problems


def _runtime(entry):
    runtime = entry["runtime"]
    if not isinstance(runtime, dict):
        return ["runtime: must be an object"]
    problems = []
    for field in ("engine", "min_version"):
        if not isinstance(runtime.get(field), str) or not runtime[field].strip():
            problems.append(f"runtime.{field}: must be a non-empty string")
    formats = runtime.get("formats")
    if not isinstance(formats, list) or not formats or not all(isinstance(item, str) for item in formats):
        problems.append("runtime.formats: must be a non-empty list of strings")
    return problems


def _platforms(entry):
    platforms = entry["platforms"]
    if not isinstance(platforms, dict):
        return ["platforms: must be an object"]
    problems = []
    for field, allowed in (("os", SYSTEMS), ("arch", ARCHITECTURES)):
        values = platforms.get(field)
        if not isinstance(values, list) or not values:
            problems.append(f"platforms.{field}: must be a non-empty list")
            continue
        unknown = [item for item in values if item not in allowed]
        if unknown:
            problems.append(f"platforms.{field}: unknown values: {', '.join(unknown)}")
    backends = platforms.get("backends")
    if not isinstance(backends, list) or not backends or not all(isinstance(item, str) for item in backends):
        problems.append("platforms.backends: must be a non-empty list of strings")
    return problems


def _resources(entry):
    resources = entry["resources"]
    if not isinstance(resources, dict):
        return ["resources: must be an object"]
    problems = []
    for field in ("ram_gb", "disk_gb"):
        value = resources.get(field)
        if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= 0:
            problems.append(f"resources.{field}: must be a positive number")
    vram = resources.get("vram_gb", 0)
    if not isinstance(vram, (int, float)) or isinstance(vram, bool) or vram < 0:
        problems.append("resources.vram_gb: must be zero or a positive number")
    return problems


def _api(entry):
    api = entry["api"]
    if not isinstance(api, dict):
        return ["api: must be an object"]
    problems = []
    for field in ("min_schema", "max_schema"):
        value = api.get(field)
        if not isinstance(value, int) or isinstance(value, bool) or value < 1:
            problems.append(f"api.{field}: must be an integer of at least 1")
    if isinstance(api.get("min_schema"), int) and isinstance(api.get("max_schema"), int):
        if api["min_schema"] > api["max_schema"]:
            problems.append("api: min_schema is greater than max_schema")
    return problems


def _honesty(entry):
    problems = []
    limits = entry["limits"]
    if not isinstance(limits, list) or not limits or not all(isinstance(item, str) and item.strip() for item in limits):
        problems.append("limits: must be a non-empty list of strings")
    if not isinstance(entry["capabilities"], dict):
        problems.append("capabilities: must be an object")
    if not isinstance(entry["benchmarks"], dict):
        problems.append("benchmarks: must be an object")
    if not isinstance(entry["maintenance"], str) or not entry["maintenance"].strip():
        problems.append("maintenance: must be a non-empty string")
    return problems
