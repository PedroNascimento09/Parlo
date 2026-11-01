import ctypes
import os
import shutil
import subprocess
import sys
from pathlib import Path

from .models import data_root

SERVICE = "parlo"
ENTROPY = b"parlo-credential"
UI_FORBIDDEN = 0x1


class CredentialUnavailable(RuntimeError):
    pass


def folder() -> Path:
    override = os.environ.get("PARLO_CREDENTIALS_DIR")
    if override:
        return Path(override)
    return data_root() / "credentials"


def available() -> bool:
    if sys.platform == "win32":
        return True
    if sys.platform == "darwin":
        return shutil.which("security") is not None
    return shutil.which("secret-tool") is not None


class _Blob(ctypes.Structure):
    _fields_ = [("size", ctypes.c_uint32), ("data", ctypes.POINTER(ctypes.c_char))]


def _blob(payload: bytes):
    buffer = ctypes.create_string_buffer(payload, len(payload))
    return _Blob(len(payload), ctypes.cast(buffer, ctypes.POINTER(ctypes.c_char))), buffer


def _crypt32():
    library = ctypes.windll.crypt32
    library.CryptProtectData.argtypes = [
        ctypes.POINTER(_Blob),
        ctypes.c_wchar_p,
        ctypes.POINTER(_Blob),
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.POINTER(_Blob),
    ]
    library.CryptProtectData.restype = ctypes.c_bool
    library.CryptUnprotectData.argtypes = [
        ctypes.POINTER(_Blob),
        ctypes.c_void_p,
        ctypes.POINTER(_Blob),
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.POINTER(_Blob),
    ]
    library.CryptUnprotectData.restype = ctypes.c_bool
    return library


def _protect(secret: bytes) -> bytes:
    library = _crypt32()
    kernel32 = ctypes.windll.kernel32
    kernel32.LocalFree.argtypes = [ctypes.c_void_p]
    data, keep_data = _blob(secret)
    entropy, keep_entropy = _blob(ENTROPY)
    protected = _Blob()
    if not library.CryptProtectData(
        ctypes.byref(data), None, ctypes.byref(entropy), None, None, UI_FORBIDDEN, ctypes.byref(protected)
    ):
        raise CredentialUnavailable(
            f"the system refused to protect the credential (error {ctypes.GetLastError()})"
        )
    try:
        return ctypes.string_at(protected.data, protected.size)
    finally:
        kernel32.LocalFree(protected.data)


def _unprotect(blob: bytes) -> bytes:
    library = _crypt32()
    kernel32 = ctypes.windll.kernel32
    kernel32.LocalFree.argtypes = [ctypes.c_void_p]
    data, keep_data = _blob(blob)
    entropy, keep_entropy = _blob(ENTROPY)
    clear = _Blob()
    if not library.CryptUnprotectData(
        ctypes.byref(data), None, ctypes.byref(entropy), None, None, UI_FORBIDDEN, ctypes.byref(clear)
    ):
        raise CredentialUnavailable(
            f"the stored credential cannot be read by this account (error {ctypes.GetLastError()})"
        )
    try:
        return ctypes.string_at(clear.data, clear.size)
    finally:
        kernel32.LocalFree(clear.data)


def _run(command: list[str], secret: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        command,
        input=secret.encode("utf-8") if secret is not None else None,
        capture_output=True,
        check=False,
    )


def _keychain_store(name: str, secret: str) -> None:
    result = _run(["security", "add-generic-password", "-U", "-s", SERVICE, "-a", name, "-w", secret])
    if result.returncode != 0:
        raise CredentialUnavailable(_reason(result.stderr, "the keychain refused the credential"))


def _keychain_load(name: str) -> str:
    result = _run(["security", "find-generic-password", "-s", SERVICE, "-a", name, "-w"])
    if result.returncode != 0:
        raise CredentialUnavailable(_reason(result.stderr, "the keychain has no credential"))
    return result.stdout.decode("utf-8").strip()


def _keychain_delete(name: str) -> None:
    _run(["security", "delete-generic-password", "-s", SERVICE, "-a", name])


def _secret_store(name: str, secret: str) -> None:
    result = _run(["secret-tool", "store", "--label", f"Parlo {name}", "service", SERVICE, "account", name], secret)
    if result.returncode != 0:
        raise CredentialUnavailable(_reason(result.stderr, "the secret service refused the credential"))


def _secret_load(name: str) -> str:
    result = _run(["secret-tool", "lookup", "service", SERVICE, "account", name])
    if result.returncode != 0:
        raise CredentialUnavailable(_reason(result.stderr, "the secret service has no credential"))
    return result.stdout.decode("utf-8").strip()


def _secret_delete(name: str) -> None:
    _run(["secret-tool", "clear", "service", SERVICE, "account", name])


def _reason(output: bytes, fallback: str) -> str:
    text = output.decode("utf-8", "replace").strip()
    return text.splitlines()[-1] if text else fallback


def _windows_file(name: str) -> Path:
    return folder() / f"{name}.bin"


def store(name: str, secret: str) -> None:
    if not name or not secret.strip():
        raise ValueError("a credential needs a provider and a value")
    if sys.platform == "win32":
        target = folder()
        target.mkdir(parents=True, exist_ok=True)
        path = _windows_file(name)
        path.write_bytes(_protect(secret.encode("utf-8")))
        return
    if not available():
        raise CredentialUnavailable(
            "this system has no credential store Parlo can use; the key was not written to disk"
        )
    if sys.platform == "darwin":
        _keychain_store(name, secret)
    else:
        _secret_store(name, secret)


def load(name: str) -> str:
    if sys.platform == "win32":
        path = _windows_file(name)
        if not path.exists():
            raise CredentialUnavailable(f"no credential stored for {name}")
        return _unprotect(path.read_bytes()).decode("utf-8")
    if not available():
        raise CredentialUnavailable(
            "this system has no credential store Parlo can use"
        )
    if sys.platform == "darwin":
        return _keychain_load(name)
    return _secret_load(name)


def present(name: str) -> bool:
    try:
        return bool(load(name).strip())
    except (CredentialUnavailable, OSError):
        return False


def delete(name: str) -> None:
    if sys.platform == "win32":
        _windows_file(name).unlink(missing_ok=True)
        return
    if sys.platform == "darwin":
        _keychain_delete(name)
    else:
        _secret_delete(name)
