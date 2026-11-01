# Install Parlo

Parlo is one folder and one program. Nothing is installed system-wide, no administrator password is needed, and no account exists to create.

## Windows, one click

1. Open the [latest release](https://github.com/PedroNascimento09/Parlo/releases/latest).
2. Download `Parlo-Setup.exe` and double click it.
3. If Windows says *Windows protected your PC*, choose **More info**, then **Run anyway**. The build is unsigned, so this appears once.
4. Parlo is installed in `%LOCALAPPDATA%\Programs\Parlo`, appears in the Start menu and opens by itself.

The installer carries the same files as the archive, extracts them for you and creates the shortcut. Nothing is installed system-wide, no administrator password is asked, and nothing stays open in a console.

## Windows, the short way

1. Press **Start**, type `PowerShell`, press **Enter**.
2. Paste this line and press **Enter**:

```powershell
irm https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.ps1 | iex
```

3. When it finishes, **Parlo** is in the Start menu. Open it, press **Download models** in the Models panel, and wait once.

That is the whole installation. The script downloads the Windows archive straight from the latest release with `curl`, unpacks it into `%LOCALAPPDATA%\Programs\Parlo` and adds the Start menu entry. To read it before running it, open [scripts/install.ps1](../scripts/install.ps1); it downloads and unpacks, nothing else.

Measured on the reference machine (Windows 11, no administrator rights): the archive is **80 MB** and unpacks to **204 MB**; unpacking a copy already on disk took **2.3 seconds**. The earlier **37 seconds** figure was measured when the archive was 220 MB and had to be downloaded first, so the download is now a third of the bytes. No terminal stays open, nothing is installed system-wide, and the folder can be deleted to remove it.

The wait people notice is afterwards, on the first start: about **800 MB of models**, once, from the Models panel.

## Windows, without a terminal

1. Open the [latest release](https://github.com/PedroNascimento09/Parlo/releases/latest).
2. Download `parlo-windows-x86_64.zip`.
3. Right-click the file, **Extract All**, and open the folder it creates.
4. Double-click `parlo-app`. If Windows says *Windows protected your PC*, choose **More info**, then **Run anyway**. The archive is not signed, so that warning appears once.
5. Keep the folder where it is: the translation engine lives beside the program. To move it somewhere permanent, use `parlo install` from that folder, which copies it to `%LOCALAPPDATA%\Programs\Parlo` and adds the Start menu entry.

## macOS

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

The published build is for Apple silicon. It lands in `~/Applications/Parlo`, and `parlo-app` is linked into `~/.local/bin`. Without the terminal: download `parlo-macos-aarch64.tar.gz`, unpack it with a double-click, and open `parlo-app`; macOS may ask for confirmation the first time because the build is unsigned.

## Linux

```bash
curl -fsSL https://raw.githubusercontent.com/PedroNascimento09/Parlo/main/scripts/install.sh | bash
```

It unpacks into `~/.local/opt/parlo` and links `parlo-app` into `~/.local/bin`, so the window opens from any terminal. The window needs WebKitGTK, which most desktops already have; on a minimal system install `libwebkit2gtk-4.1-0`.

## The first run

The archive carries the programs, not the models. The window opens, the Models panel lists what is missing, and one **Download models** button fetches about 800 MB into your user data folder — once, on a connection you already have. After that Parlo works with the network switched off.

The first start can take around twenty seconds while the engine loads, and the window says so instead of looking stuck.

## Updates

Parlo checks the latest published release when it starts. When a newer one exists, a slim banner shows the version and the notes, with an **Install update** button. Nothing installs without that click, nothing is uninstalled to upgrade, and a user who never opens the app is never updated behind their back.

Installing by hand again over the same folder works too: the new files replace the old ones and your models, meetings and vocabulary stay where they are.

## Uninstall

- Windows: `parlo uninstall` from the installed folder, or delete `%LOCALAPPDATA%\Programs\Parlo` and the Start menu shortcut.
- macOS and Linux: delete `~/Applications/Parlo` or `~/.local/opt/parlo`, and the `parlo-app` link in `~/.local/bin`.

Both leave your models in place unless you add `--purge`, because downloading 800 MB again is not a kindness.

## When something does not work

| What you see | What to do |
|---|---|
| *Windows protected your PC* | **More info**, then **Run anyway**. The build is unsigned. |
| The window stays on the starting screen | Wait up to four minutes on a first run; if it never loads, read `parlo-app.log` and `parlo-engine.log` in `%TEMP%`. |
| The Models panel says something is missing | Press **Download models**; a partial download resumes from where it stopped. |
| Dubbing refuses to start | `ffmpeg` and `ffprobe` must be on `PATH`; the panel says which one is absent. |
| The microphone never opens | Windows Settings, Privacy, Microphone: allow desktop apps, then reopen Parlo. |
| The shortcut does nothing | Another program owns that combination; change it in Settings, which validates it and says a restart registers it. |

More detail: [APP.md](APP.md), [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
