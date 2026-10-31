import argparse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "models"
VOICES = {
    "pt_BR-faber-medium": "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx",
    "en_US-lessac-medium": "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx",
}
DIRECTIONS = (("pt", "en"), ("en", "pt"))


def download(url: str, target: Path) -> None:
    if target.exists() and target.stat().st_size > 0:
        print(f"present   {target.name}")
        return
    print(f"download  {target.name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(url, target)


def install_voices() -> None:
    for name, url in VOICES.items():
        download(url, MODELS / f"{name}.onnx")
        download(url + ".json", MODELS / f"{name}.onnx.json")


def install_argos(source: str, target: str) -> None:
    import argostranslate.package as package

    package.update_package_index()
    match = next(
        (
            candidate
            for candidate in package.get_available_packages()
            if candidate.from_code == source and candidate.to_code == target
        ),
        None,
    )
    if match is None:
        raise SystemExit(f"argos has no package for {source}->{target}")
    package.install_from_path(match.download())
    print(f"installed argos {source}->{target}")


def warm_whisper(size: str) -> None:
    from faster_whisper import WhisperModel

    WhisperModel(size, device="cpu", compute_type="int8")
    print(f"whisper   {size} cached")


def warm_translation(source: str, target: str) -> None:
    import truststore

    truststore.inject_into_ssl()
    import argostranslate.translate

    argostranslate.translate.translate("Hello, this is a test.", source, target)
    print(f"cached    sentence model for {source}->{target}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--whisper", default="small")
    args = parser.parse_args()

    install_voices()
    for source, target in DIRECTIONS:
        install_argos(source, target)
    warm_whisper(args.whisper)
    for source, target in DIRECTIONS:
        warm_translation(source, target)
    print("models ready")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
