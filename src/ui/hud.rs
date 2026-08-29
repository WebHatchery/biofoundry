//! Screen-space HUD: the calorie balance meter (the game's "power UI"),
//! job assignment panel, build tools, and victory overlay. Pure view —
//! returns intents and whether the pointer is over HUD chrome.
//!
//! This file is the layout: it owns the panel rects, calls each piece
//! (`panels`, `inspect`, `overlays`), and folds their output into one
//! `HudFrame`.

mod inspect;
mod objective;
mod overlays;
mod panels;
mod requirements;
mod widgets;

use crate::data::GameData;
use crate::simulation;
use crate::state::creatures::Job;
use crate::state::GameSession;
use crate::ui::{HudFrame, UiAction, UiMode, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::sprite::SpriteAtlas;

const JOB_ICON_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/sprites/job-icon-atlas.png");

const PANEL_W: f32 = 252.0;

/// Compact illustrated role markers used inside the text-forward HUD.
#[derive(Debug, Clone)]
pub struct HudSprites {
    jobs: SpriteAtlas,
}

/// Per-frame UI state owned by the game shell rather than the session.
#[derive(Debug, Clone, Copy, Default)]
pub struct HudOptions {
    pub help_open: bool,
    pub save_exists: bool,
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

pub fn draw(
    session: &GameSession,
    data: &GameData,
    ui: &VirtualUi,
    sprites: &HudSprites,
    mode: &UiMode,
    selected: Option<TilePos>,
    options: HudOptions,
) -> HudFrame {
    let mut actions = Vec::new();
    let mouse = ui.mouse_position();

    let top_bar = Rect::new(12.0, 12.0, LOGICAL_WIDTH - 24.0, 48.0);
    let food_panel = Rect::new(12.0, 66.0, PANEL_W, 184.0);
    let jobs_panel = Rect::new(12.0, 256.0, PANEL_W, 400.0);
    // Keep Build & Dig beside the other opening controls. The WebGL page can
    // show a 1200x675 canvas below a header; putting this panel at the bottom
    // makes its buttons disappear below the browser fold at 1280x720.
    let tools_panel = Rect::new(PANEL_W + 28.0, 66.0, PANEL_W, 252.0);

    panels::draw_top_bar(session, top_bar, mouse, &mut actions);
    panels::draw_food_grid_panel(session, data, food_panel);
    panels::draw_jobs_panel(session, data, sprites, jobs_panel, mouse, &mut actions);
    panels::draw_tools_panel(session, data, tools_panel, mode, mouse, &mut actions);
    let tutorial_panel = panels::draw_tutorial_panel(session, data, mouse, &mut actions);
    let objective_panel = panels::draw_objective_panel(session, data);
    let inspect_panel = selected
        .and_then(|pos| inspect::draw_inspect_panel(session, data, pos, mouse, &mut actions));

    // A status-icon legend, shown only while some node is stalled — it
    // teaches the in-world badges exactly when they matter.
    if session
        .buildings
        .iter()
        .any(|b| crate::ui::legibility::building_status(session, data, b).is_some())
    {
        overlays::draw_status_legend();
    }

    let victory_up = session.won && !session.victory_shown;
    let factory_up = session.factory_complete && !session.factory_shown;
    let worm_up = session.worm_awake && !session.worm_shown;
    let colony_lost = session.creatures.is_empty() && !session.worm_awake;
    if colony_lost {
        overlays::draw_colony_failure_overlay(options.save_exists, mouse, &mut actions);
    } else if worm_up {
        overlays::draw_goal_overlay(
            "The Colossal Worm Awakens",
            &format!(
                "Fed on {:.0} offerings, the great worm rises from the deep and coils around the warren that raised it.\n\nThe campaign is complete in {:.0} minutes. The warren — and its worm — play on.",
                session.worm_fed,
                simulation::sim_seconds(session) / 60.0
            ),
            UiAction::DismissWorm,
            mouse,
            &mut actions,
        );
    } else if factory_up {
        overlays::draw_goal_overlay(
            "Factory Complete",
            &format!(
                "The Biofoundry roars: {} ingots forged by hammer and living furnace in {:.0} minutes.\n\nEvery belt breathes. Keep playing, or return to the menu.",
                session.economy.ingots_forged,
                simulation::sim_seconds(session) / 60.0
            ),
            UiAction::DismissFactory,
            mouse,
            &mut actions,
        );
    } else if victory_up {
        overlays::draw_goal_overlay(
            "Victory",
            &format!(
                "The warren thrives: a 100-food surplus and {} ore delivered in {:.0} minutes.\n\nNext: place a Blacksmith and hammer ore into {} ingots (a Smelter Den + salamander forges them in bulk).",
                session.economy.ore_delivered_total,
                simulation::sim_seconds(session) / 60.0,
                data.balance.win2_ingots
            ),
            UiAction::DismissVictory,
            mouse,
            &mut actions,
        );
    }

    if options.help_open {
        // The field guide is modal: discard any button intents collected from
        // the HUD underneath and let its Close button be the only action.
        actions.clear();
        overlays::draw_help_overlay(mouse, &mut actions);
    }

    let pointer_over_ui = victory_up
        || factory_up
        || worm_up
        || colony_lost
        || options.help_open
        || tutorial_panel.is_some_and(|r| r.contains_point(mouse))
        || objective_panel.contains_point(mouse)
        || inspect_panel.is_some_and(|r| r.contains_point(mouse))
        || [top_bar, food_panel, jobs_panel, tools_panel]
            .iter()
            .any(|r| r.contains_point(mouse));

    HudFrame {
        actions,
        pointer_over_ui,
    }
}
