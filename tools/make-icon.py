import struct
import sys
from io import BytesIO
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
TILE = 512
NIGHT = (18, 16, 13, 255)
PAPER = (247, 242, 232, 255)
GLYPH = (187, 296)
BAR_SEGMENTS = (
    (0, 12, 12, 272),
    (19, 0, 13, 296),
    (39, 15, 12, 266),
    (58, 22, 13, 43),
    (58, 145, 13, 51),
    (78, 9, 12, 46),
    (78, 159, 12, 42),
    (97, 9, 12, 47),
    (97, 149, 12, 56),
    (116, 4, 13, 201),
    (136, 20, 12, 168),
    (155, 24, 13, 157),
    (175, 44, 12, 114),
)
ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)
BARS_FROM = 48


def canvas():
    image = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.rounded_rectangle((0, 0, SIZE - 1, SIZE - 1), radius=112 * SIZE / TILE, fill=NIGHT)
    return image, draw


def build_bars():
    image, draw = canvas()
    unit = SIZE / TILE
    height = 400 * unit
    scale = height / GLYPH[1]
    left = (SIZE - GLYPH[0] * scale) / 2
    top = (SIZE - height) / 2
    for x, y, width, bar in BAR_SEGMENTS:
        draw.rounded_rectangle(
            (
                left + x * scale,
                top + y * scale,
                left + (x + width) * scale,
                top + (y + bar) * scale,
            ),
            radius=width * scale / 2,
            fill=PAPER,
        )
    return image


def build_solid():
    image, draw = canvas()
    unit = SIZE / TILE
    draw.rounded_rectangle(
        (134 * unit, 114 * unit, 192 * unit, 398 * unit), radius=29 * unit, fill=PAPER
    )
    draw.ellipse((192 * unit, 134 * unit, 376 * unit, 318 * unit), fill=PAPER)
    return image


def write_ico(path, small, detailed):
    frames = [
        (size, (small if size < BARS_FROM else detailed).resize((size, size), Image.LANCZOS))
        for size in ICO_SIZES
    ]
    header = struct.pack("<HHH", 0, 1, len(frames))
    entries = b""
    blobs = b""
    offset = 6 + 16 * len(frames)
    for size, frame in frames:
        buffer = BytesIO()
        frame.save(buffer, format="PNG")
        blob = buffer.getvalue()
        entries += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
        blobs += blob
    path.write_bytes(header + entries + blobs)


def main() -> int:
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "assets")
    out.mkdir(parents=True, exist_ok=True)
    bars = build_bars()
    solid = build_solid()
    bars.resize((512, 512), Image.LANCZOS).save(out / "parlo.png")
    bars.resize((256, 256), Image.LANCZOS).save(out / "parlo-256.png")
    write_ico(out / "parlo.ico", solid, bars)
    print(f"wrote {out / 'parlo.png'}, {out / 'parlo-256.png'}, {out / 'parlo.ico'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
