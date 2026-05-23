# Collect Agent OS hang diagnostics into one timestamped folder.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/collect-hang-diagnostics.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/collect-hang-diagnostics.ps1 -SinceHours 6

param(
    [int]$SinceHours = 24,
    [string]$OutDir = "",
    [int]$TailLines = 800
)

$ErrorActionPreference = "Continue"

$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $OutDir = Join-Path $root "tasks\diagnostics\hang-$stamp"
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

function Write-JsonFile {
    param(
        [string]$Path,
        [object]$Value,
        [int]$Depth = 8
    )
    $Value | ConvertTo-Json -Depth $Depth | Set-Content -Path $Path -Encoding UTF8
}

function Write-TextFile {
    param(
        [string]$Path,
        [object]$Value
    )
    $Value | Out-String -Width 240 | Set-Content -Path $Path -Encoding UTF8
}

function Try-Collect {
    param(
        [string]$Name,
        [scriptblock]$Block
    )
    try {
        & $Block
        return @{ name = $Name; status = "ok" }
    } catch {
        return @{ name = $Name; status = "error"; error = $_.Exception.Message }
    }
}

$since = (Get-Date).AddHours(-1 * $SinceHours)
$summary = [ordered]@{
    generated_at = (Get-Date).ToUniversalTime().ToString("o")
    root = $root
    since_hours = $SinceHours
    out_dir = $OutDir
    collection = @()
    findings = [ordered]@{}
}

$summary.collection += Try-Collect "environment" {
    $installed = "C:\Users\iohan\AppData\Local\Agent OS\agent-os.exe"
    $tauriConf = Join-Path $root "src-tauri\tauri.conf.json"
    $envInfo = [ordered]@{
        powershell = $PSVersionTable.PSVersion.ToString()
        os = (Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, LastBootUpTime)
        gpu = (Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion, DriverDate, AdapterRAM)
        installed_agent_os = if (Test-Path $installed) { Get-Item $installed | Select-Object FullName, Length, LastWriteTime, VersionInfo } else { $null }
        repo_version = if (Test-Path $tauriConf) { (Get-Content $tauriConf -Raw | ConvertFrom-Json).version } else { "" }
        git_head = (& git -C $root rev-parse --short HEAD 2>$null)
        git_status = (& git -C $root status --short 2>$null)
        free_space = (Get-PSDrive -Name C | Select-Object Name, Free, Used)
    }
    Write-JsonFile (Join-Path $OutDir "environment.json") $envInfo 12
}

$summary.collection += Try-Collect "processes" {
    $names = @("agent-os", "msedgewebview2", "codex", "claude", "node", "cargo")
    $proc = foreach ($name in $names) {
        Get-Process -Name $name -ErrorAction SilentlyContinue |
            Select-Object Id, ProcessName, CPU, WorkingSet64, Responding, StartTime, Path
    }
    $cim = Get-CimInstance Win32_Process |
        Where-Object { $_.Name -match 'agent-os|msedgewebview2|codex|claude|node|cargo' } |
        Select-Object ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine
    Write-JsonFile (Join-Path $OutDir "processes.json") @{ processes = $proc; command_lines = $cim } 8
}

$summary.collection += Try-Collect "application_events" {
    $events = Get-WinEvent -FilterHashtable @{ LogName = "Application"; StartTime = $since } -ErrorAction SilentlyContinue |
        Where-Object {
            $_.ProviderName -match 'Application Hang|Application Error|Windows Error Reporting|MsiInstaller' -or
            $_.Message -match 'agent-os|Agent OS|msedgewebview2|AppHangB1|AppHangTransient|LiveKernelEvent|WebView|AMD_WATCHDOG'
        } |
        Sort-Object TimeCreated -Descending |
        Select-Object TimeCreated, ProviderName, Id, LevelDisplayName, @{n = "Message"; e = { $_.Message }}
    Write-JsonFile (Join-Path $OutDir "events-application.json") $events 6
    $summary.findings.application_hang_count = @($events | Where-Object { $_.ProviderName -eq "Application Hang" -or $_.Message -match "AppHang" }).Count
    $summary.findings.application_watchdog_count = @($events | Where-Object { $_.Message -match "WATCHDOG|LiveKernelEvent|TDR|display driver|GPU|amdkmdag" }).Count
}

$summary.collection += Try-Collect "system_events" {
    $events = Get-WinEvent -FilterHashtable @{ LogName = "System"; StartTime = $since } -ErrorAction SilentlyContinue |
        Where-Object {
            $_.ProviderName -match 'Display|WHEA|Kernel-Power|DxgKrnl|Microsoft-Windows-WER-SystemErrorReporting' -or
            $_.Message -match 'display driver|LiveKernelEvent|GPU|TDR|reset|amdkmdag|AMD_WATCHDOG|WATCHDOG|bugcheck|unexpectedly shut down'
        } |
        Sort-Object TimeCreated -Descending |
        Select-Object TimeCreated, ProviderName, Id, LevelDisplayName, @{n = "Message"; e = { $_.Message }}
    Write-JsonFile (Join-Path $OutDir "events-system.json") $events 6
    $summary.findings.system_watchdog_count = @($events | Where-Object { $_.Message -match "WATCHDOG|LiveKernelEvent|TDR|display driver|GPU|amdkmdag" }).Count
}

$summary.collection += Try-Collect "wer_index" {
    $roots = @(
        "C:\ProgramData\Microsoft\Windows\WER\ReportArchive",
        "C:\ProgramData\Microsoft\Windows\WER\ReportQueue"
    )
    $items = foreach ($werRoot in $roots) {
        if (-not (Test-Path $werRoot)) { continue }
        Get-ChildItem $werRoot -Directory -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -match 'agent-os|Kernel_|LiveKernel|WATCHDOG|WebView|msedge' } |
            ForEach-Object {
                $report = Join-Path $_.FullName "Report.wer"
                $accessible = $false
                $firstLines = @()
                try {
                    $reportItem = Get-Item $report -ErrorAction Stop
                    if ($reportItem) {
                        $accessible = $true
                    }
                    $firstLines = Get-Content $report -TotalCount 80 -ErrorAction Stop
                } catch {
                    $accessible = $false
                }
                [pscustomobject]@{
                    FullName = $_.FullName
                    Name = $_.Name
                    LastWriteTime = $_.LastWriteTime
                    ReportWerAccessible = $accessible
                    ReportWerHead = $firstLines
                }
            }
    }
    Write-JsonFile (Join-Path $OutDir "wer-index.json") $items 8
    $summary.findings.wer_agentos_reports = @($items | Where-Object { $_.Name -match "agent-os" }).Count
}

$summary.collection += Try-Collect "live_kernel_reports" {
    $items = if (Test-Path "C:\WINDOWS\LiveKernelReports") {
        Get-ChildItem "C:\WINDOWS\LiveKernelReports" -Recurse -ErrorAction SilentlyContinue |
            Select-Object FullName, Length, LastWriteTime |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 80
    } else {
        @()
    }
    Write-JsonFile (Join-Path $OutDir "live-kernel-reports.json") $items 5
}

$summary.collection += Try-Collect "agentos_logs" {
    $files = @(
        "tasks\agent-os.log",
        "tasks\.ui-diagnostics.jsonl",
        "tasks\.operations.jsonl",
        "tasks\.delegation-log.jsonl",
        "tasks\.notifications.jsonl",
        "tasks\.signals.jsonl"
    )
    foreach ($rel in $files) {
        $path = Join-Path $root $rel
        if (Test-Path $path) {
            Get-Content $path -Tail $TailLines -ErrorAction SilentlyContinue |
                Set-Content -Path (Join-Path $OutDir (($rel -replace '[\\/:*?"<>|]', "_") + ".tail.txt")) -Encoding UTF8
        }
    }
}

$summary.collection += Try-Collect "classification" {
    $appEventsPath = Join-Path $OutDir "events-application.json"
    $sysEventsPath = Join-Path $OutDir "events-system.json"
    $uiDiagPath = Join-Path $root "tasks\.ui-diagnostics.jsonl"
    $agentLogPath = Join-Path $root "tasks\agent-os.log"

    $appEventsText = if (Test-Path $appEventsPath) { Get-Content $appEventsPath -Raw } else { "" }
    $sysEventsText = if (Test-Path $sysEventsPath) { Get-Content $sysEventsPath -Raw } else { "" }
    $appEvents = if ($appEventsText) { @($appEventsText | ConvertFrom-Json) } else { @() }
    $sysEvents = if ($sysEventsText) { @($sysEventsText | ConvertFrom-Json) } else { @() }
    $uiText = if (Test-Path $uiDiagPath) { (Get-Content $uiDiagPath -Tail $TailLines) -join "`n" } else { "" }
    $agentText = if (Test-Path $agentLogPath) { (Get-Content $agentLogPath -Tail $TailLines) -join "`n" } else { "" }

    function Convert-EventTime {
        param([object]$Value)
        if ($Value -is [datetime]) { return $Value }
        $text = [string]$Value
        if ($text -match '/Date\((\d+)\)/') {
            return [DateTimeOffset]::FromUnixTimeMilliseconds([int64]$matches[1]).LocalDateTime
        }
        try { return [datetime]$text } catch { return $null }
    }

    $appHangEvents = @($appEvents | Where-Object {
            $_.ProviderName -eq "Application Hang" -or $_.Message -match "agent-os.exe.*AppHang|AppHangB1"
        })
    $watchdogEvents = @($appEvents + $sysEvents | Where-Object {
            $_.Message -match "LiveKernelEvent|AMD_WATCHDOG|WATCHDOG|TDR|display driver|GPU|amdkmdag"
        })
    $watchdogNearAppHang = $false
    foreach ($hang in $appHangEvents) {
        $hangTime = Convert-EventTime $hang.TimeCreated
        if (-not $hangTime) { continue }
        foreach ($watchdog in $watchdogEvents) {
            $watchdogTime = Convert-EventTime $watchdog.TimeCreated
            if (-not $watchdogTime) { continue }
            if ([Math]::Abs(($watchdogTime - $hangTime).TotalMinutes) -le 5) {
                $watchdogNearAppHang = $true
                break
            }
        }
        if ($watchdogNearAppHang) { break }
    }

    $signals = [ordered]@{
        agentos_apphang = @($appHangEvents).Count -gt 0
        ui_main_thread_lag = $uiText -match "event_loop_lag|long_task"
        ui_js_error = $uiText -match "window_error|unhandled_rejection"
        kernel_or_gpu_watchdog = @($watchdogEvents).Count -gt 0
        kernel_or_gpu_watchdog_near_apphang = $watchdogNearAppHang
        multiple_agentos_instances = $agentText -match "3334|single-instance guard port 3329 is busy"
        backend_started_recently = $agentText -match "Agent OS v[0-9.]+ starting"
    }

    $verdict = "unknown"
    if ($signals.kernel_or_gpu_watchdog_near_apphang -and $signals.agentos_apphang) {
        $verdict = "mixed_apphang_with_kernel_or_gpu_watchdog"
    } elseif ($signals.ui_main_thread_lag -and $signals.agentos_apphang) {
        $verdict = "probable_ui_main_thread_starvation"
    } elseif ($signals.agentos_apphang) {
        $verdict = "apphang_without_ui_or_kernel_proof"
    } elseif ($signals.kernel_or_gpu_watchdog) {
        $verdict = "kernel_or_gpu_watchdog_without_agentos_apphang"
    }

    $classification = [ordered]@{
        verdict = $verdict
        signals = $signals
        rules = @(
            "event_loop_lag or long_task near freeze => UI/WebView main-thread starvation",
            "agent-os AppHangB1 without UI diagnostics => native/WebView/backend stall still possible",
            "LiveKernelEvent/AMD_WATCHDOG near freeze => GPU/driver/OS layer involved",
            "backend logs continue while UI is frozen => UI/WebView is more likely",
            "backend logs stop before UI freezes => backend/native loop or process deadlock is more likely"
        )
    }
    Write-JsonFile (Join-Path $OutDir "classification.json") $classification 8
    $summary.findings.verdict = $verdict
    $summary.findings.signals = $signals
}

Write-JsonFile (Join-Path $OutDir "summary.json") $summary 8

Write-Host ""
Write-Host "Agent OS hang diagnostics collected:"
Write-Host "  $OutDir"
Write-Host ""
Write-Host "Key files:"
Write-Host "  summary.json"
Write-Host "  classification.json"
Write-Host "  events-application.json"
Write-Host "  events-system.json"
Write-Host "  wer-index.json"
Write-Host "  tasks_agent-os.log.tail.txt"
