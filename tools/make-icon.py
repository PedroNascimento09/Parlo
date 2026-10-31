import sys
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
PAPER = (250, 246, 239, 255)
INK = (27, 22, 19, 255)
VERMILION = (217, 72, 31, 255)
TEAL = (15, 118, 110, 255)
SIZES = (16, 24, 32, 48, 64, 128, 256)


def rounded(draw, box, radius, fill):
    draw.rounded_rectangle(box, radius=radius, fill=fill)


def glyph(draw, scale):
    centre = SIZE // 2
    radius = int(SIZE * 0.33)
    draw.ellipse(
        (centre - radius, centre - radius, centre + radius, centre + radius),
        fill=VERMILION,
    )
    bar = int(SIZE * 0.045)
    gap = int(SIZE * 0.055)
    heights = (0.10, 0.20, 0.29, 0.20, 0.10)
    total = len(heights) * bar + (len(heights) - 1) * gap
    left = centre - total // 2
    for index, height in enumerate(heights):
        half = int(SIZE * height)
        x = left + index * (bar + gap)
        colour = PAPER if index != 2 else TEAL
        rounded(draw, (x, centre - half, x + bar, centre + half), bar // 2, colour)


def build() -> Image.Image:
    image = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    margin = int(SIZE * 0.06)
    rounded(draw, (margin, margin, SIZE - margin, SIZE - margin), int(SIZE * 0.22), INK)
    inner = margin + int(SIZE * 0.055)
    rounded(
        draw,
        (inner, inner, SIZE - inner, SIZE - inner),
        int(SIZE * 0.18),
        PAPER,
    )
    glyph(draw, SIZE)
    return image


def main() -> int:
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "assets")
    out.mkdir(parents=True, exist_ok=True)
    image = build()
    image.resize((512, 512), Image.LANCZOS).save(out / "parlo.png")
    image.save(
        out / "parlo.ico",
        sizes=[(size, size) for size in SIZES],
    )
    image.resize((256, 256), Image.LANCZOS).save(out / "parlo-256.png")
    print(f"wrote {out / 'parlo.png'}, {out / 'parlo.ico'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
