<#
.SYNOPSIS
    Audit Biofoundry's settled touch targets at a chosen viewport.

.DESCRIPTION
    Captures each requested screen for two settled frames, then prints the
    shared toolkit's touch-target report. The game exits normally after each
    capture; its opt-in report file is collected here because the shared
    capture wrapper removes child-process logs after a successful run.

    Scene names are the normal capture scene names. The script adds the
    touch_audit_ prefix automatically.

.EXAMPLE
    ./scripts/audit_touch_targets.ps1
    ./scripts/audit_touch_targets.ps1 -Scenes endless_routes,completion -WindowWidth 800 -WindowHeight 450
#>
param(
    [string[]]$Scenes = @(
        "menu",
        "settings",
        "warren",
        "new_warren_confirm",
        "help",
        "event_log",
        "blacksmith",
        "breeding",
        "shrine",
        "completion",
        "endless_load_preview",
        "endless_routes"
    ),
    [int]$WindowWidth = 1280,
    [int]$WindowHeight = 720,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$relativeOutputDir = Join-Path "target" ("biofoundry-touch-audit-{0}" -f $PID)
$auditDir = Join-Path $gameDir $relativeOutputDir
New-Item -ItemType Directory -Force -Path $auditDir | Out-Null

try {
    $first = $true
    foreach ($scene in $Scenes) {
        $safe = [regex]::Replace($scene, '[^A-Za-z0-9._+-]', '_')
        $reportPath = Join-Path $auditDir ("{0}.txt" -f $safe)
        $env:BIOFOUNDRY_TOUCH_AUDIT_REPORT = $reportPath
        $auditScene = "touch_audit_{0}" -f $scene

        $captureArgs = @{
            Scenes = $auditScene
            Frames = 3
            WindowWidth = $WindowWidth
            WindowHeight = $WindowHeight
            OutputDir = $relativeOutputDir
            SkipBuild = !$first
        }
        if ($Release) {
            $captureArgs.Release = $true
        }
        & (Join-Path $PSScriptRoot "capture_ui.ps1") @captureArgs
        if (-not (Test-Path -LiteralPath $reportPath)) {
            throw "No touch audit report was written for '$scene'."
        }

        Write-Output ("--- touch audit: {0} @ {1}x{2} ---" -f $scene, $WindowWidth, $WindowHeight)
        Get-Content -LiteralPath $reportPath
        if (Select-String -LiteralPath $reportPath -Pattern "overlap by" -Quiet) {
            throw "Touch-target ambiguity detected in '$scene'."
        }
        $first = $false
    }
}
finally {
    Remove-Item Env:BIOFOUNDRY_TOUCH_AUDIT_REPORT -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $auditDir) {
        Remove-Item -LiteralPath $auditDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
