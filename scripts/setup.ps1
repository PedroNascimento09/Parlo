$ErrorActionPreference = "Stop"

$archive = Join-Path $PSScriptRoot "parlo-windows-x86_64.zip"
if (-not (Test-Path $archive)) {
    throw "The installer is incomplete: parlo-windows-x86_64.zip is missing beside it."
}

& (Join-Path $PSScriptRoot "install.ps1") -Archive $archive
