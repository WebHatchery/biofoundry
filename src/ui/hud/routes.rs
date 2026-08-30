//! Modal route ledger for the post-awakening Outpost network.

use super::inspect::inspect_status;
use crate::data::GameData;
use crate::state::outposts::{Outpost, TransitDirection};
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::{UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const PANEL_MARGIN_X: f32 = 70.0;
const PANEL_TOP: f32 = 48.0;
const PANEL_WIDTH: f32 = LOGICAL_WIDTH - PANEL_MARGIN_X * 2.0;
const PANEL_BOTTOM: f32 = LOGICAL_HEIGHT - 48.0;
const CARD_GAP: f32 = 12.0;
const CARD_HEIGHT: f32 = 96.0;

/// Draw the route network as a modal ledger. The rows intentionally reuse
/// the same status derivation as the building inspection card, so a player
/// can choose a route from one stable status vocabulary even when it is off
/// the current camera view.
pub(super) fn draw_route_overview(
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
        Color::new(0.0, 0.0, 0.0, 0.64),
    );

    let column_count = route_column_count(session.outposts.len());
    let route_rows = session.outposts.len().div_ceil(column_count).max(1);
    let desired_height = 112.0 + route_rows as f32 * CARD_HEIGHT + 56.0;
    let panel_height = desired_height.min(PANEL_BOTTOM - PANEL_TOP);
    let panel = Rect::new(PANEL_MARGIN_X, PANEL_TOP, PANEL_WIDTH, panel_height);
    draw_surface_with_title(
        panel,
        Some("Worm Route Ledger"),
        &panel_style(),
        TextStyle::new(21.0, dark::TEXT_BRIGHT),
    );
    draw_ui_text_ex(
        &format!(
            "{} awakened route{} · tap Inspect to open its existing Outpost card",
            session.outposts.len(),
            if session.outposts.len() == 1 { "" } else { "s" }
        ),
        panel.x + 24.0,
        panel.y + 58.0,
        TextStyle::new(14.0, dark::TEXT_DIM).params(),
    );

    let card_width =
        (panel.w - 48.0 - CARD_GAP * (column_count as f32 - 1.0)) / column_count as f32;
    for (index, outpost) in session.outposts.iter().enumerate() {
        let column = index % column_count;
        let row = index / column_count;
        let card = Rect::new(
            panel.x + 24.0 + column as f32 * (card_width + CARD_GAP),
            panel.y + 74.0 + row as f32 * CARD_HEIGHT,
            card_width,
            CARD_HEIGHT - CARD_GAP,
        );
        draw_route_card(session, data, index, outpost, card, mouse, actions);
    }

    if hud_button(
        Rect::new(
            panel.x + panel.w * 0.5 - 70.0,
            panel.bottom() - 44.0,
            140.0,
            32.0,
        ),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleRoutes);
    }
}

fn draw_route_card(
    session: &GameSession,
    data: &GameData,
    index: usize,
    outpost: &Outpost,
    card: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_surface(
        card,
        &SurfaceStyle::new(Color::new(0.08, 0.10, 0.13, 0.96))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.7)),
    );
    let x = card.x + 12.0;
    draw_ui_text_ex(
        &route_name(index, outpost.pos),
        x,
        card.y + 20.0,
        TextStyle::new(15.0, dark::TEXT_BRIGHT).params(),
    );

    let (status, status_color) = session
        .building_at(outpost.pos)
        .map(|building| inspect_status(session, data, building))
        .unwrap_or(("Missing building", dark::NEGATIVE));
    draw_ui_text_ex(
        status,
        x,
        card.y + 42.0,
        TextStyle::new(13.0, status_color).params(),
    );
    draw_ui_text_ex(
        &route_metrics(session, data, outpost),
        x,
        card.y + 62.0,
        TextStyle::new(12.0, dark::TEXT).params(),
    );
    draw_ui_text_ex(
        route_policy_label(outpost),
        x,
        card.y + 80.0,
        TextStyle::new(11.0, dark::TEXT_DIM).params(),
    );

    if hud_button(
        Rect::new(card.right() - 88.0, card.y + 28.0, 76.0, 32.0),
        "Inspect",
        true,
        mouse,
    ) {
        actions.push(UiAction::SelectBuilding(outpost.pos));
    }
}

fn route_name(index: usize, pos: TilePos) -> String {
    format!("Route {} · ({}, {})", index + 1, pos.x, pos.y)
}

fn route_column_count(route_count: usize) -> usize {
    if route_count > 6 {
        4
    } else {
        3
    }
}

fn route_metrics(session: &GameSession, data: &GameData, outpost: &Outpost) -> String {
    if let Some(transit) = session
        .worm_transit
        .as_ref()
        .filter(|transit| transit.outpost == outpost.pos)
    {
        let direction = match transit.direction {
            TransitDirection::ToOutpost => "outbound",
            TransitDirection::ToShrine => "returning",
        };
        let cargo = transit
            .ore
            .saturating_add(transit.ingots)
            .saturating_add(transit.food.max(0.0) as u32);
        return format!(
            "{direction} · {cargo} cargo · {} crew",
            transit.passengers.len()
        );
    }

    format!(
        "Cargo {}/{} · Crew {}/{}",
        outpost.cargo_total(),
        crate::simulation::outposts::storage_capacity(outpost, data),
        outpost.crew.len(),
        data.balance.outpost_capacity
    )
}

fn route_policy_label(outpost: &Outpost) -> &'static str {
    match (
        outpost.expedition_paused,
        outpost.auto_return_cargo,
        outpost.auto_resupply_food,
    ) {
        (true, _, _) => "Scouting paused",
        (false, true, true) => "Auto cargo return · Auto food resupply",
        (false, true, false) => "Auto cargo return",
        (false, false, true) => "Auto food resupply",
        (false, false, false) => "Manual route",
    }
}

#[cfg(test)]
mod tests;
