from .manifest import AUTO_STATES


def admit(entry, profile, task=None, locale=None, schema=1, allow_states=AUTO_STATES):
    if task is not None and entry.get("task") != task:
        return False, "task"
    if entry.get("state") not in allow_states:
        return False, f"state:{entry.get('state')}"
    if entry.get("evidence") != "validated":
        return False, "evidence"
    licence = entry.get("license") or {}
    if licence.get("weights_verified") is not True:
        return False, "license"

    platforms = entry.get("platforms") or {}
    if profile.get("os") not in (platforms.get("os") or []):
        return False, "os"
    if profile.get("arch") not in (platforms.get("arch") or []):
        return False, "arch"
    if not set(platforms.get("backends") or []) & set(profile.get("backends") or []):
        return False, "backend"

    api = entry.get("api") or {}
    low = api.get("min_schema")
    high = api.get("max_schema")
    if isinstance(low, int) and isinstance(high, int) and not low <= schema <= high:
        return False, "api"

    validated = (entry.get("languages") or {}).get("validated") or []
    if locale is not None and validated and locale not in validated:
        return False, "locale"

    resources = entry.get("resources") or {}
    if float(resources.get("ram_gb") or 0) > float(profile.get("ram_available_gb") or 0):
        return False, "ram"
    if float(resources.get("vram_gb") or 0) > float(profile.get("vram_gb") or 0):
        return False, "vram"
    if float(resources.get("disk_gb") or 0) > float(profile.get("disk_free_gb") or 0):
        return False, "disk"
    return True, None


def rank(catalogue, profile, task=None, locale=None, schema=1, allow_states=AUTO_STATES):
    admitted = []
    rejected = []
    for entry in catalogue:
        ok, reason = admit(
            entry,
            profile,
            task=task,
            locale=locale,
            schema=schema,
            allow_states=allow_states,
        )
        if ok:
            admitted.append(entry.get("id"))
        else:
            rejected.append({"id": entry.get("id"), "reason": reason})
    return {"admitted": sorted(admitted), "rejected": rejected}
