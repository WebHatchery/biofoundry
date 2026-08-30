<#
.SYNOPSIS
    Headless screenshot harness for Biofoundry.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds
    the debug exe and drives it through the env-var capture hook
    (BIOFOUNDRY_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes: menu, new_warren_confirm, warren, tutorial_food, tutorial_factory, tutorial_worm, help, pause, collapse, mine, blacksmith, smelter, cook_pot, kiln, blacksmith_queue_full, equipment, overseer, factory, victory, security_stuck, factory_complete, optional, endless, endless_empty, endless_failure, endless_in_flight, endless_arrived, famine, food_warning, raid_food_warning, raid, raid_warning, breeding, shrine, worm, completion.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes warren -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "new_warren_confirm", "warren", "tutorial_food", "tutorial_factory", "tutorial_worm", "help", "pause", "collapse", "mine", "blacksmith", "smelter", "cook_pot", "kiln", "waste", "blacksmith_queue_full", "equipment", "overseer", "factory", "victory", "security_stuck", "factory_complete", "optional", "endless", "endless_empty", "endless_failure", "endless_in_flight", "endless_arrived", "famine", "food_warning", "raid_food_warning", "raid", "raid_warning", "breeding", "breeding_locked", "shrine", "worm", "completion"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "BIOFOUNDRY" -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
