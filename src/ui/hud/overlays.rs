//! Full-screen goal overlays, the revisitable field guide, and the in-world
//! status-badge legend.

use super::ColonyFailure;
use crate::data::GameData;
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::{UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const INSPECT_HELP_BODY: &str =
    "Tap a building on the map to see its status and controls. Tap an equipment button in the Blacksmith card to queue it; after onboarding, Breeding Pit buttons show specialist benefits. At an awakened Outpost, tap Load order for cargo priority, Crew per run for cargo-only or scout counts, then Load to send food and crew to scout for ore; pause scouting to protect provisions, or use the cargo-only return to bring ore home while keeping scouts remote.";
const OBJECTIVE_HELP_BODY: &str =
    "Read the Objective card for the current campaign milestone and its next requirement. Locked gates name their exact unlock; after the worm wakes, inspect the Outpost to prepare cargo runs.";

pub(super) fn draw_goal_overlay(
    title: &str,
    body: &str,
    dismiss: UiAction,
    continue_label: &str,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    let panel = Rect::new(LOGICAL_WIDTH * 0.5 - 240.0, 200.0, 480.0, 250.0);
    draw_surface_with_title(
        panel,
        Some(title),
        &panel_style(),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    draw_text_block(
        body,
        panel.x + 20.0,
        panel.y + 60.0,
        panel.w - 40.0,
        110.0,
        17.0,
        5.0,
        dark::TEXT,
    );

    if hud_button(
        Rect::new(panel.x + 40.0, panel.bottom() - 56.0, 195.0, 38.0),
        continue_label,
        true,
        mouse,
    ) {
        actions.push(dismiss);
    }
    if hud_button(
        Rect::new(panel.x + 245.0, panel.bottom() - 56.0, 195.0, 38.0),
        "Return to Menu",
        true,
        mouse,
    ) {
        actions.push(UiAction::BackToMenu);
    }
}

/// Recovery screen for the one unambiguous non-viable colony state: no
/// creatures remain to produce food or advance the campaign.
pub(super) fn draw_colony_failure_overlay(
    failure: ColonyFailure,
    save_exists: bool,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.58),
    );
    let panel = Rect::new(LOGICAL_WIDTH * 0.5 - 240.0, 185.0, 480.0, 280.0);
    draw_surface_with_title(
        panel,
        Some(colony_failure_title(failure)),
        &panel_style(),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    let body = colony_failure_body(failure, save_exists);
    draw_text_block(
        body,
        panel.x + 20.0,
        panel.y + 60.0,
        panel.w - 40.0,
        125.0,
        17.0,
        5.0,
        dark::TEXT,
    );

    let primary_action = if save_exists {
        UiAction::Load
    } else {
        UiAction::StartWarren
    };
    let primary_label = if save_exists {
        "Load Last Safe"
    } else {
        "Start New Warren"
    };
    if hud_button(
        Rect::new(panel.x + 40.0, panel.bottom() - 56.0, 195.0, 38.0),
        primary_label,
        true,
        mouse,
    ) {
        actions.push(primary_action);
    }
    if hud_button(
        Rect::new(panel.x + 245.0, panel.bottom() - 56.0, 195.0, 38.0),
        "Return to Menu",
        true,
        mouse,
    ) {
        actions.push(UiAction::BackToMenu);
    }
}

fn colony_failure_title(failure: ColonyFailure) -> &'static str {
    match failure {
        ColonyFailure::Silent => "The Warren Falls Silent",
        ColonyFailure::GuardHandoff => "Guard Handoff Blocked",
    }
}

fn colony_failure_body(failure: ColonyFailure, save_exists: bool) -> &'static str {
    match (failure, save_exists) {
        (ColonyFailure::Silent, true) => {
            "No creatures remain, so this warren cannot produce food or advance the campaign.\n\nLoad the last safe warren to recover your progress, or start fresh."
        }
        (ColonyFailure::Silent, false) => {
            "No creatures remain, so this warren cannot produce food or advance the campaign.\n\nStart a new warren to begin again."
        }
        (ColonyFailure::GuardHandoff, true) => {
            "No reassignable workers remain, so this warren cannot staff the Guard post or advance onboarding.\n\nLoad the last safe warren to recover your progress, or start fresh."
        }
        (ColonyFailure::GuardHandoff, false) => {
            "No reassignable workers remain, so this warren cannot staff the Guard post or advance onboarding.\n\nStart a new warren to begin again."
        }
    }
}

/// A touch-readable guide to the controls and the short decision loop. It is
/// deliberately independent of tutorial progress so it remains useful after
/// the opening lesson is skipped or completed.
pub(super) fn draw_help_overlay(
    session: &GameSession,
    data: &GameData,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.62),
    );
    let panel = Rect::new(110.0, 82.0, 1060.0, 556.0);
    draw_surface_with_title(
        panel,
        Some("Warren Field Guide"),
        &panel_style(),
        TextStyle::new(21.0, dark::TEXT_BRIGHT),
    );
    draw_ui_text_ex(
        "Everything below has a visible touch or pointer control.",
        panel.x + 26.0,
        panel.y + 60.0,
        TextStyle::new(15.0, dark::TEXT_DIM).params(),
    );

    let left = panel.x + 28.0;
    let right = panel.x + 550.0;
    let recovery_body = recovery_guide_body(session, data);
    for (x, title, body, y) in [
        (
            left,
            "Camera",
            "Drag the map to look around. Tap + or − in the top bar to zoom. Pinch also zooms on touch screens.",
            166.0,
        ),
        (
            left,
            "Build & Dig",
            "Tap a building button, then tap open floor to place it. Tap Dig, then tap rock. Tap the active button again to return to Inspect.",
            270.0,
        ),
        (
            left,
            "Jobs",
            "Tap + or − beside Miner, Carrier, Cook, Smith, or Guard to move goblins between jobs.",
            374.0,
        ),
        (
            left,
            "Inspect & craft",
            INSPECT_HELP_BODY,
            478.0,
        ),
        (
            right,
            "Food Grid",
            "Keep Production above Upkeep. The forecast says how long the cooked-food reserve lasts at the current rate.",
            166.0,
        ),
        (
            right,
            "Objective",
            OBJECTIVE_HELP_BODY,
            270.0,
        ),
        (
            right,
            "Recovery",
            recovery_body.as_str(),
            374.0,
        ),
        (
            right,
            "Save & return",
            "Tap Pause to stop the simulation while you plan. Tap Save for a manual checkpoint; Menu also autosaves a viable run before returning to the title. Load restores the last saved Warren.",
            478.0,
        ),
    ] {
        draw_ui_text_ex(title, x, y, TextStyle::new(15.0, dark::ACCENT).params());
        draw_text_block(body, x, y + 22.0, 460.0, 62.0, 14.0, 3.0, dark::TEXT);
    }

    if hud_button(
        Rect::new(
            panel.x + panel.w * 0.5 - 70.0,
            panel.bottom() - 48.0,
            140.0,
            32.0,
        ),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
}

fn recovery_guide_body(session: &GameSession, data: &GameData) -> String {
    format!(
        "When food falls, {}. Before a raid, {}. The warning bar names the response.",
        super::panels::compact_food_recovery_hint(session, data),
        super::panels::compact_raid_defense_hint(session, data)
    )
}

/// A one-line legend for the in-world status badges, in a thin strip along
/// the bottom of the world view (shown only while a node is stalled).
pub(super) fn draw_status_legend(session: &GameSession, data: &GameData) {
    use crate::ui::legibility::BuildingStatus as St;
    let strip = Rect::new(280.0, LOGICAL_HEIGHT - 30.0, LOGICAL_WIDTH - 292.0, 24.0);
    draw_surface(
        strip,
        &SurfaceStyle::new(Color::new(0.06, 0.07, 0.09, 0.88))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.4)),
    );
    let items = [
        (St::NoWorker, Color::new(0.95, 0.85, 0.30, 1.0)),
        (St::InputStarved, Color::new(0.95, 0.55, 0.20, 1.0)),
        (St::OutputFull, Color::new(0.92, 0.32, 0.26, 1.0)),
        (St::AwaitingHaul, Color::new(0.40, 0.80, 0.92, 1.0)),
        (St::Exhausted, Color::new(0.60, 0.60, 0.66, 1.0)),
        (St::RouteInactive, Color::new(0.70, 0.62, 0.85, 1.0)),
        (St::ExpeditionNoCrew, Color::new(0.95, 0.85, 0.30, 1.0)),
        (St::ExpeditionPaused, Color::new(0.95, 0.72, 0.35, 1.0)),
        (St::ExpeditionNeedsFood, Color::new(0.95, 0.55, 0.20, 1.0)),
        (St::ExpeditionHoldFull, Color::new(0.92, 0.32, 0.26, 1.0)),
        (St::WasteOverflow, Color::new(0.65, 0.85, 0.35, 1.0)),
    ];
    let mut lx = strip.x + 12.0;
    let cy = strip.y + strip.h * 0.5;
    for (status, color) in items {
        if !session.buildings.iter().any(|building| {
            crate::ui::legibility::building_status(session, data, building) == Some(status)
        }) {
            continue;
        }
        draw_circle(lx, cy, 7.5, Color::new(0.08, 0.08, 0.10, 0.92));
        crate::ui::warren::draw_status_glyph(vec2(lx, cy), 6.0, status, color);
        let label = status.label();
        draw_ui_text_ex(
            label,
            lx + 12.0,
            cy + 5.0,
            TextStyle::new(13.0, dark::TEXT).params(),
        );
        lx += 20.0 + label.len() as f32 * 8.0;
    }
}

#[cfg(test)]
mod tests;
