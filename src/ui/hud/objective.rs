//! Persistent campaign objective copy and progress for the warren HUD.

use crate::data::GameData;
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
            let next = if !session.unlocked.contains("worm_transit") {
                "Next: keep forging ingots to unlock Worm Transit."
            } else if session.buildings_of("outpost").next().is_none() {
                "Next: build a Worm Outpost and send cargo through the awakened route."
            } else if session
                .outposts
                .iter()
                .any(|outpost| outpost.last_failure.is_some() && !outpost.active)
            {
                "Next: tap the failed Worm Outpost, then tap Activate route before sending cargo."
            } else if session
                .outposts
                .iter()
                .any(|outpost| outpost.last_failure.is_some())
            {
                "Next: tap the failed Worm Outpost, then try the cargo run again."
            } else if session.outposts.iter().any(|outpost| outpost.active) {
                "Next: send a cargo run through the active Worm Outpost."
            } else {
                "Next: activate the Worm Outpost, then send a cargo run."
            };
            return Self {
                title: "Campaign complete".to_owned(),
                progress: "The Colossal Worm is awake".to_owned(),
                next: next.to_owned(),
                ratio: 1.0,
                complete: true,
            };
        }

        if !session.won {
            let food_goal = data.balance.win_food_surplus;
            let food = session.economy.food.min(food_goal);
            let ore_goal = data.balance.win_ore_delivered;
            let ore = session.economy.ore_delivered_total.min(ore_goal) as f32;
            let next = if food < food_goal {
                "Next: keep Food Grid production above Upkeep."
            } else if ore < ore_goal as f32 {
                "Next: keep the Mine staffed and carriers hauling ore."
            } else {
                "Next: hold both reserves until the warren is secure."
            };
            return Self {
                title: "Secure the warren".to_owned(),
                progress: format!(
                    "Food {:.0}/{:.0}  ·  Ore {}/{}",
                    food, food_goal, ore as u32, ore_goal
                ),
                next: next.to_owned(),
                ratio: ((food / food_goal.max(1.0)) + (ore / ore_goal.max(1) as f32)) / 2.0,
                complete: false,
            };
        }

        if !session.factory_complete {
            let goal = data.balance.win2_ingots;
            let forged = session.economy.ingots_forged.min(goal);
            let next = if session.buildings_of("blacksmith").next().is_none() {
                "Next: tap Blacksmith in Build & Dig, then place it on open floor."
            } else if session.job_count(crate::state::creatures::Job::Smith) == 0 {
                "Next: tap + beside Smith in the Jobs panel."
            } else {
                "Next: keep the Blacksmith supplied while it forges ingots."
            };
            return Self {
                title: "Complete the Biofoundry".to_owned(),
                progress: format!("Ingots forged {forged}/{goal}"),
                next: next.to_owned(),
                ratio: forged as f32 / goal.max(1) as f32,
                complete: false,
            };
        }

        if session.buildings_of("worm_shrine").next().is_none() {
            return Self {
                title: "Awaken the Worm".to_owned(),
                progress: "Worm Shrine · ready to build".to_owned(),
                next: "Next: tap Shrine in Build & Dig, then place it on open floor.".to_owned(),
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

#[cfg(test)]
mod tests;
