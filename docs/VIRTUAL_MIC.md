# Virtual microphone

A virtual microphone is a device other apps can select while Parlo writes audio into it. This is the most fragile part of the product, because every operating system treats audio drivers differently.

## Linux: PipeWire

No driver required. Parlo creates and removes the null sink for you:

```bash
parlo mic setup
parlo mic status
parlo mic remove
```

Under the hood these are ordinary PipeWire commands, if you prefer doing it by hand:

```bash
pactl load-module module-null-sink sink_name=parlo sink_properties=device.description=Parlo
pactl list short modules
pactl unload-module <id>
```

In Zoom, Meet, or Discord, select the monitor of `Parlo` as the microphone. `parlo mic setup` is idempotent: running it twice reports the existing module instead of creating a second sink.

This is the only platform where the virtual microphone can ship without signing anything, which is why Linux comes first.

## macOS: audio server plug-in

macOS needs a user-space Audio Server Plug-in installed with administrator approval, signed and notarized. [BlackHole](https://github.com/ExistentialAudio/BlackHole) is GPL, so Parlo cannot link it and stay MIT. Options:

- Write a minimal MIT-licensed plug-in.
- Ask the user to install BlackHole and route through it, stated clearly as a prerequisite.

The second option ships this month; the first ships with a signed installer.

## Windows: signed driver

Windows needs a virtual audio driver with a valid signature, and no program can create a microphone device in user space: the driver has to be signed with an EV certificate and attested by Microsoft. Parlo therefore ships no driver and never offers to install one.

What `parlo mic status` and `parlo mic setup` do instead is the guided setup with a cable that is already signed and installed:

1. Install [VB-CABLE](https://vb-audio.com/Cable/) (donationware), VoiceMeeter or Scream, and restart.
2. Run `parlo mic status`. It lists the output devices, names the cable it found and its index, so the command is literally printed: `parlo listen --play --speaker 2`.
3. In Zoom, Teams or Meet, choose the cable's **recording** side (`CABLE Output`) as the microphone; the device Parlo plays into (`CABLE Input`) is not the one the call listens to.

Without a cable, the same command says so in one sentence and names what to install, instead of failing. `parlo mic remove` on Windows says plainly that Parlo installed no driver and that the cable is removed by its own uninstaller.

This keeps the interface identical to Linux — `setup`, `status`, `remove` — with the honest difference that on Windows Parlo routes an existing device rather than creating one. **Status: the matching and the sentences are covered by tests; nothing was routed into a real cable here.**

## Feedback loops

The translated voice leaves through the speakers, comes back through the microphone, and gets translated again. Three defences, in order:

1. Recommend headphones during setup and detect speakers when possible.
2. Gate capture while the engine is speaking, when the selected mode requires it.
3. Run a loop test in the setup wizard: play a short phrase, listen, and warn if the output re-enters the input.

The `VoiceGate` in the core is the first line of defence: silence does not reach the engine, and no engine output is produced for silent frames.

## Output modes

| Mode | Behaviour | Use case |
|---|---|---|
| Translated only | The other side hears only the translated voice | Default |
| Translated plus quiet original | Original at low level under the translation | When nuance matters |
| Passthrough | Audio is forwarded untouched | Parlo temporarily off |
