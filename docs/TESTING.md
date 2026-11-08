# Testing dictation by hand

Dictation is the one part of Parlo a test suite cannot prove, because it needs a
real window in front of a real microphone. Run this matrix before every release
that touches the island, the insertion path or the engine.

## What has to be true

For each application below, three dictations in a row must land where the cursor
is, complete, with no lost first word and no duplicate text.

| Application | What to do | Run 1 | Run 2 | Run 3 |
| --- | --- | --- | --- | --- |
| Notepad | Open Notepad, click in the body, hold `Alt+Z`, say "the quick brown fox jumps over the lazy dog, comma then stops period", release | | | |
| Chrome | Open any text field (a search box, a comment box), click inside, hold `Alt+Z`, same sentence | | | |
| VS Code | Open a file, click at the end of a line, hold `Alt+Z`, same sentence | | | |

Write a tick or the failure next to every run. A row with a failure blocks the
release.

## What else to check in the same session

- The island follows the pointer when dragged and stays where it is dropped.
- The language menu opens fully: every pair is readable and the list scrolls.
- The island hides about four seconds after the text is written.
- `Ctrl+Alt+Z` removes the last dictation.
- `Ctrl+Alt+Shift+D` keeps listening until it is pressed again.
- The island shows no timer. The sentence stays visible while you speak, so you
  see it before you stop.
- Speaking quietly still produces text: the first second of audio is measured
  and lifted when the room is silent.
- "new line", "bullet point", "comma", "period" and "scratch that" change what
  is written, in English and in Portuguese.

## When something fails

Read `%TEMP%\parlo-app.log` and `%TEMP%\parlo-engine.log` first. The app log
names the window it captured before recording, the engine log names the model it
loaded. A dictation that produced no text at all is usually one of three things:
no engine on the port, the wrong input device, or a window that refused to come
forward.
