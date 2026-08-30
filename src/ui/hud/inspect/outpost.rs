//! Compact layout for the awakened Outpost's route controls.

use crate::data::GameData;
use crate::state::outposts::Outpost;
use crate::state::GameSession;
use crate::ui::hud::widgets::hud_button;
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;

pub(super) struct CompactRouteContext<'a> {
    pub(super) session: &'a GameSession,
    pub(super) data: &'a GameData,
    pub(super) pos: TilePos,
    pub(super) outpost: Option<&'a Outpost>,
    pub(super) panel: Rect,
    pub(super) x: f32,
    pub(super) y: &'a mut f32,
    pub(super) crew: usize,
    pub(super) button_height: f32,
    pub(super) button_step: f32,
    pub(super) mouse: Vec2,
    pub(super) actions: &'a mut Vec<UiAction>,
}

/// Keep the route policies and quotas legible on a narrow fixed-resolution
/// canvas by pairing the two cycling control groups while leaving primary
/// return, scouting, upgrade, and load actions full width.
pub(super) fn draw_compact_route_controls(context: CompactRouteContext<'_>) {
    let CompactRouteContext {
        session,
        data,
        pos,
        outpost,
        panel,
        x,
        y,
        crew,
        button_height,
        button_step,
        mouse,
        actions,
    } = context;
    let column_width = (panel.w - 36.0) * 0.5;
    let second_column_x = x + column_width + 8.0;
    let auto_return_label = if outpost.is_some_and(|route| route.auto_return_cargo) {
        "Cargo: auto"
    } else {
        "Cargo: manual"
    };
    if hud_button(
        Rect::new(x, *y, column_width, button_height),
        auto_return_label,
        session.worm_transit.is_none(),
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoReturn(pos));
    }
    let auto_resupply_label = if outpost.is_some_and(|route| route.auto_resupply_food) {
        "Food: auto"
    } else {
        "Food: manual"
    };
    if hud_button(
        Rect::new(second_column_x, *y, column_width, button_height),
        auto_resupply_label,
        session.worm_transit.is_none(),
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoResupply(pos));
    }
    *y += button_step;

    let priority = outpost
        .map(|route| format!("Load: {}", route.cargo_priority.label()))
        .unwrap_or_else(|| "Load: Ore".to_owned());
    if hud_button(
        Rect::new(x, *y, column_width, button_height),
        &priority,
        session.worm_transit.is_none(),
        mouse,
    ) {
        actions.push(UiAction::CycleOutpostCargo(pos));
    }
    let crew_label = outpost
        .and_then(|route| {
            route
                .crew_dispatch_limit
                .map(|limit| format!("Crew: {limit}"))
        })
        .unwrap_or_else(|| "Crew: Auto".to_owned());
    if hud_button(
        Rect::new(second_column_x, *y, column_width, button_height),
        &crew_label,
        session.worm_transit.is_none(),
        mouse,
    ) {
        actions.push(UiAction::CycleOutpostCrew(pos));
    }
    *y += button_step;

    if outpost.is_some_and(|route| !route.storage_upgraded) {
        let upgrade_cost = data.balance.outpost_upgrade_ingots;
        if hud_button(
            Rect::new(x, *y, panel.w - 28.0, button_height),
            &format!("Expand hold · {upgrade_cost} ingots"),
            session.economy.ingots_stock >= upgrade_cost,
            mouse,
        ) {
            actions.push(UiAction::UpgradeOutpost(pos));
        }
        *y += button_step;
    }
    if outpost.is_some_and(|route| !route.crew_upgraded) {
        let upgrade_cost = data.balance.outpost_crew_upgrade_ingots;
        if hud_button(
            Rect::new(x, *y, panel.w - 28.0, button_height),
            &format!("Expand camp · {upgrade_cost} ingots"),
            session.economy.ingots_stock >= upgrade_cost,
            mouse,
        ) {
            actions.push(UiAction::UpgradeOutpostCrew(pos));
        }
        *y += button_step;
    }
    if crew > 0
        && hud_button(
            Rect::new(x, *y, panel.w - 28.0, button_height),
            if outpost.is_some_and(|route| route.expedition_paused) {
                "Resume scouting"
            } else {
                "Pause scouting"
            },
            session.worm_transit.is_none(),
            mouse,
        )
    {
        actions.push(UiAction::ToggleOutpostExpedition(pos));
    }
    *y += 38.0;
}
