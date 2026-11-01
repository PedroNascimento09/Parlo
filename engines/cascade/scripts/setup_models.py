import argparse
import hashlib
import json
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from parlo_cascade.models import models_dir

MODELS = models_dir()
API = "https://huggingface.co/api/models"
CHUNK = 1024 * 1024
PARAKEET_REPO = "csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8"
VOICE_REPO = "rhasspy/piper-voices"
VOICES = {
    "pt_BR-faber-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx",
        "pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx",
    ),
    "en_US-lessac-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx",
        "en/en_US/lessac/medium/en_US-lessac-medium.onnx",
    ),
    "pt_PT-tugão-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_PT/tug%C3%A3o/medium/pt_PT-tug%C3%A3o-medium.onnx",
        "pt/pt_PT/tugão/medium/pt_PT-tugão-medium.onnx",
    ),
}
DEFAULT_VOICES = ("pt_BR-faber-medium", "en_US-lessac-medium")
OPTIONAL_VOICES = {"pt_pt": ("pt_PT-tugão-medium",)}
DIRECTIONS = (("pt", "en"), ("en", "pt"))
PARAKEET_BASE = f"https://huggingface.co/{PARAKEET_REPO}/resolve/main"
PARAKEET_FILES = ("encoder.int8.onnx", "decoder.int8.onnx", "joiner.int8.onnx", "tokens.txt")


def metadata(repo: str, path: str) -> tuple[str | None, int]:
    directory = path.rsplit("/", 1)[0] if "/" in path else ""
    url = f"{API}/{repo}/tree/main/{directory}" if directory else f"{API}/{repo}/tree/main"
    with urllib.request.urlopen(url, timeout=30) as response:
        entries = json.load(response)
    for entry in entries:
        if entry.get("type") != "file" or entry.get("path") != path:
            continue
        lfs = entry.get("lfs") or {}
        digest = lfs.get("oid")
        size = int(lfs.get("size") or entry.get("size") or 0)
        return (digest, size)
    raise LookupError(f"{repo} does not publish {path}")


def expected(repo: str, path: str, target: Path) -> tuple[str | None, int | None]:
    try:
        digest, size = metadata(repo, path)
    except Exception as error:
        print(f"warning   cannot read the published digest of {target.name}: {error}", file=sys.stderr)
        return (None, None)
    return (digest, size)


def download(url: str, target: Path, sha256: str | None = None, size: int | None = None) -> None:
    if target.exists() and target.stat().st_size > 0:
        if size is None or target.stat().st_size == size:
            print(f"present   {target.name}")
            return
        print(f"repairing {target.name}, {target.stat().st_size} bytes is not the published {size}")
    print(f"download  {target.name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    part = target.with_suffix(target.suffix + ".part")
    digest = hashlib.sha256()
    written = 0
    try:
        with urllib.request.urlopen(url, timeout=60) as response, part.open("wb") as sink:
            while True:
                block = response.read(CHUNK)
                if not block:
                    break
                digest.update(block)
                sink.write(block)
                written += len(block)
        if size is not None and written != size:
            raise SystemExit(
                f"{target.name} arrived with {written} bytes, the server published {size}; "
                "nothing was installed, run again"
            )
        if sha256 is not None and digest.hexdigest() != sha256:
            raise SystemExit(
                f"{target.name} does not match the digest the server published; "
                "nothing was installed, run again"
            )
        part.replace(target)
    except BaseException:
        part.unlink(missing_ok=True)
        raise


def install_voices(names=None) -> None:
    for name in names or DEFAULT_VOICES:
        url, path = VOICES[name]
        files = (
            (path, f"{name}.onnx", url),
            (path + ".json", f"{name}.onnx.json", url + ".json"),
        )
        for remote, local, source in files:
            target = MODELS / local
            digest, size = expected(VOICE_REPO, remote, target)
            download(source, target, digest, size)


def install_parakeet() -> None:
    for name in PARAKEET_FILES:
        target = MODELS / "parakeet" / name
        digest, size = expected(PARAKEET_REPO, name, target)
        download(f"{PARAKEET_BASE}/{name}", target, digest, size)


def install_pack(source: str, target: str) -> None:
    from parlo_cascade import packs

    try:
        installed = packs.install(source, target)
    except packs.PackError as error:
        raise SystemExit(f"cannot install the {source}->{target} package: {error}")
    print(f"installed {source}->{target} in {installed}")


def warm_whisper(size: str) -> None:
    from faster_whisper import WhisperModel

    WhisperModel(size, device="cpu", compute_type="int8")
    print(f"whisper   {size} cached")


def warm_translation(source: str, target: str) -> None:
    from parlo_cascade import mt

    translator = mt.from_environment(source, target)
    if translator is None:
        raise SystemExit(f"no translation package installed for {source}->{target}")
    translator("Hello, this is a test.")
    print(f"cached    sentence model for {source}->{target}")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stt", choices=["parakeet", "whisper", "both"], default="parakeet")
    parser.add_argument("--voice", choices=["", "pt_pt"], default="")
    parser.add_argument(
        "--whisper",
        choices=["tiny", "base", "small", "medium", "large-v3"],
        default="base",
    )
    args = parser.parse_args(argv)

    install_voices(DEFAULT_VOICES + OPTIONAL_VOICES.get(args.voice, ()))
    if args.stt in ("parakeet", "both"):
        install_parakeet()
    if args.stt in ("whisper", "both"):
        warm_whisper(args.whisper)
    for source, target in DIRECTIONS:
        install_pack(source, target)
    for source, target in DIRECTIONS:
        warm_translation(source, target)
    print("models ready")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
