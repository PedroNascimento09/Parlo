import hashlib
import json
import os
import shutil
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from urllib.request import Request, urlopen

from .models import models_dir

INDEX_URL = "https://raw.githubusercontent.com/argosopentech/argospm-index/main/index.json"
USER_AGENT = "parlo (+https://github.com/PedroNascimento09/parlo)"
CHUNK = 1024 * 1024
MAX_BYTES = 512 * 1024 * 1024
REQUIRED_FILES = ("model/model.bin", "sentencepiece.model")


class PackError(RuntimeError):
    pass


def packs_dir():
    return models_dir() / "mt"


def fetch(url):
    return Request(url, headers={"User-Agent": USER_AGENT})


def index_entries(index_url=INDEX_URL, opener=urlopen, timeout=30):
    with opener(fetch(index_url), timeout=timeout) as response:
        data = json.load(response)
    if not isinstance(data, list):
        raise PackError(f"{index_url} is not a list of packages")
    return data


def choose(entries, source, target):
    for entry in entries:
        if entry.get("from_code") == source and entry.get("to_code") == target:
            links = entry.get("links") or []
            if not links:
                raise PackError(f"the index lists {source}->{target} without a download link")
            return entry, links[0]
    raise PackError(f"the index has no package for {source}->{target}")


def folder_name(source, target, version):
    return f"translate-{source}_{target}-{str(version).replace('.', '_')}"


def download(url, destination, opener=urlopen, timeout=60):
    digest = hashlib.sha256()
    written = 0
    destination.parent.mkdir(parents=True, exist_ok=True)
    with opener(fetch(url), timeout=timeout) as response, destination.open("wb") as sink:
        while True:
            block = response.read(CHUNK)
            if not block:
                break
            written += len(block)
            if written > MAX_BYTES:
                raise PackError(f"{url} passes {MAX_BYTES} bytes, stopped without installing")
            digest.update(block)
            sink.write(block)
    if written == 0:
        raise PackError(f"{url} answered with no bytes")
    return digest.hexdigest(), written


def inner_prefix(members):
    names = [member.filename for member in members if member.filename.strip("/")]
    if not names:
        return ""
    first = names[0].split("/", 1)[0] + "/"
    return first if all(name.startswith(first) for name in names) else ""


def extract(archive, destination):
    root = destination.resolve()
    with zipfile.ZipFile(archive) as bundle:
        members = bundle.infolist()
        prefix = inner_prefix(members)
        for member in members:
            if member.is_dir():
                continue
            relative = member.filename[len(prefix) :] if prefix else member.filename
            if not relative:
                continue
            target = (destination / relative).resolve()
            if target != root and root not in target.parents:
                raise PackError(f"the archive contains {member.filename}, which escapes the folder")
            target.parent.mkdir(parents=True, exist_ok=True)
            with bundle.open(member) as source, target.open("wb") as sink:
                shutil.copyfileobj(source, sink)
    missing = [name for name in REQUIRED_FILES if not (destination / name).is_file()]
    if missing:
        raise PackError(f"the archive does not carry {missing}")


def install(source, target, root=None, index_url=INDEX_URL, opener=urlopen):
    root = Path(root) if root else packs_dir()
    entry, url = choose(index_entries(index_url, opener), source, target)
    name = folder_name(source, target, entry.get("package_version") or "0")
    final = root / name
    if (final / REQUIRED_FILES[0]).is_file():
        return final
    root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=root) as staging:
        archive = Path(staging) / "package.argosmodel"
        digest, size = download(url, archive, opener)
        unpacked = Path(staging) / "unpacked"
        unpacked.mkdir()
        extract(archive, unpacked)
        record = {
            "id": f"opus-mt-{source}-{target}",
            "version": entry.get("package_version"),
            "source": url,
            "index": index_url,
            "sha256": digest,
            "bytes": size,
            "packages": [entry.get("from_name"), entry.get("to_name")],
            "fetched": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        }
        (unpacked / "parlo.json").write_text(
            json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        if final.exists():
            os.rmdir(final)
        unpacked.replace(final)
    return final
