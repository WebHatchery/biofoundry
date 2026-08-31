<#
.SYNOPSIS
    Headless screenshot harness for Biofoundry.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds
    the debug exe and drives it through the env-var capture hook
    (BIOFOUNDRY_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes: menu, settings, new_warren_confirm, load_confirm, load_confirm_resolved, warren, tutorial_food, tutorial_factory, tutorial_worm, help, event_log, event_log_older, pause, save_failure, save_failure_first_save, menu_save_guard, save_recovery_failure, collapse, mine, blacksmith, smelter, cook_pot, kiln, waste, blacksmith_queue_full, equipment, overseer, factory, victory, security_stuck, factory_complete, optional, endless, endless_load_preview, endless_routes, endless_auto_return, endless_auto_resupply, endless_upgrade, endless_upgraded, endless_expedition_paused, endless_expedition_report, endless_forge, endless_empty, endless_failure, route_failure, endless_in_flight, endless_arrived, famine, food_warning, raid_food_warning, raid, raid_warning, breeding, breeding_locked, shrine, worm, unreachable_workstation, completion. Prefix any scene with touch_audit_ (for example touch_audit_endless_routes) to print settled hit-target size and overlap diagnostics.
    The focused Outpost expansion scenes are endless_crew_upgrade,
    endless_crew_upgraded, endless_survey_upgrade, endless_survey_upgraded,
    endless_resonator_upgrade, endless_resonator_upgraded, endless_charter,
    endless_charter_awarded, and endless_wormbone_drill.

    The explicit `crowding` scene demonstrates the Jobs-panel recovery guidance.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes warren -Frames 60 -SkipBuild
    ./scripts/capture_ui.ps1 -Scenes completion,endless -WindowWidth 800 -WindowHeight 450 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "new_warren_confirm", "load_confirm", "load_confirm_resolved", "warren", "tutorial_food", "tutorial_factory", "tutorial_worm", "help", "event_log", "event_log_older", "pause", "save_failure", "save_failure_first_save", "menu_save_guard", "save_recovery_failure", "collapse", "mine", "blacksmith", "smelter", "cook_pot", "kiln", "waste", "blacksmith_queue_full", "equipment", "overseer", "factory", "victory", "security_stuck", "factory_complete", "optional", "endless", "endless_load_preview", "endless_routes", "endless_auto_return", "endless_auto_resupply", "endless_upgrade", "endless_upgraded", "endless_expedition_paused", "endless_expedition_report", "endless_forge", "endless_empty", "endless_failure", "endless_in_flight", "endless_arrived", "famine", "food_warning", "raid_food_warning", "raid", "raid_warning", "breeding", "breeding_locked", "shrine", "shrine_waiting", "worm", "unreachable_workstation", "completion"),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild,
    [switch]$Release,
    [switch]$Visible
)

$ErrorActionPreference = "Stop"
if (-not $PSBoundParameters.ContainsKey("Scenes")) {
    $Scenes += "crowding"
    $Scenes += @(
        "endless_crew_upgrade",
        "endless_crew_upgraded",
        "endless_survey_upgrade",
        "endless_survey_upgraded",
        "endless_resonator_upgrade",
        "endless_resonator_upgraded",
        "endless_charter",
        "endless_charter_awarded",
        "endless_wormbone_drill"
    )
}
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "BIOFOUNDRY" -Scenes $Scenes -Frames $Frames -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir -SkipBuild:$SkipBuild -Release:$Release -Visible:$Visible
