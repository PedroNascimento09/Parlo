import argparse
import ipaddress
import json
import socket
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

MAX_BYTES = 2 * 1024 * 1024 * 1024
PAGE_TYPES = {"application/json", "application/xml", "application/xhtml+xml"}


class Refused(Exception):
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
    opener = urllib.request.build_opener(NoRedirect)
    request = urllib.request.Request(url, headers={"User-Agent": "Parlo/0.2"})
    try:
        with opener.open(request, timeout=timeout) as response:
            content_type = response.headers.get_content_type()
            if content_type.startswith("text/") or content_type in PAGE_TYPES:
                raise Refused("the link is a web page, not an audio or video file")
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
