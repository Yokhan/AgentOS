# Build signed Tauri updater artifacts and emit a latest.json for GitHub Releases.
#
# Usage examples:
#   powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1 -Tag v0.2.0

param(
    [string]$Tag = "",
    [string]$Repo = "Yokhan/AgentOS",
    [switch]$UseMsi,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$tauriConf = Join-Path $root "src-tauri\tauri.conf.json"
$conf = Get-Content $tauriConf | ConvertFrom-Json
$version = $conf.version
if ([string]::IsNullOrWhiteSpace($Tag)) {
    $Tag = "v$version"
}

if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    $defaultKey = Join-Path $HOME ".tauri\agent-os.key"
    if (Test-Path $defaultKey) {
        $env:TAURI_SIGNING_PRIVATE_KEY = $defaultKey
    }
}

if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

$env:CI = "true"

if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    throw "TAURI_SIGNING_PRIVATE_KEY is not set and ~/.tauri/agent-os.key was not found."
}

if (-not $SkipBuild) {
    Push-Location $root
    try {
        cmd /c npm run build | Out-Host
    } finally {
        Pop-Location
    }
}

$bundleRoot = Join-Path $root "src-tauri\target\release\bundle"
$nsisInstaller = Get-ChildItem -Path (Join-Path $bundleRoot "nsis") -Filter *.exe -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$msiInstaller = Get-ChildItem -Path (Join-Path $bundleRoot "msi") -Filter *.msi -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1

if (-not $nsisInstaller -and -not $msiInstaller) {
    throw "No installer artifacts were found under $bundleRoot."
}

function Get-GitHubAssetName([string]$name) {
    return ($name -replace ' ', '.')
}

$platforms = [ordered]@{}

if ($nsisInstaller) {
    $nsisSigPath = $nsisInstaller.FullName + ".sig"
    if (-not (Test-Path $nsisSigPath)) {
        throw "Missing NSIS updater signature: $nsisSigPath"
    }
    $nsisAssetName = Get-GitHubAssetName $nsisInstaller.Name
    $platforms["windows-x86_64-nsis"] = [ordered]@{
        url = "https://github.com/$Repo/releases/download/$Tag/$nsisAssetName"
        signature = (Get-Content $nsisSigPath -Raw).Trim()
    }
}

if ($msiInstaller) {
    $msiSigPath = $msiInstaller.FullName + ".sig"
    if (-not (Test-Path $msiSigPath)) {
        throw "Missing MSI updater signature: $msiSigPath"
    }
    $msiAssetName = Get-GitHubAssetName $msiInstaller.Name
    $platforms["windows-x86_64-msi"] = [ordered]@{
        url = "https://github.com/$Repo/releases/download/$Tag/$msiAssetName"
        signature = (Get-Content $msiSigPath -Raw).Trim()
    }
}

if (-not $UseMsi -and $nsisInstaller) {
    $platforms["windows-x86_64"] = $platforms["windows-x86_64-nsis"]
} elseif ($msiInstaller) {
    $platforms["windows-x86_64"] = $platforms["windows-x86_64-msi"]
}

$notes = ""
$releaseNotesPath = Join-Path $root "tasks\RELEASE_NOTES.md"
if (Test-Path $releaseNotesPath) {
    $notes = Get-Content $releaseNotesPath -Raw
}

$json = [ordered]@{
    version = $version
    notes = $notes
    pub_date = (Get-Date).ToUniversalTime().ToString("o")
    platforms = $platforms
}

$latestJsonPath = Join-Path $bundleRoot "latest.json"
$json | ConvertTo-Json -Depth 8 | Set-Content -Path $latestJsonPath -Encoding UTF8

Write-Host ""
Write-Host "Prepared updater release manifest:"
if ($nsisInstaller) {
    Write-Host "  NSIS:       $($nsisInstaller.FullName)"
    Write-Host "  NSIS .sig:  $($nsisInstaller.FullName).sig"
}
if ($msiInstaller) {
    Write-Host "  MSI:        $($msiInstaller.FullName)"
    Write-Host "  MSI .sig:   $($msiInstaller.FullName).sig"
}
Write-Host "  latest.json $latestJsonPath"
Write-Host ""
Write-Host "Upload these release assets to GitHub tag ${Tag}:"
if ($nsisInstaller) {
    Write-Host "  - $($nsisInstaller.Name)"
    Write-Host "  - $($nsisInstaller.Name).sig"
}
if ($msiInstaller) {
    Write-Host "  - $($msiInstaller.Name)"
    Write-Host "  - $($msiInstaller.Name).sig"
}
Write-Host "  - latest.json"
