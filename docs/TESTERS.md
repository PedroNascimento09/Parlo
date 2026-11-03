# Testing Parlo from outside

This is the kit for someone who is not the author and is willing to use Parlo for a while and say what broke. No external tester has used this build yet, so G2 is the author's own first session on a clean install for now; this kit is ready for the first person who does run it, and a release never claims validation by users it has not had.

## What you need

- Windows 10 or 11 is the only platform validated end to end. macOS and Linux builds are compiled by CI and have never had the window opened on them; if you run one, you are the first.
- A microphone, about 1 GB of free disk for the models, and no account of any kind.
- One archive, unzipped, `parlo-app` double-clicked. The build is **unsigned**: Windows will show *Windows protected your PC*, and **More info → Run anyway** is expected. Compare the SHA-256 of the archive with the one published next to the release before you do.
- `ffmpeg` on `PATH` only if you want to test dubbing.

## Twenty minutes, in this order

1. **First run.** Open the window, press **Download models**, wait. Write down how long it took and whether the panel ever looked stuck.
2. **One sentence.** Hold the record card, say a sentence in Portuguese, let go. Did the translation arrive, and did it say what you said?
3. **The other direction.** Change the pair, restart, translate one English sentence.
4. **Dictate.** Open a text editor, press the dictation shortcut, say a sentence, press it again. Did the text land in the editor, and was the focus still in the editor?
5. **A correction.** Dictate «Quero comprar 2 bilhetes para o cinema, quer dizer, 3 bilhetes para o cinema» and report exactly what appeared. Then dictate «Gosto de cinema, quer dizer, gosto especialmente de filmes de ficção científica» and check that both ideas survive.
6. **Live.** Turn Live on over a news broadcast for a minute. How far behind was it, and did it ever stop?
7. **Quit and reopen.** Does everything you changed come back?

## What to report

Send, in plain words and without a template you have to fill:

- What you did, what you expected, what happened.
- The version from the top strip, your Windows version, and whether the microphone is built in or USB.
- If something failed, the two log files from `%TEMP%`: `parlo-app.log` and `parlo-engine.log`.

A wrong translation is worth reporting too, with the sentence you said and the sentence you got: that is the only way the numbers stop being the author's.

## What is already known, so you do not have to report it

- Machine translation, not human quality. Names are the weakest part.
- PT→DE exists only through English and has not been read by a German speaker.
- The first translation takes a few seconds; live mode keeps that wait for each window.
- Live cannot reach a call application without a virtual cable.
- Dictation cleanup removes hesitations and self-corrections; it can also collapse a deliberate repetition.
- The update button replaces the binary behind the page, not the window program.
- The macOS and Linux builds are untested in the window.
- The installers are unsigned.

## Privacy

Audio stays on your machine. There is no telemetry, no account and no upload; the only outside request the app makes is the update check, which carries no audio and no identifier. Feedback is whatever you choose to send, and nothing is collected without you sending it.
