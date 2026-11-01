from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PAPER = (246, 242, 234)
CARD = (253, 251, 246)
INK = (21, 18, 15)
MUTED = (110, 102, 96)
FAINT = (168, 158, 146)
RULE = (219, 210, 196)
DOT = (236, 229, 217)
ACCENT = (194, 69, 42)
TEAL = (23, 69, 63)
NIGHT = (22, 19, 16)
CREAM = (247, 243, 235)

FONTS = Path("C:/Windows/Fonts")
SERIF = "georgiab.ttf"
BODY = "segoeui.ttf"
MICRO = "bahnschrift.ttf"
MONO = "consola.ttf"

ASSETS = Path("assets")
WRITTEN = "Book a table for Monday."
SPOKEN = "Book a table for Friday, I mean, for Monday."
STEPS = (
    ("AUDIO", "PAVE", "microphone in"),
    ("TRANSCRIBE", "PSTE", "speech becomes text"),
    ("CLEAN UP", "PIE", "the sentence you meant"),
    ("WRITE", "YOUR APP", "typed where you were"),
)
LIVE_PAIRS = (
    ("A reunião começa às nove.", "The meeting starts at nine."),
    ("Depois seguimos para o Porto.", "Then we go on to Porto."),
    ("Traz o contrato assinado.", "Bring the signed contract."),
    ("Precisamos de mais duas cadeiras.", "We need two more chairs."),
    ("Digo-te o preço na sexta.", "I'll tell you the price on Friday."),
)
MEETING_ROWS = (
    ("ANA", "12:14:02", "Let's start with the budget."),
    ("RUI", "12:14:07", "The numbers are on the second page."),
    ("ANA", "12:14:12", "That delta is the Lisbon office."),
    ("RUI", "12:14:19", "Then we hold the hire until spring."),
    ("ANA", "12:14:26", "Agreed. Send me the revised sheet."),
    ("RUI", "12:14:31", "Before Friday, with the two scenarios."),
)
WAVE = (10, 18, 30, 16, 38, 52, 34, 24, 44, 20, 14, 28, 40, 22, 32, 12, 26, 46, 30, 18, 36, 24, 16, 34)


def font(name: str, size: int):
    return ImageFont.truetype(str(FONTS / name), size)


def paper(width: int, height: int, step=24):
    image = Image.new("RGB", (width, height), PAPER)
    draw = ImageDraw.Draw(image)
    for x in range(step, width, step):
        for y in range(step, height, step):
            draw.point((x, y), fill=DOT)
    return image.convert("RGBA")


def micro(draw, xy, text, fill, size=13, gap=2.4, face=MICRO):
    x, y = xy
    label = font(face, size)
    for character in text:
        draw.text((x, y), character, font=label, fill=fill)
        x += draw.textlength(character, font=label) + gap
    return x


def text(draw, xy, content, size=18, fill=INK, face=BODY):
    draw.text(xy, content, font=font(face, size), fill=fill)


def wrap(draw, content, face, size, width):
    lines, current = [], ""
    for word in content.split(" "):
        candidate = (current + " " + word).strip()
        if draw.textlength(candidate, font=font(face, size)) > width and current:
            lines.append(current)
            current = word
        else:
            current = candidate
    if current:
        lines.append(current)
    return lines


def mark(draw, x, y, size, ink=INK, accent=ACCENT):
    centre = (x + size / 2, y + size / 2)
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


def shadow(image, box, radius, spread=26, strength=38):
    layer = Image.new("RGBA", image.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)
    offsets = list(range(spread, 0, -4))
    for index, offset in enumerate(offsets):
        alpha = max(2, int(strength * (index + 1) / len(offsets)))
        draw.rounded_rectangle(
            (box[0] - offset, box[1] - offset + 12, box[2] + offset, box[3] + offset + 12),
            radius=radius + offset,
            fill=(24, 20, 16, alpha),
        )
    image.alpha_composite(layer)


def card(draw, box, fill=CARD, radius=14, outline=RULE):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=1)


def chip(draw, xy, label, fill=TEAL, ink=CREAM, size=11, face=MICRO, gap=2.2):
    probe = font(face, size)
    width = sum(draw.textlength(character, font=probe) + gap for character in label) + 26
    draw.rounded_rectangle((xy[0], xy[1], xy[0] + width, xy[1] + 30), radius=15, fill=fill)
    micro(draw, (xy[0] + 13, xy[1] + 9 + (1 if face == BODY else 0)), label, ink, size, gap, face)


def caret(draw, xy, height, fill=ACCENT):
    draw.rectangle((xy[0], xy[1], xy[0] + 2, xy[1] + height), fill=fill)


def wave(draw, xy, width, height, progress=1.0, fill=ACCENT, bars=24):
    step = width / bars
    for index in range(bars):
        level = WAVE[index % len(WAVE)] / 52 * height
        x = xy[0] + index * step
        lit = index / bars <= progress
        draw.rounded_rectangle(
            (x, xy[1] + (height - level) / 2, x + 3, xy[1] + (height + level) / 2),
            radius=2,
            fill=fill if lit else RULE,
        )


def island(draw, box, sentence, state="INSERTED", sub="Typed into the field you were in.", elapsed="1.9 s"):
    radius = (box[3] - box[1]) / 2
    draw.rounded_rectangle(box, radius=radius, fill=NIGHT)
    draw.ellipse((box[0] + 44, (box[1] + box[3]) / 2 - 9, box[0] + 62, (box[1] + box[3]) / 2 + 9), fill=ACCENT)
    micro(draw, (box[0] + 96, box[1] + 40), state, FAINT, 12)
    text(draw, (box[0] + 96, box[1] + 64), sentence, 29, CREAM)
    if sub:
        text(draw, (box[0] + 96, box[1] + 108), sub, 17, FAINT)
    if elapsed:
        label = font(MONO, 20)
        draw.text((box[2] - 70 - draw.textlength(elapsed, font=label), box[1] + 66), elapsed, font=label, fill=FAINT)


def banner():
    width, height = 1600, 610
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, width, 6), fill=ACCENT)
    mark(draw, 96, 68, 30)
    micro(draw, (160, 80), "PARLO", INK, 14, 3.4)
    micro(draw, (1042, 82), "NO ACCOUNT · NO TELEMETRY · NO SUBSCRIPTION", MUTED, 11, 2.0)

    text(draw, (94, 176), "Type with your voice,", 64, INK, SERIF)
    text(draw, (94, 246), "without leaving", 64, INK, SERIF)
    text(draw, (94, 316), "your computer.", 64, ACCENT, SERIF)
    for index, line in enumerate(("Speech becomes clean text, and text becomes", "another language, with every model on your disk.")):
        text(draw, (98, 424 + index * 30), line, 20, MUTED)

    box = (900, 196, 1524, 360)
    shadow(image, box, 82, 16, 22)
    draw = ImageDraw.Draw(image)
    island(draw, box, WRITTEN)

    micro(draw, (900, 392), "61.9 MS WITHOUT A MODEL · 420.7 MS WITH RECOGNITION", FAINT, 10, 1.8)
    draw.line((896, 462, 1504, 462), fill=RULE, width=1)
    draw.line((96, 508, 1504, 508), fill=RULE, width=1)
    for index, (label, caption) in enumerate(
        (("DICTATE", "anywhere you type"), ("LIVE", "translated as you speak"), ("MEETINGS", "every line, one speaker"), ("TRANSLATE", "files, calls, pages"))
    ):
        x = 96 + index * 232
        micro(draw, (x, 538), label, INK, 11, 2.2)
        text(draw, (x, 562), caption, 14, MUTED)
        if index:
            draw.line((x - 26, 534, x - 26, 582), fill=RULE, width=1)

    image.convert("RGB").save(ASSETS / "banner.png")


def dictate():
    width, height = 1440, 770
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (80, 62), "PARLO DICTATE", ACCENT, 12, 2.4)
    text(draw, (78, 88), "Press once. Keep working.", 42, INK, SERIF)
    for index, line in enumerate(
        ("The island listens while you talk, then writes the sentence into the field you were in.", "No window to switch to, no text to paste, no reason to stop typing.")
    ):
        text(draw, (80, 152 + index * 28), line, 18, MUTED)

    window = (80, 240, 1360, 600)
    shadow(image, window, 14, 22, 30)
    draw = ImageDraw.Draw(image)
    card(draw, window)
    draw.line((80, 292, 1360, 292), fill=RULE, width=1)
    text(draw, (112, 258), "Notes — untitled", 15, MUTED)
    micro(draw, (1180, 262), "UNSAVED", FAINT, 10, 2.0)

    text(draw, (112, 330), "Friday lunch with the team.", 22, INK)
    text(draw, (112, 372), WRITTEN, 22, ACCENT)
    caret(draw, (112 + draw.textlength(WRITTEN, font=font(BODY, 22)) + 4, 370), 28)
    for index, line in enumerate(("Confirm the room and the time, then send the", "calendar invite to both offices.")):
        text(draw, (112, 424 + index * 30), line, 18, FAINT)

    box = (620, 520, 1360, 668)
    shadow(image, box, 74, 16, 24)
    draw = ImageDraw.Draw(image)
    island(draw, box, WRITTEN)

    micro(draw, (82, 712), "CTRL ALT D", INK, 11, 2.2)
    text(draw, (190, 706), "shows the island from anywhere, including full screen apps.", 15, MUTED)
    image.convert("RGB").save(ASSETS / "dictate.png")


def live_frame(pairs, reveal, pulse, width=1200, height=560):
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (56, 44), "PARLO LIVE", ACCENT, 12, 2.4)
    text(draw, (54, 68), "Read the other language while you speak.", 34, INK, SERIF)

    chip(draw, (56, 128), "PT → EN", TEAL, CREAM, 11, BODY, 1.0)
    chip(draw, (188, 128), "STREAMING", ACCENT if pulse else (188, 178, 164))

    left = (56, 190, 584, 512)
    right = (616, 190, 1144, 512)
    for box, label in ((left, "YOU SAY"), (right, "PARLO SAYS")):
        card(draw, box)
        micro(draw, (box[0] + 30, box[1] + 24), label, FAINT, 11, 2.2)
        draw.line((box[0] + 1, box[1] + 62, box[2] - 1, box[1] + 62), fill=RULE, width=1)

    for index in range(min(pairs, len(LIVE_PAIRS))):
        y = 264 + index * 62
        text(draw, (left[0] + 30, y), LIVE_PAIRS[index][0], 19, INK)
    for index in range(min(reveal, len(LIVE_PAIRS))):
        y = 264 + index * 62
        text(draw, (right[0] + 30, y), LIVE_PAIRS[index][1], 19, TEAL)

    wave(draw, (56 + 30, 452), 498, 34, min(1.0, reveal / max(1, len(LIVE_PAIRS))), ACCENT)
    wave(draw, (616 + 30, 452), 498, 34, min(1.0, reveal / max(1, len(LIVE_PAIRS))), TEAL)
    micro(draw, (56, 520), "0.9 S BEHIND · SPOKEN BACK IN THE OTHER LANGUAGE", FAINT, 10, 1.8)
    return image


def live():
    frames, durations = [], []
    for index in range(len(LIVE_PAIRS)):
        for tick in range(7):
            reveal = index + 1 if tick >= 3 else index
            frames.append(live_frame(index + 1, reveal, tick % 2 == 0))
            durations.append(190)
    for tick in range(10):
        frames.append(live_frame(len(LIVE_PAIRS), len(LIVE_PAIRS), tick % 2 == 0))
        durations.append(240)
    frames[0].save(
        ASSETS / "live.gif",
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )


def meetings_frame(rows, playhead, width=1200, height=600):
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (56, 44), "PARLO MEETINGS", ACCENT, 12, 2.4)
    text(draw, (54, 68), "Every line, with a name and a time.", 34, INK, SERIF)
    chip(draw, (56, 128), "THURSDAY REVIEW · 42 MIN", TEAL)
    chip(draw, (330, 128), "2 SPEAKERS", (188, 178, 164), INK)

    box = (56, 190, 1144, 540)
    card(draw, box)
    micro(draw, (box[0] + 30, box[1] + 24), "TRANSCRIPT", FAINT, 11, 2.2)
    micro(draw, (box[2] - 250, box[1] + 24), "SAVED IN YOUR FOLDER", FAINT, 10, 2.0)
    draw.line((box[0] + 1, box[1] + 62, box[2] - 1, box[1] + 62), fill=RULE, width=1)

    for index in range(min(rows, len(MEETING_ROWS))):
        speaker, moment, line = MEETING_ROWS[index]
        y = 272 + index * 42
        text(draw, (box[0] + 30, y), moment, 14, FAINT, MONO)
        colour = ACCENT if speaker == "ANA" else TEAL
        draw.rounded_rectangle((box[0] + 126, y - 2, box[0] + 190, y + 22), radius=12, fill=colour)
        micro(draw, (box[0] + 138, y + 5), speaker, CREAM, 10, 2.0)
        text(draw, (box[0] + 208, y), line, 17, INK)

    draw.line((box[0] + 30, 508, box[2] - 30, 508), fill=RULE, width=2)
    draw.line((box[0] + 30, 508, box[0] + 30 + playhead, 508), fill=ACCENT, width=2)
    micro(draw, (56, 566), "RECORDED FROM THE ROOM · NOTHING UPLOADED", FAINT, 10, 1.8)
    return image


def meetings():
    frames, durations = [], []
    for index in range(len(MEETING_ROWS)):
        for tick in range(6):
            frames.append(meetings_frame(index + 1, 30 + index * 170 + tick * 28))
            durations.append(200)
    for tick in range(8):
        frames.append(meetings_frame(len(MEETING_ROWS), 1020, tick % 2 == 0))
        durations.append(260)
    frames[0].save(
        ASSETS / "meetings.gif",
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )


def translate():
    width, height = 1440, 800
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (80, 62), "PARLO TRANSLATE", ACCENT, 12, 2.4)
    text(draw, (78, 88), "A file, in the other language.", 42, INK, SERIF)
    for index, line in enumerate(
        ("Drop an audio or video file and get it back dubbed, with subtitles written beside it.", "It runs at home: the recording is never uploaded to a server.")
    ):
        text(draw, (80, 152 + index * 28), line, 18, MUTED)

    left = (80, 240, 660, 660)
    right = (700, 240, 1360, 660)
    for box in (left, right):
        shadow(image, box, 14, 18, 24)
    draw = ImageDraw.Draw(image)
    card(draw, left)
    card(draw, right)

    micro(draw, (112, 272), "SOURCE", FAINT, 11, 2.2)
    text(draw, (110, 300), "interview-2026-10-31.mp4", 19, INK)
    text(draw, (112, 336), "42 min 18 s · 318 MB", 15, MUTED)
    chip(draw, (112, 380), "PT → EN", TEAL, CREAM, 11, BODY, 1.0)
    chip(draw, (248, 380), ".SRT", (188, 178, 164), INK)
    draw.rounded_rectangle((112, 452, 628, 462), radius=5, fill=RULE)
    draw.rounded_rectangle((112, 452, 112 + 350, 462), radius=5, fill=ACCENT)
    micro(draw, (112, 486), "68% · 12 MIN LEFT", ACCENT, 11, 2.0)
    wave(draw, (112, 530), 516, 44, 0.68, TEAL)
    micro(draw, (112, 604), "DUBBED AUDIO PLAYS HERE WHEN IT ENDS", FAINT, 10, 1.8)

    micro(draw, (732, 272), "SUBTITLES", FAINT, 11, 2.2)
    preview = (
        ("00:00:04,120", "The meeting starts at nine."),
        ("00:00:09,480", "Then we go on to Porto."),
        ("00:00:14,900", "Bring the signed contract."),
        ("00:00:21,260", "We need two more chairs."),
        ("00:00:26,010", "I'll tell you the price on Friday."),
    )
    for index, (moment, line) in enumerate(preview):
        y = 306 + index * 58
        text(draw, (732, y), moment, 13, FAINT, MONO)
        text(draw, (732, y + 20), line, 17, INK)
    micro(draw, (732, 604), "SAME TIMING AS THE SOURCE · NEVER UPLOADED", FAINT, 10, 1.8)

    image.convert("RGB").save(ASSETS / "translate.png")


def layout(draw, active):
    for index, (label, engine, caption) in enumerate(STEPS):
        x = 60 + index * 286
        lit = index == active
        draw.rounded_rectangle((x, 74, x + 246, 206), radius=10, fill=CARD if lit else PAPER, outline=ACCENT if lit else RULE, width=1)
        micro(draw, (x + 22, 96), label, ACCENT if lit else INK, 14)
        micro(draw, (x + 22, 124), engine, MUTED, 10, 2.0)
        text(draw, (x + 22, 152), caption, 16, MUTED)
        if index < len(STEPS) - 1:
            draw.line((x + 256, 140, x + 276, 140), fill=RULE, width=1)
            draw.line((x + 270, 135, x + 277, 140), fill=RULE, width=1)
            draw.line((x + 270, 145, x + 277, 140), fill=RULE, width=1)


def flow():
    width, height = 1200, 380
    frames, durations = [], []
    plan = ((0, 6), (1, 6), (2, 14), (3, 12))
    counter = 0
    for phase, ticks in plan:
        for tick in range(ticks):
            counter += 1
            image = paper(width, height)
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, width, 4), fill=ACCENT)
            layout(draw, phase)
            filled = int(1140 * counter / 38)
            draw.line((60, 232, 1140, 232), fill=RULE, width=2)
            if filled > 4:
                draw.line((60, 232, 60 + filled, 232), fill=ACCENT, width=2)
            if phase == 0:
                text(draw, (60, 262), "Listening. Audio never leaves this machine.", 23, INK)
            elif phase == 1:
                text(draw, (60, 262), "Recognising the words on the device.", 23, INK)
            elif phase == 2:
                text(draw, (60, 258), "«" + SPOKEN + "»", 22, MUTED)
                if tick >= 6:
                    text(draw, (60, 298), "→ " + WRITTEN, 25, ACCENT)
                else:
                    micro(draw, (60, 302), "FIXING THE SELF-CORRECTION", FAINT, 11, 1.8)
            else:
                visible = max(1, round(len(WRITTEN) * (tick + 1) / ticks))
                text(draw, (60, 262), WRITTEN[:visible] + ("▌" if tick < ticks - 1 else ""), 25, INK)
                micro(draw, (60, 306), "TYPED INTO THE WINDOW YOU WERE IN", FAINT, 11, 1.8)
            frames.append(image.convert("RGB"))
            durations.append(150 if phase < 2 else 160)
    for tick in range(10):
        image = paper(width, height)
        draw = ImageDraw.Draw(image)
        draw.rectangle((0, 0, width, 4), fill=ACCENT)
        layout(draw, 3)
        draw.line((60, 232, 1140, 232), fill=ACCENT, width=2)
        text(draw, (60, 262), WRITTEN + ("▌" if tick % 2 == 0 else ""), 25, INK)
        micro(draw, (60, 306), "RULES IN MICROSECONDS · NO MODEL, NO NETWORK, NOTHING UPLOADED", FAINT, 11, 1.8)
        frames.append(image.convert("RGB"))
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
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, width, 6), fill=ACCENT)
    mark(draw, 88, 80, 32)
    micro(draw, (148, 92), "PARLO", INK, 14, 3.4)
    text(draw, (86, 206), "Type with your voice,", 58, INK, SERIF)
    text(draw, (86, 276), "without leaving", 58, INK, SERIF)
    text(draw, (86, 346), "your computer.", 58, ACCENT, SERIF)
    text(draw, (90, 452), "Offline dictation and translation for Windows, macOS and Linux.", 21, MUTED)
    text(draw, (90, 488), "No account. No telemetry. No subscription.", 21, MUTED)
    draw.line((88, 560, 1192, 560), fill=RULE, width=1)
    micro(draw, (88, 582), "DICTATE · LIVE · MEETINGS · TRANSLATE", MUTED, 11, 2.2)
    image.convert("RGB").save(ASSETS / "social.png")


def mark_file():
    size = 512
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    mark(draw, 76, 76, 360)
    image.save(ASSETS / "mark.png")


def main():
    ASSETS.mkdir(exist_ok=True)
    banner()
    dictate()
    live()
    meetings()
    translate()
    flow()
    social()
    mark_file()
    print("wrote banner, dictate, live.gif, meetings.gif, translate, flow.gif, social and mark")


if __name__ == "__main__":
    main()
