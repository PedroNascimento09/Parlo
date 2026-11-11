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
    "de_DE-thorsten-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/de/de_DE/thorsten/medium/de_DE-thorsten-medium.onnx",
        "de/de_DE/thorsten/medium/de_DE-thorsten-medium.onnx",
    ),
    "fr_FR-siwis-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/fr/fr_FR/siwis/medium/fr_FR-siwis-medium.onnx",
        "fr/fr_FR/siwis/medium/fr_FR-siwis-medium.onnx",
    ),
    "es_ES-davefx-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_ES/davefx/medium/es_ES-davefx-medium.onnx",
        "es/es_ES/davefx/medium/es_ES-davefx-medium.onnx",
    ),
    "it_IT-paola-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/it/it_IT/paola/medium/it_IT-paola-medium.onnx",
        "it/it_IT/paola/medium/it_IT-paola-medium.onnx",
    ),
    "nl_NL-mls-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/nl/nl_NL/mls/medium/nl_NL-mls-medium.onnx",
        "nl/nl_NL/mls/medium/nl_NL-mls-medium.onnx",
    ),
    "ru_RU-irina-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/ru/ru_RU/irina/medium/ru_RU-irina-medium.onnx",
        "ru/ru_RU/irina/medium/ru_RU-irina-medium.onnx",
    ),
    "pl_PL-gosia-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/pl/pl_PL/gosia/medium/pl_PL-gosia-medium.onnx",
        "pl/pl_PL/gosia/medium/pl_PL-gosia-medium.onnx",
    ),
    "sv_SE-lisa-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/sv/sv_SE/lisa/medium/sv_SE-lisa-medium.onnx",
        "sv/sv_SE/lisa/medium/sv_SE-lisa-medium.onnx",
    ),
    "cs_CZ-jirka-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/cs/cs_CZ/jirka/medium/cs_CZ-jirka-medium.onnx",
        "cs/cs_CZ/jirka/medium/cs_CZ-jirka-medium.onnx",
    ),
    "da_DK-talesyntese-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/da/da_DK/talesyntese/medium/da_DK-talesyntese-medium.onnx",
        "da/da_DK/talesyntese/medium/da_DK-talesyntese-medium.onnx",
    ),
    "fi_FI-harri-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/fi/fi_FI/harri/medium/fi_FI-harri-medium.onnx",
        "fi/fi_FI/harri/medium/fi_FI-harri-medium.onnx",
    ),
    "el_GR-joy-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/el/el_GR/joy/medium/el_GR-joy-medium.onnx",
        "el/el_GR/joy/medium/el_GR-joy-medium.onnx",
    ),
    "ro_RO-mihai-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/ro/ro_RO/mihai/medium/ro_RO-mihai-medium.onnx",
        "ro/ro_RO/mihai/medium/ro_RO-mihai-medium.onnx",
    ),
    "hu_HU-anna-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/hu/hu_HU/anna/medium/hu_HU-anna-medium.onnx",
        "hu/hu_HU/anna/medium/hu_HU-anna-medium.onnx",
    ),
    "tr_TR-dfki-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/tr/tr_TR/dfki/medium/tr_TR-dfki-medium.onnx",
        "tr/tr_TR/dfki/medium/tr_TR-dfki-medium.onnx",
    ),
    "uk_UA-tetiana-high": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/uk/uk_UA/tetiana/high/uk_UA-tetiana-high.onnx",
        "uk/uk_UA/tetiana/high/uk_UA-tetiana-high.onnx",
    ),
    "ca_ES-upc_ona-medium": (
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/ca/ca_ES/upc_ona/medium/ca_ES-upc_ona-medium.onnx",
        "ca/ca_ES/upc_ona/medium/ca_ES-upc_ona-medium.onnx",
    ),
}

DEFAULT_VOICES = ("pt_PT-tugão-medium", "en_US-lessac-medium")
OPTIONAL_VOICES = {"pt_br": ("pt_BR-faber-medium",)}
PAIR_UNITS = (
    ("mt-pt-en", "pt", "en"),
    ("mt-en-pt", "en", "pt"),
    ("mt-pt-es", "pt", "es"),
    ("mt-es-pt", "es", "pt"),
    ("mt-pt-fr", "pt", "fr"),
    ("mt-fr-pt", "fr", "pt"),
    ("mt-pt-it", "pt", "it"),
    ("mt-pt-nl", "pt", "nl"),
    ("mt-pt-ru", "pt", "ru"),
    ("mt-en-es", "en", "es"),
    ("mt-en-fr", "en", "fr"),
    ("mt-en-it", "en", "it"),
    ("mt-de-en", "de", "en"),
    ("mt-fr-de", "fr", "de"),
    ("mt-es-en", "es", "en"),
    ("mt-en-pl", "en", "pl"),
    ("mt-en-sv", "en", "sv"),
    ("mt-en-cs", "en", "cs"),
    ("mt-en-da", "en", "da"),
    ("mt-en-fi", "en", "fi"),
    ("mt-en-el", "en", "el"),
    ("mt-en-ro", "en", "ro"),
    ("mt-en-hu", "en", "hu"),
    ("mt-en-tr", "en", "tr"),
    ("mt-en-uk", "en", "uk"),
    ("mt-en-ca", "en", "ca"),
    ("mt-pl-en", "pl", "en"),
    ("mt-sv-en", "sv", "en"),
    ("mt-cs-en", "cs", "en"),
    ("mt-da-en", "da", "en"),
    ("mt-fi-en", "fi", "en"),
    ("mt-el-en", "el", "en"),
    ("mt-ro-en", "ro", "en"),
    ("mt-hu-en", "hu", "en"),
    ("mt-tr-en", "tr", "en"),
    ("mt-uk-en", "uk", "en"),
    ("mt-ca-en", "ca", "en"),
    ("mt-it-en", "it", "en"),
    ("mt-nl-en", "nl", "en"),
    ("mt-ru-en", "ru", "en"),
)

DIRECTIONS = (
    ("pt", "en"),
    ("en", "pt"),
    ("pt", "es"),
    ("es", "pt"),
    ("pt", "fr"),
    ("fr", "pt"),
    ("pt", "it"),
    ("pt", "nl"),
    ("pt", "ru"),
    ("en", "es"),
    ("en", "fr"),
    ("en", "it"),
    ("de", "en"),
    ("fr", "de"),
    ("es", "en"),
    ("en", "pl"),
    ("en", "sv"),
    ("en", "cs"),
    ("en", "da"),
    ("en", "fi"),
    ("en", "el"),
    ("en", "ro"),
    ("en", "hu"),
    ("en", "tr"),
    ("en", "uk"),
    ("en", "ca"),
    ("pl", "en"),
    ("sv", "en"),
    ("cs", "en"),
    ("da", "en"),
    ("fi", "en"),
    ("el", "en"),
    ("ro", "en"),
    ("hu", "en"),
    ("tr", "en"),
    ("uk", "en"),
    ("ca", "en"),
    ("it", "en"),
    ("nl", "en"),
    ("ru", "en"),
)
PARAKEET_BASE = f"https://huggingface.co/{PARAKEET_REPO}/resolve/main"
PARAKEET_FILES = ("encoder.int8.onnx", "decoder.int8.onnx", "joiner.int8.onnx", "tokens.txt")
SPEAKER_FILE = "3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx"
SPEAKER_URL = (
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/"
    "speaker-recongition-models/" + SPEAKER_FILE
)


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
    target.parent.mkdir(parents=True, exist_ok=True)
    part = target.with_suffix(target.suffix + ".part")
    held = part.stat().st_size if part.exists() else 0
    if size is not None and held > size:
        part.unlink()
        held = 0
    if held and size is not None and held == size:
        print(f"checking  {target.name}, every byte is already here")
        verify(part, target, held, sha256, size)
        return
    print(f"download  {target.name}" + (f", resuming at {held} bytes" if held else ""))
    digest = hashlib.sha256()
    if held:
        with part.open("rb") as existing:
            for block in iter(lambda: existing.read(CHUNK), b""):
                digest.update(block)
    headers = {"Range": f"bytes={held}-"} if held else {}
    written = held
    try:
        request = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(request, timeout=60) as response:
            resumed = held > 0 and getattr(response, "status", None) == 206
            if held and not resumed:
                written = 0
                digest = hashlib.sha256()
            with part.open("ab" if resumed else "wb") as sink:
                while True:
                    block = response.read(CHUNK)
                    if not block:
                        break
                    digest.update(block)
                    sink.write(block)
                    written += len(block)
        if size is not None and written != size:
            part.unlink(missing_ok=True)
            raise SystemExit(
                f"{target.name} arrived with {written} bytes, the server published {size}; "
                "nothing was installed, run again"
            )
        verify(part, target, written, sha256, size)
    except SystemExit:
        raise
    except BaseException:
        if written:
            print(f"kept      {written} bytes in {part.name}, run again to resume")
        raise


def verify(part: Path, target: Path, written: int, sha256: str | None, size: int | None) -> None:
    if sha256 is not None:
        digest = hashlib.sha256()
        with part.open("rb") as held:
            for block in iter(lambda: held.read(CHUNK), b""):
                digest.update(block)
        if digest.hexdigest() != sha256:
            part.unlink(missing_ok=True)
            raise SystemExit(
                f"{target.name} does not match the digest the server published; "
                "nothing was installed, run again"
            )
    if size is not None and written != size:
        raise SystemExit(
            f"{target.name} arrived with {written} bytes, the server published {size}; "
            "nothing was installed, run again"
        )
    part.replace(target)


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


def install_speakers() -> None:
    download(SPEAKER_URL, MODELS / SPEAKER_FILE)


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


UNITS = (
    "parakeet",
    "voice-pt",
    "voice-en",
    "voice-de",
    "voice-fr",
    "voice-es",
    "voice-it",
    "voice-nl",
    "voice-ru",
    "voice-pl",
    "voice-sv",
    "voice-cs",
    "voice-da",
    "voice-fi",
    "voice-el",
    "voice-ro",
    "voice-hu",
    "voice-tr",
    "voice-uk",
    "voice-ca",
    "mt-pt-en",
    "mt-en-pt",
    "mt-pt-es",
    "mt-es-pt",
    "mt-pt-fr",
    "mt-fr-pt",
    "mt-pt-it",
    "mt-pt-nl",
    "mt-pt-ru",
    "mt-en-es",
    "mt-en-fr",
    "mt-en-it",
    "mt-de-en",
    "mt-fr-de",
    "mt-es-en",
    "mt-en-pl",
    "mt-en-sv",
    "mt-en-cs",
    "mt-en-da",
    "mt-en-fi",
    "mt-en-el",
    "mt-en-ro",
    "mt-en-hu",
    "mt-en-tr",
    "mt-en-uk",
    "mt-en-ca",
    "mt-pl-en",
    "mt-sv-en",
    "mt-cs-en",
    "mt-da-en",
    "mt-fi-en",
    "mt-el-en",
    "mt-ro-en",
    "mt-hu-en",
    "mt-tr-en",
    "mt-uk-en",
    "mt-ca-en",
    "mt-it-en",
    "mt-nl-en",
    "mt-ru-en",
    "speakers",
    "whisper",
)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stt", choices=["parakeet", "whisper", "both"], default="parakeet")
    parser.add_argument("--voice", choices=["", "pt_br"], default="")
    parser.add_argument(
        "--whisper",
        choices=["tiny", "base", "small", "medium", "large-v3"],
        default="base",
    )
    parser.add_argument("--only", nargs="+", choices=list(UNITS), default=None)
    args = parser.parse_args(argv)

    only = set(args.only) if args.only else None

    def wanted(unit: str) -> bool:
        return only is None or unit in only

    voices = []
    if only is None:
        voices = list(DEFAULT_VOICES) + list(OPTIONAL_VOICES.get(args.voice, ()))
    else:
        if "voice-pt" in only:
            voices.append("pt_PT-tugão-medium")
        if "voice-en" in only:
            voices.append("en_US-lessac-medium")
        if "voice-de" in only:
            voices.append("de_DE-thorsten-medium")
        if "voice-fr" in only:
            voices.append("fr_FR-siwis-medium")
        if "voice-es" in only:
            voices.append("es_ES-davefx-medium")
        if "voice-it" in only:
            voices.append("it_IT-paola-medium")
        if "voice-nl" in only:
            voices.append("nl_NL-mls-medium")
        if "voice-ru" in only:
            voices.append("ru_RU-irina-medium")
        if "voice-pl" in only:
            voices.append("pl_PL-gosia-medium")
        if "voice-sv" in only:
            voices.append("sv_SE-lisa-medium")
        if "voice-cs" in only:
            voices.append("cs_CZ-jirka-medium")
        if "voice-da" in only:
            voices.append("da_DK-talesyntese-medium")
        if "voice-fi" in only:
            voices.append("fi_FI-harri-medium")
        if "voice-el" in only:
            voices.append("el_GR-joy-medium")
        if "voice-ro" in only:
            voices.append("ro_RO-mihai-medium")
        if "voice-hu" in only:
            voices.append("hu_HU-anna-medium")
        if "voice-tr" in only:
            voices.append("tr_TR-dfki-medium")
        if "voice-uk" in only:
            voices.append("uk_UA-tetiana-high")
        if "voice-ca" in only:
            voices.append("ca_ES-upc_ona-medium")
    install_voices(voices)

    if only is None:
        install_parakeet() if args.stt in ("parakeet", "both") else None
        warm_whisper(args.whisper) if args.stt in ("whisper", "both") else None
    elif "parakeet" in only:
        install_parakeet()
    elif "whisper" in only:
        warm_whisper(args.whisper)

    if only and "speakers" in only:
        install_speakers()

    directions = []
    if only is None:
        directions = list(DIRECTIONS)
    else:
        for unit, source, target in PAIR_UNITS:
            if unit in only:
                directions.append((source, target))
    for source, target in directions:
        install_pack(source, target)
    for source, target in directions:
        warm_translation(source, target)

    if wanted("parakeet"):
        print("parakeet is in place")
    print("models ready")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
