//! Screen-space HUD: the calorie balance meter (the game's "power UI"),
//! job assignment panel, build tools, and victory overlay. Pure view —
//! returns intents and whether the pointer is over HUD chrome.
//!
//! This file is the layout: it owns the panel rects, calls each piece
//! (`panels`, `inspect`, `overlays`), and folds their output into one
//! `HudFrame`.

pub mod dock;
pub mod inspect;
pub mod objective;
pub mod overlays;
pub mod panels;
pub mod requirements;
pub mod routes;
pub mod widgets;

use crate::data::GameData;
use crate::simulation;
use crate::state::creatures::Job;
use crate::state::GameSession;
use crate::ui::{HudFrame, HudPanel, UiAction, UiMode, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::notifications::LoggedNotification;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::sprite::SpriteAtlas;
use macroquad_toolkit::ui::MIN_TARGET;

const JOB_ICON_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/sprites/job-icon-atlas.png");

/// Compact illustrated role markers used inside the text-forward HUD.
#[derive(Debug, Clone)]
pub struct HudSprites {
    jobs: SpriteAtlas,
}

/// Per-frame UI state owned by the game shell rather than the session.
#[derive(Debug, Clone, Copy, Default)]
pub struct HudOptions<'a> {
    pub help_open: bool,
    pub event_log_open: bool,
    pub event_log_page: usize,
    pub event_history: &'a [LoggedNotification],
    pub hud_panel: Option<HudPanel>,
    pub routes_open: bool,
    pub confirm_load: bool,
    pub paused: bool,
    pub save_exists: bool,
    /// Persistent shell warning when the last checkpoint write was rejected.
    pub checkpoint_warning: Option<&'a str>,
    /// Touch release in logical screen coordinates, when the gesture was a
    /// tap. It keeps world-click suppression aligned with touch UI hits.
    pub touch_position: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColonyFailure {
    Silent,
    GuardHandoff,
}

impl HudSprites {
    pub fn load() -> Self {
        let texture = Texture2D::from_file_with_format(JOB_ICON_ATLAS_BYTES, None);
        texture.set_filter(FilterMode::Linear);
        Self {
            jobs: SpriteAtlas::new(texture, 512.0, 512.0),
        }
    }

    pub fn draw_job(&self, job: Job, center: Vec2) {
        let frame = match job {
            Job::Miner => 0,
            Job::Carrier => 1,
            Job::Cook => 2,
            Job::Smith | Job::Smelter | Job::Engineer => 3,
            Job::Guard => 4,
            Job::Janitor => 2,
            Job::Courier => 1,
            Job::Idle => 5,
        };
        self.jobs
            .draw_frame(frame, center, vec2(20.0, 20.0), false, WHITE);
    }
}

pub(crate) fn draw(
    session: &GameSession,
    data: &GameData,
    ui: &VirtualUi,
    sprites: &HudSprites,
    mode: &UiMode,
    selected: Option<TilePos>,
    options: HudOptions<'_>,
) -> HudFrame {
    let mut actions = Vec::new();
    let mouse = ui.mouse_position();
    // A touch release may not update macroquad's mouse position. Use the
    // semantic touch point when deciding whether the release belongs to HUD
    // chrome, otherwise a button can fire and the same contact can fall
    // through to a world action underneath it.
    let interaction_point = interaction_point(ui, mouse, options.touch_position);

    let compact = panels::compact_top_bar(ui.scale);
    let top_bar = Rect::new(12.0, 12.0, LOGICAL_WIDTH - 24.0, 48.0);
    let routes_available = session.worm_awake && !session.outposts.is_empty();
    let tutorial_available = crate::tutorial::current_step(session, data).is_some();
    let command_strip = dock::draw_command_strip(
        session,
        &options,
        tutorial_available,
        routes_available,
        mouse,
        ui.scale,
        &mut actions,
    );

    // These drawers all share the same left landing zone. Only one is drawn
    // at a time, leaving the rest of the map available for panning and play.
    let drawer_x = 24.0;
    let drawer_top = 76.0;
    let food_panel = Rect::new(drawer_x, drawer_top, 300.0, 184.0);
    let jobs_panel_height = if compact { 464.0 } else { 400.0 };
    let jobs_panel = Rect::new(drawer_x, drawer_top, 300.0, jobs_panel_height);
    let tools_panel_height = if compact { 520.0 } else { 252.0 };
    let tools_panel_width = if compact { 360.0 } else { 330.0 };
    let tools_panel = Rect::new(drawer_x, drawer_top, tools_panel_width, tools_panel_height);

    panels::draw_top_bar(
        session,
        data,
        top_bar,
        mouse,
        ui.scale,
        panels::TopBarState {
            paused: options.paused,
            save_exists: options.save_exists,
            checkpoint_warning: options.checkpoint_warning,
        },
        &mut actions,
    );
    if options.hud_panel == Some(HudPanel::Food) {
        panels::draw_food_grid_panel(session, data, food_panel);
    }
    if options.hud_panel == Some(HudPanel::Jobs) {
        panels::draw_jobs_panel(
            session,
            data,
            sprites,
            jobs_panel,
            mouse,
            ui.scale,
            &mut actions,
        );
    }
    if options.hud_panel == Some(HudPanel::Build) {
        panels::draw_tools_panel(
            session,
            data,
            tools_panel,
            mode,
            mouse,
            ui.scale,
            &mut actions,
        );
    }
    let tutorial_panel = if options.hud_panel == Some(HudPanel::Tutorial) {
        panels::draw_tutorial_panel(session, data, mouse, ui.scale, &mut actions)
    } else {
        None
    };
    let objective_panel = (options.hud_panel == Some(HudPanel::Objective))
        .then(|| panels::draw_objective_panel(session, data));
    let inspect_top = inspect_panel_top(tutorial_panel, compact);
    let outpost_open = selected.is_some_and(|pos| {
        session
            .building_at(pos)
            .is_some_and(|building| building.kind == "outpost")
    });
    let inspect_panel = selected.and_then(|pos| {
        inspect::draw_inspect_panel(
            session,
            data,
            pos,
            inspect_top,
            mouse,
            ui.scale,
            &mut actions,
        )
    });

    // A status-icon legend, shown only while some node is stalled — it
    // teaches the in-world badges exactly when they matter.
    if !outpost_open
        && session
            .buildings
            .iter()
            .any(|b| crate::ui::legibility::building_status(session, data, b).is_some())
    {
        overlays::draw_status_legend(session, data);
    }

    let victory_up = session.won && !session.victory_shown;
    let factory_up = session.factory_complete && !session.factory_shown;
    let worm_up = session.worm_awake && !session.worm_shown;
    let routes_up = options.routes_open && session.worm_awake && !session.outposts.is_empty();
    let colony_failure = colony_failure_reason(session, data);
    let modal_overlay =
        colony_failure.is_some() || options.confirm_load || worm_up || factory_up || victory_up;
    if modal_overlay {
        // Goal and recovery overlays must own the frame's input. Without
        // clearing the HUD intents collected above, a click on a visible
        // overlay choice could also save, pause, or navigate underneath it.
        actions.clear();
    }
    if let Some(failure) = colony_failure {
        overlays::draw_colony_failure_overlay(
            failure,
            options.save_exists,
            ui.scale,
            mouse,
            &mut actions,
        );
    } else if options.confirm_load {
        overlays::draw_load_confirmation(data, ui.scale, mouse, &mut actions);
    } else if worm_up {
        overlays::draw_goal_overlay(
            "The Colossal Worm Awakens",
            &worm_completion_body(session),
            UiAction::DismissWorm,
            "Continue in Endless",
            ui.scale,
            mouse,
            &mut actions,
        );
    } else if factory_up {
        overlays::draw_goal_overlay(
            "Factory Complete",
            &factory_completion_body(session),
            UiAction::DismissFactory,
            "Continue to Worm",
            ui.scale,
            mouse,
            &mut actions,
        );
    } else if victory_up {
        overlays::draw_goal_overlay(
            "Warren Secured",
            &warren_victory_body(session, data),
            UiAction::DismissVictory,
            warren_victory_continue_label(session),
            ui.scale,
            mouse,
            &mut actions,
        );
    }

    if routes_up {
        // The route ledger is a modal shortcut into the existing inspection
        // card. Clearing the underlying HUD intents prevents a tap on a route
        // row from also changing a job or tool beneath the overlay.
        actions.clear();
        routes::draw_route_overview(session, data, mouse, ui.scale, &mut actions);
    }

    if options.help_open {
        // The field guide is modal: discard any button intents collected from
        // the HUD underneath and let its Close button be the only action.
        actions.clear();
        if options.event_log_open {
            overlays::draw_event_log_overlay(
                options.event_history,
                options.event_log_page,
                ui.scale,
                mouse,
                &mut actions,
            );
        } else {
            overlays::draw_help_overlay(session, data, ui.scale, mouse, &mut actions);
        }
    }

    let pointer_over_ui = modal_owns_world_input(
        victory_up,
        factory_up,
        worm_up,
        routes_up,
        colony_failure.is_some(),
        options.help_open,
        options.confirm_load,
    ) || tutorial_panel
        .is_some_and(|r| panel_input_rect(r, ui.scale).contains_point(interaction_point))
        || objective_panel
            .is_some_and(|r| panel_input_rect(r, ui.scale).contains_point(interaction_point))
        || inspect_panel
            .is_some_and(|r| panel_input_rect(r, ui.scale).contains_point(interaction_point))
        || [
            top_bar_input_rect(top_bar, ui.scale),
            panel_input_rect(command_strip, ui.scale),
        ]
        .iter()
        .any(|r| r.contains_point(interaction_point))
        || [
            (options.hud_panel == Some(HudPanel::Food)).then_some(food_panel),
            (options.hud_panel == Some(HudPanel::Jobs)).then_some(jobs_panel),
            (options.hud_panel == Some(HudPanel::Build)).then_some(tools_panel),
        ]
        .into_iter()
        .flatten()
        .any(|r| panel_input_rect(r, ui.scale).contains_point(interaction_point));

    HudFrame {
        actions,
        pointer_over_ui,
    }
}

pub fn modal_owns_world_input(
    victory_up: bool,
    factory_up: bool,
    worm_up: bool,
    routes_up: bool,
    colony_failure: bool,
    help_open: bool,
    confirm_load: bool,
) -> bool {
    victory_up || factory_up || worm_up || routes_up || colony_failure || help_open || confirm_load
}

pub fn colony_failure_reason(session: &GameSession, data: &GameData) -> Option<ColonyFailure> {
    if session.worm_awake {
        return None;
    }
    if session.local_creature_count() == 0 {
        return Some(ColonyFailure::Silent);
    }
    session
        .is_non_viable(data)
        .then_some(ColonyFailure::GuardHandoff)
}

pub fn worm_completion_body(session: &GameSession) -> String {
    format!(
        "Fed on {:.0} food and {} ingots, the great worm rises from the deep and coils around the warren that raised it.\n\nThe campaign is complete in {:.0} minutes. Tap Continue in Endless to keep the warren growing, or tap Return to Menu.",
        session.worm_fed,
        session.worm_ingots_fed,
        simulation::sim_seconds(session) / 60.0
    )
}

pub fn warren_victory_body(session: &GameSession, data: &GameData) -> String {
    let base = format!(
        "The warren thrives: the {:.0}-food surplus and {} ore delivered in {:.0} minutes secure the colony.",
        data.balance.win_food_surplus,
        session.economy.ore_delivered_total,
        simulation::sim_seconds(session) / 60.0
    );
    if session.job_count(Job::Guard) > 0 {
        format!(
            "{base}\n\nOnboarding is complete. Tap Continue to Factory, then tap Blacksmith in Build & Dig and tap open floor to forge {} ingots.",
            data.balance.win2_ingots
        )
    } else {
        format!(
            "{base}\n\nThe reserve gate is complete. Onboarding still needs a Guard. After closing this report, {}.",
            objective::security_handoff_action_hint(session, data)
        )
    }
}

pub fn warren_victory_continue_label(session: &GameSession) -> &'static str {
    if session.job_count(Job::Guard) > 0 {
        "Continue to Factory"
    } else {
        "Return to Warren"
    }
}

pub fn factory_completion_body(session: &GameSession) -> String {
    format!(
        "The Biofoundry roars: {} ingots forged by hammer and living furnace in {:.0} minutes.\n\nEvery belt breathes. Tap Continue to Worm, then tap Shrine in Build & Dig and tap open floor, or tap Return to Menu.",
        session.economy.ingots_forged,
        simulation::sim_seconds(session) / 60.0
    )
}

/// Claim the small invisible margins around top-bar buttons for the HUD too.
/// Otherwise a release on a scaled touch target just outside the drawn bar can
/// activate the button and also fall through to a world tile click.
pub fn top_bar_input_rect(bar: Rect, ui_scale: f32) -> Rect {
    let scale = if ui_scale.is_finite() && ui_scale > 0.0 {
        ui_scale
    } else {
        1.0
    };
    let margin = ((MIN_TARGET / scale - 32.0) * 0.5).max(0.0);
    Rect::new(
        bar.x - margin,
        bar.y - margin,
        bar.w + margin * 2.0,
        bar.h + margin * 2.0,
    )
}

/// Claim the invisible margins around the smallest panel buttons as well.
/// Tutorial, inspection, and tool controls can otherwise trigger their action
/// while a release just outside the drawn card still falls through to a map
/// click.
pub fn panel_input_rect(panel: Rect, ui_scale: f32) -> Rect {
    let scale = if ui_scale.is_finite() && ui_scale > 0.0 {
        ui_scale
    } else {
        1.0
    };
    let margin = ((MIN_TARGET / scale - 22.0) * 0.5).max(0.0);
    Rect::new(
        panel.x - margin,
        panel.y - margin,
        panel.w + margin * 2.0,
        panel.h + margin * 2.0,
    )
}

pub fn interaction_point(ui: &VirtualUi, mouse: Vec2, touch_position: Option<Vec2>) -> Vec2 {
    touch_position
        .and_then(|position| ui.screen_to_ui_checked(position))
        .unwrap_or(mouse)
}

pub fn inspect_panel_top(tutorial_panel: Option<Rect>, _compact: bool) -> f32 {
    tutorial_panel.map_or(76.0, |panel| panel.y + panel.h + 10.0)
}
