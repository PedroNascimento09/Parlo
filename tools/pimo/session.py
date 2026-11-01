from . import compat

TRANSLATION = "mt"
CLONE = "clone"


class LiveError(ValueError):
    pass


def live(entries, device, speakers, schema=1):
    problems = _speakers(speakers)
    if problems:
        raise LiveError("; ".join(problems))
    directions = [
        _direction(entries, device, speaker, listener, schema)
        for speaker in speakers
        for listener in speakers
        if speaker["id"] != listener["id"]
    ]
    return {"directions": directions, "notice": _notice(directions)}


def _speakers(speakers):
    if not isinstance(speakers, list) or len(speakers) < 2:
        return ["at least two speakers are required"]
    problems = []
    seen = set()
    for speaker in speakers:
        if not isinstance(speaker, dict):
            problems.append("every speaker must be an object")
            continue
        identifier = speaker.get("id")
        if not isinstance(identifier, str) or not identifier.strip():
            problems.append("speaker id must be a non-empty string")
        elif identifier in seen:
            problems.append(f"duplicate speaker id: {identifier}")
        else:
            seen.add(identifier)
        if not isinstance(speaker.get("locale"), str) or not speaker["locale"].strip():
            problems.append(f"speaker {identifier}: locale must be a non-empty string")
    return problems


def _direction(entries, device, speaker, listener, schema):
    target = listener["locale"]
    direction = {"from": speaker["id"], "to": listener["id"], "locale": target}
    if speaker["locale"] == target:
        direction["translation"] = {"id": None, "reason": "same locale, nothing to translate"}
        direction["voice"] = {"id": None, "reason": "same locale, nothing to speak"}
        direction["identity"] = "not applicable"
        return direction
    direction["translation"] = _translation(entries, device, target, schema)
    direction["voice"] = _voice(entries, device, speaker, target, schema)
    direction["identity"] = direction["voice"]["identity"]
    return direction


def _translation(entries, device, locale, schema):
    admitted = compat.rank(
        entries, device, task=TRANSLATION, locale=locale, schema=schema
    )["admitted"]
    if not admitted:
        return {"id": None, "reason": f"no admitted translation model for {locale}"}
    if len(admitted) > 1:
        return {
            "id": None,
            "reason": "several admitted models without a measured comparison",
            "candidates": sorted(admitted),
        }
    return {"id": admitted[0], "reason": None}


def _voice(entries, device, speaker, locale, schema):
    if not speaker.get("profile"):
        return {
            "id": None,
            "reason": "no voice profile for this speaker",
            "identity": "unavailable",
        }
    if speaker.get("authorised") is not True:
        return {
            "id": None,
            "reason": "voice profile not authorised for this use",
            "identity": "unavailable",
        }
    admitted = compat.rank(
        entries, device, task=CLONE, locale=locale, schema=schema
    )["admitted"]
    if not admitted:
        return {
            "id": None,
            "reason": f"no admitted cloning model that speaks {locale}",
            "identity": "unavailable",
        }
    if len(admitted) > 1:
        return {
            "id": None,
            "reason": "several admitted clones without a measured comparison",
            "candidates": sorted(admitted),
            "identity": "unmeasured",
        }
    entry = _entry(entries, admitted[0])
    identity = "measured" if entry.get("benchmarks", {}).get("identity") else "unmeasured"
    return {"id": admitted[0], "reason": None, "identity": identity}


def _entry(entries, identifier):
    for entry in entries:
        if entry.get("id") == identifier:
            return entry
    return {}


def _notice(directions):
    states = [
        direction["identity"]
        for direction in directions
        if direction["identity"] != "not applicable"
    ]
    if not states:
        return "every direction is between speakers of the same locale"
    if all(state == "measured" for state in states):
        return None
    if "unavailable" in states:
        return "identity cannot be promised in at least one direction"
    return "identity is not measured in at least one direction"
