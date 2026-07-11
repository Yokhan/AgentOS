param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "debug"
)

$ErrorActionPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $repo "src-tauri\target\$Profile\agent-os.exe"
if (-not (Test-Path -LiteralPath $exe)) {
    throw "Debug binary not found: $exe"
}

$fixture = Join-Path ([System.IO.Path]::GetTempPath()) ("agent-os-ui-e2e-" + [guid]::NewGuid().ToString("N"))
$root = Join-Path $fixture "root"
$data = Join-Path $fixture "data"
$dashboard = Join-Path $root "n8n\dashboard"
$process = $null
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

try {
    New-Item -ItemType Directory -Path $dashboard, $data -Force | Out-Null
    [System.IO.File]::WriteAllText((Join-Path $root "CLAUDE.md"), "# Agent OS E2E fixture", $utf8NoBom)
    $config = @{
        projects = @{}
        permissions = @{}
        documents_dir = $root
    } | ConvertTo-Json
    [System.IO.File]::WriteAllText((Join-Path $root "n8n\config.json"), $config, $utf8NoBom)
    [System.IO.File]::WriteAllText((Join-Path $dashboard "segments.json"), '{"segments":[]}', $utf8NoBom)

    $env:AGENT_OS_ROOT = $root
    $env:AGENT_OS_DATA_DIR = $data
    $env:AGENT_OS_E2E = "1"
    $process = Start-Process -FilePath $exe -WorkingDirectory $repo -PassThru

    $reportPath = Join-Path $data "tasks\.ui-diagnostics.jsonl"
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    $report = $null
    while ([DateTime]::UtcNow -lt $deadline -and -not $report) {
        if ($process.HasExited) {
            throw "Agent OS exited before producing an E2E report (exit $($process.ExitCode))"
        }
        if (Test-Path -LiteralPath $reportPath) {
            $entries = Get-Content -LiteralPath $reportPath | ForEach-Object { $_ | ConvertFrom-Json }
            $report = $entries | Where-Object { $_.kind -eq "e2e_report" } | Select-Object -Last 1
        }
        if (-not $report) { Start-Sleep -Milliseconds 250 }
    }

    if (-not $report) { throw "Timed out waiting for the Agent OS E2E report" }
    $event = $report.event
    $event | ConvertTo-Json -Depth 12
    if (-not $event.passed) { throw "Agent OS UI E2E failed" }
    Write-Host "ui e2e ok: startup=$($event.startup.readyMs)ms startup-health-p95=$($event.apiLatencyMs.startupLoad.p95)ms steady-health-p95=$($event.apiLatencyMs.steady.p95)ms steps=$($event.steps.Count)"
}
finally {
    if ($process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit(5000) | Out-Null
    }
    Remove-Item Env:AGENT_OS_ROOT -ErrorAction SilentlyContinue
    Remove-Item Env:AGENT_OS_DATA_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:AGENT_OS_E2E -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $fixture) {
        Remove-Item -LiteralPath $fixture -Recurse -Force
    }
}
