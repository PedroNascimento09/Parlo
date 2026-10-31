param(
    [int]$Clips = 10,
    [int]$Seconds = 6,
    [switch]$Auto,
    [switch]$SkipLive,
    [Nullable[int]]$Device = $null
)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$exe = Join-Path $root "target\release\parlo.exe"
$python = Join-Path $root ".venv\Scripts\python.exe"
$voice = Join-Path $root "out\voice"
$report = Join-Path $root "out\voice\report.txt"

Write-Host "A compilar o Parlo, demora um minuto na primeira vez..."
cargo build --release -p parlo-cli
if ($LASTEXITCODE -ne 0) { throw "a compilacao falhou" }

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
Write-Host "Fala em voz normal, a um palmo do microfone."
Write-Host ""

$chosen = $Device
if ($null -eq $chosen) {
    Write-Host "A procurar o microfone: fala em voz normal durante a sonda de 3 segundos por dispositivo."
    Write-Host ""
    $probe = & $exe record --probe --seconds 3 2>&1
    $probe | ForEach-Object { Write-Host $_ }
    $lines += $probe
    $best = -120.0
    $bestIndex = $null
    foreach ($line in $probe) {
        if ($line -match "^\s*(\d+)\s+.*?(-?\d+\.\d)\s*dBFS") {
            $level = [double]$matches[2]
            if ($level -gt $best) { $best = $level; $bestIndex = [int]$matches[1] }
        }
    }
    if (($null -eq $bestIndex) -or ($best -lt -45)) {
        Write-Host ""
        Write-Host "NENHUM DISPOSITIVO OUVIU VOZ."
        Write-Host "Fala durante a sonda, e verifica Definicoes > Sistema > Som > Entrada."
        Write-Host "Se o Discord te ouve, diz ao agente o resultado da sonda acima."
        $lines | Set-Content -Path $report -Encoding UTF8
        exit 1
    }
    Write-Host ("A usar o dispositivo {0} com {1} dBFS" -f $bestIndex, $best)
    $chosen = $bestIndex
}

$rows = @()
for ($i = 1; $i -le $Clips; $i++) {
    $prompt = $prompts[($i - 1) % $prompts.Count]
    Write-Host ""
    Write-Host ("Frase {0} de {1}: {2}" -f $i, $Clips, $prompt)
    if (-not $Auto) { Read-Host "Enter para gravar" | Out-Null }
    Write-Host ("A GRAVAR {0} SEGUNDOS... fala agora" -f $Seconds)
    $chunk = & $exe record (Join-Path $voice ("{0:D2}.wav" -f $i)) --seconds $Seconds --device $chosen 2>&1
    $chunk | ForEach-Object { Write-Host $_ }
    $lines += $chunk
    $output = $chunk | Out-String
    if ($output -match "captured almost nothing") {
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
    $gainText = ""
    if ($output -match "gain\s+(\+[0-9.]+) dB") {
        $gainText = $matches[1] + " dB"
        Write-Host ("O microfone e baixo, a gravacao foi amplificada em {0} dB." -f $matches[1])
    }
    if ($output -match "peak\s+(-?[0-9.]+) dBFS") {
        $rows += [pscustomobject]@{ Index = $i; Peak = $matches[1]; Value = [double]$matches[1]; Gain = $gainText }
    }
}

Write-Host ""
Write-Host "NIVEIS DOS CLIPES"
foreach ($row in $rows) {
    Write-Host ("  {0:D2}  {1,6} dBFS  {2}" -f $row.Index, $row.Peak, $row.Gain)
}
if ($rows.Count -gt 0) {
    $quietest = $rows | Sort-Object Value | Select-Object -First 1
    $loudest = $rows | Sort-Object Value | Select-Object -Last 1
    Write-Host ("  mais baixo: clipe {0:D2} com {1} dBFS, mais alto: clipe {2:D2} com {3} dBFS" -f $quietest.Index, $quietest.Peak, $loudest.Index, $loudest.Peak)
    $boosted = @($rows | Where-Object { $_.Gain })
    if ($boosted.Count -eq $rows.Count) {
        Write-Host "  O microfone e baixo: os clipes foram amplificados, por isso soam tao alto como no Discord."
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
    $voiced = 0
    $translated = 0.0
    foreach ($line in $live) {
        if ($line -match "voiced frames\s+(\d+)") { $voiced = [int]$matches[1] }
        if ($line -match "translated audio\s+([0-9.]+) s") { $translated = [double]$matches[1] }
    }
    Write-Host ""
    Write-Host ("RESUMO AO VIVO: {0} frames de voz, {1} s de audio traduzido." -f $voiced, $translated)
    $lines += ("RESUMO AO VIVO: {0} frames de voz, {1} s de audio traduzido." -f $voiced, $translated)
    if ($translated -le 0.0) {
        Write-Host "NADA CHEGOU AO MOTOR: o teste ao vivo nao mediu atraso nenhum."
        $lines += "NADA CHEGOU AO MOTOR: o teste ao vivo nao mediu atraso nenhum."
    }
}

$lines | Set-Content -Path $report -Encoding UTF8

Write-Host ""
Write-Host "FEITO."
Write-Host ("Relatorio guardado em: {0}" -f $report)
Write-Host "Diz ao agente: ja testei, le o relatorio."
