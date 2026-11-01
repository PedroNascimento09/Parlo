param(
    [string]$Dest = (Join-Path $env:LOCALAPPDATA "Programs\Parlo"),
    [string]$Archive = "",
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$url = "https://github.com/PedroNascimento09/Parlo/releases/latest/download/parlo-windows-x86_64.zip"

if ($Archive -ne "") {
    $zip = $Archive
} else {
    $zip = Join-Path $env:TEMP "parlo-windows-x86_64.zip"
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        & curl.exe --fail --location --retry 2 --progress-bar --output $zip $url
        if ($LASTEXITCODE -ne 0) {
            throw "The download failed. Check the connection and run this again; it starts from the beginning."
        }
    } else {
        Write-Host "Downloading Parlo..."
        Invoke-WebRequest -Uri $url -OutFile $zip
    }
}

if (Test-Path $Dest) {
    Remove-Item $Dest -Recurse -Force
}
New-Item -ItemType Directory -Path $Dest | Out-Null

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::ExtractToDirectory($zip, $Dest)
if ($Archive -eq "") {
    Remove-Item $zip -Force
}

$app = Get-ChildItem -Path $Dest -Recurse -Filter "parlo-app.exe" | Select-Object -First 1
if (-not $app) {
    throw "The archive did not contain parlo-app.exe. Nothing usable was installed in $Dest."
}

$shell = New-Object -ComObject WScript.Shell
$menu = [Environment]::GetFolderPath("Programs")
$shortcut = $shell.CreateShortcut((Join-Path $menu "Parlo.lnk"))
$shortcut.TargetPath = $app.FullName
$shortcut.WorkingDirectory = $app.DirectoryName
$shortcut.IconLocation = "$($app.FullName),0"
$shortcut.Description = "Parlo, dictation and translation on this computer"
$shortcut.Save()

Write-Host "Parlo is installed in $Dest"
Write-Host "It is in the Start menu as Parlo. The first run downloads about 800 MB of models, once."

if (-not $NoLaunch) {
    Start-Process -FilePath $app.FullName -WorkingDirectory $app.DirectoryName
}
