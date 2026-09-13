//! Persistent campaign objective copy and progress for the warren HUD.

use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::TransitDirection;
use crate::state::GameSession;

/// The single campaign milestone the player should read first.
#[derive(Debug, Clone, PartialEq)]
pub struct CampaignObjective {
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
                "Next: tap Outpost in Build & Dig, tap open floor, then tap Activate route."
                    .to_owned()
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
                "Next: tap the Worm Outpost, then tap Activate route.".to_owned()
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
                        outpost.waypoint_upgraded,
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
                "Next: tap Blacksmith in Build & Dig, then tap open floor.".to_owned()
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

mod guidance;
pub use guidance::{
    active_outpost_next_step, charter_guidance, chorus_contract_guidance,
    circuit_contract_guidance, concord_contract_guidance, convoy_contract_guidance,
    encore_contract_guidance, endless_forge_next_step, job_assignment_action_hint,
    muster_contract_guidance, outpost_has_loadable_payload, outpost_upgrade_next_step,
    pending_build_site, reassignable_job_count, relay_contract_guidance,
    security_handoff_action_hint, shrine_build_requirement,
};
