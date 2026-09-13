//! Derived route summaries and labels for the Outpost ledger.

use super::*;

pub fn route_name(index: usize, pos: TilePos) -> String {
    format!("Route {} · ({}, {})", index + 1, pos.x, pos.y)
}

pub fn route_column_count(route_count: usize) -> usize {
    if route_count > 6 {
        4
    } else {
        3
    }
}

pub fn route_network_summary(session: &GameSession, data: &GameData) -> String {
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
    let chorus_ingots: u32 = session
        .outposts
        .iter()
        .map(|outpost| outpost.chorus_ingots)
        .sum();
    let chorus_summary = if chorus_ingots > 0 {
        format!(" · Chorus kept {chorus_ingots}")
    } else {
        String::new()
    };
    format!(
        "Routes {} · Active {} · Held cargo {} · Remote crew {} · Ore scouted {} · {} · Attention {}{}{}",
        session.outposts.len(),
        active,
        held_cargo,
        remote_crew,
        scouted_ore,
        charter,
        attention,
        cache_summary,
        chorus_summary
    )
}

pub fn charter_summary(session: &GameSession, data: &GameData) -> String {
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

pub fn relay_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
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

pub fn convoy_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
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

pub fn muster_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
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

pub fn concord_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
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

pub fn circuit_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_concord_claimed || data.balance.outpost_circuit_route_goal < 2 {
        return None;
    }
    if session.outpost_circuit_claimed {
        return Some(format!(
            "Wormsong Circuit complete · {} complete routes · +{} ingots",
            data.balance.outpost_circuit_route_goal, data.balance.outpost_circuit_reward_ingots
        ));
    }
    let (complete_routes, active_routes) =
        crate::simulation::outposts::outpost_circuit_progress(session, data);
    Some(format!(
        "Wormsong Circuit · {complete_routes}/{} complete routes · {active_routes} active routes · +{} ingots",
        data.balance.outpost_circuit_route_goal,
        data.balance.outpost_circuit_reward_ingots
    ))
}

pub fn encore_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_circuit_claimed || data.balance.outpost_encore_haul_goal == 0 {
        return None;
    }
    let progress = crate::simulation::outposts::outpost_encore_progress(session, data);
    if session.outpost_encore_claims > 0 {
        return Some(format!(
            "Wormsong Encore · {} cleared · next {progress}/{} boosted hauls · +{} ingots",
            session.outpost_encore_claims,
            data.balance.outpost_encore_haul_goal,
            data.balance.outpost_encore_reward_ingots
        ));
    }
    Some(format!(
        "Wormsong Encore · {progress}/{} boosted hauls · +{} ingots",
        data.balance.outpost_encore_haul_goal, data.balance.outpost_encore_reward_ingots
    ))
}

pub fn chorus_contract_summary(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_circuit_claimed || data.balance.outpost_chorus_route_goal == 0 {
        return None;
    }
    let (complete_routes, active_routes) =
        crate::simulation::outposts::outpost_chorus_progress(session, data);
    if session.outpost_chorus_claimed {
        return Some(format!(
            "Wormsong Chorus complete · {} complete routes · +{} ingot/haul · Kept {}",
            data.balance.outpost_chorus_route_goal,
            data.balance.outpost_chorus_ingots_per_haul,
            session
                .outposts
                .iter()
                .map(|outpost| outpost.chorus_ingots)
                .sum::<u32>()
        ));
    }
    Some(format!(
        "Wormsong Chorus · {complete_routes}/{} complete routes · {active_routes} active routes · +{} ingots",
        data.balance.outpost_chorus_route_goal,
        data.balance.outpost_chorus_reward_ingots
    ))
}

pub fn worm_transit_summary(session: &GameSession) -> Option<String> {
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

pub fn automatic_route_summary(session: &GameSession, data: &GameData) -> String {
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

pub fn route_needs_attention(session: &GameSession, outpost: &Outpost, data: &GameData) -> bool {
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

pub fn route_metrics(session: &GameSession, data: &GameData, outpost: &Outpost) -> String {
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

pub fn route_policy_label(outpost: &Outpost) -> &'static str {
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

pub fn compact_route_metrics(session: &GameSession, data: &GameData, outpost: &Outpost) -> String {
    format!(
        "{}/{} cargo · {}/{} crew",
        outpost.cargo_total(),
        crate::simulation::outposts::route_storage_capacity(session, data, outpost),
        outpost.crew.len(),
        crate::simulation::outposts::crew_capacity(outpost, data)
    )
}

pub fn route_policy_summary(
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
    let bonus = if compact {
        let chorus_ingots =
            crate::simulation::outposts::route_chorus_ingots(session, data, outpost);
        if chorus_ingots > 0 {
            Some(format!("Chorus +{chorus_ingots}/haul"))
        } else if !crate::simulation::outposts::wormsong_route_bonus(session, data, outpost)
            .is_empty()
        {
            Some("Wormsong".to_owned())
        } else {
            None
        }
    } else {
        crate::simulation::outposts::route_bonus_summary(session, data, outpost, false)
    };
    bonus.map_or(policy.clone(), |bonus| format!("{policy} · {bonus}"))
}
