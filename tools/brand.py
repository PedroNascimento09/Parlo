import math
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
CREAM = (247, 243, 235)

FONTS = Path("C:/Windows/Fonts")
SERIF = "georgiab.ttf"
BODY = "segoeui.ttf"
MICRO = "bahnschrift.ttf"
MONO = "consola.ttf"

ASSETS = Path("assets")
SPOKEN_A = "Book a table for Friday,"
SPOKEN_B = "I mean, for Monday."
WRITTEN = "Book a table for Monday."
SECOND = "Send it to the team before Friday."
MEETING_ROWS = (
    ("SPEAKER 1", "12:14:02", "Let's start with the budget."),
    ("SPEAKER 2", "12:14:07", "The numbers are on the second page."),
    ("SPEAKER 1", "12:14:12", "That delta is the second office."),
    ("SPEAKER 2", "12:14:19", "Then we hold the hire until spring."),
    ("SPEAKER 1", "12:14:26", "Agreed. Send me the revised sheet."),
    ("SPEAKER 2", "12:14:31", "Before Friday, with the two scenarios."),
)
WAVE = (8, 16, 26, 14, 34, 46, 30, 20, 40, 18, 12, 24, 36, 20, 28, 10, 22, 42, 26, 16, 32, 22, 14, 30)


def font(name: str, size: int):
    return ImageFont.truetype(str(FONTS / name), size)


def mix(a, b, t):
    t = max(0.0, min(1.0, t))
    return tuple(int(a[index] + (b[index] - a[index]) * t) for index in range(3))


def ease(t):
    t = max(0.0, min(1.0, t))
    return 1 - (1 - t) ** 3


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


def rail(draw, y, width, progress, label):
    draw.line((48, y, width - 48, y), fill=RULE, width=2)
    draw.line((48, y, 48 + (width - 96) * max(0.0, min(1.0, progress)), y), fill=ACCENT, width=2)
    micro(draw, (48, y + 14), label, FAINT, 10, 1.8)


def steps(draw, x, y, active):
    for index, label in enumerate(("SPEAK", "CORRECT", "WRITE")):
        colour = ACCENT if index == active else FAINT
        micro(draw, (x, y), "0" + str(index + 1), colour, 10, 1.8)
        micro(draw, (x + 22, y), label, colour, 10, 1.8)
        x += 22 + len(label) * 8.4 + 26


def island(draw, box, sentence, state="INSERTED", sub="Typed into the field you were in.", elapsed="1.9 s"):
    radius = (box[3] - box[1]) / 2
    draw.rounded_rectangle(box, radius=radius, fill=INK)
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
        (("DICTATE", "anywhere you type"), ("LIVE", "text as you speak"), ("MEETINGS", "every line, one speaker"), ("TRANSLATE", "files, calls, pages"))
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


def save_gif(name, frames, durations, colors=16, downsample=1):
    kept = frames[::downsample]
    spans = [sum(durations[index : index + downsample]) for index in range(0, len(durations), downsample)]
    palette = [frame.convert("RGB").quantize(colors=colors, method=Image.MEDIANCUT) for frame in kept]
    palette[0].save(
        ASSETS / name,
        save_all=True,
        append_images=palette[1:],
        duration=spans,
        loop=0,
        optimize=True,
        disposal=2,
    )


def flow_frame(spoken_words, fade, corrected, typed, active, progress, blink):
    width, height = 1200, 460
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (48, 30), "PARLO DICTATE", ACCENT, 11, 2.2)
    steps(draw, 700, 30, active)

    words_a = SPOKEN_A.split(" ")
    words_b = SPOKEN_B.split(" ")
    shown_a = " ".join(words_a[: min(spoken_words, len(words_a))])
    shown_b = " ".join(words_b[: max(0, spoken_words - len(words_a))])
    if shown_a:
        text(draw, (48, 96), shown_a, 26, mix(MUTED, PAPER, fade), SERIF)
    if shown_b:
        text(draw, (48, 138), shown_b, 26, mix(MUTED, PAPER, fade), SERIF)

    if corrected > 0:
        offset = (1 - ease(corrected)) * 16
        text(draw, (48, 194 + offset), WRITTEN, 33, mix(PAPER, ACCENT, ease(corrected)), SERIF)

    box = (48, 262, 1152, 376)
    card(draw, box)
    micro(draw, (78, 282), "YOUR FIELD", FAINT, 10, 1.8)
    if typed:
        text(draw, (78, 306), typed, 24, INK)
        cursor_x = 78 + draw.textlength(typed, font=font(BODY, 24)) + 4
        draw.rectangle((48, 262, 52, 262 + 114 * ease(len(typed) / len(WRITTEN))), fill=ACCENT)
    else:
        cursor_x = 78
        for bar in range(3):
            draw.rounded_rectangle((78, 314 + bar * 18, 78 + 320 - bar * 60, 320 + bar * 18), radius=3, fill=(238, 231, 219))
    if blink:
        caret(draw, (cursor_x, 304), 28)

    rail(draw, 418, width, progress, "RULES FIRST, A MODEL ONLY IF YOU ALLOWED ONE · NOTHING LEAVES THE MACHINE")
    return image.convert("RGB")


def flow():
    frames, durations = [], []
    plan = []
    for word in range(1, 10):
        plan.append((word, 0.0, 0.0, "", 0, 0.06 * word))
    for step in range(1, 7):
        plan.append((9, step / 6, 0.0, "", 1, 0.55 + 0.03 * step))
    for step in range(1, 6):
        plan.append((9, 1.0, step / 5, "", 1, 0.74 + 0.02 * step))
    for step in range(1, 11):
        plan.append((9, 1.0, 1.0, WRITTEN[: round(len(WRITTEN) * step / 10)], 2, 0.85 + 0.012 * step))
    for step in range(0, 10):
        plan.append((9, 1.0, 1.0, WRITTEN, 2, 1.0))
    total = len(plan)
    for index, (words, fade, corrected, typed, active, progress) in enumerate(plan):
        frames.append(flow_frame(words, fade, corrected, typed, active, min(1.0, progress), index % 2 == 0))
        durations.append(200 if index < total - 10 else 280)
    save_gif("flow.gif", frames, durations)


def live_frame(kept, current_words, closed, phase, progress, blink):
    width, height = 1200, 430
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (48, 30), "PARLO LIVE", ACCENT, 11, 2.2)
    micro(draw, (1040, 30), "KEPT " + str(len(kept)), MUTED, 10, 1.8)
    draw.ellipse((1120, 32, 1132, 44), fill=ACCENT if blink else RULE)

    for index, line in enumerate(kept[-2:]):
        text(draw, (48, 80 + index * 26), line, 16, FAINT)

    current = " ".join(current_words)
    caret_x = 48
    if current:
        lines = wrap(draw, current, BODY, 32, 1080)
        for index, line in enumerate(lines):
            text(draw, (48, 146 + index * 40), line, 32, INK)
        caret_x = 48 + draw.textlength(lines[-1], font=font(BODY, 32)) + 4
    if blink and not closed:
        caret(draw, (caret_x, 142), 36)

    baseline = 304
    for index in range(48):
        level = WAVE[index % len(WAVE)] * (0.4 + 0.6 * abs(math.sin(phase + index * 0.35)))
        x = 48 + index * 22.5
        lit = index / 48 <= progress
        draw.rounded_rectangle((x, baseline - level, x + 6, baseline + level), radius=3, fill=ACCENT if lit else RULE)
    caption = "WORDS APPEAR BEFORE THE SENTENCE CLOSES · 849 MS TO THE FIRST PARTIAL"
    if closed:
        caption = "SENTENCE CLOSED · CLEANED, PUNCTUATED AND KEPT"
    micro(draw, (48, 362), caption, FAINT, 10, 1.8)
    rail(draw, 400, width, progress, "CLEANED AND PUNCTUATED WHEN THE SENTENCE CLOSES")
    return image.convert("RGB")


def live():
    frames, durations = [], []
    first = SECOND.split(" ")
    plan = []
    for word in range(1, len(Written := WRITTEN.split(" ")) + 1):
        plan.append(([], Written[:word], False, 0.10 * word))
        plan.append(([], Written[:word], False, 0.10 * word))
    plan.append(([], WRITTEN.split(" "), True, 1.0))
    plan.append(([WRITTEN], [], False, 0.0))
    for word in range(1, len(first) + 1):
        plan.append(([WRITTEN], first[:word], False, 0.12 * word))
        plan.append(([WRITTEN], first[:word], False, 0.12 * word))
    plan.append(([WRITTEN], first, True, 1.0))
    plan.append(([WRITTEN, SECOND], [], False, 0.0))
    for tick in range(8):
        plan.append(([WRITTEN, SECOND], [], False, 0.0))
    total = len(plan)
    for index, (kept, words, closed, progress) in enumerate(plan):
        frames.append(live_frame(kept, words, closed, index * 0.5, min(1.0, progress), index % 2 == 0))
        durations.append(220 if index < total - 8 else 280)
    save_gif("live.gif", frames, durations)


def meetings_frame(rows, frame_in_row, progress, pulse):
    width, height = 1200, 500
    image = paper(width, height)
    draw = ImageDraw.Draw(image)
    micro(draw, (48, 30), "PARLO MEETINGS", ACCENT, 11, 2.2)
    chip(draw, (760, 22), "THURSDAY REVIEW · 42 MIN", TEAL)
    draw.ellipse((1110, 32, 1122, 44), fill=ACCENT if pulse else RULE)
    micro(draw, (48, 62), "SPEAKERS DETECTED, EVERY LINE TIMED · NOTHING UPLOADED", FAINT, 10, 1.8)

    box = (48, 96, 1152, 412)
    card(draw, box)
    draw.line((box[0] + 1, box[1] + 46, box[2] - 1, box[1] + 46), fill=RULE, width=1)
    micro(draw, (box[0] + 26, box[1] + 18), "TRANSCRIPT", FAINT, 10, 2.0)
    micro(draw, (box[2] - 230, box[1] + 18), "SAVED IN YOUR FOLDER", FAINT, 9, 1.8)

    for index in range(len(MEETING_ROWS)):
        speaker, moment, line = MEETING_ROWS[index]
        y = 158 + index * 40
        entered = index < rows
        entering = index == rows - 1
        shift = (1 - ease(frame_in_row / 3)) * 26 if entering else 0
        fade = ease(frame_in_row / 3) if entering else (1.0 if entered else 0.0)
        tint = 0.55 if entering else (0.0 if not entered else 0.0)
        if entered and entering:
            draw.rounded_rectangle((box[0] + 8, y - 10, box[2] - 8, y + 32), radius=8, fill=mix(CARD, (247, 236, 228), tint))
        band = mix(PAPER, ACCENT if speaker == "SPEAKER 1" else TEAL, max(fade, 0.12))
        draw.rectangle((box[0], y - 2 + shift, box[0] + 4, y + 30 + shift), fill=band)
        text(draw, (box[0] + 26, y + shift), moment, 13, mix(PAPER, FAINT, max(fade, 0.14)), MONO)
        micro(draw, (box[0] + 116, y + 2 + shift), speaker, mix(PAPER, INK, max(fade, 0.14)), 9, 2.0)
        text(draw, (box[0] + 216, y + shift), line, 17, mix(PAPER, INK, max(fade, 0.14)))

    rail(draw, 446, width, progress, "CAPTURED FROM THE ROOM · NOTHING UPLOADED")
    return image.convert("RGB")


def meetings():
    frames, durations = [], []
    for index in range(len(MEETING_ROWS)):
        for tick in range(4):
            frames.append(meetings_frame(index + 1, tick, (index * 4 + tick) / 30, tick % 2 == 0))
            durations.append(220)
    for tick in range(8):
        frames.append(meetings_frame(len(MEETING_ROWS), 3, 1.0, tick % 2 == 0))
        durations.append(300)
    save_gif("meetings.gif", frames, durations)


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
    text(draw, (110, 300), "board-review-2026-10-31.mp4", 19, INK)
    text(draw, (112, 336), "42 min 18 s · 318 MB", 15, MUTED)
    chip(draw, (112, 380), "EN SUBTITLES", TEAL)
    chip(draw, (268, 380), ".SRT", (188, 178, 164), INK)
    draw.rounded_rectangle((112, 452, 628, 462), radius=5, fill=RULE)
    draw.rounded_rectangle((112, 452, 112 + 350, 462), radius=5, fill=ACCENT)
    micro(draw, (112, 486), "68% · 12 MIN LEFT", ACCENT, 11, 2.0)

    peak = 44
    for index in range(26):
        level = WAVE[index % len(WAVE)] / 46 * peak
        x = 112 + index * 20
        lit = index / 26 <= 0.68
        draw.rounded_rectangle((x, 560 - level, x + 6, 560 + level), radius=3, fill=TEAL if lit else RULE)
    micro(draw, (112, 604), "DUBBED AUDIO PLAYS HERE WHEN IT ENDS", FAINT, 10, 1.8)

    micro(draw, (732, 272), "SUBTITLES", FAINT, 11, 2.2)
    preview = (
        ("00:00:04,120", "The meeting starts at nine."),
        ("00:00:09,480", "Then we go on to the second office."),
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
