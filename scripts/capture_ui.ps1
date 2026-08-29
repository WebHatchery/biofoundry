<#
.SYNOPSIS
    Headless screenshot harness for Biofoundry.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds
    the debug exe and drives it through the env-var capture hook
    (BIOFOUNDRY_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes: menu, new_warren_confirm, warren, help, pause, collapse, mine, blacksmith, equipment, overseer, factory, tutorial_factory, victory, factory_complete, optional, endless, endless_failure, endless_in_flight, famine, food_warning, raid_food_warning, raid, raid_warning, breeding, shrine, worm, completion.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes warren -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "new_warren_confirm", "warren", "help", "pause", "collapse", "mine", "blacksmith", "equipment", "overseer", "factory", "tutorial_factory", "victory", "factory_complete", "optional", "endless", "endless_failure", "endless_in_flight", "famine", "food_warning", "raid_food_warning", "raid", "raid_warning", "breeding", "shrine", "worm", "completion"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "BIOFOUNDRY" -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
