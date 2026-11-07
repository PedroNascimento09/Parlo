#!/usr/bin/env python3
"""Build the Parlo book preview images for the README and docs.

This is the small helper that turns a short script into a couple of captioned stills
so the README can talk about "a book-like preview" without shipping prose alone.
It is intentionally tiny: plain Python, Pillow only, no model download, no network.

Usage from the repository root:

    python engines/cascade/scripts/book_preview.py

Outputs:
* assets/parlo-book-preview-*.png — a neutral cover and an interior page pin-up
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:
    sys.exit(
        "Pillow is required for the book preview script: "
        "python -m pip install Pillow"
    )

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
ASSETS.mkdir(parents=True, exist_ok=True)

BRAND_GREEN = (47, 107, 79)
BRAND_GREEN_DARK = (30, 71, 52)
INK = (25, 21, 18)
PAPER = (253, 251, 248)
FADED = (120, 106, 92)
RULE = (25, 21, 18, 0.16)


def _font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/Library/Fonts/Helvetica Neue Bold.ttf" if bold else "/Library/Fonts/Helvetica Neue.ttf",
        "C:/Windows/Fonts/seguisym.ttf" if bold else "C:/Windows/Fonts/segoeui.ttf",
    ]
    for path in candidates:
        try:
            return ImageFont.truetype(path, size=max(int(size), 12))
        except OSError:
            continue
    return ImageFont.load_default()


def cover(width: int = 1280, height: int = 1920) -> Image.Image:
    """A readable cover card for the README hero.

    The design is deliberately boring on purpose: a brand band, one line of type, and
    a faint page texture. It should age better than a glossy mock-up.
    """
    img = Image.new("RGB", (width, height), PAPER)
    draw = ImageDraw.Draw(img)

    # Top brand band.
    band = height // 6
    for y in range(band):
        frac = y / max(band, 1)
        r = int(BRAND_GREEN[0] * (0.92 + 0.08 * (1 - frac)))
        g = int(BRAND_GREEN[1] * (0.92 + 0.08 * (1 - frac)))
        b = int(BRAND_GREEN[2] * (0.92 + 0.08 * (1 - frac)))
        draw.line([(0, y), (width, y)], fill=(r, g, b))

    title_font = _font(int(height * 0.08), bold=True)
    title = "Parlo"
    title_w = int(draw.textlength(title, font=title_font))
    draw.text(((width - title_w) // 2, int(height * 0.30)), title, font=title_font, fill=INK)

    subtitle_font = _font(int(height * 0.034))
    subtitle = "Speak once. Be heard."
    sub_w = int(draw.textlength(subtitle, font=subtitle_font))
    draw.text(((width - sub_w) // 2, int(height * 0.42)), subtitle, font=subtitle_font, fill=BRAND_GREEN_DARK)

    # Faint rule under the cover text.
    rule_y = int(height * 0.50)
    margin = int(width * 0.18)
    draw.rectangle([margin, rule_y, width - margin, rule_y + 2], fill=BRAND_GREEN)

    body_font = _font(int(height * 0.026))
    lines = [
        "A local first translator for Portuguese and English.",
        "Dictation, live text, captions, meetings and file dubbing.",
        "Nothing leaves this computer unless you ask it to.",
    ]
    y = int(height * 0.56)
    leading = int(height * 0.05)
    for line in lines:
        w = int(draw.textlength(line, font=body_font))
        draw.text(((width - w) // 2, y), line, font=body_font, fill=FADED)
        y += leading

    return img


def page(width: int = 1280, height: int = 1920) -> Image.Image:
    """A neutral interior page so the README can show a 'reading' frame too.

    Most of the page is white with a small ruled column on the left and a single
    highlighted sentence near the middle, mimicking a transcript review rather than
    a marketing poster.
    """
    img = Image.new("RGB", (width, height), PAPER)
    draw = ImageDraw.Draw(img)

    # Left margin rule.
    margin = int(width * 0.10)
    draw.rectangle([margin, 0, margin + 3, height], fill=BRAND_GREEN_DARK)

    header_font = _font(int(height * 0.028))
    header = "Meeting transcript · reviewed"
    hw = int(draw.textlength(header, font=header_font))
    draw.text(((width - hw) // 2, int(height * 0.08)), header, font=header_font, fill=INK)

    body_font = _font(int(height * 0.03))
    small_font = _font(int(height * 0.024))

    paragraphs = [
        ("Speaker A", "Quero confirmar o horário da reunião da próxima segunda."),
        ("Speaker B", "A segunda às dez horas funciona para a equipa."),
        ("Speaker A", "Então fica confirmado. A gravação fica na pasta partilhada."),
    ]
    y = int(height * 0.18)
    leading = int(height * 0.056)
    for speaker, sentence in paragraphs:
        sw = int(draw.textlength(speaker, font=small_font))
        draw.text((margin + int(width * 0.04), y), speaker, font=small_font, fill=BRAND_GREEN_DARK)
        draw.text((margin + int(width * 0.04) + sw + int(width * 0.02), y), "—", font=small_font, fill=FADED)
        draw.text((margin + int(width * 0.04) + sw + int(width * 0.02) + int(draw.textlength("—", font=small_font)) + int(width * 0.02), y), sentence, font=body_font, fill=INK)
        y += leading
        # small rule between speakers
        draw.rectangle([margin + int(width * 0.04), y - int(leading * 0.18), width - margin - int(width * 0.04), y - int(leading * 0.16)], fill=BRAND_GREEN_DARK)

    # Highlighted line at the bottom third: a note about where the file goes.
    note_font = _font(int(height * 0.032), bold=True)
    note = "O resumo fica no teu computador. Nada sobe para a nuvem."
    nw = int(draw.textlength(note, font=note_font))
    note_y = int(height * 0.78)
    draw.rectangle(
        [margin + int(width * 0.04), note_y - int(height * 0.006),
         width - margin - int(width * 0.04), note_y + int(height * 0.054)],
        fill=(240, 249, 242),
    )
    draw.text(((width - nw) // 2, note_y), note, font=note_font, fill=BRAND_GREEN_DARK)

    footer_font = _font(int(height * 0.022))
    footer = "Parlo · local translation · nothing leaves this computer"
    fw = int(draw.textlength(footer, font=footer_font))
    draw.text(((width - fw) // 2, height - int(height * 0.06)), footer, font=footer_font, fill=FADED)

    return img


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render the Parlo book preview stills."
    )
    parser.add_argument(
        "--assets",
        type=Path,
        default=ASSETS,
        help="folder for the outputs (default: assets/)",
    )
    parser.add_argument(
        "--size",
        type=int,
        default=1280,
        help="width in pixels for the stills (default: 1280)",
    )
    args = parser.parse_args()

    args.assets.mkdir(parents=True, exist_ok=True)

    cover_path = args.assets / "parlo-book-preview-cover.png"
    cover(args.size, int(args.size * 1.5)).save(cover_path, "PNG")
    print(f"wrote {cover_path}")

    page_path = args.assets / "parlo-book-preview-page.png"
    page(args.size, int(args.size * 1.5)).save(page_path, "PNG")
    print(f"wrote {page_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
