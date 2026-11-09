# Stops a running OpenDeck and starts `deno task tauri dev`, writing all output to a log file.
# Run it in the background and wait for "Virtual deck window loaded" in the log.
# Usage: powershell -NoProfile -File dev.ps1 [-Log <path>] [-Repo <path>]
param(
  [string]$Log = (Join-Path $env:TEMP "opendeck-dev.log"),
  [string]$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
)

# A fresh shell may not know cargo/deno yet after installing them.
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')

# OpenDeck is single-instance: a second start would only focus the first one.
Get-Process opendeck, opendeck-starterpack -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

"Starting OpenDeck dev build from $Repo, log: $Log"
Set-Location $Repo
deno task tauri dev *>&1 | Out-File -Encoding utf8 $Log
