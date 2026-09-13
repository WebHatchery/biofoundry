//! Campaign guidance strings and route milestone next steps.

use super::*;

pub fn endless_forge_next_step(session: &GameSession, data: &GameData) -> String {
    if session.buildings_of("blacksmith").next().is_none() {
        if pending_build_site(session, "blacksmith") {
            return "Next: keep carriers delivering ore to the Blacksmith site.".to_owned();
        }
        return "Next: tap Blacksmith in Build & Dig, then tap open floor.".to_owned();
    }
    if !session.buildings_of("mine").any(|mine| mine.reserve > 0.0) {
        if pending_build_site(session, "mine") {
            return "Next: keep carriers delivering ore to the new Mine site.".to_owned();
        }
        return "Next: tap Mine in Build & Dig, then tap open floor for a new Mine.".to_owned();
    }
    if session.job_count(Job::Smith) == 0 {
        return format!(
            "Next: {}.",
            job_assignment_action_hint(
                session,
                data,
                Job::Smith,
                &[Job::Miner, Job::Carrier, Job::Cook, Job::Guard]
            )
        );
    }
    if session.job_count(Job::Carrier) == 0 {
        return format!(
            "Next: {}.",
            job_assignment_action_hint(
                session,
                data,
                Job::Carrier,
                &[Job::Miner, Job::Cook, Job::Guard, Job::Smith]
            )
        );
    }
    let transit_goal = data
        .unlocks
        .iter()
        .find(|unlock| unlock.id == "worm_transit")
        .map(|unlock| unlock.threshold)
        .unwrap_or(60);
    let forged = session.economy.ingots_forged.min(transit_goal);
    format!("Next: forge ingots to unlock Worm Transit ({forged}/{transit_goal}).")
}

pub fn pending_build_site(session: &GameSession, kind: &str) -> bool {
    session
        .build_sites
        .iter()
        .any(|site| site.kind == kind && site.remaining() > 0)
}

pub fn shrine_build_requirement(session: &GameSession, data: &GameData) -> (String, String) {
    if session.unlocked.contains("worm_shrine") {
        return (
            "Worm Shrine · ready to build".to_owned(),
            "Next: tap Shrine in Build & Dig, then tap open floor.".to_owned(),
        );
    }

    let Some(unlock) = data
        .unlocks
        .iter()
        .find(|unlock| unlock.id == "worm_shrine")
    else {
        return (
            "Worm Shrine · locked".to_owned(),
            "Next: meet the Worm Shrine prerequisite.".to_owned(),
        );
    };
    let current = crate::simulation::wildlife::counter_value(session, &unlock.counter);
    let requirement = crate::ui::hud::requirements::unlock_requirement(data, "worm_shrine")
        .unwrap_or_else(|| "meet the prerequisite".to_owned());
    (
        format!(
            "Worm Shrine · {requirement} ({current}/{})",
            unlock.threshold
        ),
        format!("Next: {requirement} to unlock the Worm Shrine."),
    )
}

pub fn security_handoff_action_hint(session: &GameSession, data: &GameData) -> String {
    job_assignment_action_hint(
        session,
        data,
        Job::Guard,
        &[Job::Miner, Job::Carrier, Job::Cook, Job::Smith],
    )
}

pub fn job_assignment_action_hint(
    session: &GameSession,
    data: &GameData,
    target: Job,
    sources: &[Job],
) -> String {
    if reassignable_job_count(session, data, Job::Idle) > 0 {
        return format!("tap + beside {} in Jobs", target.label());
    }
    sources
        .iter()
        .find(|job| reassignable_job_count(session, data, **job) > 0)
        .map(|job| {
            format!(
                "tap − beside {}, then + beside {} in Jobs",
                job.label(),
                target.label()
            )
        })
        .unwrap_or_else(|| {
            format!(
                "free a worker, then tap + beside {} in Jobs",
                target.label()
            )
        })
}

pub fn reassignable_job_count(session: &GameSession, data: &GameData, job: Job) -> usize {
    session
        .creatures
        .iter()
        .filter(|creature| {
            !creature.is_remote()
                && creature.job == job
                && data
                    .species
                    .get(&creature.species)
                    .is_some_and(|species| species.reassignable)
        })
        .count()
}

pub fn active_outpost_next_step(session: &GameSession, data: &GameData) -> String {
    let Some(outpost) = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .find(|outpost| outpost.cargo_total() > 0 && outpost.crew.is_empty())
        .or_else(|| {
            session
                .outposts
                .iter()
                .filter(|outpost| outpost.active)
                .find(|outpost| {
                    matches!(
                        crate::simulation::outposts::expedition_state_with_session(
                            session, data, outpost,
                        ),
                        crate::simulation::outposts::ExpeditionState::Paused
                    )
                })
        })
        .or_else(|| {
            session
                .outposts
                .iter()
                .filter(|outpost| outpost.active)
                .find(|outpost| {
                    matches!(
                        crate::simulation::outposts::expedition_state_with_session(
                            session, data, outpost,
                        ),
                        crate::simulation::outposts::ExpeditionState::NeedsFood { .. }
                    )
                })
        })
        .or_else(|| {
            session
                .outposts
                .iter()
                .filter(|outpost| outpost.active)
                .find(|outpost| outpost_has_loadable_payload(session, data, outpost))
        })
        .or_else(|| session.outposts.iter().find(|outpost| outpost.active))
    else {
        return "Next: tap the Worm Outpost, then tap Activate route.".to_owned();
    };
    let has_cargo = outpost.cargo_total() > 0;
    let has_crew = !outpost.crew.is_empty();
    let local_crew_ready = session.creatures.iter().any(|creature| {
        !creature.is_remote()
            && creature.carrying.is_none()
            && creature.tile() == session.stockpile_pos()
    });
    if !has_cargo && !has_crew && outpost.crew_dispatch_limit == Some(0) && local_crew_ready {
        return "Next: tap Crew per run on the Outpost, then load scouts from the warren."
            .to_owned();
    }
    if has_crew {
        match crate::simulation::outposts::expedition_state_with_session(session, data, outpost) {
            crate::simulation::outposts::ExpeditionState::NeedsFood { .. } => {
                if outpost.auto_resupply_food
                    && session.economy.food - data.balance.worm_feed_reserve >= 1.0
                {
                    return "Next: let Auto-resupply deliver food to the remote scouts.".to_owned();
                }
                if outpost.auto_load
                    && crate::simulation::outposts::has_loadable_payload(session, data, outpost)
                {
                    return "Next: let Auto-load refill the active Worm Outpost.".to_owned();
                }
                if session.economy.food - data.balance.worm_feed_reserve >= 1.0 {
                    return "Next: tap the active Worm Outpost, then load food for its expedition."
                        .to_owned();
                }
                return "Next: keep cooked Food above reserve, then load the Outpost expedition."
                    .to_owned();
            }
            crate::simulation::outposts::ExpeditionState::Paused => {
                return "Next: tap the active Worm Outpost, then Resume scouting.".to_owned();
            }
            crate::simulation::outposts::ExpeditionState::Scouting { .. } => {
                if let Some(guidance) = charter_guidance(session, data) {
                    return guidance;
                }
                return "Next: let the Outpost expedition finish, then return its ore while keeping the scouts remote."
                    .to_owned()
            }
            crate::simulation::outposts::ExpeditionState::HoldFull => {
                return "Next: tap the active Worm Outpost, then return its cargo while keeping the scouts remote."
                    .to_owned()
            }
            crate::simulation::outposts::ExpeditionState::Inactive
            | crate::simulation::outposts::ExpeditionState::NoCrew => {}
        }
    }
    if !has_cargo && !has_crew {
        if let Some(hint) = outpost_upgrade_next_step(session, data, outpost) {
            return hint;
        }
    }
    if outpost.auto_load
        && !outpost.expedition_paused
        && session.worm_transit.is_none()
        && crate::simulation::outposts::has_loadable_payload(session, data, outpost)
    {
        return "Next: let Auto-load dispatch the active Worm Outpost.".to_owned();
    }
    match (has_cargo, has_crew) {
        (true, true) => {
            "Next: tap the active Worm Outpost, then return its cargo while keeping the scouts remote."
        }
        (true, false) => "Next: tap the active Worm Outpost, then send its cargo to the shrine.",
        (false, true) => "Next: tap the active Worm Outpost, then send its crew to the shrine.",
        (false, false) if outpost_has_loadable_payload(session, data, outpost) => {
            "Next: tap the active Worm Outpost, then load it from the warren."
        }
        (false, false) => {
            "Next: keep cargo or crew ready at the warren, then load the active Worm Outpost."
        }
    }
    .to_owned()
}

pub fn charter_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    let goal = data.balance.outpost_charter_haul_goal;
    if goal == 0 {
        return None;
    }
    if session.outpost_charter_claimed {
        if session.outpost_archive_claims == 0
            && crate::simulation::outposts::outpost_archive_progress(session, data) == 0
        {
            return None;
        }
        if let Some(guidance) = relay_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = concord_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = circuit_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = chorus_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = encore_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = muster_contract_guidance(session, data) {
            return Some(guidance);
        }
        if let Some(guidance) = convoy_contract_guidance(session, data) {
            return Some(guidance);
        }
        return Some(format!(
            "Next: keep scouting · Archive {}/{} · +{} ingots.",
            crate::simulation::outposts::outpost_archive_progress(session, data),
            data.balance.outpost_archive_haul_goal,
            data.balance.outpost_archive_reward_ingots
        ));
    }
    let completed = crate::simulation::outposts::total_expeditions(session).min(goal);
    Some(format!(
        "Next: finish scouting · Charter {completed}/{goal} · +{} ingots.",
        data.balance.outpost_charter_reward_ingots
    ))
}

pub fn relay_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if session.outpost_archive_claims == 0
        || session.outpost_relay_claimed
        || data.balance.outpost_relay_route_goal == 0
        || data.balance.outpost_relay_haul_goal == 0
    {
        return None;
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_relay_progress(session);
    Some(format!(
        "Next: run the Worm Road Relay · {active_routes}/{} active routes · {completed_hauls}/{} hauls · +{} ingots.",
        data.balance.outpost_relay_route_goal,
        data.balance.outpost_relay_haul_goal,
        data.balance.outpost_relay_reward_ingots
    ))
}

pub fn convoy_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_relay_claimed
        || data.balance.outpost_convoy_route_goal == 0
        || data.balance.outpost_convoy_haul_goal == 0
    {
        return None;
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_convoy_progress(session, data);
    Some(format!(
        "Next: run the Worm Road Convoy · {active_routes}/{} active routes · {completed_hauls}/{} hauls · +{} ingots.",
        data.balance.outpost_convoy_route_goal,
        data.balance.outpost_convoy_haul_goal,
        data.balance.outpost_convoy_reward_ingots
    ))
}

pub fn muster_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_relay_claimed
        || session.outpost_convoy_claims == 0
        || data.balance.outpost_muster_route_goal == 0
        || data.balance.outpost_muster_haul_goal == 0
    {
        return None;
    }
    let (active_routes, completed_hauls) =
        crate::simulation::outposts::outpost_muster_progress(session, data);
    Some(format!(
        "Next: run the Worm Road Muster · {active_routes}/{} active routes · {completed_hauls}/{} hauls · +{} ingots.",
        data.balance.outpost_muster_route_goal,
        data.balance.outpost_muster_haul_goal,
        data.balance.outpost_muster_reward_ingots
    ))
}

pub fn concord_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if session.outpost_muster_claims == 0
        || session.outpost_concord_claimed
        || data.balance.outpost_concord_role_goal == 0
    {
        return None;
    }
    let (roles, active_routes) =
        crate::simulation::outposts::outpost_concord_progress(session, data);
    Some(format!(
        "Next: station a Wormsong carrier, miner, smith, and guard on active routes · Concord {roles}/{} roles · {active_routes} active routes · +{} ingots.",
        data.balance.outpost_concord_role_goal,
        data.balance.outpost_concord_reward_ingots
    ))
}

pub fn circuit_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_concord_claimed
        || session.outpost_circuit_claimed
        || data.balance.outpost_circuit_route_goal < 2
    {
        return None;
    }
    let (complete_routes, active_routes) =
        crate::simulation::outposts::outpost_circuit_progress(session, data);
    Some(format!(
        "Next: keep {} active routes carrying complete Wormsong crews for the Wormsong Circuit · {complete_routes}/{} complete routes · {active_routes} active routes · +{} ingots.",
        data.balance.outpost_circuit_route_goal,
        data.balance.outpost_circuit_route_goal,
        data.balance.outpost_circuit_reward_ingots
    ))
}

pub fn encore_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_circuit_claimed || data.balance.outpost_encore_haul_goal == 0 {
        return None;
    }
    let progress = crate::simulation::outposts::outpost_encore_progress(session, data);
    Some(format!(
        "Next: keep a complete Wormsong crew scouting · Encore {progress}/{} boosted hauls · +{} ingots.",
        data.balance.outpost_encore_haul_goal,
        data.balance.outpost_encore_reward_ingots
    ))
}

pub fn chorus_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
    if !session.outpost_circuit_claimed
        || session.outpost_chorus_claimed
        || data.balance.outpost_chorus_route_goal == 0
    {
        return None;
    }
    let (complete_routes, active_routes) =
        crate::simulation::outposts::outpost_chorus_progress(session, data);
    Some(format!(
        "Next: keep {} active routes carrying complete Wormsong crews for the Wormsong Chorus · {complete_routes}/{} complete routes · {active_routes} active routes · +{} ingots and +{} ingot/haul.",
        data.balance.outpost_chorus_route_goal,
        data.balance.outpost_chorus_route_goal,
        data.balance.outpost_chorus_reward_ingots,
        data.balance.outpost_chorus_ingots_per_haul
    ))
}

pub fn outpost_upgrade_next_step(
    session: &GameSession,
    data: &GameData,
    outpost: &crate::state::outposts::Outpost,
) -> Option<String> {
    let available = session.economy.ingots_stock;
    if !outpost.storage_upgraded && available >= data.balance.outpost_upgrade_ingots {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Expand hold for {} ingots.",
            data.balance.outpost_upgrade_ingots
        ));
    }
    if outpost.storage_upgraded
        && !outpost.crew_upgraded
        && available >= data.balance.outpost_crew_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Expand camp for {} ingots.",
            data.balance.outpost_crew_upgrade_ingots
        ));
    }
    if outpost.storage_upgraded
        && outpost.crew_upgraded
        && !outpost.survey_upgraded
        && available >= data.balance.outpost_survey_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Install survey for {} ingots.",
            data.balance.outpost_survey_upgrade_ingots
        ));
    }
    if outpost.survey_upgraded
        && !outpost.resonator_upgraded
        && available >= data.balance.outpost_resonator_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Tune beacon for {} ingots.",
            data.balance.outpost_resonator_upgrade_ingots
        ));
    }
    if outpost.resonator_upgraded
        && session.outpost_charter_claimed
        && !outpost.deep_survey_upgraded
        && available >= data.balance.outpost_deep_survey_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Calibrate deep survey for {} ingots.",
            data.balance.outpost_deep_survey_upgrade_ingots
        ));
    }
    if outpost.deep_survey_upgraded
        && session.outpost_relay_claimed
        && !outpost.signal_cache_upgraded
        && available >= data.balance.outpost_signal_cache_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Install signal cache for {} ingots.",
            data.balance.outpost_signal_cache_upgrade_ingots
        ));
    }
    if outpost.signal_cache_upgraded
        && session.outpost_convoy_claims > 0
        && !outpost.waypoint_upgraded
        && available >= data.balance.outpost_waypoint_upgrade_ingots
    {
        return Some(format!(
            "Next: tap the active Worm Outpost, then Install Worm Road waypoint for {} ingots.",
            data.balance.outpost_waypoint_upgrade_ingots
        ));
    }
    None
}

pub fn outpost_has_loadable_payload(
    session: &GameSession,
    data: &GameData,
    outpost: &crate::state::outposts::Outpost,
) -> bool {
    let room = crate::simulation::outposts::route_storage_capacity(session, data, outpost)
        .saturating_sub(outpost.cargo_total());
    let food_ready = session.economy.food - data.balance.worm_feed_reserve >= 1.0;
    let cargo_ready = room > 0
        && (session.economy.ore_stock > 0 || session.economy.ingots_stock > 0 || food_ready);
    let crew_ready = outpost.crew.len()
        < crate::simulation::outposts::crew_capacity(outpost, data) as usize
        && session.creatures.iter().any(|creature| {
            !creature.is_remote()
                && creature.carrying.is_none()
                && creature.tile() == session.stockpile_pos()
        });
    cargo_ready || crew_ready
}
