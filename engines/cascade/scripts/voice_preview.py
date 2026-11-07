#!/usr/bin/env python3
"""Build a neutral voice preview panel image for the README and docs.

It is not a real voice capture and not a real waveform from a user. It is a small
illustration that shows what the voice preview panel looks like, so the README can
talk about the panel without pretending the screenshot is live audio.

Run from the repository root:

    python engines/cascade/scripts/voice_preview.py

Outputs:
* assets/parlo-voice-preview.png
* assets/parlo-voice-preview-dark.png
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
        "Pillow is required for the voice preview script: "
        "python -m pip install Pillow"
    )

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
ASSETS.mkdir(parents=True, exist_ok=True)

INK = (25, 21, 18)
PAPER = (253, 251, 248)
BRAND = (47, 107, 79)
BRAND_DARK = (30, 71, 52)
FADED = (120, 106, 92)
MUTED = (164, 150, 136)
WAVE_GREEN = (99, 180, 139)
WAVE_KIND = (238, 203, 100)


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


def _wave(size: int, color: tuple[int, int, int], phase: float = 0.0) -> list[tuple[float, float]]:
    width, height = size, int(size * 0.16)
    points: list[tuple[float, float]] = []
    for x in range(width):
        t = (x / width) * math.pi * 6 + phase
        envelope = math.sin((x / width) * math.pi) ** 1.5
        y = height / 2 + math.sin(t) * height * 0.35 * envelope
        points.append((x, y))
    return points


def mic_small(size: int, ink: tuple[int, int, int], accent: tuple[int, int, int]) -> Image.Image:
    """A small microphone icon on a transparent RGBA background."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    cx = size / 2
    top_y = size * 0.18
    base_y = size * 0.80
    body_w = size * 0.34
    x0 = cx - body_w / 2
    x1 = cx + body_w / 2
    draw.rounded_rectangle([x0, top_y, x1, base_y], radius=body_w / 2, fill=ink)
    holder_h = size * 0.12
    draw.rounded_rectangle(
        [cx - body_w * 1.5, top_y - holder_h, cx + body_w * 1.5, top_y],
        radius=holder_h / 2,
        fill=accent,
    )
    stand_h = size * 0.08
    draw.rounded_rectangle(
        [cx - body_w * 0.2, base_y, cx + body_w * 0.2, base_y + stand_h],
        radius=stand_h / 2,
        fill=ink,
    )
    return img


def voice_panel(width: int = 1280, height: int = 720,
                ink: tuple[int, int, int] = INK,
                paper: tuple[int, int, int] = PAPER,
                accent: tuple[int, int, int] = BRAND) -> Image.Image:
    """A calm preview card with a mic button, waveform and a short caption."""
    img = Image.new("RGB", (width, height), paper)
    draw = ImageDraw.Draw(img)

    # Soft brand band on the left edge only, so the card reads as a panel, not a poster.
    panel = int(width * 0.34)
    for x in range(panel):
        frac = x / max(panel, 1)
        c = tuple(int(accent[i] * (0.92 + 0.08 * (1 - frac))) for i in range(3))
        draw.line([(x, 0), (x, height)], fill=c)

    title_font = _font(int(height * 0.06), bold=True)
    title = "Voice preview"
    tw = int(draw.textlength(title, font=title_font))
    draw.text(((width - tw) // 2, int(height * 0.08)), title, font=title_font, fill=ink)

    subtitle_font = _font(int(height * 0.028))
    subtitle = "Hear how Parlo sounds before you send a full voice."
    sw = int(draw.textlength(subtitle, font=subtitle_font))
    draw.text(((width - sw) // 2, int(height * 0.16)), subtitle, font=subtitle_font, fill=FADED)

    # Mic button: a rounded square with a faint mic icon inside.
    btn_size = int(height * 0.22)
    btn_x = (width - btn_size) // 2
    btn_y = int(height * 0.30)
    draw.rounded_rectangle(
        [btn_x, btn_y, btn_x + btn_size, btn_y + btn_size],
        radius=btn_size * 0.14,
        fill=ink,
    )
    inner = int(btn_size * 0.7)
    inner_x = btn_x + (btn_size - inner) // 2
    inner_y = btn_y + (btn_size - inner) // 2
    draw.rounded_rectangle(
        [inner_x, inner_y, inner_x + inner, inner_y + inner],
        radius=inner * 0.10,
        fill=paper,
    )
    mic_small_size = int(inner * 0.5)
    mic_layer = mic_small(mic_small_size, ink, accent)
    mic_small_final = int(mic_small_size * 0.62)
    mic_layer = mic_layer.resize((mic_small_final, mic_small_final), Image.LANCZOS)
    mic_final_x = btn_x + (btn_size - mic_small_final) // 2
    mic_final_y = btn_y + (btn_size - mic_small_final) // 2
    img.paste(mic_layer, (mic_final_x, mic_final_y), mic_layer)

    # Waveform under the button.
    wave_y = btn_y + btn_size + int(height * 0.04)
    wave = _wave(width, accent, phase=0.6)
    for i in range(len(wave) - 1):
        draw.line([wave[i], wave[i + 1]], fill=accent, width=3)

    wave2 = _wave(width, MUTED, phase=1.2)
    for i in range(len(wave2) - 1):
        draw.line([wave2[i], wave2[i + 1]], fill=MUTED, width=2)

    # Caption under the waveform.
    caption_font = _font(int(height * 0.03))
    caption = "Sintética · precisa ajustar o volume e a claridade"
    cw = int(draw.textlength(caption, font=caption_font))
    draw.text(((width - cw) // 2, wave_y + int(height * 0.06)), caption, font=caption_font, fill=FADED)

    # Footer note.
    footer_font = _font(int(height * 0.022))
    footer = "Este painel mostra a interface, não gravações do teu microfone."
    fw = int(draw.textlength(footer, font=footer_font))
    draw.text(((width - fw) // 2, height - int(height * 0.06)), footer, font=footer_font, fill=MUTED)

    return img


def voice_panel_dark(width: int = 1280, height: int = 720) -> Image.Image:
    return voice_panel(
        width,
        height,
        ink=(255, 255, 255),
        paper=(24, 21, 18),
        accent=(158, 217, 180),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Render the Parlo voice preview panel.")
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
        help="panel width in pixels (default: 1280)",
    )
    args = parser.parse_args()

    args.assets.mkdir(parents=True, exist_ok=True)

    light = voice_panel(args.size, int(args.size * 0.56))
    light_path = args.assets / "parlo-voice-preview.png"
    light.save(light_path, "PNG")
    print(f"wrote {light_path}")

    dark = voice_panel_dark(args.size, int(args.size * 0.56))
    dark_path = args.assets / "parlo-voice-preview-dark.png"
    dark.save(dark_path, "PNG")
    print(f"wrote {dark_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
