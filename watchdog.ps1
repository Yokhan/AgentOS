# Agent OS Watchdog — runs independently, updates even if app is broken
# Install: create a scheduled task that runs this every 6 hours
# Or: run manually when app won't start

param(
    [switch]$Install,  # Install as scheduled task
    [switch]$Check,    # Just check, don't update
    [switch]$Force     # Force update even if current version matches
)

$ErrorActionPreference = "Continue"
$repo = "Yokhan/AgentOS"
$appDir = $PSScriptRoot
$configFile = Join-Path $appDir "n8n\config.json"
$versionFile = Join-Path $appDir "src-tauri\tauri.conf.json"

function Get-CurrentVersion {
    if (Test-Path $versionFile) {
        $conf = Get-Content $versionFile | ConvertFrom-Json
        return $conf.version
    }
    return "0.0.0"
}

function Get-LatestRelease {
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -TimeoutSec 10
        return @{
            Version = $release.tag_name -replace '^v', ''
            Url = ($release.assets | Where-Object { $_.name -match '\.msi$|\.exe$|nsis' } | Select-Object -First 1).browser_download_url
            Notes = $release.body
        }
    } catch {
        Write-Host "Cannot reach GitHub: $_"
        return $null
    }
}

function Update-App {
    param($Url, $Version)

    $tempDir = Join-Path $env:TEMP "agent-os-update"
    $installer = Join-Path $tempDir "agent-os-$Version-setup.exe"

    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    Write-Host "Downloading $Version..."
    try {
        Invoke-WebRequest -Uri $Url -OutFile $installer -TimeoutSec 120
    } catch {
        Write-Host "Download failed: $_"
        return $false
    }

    # Kill running app
    Get-Process -Name "Agent OS" -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 2

    # Run installer silently
    Write-Host "Installing..."
    Start-Process -FilePath $installer -ArgumentList "/S" -Wait -NoNewWindow

    # Cleanup
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host "Updated to $Version"
    return $true
}

function Install-ScheduledTask {
    $taskName = "AgentOS-Watchdog"
    $scriptPath = $PSCommandPath

    # Remove existing
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-ExecutionPolicy Bypass -WindowStyle Hidden -File `"$scriptPath`""
    $trigger = New-ScheduledTaskTrigger -Daily -At "09:00" -RepetitionInterval (New-TimeSpan -Hours 6)
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Settings $settings -Description "Agent OS auto-updater watchdog"

    Write-Host "Scheduled task '$taskName' installed (every 6 hours)"
}

# Main
if ($Install) {
    Install-ScheduledTask
    return
}

$current = Get-CurrentVersion
$latest = Get-LatestRelease

if (-not $latest) {
    Write-Host "Cannot check for updates (offline?)"
    exit 1
}

Write-Host "Current: $current, Latest: $($latest.Version)"

if ($Force -or ([version]$latest.Version -gt [version]$current)) {
    if ($Check) {
        Write-Host "Update available: $($latest.Version)"
        Write-Host $latest.Notes
        exit 0
    }

    if ($latest.Url) {
        $success = Update-App -Url $latest.Url -Version $latest.Version
        if ($success) {
            # Relaunch app
            $exe = Get-ChildItem -Path "$env:LOCALAPPDATA\Agent OS" -Filter "Agent OS.exe" -Recurse | Select-Object -First 1
            if ($exe) {
                Start-Process -FilePath $exe.FullName
                Write-Host "App relaunched"
            }
        }
    } else {
        Write-Host "No installer found in release assets"
    }
} else {
    Write-Host "Up to date"
}
