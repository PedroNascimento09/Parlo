import json
import sys

from . import credentials
from .intelligence import PROVIDERS, Intelligence, settings_from_config

SAMPLE = (
    "Quero marcar uma reunião para amanhã às três, quer dizer, às quatro, "
    "com o João e com a equipa de lisboa."
)


def _status(request: dict) -> dict:
    return {
        "store": credentials.available(),
        "providers": [
            {
                "name": provider.name,
                "label": provider.label,
                "models": list(provider.models),
                "key": credentials.present(provider.name),
            }
            for provider in PROVIDERS.values()
        ],
    }


def _store(request: dict) -> dict:
    provider = _provider(request)
    key = request.get("key")
    if not isinstance(key, str) or not key.strip():
        raise ValueError("a key is needed to store one")
    if len(key) > 512:
        raise ValueError("that key is longer than any provider issues")
    credentials.store(provider, key.strip())
    return {"stored": True}


def _delete(request: dict) -> dict:
    credentials.delete(_provider(request))
    return {"deleted": True}


def _test(request: dict) -> dict:
    editor = Intelligence(settings_from_config())
    edit = editor.edit(SAMPLE)
    return {
        "engine": edit.engine,
        "provider": edit.provider,
        "model": edit.model,
        "fallback": edit.fallback,
        "reason": edit.reason or editor.last_reason,
        "text": edit.text,
    }


def _provider(request: dict) -> str:
    provider = request.get("provider")
    if not isinstance(provider, str) or provider not in PROVIDERS:
        raise ValueError(f"unknown provider {provider}, expected one of {tuple(PROVIDERS)}")
    return provider


ACTIONS = {"status": _status, "store": _store, "delete": _delete, "test": _test}


def answer(request: dict) -> dict:
    action = request.get("action")
    handler = ACTIONS.get(action) if isinstance(action, str) else None
    if handler is None:
        return {"error": f"unknown action {action}"}
    try:
        return handler(request)
    except (
        credentials.CredentialUnavailable,
        ValueError,
        KeyError,
        OSError,
    ) as error:
        return {"error": str(error)}


def main() -> int:
    try:
        request = json.loads(sys.stdin.read() or "{}")
    except ValueError:
        request = {}
    reply = answer(request if isinstance(request, dict) else {})
    sys.stdout.write(json.dumps(reply) + "\n")
    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
