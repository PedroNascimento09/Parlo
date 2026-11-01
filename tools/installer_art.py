import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from PIL import Image, ImageDraw

from brand import ACCENT, BODY, INK, MUTED, RULE, micro, mark, paper, text

OUT = Path(__file__).resolve().parent.parent / "assets" / "installer"


def wizard() -> None:
    width, height = 164, 314
    image = paper(width, height)
    draw = ImageDraw.Draw(image)

    for y in range(0, height, 24):
        draw.line((0, y, width, y), fill=RULE, width=1)

    draw.rectangle((0, 0, 5, height), fill=ACCENT)
    mark(draw, 26, 26, 34)
    micro(draw, (26, 72), "PARLO", INK, size=15)
    micro(draw, (26, 94), "ON THIS MACHINE", ACCENT, size=8)

    draw.line((26, 122, width - 22, 122), fill=INK, width=1)

    lines = ("Speech into text.", "Text into another", "language.", "And a voice back.")
    y = 140
    for line in lines:
        text(draw, (26, y), line, size=12, fill=INK, face=BODY)
        y += 17

    draw.line((26, height - 76, width - 22, height - 76), fill=RULE, width=1)
    micro(draw, (26, height - 62), "NO ACCOUNT", INK, size=8)
    micro(draw, (26, height - 44), "NO UPLOAD", ACCENT, size=8)
    micro(draw, (26, height - 26), "NO TELEMETRY", MUTED, size=8)

    OUT.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(OUT / "wizard.bmp")
    image.convert("RGB").save(OUT / "wizard-preview.png")


def small() -> None:
    size = 58
    image = paper(size, size)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, size, 5), fill=ACCENT)
    mark(draw, 13, 16, 32)
    OUT.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(OUT / "small.bmp")
    image.convert("RGB").save(OUT / "small-preview.png")


def main() -> int:
    wizard()
    small()
    for path in sorted(OUT.iterdir()):
        print(path.name, path.stat().st_size, "bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
