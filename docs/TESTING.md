# Testing dictation by hand

Dictation is the one part of Parlo a test suite cannot prove, because it needs a
real window in front of a real microphone. Run this matrix before every release
that touches the island, the insertion path or the engine.

## What has to be true

For each application below, copy and paste three dictations in a row. Each result
must be complete, with no lost first word or duplicate text. Automatic insertion
is disabled until Parlo can verify the target field and cursor.

| Application | What to do | Run 1 | Run 2 | Run 3 |
| --- | --- | --- | --- | --- |
| Notepad | Open Notepad, click in the body, hold `Ctrl+Alt+D`, say "the quick brown fox jumps over the lazy dog, comma then stops period", release | | | |
| Chrome | Open any text field (a search box, a comment box), click inside, hold `Ctrl+Alt+D`, same sentence | | | |
| VS Code | Open a file, click at the end of a line, hold `Ctrl+Alt+D`, same sentence | | | |

Write a tick or the failure next to every run. A row with a failure blocks the
release. Record the Parlo version or commit, Windows version, microphone,
selected model and whether Polish is enabled. Repeat with Polish on and off.

## Focus and recovery

Run these checks in a disposable document without personal or important text.
Record the result for each application above.

1. Dictate a short sentence, move the caret before the text settles and type a
   word. Check that the typed word and surrounding text remain intact.
2. Start dictating, switch to a different window before the result arrives, and
   check where the result lands. It must not change text in the wrong field.
3. Close the target while dictating. Confirm the failure is visible and the
   transcript can be recovered without recording again.
4. Press Escape while recording. Check the resulting field and history for any
   text that was already written; record exactly what remains.
5. Copy text and an image before separate dictations. Check the clipboard after
   each insertion, including when insertion fails.

A failed check blocks a release that changes the relevant path. Do not assume a
window handle identifies a particular text field or that a successful simulated
keystroke proves the application accepted it.

## Comparing dictation products

Use consenting speakers and test phrases that contain no sensitive information.
For each speaker, prepare the same intended text in PT-PT or English, including
names, amounts, negations, punctuation, hesitations and explicit corrections.
Keep spontaneous speech as a separate set from read phrases. Record audio only
with permission; explain where it will be stored, who can access it and when it
will be deleted. Do not upload recordings to a service without explicit consent.

Test Parlo, Wispr Flow and Handy on the same computer and microphone. Record
product and model versions, language, cleanup settings, hardware, network state,
room conditions and whether each product processes audio or text in the cloud.
Where an app cannot accept the same recording, have the same speaker read the
same prompts and mark the comparison as repeated speech, not identical audio.
Use the same target application and alternate product order between speakers.

For each attempt keep the reference, raw output, final output, transcription
errors, missing or changed numbers and negations, manual edits, failures and the
time from the end of speech until usable text appears in the field. Include
failed attempts in the report. State the number of speakers and attempts, the
measurement method and whether the model was warm. Do not compare the app's
internal timing with an end-to-end stopwatch as if they measured the same thing.
Never publish recordings, transcripts or identifiable results without permission.

## What else to check in the same session

- The island follows the pointer when dragged and stays where it is dropped.
- The language menu opens fully: every pair is readable and the list scrolls.
- The result stays available until it is copied; a new dictation must not replace an uncopied result.
- Set an unused copy shortcut in Settings, copy a result with it, then disable it and confirm it no longer acts. Try a conflicting shortcut and check the error.
- Close and reopen Parlo with an uncopied result and check that the saved text can be reviewed, copied or discarded.
- `Ctrl+Alt+Z` removes the text Parlo typed last, while that window is still in front; it sends one backspace per character and cannot see the cursor.
- `Ctrl+Alt+Shift+D` keeps listening until it is pressed again.
- The island shows no timer. The sentence stays visible while you speak, so you
  see it before you stop.
- Check Polish off and on separately. In both modes the result is available to
  copy, and with Polish on the sentence in the field is swapped for the cleaned one.
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
