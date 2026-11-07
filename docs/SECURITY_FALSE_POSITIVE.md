# Security software false positives

Parlo is free, open-source software that runs entirely on your computer. It does not send audio anywhere, does not phone home, and has no hidden network component in the local mode.

If Windows Defender, SmartScreen, another antivirus, or a corporate endpoint blocks Parlo-Setup.exe, that is usually a **false positive**, not evidence that the file is malicious.

## Why this happens

The release workflow builds Parlo on GitHub Actions from public source and signs nothing, because this project does not yet have a code-signing certificate. Microsoft Defender SmartScreen and many antivirus engines flag that combination hard:

- new file hash on every release
- no Authenticode signature
- an unsigned Inno Setup installer wrapping a Rust binary and a PyInstaller-based engine

A common detection name on this stack is `Trojan:Win32/Wacatac.B!ml` (or similar `!ml` names). The `!ml` suffix means the verdict came from a machine-learning heuristic, not from a known-malware signature. That class of detection is a known recurring false positive for unsigned Rust/Tauri/Go installers and for PyInstaller-wrapped engines. It is also the class that frequently gets cleared after Microsoft reviews the file.

## What a blocked user can do right now

1. Verify the file really came from this release.
   - Download from the project release page, not a mirror or a search result.
   - Under the download, the release also carries `SHA256SUMS.txt`.
   - In PowerShell: `certutil -hashfile Parlo-Setup.exe SHA256`
   - The line must match the one in `SHA256SUMS.txt`.
   - With the GitHub CLI authenticated, you can also run `gh attestation verify Parlo-Setup.exe --repo PedroNascimento09/Parlo` to confirm the file came from this repository and workflow.
2. If you trust the source, bypassing SmartScreen once is normal:
   - **More info** → **Run anyway**.
3. If Defender already quarantined the file, do not keep clicking through.
   - Restore the quarantined file from Windows Security, or rebuild/download the same version from the release.
   - Submit that exact file to Microsoft as a false positive (below).
   - Only after Microsoft clears it should you keep using that build path.

## How to report the false positive to Microsoft

Use the Microsoft Security Intelligence file submission portal: `https://www.microsoft.com/wdsi/filesubmission`.

Submit the **exact file** that was blocked, as a **Software developer**, using the **Clean (false positive)** category.

Suggested values for the submission form:

- Product: `Microsoft Defender SmartScreen`
- If the file was also quarantined by Defender Antivirus, add a second product selection for the antivirus that actually blocked it, most often `Microsoft Defender Antivirus (Windows 10)` or `Microsoft Defender Antivirus (Windows 11)`.
- Category: `Incorrectly detected as malware/malicious`
- File: `Parlo-Setup.exe` from the affected release
- Detection name: whatever Windows showed you, for example `Trojan:Win32/Wacatac.B!ml`
- Definition version: the antivirus definition version on the machine that blocked the file, if you have it from Windows Security
- Additional information: state plainly that this is the official Parlo release built from public source by the project’s GitHub Actions release workflow, link the release URL, and include the file’s SHA-256.

For v0.2.5 from the project’s release workflow, the file identification is:

- File: `Parlo-Setup.exe`
- Size: 40,181,386 bytes
- SHA-256: `1D58A187FC407104434FF9525EBD8B06F08F72F5B39FBFBEE053E212CFB10989`
- Source: `https://github.com/PedroNascimento09/Parlo/releases`
- Build: Windows x86_64 release from `cargo build --release` plus Inno Setup, no code signature configured in the workflow yet

If Microsoft’s final determination is wrong, use the developer contact form linked from the submission result. That is the supported escalation path; there is no separate whitelist or manual reputation enrollment channel.

## What this does and does not do

Submitting a false positive can get Microsoft’s detection cleared, and that fix propagates to users through the normal definition update process. It does not guarantee that every engine clears it immediately, and it does not replace the durability that a signed release would give you.

This project currently avoids the paid path on purpose. The honest state is written next to the download: the installer is unsigned, SmartScreen may warn once, and the checksum plus build provenance are how you verify the file you got is the file the project built.

## For maintainers

When a new release gets flagged:

1. Capture the exact artifact, its SHA-256, the detection name and the definition version.
2. Submit it through the portal above as a software developer.
3. Keep the submission ID and the final determination in the release notes or in a closed issue so a repeat flag has context.
4. If the same class of detection recurs on several releases, the durable fix is code signing with a trusted certificate; the release workflow already has a signing hook that stays quiet until `WINDOWS_SIGNING_PFX_BASE64` and `WINDOWS_SIGNING_PASSWORD` are set.
