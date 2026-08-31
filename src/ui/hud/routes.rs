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

    let column_count = route_column_count(session.outposts.len());
    let relay_summary = relay_contract_summary(session, data);
    let convoy_summary = convoy_contract_summary(session, data);
    let muster_summary = muster_contract_summary(session, data);
    let concord_summary = concord_contract_summary(session, data);
    let transit_summary = worm_transit_summary(session);
    let summary_lines = 2
        + relay_summary.is_some() as u32
        + convoy_summary.is_some() as u32
        + muster_summary.is_some() as u32
        + concord_summary.is_some() as u32
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
    draw_ui_text_ex(
        &route_network_summary(session, data),
        panel.x + 24.0,
        panel.y + 58.0,
        TextStyle::new(14.0, dark::TEXT_DIM).params(),
    );
    let mut summary_y = panel.y + 76.0;
    draw_ui_text_ex(
        &format!("Auto dispatch · {}", session.auto_route_priority.label()),
        panel.x + 24.0,
        summary_y,
        TextStyle::new(13.0, dark::TEXT).params(),
    );
    if hud_button(
        Rect::new(panel.right() - 164.0, summary_y - 22.0, 140.0, 32.0),
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
    let compact = super::panels::compact_top_bar(ui_scale);
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

fn route_network_summary(session: &GameSession, data: &GameData) -> String {
    let active = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count();
    let held_cargo = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.cargo_total())
    });
    let remote_crew = session.outposts.iter().fold(0usize, |total, outpost| {
        total.saturating_add(outpost.crew.len())
    });
    let scouted_ore = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.ore_scouted)
    });
    let cached_ingots = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.signal_cache_ingots)
    });
    let attention = session
        .outposts
        .iter()
        .filter(|outpost| route_needs_attention(session, outpost, data))
        .count();
    let charter = charter_summary(session, data);
    let cache_summary = if cached_ingots > 0 {
        format!(" · Cache kept {cached_ingots}")
    } else {
        String::new()
    };
    format!(
        "Routes {} · Active {} · Held cargo {} · Remote crew {} · Ore scouted {} · {} · Attention {}{}",
        session.outposts.len(),
        active,
        held_cargo,
        remote_crew,
        scouted_ore,
        charter,
        attention,
        cache_summary
    )
}

fn charter_summary(session: &GameSession, data: &GameData) -> String {
    let goal = data.balance.outpost_charter_haul_goal;
    if session.outpost_charter_claimed {
        let archive_goal = data.balance.outpost_archive_haul_goal;
        if archive_goal == 0 {
            return "Charter complete".to_owned();
        }
        return format!(
            "Charter complete · Archive {}/{} · Pages {} · +{} ingots",
            crate::simulation::outposts::outpost_archive_progress(session, data),
            archive_goal,
            session.outpost_archive_claims,
            data.balance.outpost_archive_reward_ingots
        );
    }
    if goal == 0 {
        return "Charter unavailable".to_owned();
    }
    let completed = crate::simulation::outposts::total_expeditions(session).min(goal);
    format!(
        "Charter {completed}/{goal} · +{} ingots",
        data.balance.outpost_charter_reward_ingots
    )
}

fn relay_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if session.outpost_archive_claims == 0 || data.balance.outpost_relay_route_goal == 0 {
        return None;
    }
    if session.outpost_relay_claimed {
        return Some(format!(
            "Worm Road Relay complete · +{} ingots",
            data.balance.outpost_relay_reward_ingots
        ));
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_relay_progress(session);
    Some(format!(
        "Worm Road Relay · {active_routes}/{} routes · {completed_hauls}/{} hauls · +{} ingots",
        data.balance.outpost_relay_route_goal,
        data.balance.outpost_relay_haul_goal,
        data.balance.outpost_relay_reward_ingots
    ))
}

fn convoy_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_relay_claimed
        || data.balance.outpost_convoy_route_goal == 0
        || data.balance.outpost_convoy_haul_goal == 0
    {
        return None;
    }
    if session.outpost_convoy_claims > 0 {
        return Some(format!(
            "Worm Road Convoy · {} cleared · next {}/{} hauls · +{} ingots",
            session.outpost_convoy_claims,
            crate::simulation::outposts::outpost_convoy_progress(session, data).1,
            data.balance.outpost_convoy_haul_goal,
            data.balance.outpost_convoy_reward_ingots
        ));
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_convoy_progress(session, data);
    Some(format!(
        "Worm Road Convoy · {active_routes}/{} routes · {completed_hauls}/{} hauls · +{} ingots",
        data.balance.outpost_convoy_route_goal,
        data.balance.outpost_convoy_haul_goal,
        data.balance.outpost_convoy_reward_ingots
    ))
}

fn muster_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_relay_claimed
        || session.outpost_convoy_claims == 0
        || data.balance.outpost_muster_route_goal == 0
        || data.balance.outpost_muster_haul_goal == 0
    {
        return None;
    }
    if session.outpost_muster_claims > 0 {
        return Some(format!(
            "Worm Road Muster · {} cleared · next {}/{} hauls · +{} ingots",
            session.outpost_muster_claims,
            crate::simulation::outposts::outpost_muster_progress(session, data).1,
            data.balance.outpost_muster_haul_goal,
            data.balance.outpost_muster_reward_ingots
        ));
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_muster_progress(session, data);
    Some(format!(
        "Worm Road Muster · {active_routes}/{} routes · {completed_hauls}/{} hauls · +{} ingots",
        data.balance.outpost_muster_route_goal,
        data.balance.outpost_muster_haul_goal,
        data.balance.outpost_muster_reward_ingots
    ))
}

fn concord_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if session.outpost_muster_claims == 0 || data.balance.outpost_concord_role_goal == 0 {
        return None;
    }
    if session.outpost_concord_claimed {
        return Some(format!(
            "Wormsong Concord complete · {} roles on the road · +{} ingots",
            data.balance.outpost_concord_role_goal, data.balance.outpost_concord_reward_ingots
        ));
    }
    let (roles, active_routes) =
        crate::simulation::outposts::outpost_concord_progress(session, data);
    Some(format!(
        "Wormsong Concord · {roles}/{} roles · {active_routes} active routes · +{} ingots",
        data.balance.outpost_concord_role_goal, data.balance.outpost_concord_reward_ingots
    ))
}

fn worm_transit_summary(session: &GameSession) -> Option<String> {
    let transit = session.worm_transit.as_ref()?;
    let route = session
        .outposts
        .iter()
        .position(|outpost| outpost.pos == transit.outpost)
        .map(|index| format!("Route {}", index + 1))
        .unwrap_or_else(|| format!("Route at ({}, {})", transit.outpost.x, transit.outpost.y));
    let direction = match transit.direction {
        TransitDirection::ToOutpost => "outbound",
        TransitDirection::ToShrine => "returning",
    };
    Some(format!(
        "Worm in transit · {route} {direction} · {:.0}s remaining",
        transit.remaining.max(0.0)
    ))
}

fn automatic_route_summary(session: &GameSession, data: &GameData) -> String {
    if session.worm_transit.is_some() {
        return "Automatic jobs · waiting for Worm".to_owned();
    }
    let Some(preview) = crate::simulation::outposts::automatic_route_preview(session, data) else {
        return "Automatic jobs · none ready".to_owned();
    };
    let route = session
        .outposts
        .iter()
        .position(|outpost| outpost.pos == preview.outpost)
        .map(|index| format!("Route {}", index + 1))
        .unwrap_or_else(|| "Route".to_owned());
    format!(
        "Next automatic · {route} · {}",
        preview.priority.job_label()
    )
}

fn route_needs_attention(session: &GameSession, outpost: &Outpost, data: &GameData) -> bool {
    !outpost.active
        || outpost.last_failure.is_some()
        || matches!(
            crate::simulation::outposts::expedition_state_with_session(session, data, outpost),
            crate::simulation::outposts::ExpeditionState::NoCrew
                | crate::simulation::outposts::ExpeditionState::Paused
                | crate::simulation::outposts::ExpeditionState::NeedsFood { .. }
                | crate::simulation::outposts::ExpeditionState::HoldFull
        )
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

    let survey_summary = if outpost.deep_survey_upgraded {
        format!(
            " · Deep yield {}/scout",
            crate::simulation::outposts::ore_per_crew(outpost, data)
        )
    } else if outpost.survey_upgraded {
        format!(
            " · Yield {}/scout",
            crate::simulation::outposts::ore_per_crew(outpost, data)
        )
    } else {
        String::new()
    };
    let resonator_summary = if outpost.resonator_upgraded {
        format!(
            " · Cycle {:.0}s",
            crate::simulation::outposts::route_expedition_cycle_sec(session, data, outpost)
        )
    } else {
        String::new()
    };
    let cache_summary = if outpost.signal_cache_upgraded {
        format!(
            " · Cache +{}/haul · Kept {}",
            crate::simulation::outposts::route_signal_cache_ingots(session, data, outpost),
            outpost.signal_cache_ingots
        )
    } else {
        String::new()
    };
    let cargo_summary = format!(
        "Cargo {}/{} · Crew {}/{}{}{}",
        outpost.cargo_total(),
        crate::simulation::outposts::route_storage_capacity(session, data, outpost),
        outpost.crew.len(),
        crate::simulation::outposts::crew_capacity(outpost, data),
        survey_summary,
        cache_summary
    );
    let cargo_summary = format!("{cargo_summary}{resonator_summary}");
    match crate::simulation::outposts::expedition_state_with_session(session, data, outpost) {
        crate::simulation::outposts::ExpeditionState::Scouting {
            progress_percent, ..
        } => format!("{cargo_summary} · Scout {progress_percent}%"),
        _ => cargo_summary,
    }
}

fn route_policy_label(outpost: &Outpost) -> &'static str {
    match (
        outpost.expedition_paused,
        outpost.auto_load,
        outpost.auto_return_cargo,
        outpost.auto_resupply_food,
    ) {
        (true, _, _, _) => "Scouting paused",
        (false, true, true, true) => "Auto-load · Auto-return · Auto-resupply",
        (false, true, true, false) => "Auto-load · Auto-return",
        (false, true, false, true) => "Auto-load · Auto-resupply",
        (false, true, false, false) => "Auto-load route",
        (false, false, true, true) => "Auto cargo return · Auto food resupply",
        (false, false, true, false) => "Auto cargo return",
        (false, false, false, true) => "Auto food resupply",
        (false, false, false, false) => "Manual route",
    }
}

fn compact_route_metrics(session: &GameSession, data: &GameData, outpost: &Outpost) -> String {
    format!(
        "{}/{} cargo · {}/{} crew",
        outpost.cargo_total(),
        crate::simulation::outposts::route_storage_capacity(session, data, outpost),
        outpost.crew.len(),
        crate::simulation::outposts::crew_capacity(outpost, data)
    )
}

fn route_policy_summary(
    session: &GameSession,
    outpost: &Outpost,
    data: &GameData,
    compact: bool,
) -> String {
    let waypoint = outpost.waypoint_upgraded.then(|| {
        let transit_time = crate::simulation::outposts::transit_time_sec(outpost, data);
        if compact {
            format!(" · Waypoint {transit_time:.0}s")
        } else {
            format!(" · Waypoint {transit_time:.0}s transit")
        }
    });
    let policy = if !compact || !outpost.signal_cache_upgraded {
        format!(
            "{}{}",
            route_policy_label(outpost),
            waypoint.unwrap_or_default()
        )
    } else {
        let policy = match (
            outpost.expedition_paused,
            outpost.auto_load,
            outpost.auto_return_cargo,
            outpost.auto_resupply_food,
        ) {
            (true, _, _, _) => "Paused",
            (false, true, true, true) => "Auto-load + return + food",
            (false, true, true, false) => "Auto-load + return",
            (false, true, false, true) => "Auto-load + food",
            (false, true, false, false) => "Auto-load",
            (false, false, true, true) => "Auto return + food",
            (false, false, true, false) => "Auto return",
            (false, false, false, true) => "Auto food",
            (false, false, false, false) => "Manual",
        };
        format!(
            "{policy} · Cache +{} · Kept {}",
            crate::simulation::outposts::route_signal_cache_ingots(session, data, outpost),
            outpost.signal_cache_ingots
        ) + waypoint.as_deref().unwrap_or_default()
    };
    if let Some(bonus) =
        crate::simulation::outposts::route_bonus_summary(session, data, outpost, compact)
    {
        format!("{policy} · {bonus}")
    } else {
        policy
    }
}

#[cfg(test)]
mod tests;
