#ifndef AppVersion
  #define AppVersion "0.2.4"
#endif

#define Root ".."
#define Payload Root + "\build\installer\payload"

[Setup]
AppId={{8F2A5C31-6B4E-4C0A-9E71-3C7D5A1B2E64}
AppName=Parlo
AppVersion={#AppVersion}
AppVerName=Parlo {#AppVersion}
AppPublisher=Parlo
AppPublisherURL=https://github.com/PedroNascimento09/Parlo
AppSupportURL=https://github.com/PedroNascimento09/Parlo/issues
AppUpdatesURL=https://github.com/PedroNascimento09/Parlo/releases
AppCopyright=Copyright (C) 2026 Parlo. Apache-2.0.
DefaultDirName={localappdata}\Programs\Parlo
DisableDirPage=auto
DisableProgramGroupPage=yes
DisableWelcomePage=no
PrivilegesRequired=lowest
UsePreviousAppDir=yes
CloseApplications=yes
RestartApplications=no
SetupLogging=yes
OutputDir={#Root}\build\installer
OutputBaseFilename=Parlo-Setup
SetupIconFile={#Root}\assets\parlo.ico
UninstallDisplayIcon={app}\parlo-app.exe
UninstallDisplayName=Parlo
WizardStyle=modern
WizardImageFile={#Root}\assets\installer\wizard.bmp
WizardImageStretch=no
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
MinVersion=10.0
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
VersionInfoCompany=Parlo
VersionInfoDescription=Parlo, speech and translation on this computer
VersionInfoOriginalFileName=Parlo-Setup.exe
VersionInfoProductName=Parlo
VersionInfoProductVersion={#AppVersion}
VersionInfoVersion={#AppVersion}
VersionInfoCopyright=Copyright (C) 2026 Parlo

#ifdef SignedInstall
SignTool=parlo
SignedUninstaller=yes
#endif

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Messages]
WelcomeLabel1=Install Parlo
WelcomeLabel2=Parlo turns speech into clean text and writes it into the field you were already typing in. Everything stays on this computer: no account, no upload, no telemetry.%n%nSetup installs for your user only, so Windows never asks for an administrator password. The speech model is a separate download of about 640 MB, chosen inside the app.
FinishedHeadingLabel=Parlo is ready
FinishedLabel=Open Parlo and press Ctrl+Alt+D anywhere to speak. The first run offers the speech model, and nothing downloads until you choose it.

[Tasks]
Name: "desktopicon"; Description: "Create a Desktop shortcut"; Flags: unchecked
Name: "startup"; Description: "Start Parlo when I sign in to Windows"; Flags: unchecked

[Files]
Source: "{#Payload}\parlo.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo-app.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo.png"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo-engine\*"; DestDir: "{app}\parlo-engine"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#Root}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\LICENSES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Root}\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Parlo"; Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"
Name: "{group}\Uninstall Parlo"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Parlo"; Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Parlo"; ValueData: """{app}\parlo-app.exe"""; Tasks: startup; Flags: uninsdeletevalue

[Run]
Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"; Description: "Open Parlo"; Flags: nowait postinstall skipifsilent
