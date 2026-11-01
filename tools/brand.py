from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PAPER = (247, 244, 237)
CARD = (252, 250, 245)
INK = (18, 16, 14)
MUTED = (112, 105, 96)
RULE = (214, 206, 193)
ACCENT = (176, 60, 30)
TEAL = (30, 79, 74)
DARK = (20, 18, 16)
CREAM = (247, 244, 237)
SOFT = (168, 158, 146)

FONTS = Path("C:/Windows/Fonts")
SERIF = "georgiab.ttf"
BODY = "segoeui.ttf"
MICRO = "bahnschrift.ttf"
MONO = "consola.ttf"

ASSETS = Path("assets")
SPOKEN = "«Book a table for Friday, I mean, for Monday.»"
WRITTEN = "Book a table for Monday."
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
SURFACES = (
    ("PARLO DICTATE", "Speak once and the sentence is typed into the window you were in.", "AVAILABLE"),
    ("PARLO LIVE", "Keeps translating as you speak, one window at a time.", "AVAILABLE"),
    ("PARLO MEETINGS", "Records the room and keeps every line with its speaker.", "AVAILABLE"),
    ("PARLO TRANSLATE", "A file, a call or a page, in the other language.", "AVAILABLE"),
)


def font(name: str, size: int):
    return ImageFont.truetype(str(FONTS / name), size)


def canvas(width: int, height: int, color=PAPER, alpha=False):
    mode = "RGBA" if alpha else "RGB"
    return Image.new(mode, (width, height), color + (0,) if alpha else color)


def micro(draw, xy, text, fill, size=13, gap=2.4, face=MICRO):
    x, y = xy
    label = font(face, size)
    for character in text:
        draw.text((x, y), character, font=label, fill=fill)
        x += draw.textlength(character, font=label) + gap
    return x


def mark(draw, x, y, size, ink=INK, accent=ACCENT):
    radius = size / 2
    centre = (x + radius, y + radius)
    draw.ellipse(
        (centre[0] - size * 0.09, centre[1] - size * 0.09, centre[0] + size * 0.09, centre[1] + size * 0.09),
        fill=ink,
    )
    for index, factor in enumerate((0.34, 0.58, 0.84)):
        box = (
            centre[0] - size * factor,
            centre[1] - size * factor,
            centre[0] + size * factor,
            centre[1] + size * factor,
        )
        draw.arc(box, start=-58, end=58, fill=accent if index == 2 else ink, width=max(2, int(size * 0.035)))


def rule(draw, y, x0, x1, color=RULE, width=1):
    draw.line((x0, y, x1, y), fill=color, width=width)


def banner():
    width, height = 1600, 580
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, width, 5), fill=ACCENT)
    mark(draw, 108, 64, 28)
    micro(draw, (172, 76), "PARLO", INK, 14, 3.4)
    micro(draw, (1290, 76), "NO ACCOUNT · NO TELEMETRY", MUTED, 12)

    draw.text((108, 168), "Type with your voice,", font=font(SERIF, 72), fill=INK)
    draw.text((108, 254), "without leaving", font=font(SERIF, 72), fill=INK)
    draw.text((108, 340), "your computer.", font=font(SERIF, 72), fill=ACCENT)

    draw.text(
        (112, 448),
        "Speech becomes clean text, and text becomes another language,",
        font=font(BODY, 21),
        fill=MUTED,
    )
    draw.text(
        (112, 480),
        "with every model on your own disk.",
        font=font(BODY, 21),
        fill=MUTED,
    )

    panel = (1000, 150, 1492, 486)
    draw.rectangle(panel, fill=CARD, outline=RULE, width=1)
    micro(draw, (1032, 182), "INPUT", MUTED, 12)
    draw.text((1032, 206), "Book a table for Friday,", font=font(MONO, 17), fill=INK)
    draw.text((1032, 232), "I mean, for Monday.", font=font(MONO, 17), fill=INK)
    rule(draw, 268, 1032, 1460)
    micro(draw, (1032, 290), "ENGINE", MUTED, 12)
    draw.text((1032, 314), "PIE · rules only · 61.9 µs", font=font(MONO, 17), fill=INK)
    rule(draw, 350, 1032, 1460)
    micro(draw, (1032, 372), "OUTPUT", MUTED, 12)
    draw.text((1032, 396), WRITTEN, font=font(MONO, 20), fill=ACCENT)
    micro(draw, (1032, 446), "NOWHERE BUT HERE", MUTED, 12)

    rule(draw, 520, 110, 1490)
    columns = (
        ("DICTATE", "anywhere you type"),
        ("LIVE", "translated as you speak"),
        ("MEETINGS", "every line, with a speaker"),
        ("TRANSLATE", "files, calls, pages"),
    )
    x = 110
    for title, caption in columns:
        micro(draw, (x, 542), title, INK, 12)
        draw.text((x + 92, 540), caption, font=font(BODY, 15), fill=MUTED)
        x += 372

    image.save(ASSETS / "banner.png")


def surfaces():
    width, height = 1500, 400
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    rule(draw, 96, 60, 1440)
    for index, (title, sentence, state) in enumerate(SURFACES):
        x = 60 + index * 372
        micro(draw, (x, 124), title, INK, 13)
        words = sentence.split(" ")
        lines = []
        current = ""
        for word in words:
            candidate = (current + " " + word).strip()
            if draw.textlength(candidate, font=font(BODY, 18)) > 292:
                lines.append(current)
                current = word
            else:
                current = candidate
        lines.append(current)
        for line_index, line in enumerate(lines):
            draw.text((x, 162 + line_index * 28), line, font=font(BODY, 18), fill=MUTED)
        micro(draw, (x, 306), state, ACCENT, 11)
        if index < len(SURFACES) - 1:
            draw.line((x + 330, 124, x + 330, 330), fill=RULE, width=1)
    rule(draw, 352, 60, 1440)
    image.save(ASSETS / "surfaces.png")


def island():
    width, height = 1120, 240
    image = canvas(width, height, alpha=True)
    draw = ImageDraw.Draw(image)
    draw.rounded_rectangle((20, 24, 1100, 216), radius=92, fill=DARK + (240,))
    draw.ellipse((70, 106, 90, 126), fill=ACCENT)
    micro(draw, (122, 74), "INSERTED", SOFT, 12)
    draw.text((122, 100), WRITTEN, font=font(BODY, 31), fill=CREAM)
    draw.text((122, 148), "Typed into the field you were in.", font=font(BODY, 18), fill=SOFT)
    draw.text((952, 100), "1.9 s", font=font(MONO, 21), fill=SOFT)
    image.save(ASSETS / "island.png")


def layout(draw, active):
    for index, (title, caption) in enumerate(STEPS):
        x = 60 + index * 286
        lit = index == active
        draw.rectangle(
            (x, 78, x + 246, 198),
            fill=CARD if lit else PAPER,
            outline=ACCENT if lit else RULE,
            width=1,
        )
        micro(draw, (x + 22, 100), title, ACCENT if lit else INK, 13)
        draw.text((x + 22, 134), caption, font=font(BODY, 17), fill=MUTED)
        if index < len(STEPS) - 1:
            draw.line((x + 256, 138, x + 276, 138), fill=RULE, width=1)
            draw.line((x + 270, 133, x + 277, 138), fill=RULE, width=1)
            draw.line((x + 270, 143, x + 277, 138), fill=RULE, width=1)


def flow():
    width, height = 1200, 330
    frames = []
    for phase in range(len(STEPS)):
        for tick in range(6):
            image = canvas(width, height)
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, width, 4), fill=ACCENT)
            layout(draw, phase)
            total = len(STEPS) * 6
            filled = int(1140 * (phase * 6 + tick + 1) / total)
            draw.line((60, 218, 1140, 218), fill=RULE, width=2)
            if filled > 4:
                draw.line((60, 218, 60 + filled, 218), fill=ACCENT, width=2)
            text = CAPTIONS[phase]
            if phase == len(STEPS) - 1:
                visible = max(1, round(len(text) * (tick + 1) / 6))
                text = text[:visible] + ("▌" if tick < 5 else "")
            draw.text((60, 246), text, font=font(BODY, 24), fill=INK)
            micro(draw, (60, 292), "RULES IN MICROSECONDS · A MODEL ONLY IF YOU ALLOWED ONE", SOFT, 10, 1.8)
            frames.append(image)
    frames[0].save(
        ASSETS / "flow.gif",
        save_all=True,
        append_images=frames[1:],
        duration=140,
        loop=0,
        optimize=True,
    )


def mark_file():
    size = 512
    image = canvas(size, size, alpha=True)
    draw = ImageDraw.Draw(image)
    mark(draw, 76, 76, 360)
    image.save(ASSETS / "mark.png")


def main():
    ASSETS.mkdir(exist_ok=True)
    banner()
    surfaces()
    island()
    flow()
    mark_file()
    print("wrote banner, surfaces, island, flow.gif and mark")


if __name__ == "__main__":
    main()
