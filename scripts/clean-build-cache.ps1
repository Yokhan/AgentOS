# Keep local Rust/Tauri build cache from eating the disk.
#
# Default policy:
# - if src-tauri/target is over 30 GiB, run cargo clean;
# - if free space on the repo drive is under 12 GiB, run cargo clean;
# - print before/after sizes so release logs show what happened.

param(
    [double]$MaxTargetGB = 30,
    [double]$MinFreeGB = 12,
    [switch]$Always
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "src-tauri\Cargo.toml"
$target = Join-Path $root "src-tauri\target"

function Get-DirectorySizeBytes {
    param([string]$Path)

    if (-not (Test-Path $Path)) {
        return 0
    }

    $sum = (Get-ChildItem -Path $Path -Recurse -Force -ErrorAction SilentlyContinue |
        Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) {
        return 0
    }
    return [double]$sum
}

function Format-GB {
    param([double]$Bytes)
    return [math]::Round($Bytes / 1GB, 2)
}

$targetBytes = Get-DirectorySizeBytes -Path $target
$targetGB = Format-GB $targetBytes
$driveRoot = [System.IO.Path]::GetPathRoot($root)
$driveName = $driveRoot.Substring(0, 1)
$drive = Get-PSDrive -Name $driveName
$freeGB = [math]::Round($drive.Free / 1GB, 2)

Write-Host "[cache] src-tauri/target: ${targetGB} GiB; drive ${driveName}: free ${freeGB} GiB"

$overTarget = $targetGB -gt $MaxTargetGB
$lowFree = $freeGB -lt $MinFreeGB

if (-not $Always -and -not $overTarget -and -not $lowFree) {
    Write-Host "[cache] ok: below ${MaxTargetGB} GiB and free space above ${MinFreeGB} GiB"
    exit 0
}

$reasons = @()
if ($Always) { $reasons += "forced" }
if ($overTarget) { $reasons += "target ${targetGB} GiB > ${MaxTargetGB} GiB" }
if ($lowFree) { $reasons += "free ${freeGB} GiB < ${MinFreeGB} GiB" }
Write-Host "[cache] cleaning cargo target: $($reasons -join '; ')"

$process = Start-Process `
    -FilePath "cargo" `
    -ArgumentList "clean", "--manifest-path", $manifest `
    -WorkingDirectory $root `
    -NoNewWindow `
    -Wait `
    -PassThru

if ($process.ExitCode -ne 0) {
    throw "cargo clean failed with exit code $($process.ExitCode)"
}

$afterBytes = Get-DirectorySizeBytes -Path $target
$afterGB = Format-GB $afterBytes
$driveAfter = Get-PSDrive -Name $driveName
$freeAfterGB = [math]::Round($driveAfter.Free / 1GB, 2)
Write-Host "[cache] cleaned: target ${afterGB} GiB; drive ${driveName}: free ${freeAfterGB} GiB"
