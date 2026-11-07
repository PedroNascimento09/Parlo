#!/usr/bin/env python3
"""Build the Parlo logo and the short trailer frames used in the README.

Run from the repository root with the environment the cascade scripts expect:

    python engines/cascade/scripts/logo_trail.py

Outputs:
* assets/parlo-wordmark.png
* assets/parlo-trailer-*.png (720x1280 stills, one per second of the 9 s trailer)

No model download is required. The script only needs Pillow; if it is missing the
script prints what to install and exits with a non-zero code.
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:
    sys.exit(
        "Pillow is required for the logo/trailer script: "
        "python -m pip install Pillow"
    )

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
ASSETS.mkdir(parents=True, exist_ok=True)

# Brand colours. Keep these in sync with the CSS variables in crates/parlo-cli/web/app.css.
BRAND_GREEN = (47, 107, 79)
BRAND_GREEN_DARK = (30, 71, 52)
INK = (25, 21, 18)
PAPER = (253, 251, 248)
FADED = (120, 106, 92)
WHITE = (255, 255, 255)


def wordmark(size: int) -> Image.Image:
    """Render the Parlo wordmark with the mark beside the text.

    The SVG in index.html is the reference; this is a raster stand-in good enough for
    the README hero and the trailer end card.
    """
    scale = size / 280
    img = Image.new("RGB", (size, int(size * 0.52)), PAPER)
    draw = ImageDraw.Draw(img)

    mark_x = int(28 * scale)
    mark_top = int(16.86 * scale)
    mark = [
        (0.00, 16.86, 1.91, 45.12),
        (3.23, 14.85, 1.91, 49.15),
        (6.47, 17.38, 1.91, 44.09),
        (9.70, 18.50, 1.91, 7.01),
        (9.70, 39.03, 1.91, 8.29),
        (12.93, 16.31, 1.91, 7.56),
        (12.93, 41.34, 1.91, 6.75),
        (16.17, 16.35, 1.91, 7.63),
        (16.17, 39.61, 1.91, 9.18),
        (19.40, 15.55, 1.91, 33.34),
        (22.64, 18.18, 1.91, 27.79),
        (25.87, 18.80, 1.91, 25.96),
        (29.10, 22.11, 1.91, 18.90),
        (35.57, 32.07, 1.91, 13.97),
        (38.80, 17.93, 1.91, 6.84),
        (38.80, 29.65, 1.91, 18.66),
        (42.04, 16.44, 1.91, 7.45),
        (42.04, 29.43, 1.91, 18.52),
        (45.27, 16.49, 1.91, 6.18),
        (45.27, 28.26, 1.91, 6.60),
        (45.27, 41.09, 1.91, 7.55),
        (48.50, 16.40, 1.91, 6.21),
        (48.50, 28.11, 1.91, 5.58),
        (48.50, 40.82, 1.91, 7.20),
        (51.74, 16.26, 1.91, 10.33),
        (51.74, 27.85, 1.91, 5.22),
        (51.74, 36.17, 1.91, 9.44),
        (54.97, 15.56, 1.91, 33.48),
        (58.21, 19.08, 1.91, 28.04),
        (61.44, 23.67, 1.91, 23.14),
        (71.14, 15.93, 1.91, 32.73),
        (74.37, 15.96, 1.91, 32.67),
        (77.61, 18.03, 1.91, 28.54),
        (80.84, 18.22, 1.91, 9.18),
        (84.07, 16.06, 1.91, 9.09),
        (87.31, 16.20, 1.91, 8.82),
        (93.78, 4.07, 1.91, 41.61),
        (97.01, 1.88, 1.91, 45.99),
        (100.24, 0.00, 1.91, 49.75),
        (103.48, 2.68, 1.91, 44.39),
        (109.94, 25.55, 1.91, 13.78),
        (113.18, 19.79, 1.91, 25.15),
        (116.41, 17.06, 1.91, 30.61),
        (119.64, 17.90, 1.91, 28.93),
        (122.88, 16.49, 1.91, 7.95),
        (122.88, 39.60, 1.91, 9.05),
        (126.11, 16.02, 1.91, 7.57),
        (126.11, 41.44, 1.91, 6.70),
        (129.35, 16.30, 1.91, 8.17),
        (129.35, 39.56, 1.91, 9.13),
        (132.58, 16.96, 1.91, 30.52),
        (135.81, 19.11, 1.91, 26.07),
        (139.05, 19.79, 1.91, 24.87),
        (142.28, 24.09, 1.91, 15.97),
    ]
    for x, y, w, h in mark:
        rect = (
            mark_x + int(x * scale),
            mark_top + int(y * scale),
            mark_x + int((x + w) * scale),
            mark_top + int((y + h) * scale),
        )
        draw.rounded_rectangle(rect, radius=int(0.96 * scale), fill=BRAND_GREEN)

    text_x = mark_x + int(58 * scale)
    text_top = int(18 * scale)
    font = _font(size, bold=True)

    # "Parlo" with the P in brand green and the rest in ink, matching the mark's weight.
    label = "Parlo"
    for idx, char in enumerate(label):
        cx = text_x + int(sum(font.getlength(label[: i + 1]) for i in range(idx)))
        if idx == 0:
            fill = BRAND_GREEN
        else:
            fill = INK
        draw.text((cx, text_top), char, font=font, fill=fill)
    return img


def _font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/Library/Fonts/Helvetica Neue Bold.ttf" if bold else "/Library/Fonts/Helvetica Neue.ttf",
        "C:/Windows/Fonts/seguisym.ttf" if bold else "C:/Windows/Fonts/segoeui.ttf",
    ]
    for path in candidates:
        try:
            return ImageFont.truetype(path, size=max(int(size * 0.18), 12))
        except OSError:
            continue
    return ImageFont.load_default()


def trailer_frames(frame_rate: int = 12) -> list[Image.Image]:
    """Return one PIL image per frame of the 9-second brand trailer.

    The trailer is a small brand clip made from the same colour tokens as the app. It is a
    stand-in for a real rendered trailer until there is one, and each frame is a still
    that can be used in docs without pretending it is live audio.
    """
    width, height = 720, 1280
    frames: list[Image.Image] = []
    duration = 9.0
    total = int(duration * frame_rate)

    for frame in range(total):
        t = frame / frame_rate
        img = Image.new("RGB", (width, height), PAPER)
        draw = ImageDraw.Draw(img)

        # Soft brand band behind the content.
        band_top = int(height * 0.30)
        band_bottom = int(height * 0.70)
        for y in range(band_top, band_bottom, 2):
            frac = (y - band_top) / max(band_bottom - band_top, 1)
            r = int(BRAND_GREEN[0] * (0.85 + 0.15 * math.sin(frac * math.pi)))
            g = int(BRAND_GREEN[1] * (0.85 + 0.15 * math.sin(frac * math.pi)))
            b = int(BRAND_GREEN[2] * (0.85 + 0.15 * math.sin(frac * math.pi)))
            draw.line([(0, y), (width, y)], fill=(r, g, b))

        # Parlo mark, breathing very slightly with time.
        mark_size = int(min(width, height) * 0.16)
        pulse = 1.0 + 0.04 * math.sin(t * 2.0)
        word = wordmark(int(mark_size * pulse))
        word = word.resize((int(word.width * pulse), int(word.height * pulse)), Image.LANCZOS)
        draw.bitmap(
            ((width - word.width) // 2, int(height * 0.34)),
            word.convert("L"),
            fill=INK,
        )

        # Headline animates in during the first three seconds.
        if t < 3.0:
            alpha = min(1.0, t / 2.2)
            headline = "Speak once."
            sub = "Be heard."
            headline_font = _font(int(height * 0.07), bold=True)
            sub_font = _font(int(height * 0.034))
            headline_w = int(draw.textlength(headline, font=headline_font) * alpha)
            sub_w = int(draw.textlength(sub, font=sub_font) * alpha)
            draw.text(
                ((width - headline_w) // 2, int(height * 0.58)),
                headline,
                font=headline_font,
                fill=tuple(int(c * alpha + PAPER[i] * (1 - alpha)) for i, c in enumerate(INK)),
            )
            draw.text(
                ((width - sub_w) // 2, int(height * 0.66)),
                sub,
                font=sub_font,
                fill=tuple(int(c * alpha + PAPER[i] * (1 - alpha)) for i, c in enumerate(FADED)),
            )

        # Lower third lists the three things a first-time user should trust.
        if t >= 4.0:
            frac = min(1.0, (t - 4.0) / 1.5)
            items = ["Nothing leaves this computer", "Portuguese and English, both ways", "One shortcut, anywhere"]
            item_font = _font(int(height * 0.028))
            y = int(height * 0.78)
            for item in items:
                w = int(draw.textlength(item, font=item_font) * frac)
                draw.text(
                    ((width - w) // 2, y),
                    item,
                    font=item_font,
                    fill=tuple(int(c * frac + PAPER[i] * (1 - frac)) for i, c in enumerate(BRAND_GREEN_DARK)),
                )
                y += int(height * 0.058)

        # End card fades in at the end.
        if t >= 7.5:
            frac = min(1.0, (t - 7.5) / 1.4)
            end = wordmark(int(min(width, height) * 0.22 * frac))
            draw.bitmap(
                ((width - end.width) // 2, int(height * 0.86)),
                end.convert("L"),
                fill=INK,
            )
            tag = "Parlo · local translation"
            tag_font = _font(int(height * 0.026))
            tw = int(draw.textlength(tag, font=tag_font) * frac)
            draw.text(
                ((width - tw) // 2, int(height * 0.93)),
                tag,
                font=tag_font,
                fill=tuple(int(c * frac + PAPER[i] * (1 - frac)) for i, c in enumerate(FADED)),
            )

        frames.append(img)
    return frames


def write_frame(frame: Image.Image, out_dir: Path, index: int) -> Path:
    path = out_dir / f"parlo-trailer-{index:02d}.png"
    frame.save(path, "PNG")
    return path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render the Parlo wordmark and trailer stills."
    )
    parser.add_argument(
        "--assets",
        type=Path,
        default=ASSETS,
        help="folder for the outputs (default: assets/)",
    )
    parser.add_argument(
        "--wordmark",
        type=int,
        default=280,
        help="wordmark width in pixels (default: 280)",
    )
    parser.add_argument(
        "--stills",
        action="store_true",
        help="also write the 9 s trailer as a sequence of stills",
    )
    args = parser.parse_args()

    args.assets.mkdir(parents=True, exist_ok=True)
    wordmark_path = args.assets / "parlo-wordmark.png"
    wordmark(args.wordmark).save(wordmark_path, "PNG")
    print(f"wrote {wordmark_path}")

    if args.stills:
        frames = trailer_frames()
        for idx, frame in enumerate(frames):
            print(f"wrote {write_frame(frame, args.assets, idx)}")
        print(f"wrote {args.assets / 'parlo-trailer-00.png'} .. {args.assets / f'parlo-trailer-{len(frames)-1:02d}.png'}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
