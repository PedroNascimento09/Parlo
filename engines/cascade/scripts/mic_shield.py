#!/usr/bin/env python3
"""Draw the Parlo microphone shield icon used in docs and the README.

The icon is a plain, line-based badge: a small mic inside a rounded shield. It is
intentionally simple so it still looks clean when the repo is cloned without any
assets folder and someone is reading the docs offline.

Run from the repository root:

    python engines/cascade/scripts/mic_shield.py

Outputs:
* assets/parlo-mic-shield.png
* assets/parlo-mic-shield-dark.png
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFilter
except ImportError as exc:
    sys.exit(
        "Pillow is required for the mic shield script: "
        "python -m pip install Pillow"
    )

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
ASSETS.mkdir(parents=True, exist_ok=True)

INK = (25, 21, 18)
PAPER = (253, 251, 248)
BRAND = (47, 107, 79)
FADED = (120, 106, 92)


def shield(size: int, ink: tuple[int, int, int], paper: tuple[int, int, int],
           accent: tuple[int, int, int]) -> Image.Image:
    """Return a rounded shield with a faint fill and a clean outline."""
    layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    margin = int(size * 0.08)
    top = margin
    bottom = size - margin
    mid = size / 2
    polygon = [
        (margin, top),
        (size - margin, top),
        (size - margin, mid),
        (margin, bottom),
    ]
    d.polygon(polygon, fill=(accent[0], accent[1], accent[2], 14))
    d.line(polygon + [polygon[0]], fill=ink, width=max(2, int(size * 0.014)), joint="curve")
    im = Image.new("RGB", (size, size), paper)
    mask = layer.split()[3].point(lambda p: 255 if p > 0 else 0)
    im.paste(layer, (0, 0), layer)
    return im


def rounded_polygon_mask(size: int, radius: int) -> Image.Image:
    """A rounded-corner polygon mask for the shield shape."""
    mask = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(mask)
    margin = int(size * 0.08)
    top = margin
    bottom = size - margin
    mid = size / 2
    polygon = [
        (margin, top),
        (size - margin, top),
        (size - margin, mid),
        (margin, bottom),
    ]
    d.polygon(polygon, fill=255)
    return mask.filter(ImageFilter.GaussianBlur(radius=max(1, int(size * 0.02))))


def shield_round(size: int, ink: tuple[int, int, int], paper: tuple[int, int, int],
                 accent: tuple[int, int, int]) -> Image.Image:
    """Rounded shield, using a blur-based rounding so it does not need rounded_polygon."""
    mask = rounded_polygon_mask(size, int(size * 0.06))
    accent_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(accent_layer)
    d.polygon([
        (int(size * 0.08), int(size * 0.08)),
        (size - int(size * 0.08), int(size * 0.08)),
        (size - int(size * 0.08), int(size * 0.5)),
        (int(size * 0.08), size - int(size * 0.08)),
    ], fill=(accent[0], accent[1], accent[2], 16))
    ink_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(ink_layer)
    d.line(mask.getbbox() and [(
        int(size * 0.08), int(size * 0.08),
        size - int(size * 0.08), int(size * 0.08),
        size - int(size * 0.08), int(size * 0.5),
        int(size * 0.08), size - int(size * 0.08),
    )[i:i + 2] for i in range(0, 8, 2)], fill=ink, width=max(2, int(size * 0.014)))
    im = Image.new("RGB", (size, size), paper)
    im.paste(accent_layer, (0, 0), accent_layer.split()[3])
    im.paste(ink_layer, (0, 0), ink_layer.split()[3])
    return im


def mic(size: int, ink: tuple[int, int, int], accent: tuple[int, int, int]) -> Image.Image:
    """Return a small microphone icon for the shield."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    cx = size / 2
    top_y = size * 0.20
    base_y = size * 0.78
    body_h = base_y - top_y
    body_w = size * 0.20
    x0 = cx - body_w / 2
    x1 = cx + body_w / 2
    draw.rounded_rectangle(
        [x0, top_y, x1, base_y],
        radius=body_w / 2,
        fill=ink,
    )
    holder_w = size * 0.34
    holder_h = size * 0.10
    holder_top = top_y - holder_h
    draw.rounded_rectangle(
        [cx - holder_w / 2, holder_top, cx + holder_w / 2, top_y],
        radius=holder_h / 2,
        fill=accent,
    )
    stand_w = size * 0.08
    stand_h = size * 0.06
    stand_y = base_y
    draw.rounded_rectangle(
        [cx - stand_w / 2, stand_y, cx + stand_w / 2, stand_y + stand_h],
        radius=stand_h / 2,
        fill=ink,
    )
    return img


def mic_shield(size: int, ink: tuple[int, int, int], paper: tuple[int, int, int],
               accent: tuple[int, int, int]) -> Image.Image:
    base = shield_round(size, ink, paper, accent)
    mic_layer = mic(int(size * 0.5), ink, accent)
    mic_layer = mic_layer.resize((int(size * 0.5), int(size * 0.5)), Image.LANCZOS)
    base.paste(mic_layer, (int(size * 0.25), int(size * 0.25)), mic_layer)
    return base


def main() -> int:
    parser = argparse.ArgumentParser(description="Draw the Parlo microphone shield icon.")
    parser.add_argument(
        "--assets",
        type=Path,
        default=ASSETS,
        help="folder for the outputs (default: assets/)",
    )
    parser.add_argument(
        "--size",
        type=int,
        default=512,
        help="icon size in pixels (default: 512)",
    )
    args = parser.parse_args()

    args.assets.mkdir(parents=True, exist_ok=True)

    light = mic_shield(args.size, INK, PAPER, BRAND)
    light_path = args.assets / "parlo-mic-shield.png"
    light.save(light_path, "PNG")
    print(f"wrote {light_path}")

    dark = mic_shield(args.size, PAPER, (28, 24, 20), (158, 217, 180))
    dark_path = args.assets / "parlo-mic-shield-dark.png"
    dark.save(dark_path, "PNG")
    print(f"wrote {dark_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
