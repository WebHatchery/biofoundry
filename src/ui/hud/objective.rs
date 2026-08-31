//! Persistent campaign objective copy and progress for the warren HUD.

use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::TransitDirection;
use crate::state::GameSession;

/// The single campaign milestone the player should read first.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CampaignObjective {
    pub title: String,
    pub progress: String,
    pub next: String,
    pub ratio: f32,
    pub complete: bool,
}

impl CampaignObjective {
    /// Derive the current milestone without mutating the simulation.
    pub fn current(session: &GameSession, data: &GameData) -> Self {
        if session.worm_awake {
            let next: String = if !session.unlocked.contains("worm_transit") {
                endless_forge_next_step(session, data)
            } else if let Some(transit) = session.worm_transit.as_ref() {
                match transit.direction {
                    TransitDirection::ToOutpost => {
                        "Next: wait for the worm to reach the outpost, then inspect its cargo."
                            .to_owned()
                    }
                    TransitDirection::ToShrine => {
                        "Next: wait for the worm to reach the shrine, then plan the next run."
                            .to_owned()
                    }
                }
            } else if session.buildings_of("outpost").next().is_none() {
                "Next: build a Worm Outpost and send cargo through the awakened route.".to_owned()
            } else if session.outposts.iter().any(|outpost| {
                !outpost.active && (outpost.cargo_total() > 0 || !outpost.crew.is_empty())
            }) {
                "Next: tap the loaded Worm Outpost, then tap Activate route to return its payload."
                    .to_owned()
            } else if session
                .outposts
                .iter()
                .any(|outpost| outpost.last_failure.is_some() && !outpost.active)
            {
                "Next: tap the failed Worm Outpost, then tap Activate route before sending cargo."
                    .to_owned()
            } else if session
                .outposts
                .iter()
                .any(|outpost| outpost.last_failure.is_some())
            {
                "Next: tap the failed Worm Outpost, then try the cargo run again.".to_owned()
            } else if session.outposts.iter().any(|outpost| outpost.active) {
                active_outpost_next_step(session, data)
            } else {
                "Next: activate the Worm Outpost, then send a cargo run.".to_owned()
            };
            let scouting_hauls = crate::simulation::outposts::total_expeditions(session);
            let route_upgrades: u32 = session
                .outposts
                .iter()
                .map(|outpost| {
                    [
                        outpost.storage_upgraded,
                        outpost.crew_upgraded,
                        outpost.survey_upgraded,
                        outpost.resonator_upgraded,
                        outpost.deep_survey_upgraded,
                        outpost.signal_cache_upgraded,
                    ]
                    .into_iter()
                    .filter(|installed| *installed)
                    .count() as u32
                })
                .sum();
            return Self {
                title: "Campaign complete".to_owned(),
                progress: format!(
                    "Worm awake · Runs {} · Hauls {scouting_hauls} · Upgrades {route_upgrades}",
                    session.progress.courier_deliveries,
                ),
                next,
                ratio: 1.0,
                complete: true,
            };
        }

        if !session.won {
            let food_goal = data.balance.win_food_surplus;
            let food = session.economy.food.min(food_goal);
            let ore_goal = data.balance.win_ore_delivered;
            let ore = session.economy.ore_delivered_total.min(ore_goal) as f32;
            let next: String = if food < food_goal {
                if pending_build_site(session, "farm") {
                    format!(
                        "Next: keep carriers delivering ore to the Farm site; if Food Grid keeps falling, {}.",
                        job_assignment_action_hint(
                            session,
                            data,
                            Job::Carrier,
                            &[Job::Miner, Job::Cook, Job::Smith, Job::Guard]
                        )
                    )
                } else {
                    "Next: keep Food Grid production above Upkeep.".to_owned()
                }
            } else if ore < ore_goal as f32 {
                "Next: keep the Mine staffed and carriers hauling ore.".to_owned()
            } else {
                "Next: hold both reserves until the warren is secure.".to_owned()
            };
            return Self {
                title: "Secure the warren".to_owned(),
                progress: format!(
                    "Food {:.0}/{:.0}  ·  Ore {}/{}",
                    food, food_goal, ore as u32, ore_goal
                ),
                next,
                ratio: ((food / food_goal.max(1.0)) + (ore / ore_goal.max(1) as f32)) / 2.0,
                complete: false,
            };
        }

        if session.job_count(Job::Guard) == 0 {
            let food_goal = data.balance.win_food_surplus;
            let ore_goal = data.balance.win_ore_delivered;
            return Self {
                title: "Finish the security handoff".to_owned(),
                progress: format!(
                    "Food {:.0}/{:.0}  ·  Ore {}/{}  ·  Guard 0/1",
                    session.economy.food.min(food_goal),
                    food_goal,
                    session.economy.ore_delivered_total.min(ore_goal),
                    ore_goal
                ),
                next: format!("Next: {}.", security_handoff_action_hint(session, data)),
                ratio: 2.0 / 3.0,
                complete: false,
            };
        }

        if !session.factory_complete {
            let goal = data.balance.win2_ingots;
            let forged = session.economy.ingots_forged.min(goal);
            let next: String = if session.buildings_of("blacksmith").next().is_none() {
                "Next: tap Blacksmith in Build & Dig, then place it on open floor.".to_owned()
            } else if session.job_count(crate::state::creatures::Job::Smith) == 0 {
                format!(
                    "Next: {}.",
                    job_assignment_action_hint(
                        session,
                        data,
                        Job::Smith,
                        &[Job::Miner, Job::Carrier, Job::Cook, Job::Guard,]
                    )
                )
            } else {
                "Next: keep the Blacksmith supplied while it forges ingots.".to_owned()
            };
            return Self {
                title: "Complete the Biofoundry".to_owned(),
                progress: format!("Ingots forged {forged}/{goal}"),
                next,
                ratio: forged as f32 / goal.max(1) as f32,
                complete: false,
            };
        }

        if session.buildings_of("worm_shrine").next().is_none() {
            let (progress, next) = shrine_build_requirement(session, data);
            return Self {
                title: "Awaken the Worm".to_owned(),
                progress,
                next,
                ratio: 0.0,
                complete: false,
            };
        }

        let food_goal = data.balance.worm_awaken_at;
        let ingot_goal = data.balance.worm_awaken_ingots;
        let food_ratio = session.worm_fed / food_goal.max(1.0);
        let ingot_ratio = session.worm_ingots_fed as f32 / ingot_goal.max(1) as f32;
        let next = if session.worm_feeding_paused {
            "Next: tap Resume offerings in the Worm Shrine panel."
        } else if session.worm_fed < food_goal {
            "Next: keep Food Grid production above Upkeep for the offerings."
        } else if session.worm_ingots_fed < ingot_goal {
            "Next: keep forging ingots for the remaining offerings."
        } else {
            "Next: keep the Shrine running until the worm wakes."
        };
        Self {
            title: "Awaken the Worm".to_owned(),
            progress: format!(
                "Offerings  ·  Food {:.0}/{:.0}  ·  Ingots {}/{}",
                session.worm_fed, food_goal, session.worm_ingots_fed, ingot_goal
            ),
            next: next.to_owned(),
            ratio: (food_ratio + ingot_ratio) / 2.0,
            complete: false,
        }
    }
}

fn endless_forge_next_step(session: &GameSession, data: &GameData) -> String {
    if session.buildings_of("blacksmith").next().is_none() {
        if pending_build_site(session, "blacksmith") {
            return "Next: keep carriers delivering ore to the Blacksmith site.".to_owned();
        }
        return "Next: tap Blacksmith in Build & Dig, then place it on open floor.".to_owned();
    }
    if !session.buildings_of("mine").any(|mine| mine.reserve > 0.0) {
        if pending_build_site(session, "mine") {
            return "Next: keep carriers delivering ore to the new Mine site.".to_owned();
        }
        return "Next: tap Mine in Build & Dig, then place a new Mine on open floor.".to_owned();
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

fn pending_build_site(session: &GameSession, kind: &str) -> bool {
    session
        .build_sites
        .iter()
        .any(|site| site.kind == kind && site.remaining() > 0)
}

fn shrine_build_requirement(session: &GameSession, data: &GameData) -> (String, String) {
    if session.unlocked.contains("worm_shrine") {
        return (
            "Worm Shrine · ready to build".to_owned(),
            "Next: tap Shrine in Build & Dig, then place it on open floor.".to_owned(),
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
    let requirement = super::requirements::unlock_requirement(data, "worm_shrine")
        .unwrap_or_else(|| "meet the prerequisite".to_owned());
    (
        format!(
            "Worm Shrine · {requirement} ({current}/{})",
            unlock.threshold
        ),
        format!("Next: {requirement} to unlock the Worm Shrine."),
    )
}

pub(super) fn security_handoff_action_hint(session: &GameSession, data: &GameData) -> String {
    job_assignment_action_hint(
        session,
        data,
        Job::Guard,
        &[Job::Miner, Job::Carrier, Job::Cook, Job::Smith],
    )
}

pub(super) fn job_assignment_action_hint(
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

fn reassignable_job_count(session: &GameSession, data: &GameData, job: Job) -> usize {
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

fn active_outpost_next_step(session: &GameSession, data: &GameData) -> String {
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
                        crate::simulation::outposts::expedition_state(outpost, data),
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
                        crate::simulation::outposts::expedition_state(outpost, data),
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
        return "Next: activate the Worm Outpost, then send a cargo run.".to_owned();
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
        match crate::simulation::outposts::expedition_state(outpost, data) {
            crate::simulation::outposts::ExpeditionState::NeedsFood { .. } => {
                if outpost.auto_resupply_food
                    && session.economy.food - data.balance.worm_feed_reserve >= 1.0
                {
                    return "Next: let Auto-resupply deliver food to the remote scouts.".to_owned();
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

fn charter_guidance(session: &GameSession, data: &GameData) -> Option<String> {
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

fn relay_contract_guidance(session: &GameSession, data: &GameData) -> Option<String> {
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

fn outpost_upgrade_next_step(
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
    None
}

fn outpost_has_loadable_payload(
    session: &GameSession,
    data: &GameData,
    outpost: &crate::state::outposts::Outpost,
) -> bool {
    let room = crate::simulation::outposts::storage_capacity(outpost, data)
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

#[cfg(test)]
mod tests;
