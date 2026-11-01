from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PAPER = (250, 246, 239)
INK = (27, 22, 19)
MUTED = (107, 98, 87)
ACCENT = (242, 105, 58)
TEAL = (47, 111, 106)
LINE = (226, 217, 204)
DARK = (20, 17, 16)
CREAM = (250, 246, 239)
SOFT = (182, 170, 155)

FONTS = Path("C:/Windows/Fonts")
BOLD = "segoeuib.ttf"
REGULAR = "segoeui.ttf"
MONO = "consola.ttf"

ASSETS = Path("assets")


def font(name: str, size: int):
    return ImageFont.truetype(str(FONTS / name), size)


def spaced(draw, xy, text, face, fill, gap):
    x, y = xy
    for character in text:
        draw.text((x, y), character, font=face, fill=fill)
        x += draw.textlength(character, font=face) + gap
    return x


def canvas(width: int, height: int, color=PAPER, alpha=False):
    mode = "RGBA" if alpha else "RGB"
    background = color + (0,) if alpha else color
    return Image.new(mode, (width, height), background)


def card(draw, box, radius, fill, outline=None, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def banner():
    width, height = 1600, 520
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, width, 8), fill=ACCENT)
    draw.text((96, 78), "Parlo", font=font(BOLD, 104), fill=INK)
    draw.text(
        (96, 210),
        "Speak Portuguese.",
        font=font(BOLD, 46),
        fill=INK,
    )
    draw.text(
        (96, 268),
        "Be heard in English.",
        font=font(BOLD, 46),
        fill=ACCENT,
    )
    draw.text(
        (96, 348),
        "Dictation that writes the sentence you meant, and translation that",
        font=font(REGULAR, 24),
        fill=MUTED,
    )
    draw.text(
        (96, 382),
        "stays on your computer. Nothing uploaded, no account, no telemetry.",
        font=font(REGULAR, 24),
        fill=MUTED,
    )

    pill = (900, 120, 1504, 300)
    card(draw, pill, 90, DARK)
    draw.ellipse((946, 196, 970, 220), fill=ACCENT)
    spaced(draw, (1000, 160), "LISTENING", font(BOLD, 16), SOFT, 3)
    draw.text(
        (1000, 192),
        "«Quero marcar para sexta, quer dizer, segunda.»",
        font=font(REGULAR, 21),
        fill=CREAM,
    )
    draw.text((1000, 232), "2.4 s", font=font(MONO, 19), fill=SOFT)

    chips = (("LOCAL ENGINE", ACCENT), ("YOUR KEYS", TEAL), ("NO TELEMETRY", MUTED))
    x = 96
    for label, color in chips:
        text_width = sum(draw.textlength(character, font=font(BOLD, 15)) + 2 for character in label)
        card(draw, (x, 430, x + text_width + 44, 470), 20, PAPER, LINE, 1)
        spaced(draw, (x + 22, 442), label, font(BOLD, 15), color, 2)
        x += text_width + 66

    image.save(ASSETS / "banner.png")


def island():
    width, height = 1120, 260
    image = canvas(width, height, alpha=True)
    draw = ImageDraw.Draw(image)
    card(draw, (20, 30, 1100, 230), 96, DARK + (238,))
    draw.ellipse((70, 118, 94, 142), fill=ACCENT)
    spaced(draw, (124, 84), "INSERTED", font(BOLD, 15), SOFT, 3)
    draw.text((124, 112), "Vou ao Porto na sexta-feira.", font=font(REGULAR, 30), fill=CREAM)
    draw.text(
        (124, 162),
        "Written into the field you were in.",
        font=font(REGULAR, 19),
        fill=SOFT,
    )
    draw.text((946, 112), "1.9 s", font=font(MONO, 22), fill=SOFT)
    image.save(ASSETS / "island.png")


def pipeline():
    width, height = 1500, 430
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    steps = [
        ("PAVE", "audio in, 16 kHz", INK),
        ("PSTE", "speech to text", INK),
        ("PIE", "the sentence you meant", ACCENT),
        ("FIELD", "typed where you were", TEAL),
    ]
    for index, (title, caption, color) in enumerate(steps):
        x = 60 + index * 350
        card(draw, (x, 120, x + 300, 260), 28, PAPER, LINE, 2)
        draw.text((x + 32, 148), title, font=font(BOLD, 30), fill=color)
        draw.text((x + 32, 196), caption, font=font(REGULAR, 20), fill=MUTED)
        if index < len(steps) - 1:
            arrow = x + 316
            draw.line((arrow, 190, arrow + 22, 190), fill=MUTED, width=3)
            draw.line((arrow + 14, 182, arrow + 24, 190), fill=MUTED, width=3)
            draw.line((arrow + 14, 198, arrow + 24, 190), fill=MUTED, width=3)
    draw.text(
        (60, 300),
        "Rules in microseconds, then a model only if you allowed one, then a check that refuses to change meaning.",
        font=font(REGULAR, 22),
        fill=MUTED,
    )
    draw.text(
        (60, 336),
        "PAVE audio · PSTE transcription · PIE intelligence · PTE translation · PIMO models · PME memory",
        font=font(MONO, 17),
        fill=SOFT,
    )
    image.save(ASSETS / "pipeline.png")


def main():
    ASSETS.mkdir(exist_ok=True)
    banner()
    island()
    pipeline()
    print("wrote assets/banner.png, assets/island.png, assets/pipeline.png")


if __name__ == "__main__":
    main()
