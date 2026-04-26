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
        $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $defaultKey -Raw
    }
}

if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

$env:CI = "true"

if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    throw "Set TAURI_SIGNING_PRIVATE_KEY before building updater artifacts."
}

function Invoke-TauriBuild {
    param(
        [string]$WorkingDirectory
    )

    $process = Start-Process `
        -FilePath "npx.cmd" `
        -ArgumentList "@tauri-apps/cli", "build" `
        -WorkingDirectory $WorkingDirectory `
        -NoNewWindow `
        -Wait `
        -PassThru

    if ($process.ExitCode -ne 0) {
        throw "Tauri build failed with exit code $($process.ExitCode)."
    }
}

function Invoke-UiChecks {
    param(
        [string]$WorkingDirectory
    )

    $process = Start-Process `
        -FilePath "npm.cmd" `
        -ArgumentList "run", "check:ui" `
        -WorkingDirectory $WorkingDirectory `
        -NoNewWindow `
        -Wait `
        -PassThru

    if ($process.ExitCode -ne 0) {
        throw "UI checks failed with exit code $($process.ExitCode)."
    }
}

if (-not $SkipBuild) {
    Invoke-UiChecks -WorkingDirectory $root
    Invoke-TauriBuild -WorkingDirectory $root
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

function New-PlatformEntry {
    param(
        [string]$Url,
        [string]$Signature
    )

    return [pscustomobject]@{
        url = $Url
        signature = $Signature
    }
}

function Read-NormalizedText {
    param(
        [string]$Path
    )

    return [string]::Concat((Get-Content $Path -Raw))
}

function Write-Utf8NoBom {
    param(
        [string]$Path,
        [string]$Content
    )

    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

$platforms = [ordered]@{}

if ($nsisInstaller) {
    $nsisSigPath = $nsisInstaller.FullName + ".sig"
    if (-not (Test-Path $nsisSigPath)) {
        throw "Missing NSIS updater signature: $nsisSigPath"
    }
    $nsisAssetName = Get-GitHubAssetName $nsisInstaller.Name
    $platforms["windows-x86_64-nsis"] = New-PlatformEntry `
        -Url "https://github.com/$Repo/releases/download/$Tag/$nsisAssetName" `
        -Signature ((Read-NormalizedText -Path $nsisSigPath).Trim())
}

if ($msiInstaller) {
    $msiSigPath = $msiInstaller.FullName + ".sig"
    if (-not (Test-Path $msiSigPath)) {
        throw "Missing MSI updater signature: $msiSigPath"
    }
    $msiAssetName = Get-GitHubAssetName $msiInstaller.Name
    $platforms["windows-x86_64-msi"] = New-PlatformEntry `
        -Url "https://github.com/$Repo/releases/download/$Tag/$msiAssetName" `
        -Signature ((Read-NormalizedText -Path $msiSigPath).Trim())
}

if (-not $UseMsi -and $nsisInstaller) {
    $platforms["windows-x86_64"] = New-PlatformEntry `
        -Url $platforms["windows-x86_64-nsis"].url `
        -Signature $platforms["windows-x86_64-nsis"].signature
} elseif ($msiInstaller) {
    $platforms["windows-x86_64"] = New-PlatformEntry `
        -Url $platforms["windows-x86_64-msi"].url `
        -Signature $platforms["windows-x86_64-msi"].signature
}

$notes = ""
$releaseNotesPath = Join-Path $root "tasks\RELEASE_NOTES.md"
if (Test-Path $releaseNotesPath) {
    $notes = Read-NormalizedText -Path $releaseNotesPath
}

$json = [ordered]@{
    version = $version
    notes = $notes
    pub_date = (Get-Date).ToUniversalTime().ToString("o")
    platforms = $platforms
}

$latestJsonPath = Join-Path $bundleRoot "latest.json"
Write-Utf8NoBom -Path $latestJsonPath -Content ($json | ConvertTo-Json -Depth 8)

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
