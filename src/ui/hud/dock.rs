//! The small command strip that keeps the world visible between decisions.

use super::widgets::hud_button;
use crate::state::GameSession;
use crate::ui::{HudPanel, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const DOCK_MARGIN: f32 = 16.0;

/// Draw the always-available management controls along the bottom edge. The
/// strip is intentionally short: the map remains the primary composition and
/// the ledgers appear only after one of these controls is tapped.
pub(super) fn draw_command_strip(
    session: &GameSession,
    options: &super::HudOptions<'_>,
    tutorial_available: bool,
    routes_available: bool,
    mouse: Vec2,
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) -> Rect {
    let compact = ui_scale.is_finite() && ui_scale < 0.9;
    let dock_height = if compact { 84.0 } else { 62.0 };
    let dock_width = 24.0
        + 216.0
        + 8.0
        + (88.0 + 88.0 + 120.0 + 88.0 + 4.0 * 8.0)
        + 8.0
        + 96.0
        + if routes_available { 8.0 + 100.0 } else { 0.0 };
    let dock = Rect::new(
        ((LOGICAL_WIDTH - dock_width) * 0.5).max(DOCK_MARGIN),
        LOGICAL_HEIGHT - dock_height - 10.0,
        dock_width.min(LOGICAL_WIDTH - DOCK_MARGIN * 2.0),
        dock_height,
    );
    draw_surface(
        dock,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.085, 0.96))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.7)),
    );

    draw_resource_summary(
        session,
        Rect::new(dock.x + 12.0, dock.y + 11.0, 216.0, 38.0),
    );

    let button_height = if compact { 72.0 } else { 38.0 };
    let button_y = dock.y + if compact { 6.0 } else { 12.0 };
    let mut x = dock.x + 238.0;
    for (panel, label, width) in [
        (HudPanel::Food, "Food", 88.0),
        (HudPanel::Jobs, "Jobs", 88.0),
        (HudPanel::Build, "Build & Dig", 120.0),
        (HudPanel::Objective, "Goal", 88.0),
    ] {
        let rect = Rect::new(x, button_y, width, button_height);
        if hud_button(rect, label, true, mouse) {
            actions.push(UiAction::ToggleHudPanel(panel));
        }
        if options.hud_panel == Some(panel) {
            draw_active_marker(rect);
        }
        x += width + 8.0;
    }

    let guide_width = 96.0;
    let guide_rect = Rect::new(x, button_y, guide_width, button_height);
    if hud_button(
        guide_rect,
        if tutorial_available {
            "Tutorial"
        } else {
            "Guide"
        },
        true,
        mouse,
    ) {
        if tutorial_available {
            actions.push(UiAction::ToggleHudPanel(HudPanel::Tutorial));
        } else {
            actions.push(UiAction::ToggleHelp);
        }
    }
    if options.hud_panel == Some(HudPanel::Tutorial) {
        draw_active_marker(guide_rect);
    }
    x += guide_width + 8.0;

    if routes_available {
        let routes_rect = Rect::new(x, button_y, 100.0, button_height);
        if hud_button(routes_rect, "Routes", true, mouse) {
            actions.push(UiAction::ToggleRoutes);
        }
        if options.routes_open {
            draw_active_marker(routes_rect);
        }
    }

    macroquad_toolkit::ui::occlude(dock);
    dock
}

fn draw_resource_summary(session: &GameSession, rect: Rect) {
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.09, 0.105, 0.13, 1.0))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.42)),
    );
    draw_ui_text_ex(
        &format!(
            "FOOD {:.0}   ORE {}   INGOTS {}",
            session.economy.food, session.economy.ore_stock, session.economy.ingots_stock
        ),
        rect.x + 12.0,
        rect.y + 24.0,
        TextStyle::new(13.0, dark::TEXT).params(),
    );
}

fn draw_active_marker(rect: Rect) {
    draw_rectangle(rect.x, rect.bottom() - 3.0, rect.w, 3.0, dark::ACCENT);
}
