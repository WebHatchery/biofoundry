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

mod network;

pub use network::compact_route_network_summary;

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
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.64),
    );

    let compact = super::panels::compact_top_bar(ui_scale);
    let column_count = route_column_count(session.outposts.len());
    let relay_summary = relay_contract_summary(session, data);
    let convoy_summary = convoy_contract_summary(session, data);
    let muster_summary = muster_contract_summary(session, data);
    let concord_summary = concord_contract_summary(session, data);
    let circuit_summary = circuit_contract_summary(session, data);
    let encore_summary = encore_contract_summary(session, data);
    let chorus_summary = chorus_contract_summary(session, data);
    let transit_summary = worm_transit_summary(session);
    let summary_lines = 2
        + compact as u32
        + relay_summary.is_some() as u32
        + convoy_summary.is_some() as u32
        + muster_summary.is_some() as u32
        + concord_summary.is_some() as u32
        + circuit_summary.is_some() as u32
        + encore_summary.is_some() as u32
        + chorus_summary.is_some() as u32
        + transit_summary.is_some() as u32;
    let summary_extra = summary_lines as f32 * 18.0;
    let route_rows = session.outposts.len().div_ceil(column_count).max(1);
    let desired_height = 112.0 + summary_extra + route_rows as f32 * CARD_HEIGHT + 56.0;
    let panel_height = desired_height.min(PANEL_BOTTOM - PANEL_TOP);
    let panel = Rect::new(PANEL_MARGIN_X, PANEL_TOP, PANEL_WIDTH, panel_height);
    draw_surface_with_title(
        panel,
        Some("Worm Route Ledger"),
        &panel_style(),
        TextStyle::new(21.0, dark::TEXT_BRIGHT),
    );
    // The ledger owns the dimmed route-planning screen. Remove all covered HUD
    // controls from the touch audit before the card buttons register their own
    // hit areas, otherwise a route tap can be reported as ambiguous with a
    // control painted underneath the modal.
    macroquad_toolkit::ui::occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let mut summary_y = panel.y + 58.0;
    if compact {
        let (headline, detail) = compact_route_network_summary(session, data);
        draw_ui_text_ex(
            &headline,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(14.0, dark::TEXT_DIM).params(),
        );
        summary_y += 18.0;
        draw_ui_text_ex(
            &detail,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::TEXT_DIM).params(),
        );
    } else {
        draw_ui_text_ex(
            &route_network_summary(session, data),
            panel.x + 24.0,
            summary_y,
            TextStyle::new(14.0, dark::TEXT_DIM).params(),
        );
    }
    summary_y += 18.0;
    draw_ui_text_ex(
        &format!("Auto dispatch · {}", session.auto_route_priority.label()),
        panel.x + 24.0,
        summary_y,
        TextStyle::new(13.0, dark::TEXT).params(),
    );
    let cycle_height = if compact { 72.0 } else { 32.0 };
    let cycle_y = summary_y - if compact { 42.0 } else { 22.0 };
    if hud_button(
        Rect::new(panel.right() - 164.0, cycle_y, 140.0, cycle_height),
        "Cycle order",
        true,
        mouse,
    ) {
        actions.push(UiAction::CycleAutoRoutePriority);
    }
    summary_y += 18.0;
    draw_ui_text_ex(
        &automatic_route_summary(session, data),
        panel.x + 24.0,
        summary_y,
        TextStyle::new(13.0, dark::TEXT_DIM).params(),
    );
    summary_y += 18.0;
    if let Some(summary) = relay_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = convoy_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = muster_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = concord_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = circuit_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = encore_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = chorus_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::POSITIVE).params(),
        );
        summary_y += 18.0;
    }
    if let Some(summary) = transit_summary {
        draw_ui_text_ex(
            &summary,
            panel.x + 24.0,
            summary_y,
            TextStyle::new(13.0, dark::WARNING).params(),
        );
    }

    let card_width =
        (panel.w - 48.0 - CARD_GAP * (column_count as f32 - 1.0)) / column_count as f32;
    for (index, outpost) in session.outposts.iter().enumerate() {
        let column = index % column_count;
        let row = index / column_count;
        let card = Rect::new(
            panel.x + 24.0 + column as f32 * (card_width + CARD_GAP),
            panel.y + 74.0 + summary_extra + row as f32 * CARD_HEIGHT,
            card_width,
            CARD_HEIGHT - CARD_GAP,
        );
        draw_route_card(RouteCardContext {
            session,
            data,
            index,
            outpost,
            card,
            mouse,
            compact,
            actions,
        });
    }

    let close_height = if compact { 72.0 } else { 32.0 };
    let close_y = panel.bottom() - if compact { 84.0 } else { 44.0 };
    if hud_button(
        Rect::new(panel.x + panel.w * 0.5 - 70.0, close_y, 140.0, close_height),
        "Close",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleRoutes);
    }
}

struct RouteCardContext<'a> {
    session: &'a GameSession,
    data: &'a GameData,
    index: usize,
    outpost: &'a Outpost,
    card: Rect,
    mouse: Vec2,
    compact: bool,
    actions: &'a mut Vec<UiAction>,
}

fn draw_route_card(context: RouteCardContext<'_>) {
    let RouteCardContext {
        session,
        data,
        index,
        outpost,
        card,
        mouse,
        compact,
        actions,
    } = context;
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
    let metrics = if compact {
        compact_route_metrics(session, data, outpost)
    } else {
        route_metrics(session, data, outpost)
    };
    draw_ui_text_ex(
        &metrics,
        x,
        card.y + 62.0,
        TextStyle::new(12.0, dark::TEXT).params(),
    );
    draw_ui_text_ex(
        &route_policy_summary(session, outpost, data, compact),
        x,
        card.y + 80.0,
        TextStyle::new(11.0, dark::TEXT_DIM).params(),
    );

    let inspect_height = if compact { 72.0 } else { 32.0 };
    let inspect_y = card.y + if compact { 14.0 } else { 28.0 };
    if hud_button(
        Rect::new(card.right() - 88.0, inspect_y, 76.0, inspect_height),
        "Inspect",
        true,
        mouse,
    ) {
        actions.push(UiAction::SelectBuilding(outpost.pos));
    }
}

mod summaries;

pub use summaries::{
    automatic_route_summary, charter_summary, chorus_contract_summary, circuit_contract_summary,
    compact_route_metrics, concord_contract_summary, convoy_contract_summary,
    encore_contract_summary, muster_contract_summary, relay_contract_summary, route_column_count,
    route_metrics, route_name, route_needs_attention, route_network_summary, route_policy_label,
    route_policy_summary, worm_transit_summary,
};
