param(
    [string]$Dest = (Join-Path $env:LOCALAPPDATA "Programs\Parlo"),
    [string]$Archive = "",
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/PedroNascimento09/Parlo/releases/latest"
$asset = $release.assets | Where-Object { $_.name -like "*windows*x86_64*.zip" } | Select-Object -First 1

if (-not $asset) {
    throw "The latest release has no Windows archive. Install from source with scripts\parlo-app.cmd instead."
}

if ($Archive -ne "") {
    $zip = $Archive
} else {
    $zip = Join-Path $env:TEMP $asset.name
    Write-Host "Downloading Parlo $($release.tag_name), $([math]::Round($asset.size / 1MB)) MB..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zip
}

if (Test-Path $Dest) {
    Remove-Item $Dest -Recurse -Force
}
New-Item -ItemType Directory -Path $Dest | Out-Null
Expand-Archive -Path $zip -DestinationPath $Dest -Force
if ($Archive -eq "") {
    Remove-Item $zip -Force
}

$app = Get-ChildItem -Path $Dest -Recurse -Filter "parlo-app.exe" | Select-Object -First 1
if (-not $app) {
    throw "The archive did not contain parlo-app.exe. Nothing was installed correctly in $Dest."
}

$shell = New-Object -ComObject WScript.Shell
$menu = [Environment]::GetFolderPath("Programs")
$shortcut = $shell.CreateShortcut((Join-Path $menu "Parlo.lnk"))
$shortcut.TargetPath = $app.FullName
$shortcut.WorkingDirectory = $app.DirectoryName
$shortcut.IconLocation = "$($app.FullName),0"
$shortcut.Description = "Parlo, dictation and translation on this computer"
$shortcut.Save()

Write-Host "Parlo $($release.tag_name) is installed in $Dest"
Write-Host "It is in the Start menu as Parlo. The first run downloads about 800 MB of models, once."

if (-not $NoLaunch) {
    Start-Process -FilePath $app.FullName -WorkingDirectory $app.DirectoryName
}
