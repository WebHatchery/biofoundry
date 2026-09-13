//! Outpost inspection details and route controls.

use super::*;

// The renderer receives the shared card geometry and action sink separately
// so the compact and full route controls can keep their own context structs.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_outpost_details(
    session: &GameSession,
    data: &GameData,
    pos: TilePos,
    panel: Rect,
    mouse: Vec2,
    compact: bool,
    compact_outpost: bool,
    outpost_button_height: f32,
    outpost_button_step: f32,
    line_step: f32,
    x: f32,
    y: &mut f32,
    actions: &mut Vec<UiAction>,
) {
    let line = |text: &str, color: Color, y: &mut f32| {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, color).params());
        *y += line_step;
    };
    let outpost = session.outposts.iter().find(|o| o.pos == pos);
    let active = outpost.is_some_and(|o| o.active);
    let cargo = outpost.map(|o| o.cargo_total()).unwrap_or(0);
    let crew = outpost.map(|o| o.crew.len()).unwrap_or(0);
    let crew_cap = outpost
        .map(|route| crate::simulation::outposts::crew_capacity(route, data))
        .unwrap_or(data.balance.outpost_capacity);
    let crew_dispatch_limit = outpost.and_then(|route| route.crew_dispatch_limit);
    let storage_cap = outpost
        .map(|route| crate::simulation::outposts::route_storage_capacity(session, data, route))
        .unwrap_or(data.balance.outpost_storage_cap);
    let cargo_ore = outpost
        .and_then(|o| o.cargo.get(&Good::Ore))
        .copied()
        .unwrap_or(0);
    let cargo_ingots = outpost
        .and_then(|o| o.cargo.get(&Good::Ingot))
        .copied()
        .unwrap_or(0);
    let cargo_food = outpost
        .and_then(|o| o.cargo.get(&Good::CookedFood))
        .copied()
        .unwrap_or(0);
    line(
        &format!(
            "{} · Cargo {}/{} · Crew {}/{}",
            if active { "Active" } else { "Inactive" },
            cargo,
            storage_cap,
            crew,
            crew_cap
        ),
        if active {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
        y,
    );
    line(
        &format!(
            "Ore {} · Ingots {} · Food {}",
            cargo_ore, cargo_ingots, cargo_food
        ),
        dark::TEXT_DIM,
        y,
    );
    if active && session.worm_awake {
        if let Some(route) = outpost {
            if compact_outpost {
                let mut route_details = vec![format!(
                    "Scouted ore {} · Hauls {}",
                    route.ore_scouted, route.expeditions_completed
                )];
                if let Some(route_bonus) =
                    crate::simulation::outposts::route_bonus_summary(session, data, route, true)
                {
                    route_details.push(route_bonus);
                }
                if let Some(archive) = outpost::compact_archive_summary(session, data) {
                    route_details.push(archive);
                }
                if let Some(cache_summary) = outpost_signal_cache_summary(route, data, true) {
                    route_details.push(cache_summary);
                }
                if let Some(waypoint_summary) = outpost_waypoint_summary(route, data, true) {
                    route_details.push(waypoint_summary);
                }
                line(&route_details.join(" · "), dark::TEXT_DIM, y);
            } else {
                line(
                    &format!(
                        "Scouted ore {} · Hauls {}",
                        route.ore_scouted, route.expeditions_completed
                    ),
                    dark::TEXT_DIM,
                    y,
                );
                if let Some(route_bonus) =
                    crate::simulation::outposts::route_bonus_summary(session, data, route, false)
                {
                    line(&route_bonus, dark::POSITIVE, y);
                }
                outpost::draw_archive_summary(session, data, x, y, false);
                if let Some(cache_summary) = outpost_signal_cache_summary(route, data, false) {
                    line(&cache_summary, dark::POSITIVE, y);
                }
                if let Some(waypoint_summary) = outpost_waypoint_summary(route, data, false) {
                    line(&waypoint_summary, dark::POSITIVE, y);
                }
            }
        }
        let in_transit = session
            .worm_transit
            .as_ref()
            .is_some_and(|transit| transit.outpost == pos);
        if !in_transit {
            if let Some(expedition_hint) =
                outpost.and_then(|route| outpost_expedition_hint_with_session(session, data, route))
            {
                line(&expedition_hint, dark::TEXT_DIM, y);
            }
        }
    }
    if let Some(transit) = session.worm_transit.as_ref() {
        if transit.outpost == pos {
            line(&transit_payload_line(transit), dark::POSITIVE, y);
        }
        let route = if transit.outpost == pos {
            transit_destination(transit.direction)
        } else {
            "another outpost"
        };
        line(
            &format!(
                "Transit to {route} · {:.0}s remaining",
                transit.remaining.max(0.0)
            ),
            dark::POSITIVE,
            y,
        );
        line("Wait for the worm to arrive", dark::TEXT_DIM, y);
    } else {
        if hud_button(
            Rect::new(x, *y, panel.w - 28.0, outpost_button_height),
            if active {
                "Deactivate route"
            } else {
                "Activate route"
            },
            true,
            mouse,
        ) {
            actions.push(UiAction::ActivateOutpost(pos));
        }
        // Leave a full text-line gap before recovery copy so the
        // baseline cannot crowd the button's lower border.
        *y += if compact_outpost {
            outpost_button_step
        } else if compact {
            34.0
        } else {
            32.0
        };
        if !active && (cargo > 0 || crew > 0) {
            line("Reactivate route to return payload", dark::WARNING, y);
        }
        if active && session.worm_awake {
            let loadable_payload = outpost_has_loadable_payload(
                session,
                data,
                cargo,
                crew,
                storage_cap,
                crew_cap,
                crew_dispatch_limit,
            );
            if compact {
                draw_compact_route_controls(outpost::CompactRouteContext {
                    session,
                    data,
                    pos,
                    outpost,
                    panel,
                    x,
                    y,
                    crew,
                    button_height: outpost_button_height,
                    button_step: outpost_button_step,
                    mouse,
                    actions,
                });
            } else {
                let return_label = outpost_return_label(cargo, crew);
                if hud_button(
                    Rect::new(x, *y, panel.w - 28.0, outpost_button_height),
                    &return_label,
                    cargo > 0 || crew > 0,
                    mouse,
                ) {
                    actions.push(UiAction::TransitToShrine(pos));
                }
                *y += 24.0;
                if cargo > 0
                    && crew > 0
                    && hud_button(
                        Rect::new(x, *y, panel.w - 28.0, outpost_button_height),
                        &outpost_cargo_only_return_label(cargo),
                        true,
                        mouse,
                    )
                {
                    actions.push(UiAction::TransitCargoToShrine(pos));
                }
                if cargo > 0 && crew > 0 {
                    *y += 26.0;
                }
                draw_full_route_controls(FullRouteContext {
                    session,
                    data,
                    pos,
                    outpost,
                    x,
                    y,
                    width: panel.w - 28.0,
                    crew,
                    mouse,
                    actions,
                });
            }
            if !compact_outpost {
                if let Some(load_hint) =
                    outpost.and_then(|route| outpost_load_hint(session, data, route))
                {
                    line(&load_hint, dark::TEXT_DIM, y);
                }
            }
            if hud_button(
                Rect::new(x, *y, panel.w - 28.0, outpost_button_height),
                "Load outpost from warren",
                loadable_payload,
                mouse,
            ) {
                actions.push(UiAction::TransitToOutpost(pos));
            }
            *y += 36.0;
            if cargo == 0 && crew == 0 && !loadable_payload {
                line("No payload ready at warren", dark::WARNING, y);
            }
        } else if active {
            line("Awaiting the worm's awakening", dark::TEXT_DIM, y);
        }
    }
    if let Some(failure) = outpost.and_then(|o| o.last_failure.as_deref()) {
        draw_text_block(
            failure,
            x,
            *y,
            panel.w - 28.0,
            38.0,
            13.0,
            3.0,
            dark::NEGATIVE,
        );
    }
}
