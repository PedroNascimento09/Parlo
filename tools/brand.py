from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PAPER = (250, 246, 239)
CARD = (255, 252, 246)
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
SPOKEN = "«Book a table for Friday, I mean, for Monday.»"
WRITTEN = "Book a table for Monday."


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
    draw.text((96, 210), "Dictation that writes", font=font(BOLD, 46), fill=INK)
    draw.text((96, 268), "what you actually meant.", font=font(BOLD, 46), fill=ACCENT)
    draw.text(
        (96, 348),
        "Translation too, on your own computer. Nothing uploaded,",
        font=font(REGULAR, 24),
        fill=MUTED,
    )
    draw.text(
        (96, 382),
        "no account, no telemetry, no subscription.",
        font=font(REGULAR, 24),
        fill=MUTED,
    )

    card(draw, (880, 112, 1504, 300), 90, DARK)
    draw.ellipse((926, 188, 950, 212), fill=ACCENT)
    spaced(draw, (980, 152), "LISTENING", font(BOLD, 16), SOFT, 3)
    draw.text((980, 184), SPOKEN, font=font(REGULAR, 20), fill=CREAM)
    draw.text((980, 224), "1.9 s", font=font(MONO, 19), fill=SOFT)

    chips = (("LOCAL MODELS", ACCENT), ("YOUR KEYS", TEAL), ("NO TELEMETRY", MUTED))
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
    draw.text((124, 112), WRITTEN, font=font(REGULAR, 32), fill=CREAM)
    draw.text(
        (124, 162),
        "Typed into the field you were in.",
        font=font(REGULAR, 19),
        fill=SOFT,
    )
    draw.text((946, 112), "1.9 s", font=font(MONO, 22), fill=SOFT)
    image.save(ASSETS / "island.png")


STEPS = (
    ("PAVE", "audio in"),
    ("PSTE", "speech to text"),
    ("PIE", "the sentence you meant"),
    ("FIELD", "where you were writing"),
)

CAPTIONS = (
    "Listening on your microphone.",
    "Turning speech into text, on this machine.",
    SPOKEN,
    WRITTEN,
)


def layout(draw, active):
    for index, (title, caption) in enumerate(STEPS):
        x = 60 + index * 290
        lit = index == active
        card(draw, (x, 84, x + 250, 206), 24, CARD if lit else PAPER, ACCENT if lit else LINE, 3 if lit else 2)
        draw.text((x + 24, 106), title, font=font(BOLD, 26), fill=ACCENT if lit else INK)
        draw.text((x + 24, 150), caption, font=font(REGULAR, 17), fill=MUTED)
        if index < len(STEPS) - 1:
            draw.line((x + 260, 145, x + 280, 145), fill=MUTED, width=2)
            draw.line((x + 273, 138, x + 281, 145), fill=MUTED, width=2)
            draw.line((x + 273, 152, x + 281, 145), fill=MUTED, width=2)


def flow():
    width, height = 1200, 360
    frames = []
    for phase in range(len(STEPS)):
        for tick in range(6):
            image = canvas(width, height)
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, width, 6), fill=ACCENT)
            layout(draw, phase)
            text = CAPTIONS[phase]
            if phase == len(STEPS) - 1:
                visible = max(1, round(len(text) * (tick + 1) / 6))
                text = text[:visible] + ("▌" if tick < 5 else "")
            draw.text((60, 260), text, font=font(REGULAR, 25), fill=INK)
            draw.text(
                (60, 312),
                "Rules in microseconds · a model only if you allowed one · a check that refuses to change meaning",
                font=font(REGULAR, 18),
                fill=SOFT,
            )
            frames.append(image)
    frames[0].save(
        ASSETS / "flow.gif",
        save_all=True,
        append_images=frames[1:],
        duration=140,
        loop=0,
        optimize=True,
    )


def pipeline():
    width, height = 1500, 430
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    for index, (title, caption) in enumerate(STEPS):
        x = 60 + index * 350
        card(draw, (x, 110, x + 300, 250), 28, PAPER, LINE, 2)
        draw.text((x + 32, 138), title, font=font(BOLD, 30), fill=ACCENT if title == "PIE" else INK)
        draw.text((x + 32, 186), caption, font=font(REGULAR, 20), fill=MUTED)
        if index < len(STEPS) - 1:
            arrow = x + 316
            draw.line((arrow, 180, arrow + 22, 180), fill=MUTED, width=3)
            draw.line((arrow + 14, 172, arrow + 24, 180), fill=MUTED, width=3)
            draw.line((arrow + 14, 188, arrow + 24, 180), fill=MUTED, width=3)
    draw.text(
        (60, 296),
        "Rules in microseconds, then a model only if you allowed one, then a check that refuses to change meaning.",
        font=font(REGULAR, 22),
        fill=MUTED,
    )
    draw.text(
        (60, 336),
        "Languages today: Portuguese and English, both directions. PAVE audio · PSTE transcription · PIE intelligence · PTE translation",
        font=font(MONO, 15),
        fill=SOFT,
    )
    image.save(ASSETS / "pipeline.png")


def main():
    ASSETS.mkdir(exist_ok=True)
    banner()
    island()
    pipeline()
    flow()
    print("wrote assets/banner.png, assets/island.png, assets/pipeline.png, assets/flow.gif")


if __name__ == "__main__":
    main()
