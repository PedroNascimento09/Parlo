import argparse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / "models"
VOICES = {
    "pt_BR-faber-medium": "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium/pt_BR-faber-medium.onnx",
}


def download(url: str, target: Path) -> None:
    if target.exists() and target.stat().st_size > 0:
        print(f"present   {target.name}")
        return
    print(f"download  {target.name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    urllib.request.urlretrieve(url, target)


def install_voice(name: str) -> None:
    url = VOICES[name]
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

    argostranslate.translate.translate("Ola, este e um teste.", source, target)
    print(f"cached    sentence model for {source}->{target}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", default="pt")
    parser.add_argument("--target", default="en")
    parser.add_argument("--whisper", default="small")
    parser.add_argument("--voice", default="pt_BR-faber-medium", choices=sorted(VOICES))
    args = parser.parse_args()

    install_voice(args.voice)
    install_argos(args.source, args.target)
    warm_whisper(args.whisper)
    warm_translation(args.source, args.target)
    print("models ready")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
