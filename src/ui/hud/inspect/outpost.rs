//! Compact layout for the awakened Outpost's route controls.

use crate::data::GameData;
use crate::state::outposts::Outpost;
use crate::state::GameSession;
use crate::ui::hud::widgets::hud_button;
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;

pub(super) fn outpost_archive_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_charter_claimed || data.balance.outpost_archive_haul_goal == 0 {
        return None;
    }
    Some(format!(
        "Archive pages {} · Next {}/{}",
        session.outpost_archive_claims,
        crate::simulation::outposts::outpost_archive_progress(session, data),
        data.balance.outpost_archive_haul_goal
    ))
}

pub(super) fn outpost_signal_cache_summary(
    outpost: &Outpost,
    data: &GameData,
    compact: bool,
) -> Option<String> {
    outpost.signal_cache_upgraded.then(|| {
        if compact {
            format!(
                "Cache +{}/haul · Kept {}",
                data.balance.outpost_signal_cache_ingots_per_haul, outpost.signal_cache_ingots
            )
        } else {
            format!(
                "Signal cache · +{}/haul · Kept {}",
                data.balance.outpost_signal_cache_ingots_per_haul, outpost.signal_cache_ingots
            )
        }
    })
}

pub(super) fn outpost_waypoint_summary(
    outpost: &Outpost,
    data: &GameData,
    compact: bool,
) -> Option<String> {
    outpost.waypoint_upgraded.then(|| {
        let transit_time = crate::simulation::outposts::transit_time_sec(outpost, data);
        if compact {
            format!("Waypoint · {transit_time:.0}s")
        } else {
            format!("Worm Road Waypoint · {transit_time:.0}s transit")
        }
    })
}

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

pub(super) struct FullRouteContext<'a> {
    pub(super) session: &'a GameSession,
    pub(super) data: &'a GameData,
    pub(super) pos: TilePos,
    pub(super) outpost: Option<&'a Outpost>,
    pub(super) x: f32,
    pub(super) y: &'a mut f32,
    pub(super) width: f32,
    pub(super) crew: usize,
    pub(super) mouse: Vec2,
    pub(super) actions: &'a mut Vec<UiAction>,
}

pub(super) struct SurveyUpgradeContext<'a> {
    pub(super) session: &'a GameSession,
    pub(super) data: &'a GameData,
    pub(super) pos: TilePos,
    pub(super) outpost: Option<&'a Outpost>,
    pub(super) rect: Rect,
    pub(super) y: &'a mut f32,
    pub(super) step: f32,
    pub(super) mouse: Vec2,
    pub(super) actions: &'a mut Vec<UiAction>,
}

pub(super) struct RouteUpgradeContext<'a> {
    pub(super) session: &'a GameSession,
    pub(super) data: &'a GameData,
    pub(super) pos: TilePos,
    pub(super) outpost: Option<&'a Outpost>,
    pub(super) x: f32,
    pub(super) width: f32,
    pub(super) y: &'a mut f32,
    pub(super) button_height: f32,
    pub(super) button_step: f32,
    pub(super) mouse: Vec2,
    pub(super) actions: &'a mut Vec<UiAction>,
}

pub(super) fn draw_survey_upgrade_control(context: SurveyUpgradeContext<'_>) {
    let SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step,
        mouse,
        actions,
    } = context;
    if !outpost.is_some_and(|route| {
        route.storage_upgraded && route.crew_upgraded && !route.survey_upgraded
    }) {
        return;
    }
    let upgrade_cost = data.balance.outpost_survey_upgrade_ingots;
    if hud_button(
        Rect::new(rect.x, *y, rect.w, rect.h),
        &format!("Install survey · {upgrade_cost} ingots"),
        session.economy.ingots_stock >= upgrade_cost,
        mouse,
    ) {
        actions.push(UiAction::UpgradeOutpostSurvey(pos));
    }
    *y += step;
}

pub(super) fn draw_resonator_upgrade_control(context: SurveyUpgradeContext<'_>) {
    let SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step,
        mouse,
        actions,
    } = context;
    if !outpost.is_some_and(|route| route.survey_upgraded && !route.resonator_upgraded) {
        return;
    }
    let upgrade_cost = data.balance.outpost_resonator_upgrade_ingots;
    if hud_button(
        Rect::new(rect.x, *y, rect.w, rect.h),
        &format!("Tune beacon · {upgrade_cost} ingots"),
        session.economy.ingots_stock >= upgrade_cost,
        mouse,
    ) {
        actions.push(UiAction::UpgradeOutpostResonator(pos));
    }
    *y += step;
}

pub(super) fn draw_deep_survey_control(context: SurveyUpgradeContext<'_>) {
    let SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step,
        mouse,
        actions,
    } = context;
    if !outpost.is_some_and(|route| route.resonator_upgraded && !route.deep_survey_upgraded) {
        return;
    }
    let upgrade_cost = data.balance.outpost_deep_survey_upgrade_ingots;
    let charter_claimed = session.outpost_charter_claimed;
    let label = if charter_claimed {
        format!("Calibrate deep survey · {upgrade_cost} ingots")
    } else {
        "Deep survey · Charter required".to_owned()
    };
    if hud_button(
        Rect::new(rect.x, *y, rect.w, rect.h),
        &label,
        charter_claimed && session.economy.ingots_stock >= upgrade_cost,
        mouse,
    ) {
        actions.push(UiAction::UpgradeOutpostDeepSurvey(pos));
    }
    *y += step;
}

pub(super) fn draw_signal_cache_control(context: SurveyUpgradeContext<'_>) {
    let SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step,
        mouse,
        actions,
    } = context;
    if !outpost.is_some_and(|route| route.deep_survey_upgraded && !route.signal_cache_upgraded) {
        return;
    }
    let relay_claimed = session.outpost_relay_claimed;
    let cache_cost = data.balance.outpost_signal_cache_upgrade_ingots;
    let label = if relay_claimed {
        format!("Install signal cache · {cache_cost} ingots")
    } else {
        "Signal cache · Relay required".to_owned()
    };
    if hud_button(
        Rect::new(rect.x, *y, rect.w, rect.h),
        &label,
        relay_claimed && session.economy.ingots_stock >= cache_cost,
        mouse,
    ) {
        actions.push(UiAction::UpgradeOutpostSignalCache(pos));
    }
    *y += step;
}

pub(super) fn draw_waypoint_control(context: SurveyUpgradeContext<'_>) {
    let SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step,
        mouse,
        actions,
    } = context;
    if !outpost.is_some_and(|route| route.signal_cache_upgraded && !route.waypoint_upgraded) {
        return;
    }
    let waypoint_cost = data.balance.outpost_waypoint_upgrade_ingots;
    let convoy_claimed = session.outpost_convoy_claims > 0;
    let label = if convoy_claimed {
        format!("Install Worm Road waypoint · {waypoint_cost} ingots")
    } else {
        "Worm Road waypoint · Convoy required".to_owned()
    };
    if hud_button(
        Rect::new(rect.x, *y, rect.w, rect.h),
        &label,
        convoy_claimed && session.economy.ingots_stock >= waypoint_cost,
        mouse,
    ) {
        actions.push(UiAction::UpgradeOutpostWaypoint(pos));
    }
    *y += step;
}

pub(super) fn draw_route_upgrade_controls(context: RouteUpgradeContext<'_>) {
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
    if outpost.is_some_and(|route| !route.storage_upgraded) {
        let upgrade_cost = data.balance.outpost_upgrade_ingots;
        if hud_button(
            Rect::new(x, *y, width, button_height),
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
            Rect::new(x, *y, width, button_height),
            &format!("Expand camp · {upgrade_cost} ingots"),
            session.economy.ingots_stock >= upgrade_cost,
            mouse,
        ) {
            actions.push(UiAction::UpgradeOutpostCrew(pos));
        }
        *y += button_step;
    }
    let rect = Rect::new(x, 0.0, width, button_height);
    draw_survey_upgrade_control(SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step: button_step,
        mouse,
        actions,
    });
    draw_resonator_upgrade_control(SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step: button_step,
        mouse,
        actions,
    });
    draw_deep_survey_control(SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step: button_step,
        mouse,
        actions,
    });
    draw_signal_cache_control(SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step: button_step,
        mouse,
        actions,
    });
    draw_waypoint_control(SurveyUpgradeContext {
        session,
        data,
        pos,
        outpost,
        rect,
        y,
        step: button_step,
        mouse,
        actions,
    });
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

    let auto_load_label = if outpost.is_some_and(|route| route.auto_load) {
        "Dispatch: auto"
    } else {
        "Dispatch: manual"
    };
    if hud_button(
        Rect::new(x, *y, panel.w - 28.0, button_height),
        auto_load_label,
        session.worm_transit.is_none(),
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoLoad(pos));
    }
    *y += button_step;

    draw_route_upgrade_controls(RouteUpgradeContext {
        session,
        data,
        pos,
        outpost,
        x,
        width: panel.w - 28.0,
        y,
        button_height,
        button_step,
        mouse,
        actions,
    });
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

pub(super) fn draw_full_route_controls(context: FullRouteContext<'_>) {
    let FullRouteContext {
        session,
        data,
        pos,
        outpost,
        x,
        y,
        width,
        crew,
        mouse,
        actions,
    } = context;
    let route_controls_enabled = session.worm_transit.is_none();
    let auto_return_label = outpost
        .map(|route| route.auto_return_label())
        .unwrap_or("Auto-return · Off");
    if hud_button(
        Rect::new(x, *y, width, 24.0),
        auto_return_label,
        route_controls_enabled,
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoReturn(pos));
    }
    *y += 26.0;
    let auto_resupply_label = outpost
        .map(|route| route.auto_resupply_label())
        .unwrap_or("Auto-resupply · Off");
    if hud_button(
        Rect::new(x, *y, width, 24.0),
        auto_resupply_label,
        route_controls_enabled,
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoResupply(pos));
    }
    *y += 26.0;
    let auto_load_label = outpost
        .map(|route| route.auto_load_label())
        .unwrap_or("Auto-load · Off");
    if hud_button(
        Rect::new(x, *y, width, 24.0),
        auto_load_label,
        route_controls_enabled,
        mouse,
    ) {
        actions.push(UiAction::ToggleOutpostAutoLoad(pos));
    }
    *y += 26.0;
    let priority = outpost
        .map(|route| route.cargo_priority.label())
        .unwrap_or("Ore first");
    if hud_button(
        Rect::new(x, *y, width, 24.0),
        &format!("Load order · {priority}"),
        route_controls_enabled,
        mouse,
    ) {
        actions.push(UiAction::CycleOutpostCargo(pos));
    }
    *y += 26.0;
    let crew_label = outpost
        .map(|route| {
            route.crew_dispatch_label(crate::simulation::outposts::crew_capacity(route, data))
        })
        .unwrap_or_else(|| "Crew per run · Auto".to_owned());
    if hud_button(
        Rect::new(x, *y, width, 24.0),
        &crew_label,
        route_controls_enabled,
        mouse,
    ) {
        actions.push(UiAction::CycleOutpostCrew(pos));
    }
    *y += 26.0;
    draw_route_upgrade_controls(RouteUpgradeContext {
        session,
        data,
        pos,
        outpost,
        x,
        width,
        y,
        button_height: 24.0,
        button_step: 26.0,
        mouse,
        actions,
    });
    if crew > 0
        && hud_button(
            Rect::new(x, *y, width, 24.0),
            if outpost.is_some_and(|route| route.expedition_paused) {
                "Resume scouting"
            } else {
                "Pause scouting"
            },
            route_controls_enabled,
            mouse,
        )
    {
        actions.push(UiAction::ToggleOutpostExpedition(pos));
    }
    *y += 38.0;
}
