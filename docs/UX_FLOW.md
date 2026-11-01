# Parlo UX flow

One product with five levels of presence. The user never chooses which "Parlo app" to open.

```
Parlo Presence (tray)
   ├── click            → Contextual surface: Dictate
   ├── Live             → Live surface
   ├── Open Parlo       → Hub
   ├── Settings         → Hub, settings group
   └── Quit
```

## States

| State | Surface | What the user sees | How they leave it |
|---|---|---|---|
| Presence idle | tray only | One icon, no window, no panel | click, Live, Open Parlo |
| Dictate listening | island 560 × 96 | dot, `LISTENING`, level, timer | click tray again, `Ctrl+Alt+D`, `Esc` |
| Dictate writing | island | `WRITING` | automatic |
| Dictate delivered | island | `INSERTED` or `COPIED`, one line of text | hides after 4 s |
| Dictate failed | island | `FAILED` and the reason | click tray to retry |
| Live running | Live surface | direction, who speaks, last sentence both languages, voice identity state | Pause, Stop |
| Hub | main window | five navigation groups | close |

## Rules that make it feel like one product

1. The tray is the only permanent presence: no dock clutter, no second launcher, no menu behind a shortcut.
2. Click first, shortcut later: every action in the tray menu is reachable by clicking; shortcuts are accelerators and can be changed or removed.
3. Nothing technical appears in a contextual surface: no models, no quantisation, no paths, no download progress.
4. Dictation inserts into the field that was in front when listening started, and only if that same window is still in front. Otherwise the text goes to the clipboard and the island says so.
5. The Hub is opened deliberately, never as a step of a task.
6. Live starts from the remembered configuration; the setup screen appears only on the first run or when something is incompatible.

## Dictate, end to end

```
user focuses a text field in any app
        │
        ├── click the tray icon ──┐
        │                          ├── island shown, target window captured
        ├── Ctrl+Alt+D ────────────┘
                                    │
                              speak, then
        ┌───────────────────────────┤
        │  click tray again         │
        │  Ctrl+Alt+D               │
        │  Esc                      │
        └───────────────┬───────────┘
                        │
                 /dictate → text
                        │
        target still in front? ── yes ── /insert → text lands in the field
                        │
                        no ── clipboard, island says "COPIED"
```

## What the first version implements

Implemented here: tray presence with menu, click to dictate, the island states above, target capture and guarded insertion on Windows, clipboard fallback, the existing shortcut kept as an accelerator.

Not implemented here, and not promised in the interface: the Live surface (the menu item opens the Hub where Live lives today), macOS and Linux insertion, do-not-disturb during calls.

## Coherence check

- Every state has an exit, and no state needs a second window to explain the first.
- The presence menu contains an action for each level, so the shortcut is never the only route.
- Insertion can fail for one reason only (the target window changed); when it does, the product does not lose the text.
- The Hub keeps every capability, so moving something out of the daily flow never makes it unreachable.
