<#
.SYNOPSIS
    Builds a portable, self-contained release of MicMuteOverlay.

.DESCRIPTION
    Compiles the release binary, assembles a distributable folder
    (exe + Sounds + default config.json), and zips it. The resulting
    executable is fully self-contained: no .NET, no Visual C++ runtime,
    and no installer are required - just unzip and run.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1
#>
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    Write-Host "Building release binary..." -ForegroundColor Cyan
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

    $exe = Join-Path $root "target\release\micmuteoverlay.exe"
    if (-not (Test-Path $exe)) { throw "Build did not produce $exe" }

    $distRoot = Join-Path $root "dist"
    $out = Join-Path $distRoot "MicMuteOverlay"
    if (Test-Path $out) { Remove-Item $out -Recurse -Force }
    New-Item -ItemType Directory -Path $out | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $out "Sounds") | Out-Null

    Copy-Item $exe (Join-Path $out "MicMuteOverlay.exe")
    Copy-Item (Join-Path $root "assets\mute.wav")   (Join-Path $out "Sounds\mute.wav")
    Copy-Item (Join-Path $root "assets\unmute.wav") (Join-Path $out "Sounds\unmute.wav")
    Copy-Item (Join-Path $root "config.example.json") (Join-Path $out "config.json")

    $zip = Join-Path $distRoot "MicMuteOverlay.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path (Join-Path $out "*") -DestinationPath $zip

    Write-Host ""
    Write-Host "Release ready:" -ForegroundColor Green
    Write-Host "  Folder: $out"
    Write-Host "  Zip:    $zip"
}
finally {
    Pop-Location
}
