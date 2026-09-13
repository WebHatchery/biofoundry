//! The five-beat onboarding card and its state-aware recovery instructions.

use super::compact_top_bar;
use crate::data::{GameData, TutorialStepDef};
use crate::state::creatures::Job;
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::{HudPanel, UiAction, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

const TUTORIAL_PANEL_HEIGHT: f32 = 168.0;
const TUTORIAL_BODY_HEIGHT: f32 = 94.0;

/// The tutorial card, top-right: current step, progress chip, and a skip
/// button. Returns its rect while visible (for pointer-over-UI checks).
pub(crate) fn draw_tutorial_panel(
    session: &GameSession,
    data: &GameData,
    mouse: Vec2,
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) -> Option<Rect> {
    let step = crate::tutorial::current_step(session, data)?;
    let (done, total) = crate::tutorial::progress(session, data);

    let compact = compact_top_bar(ui_scale);
    let panel_height = if compact {
        230.0
    } else {
        TUTORIAL_PANEL_HEIGHT
    };
    let panel = Rect::new(LOGICAL_WIDTH - 342.0, 72.0, 330.0, panel_height);
    draw_surface_with_title(
        panel,
        Some(&format!("Tutorial {}/{} — {}", done + 1, total, step.title)),
        &panel_style(),
        TextStyle::new(15.0, dark::TEXT_BRIGHT),
    );

    let body = tutorial_body(step, session, data);
    draw_text_block(
        &body,
        panel.x + 14.0,
        panel.y + 42.0,
        panel.w - 28.0,
        TUTORIAL_BODY_HEIGHT,
        14.0,
        4.0,
        dark::TEXT,
    );

    let skip_width = if compact { 78.0 } else { 64.0 };
    let skip_height = if compact { 72.0 } else { 22.0 };
    if hud_button(
        Rect::new(
            panel.x + 10.0,
            panel.bottom() - skip_height - 8.0,
            skip_width,
            skip_height,
        ),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHudPanel(HudPanel::Tutorial));
    }
    if hud_button(
        Rect::new(
            panel.right() - skip_width - 10.0,
            panel.bottom() - skip_height - 8.0,
            skip_width,
            skip_height,
        ),
        "Skip",
        true,
        mouse,
    ) {
        actions.push(UiAction::SkipTutorial);
    }

    Some(panel)
}

pub fn tutorial_body(step: &TutorialStepDef, session: &GameSession, data: &GameData) -> String {
    match step.id.as_str() {
        "food" => format!(
            "Read Food Grid: keep Production above Upkeep. Tap Farm, then tap open floor. Wait for Farm construction. If food pressure rises, {}.",
            super::super::objective::job_assignment_action_hint(
                session,
                data,
                Job::Carrier,
                &[Job::Miner, Job::Smith, Job::Guard],
            )
        ),
        "factory" => format!(
            "Tap the existing Mine to read its rate. Tap Blacksmith in Build & Dig, then tap open floor. To staff it, {}. Tap the placed Blacksmith, then tap Iron Pickaxe; the miner equips it and the Mine speeds up.",
            super::super::objective::job_assignment_action_hint(
                session,
                data,
                Job::Smith,
                &[Job::Miner, Job::Carrier, Job::Cook, Job::Guard],
            )
        ),
        "secure" => format!(
            "Deliver 50 ore and hold 100 food in Objective. Before a raid, {}. This secures the warren and ends onboarding; next, forge 20 ingots and raise the Shrine.",
            super::super::objective::security_handoff_action_hint(session, data)
        ),
        _ => step.body.clone(),
    }
}
