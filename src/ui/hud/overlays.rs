//! Full-screen goal overlays, the revisitable field guide, and the in-world
//! status-badge legend.

use super::ColonyFailure;
use crate::data::GameData;
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::{UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::notifications::LoggedNotification;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const INSPECT_HELP_BODY: &str =
    "Tap a building on the map to read its status and controls. Tap a Blacksmith recipe to queue equipment; after onboarding, Breeding Pit buttons show specialist costs and benefits. Tap a Study Pen to turn captured specimens into observation.";
const OBJECTIVE_HELP_BODY: &str =
    "Read the Objective card for the current milestone. Locked gates name the exact unlock. Use the visible Jobs and Build & Dig controls it names.";
const ENDLESS_INSPECT_HELP_BODY: &str =
    "At an awakened Outpost, tap Load order and Crew per run, then Load. Pause scouting to protect food; return cargo while keeping remote crew. Tap Routes to plan multiple runs.";
const ENDLESS_OBJECTIVE_HELP_BODY: &str =
    "After the worm wakes, the Objective points to Routes, cargo, crew, upgrades, contracts, and shared-Worm dispatch order. Tap Routes to inspect the network.";
const FIELD_GUIDE_INTRO: &str =
    "Everything below has a visible touch or pointer control. Review Recent events when a toast has faded; use Older or Newer to browse further.";
pub(super) const LOAD_CONFIRMATION_TEXT: &str =
    "Load the last saved Warren? Any work since that checkpoint will be discarded.\n\nChoose Load Last Save to restore it, or Keep Current to continue this run.";
const EVENTS_PER_PAGE: usize = 10;

pub(super) fn draw_goal_overlay(
    title: &str,
    body: &str,
    dismiss: UiAction,
    continue_label: &str,
    ui_scale: f32,
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
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
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

    let compact = super::panels::compact_top_bar(ui_scale);
    let button_height = if compact { 72.0 } else { 38.0 };
    let button_y = panel.bottom() - if compact { 78.0 } else { 56.0 };
    if hud_button(
        Rect::new(panel.x + 40.0, button_y, 195.0, button_height),
        continue_label,
        true,
        mouse,
    ) {
        actions.push(dismiss);
    }
    if hud_button(
        Rect::new(panel.x + 245.0, button_y, 195.0, button_height),
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
    ui_scale: f32,
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
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
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

    let ((primary_action, primary_label), (secondary_action, secondary_label)) =
        colony_failure_actions(save_exists);
    let compact = super::panels::compact_top_bar(ui_scale);
    let button_height = if compact { 72.0 } else { 38.0 };
    let button_y = panel.bottom() - if compact { 82.0 } else { 56.0 };
    if hud_button(
        Rect::new(panel.x + 40.0, button_y, 195.0, button_height),
        primary_label,
        true,
        mouse,
    ) {
        actions.push(primary_action);
    }
    if hud_button(
        Rect::new(panel.x + 245.0, button_y, 195.0, button_height),
        secondary_label,
        true,
        mouse,
    ) {
        actions.push(secondary_action);
    }
}

fn colony_failure_actions(
    save_exists: bool,
) -> ((UiAction, &'static str), (UiAction, &'static str)) {
    if save_exists {
        (
            (UiAction::Load, "Load Last Safe"),
            (UiAction::StartWarren, "Start New Warren"),
        )
    } else {
        (
            (UiAction::StartWarren, "Start New Warren"),
            (UiAction::BackToMenu, "Return to Menu"),
        )
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
            "No creatures remain, so this warren cannot produce food or advance the campaign.\n\nLoad the last safe warren to recover your progress, or tap Start New Warren to replace this checkpoint."
        }
        (ColonyFailure::Silent, false) => {
            "No creatures remain, so this warren cannot produce food or advance the campaign.\n\nStart a new warren to begin again."
        }
        (ColonyFailure::GuardHandoff, true) => {
            "No reassignable workers remain, so this warren cannot staff the Guard post or advance onboarding.\n\nLoad the last safe warren to recover your progress, or tap Start New Warren to replace this checkpoint."
        }
        (ColonyFailure::GuardHandoff, false) => {
            "No reassignable workers remain, so this warren cannot staff the Guard post or advance onboarding.\n\nStart a new warren to begin again."
        }
    }
}

/// Protect an active run from an accidental top-bar Load click. Recovery
/// overlays still use the direct Load action because they already explain why
/// the current Warren cannot continue.
pub(super) fn draw_load_confirmation(ui_scale: f32, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.62),
    );
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let compact = super::panels::compact_top_bar(ui_scale);
    let panel = if compact {
        Rect::new(LOGICAL_WIDTH * 0.5 - 260.0, 220.0, 520.0, 250.0)
    } else {
        Rect::new(LOGICAL_WIDTH * 0.5 - 260.0, 250.0, 520.0, 210.0)
    };
    draw_surface_with_title(
        panel,
        Some("Load the Last Save?"),
        &panel_style(),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );
    draw_text_block(
        LOAD_CONFIRMATION_TEXT,
        panel.x + 20.0,
        panel.y + 58.0,
        panel.w - 40.0,
        92.0,
        16.0,
        4.0,
        dark::TEXT,
    );
    let button_height = if compact { 72.0 } else { 36.0 };
    let button_y = panel.bottom() - if compact { 82.0 } else { 52.0 };
    if hud_button(
        Rect::new(panel.x + 24.0, button_y, 220.0, button_height),
        "Load Last Save",
        true,
        mouse,
    ) {
        actions.push(UiAction::Load);
    }
    if hud_button(
        Rect::new(panel.x + 276.0, button_y, 220.0, button_height),
        "Keep Current",
        true,
        mouse,
    ) {
        actions.push(UiAction::CancelLoad);
    }
}

/// A touch-readable guide to the controls and the short decision loop. It is
/// deliberately independent of tutorial progress so it remains useful after
/// the opening lesson is skipped or completed.
pub(super) fn draw_help_overlay(
    session: &GameSession,
    data: &GameData,
    ui_scale: f32,
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
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let panel = Rect::new(110.0, 82.0, 1060.0, 556.0);
    draw_surface_with_title(
        panel,
        Some("Warren Field Guide"),
        &panel_style(),
        TextStyle::new(21.0, dark::TEXT_BRIGHT),
    );
    draw_ui_text_ex(
        FIELD_GUIDE_INTRO,
        panel.x + 26.0,
        panel.y + 60.0,
        TextStyle::new(15.0, dark::TEXT_DIM).params(),
    );

    let left = panel.x + 28.0;
    let right = panel.x + 550.0;
    let recovery_body = recovery_guide_body(session, data);
    let (inspect_title, inspect_help_body) = field_guide_inspect_content(session.worm_awake);
    let objective_help_body = field_guide_objective_content(session.worm_awake);
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
            inspect_title,
            inspect_help_body,
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
            objective_help_body,
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

    let compact = super::panels::compact_top_bar(ui_scale);
    let button_height = if compact { 72.0 } else { 32.0 };
    let footer_y = panel.bottom() - if compact { 82.0 } else { 48.0 };
    if hud_button(
        Rect::new(panel.x + 28.0, footer_y, 160.0, button_height),
        "Recent events",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleEventLog);
    }
    if hud_button(
        Rect::new(
            panel.x + panel.w * 0.5 - 70.0,
            footer_y,
            140.0,
            button_height,
        ),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
}

/// A bounded, newest-first record of the messages that have guided the run.
/// Toasts are deliberately transient, but recovery instructions and unlock
/// notices should remain available while the player decides what to do next.
pub(super) fn draw_event_log_overlay(
    history: &[LoggedNotification],
    page: usize,
    ui_scale: f32,
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
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let panel = Rect::new(260.0, 118.0, 760.0, 484.0);
    let page_count = event_log_page_count(history.len());
    let page = page.min(page_count.saturating_sub(1));
    let title = format!("Recent Events · {}/{}", page + 1, page_count);
    draw_surface_with_title(
        panel,
        Some(&title),
        &panel_style(),
        TextStyle::new(21.0, dark::TEXT_BRIGHT),
    );
    draw_ui_text_ex(
        "Newest first — use Older or Newer to browse the saved history.",
        panel.x + 26.0,
        panel.y + 60.0,
        TextStyle::new(15.0, dark::TEXT_DIM).params(),
    );

    let list_x = panel.x + 28.0;
    let list_y = panel.y + 96.0;
    let row_height = 31.0;
    let (page_start, page_end) = event_log_page_bounds(history.len(), page);
    if page_start == page_end {
        draw_ui_text_ex(
            "No events yet. Actions and warnings will appear here.",
            list_x,
            list_y + 18.0,
            TextStyle::new(16.0, dark::TEXT).params(),
        );
    } else {
        for (row, event) in history[page_start..page_end].iter().rev().enumerate() {
            let y = list_y + row as f32 * row_height;
            let color = event.notification_type.color();
            draw_circle(list_x + 7.0, y + 8.0, 5.0, color);
            let message =
                macroquad_toolkit::ui::truncate_text_to_width(&event.message, panel.w - 76.0, 15.0);
            draw_ui_text_ex(
                &message,
                list_x + 22.0,
                y + 14.0,
                TextStyle::new(15.0, dark::TEXT).params(),
            );
        }
    }

    let compact = super::panels::compact_top_bar(ui_scale);
    let button_height = if compact { 72.0 } else { 32.0 };
    let footer_y = panel.bottom() - if compact { 82.0 } else { 48.0 };
    if hud_button(
        Rect::new(panel.x + 24.0, footer_y, 140.0, button_height),
        "Field Guide",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleEventLog);
    }
    if hud_button(
        Rect::new(panel.x + 196.0, footer_y, 104.0, button_height),
        "Older",
        page + 1 < page_count,
        mouse,
    ) {
        actions.push(UiAction::EventLogOlder);
    }
    if hud_button(
        Rect::new(panel.x + 308.0, footer_y, 104.0, button_height),
        "Newer",
        page > 0,
        mouse,
    ) {
        actions.push(UiAction::EventLogNewer);
    }
    if hud_button(
        Rect::new(panel.right() - 164.0, footer_y, 140.0, button_height),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
}

fn event_log_page_count(history_len: usize) -> usize {
    history_len.div_ceil(EVENTS_PER_PAGE).max(1)
}

fn field_guide_inspect_content(worm_awake: bool) -> (&'static str, &'static str) {
    if worm_awake {
        ("Endless routes", ENDLESS_INSPECT_HELP_BODY)
    } else {
        ("Inspect & craft", INSPECT_HELP_BODY)
    }
}

fn field_guide_objective_content(worm_awake: bool) -> &'static str {
    if worm_awake {
        ENDLESS_OBJECTIVE_HELP_BODY
    } else {
        OBJECTIVE_HELP_BODY
    }
}

fn event_log_page_bounds(history_len: usize, page: usize) -> (usize, usize) {
    let page_end = history_len.saturating_sub(page.saturating_mul(EVENTS_PER_PAGE));
    (page_end.saturating_sub(EVENTS_PER_PAGE), page_end)
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
        (St::NoValidRoute, Color::new(0.70, 0.62, 0.85, 1.0)),
        (St::InputStarved, Color::new(0.95, 0.55, 0.20, 1.0)),
        (St::OutputFull, Color::new(0.92, 0.32, 0.26, 1.0)),
        (St::AwaitingHaul, Color::new(0.40, 0.80, 0.92, 1.0)),
        (St::Exhausted, Color::new(0.60, 0.60, 0.66, 1.0)),
        (St::RouteInactive, Color::new(0.70, 0.62, 0.85, 1.0)),
        (St::ExpeditionNoCrew, Color::new(0.95, 0.85, 0.30, 1.0)),
        (St::ExpeditionPaused, Color::new(0.95, 0.72, 0.35, 1.0)),
        (St::ExpeditionNeedsFood, Color::new(0.95, 0.55, 0.20, 1.0)),
        (St::ExpeditionHoldFull, Color::new(0.92, 0.32, 0.26, 1.0)),
        (St::ShrineOfferingsPaused, Color::new(0.95, 0.72, 0.35, 1.0)),
        (St::ShrineNeedsFood, Color::new(0.95, 0.55, 0.20, 1.0)),
        (St::ShrineNeedsIngots, Color::new(0.95, 0.85, 0.30, 1.0)),
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
