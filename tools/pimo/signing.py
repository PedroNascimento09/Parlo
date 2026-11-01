import base64
import binascii
import contextlib
import json
import os
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

DOMAIN = b"parlo-manifest-v1\n"
KEY_BYTES = 32


class SigningError(ValueError):
    pass


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def message(value):
    return DOMAIN + canonical(value)


def _decode(text):
    try:
        raw = base64.b64decode(text.strip(), validate=True)
    except (binascii.Error, ValueError) as error:
        raise SigningError("key is not valid base64") from error
    if len(raw) != KEY_BYTES:
        raise SigningError(f"key must be {KEY_BYTES} bytes")
    return raw


def generate(path):
    key = Ed25519PrivateKey.generate()
    target = Path(path)
    target.write_text(base64.b64encode(key.private_bytes_raw()).decode() + "\n", encoding="utf-8")
    with contextlib.suppress(OSError):
        os.chmod(target, 0o600)
    return base64.b64encode(key.public_key().public_bytes_raw()).decode()


def load_private(path):
    return Ed25519PrivateKey.from_private_bytes(_decode(Path(path).read_text(encoding="utf-8")))


def load_public(text):
    return Ed25519PublicKey.from_public_bytes(_decode(text))


def sign(value, private_path):
    signature = load_private(private_path).sign(message(value))
    return base64.b64encode(signature).decode()


def verify(value, signature, public_text):
    try:
        load_public(public_text).verify(base64.b64decode(signature.strip(), validate=True), message(value))
    except (InvalidSignature, binascii.Error, ValueError):
        return False
    return True


def signature_path(manifest_path):
    return Path(str(manifest_path) + ".sig")


def sign_file(manifest_path, private_path):
    document = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
    target = signature_path(manifest_path)
    target.write_text(sign(document, private_path) + "\n", encoding="utf-8")
    return target


def verify_file(manifest_path, public_text, signature_path_override=None):
    target = Path(signature_path_override) if signature_path_override else signature_path(manifest_path)
    signature = target.read_text(encoding="utf-8")
    document = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
    return verify(document, signature, public_text)
