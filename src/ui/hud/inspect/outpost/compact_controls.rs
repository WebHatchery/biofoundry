//! Compact route-control layout for the awakened Outpost inspection card.

use super::{CompactRouteContext, RouteUpgradeContext};
use crate::ui::hud::widgets::hud_button;
use crate::ui::UiAction;
use macroquad::prelude::*;

pub(super) fn draw_compact_route_return_controls(context: CompactRouteContext<'_>) {
    let CompactRouteContext {
        session,
        outpost,
        panel,
        x,
        y,
        crew,
        button_height,
        button_step,
        mouse,
        actions,
        ..
    } = context;
    let cargo = outpost.map(|route| route.cargo_total()).unwrap_or(0);
    let enabled = session.worm_transit.is_none();
    let Some(route) = outpost else {
        hud_button(
            Rect::new(x, *y, panel.w - 28.0, button_height),
            "No cargo or crew to return",
            false,
            mouse,
        );
        *y += button_step;
        return;
    };
    if cargo > 0 && crew > 0 {
        let column_width = (panel.w - 36.0) * 0.5;
        let second_column_x = x + column_width + 8.0;
        if hud_button(
            Rect::new(x, *y, column_width, button_height),
            "Return all",
            enabled,
            mouse,
        ) {
            actions.push(UiAction::TransitToShrine(route.pos));
        }
        if hud_button(
            Rect::new(second_column_x, *y, column_width, button_height),
            "Return cargo",
            enabled,
            mouse,
        ) {
            actions.push(UiAction::TransitCargoToShrine(route.pos));
        }
    } else if hud_button(
        Rect::new(x, *y, panel.w - 28.0, button_height),
        &super::super::outpost_return_label(cargo, crew),
        enabled && (cargo > 0 || crew > 0),
        mouse,
    ) {
        actions.push(UiAction::TransitToShrine(route.pos));
    }
    *y += button_step;
}

/// Pair the sequential route upgrades on a narrow card. Their compact labels
/// keep the visible cost and prerequisite while using the inspection text for
/// the full upgrade explanation.
pub(super) fn draw_compact_route_upgrade_controls(context: RouteUpgradeContext<'_>) {
    let RouteUpgradeContext {
        session,
        data,
        pos,
        outpost,
        x,
        width,
        y,
        button_height,
        button_step,
        mouse,
        actions,
    } = context;
    let mut controls = Vec::new();
    if outpost.is_some_and(|route| !route.storage_upgraded) {
        let cost = data.balance.outpost_upgrade_ingots;
        controls.push((
            format!("Hold · {cost}"),
            session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpost(pos),
        ));
    }
    if outpost.is_some_and(|route| !route.crew_upgraded) {
        let cost = data.balance.outpost_crew_upgrade_ingots;
        controls.push((
            format!("Camp · {cost}"),
            session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostCrew(pos),
        ));
    }
    if outpost.is_some_and(|route| {
        route.storage_upgraded && route.crew_upgraded && !route.survey_upgraded
    }) {
        let cost = data.balance.outpost_survey_upgrade_ingots;
        controls.push((
            format!("Survey · {cost}"),
            session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostSurvey(pos),
        ));
    }
    if outpost.is_some_and(|route| route.survey_upgraded && !route.resonator_upgraded) {
        let cost = data.balance.outpost_resonator_upgrade_ingots;
        controls.push((
            format!("Beacon · {cost}"),
            session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostResonator(pos),
        ));
    }
    if outpost.is_some_and(|route| route.resonator_upgraded && !route.deep_survey_upgraded) {
        let charter_claimed = session.outpost_charter_claimed;
        let cost = data.balance.outpost_deep_survey_upgrade_ingots;
        controls.push((
            if charter_claimed {
                format!("Deep · {cost}")
            } else {
                "Deep · Charter".to_owned()
            },
            charter_claimed && session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostDeepSurvey(pos),
        ));
    }
    if outpost.is_some_and(|route| route.deep_survey_upgraded && !route.signal_cache_upgraded) {
        let relay_claimed = session.outpost_relay_claimed;
        let cost = data.balance.outpost_signal_cache_upgrade_ingots;
        controls.push((
            if relay_claimed {
                format!("Cache · {cost}")
            } else {
                "Cache · Relay".to_owned()
            },
            relay_claimed && session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostSignalCache(pos),
        ));
    }
    if outpost.is_some_and(|route| route.signal_cache_upgraded && !route.waypoint_upgraded) {
        let convoy_claimed = session.outpost_convoy_claims > 0;
        let cost = data.balance.outpost_waypoint_upgrade_ingots;
        controls.push((
            if convoy_claimed {
                format!("Waypoint · {cost}")
            } else {
                "Waypoint · Convoy".to_owned()
            },
            convoy_claimed && session.economy.ingots_stock >= cost,
            UiAction::UpgradeOutpostWaypoint(pos),
        ));
    }

    let column_width = (width - 8.0) * 0.5;
    let control_count = controls.len();
    for (index, (label, enabled, action)) in controls.into_iter().enumerate() {
        let column = index % 2;
        if hud_button(
            Rect::new(
                x + column as f32 * (column_width + 8.0),
                *y,
                column_width,
                button_height,
            ),
            &label,
            enabled,
            mouse,
        ) {
            actions.push(action);
        }
        if column == 1 {
            *y += button_step;
        }
    }
    if control_count % 2 == 1 {
        *y += button_step;
    }
}
