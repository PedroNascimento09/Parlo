import json
import os
import re
import time
import urllib.error
import urllib.request
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from . import credentials, vocabulary
from .corrections import CLEAN_MODES, clean

MODES = ("off", "auto", "local", "cloud")
DETERMINISTIC = "deterministic"
LOCAL = "local"
CLOUD = "cloud"
LOOPBACK_HOSTS = ("localhost", "127.0.0.1", "::1")
LENGTH_FLOOR = 0.4
LENGTH_CEILING = 1.6
LENGTH_SLACK = 20
DEADLINE = 1.5
DEADLINE_FLOOR = 0.2
DEADLINE_CEILING = 5.0

SYSTEM_PROMPT = (
    "You edit dictated text. You are an editor, not an author.\n"
    "Keep every fact, name, number, date and place exactly as written.\n"
    "Fix punctuation, spacing and capitalisation, and remove hesitations and accidental repetitions.\n"
    "Apply a self-correction only when the speaker marked it explicitly.\n"
    "Never add information, never translate, never summarise, never explain.\n"
    "When a change is not certain, keep the original words.\n"
    "Keep European Portuguese spelling when the text is Portuguese.\n"
    'Answer with JSON only: {"text": "the edited sentence"}'
)

NUMBER = re.compile(r"\d+(?:[.,]\d+)*")
WORD = re.compile(r"[\wÀ-ÿ'-]+")
SENTENCE = re.compile(r"[^.!?…]+[.!?…]*")


class IntelligenceError(RuntimeError):
    pass


@dataclass(frozen=True)
class Provider:
    name: str
    label: str
    kind: str
    endpoint: str
    models: tuple[str, ...]
    key_env: str


PROVIDERS = {
    "openai": Provider(
        name="openai",
        label="OpenAI",
        kind="openai",
        endpoint="https://api.openai.com/v1/chat/completions",
        models=("gpt-4o-mini", "gpt-4o"),
        key_env="PARLO_OPENAI_KEY",
    ),
    "anthropic": Provider(
        name="anthropic",
        label="Anthropic",
        kind="anthropic",
        endpoint="https://api.anthropic.com/v1/messages",
        models=("claude-3-5-haiku-latest", "claude-3-5-sonnet-latest"),
        key_env="PARLO_ANTHROPIC_KEY",
    ),
    "gemini": Provider(
        name="gemini",
        label="Google Gemini",
        kind="gemini",
        endpoint="https://generativelanguage.googleapis.com/v1beta/models",
        models=("gemini-2.0-flash", "gemini-1.5-flash"),
        key_env="PARLO_GEMINI_KEY",
    ),
    "compatible": Provider(
        name="compatible",
        label="OpenAI compatible",
        kind="openai",
        endpoint="",
        models=(),
        key_env="PARLO_COMPATIBLE_KEY",
    ),
}


@dataclass(frozen=True)
class Settings:
    mode: str = "auto"
    clean: str = "light"
    allow_cloud: bool = False
    provider: str = ""
    model: str = ""
    base_url: str = ""
    local_url: str = ""
    local_model: str = ""
    timeout: float = 20.0
    deadline: float = DEADLINE

    def __post_init__(self):
        if self.mode not in MODES:
            raise ValueError(f"unknown intelligence mode {self.mode}, expected {MODES}")
        if self.clean not in CLEAN_MODES:
            raise ValueError(f"unknown cleanup mode {self.clean}, expected {CLEAN_MODES}")
        if self.provider and self.provider not in PROVIDERS:
            raise ValueError(f"unknown provider {self.provider}, expected {tuple(PROVIDERS)}")
        if self.deadline <= 0:
            raise ValueError("the deadline must be positive")


@dataclass(frozen=True)
class Route:
    engine: str
    provider: str = ""
    model: str = ""
    reason: str = ""


@dataclass(frozen=True)
class Verdict:
    ok: bool
    reason: str = ""


@dataclass(frozen=True)
class Edit:
    text: str
    original: str
    engine: str
    provider: str = ""
    model: str = ""
    changes: tuple[str, ...] = ()
    latency_ms: int = 0
    fallback: bool = False
    reason: str = ""


def configured_deadline(stored: dict | None = None) -> float:
    raw = os.environ.get("PARLO_INTELLIGENCE_DEADLINE_MS", "").strip()
    if not raw:
        saved = (stored or {}).get("deadline_ms")
        raw = str(saved) if isinstance(saved, int) and not isinstance(saved, bool) else ""
    try:
        value = float(raw)
    except ValueError:
        return DEADLINE
    return min(max(value / 1000.0, DEADLINE_FLOOR), DEADLINE_CEILING)


def settings_from_environment(clean: str = "light") -> Settings:
    provider = os.environ.get("PARLO_INTELLIGENCE_PROVIDER", "").strip().lower()
    return Settings(
        mode=os.environ.get("PARLO_INTELLIGENCE_MODE", "auto").strip().lower() or "auto",
        clean=clean,
        allow_cloud=os.environ.get("PARLO_INTELLIGENCE_CLOUD") == "1",
        provider=provider,
        model=os.environ.get("PARLO_INTELLIGENCE_MODEL", "").strip(),
        base_url=os.environ.get("PARLO_INTELLIGENCE_BASE_URL", "").strip(),
        local_url=os.environ.get("PARLO_INTELLIGENCE_LOCAL_URL", "").strip(),
        local_model=os.environ.get("PARLO_INTELLIGENCE_LOCAL_MODEL", "").strip(),
        deadline=configured_deadline(),
    )


def config_path() -> Path:
    if configured := os.environ.get("PARLO_CONFIG"):
        return Path(configured)
    from .models import data_root

    return data_root() / "config.json"


def stored_settings() -> dict:
    try:
        payload = json.loads(config_path().read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    section = payload.get("intelligence") if isinstance(payload, dict) else None
    return section if isinstance(section, dict) else {}


def _value(stored: dict, key: str, variable: str, default: str) -> str:
    from_environment = os.environ.get(variable, "").strip()
    if from_environment:
        return from_environment
    saved = stored.get(key)
    return saved.strip() if isinstance(saved, str) and saved.strip() else default


def settings_from_config(default_clean: str = "light") -> Settings:
    stored = stored_settings()
    mode = _value(stored, "mode", "PARLO_INTELLIGENCE_MODE", "auto").lower()
    if mode not in MODES:
        mode = "auto"
    clean_mode = _value(stored, "clean", "PARLO_CASCADE_CLEAN", default_clean).lower()
    if clean_mode not in CLEAN_MODES:
        clean_mode = default_clean if default_clean in CLEAN_MODES else "light"
    provider = _value(stored, "provider", "PARLO_INTELLIGENCE_PROVIDER", "").lower()
    if provider not in PROVIDERS:
        provider = ""
    return Settings(
        mode=mode,
        clean=clean_mode,
        allow_cloud=stored.get("allow_cloud") is True
        or os.environ.get("PARLO_INTELLIGENCE_CLOUD") == "1",
        provider=provider,
        model=_value(stored, "model", "PARLO_INTELLIGENCE_MODEL", ""),
        base_url=_value(stored, "base_url", "PARLO_INTELLIGENCE_BASE_URL", ""),
        local_url=_value(stored, "local_url", "PARLO_INTELLIGENCE_LOCAL_URL", ""),
        local_model=_value(stored, "local_model", "PARLO_INTELLIGENCE_LOCAL_MODEL", ""),
        deadline=configured_deadline(stored),
    )


def is_loopback(url: str) -> bool:
    from urllib.parse import urlsplit

    host = (urlsplit(url).hostname or "").lower()
    return host in LOOPBACK_HOSTS


def requires_secure_transport(url: str) -> bool:
    if url.startswith("https://"):
        return False
    return not url.startswith("http://") or not is_loopback(url)


def choose(settings: Settings, key_present: bool) -> Route:
    if settings.clean == "original":
        return Route(DETERMINISTIC, reason="cleanup is off")
    if settings.mode == "off":
        return Route(DETERMINISTIC, reason="intelligence is off")
    if settings.local_url:
        return Route(LOCAL, model=settings.local_model or "local model", reason="a local model is configured")
    if settings.mode == "local":
        return Route(DETERMINISTIC, reason="no local model is configured")
    if settings.mode == "cloud" or settings.mode == "auto":
        if not settings.allow_cloud:
            return Route(DETERMINISTIC, reason="cloud processing is not allowed")
        if not settings.provider:
            return Route(DETERMINISTIC, reason="no cloud provider is configured")
        if settings.provider == "compatible" and not settings.base_url:
            return Route(DETERMINISTIC, reason="the compatible provider needs a base URL")
        if not key_present:
            return Route(DETERMINISTIC, reason=f"no API key stored for {settings.provider}")
        model = settings.model or PROVIDERS[settings.provider].models[0]
        return Route(CLOUD, provider=settings.provider, model=model, reason="cloud processing is allowed and configured")
    return Route(DETERMINISTIC, reason="no intelligent engine is available")


def validate(original: str, edited: str) -> Verdict:
    if not edited.strip():
        return Verdict(False, "the model returned no text")
    if not original.strip():
        return Verdict(False, "there was no text to edit")
    if len(edited) > len(original) * LENGTH_CEILING + LENGTH_SLACK:
        return Verdict(False, "the model wrote more than it was given")
    if len(edited) < len(original) * LENGTH_FLOOR:
        return Verdict(False, "the model dropped most of the sentence")
    if "You edit dictated text" in edited or "```" in edited:
        return Verdict(False, "the model answered with commentary")
    if not _numbers_are_kept(original, edited):
        return Verdict(False, "the model changed or dropped a number")
    if _new_names(original, edited):
        return Verdict(False, "the model introduced a name that was not said")
    return Verdict(True)


def _numbers_are_kept(original: str, edited: str) -> bool:
    return Counter(NUMBER.findall(original)) == Counter(NUMBER.findall(edited))


def _new_names(original: str, edited: str) -> bool:
    known = {word.lower() for word in WORD.findall(original)}
    for sentence in SENTENCE.findall(edited):
        words = WORD.findall(sentence)
        for position, word in enumerate(words):
            if position == 0 or not word[:1].isupper():
                continue
            if word.lower() not in known:
                return True
    return False


class Intelligence:
    def __init__(self, settings: Settings, language: str = "pt", transport=None, key_lookup=None):
        self.settings = settings
        self.language = language
        self._transport = transport or _post_json
        self._key_lookup = key_lookup or _stored_key
        self.last_reason = ""

    def route(self) -> Route:
        provider = self.settings.provider
        key_present = bool(provider) and bool(self._key_lookup(provider))
        return choose(self.settings, key_present)

    def edit(self, text: str) -> Edit:
        started = time.perf_counter()
        base = clean(text, self.settings.clean, self.language)
        edited, personal = vocabulary.apply(base.cleaned)
        route = self.route()
        self.last_reason = route.reason
        if route.engine == DETERMINISTIC:
            return Edit(
                text=edited,
                original=base.original,
                engine=DETERMINISTIC,
                changes=personal + base.changes,
                latency_ms=_elapsed(started),
                reason=route.reason,
            )
        try:
            candidate = self._ask(
                edited, route, min(self.settings.timeout, self.settings.deadline)
            )
        except (IntelligenceError, OSError, ValueError, KeyError, IndexError) as error:
            verdict = Verdict(False, str(error))
            return self._fallback(base, route, verdict, started, personal)
        verdict = validate(edited, candidate)
        if not verdict.ok:
            return self._fallback(base, route, verdict, started, personal)
        return Edit(
            text=candidate,
            original=base.original,
            engine=route.engine,
            provider=route.provider,
            model=route.model,
            changes=personal + base.changes + (f"{route.engine} edit: {route.model}",),
            latency_ms=_elapsed(started),
        )

    def _fallback(self, base, route: Route, verdict: Verdict, started: float, personal) -> Edit:
        self.last_reason = verdict.reason
        edited, _ = vocabulary.apply(base.cleaned)
        return Edit(
            text=edited,
            original=base.original,
            engine=DETERMINISTIC,
            provider=route.provider,
            model=route.model,
            changes=personal
            + base.changes
            + (f"kept the deterministic text: {verdict.reason}",),
            latency_ms=_elapsed(started),
            fallback=True,
            reason=verdict.reason,
        )

    def _ask(self, text: str, route: Route, timeout: float) -> str:
        if route.engine == LOCAL:
            url = self.settings.local_url
            if requires_secure_transport(url) or not is_loopback(url):
                raise IntelligenceError("a local model must be reached over loopback")
            payload, headers = _openai_request(url, self.settings.local_model or "local", text, None)
            reply = self._transport(url, headers, payload, timeout)
            return _openai_text(reply)
        provider = PROVIDERS[route.provider]
        key = self._key_lookup(route.provider)
        if not key:
            raise IntelligenceError(f"no API key stored for {route.provider}")
        if provider.kind == "openai":
            endpoint = provider.endpoint or url_from_settings(self.settings)
            payload, headers = _openai_request(endpoint, route.model, text, key)
        elif provider.kind == "anthropic":
            payload, headers = _anthropic_request(provider.endpoint, route.model, text, key)
        else:
            payload, headers = _gemini_request(provider.endpoint, route.model, text, key)
        reply = self._transport(endpoint_of(provider, self.settings), headers, payload, timeout)
        if provider.kind == "openai":
            return _openai_text(reply)
        if provider.kind == "anthropic":
            return _anthropic_text(reply)
        return _gemini_text(reply)


def url_from_settings(settings: Settings) -> str:
    base = settings.base_url.rstrip("/")
    if not base:
        raise IntelligenceError("the compatible provider needs a base URL")
    return f"{base}/chat/completions"


def endpoint_of(provider: Provider, settings: Settings) -> str:
    return provider.endpoint or url_from_settings(settings)


def _elapsed(started: float) -> int:
    return int((time.perf_counter() - started) * 1000)


def _stored_key(provider: str) -> str:
    from_env = os.environ.get(PROVIDERS[provider].key_env, "").strip()
    if from_env:
        return from_env
    try:
        return credentials.load(provider).strip()
    except (credentials.CredentialUnavailable, OSError):
        return ""


def _openai_request(url: str, model: str, text: str, key: str | None):
    headers = {"Content-Type": "application/json"}
    if key:
        headers["Authorization"] = f"Bearer {key}"
    payload = {
        "model": model,
        "temperature": 0,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": text},
        ],
    }
    return payload, headers


def _anthropic_request(url: str, model: str, text: str, key: str):
    headers = {"Content-Type": "application/json", "x-api-key": key, "anthropic-version": "2023-06-01"}
    payload = {
        "model": model,
        "max_tokens": 1024,
        "temperature": 0,
        "system": SYSTEM_PROMPT,
        "messages": [{"role": "user", "content": text}],
    }
    return payload, headers


def _gemini_request(url: str, model: str, text: str, key: str):
    headers = {"Content-Type": "application/json", "x-goog-api-key": key}
    payload = {
        "system_instruction": {"parts": [{"text": SYSTEM_PROMPT}]},
        "contents": [{"role": "user", "parts": [{"text": text}]}],
        "generationConfig": {"temperature": 0},
    }
    return payload, headers


def _openai_text(reply) -> str:
    return _json_text(reply["choices"][0]["message"]["content"])


def _anthropic_text(reply) -> str:
    return _json_text(reply["content"][0]["text"])


def _gemini_text(reply) -> str:
    return _json_text(reply["candidates"][0]["content"]["parts"][0]["text"])


def _json_text(answer: str) -> str:
    stripped = answer.strip()
    if stripped.startswith("```"):
        raise IntelligenceError("the model answered with a code fence")
    try:
        payload = json.loads(stripped)
    except json.JSONDecodeError as error:
        raise IntelligenceError("the model did not answer in the expected format") from error
    text = payload.get("text") if isinstance(payload, dict) else None
    if not isinstance(text, str) or not text.strip():
        raise IntelligenceError("the model did not answer with text")
    return text.strip()


def _post_json(url: str, headers: dict, payload: dict, timeout: float):
    if requires_secure_transport(url):
        raise IntelligenceError("refusing to send text over a plain connection")
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers=headers,
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        raise IntelligenceError(f"{error.code} {_status_reason(error.code)}") from error
    except urllib.error.URLError as error:
        raise IntelligenceError(f"the provider could not be reached: {error.reason}") from error
    except TimeoutError as error:
        raise IntelligenceError("the provider took too long") from error


def _status_reason(code: int) -> str:
    if code in (401, 403):
        return "the provider refused the API key"
    if code == 429:
        return "the provider is rate limiting this key"
    if code >= 500:
        return "the provider is failing"
    return "the provider rejected the request"
