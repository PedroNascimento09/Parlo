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
    ("AUDIO", "PAVE", "microphone in"),
    ("TRANSCRIBE", "PSTE", "speech becomes text"),
    ("CLEAN UP", "PIE", "the sentence you meant"),
    ("WRITE", "YOUR APP", "typed where you were"),
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

    draw.rectangle((1000, 150, 1492, 486), fill=CARD, outline=RULE, width=1)
    micro(draw, (1032, 182), "YOU SAY", MUTED, 12)
    draw.text((1032, 206), "Book a table for Friday,", font=font(MONO, 17), fill=INK)
    draw.text((1032, 232), "I mean, for Monday.", font=font(MONO, 17), fill=INK)
    rule(draw, 268, 1032, 1460)
    micro(draw, (1032, 290), "CLEANUP", MUTED, 12)
    draw.text((1032, 314), "rules only · 61.9 µs · offline", font=font(MONO, 17), fill=INK)
    rule(draw, 350, 1032, 1460)
    micro(draw, (1032, 372), "PARLO WRITES", MUTED, 12)
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
    for index, (label, engine, caption) in enumerate(STEPS):
        x = 60 + index * 286
        lit = index == active
        draw.rectangle((x, 74, x + 246, 206), fill=CARD if lit else PAPER, outline=ACCENT if lit else RULE, width=1)
        micro(draw, (x + 22, 96), label, ACCENT if lit else INK, 14)
        micro(draw, (x + 22, 124), engine, MUTED, 10, 2.0)
        draw.text((x + 22, 152), caption, font=font(BODY, 16), fill=MUTED)
        if index < len(STEPS) - 1:
            draw.line((x + 256, 140, x + 276, 140), fill=RULE, width=1)
            draw.line((x + 270, 135, x + 277, 140), fill=RULE, width=1)
            draw.line((x + 270, 145, x + 277, 140), fill=RULE, width=1)


def flow():
    width, height = 1200, 360
    frames = []
    durations = []
    plan = ((0, 6), (1, 6), (2, 14), (3, 12))
    total = 38
    counter = 0
    for phase, ticks in plan:
        for tick in range(ticks):
            counter += 1
            image = canvas(width, height)
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, width, 4), fill=ACCENT)
            layout(draw, phase)
            filled = int(1140 * counter / total)
            draw.line((60, 226, 1140, 226), fill=RULE, width=2)
            if filled > 4:
                draw.line((60, 226, 60 + filled, 226), fill=ACCENT, width=2)
            if phase == 0:
                draw.text((60, 256), "Listening. Audio never leaves this machine.", font=font(BODY, 24), fill=INK)
            elif phase == 1:
                draw.text((60, 256), "Recognising the words on the device.", font=font(BODY, 24), fill=INK)
            elif phase == 2:
                draw.text((60, 252), SPOKEN, font=font(BODY, 24), fill=MUTED)
                if tick >= 6:
                    draw.text((60, 292), "→ " + WRITTEN, font=font(BODY, 26), fill=ACCENT)
                else:
                    micro(draw, (60, 296), "FIXING THE SELF-CORRECTION", SOFT, 11, 1.8)
            else:
                visible = max(1, round(len(WRITTEN) * (tick + 1) / ticks))
                draw.text((60, 256), WRITTEN[:visible] + ("▌" if tick < ticks - 1 else ""), font=font(BODY, 26), fill=INK)
                micro(draw, (60, 300), "TYPED INTO THE WINDOW YOU WERE IN", SOFT, 11, 1.8)
            frames.append(image)
            durations.append(150 if phase < 2 else 160)
    for tick in range(10):
        image = canvas(width, height)
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, width, 4), fill=ACCENT)
        layout(draw, 3)
        draw.line((60, 226, 1140, 226), fill=RULE, width=2)
        draw.line((60, 226, 1140, 226), fill=ACCENT, width=2)
        draw.text((60, 256), WRITTEN + ("▌" if tick % 2 == 0 else ""), font=font(BODY, 26), fill=INK)
        micro(draw, (60, 300), "RULES IN MICROSECONDS · NO MODEL, NO NETWORK, NOTHING UPLOADED", SOFT, 11, 1.8)
        frames.append(image)
        durations.append(260)
    frames[0].save(
        ASSETS / "flow.gif",
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )


def social():
    width, height = 1280, 640
    image = canvas(width, height)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, width, 6), fill=ACCENT)
    mark(draw, 88, 84, 34)
    micro(draw, (140, 96), "PARLO", INK, 15, 3.4)
    draw.text((86, 214), "Type with your voice,", font=font(SERIF, 62), fill=INK)
    draw.text((86, 288), "without leaving", font=font(SERIF, 62), fill=INK)
    draw.text((86, 362), "your computer.", font=font(SERIF, 62), fill=ACCENT)
    draw.text(
        (90, 470),
        "Offline dictation and translation for Windows, macOS and Linux.",
        font=font(BODY, 22),
        fill=MUTED,
    )
    draw.text((90, 508), "No account. No telemetry. No subscription.", font=font(BODY, 22), fill=MUTED)
    rule(draw, 574, 88, 1192)
    micro(draw, (88, 594), "PARLO DICTATE · PARLO LIVE · PARLO MEETINGS · PARLO TRANSLATE", MUTED, 11, 2.0)
    image.save(ASSETS / "social.png")


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
    social()
    mark_file()
    print("wrote banner, surfaces, island, flow.gif, social and mark")


if __name__ == "__main__":
    main()
