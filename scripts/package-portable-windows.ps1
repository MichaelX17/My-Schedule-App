<#
.SYNOPSIS
    Builds reproducible portable Windows release artifacts for My Schedule App.

.DESCRIPTION
    1. Builds the production frontend bundle (`npm run build`).
    2. Builds the standalone Tauri 2 Windows release executable (`npm run tauri build -- --no-bundle`).
    3. Stages a clean portable folder containing ONLY `MyScheduleApp.exe` and `PORTABLE_README.txt`
       (never includes a `data/` directory so upgrading over an existing portable folder preserves user data).
    4. Produces a ZIP archive and SHA-256 checksum file in `release-artifacts/`.

.EXAMPLE
    pwsh -File scripts/package-portable-windows.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $RepoRoot

$TauriConf = Get-Content (Join-Path $RepoRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$Version = $TauriConf.version
if (-not $Version) {
    throw "Unable to read version from src-tauri/tauri.conf.json"
}

Write-Host "==> Building My Schedule App v$Version (Windows Portable)" -ForegroundColor Cyan

npm ci
npm run typecheck
npm run lint
npm run test
npm run tauri build -- --no-bundle

$ReleaseExe = Join-Path $RepoRoot "src-tauri/target/release/my-schedule-app.exe"
if (-not (Test-Path $ReleaseExe)) {
    throw "Expected release executable not found at $ReleaseExe"
}

$ArtifactsRoot = Join-Path $RepoRoot "release-artifacts"
$PackageName = "MyScheduleApp-v$Version-windows-x64-portable"
$StagingDir = Join-Path $ArtifactsRoot $PackageName
$ZipPath = Join-Path $ArtifactsRoot "$PackageName.zip"
$ShaPath = "$ZipPath.sha256"

if (Test-Path $StagingDir) { Remove-Item -Recurse -Force $StagingDir }
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
if (Test-Path $ShaPath) { Remove-Item -Force $ShaPath }

New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null
Copy-Item $ReleaseExe (Join-Path $StagingDir "MyScheduleApp.exe")

@"
My Schedule App v$Version — Portable Windows Edition
====================================================

1. Extract this folder to any permanent, writable location (for example:
   C:\Apps\MyScheduleApp or a USB drive). Do NOT run directly from inside
   a ZIP preview / temporary folder.

2. Double-click MyScheduleApp.exe to launch.
   - When the folder is writable, all data is stored beside the executable in:
       data\database.sqlite
       data\backups\automatic\
       data\backups\manual\
       data\exports\
       data\logos\
       data\logs\
       data\updates\
   - If the folder is read-only (e.g. Program Files) or inside a temporary
     directory, My Schedule App automatically falls back to your per-user
     AppData folder and displays the active storage mode in Settings.

3. Upgrading to a newer version:
   - Close My Schedule App.
   - Replace ONLY MyScheduleApp.exe with the newer version.
   - Your existing data\ folder is never included in release ZIPs and will
     be preserved and migrated safely on next launch.

4. Prerequisite:
   - Microsoft Edge WebView2 Runtime (pre-installed on Windows 11 and most
     updated Windows 10 systems).
"@ | Set-Content -Encoding UTF8 (Join-Path $StagingDir "PORTABLE_README.txt")

Compress-Archive -Path (Join-Path $StagingDir "*") -DestinationPath $ZipPath -CompressionLevel Optimal

$Hash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$Hash  $PackageName.zip" | Set-Content -Encoding ASCII $ShaPath

Write-Host "==> Created portable archive: $ZipPath" -ForegroundColor Green
Write-Host "==> SHA-256: $Hash" -ForegroundColor Green
