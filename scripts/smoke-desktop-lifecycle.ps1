param(
    [string]$Executable = "src-tauri/target/debug/agent-os.exe",
    [int]$TimeoutSeconds = 45
)

$ErrorActionPreference = "Stop"
$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$exe = (Resolve-Path (Join-Path $workspaceRoot $Executable)).Path
$fixtureRoot = Join-Path $env:TEMP ("agentos-desktop-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot "n8n/dashboard") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot "tasks") | Out-Null
Set-Content -Path (Join-Path $fixtureRoot "CLAUDE.md") -Value "# AgentOS smoke fixture" -Encoding UTF8
Set-Content -Path (Join-Path $fixtureRoot "n8n/config.json") -Value '{"documents_dir":""}' -Encoding UTF8
Set-Content -Path (Join-Path $fixtureRoot "n8n/dashboard/segments.json") -Value '{"segments":{}}' -Encoding UTF8
$env:AGENT_OS_ROOT = $fixtureRoot
$env:AGENT_OS_DATA_DIR = (Join-Path $fixtureRoot "runtime")
$process = Start-Process -FilePath $exe -WorkingDirectory $fixtureRoot -WindowStyle Hidden -PassThru

try {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $health = $null
    while ((Get-Date) -lt $deadline) {
        if ($process.HasExited) {
            throw "Agent OS exited during startup with code $($process.ExitCode)"
        }
        try {
            $health = Invoke-RestMethod -Uri "http://127.0.0.1:3333/api/health" -TimeoutSec 2
            if ($health.status -eq "ok") {
                break
            }
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    if ($null -eq $health -or $health.status -ne "ok") {
        throw "Agent OS health endpoint did not become ready in $TimeoutSeconds seconds"
    }
    $expectedDataDir = (Join-Path $fixtureRoot "runtime")
    if ($health.data_dir -ne $expectedDataDir) {
        throw "Agent OS used unexpected runtime data directory: $($health.data_dir)"
    }
    if (-not (Test-Path (Join-Path $expectedDataDir "config.json"))) {
        throw "Agent OS did not migrate config into isolated runtime storage"
    }
    Write-Host "desktop lifecycle smoke ok: uptime=$($health.uptime) projects=$($health.projects)"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
        $process.WaitForExit(5000) | Out-Null
    }
    Remove-Item -LiteralPath $fixtureRoot -Recurse -Force -ErrorAction SilentlyContinue
}
