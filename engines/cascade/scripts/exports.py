import subprocess
import zipfile
from pathlib import Path


def _vtt_time(seconds):
    milliseconds = max(int(round(seconds * 1000)), 0)
    hours, rest = divmod(milliseconds, 3_600_000)
    minutes, rest = divmod(rest, 60_000)
    whole, millis = divmod(rest, 1000)
    return f"{hours:02d}:{minutes:02d}:{whole:02d}.{millis:03d}"


def _ass_time(seconds):
    centiseconds = max(int(round(seconds * 100)), 0)
    hours, rest = divmod(centiseconds, 360_000)
    minutes, rest = divmod(rest, 6_000)
    whole, centis = divmod(rest, 100)
    return f"{hours}:{minutes:02d}:{whole:02d}.{centis:02d}"


def _text(value):
    return str(value).replace("\r", " ").replace("\n", "\\N")


def _value(line, name):
    return line.get(name) if isinstance(line, dict) else getattr(line, name)


def _lines(lines):
    for line in lines:
        if _value(line, "failed") or _value(line, "skipped") or not _value(line, "translated"):
            continue
        yield line


def write_vtt(path, lines):
    blocks = ["WEBVTT", ""]
    for line in _lines(lines):
        blocks.extend(
            [
                f"{_vtt_time(_value(line, 'start'))} --> {_vtt_time(_value(line, 'end'))}",
                _text(_value(line, "translated")),
                "",
            ]
        )
    path.write_text("\n".join(blocks), encoding="utf-8")


def write_ass(path, lines, karaoke=False):
    header = [
        "[Script Info]",
        "ScriptType: v4.00+",
        "PlayResX: 1920",
        "PlayResY: 1080",
        "WrapStyle: 2",
        "ScaledBorderAndShadow: yes",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
        "Style: Default,Arial,64,&H00FFFFFF,&H0000FFFF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,3,1,2,40,40,60,1",
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
    ]
    events = []
    for line in _lines(lines):
        prefix = ""
        if karaoke:
            centiseconds = max(
                int(round((_value(line, "end") - _value(line, "start")) * 100)), 0
            )
            prefix = f"{{\\k{centiseconds}}}"
        events.append(
            f"Dialogue: 0,{_ass_time(_value(line, 'start'))},{_ass_time(_value(line, 'end'))},Default,,0,0,0,,{prefix}{_text(_value(line, 'translated'))}"
        )
    path.write_text("\n".join(header + events) + "\n", encoding="utf-8")


def _run(command, cwd=None):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "ffmpeg failed")


def burn(ffmpeg, source, track, subtitle, target, video, ass=False):
    if not video:
        return None
    source = Path(source).resolve()
    track = Path(track).resolve()
    subtitle = Path(subtitle).resolve()
    target = Path(target).resolve()
    filter_name = "ass" if ass else "subtitles"
    command = [
        ffmpeg,
        "-nostdin",
        "-y",
        "-v",
        "error",
        "-i",
        str(source),
        "-i",
        str(track),
        "-vf",
        f"{filter_name}={subtitle.name}",
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "20",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-shortest",
        str(target),
    ]
    _run(command, cwd=subtitle.parent)
    return target


def dual(ffmpeg, source, track, target, video):
    if not video:
        return None
    source = Path(source).resolve()
    track = Path(track).resolve()
    target = Path(target).resolve()
    command = [
        ffmpeg,
        "-nostdin",
        "-y",
        "-v",
        "error",
        "-i",
        str(source),
        "-i",
        str(track),
        "-map",
        "0:v:0",
        "-map",
        "0:a:0?",
        "-map",
        "1:a:0",
        "-c:v",
        "copy",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-metadata:s:a:0",
        "title=Original",
        "-metadata:s:a:1",
        "title=Dubbed",
        "-shortest",
        str(target),
    ]
    _run(command)
    return target


def audio(ffmpeg, track, wav, mp3, bitrate):
    track = Path(track).resolve()
    wav = Path(wav).resolve()
    mp3 = Path(mp3).resolve()
    _run(
        [
            ffmpeg,
            "-nostdin",
            "-y",
            "-v",
            "error",
            "-i",
            str(track),
            "-c:a",
            "pcm_s16le",
            str(wav),
        ]
    )
    _run(
        [
            ffmpeg,
            "-nostdin",
            "-y",
            "-v",
            "error",
            "-i",
            str(track),
            "-c:a",
            "libmp3lame",
            "-b:a",
            f"{bitrate}k",
            str(mp3),
        ]
    )
    return wav, mp3


def stems(target, voice, original, mix, srt):
    target = Path(target).resolve()
    voice = Path(voice).resolve()
    original = Path(original).resolve()
    mix = Path(mix).resolve()
    srt = Path(srt).resolve()
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
        for path, name in (
            (voice, "voice.wav"),
            (original, "original.wav"),
            (mix, "mix.wav"),
            (srt, "lines.srt"),
        ):
            if path.is_file():
                archive.write(path, arcname=name)
    return target


def export_all(ffmpeg, source, out, srt, track, voice, original, mix, lines, video, bitrate, work):
    vtt = srt.with_suffix(".vtt")
    ass = srt.with_suffix(".ass")
    karaoke = srt.with_name(f"{srt.stem}.karaoke.ass")
    write_vtt(vtt, lines)
    write_ass(ass, lines)
    write_ass(karaoke, lines, karaoke=True)

    exports = [
        {"format": "video", "label": "Video", "path": out},
        {"format": "srt", "label": "SRT", "path": srt},
        {"format": "vtt", "label": "WebVTT", "path": vtt},
        {"format": "ass", "label": "ASS", "path": ass},
        {"format": "karaoke", "label": "Karaoke ASS", "path": karaoke},
    ]

    burned = burn(
        ffmpeg,
        source,
        track,
        srt,
        out.with_name(f"{out.stem}.burned{out.suffix}"),
        video,
    )
    if burned is not None:
        exports.append({"format": "burned", "label": "Burned subtitles", "path": burned})

    karaoke_video = burn(
        ffmpeg,
        source,
        track,
        karaoke,
        out.with_name(f"{out.stem}.karaoke{out.suffix}"),
        video,
        ass=True,
    )
    if karaoke_video is not None:
        exports.append({"format": "karaoke-video", "label": "Karaoke video", "path": karaoke_video})

    dubbed_dual = dual(
        ffmpeg,
        source,
        track,
        out.with_name(f"{out.stem}.dual{out.suffix}"),
        video,
    )
    if dubbed_dual is not None:
        exports.append({"format": "dual", "label": "Dual audio", "path": dubbed_dual})

    wav = out.with_name(f"{out.stem}.dub.wav")
    mp3 = out.with_name(f"{out.stem}.dub.mp3")
    audio(ffmpeg, track, wav, mp3, bitrate)
    exports.extend(
        [
            {"format": "wav", "label": "WAV", "path": wav},
            {"format": "mp3", "label": "MP3", "path": mp3},
        ]
    )

    archive = out.with_name(f"{out.stem}.stems.zip")
    stems(archive, voice, original, mix, srt)
    exports.append({"format": "stems", "label": "Stems ZIP", "path": archive})

    return [
        {**item, "path": str(Path(item["path"]).resolve())}
        for item in exports
        if Path(item["path"]).is_file()
    ]
