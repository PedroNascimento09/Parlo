import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from pimo import compat, hardware, manifest, signing


def _parser():
    parser = argparse.ArgumentParser(prog="pimo", description="Parlo model orchestration tooling")
    commands = parser.add_subparsers(dest="command", required=True)

    device = commands.add_parser("profile", help="measure the local device")
    device.add_argument("--root", default=None)

    validate = commands.add_parser("validate", help="validate every manifest in a catalogue")
    validate.add_argument("path")

    admit = commands.add_parser("admit", help="list the models admitted for this device")
    admit.add_argument("path")
    admit.add_argument("--task", default=None)
    admit.add_argument("--locale", default=None)
    admit.add_argument("--schema", type=int, default=1)

    keygen = commands.add_parser("keygen", help="create an Ed25519 key pair")
    keygen.add_argument("--key", required=True)

    sign = commands.add_parser("sign", help="sign a catalogue file")
    sign.add_argument("path")
    sign.add_argument("--key", required=True)

    verify = commands.add_parser("verify", help="verify a signed catalogue file")
    verify.add_argument("path")
    verify.add_argument("--public", required=True)
    return parser


def _load(path):
    if path.is_dir():
        models = []
        for file in sorted(path.glob("*.json")):
            models.extend(manifest.entries(manifest.load(file)))
        return models
    return manifest.entries(manifest.load(path))


def _dispatch(arguments):
    if arguments.command == "profile":
        print(json.dumps(hardware.profile(arguments.root), indent=2, ensure_ascii=False))
        return 0

    if arguments.command == "validate":
        models = _load(Path(arguments.path))
        problems = []
        for entry in models:
            for problem in manifest.validate(entry):
                problems.append({"id": entry.get("id"), "problem": problem})
        print(json.dumps({"entries": len(models), "problems": problems}, indent=2, ensure_ascii=False))
        return 1 if problems else 0

    if arguments.command == "admit":
        models = _load(Path(arguments.path))
        device = hardware.profile()
        selection = compat.rank(
            models,
            device,
            task=arguments.task,
            locale=arguments.locale,
            schema=arguments.schema,
        )
        print(json.dumps({"device": device, "selection": selection}, indent=2, ensure_ascii=False))
        return 0

    if arguments.command == "keygen":
        public = signing.generate(arguments.key)
        print(json.dumps({"private_key": arguments.key, "public_key": public}, ensure_ascii=False))
        return 0

    if arguments.command == "sign":
        target = signing.sign_file(arguments.path, arguments.key)
        print(json.dumps({"signature": str(target)}, ensure_ascii=False))
        return 0

    if arguments.command == "verify":
        verified = signing.verify_file(arguments.path, arguments.public)
        print(json.dumps({"verified": verified}, ensure_ascii=False))
        return 0 if verified else 1

    return 2


def main(argv=None):
    arguments = _parser().parse_args(argv)
    try:
        return _dispatch(arguments)
    except (manifest.ManifestError, signing.SigningError, hardware.ProfileError, OSError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
