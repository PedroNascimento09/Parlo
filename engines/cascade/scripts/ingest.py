import argparse
import ipaddress
import json
import shutil
import socket
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

MAX_BYTES = 2 * 1024 * 1024 * 1024
PAGE_TYPES = {"application/json", "application/xml", "application/xhtml+xml"}
PRIVATE_MARKERS = (
    "private video",
    "sign in",
    "login required",
    "logged-in",
    "logged in",
    "account credentials",
    "only available for registered users",
    "members-only",
    "members only",
    "confirm your age",
    "age-restricted",
    "not a bot",
)
PAGE_MARKERS = (
    "unsupported url",
    "no video formats found",
    "not a valid url",
    "no media found",
    "does not look like",
)


class Refused(Exception):
    pass


class NotMedia(Refused):
    pass


class FetchFailed(Exception):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, new_url):
        raise Refused(f"the link redirects to {new_url}; paste the final address")


def check_url(url, allow_local=False):
    parts = urllib.parse.urlsplit(url)
    if parts.scheme not in ("http", "https"):
        raise Refused("only http and https links can be dubbed")
    if parts.username or parts.password:
        raise Refused("links with a username or password are refused")
    host = parts.hostname
    if not host:
        raise Refused("the link has no address")
    if not host.isascii():
        raise Refused("the link must use an ASCII address")
    if host == "localhost" or host.endswith(".localhost"):
        if not allow_local:
            raise Refused("the link points at this computer; paste a public address")
    try:
        port = parts.port or (443 if parts.scheme == "https" else 80)
    except ValueError:
        raise Refused("the link has an invalid port") from None
    try:
        resolved = socket.getaddrinfo(host, port, type=socket.SOCK_STREAM)
    except socket.gaierror as error:
        raise FetchFailed(f"the address could not be resolved: {error}") from None
    if not resolved:
        raise FetchFailed("the address could not be resolved")
    for entry in resolved:
        address = ipaddress.ip_address(entry[4][0].split("%")[0])
        if address.is_global:
            continue
        local = address.is_loopback or address.is_private or address.is_link_local
        if allow_local and local:
            continue
        raise Refused("the link points at a private address; paste a public address")
    return parts


def download(url, out, allow_local=False, max_bytes=MAX_BYTES, timeout=30):
    check_url(url, allow_local)
    try:
        fetch(url, out, max_bytes, timeout)
    except NotMedia:
        download_page(url, out, allow_local=allow_local, max_bytes=max_bytes)


def fetch(url, out, max_bytes, timeout):
    opener = urllib.request.build_opener(NoRedirect)
    request = urllib.request.Request(url, headers={"User-Agent": "Parlo/0.2"})
    try:
        with opener.open(request, timeout=timeout) as response:
            content_type = response.headers.get_content_type()
            if content_type.startswith("text/") or content_type in PAGE_TYPES:
                raise NotMedia("the link is a web page, not an audio or video file")
            length = response.headers.get("Content-Length")
            if length:
                try:
                    declared = int(length)
                except ValueError:
                    declared = None
                if declared is not None and declared > max_bytes:
                    raise Refused("the file is larger than 2 GB")
            written = 0
            with open(out, "wb") as file:
                while True:
                    chunk = response.read(65536)
                    if not chunk:
                        break
                    written += len(chunk)
                    if written > max_bytes:
                        raise Refused("the file is larger than 2 GB")
                    file.write(chunk)
            if written == 0:
                raise Refused("the link returned an empty file")
    except Refused:
        Path(out).unlink(missing_ok=True)
        raise
    except urllib.error.HTTPError as error:
        Path(out).unlink(missing_ok=True)
        if error.code == 404:
            raise FetchFailed("the link returned 404; check the address") from None
        raise FetchFailed(f"the link returned {error.code}") from None
    except urllib.error.URLError as error:
        Path(out).unlink(missing_ok=True)
        raise FetchFailed(f"the link could not be reached: {error.reason}") from None
    except OSError as error:
        Path(out).unlink(missing_ok=True)
        raise FetchFailed(f"the file could not be written: {error}") from None


def download_page(url, out, allow_local=False, max_bytes=MAX_BYTES):
    check_url(url, allow_local)
    try:
        import yt_dlp
    except ImportError:
        raise FetchFailed(
            "the link is a web page and yt-dlp is not installed; "
            "install the cascade requirements and try again"
        ) from None
    try:
        work = Path(tempfile.mkdtemp(prefix="parlo-ingest-", dir=Path(out).parent))
    except OSError as error:
        raise FetchFailed(f"the download directory could not be created: {error}") from None
    try:
        options = {
            "noplaylist": True,
            "format": "bv*+ba/b",
            "paths": {"home": str(work), "temp": str(work)},
            "outtmpl": {"default": "%(title).120B [%(id)s].%(ext)s"},
            "windowsfilenames": True,
            "quiet": True,
            "no_warnings": True,
            "noprogress": True,
            "socket_timeout": 15,
            "retries": 2,
            "fragment_retries": 2,
            "extractor_retries": 2,
            "max_filesize": max_bytes,
            "overwrites": True,
            "ignoreconfig": True,
        }
        try:
            with yt_dlp.YoutubeDL(options) as downloader:
                info = downloader.extract_info(url, download=True)
        except Exception as error:
            raise video_error(str(error)) from None
        path = downloaded_path(info, work)
        if path is None:
            raise FetchFailed("the video was not downloaded")
        try:
            size = path.stat().st_size
        except OSError as error:
            raise FetchFailed(f"the video could not be read: {error}") from None
        if size == 0:
            raise Refused("the link returned an empty file")
        if size > max_bytes:
            raise Refused("the file is larger than 2 GB")
        try:
            shutil.move(str(path), out)
        except OSError as error:
            raise FetchFailed(f"the file could not be written: {error}") from None
    finally:
        shutil.rmtree(work, ignore_errors=True)


def downloaded_path(info, work):
    candidates = []
    if isinstance(info, dict):
        for item in info.get("requested_downloads") or []:
            if isinstance(item, dict):
                path = item.get("filepath")
                if isinstance(path, str):
                    candidates.append(Path(path))
        path = info.get("filepath")
        if isinstance(path, str):
            candidates.append(Path(path))
    root = work.resolve()
    for candidate in candidates:
        try:
            resolved = candidate.resolve()
        except OSError:
            continue
        if resolved.is_file() and resolved.is_relative_to(root):
            return resolved
    files = [
        path
        for path in work.rglob("*")
        if path.is_file() and path.suffix not in {".part", ".ytdl", ".temp"}
    ]
    return max(files, key=lambda path: path.stat().st_size) if files else None


def video_error(message):
    lower = message.lower()
    if any(marker in lower for marker in PRIVATE_MARKERS):
        return Refused(
            "the video is private or needs a sign-in; "
            "Parlo only downloads public videos"
        )
    if any(marker in lower for marker in PAGE_MARKERS):
        return Refused(
            "the link is a web page Parlo cannot read; "
            "paste a direct media file or a public video page"
        )
    return FetchFailed(f"the public video could not be downloaded: {short_error(message)}")


def short_error(message):
    text = " ".join(message.split())
    return text[:300] if text else "yt-dlp reported no reason"


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--allow-local", action="store_true")
    args = parser.parse_args(argv)
    try:
        download(args.url, args.out, allow_local=args.allow_local)
    except Refused as error:
        print(json.dumps({"error": str(error)}), file=sys.stderr)
        return 2
    except FetchFailed as error:
        print(json.dumps({"error": str(error)}), file=sys.stderr)
        return 3
    return 0


if __name__ == "__main__":
    sys.exit(main())
