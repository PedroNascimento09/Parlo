import json
import subprocess
import sys
import tempfile
import threading
import types
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parent.parent / "scripts"
sys.path.insert(0, str(SCRIPTS))

import ingest

CLIP = b"media-bytes" * 100


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/clip.mp4":
            self.send_response(200)
            self.send_header("Content-Type", "video/mp4")
            self.send_header("Content-Length", str(len(CLIP)))
            self.end_headers()
            self.wfile.write(CLIP)
        elif self.path == "/missing":
            self.send_response(404)
            self.end_headers()
        elif self.path == "/redirect":
            self.send_response(302)
            self.send_header("Location", "/clip.mp4")
            self.end_headers()
        elif self.path == "/page":
            body = b"<html>not media</html>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        elif self.path == "/empty":
            self.send_response(200)
            self.send_header("Content-Type", "video/mp4")
            self.send_header("Content-Length", "0")
            self.end_headers()
        elif self.path == "/big":
            self.send_response(200)
            self.send_header("Content-Type", "video/mp4")
            self.send_header("Content-Length", "100")
            self.end_headers()
            self.wfile.write(b"x" * 100)
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass


class IngestTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.port = cls.server.server_address[1]
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.out = Path(self.directory.name) / "clip.mp4"

    def tearDown(self):
        self.directory.cleanup()

    def url(self, path):
        return f"http://127.0.0.1:{self.port}{path}"

    def ytdlp(self, failure=None, seen=None):
        class YoutubeDL:
            def __init__(self, options):
                self.options = options
                if seen is not None:
                    seen.update(options)

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return False

            def extract_info(self, url, download=True):
                if failure is not None:
                    raise RuntimeError(failure)
                path = Path(self.options["paths"]["home"]) / "video.mp4"
                path.write_bytes(CLIP)
                return {"requested_downloads": [{"filepath": str(path)}]}

        return types.SimpleNamespace(YoutubeDL=YoutubeDL)

    def test_downloads_a_public_media_link(self):
        ingest.download(self.url("/clip.mp4"), self.out, allow_local=True)
        self.assertEqual(self.out.read_bytes(), CLIP)

    def test_downloads_a_public_video_page(self):
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp()}):
            ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertEqual(self.out.read_bytes(), CLIP)

    def test_video_download_uses_no_cookies_and_one_item(self):
        seen = {}
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp(seen=seen)}):
            ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertTrue(seen["noplaylist"])
        self.assertNotIn("cookiefile", seen)
        self.assertNotIn("cookiesfrombrowser", seen)
        self.assertEqual(seen["paths"]["home"], seen["paths"]["temp"])
        self.assertGreater(seen["max_filesize"], 0)

    def test_a_cookies_file_reaches_ytdlp(self):
        seen = {}
        cookies = Path(self.directory.name) / "cookies.txt"
        cookies.write_text(
            "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\tSID\tabc\n",
            encoding="utf-8",
        )
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp(seen=seen)}):
            ingest.download(self.url("/page"), self.out, allow_local=True, cookies=cookies)
        self.assertEqual(seen["cookiefile"], str(cookies))
        self.assertEqual(self.out.read_bytes(), CLIP)

    def test_a_missing_cookies_file_is_refused_before_ytdlp(self):
        class Never:
            def __init__(self, options):
                raise AssertionError("yt-dlp must not start without the cookies file")

        missing = Path(self.directory.name) / "none.txt"
        with mock.patch.dict(sys.modules, {"yt_dlp": types.SimpleNamespace(YoutubeDL=Never)}):
            with self.assertRaises(ingest.Refused) as refused:
                ingest.download(self.url("/page"), self.out, allow_local=True, cookies=missing)
        self.assertIn("no cookies file", str(refused.exception))
        self.assertFalse(self.out.exists())

    def test_a_private_video_points_at_the_cookies_file(self):
        failure = "ERROR: [youtube] abc: Private video. Sign in if you've been granted access"
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp(failure=failure)}):
            with self.assertRaises(ingest.Refused) as refused:
                ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertIn("cookies file", str(refused.exception))

    def test_refuses_a_private_video(self):
        failure = "ERROR: [youtube] abc: Private video. Sign in if you've been granted access"
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp(failure=failure)}):
            with self.assertRaises(ingest.Refused) as refused:
                ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertIn("private", str(refused.exception))
        self.assertFalse(self.out.exists())

    def test_refuses_an_unsupported_page(self):
        failure = "ERROR: Unsupported URL: https://example.com/page"
        with mock.patch.dict(sys.modules, {"yt_dlp": self.ytdlp(failure=failure)}):
            with self.assertRaises(ingest.Refused) as refused:
                ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertIn("web page", str(refused.exception))
        self.assertFalse(self.out.exists())

    def test_reports_a_missing_ytdlp(self):
        with mock.patch.dict(sys.modules, {"yt_dlp": None}):
            with self.assertRaises(ingest.FetchFailed) as failed:
                ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertIn("yt-dlp", str(failed.exception))
        self.assertIn("web page", str(failed.exception))
        self.assertFalse(self.out.exists())

    def test_refuses_localhost_without_the_override(self):
        with self.assertRaises(ingest.Refused) as refused:
            ingest.download(self.url("/clip.mp4"), self.out)
        self.assertIn("private", str(refused.exception))
        self.assertFalse(self.out.exists())

    def test_refuses_localhost_by_name(self):
        with self.assertRaises(ingest.Refused):
            ingest.download(
                f"http://localhost:{self.port}/clip.mp4", self.out, allow_local=False
            )

    def test_refuses_a_missing_file(self):
        with self.assertRaises(ingest.FetchFailed) as failed:
            ingest.download(self.url("/missing"), self.out, allow_local=True)
        self.assertIn("404", str(failed.exception))
        self.assertFalse(self.out.exists())

    def test_refuses_a_redirect(self):
        with self.assertRaises(ingest.Refused) as refused:
            ingest.download(self.url("/redirect"), self.out, allow_local=True)
        self.assertIn("redirects", str(refused.exception))
        self.assertFalse(self.out.exists())

    def test_refuses_an_empty_file(self):
        with self.assertRaises(ingest.Refused):
            ingest.download(self.url("/empty"), self.out, allow_local=True)

    def test_refuses_a_file_over_the_limit(self):
        with self.assertRaises(ingest.Refused):
            ingest.download(self.url("/big"), self.out, allow_local=True, max_bytes=10)
        self.assertFalse(self.out.exists())

    def test_refuses_other_schemes(self):
        with self.assertRaises(ingest.Refused):
            ingest.download("ftp://example.com/clip.mp4", self.out)

    def test_refuses_credentials(self):
        with self.assertRaises(ingest.Refused):
            ingest.download(
                f"http://user:secret@127.0.0.1:{self.port}/clip.mp4", self.out
            )

    def test_command_reports_a_refusal(self):
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPTS / "ingest.py"),
                "--url",
                self.url("/clip.mp4"),
                "--out",
                str(self.out),
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("private", json.loads(result.stderr)["error"])


if __name__ == "__main__":
    unittest.main()
