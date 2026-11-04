import json
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

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

    def test_downloads_a_public_media_link(self):
        ingest.download(self.url("/clip.mp4"), self.out, allow_local=True)
        self.assertEqual(self.out.read_bytes(), CLIP)

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

    def test_refuses_a_web_page(self):
        with self.assertRaises(ingest.Refused) as refused:
            ingest.download(self.url("/page"), self.out, allow_local=True)
        self.assertIn("web page", str(refused.exception))
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
