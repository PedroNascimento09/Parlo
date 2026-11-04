import sys
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
PAPER = (247, 242, 232, 255)
VERMILION = (201, 80, 47, 255)
SIZES = (16, 24, 32, 48, 64, 128, 256)


def rounded(draw, box, radius, fill):
    draw.rounded_rectangle(box, radius=radius, fill=fill)


def build() -> Image.Image:
    image = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    unit = SIZE / 512
    rounded(draw, (0, 0, SIZE - 1, SIZE - 1), 112 * unit, VERMILION)
    rounded(draw, (134 * unit, 114 * unit, 192 * unit, 398 * unit), 29 * unit, PAPER)
    draw.ellipse((192 * unit, 134 * unit, 376 * unit, 318 * unit), fill=PAPER)
    rounded(draw, (278 * unit, 158 * unit, 290 * unit, 296 * unit), 6 * unit, VERMILION)
    return image


def main() -> int:
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "assets")
    out.mkdir(parents=True, exist_ok=True)
    image = build()
    image.resize((512, 512), Image.LANCZOS).save(out / "parlo.png")
    image.resize((256, 256), Image.LANCZOS).save(out / "parlo-256.png")
    image.save(out / "parlo.ico", sizes=[(size, size) for size in SIZES])
    print(f"wrote {out / 'parlo.png'}, {out / 'parlo-256.png'}, {out / 'parlo.ico'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
