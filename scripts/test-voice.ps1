param(
    [int]$Clips = 10,
    [int]$Seconds = 6,
    [switch]$Auto,
    [switch]$SkipLive
)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$exe = Join-Path $root "target\release\parlo.exe"
$python = Join-Path $root ".venv\Scripts\python.exe"
$voice = Join-Path $root "out\voice"
$report = Join-Path $root "out\voice\report.txt"

if (-not (Test-Path $exe)) {
    Write-Host "A compilar o Parlo, demora um minuto..."
    cargo build --release -p parlo-cli
    if ($LASTEXITCODE -ne 0) { throw "a compilacao falhou" }
}

New-Item -ItemType Directory -Force -Path $voice | Out-Null
Get-ChildItem -Path $voice -Filter "*.wav" | Remove-Item -Force -ErrorAction SilentlyContinue
$lines = @()

$prompts = @(
    "Apresenta-te: diz o teu nome e o que fazes.",
    "Conta o que fizeste hoje de manha.",
    "Diz uma frase que inclua a palavra Parlo.",
    "Diz um numero, por exemplo trezentos e vinte.",
    "Explica porque queres traduzir em tempo real.",
    "Descreve a sala onde estás.",
    "Pede uma reunião para amanha as dez.",
    "Le em voz alta: o projeto abre o microfone e traduz.",
    "Diz um nome proprio e uma cidade.",
    "Fala sobre a proxima coisa que vais fazer."
)

Write-Host ""
Write-Host "TESTE DE VOZ DO PARLO"
Write-Host "Fala alto e claro, a um palmo do microfone."
Write-Host ""

for ($i = 1; $i -le $Clips; $i++) {
    $prompt = $prompts[($i - 1) % $prompts.Count]
    Write-Host ""
    Write-Host ("Frase {0} de {1}: {2}" -f $i, $Clips, $prompt)
    if (-not $Auto) { Read-Host "Enter para gravar" | Out-Null }
    Write-Host ("A GRAVAR {0} SEGUNDOS... fala agora" -f $Seconds)
    $chunk = & $exe record (Join-Path $voice ("{0:D2}.wav" -f $i)) --seconds $Seconds 2>&1
    $chunk | ForEach-Object { Write-Host $_ }
    $lines += $chunk
    if (($chunk | Out-String) -match "nearly silent") {
        Write-Host ""
        Write-Host "O MICROFONE NAO ESTA A CAPTAR VOZ."
        Write-Host "Antes de repetir, verifica no Windows:"
        Write-Host "  1. Definicoes > Sistema > Som > Entrada: escolhe o microfone certo."
        Write-Host "  2. O volume da entrada nao pode estar a zero, e o mute tem de estar desligado."
        Write-Host "  3. Definicoes > Privacidade > Microfone: permite que as aplicacoes de ambiente de trabalho acedam."
        Write-Host "  4. Fala a um palmo do microfone, sem o tapar com a mao."
        Write-Host "Depois corre TESTAR-VOZ.cmd outra vez."
        $lines | Set-Content -Path $report -Encoding UTF8
        exit 1
    }
}

Write-Host ""
Write-Host ("A comparar Parakeet e Whisper nos teus {0} clipes, uns 40 segundos..." -f $Clips)
$lines += ""
$lines += "=== COMPARACAO ==="
$result = & $python "engines\cascade\scripts\compare.py" $voice --recognisers parakeet whisper --threads 4 2>&1
$result | ForEach-Object { Write-Host $_ }
$lines += $result

if (-not $SkipLive) {
    Write-Host ""
    Write-Host "TESTE AO VIVO"
    Write-Host "A seguir o Parlo abre o microfone durante 20 segundos."
    Write-Host "Fala normalmente, com pausas, para se medir o atraso real."
    if (-not $Auto) { Read-Host "Enter para comecar" | Out-Null }
    $lines += ""
    $lines += "=== AO VIVO ==="
    $live = & $exe listen (Join-Path $root "out\live.wav") --seconds 20 2>&1
    $live | ForEach-Object { Write-Host $_ }
    $lines += $live
}

$lines | Set-Content -Path $report -Encoding UTF8

Write-Host ""
Write-Host "FEITO."
Write-Host ("Relatorio guardado em: {0}" -f $report)
Write-Host "Diz ao agente: ja testei, le o relatorio."
