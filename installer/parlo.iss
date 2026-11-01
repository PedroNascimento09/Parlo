#ifndef AppVersion
  #define AppVersion "0.2.0"
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
DefaultDirName={localappdata}\Programs\Parlo
DefaultGroupName=Parlo
DisableProgramGroupPage=yes
DisableWelcomePage=no
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir={#Root}\build\installer
OutputBaseFilename=Parlo-Setup
SetupIconFile={#Root}\assets\parlo.ico
UninstallDisplayIcon={app}\parlo-app.exe
UninstallDisplayName=Parlo
WizardStyle=modern
WizardImageFile={#Root}\assets\installer\wizard.bmp
WizardSmallImageFile={#Root}\assets\installer\small.bmp
WizardImageStretch=no
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
MinVersion=10.0
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
VersionInfoCompany=Parlo
VersionInfoDescription=Parlo, speech and translation on this computer
VersionInfoProductName=Parlo
VersionInfoProductVersion={#AppVersion}
VersionInfoVersion={#AppVersion}
VersionInfoCopyright=Apache-2.0

[Messages]
WelcomeLabel1=Welcome to Parlo
WelcomeLabel2=Parlo turns your speech into clean text and your text into another language, on this computer alone. Nothing is uploaded and there is no account.%n%nThe models are a separate download of about 800 MB, chosen inside the app, and after that everything works offline.
FinishedLabel=Parlo is installed. Open it from the Start menu, or from the button below.
FinishedHeadingLabel=Parlo is ready

[Tasks]
Name: "desktopicon"; Description: "Put a shortcut on the Desktop"; Flags: unchecked
Name: "startup"; Description: "Open Parlo when Windows starts"; Flags: unchecked

[Files]
Source: "{#Payload}\parlo.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo-app.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo.png"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Payload}\parlo-engine\*"; DestDir: "{app}\parlo-engine"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Parlo"; Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\Parlo"; Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Parlo"; ValueData: """{app}\parlo-app.exe"""; Tasks: startup; Flags: uninsdeletevalue

[Run]
Filename: "{app}\parlo-app.exe"; WorkingDir: "{app}"; Description: "Open Parlo"; Flags: nowait postinstall skipifsilent
