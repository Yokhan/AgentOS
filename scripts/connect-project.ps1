param(
    [string]$Project = "",
    [string]$Segment = "Other",
    [ValidateSet("restrictive", "balanced", "permissive")]
    [string]$Permission = "balanced",
    [switch]$DeployTemplate,
    [switch]$DryRun,
    [switch]$Audit,
    [switch]$All,
    [string]$Root = ""
)

$ErrorActionPreference = "Stop"

if (-not $Root) {
    $Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}

function Read-JsonFile($Path, $Fallback) {
    if (Test-Path $Path) {
        $raw = Get-Content -Raw -Path $Path
        if ($raw.Trim()) {
            return $raw | ConvertFrom-Json
        }
    }
    return $Fallback
}

function Write-JsonFile($Path, $Value) {
    $json = $Value | ConvertTo-Json -Depth 20
    $parent = Split-Path -Parent $Path
    if ($parent -and -not (Test-Path $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $json + [Environment]::NewLine, $utf8)
}

function Get-SegmentMap($SegmentsObject) {
    $map = [ordered]@{}
    if ($SegmentsObject -and $SegmentsObject.segments) {
        foreach ($prop in $SegmentsObject.segments.PSObject.Properties) {
            $map[$prop.Name] = @($prop.Value)
        }
    }
    return $map
}

function Find-ProjectSegment($Segments, $ProjectName) {
    foreach ($segment in $Segments.Keys) {
        if (@($Segments[$segment]) -contains $ProjectName) {
            return $segment
        }
    }
    return "Unassigned"
}

function Get-TemplateVersion($ProjectDir) {
    $manifest = Join-Path $ProjectDir ".template-manifest.json"
    if (-not (Test-Path $manifest)) {
        return "none"
    }
    try {
        $data = Get-Content -Raw -Path $manifest | ConvertFrom-Json
        if ($data.template_version) {
            return [string]$data.template_version
        }
    } catch {}
    return "?"
}

function Get-ProjectPermission($Config, $ProjectName) {
    if ($Config.project_permissions) {
        $prop = $Config.project_permissions.PSObject.Properties[$ProjectName]
        if ($prop) {
            return [string]$prop.Value
        }
    }
    return "none"
}

function Get-ProjectStatus($ProjectDir, $Segments, $Config) {
    $name = Split-Path -Leaf $ProjectDir
    $segment = Find-ProjectSegment $Segments $name
    $permission = Get-ProjectPermission $Config $name
    $template = Get-TemplateVersion $ProjectDir
    $hasCurrent = Test-Path (Join-Path $ProjectDir "tasks/current.md")
    $hasDrift = Test-Path (Join-Path $ProjectDir "scripts/check-drift.sh")
    $actions = @()
    if ($segment -eq "Unassigned") { $actions += "assign segment" }
    if ($permission -eq "none") { $actions += "set permission" }
    if ($template -eq "none" -or -not $hasCurrent -or -not $hasDrift) { $actions += "deploy/sync template" }
    [pscustomobject]@{
        project = $name
        segment = $segment
        permission = $permission
        template = $template
        current_task = $hasCurrent
        check_drift = $hasDrift
        ready = ($actions.Count -eq 0)
        next = ($actions -join ", ")
    }
}

function Resolve-TemplateRoot($Root, $DocumentsDir) {
    $managed = Join-Path $DocumentsDir "agent-project-template"
    if ((Test-Path (Join-Path $managed "setup.sh")) -and (Test-Path (Join-Path $managed "scripts/sync-template.sh"))) {
        return $managed
    }
    return $Root
}

$configPath = Join-Path $Root "n8n/config.json"
$segmentsPath = Join-Path $Root "n8n/dashboard/segments.json"
$config = Read-JsonFile $configPath ([pscustomobject]@{})
$documentsDir = $config.documents_dir
if (-not $documentsDir) {
    $documentsDir = [Environment]::GetFolderPath("MyDocuments")
}
$documentsDir = (Resolve-Path $documentsDir).Path
$segmentsFile = Read-JsonFile $segmentsPath ([pscustomobject]@{ segments = [pscustomobject]@{} })
$segments = Get-SegmentMap $segmentsFile

$projectDirs = Get-ChildItem -Path $documentsDir -Directory -ErrorAction SilentlyContinue |
    Where-Object { Test-Path (Join-Path $_.FullName ".git") } |
    Sort-Object Name

$statuses = $projectDirs | ForEach-Object { Get-ProjectStatus $_.FullName $segments $config }

if ($Audit -or (-not $Project -and -not $All)) {
    $ready = @($statuses | Where-Object { $_.ready }).Count
    Write-Host "Project onboarding audit: $ready/$($statuses.Count) ready"
    $statuses | Where-Object { -not $_.ready } | Format-Table project, segment, permission, template, next -AutoSize
    Write-Host ""
    Write-Host "Connect one: powershell -ExecutionPolicy Bypass -File scripts/connect-project.ps1 -Project <name> -Segment Other -Permission balanced [-DeployTemplate]"
    Write-Host "Connect metadata for all: powershell -ExecutionPolicy Bypass -File scripts/connect-project.ps1 -All -Segment Other -Permission balanced [-DryRun]"
    exit 0
}

if ($All) {
    $planned = @()
    foreach ($status in $statuses) {
        $actions = @()
        if ($status.segment -eq "Unassigned") { $actions += "segment: Unassigned -> $Segment" }
        if ($status.permission -eq "none") { $actions += "permission: none -> $Permission" }
        if ($actions.Count -gt 0) {
            $planned += [pscustomobject]@{ project = $status.project; actions = ($actions -join "; ") }
        }
    }
    if ($DryRun) {
        Write-Host "Bulk dry-run: $($planned.Count) project(s)"
        $planned | Format-Table project, actions -AutoSize
        exit 0
    }
    foreach ($item in $planned) {
        $projectName = $item.project
        if ((Find-ProjectSegment $segments $projectName) -eq "Unassigned") {
            if (-not $segments.Contains($Segment)) {
                $segments[$Segment] = @()
            }
            if (@($segments[$Segment]) -notcontains $projectName) {
                $segments[$Segment] = @($segments[$Segment] + $projectName | Sort-Object)
            }
        }
        if (-not $config.PSObject.Properties["project_permissions"]) {
            $config | Add-Member -MemberType NoteProperty -Name project_permissions -Value ([pscustomobject]@{})
        }
        if ((Get-ProjectPermission $config $projectName) -eq "none") {
            $config.project_permissions | Add-Member -MemberType NoteProperty -Name $projectName -Value $Permission -Force
        }
    }
    $segmentsOut = [ordered]@{
        _comment = "Project segments for dashboard grouping. Edit to customize."
        segments = $segments
    }
    Write-JsonFile $segmentsPath $segmentsOut
    Write-JsonFile $configPath $config
    Write-Host "Bulk connected metadata for $($planned.Count) project(s)"
    $planned | Format-Table project, actions -AutoSize
    exit 0
}

$projectDir = Join-Path $documentsDir $Project
if (-not (Test-Path (Join-Path $projectDir ".git"))) {
    throw "Project '$Project' is not a git repo under $documentsDir"
}

$oldSegment = Find-ProjectSegment $segments $Project
$oldPermission = Get-ProjectPermission $config $Project
$planned = @()
if ($oldSegment -ne $Segment) { $planned += "segment: $oldSegment -> $Segment" }
if ($oldPermission -ne $Permission) { $planned += "permission: $oldPermission -> $Permission" }
if ($DeployTemplate) { $planned += "template: deploy/sync" }
if ($planned.Count -eq 0) { $planned += "no metadata change" }

if ($DryRun) {
    Write-Host "Dry-run for $Project"
    $planned | ForEach-Object { Write-Host "- $_" }
    exit 0
}

foreach ($key in @($segments.Keys)) {
    $segments[$key] = @($segments[$key] | Where-Object { $_ -ne $Project })
}
if (-not $segments.Contains($Segment)) {
    $segments[$Segment] = @()
}
if (@($segments[$Segment]) -notcontains $Project) {
    $segments[$Segment] = @($segments[$Segment] + $Project | Sort-Object)
}

$segmentsOut = [ordered]@{
    _comment = "Project segments for dashboard grouping. Edit to customize."
    segments = $segments
}
Write-JsonFile $segmentsPath $segmentsOut

if (-not $config.PSObject.Properties["project_permissions"]) {
    $config | Add-Member -MemberType NoteProperty -Name project_permissions -Value ([pscustomobject]@{})
}
$config.project_permissions | Add-Member -MemberType NoteProperty -Name $Project -Value $Permission -Force
Write-JsonFile $configPath $config

if ($DeployTemplate) {
    $templateRoot = Resolve-TemplateRoot $Root $documentsDir
    $sync = Join-Path $templateRoot "scripts/sync-template.sh"
    if (-not (Test-Path $sync)) {
        throw "sync-template.sh not found in $templateRoot"
    }
    $bash = (Get-Command bash -ErrorAction Stop).Source
    & $bash $sync $templateRoot "--project-dir" $projectDir
    if ($LASTEXITCODE -ne 0) {
        throw "Template sync failed for $Project"
    }
}

Write-Host "Connected $Project"
$planned | ForEach-Object { Write-Host "- $_" }
