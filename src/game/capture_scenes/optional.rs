//! Capture scene for optional support specialists and their active benefits.

use crate::game::Game;
use crate::state::creatures::{Good, Job, Task};
use crate::state::structures::Building;
use crate::state::{GameState, StateTransition};

pub(super) fn begin(game: &mut Game) {
    game.transition(StateTransition::StartWarren);
    if let GameState::Warren(session) = &mut game.state {
        // Show post-campaign support choices without a modal so their
        // benefits remain reviewable in the canonical HUD capture.
        session.tutorial_dismissed = true;
        session.economy.food = 300.0;
        session.economy.ore_stock = 50;
        session.won = true;
        session.victory_shown = true;
        session.factory_complete = true;
        session.factory_shown = true;
        session.creatures[0].job = Job::Guard;
        session.unlocked.insert("slime_janitor".to_owned());
        session.unlocked.insert("bat_courier".to_owned());
        // Keep one local Beetle and Salamander in the roster so the
        // optional capture proves the active benefit labels, not only
        // the pre-recruitment choices.
        session.spawn_creature(&game.data, "beetle", Job::Carrier);
        let smelter_spot = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, _)| session.can_place_building(*pos))
            .map(|(pos, _)| pos);
        if let Some(spot) = smelter_spot {
            let mut smelter = Building::new("smelter", spot);
            smelter.add_stock(Good::Ore, 1.0);
            smelter.add_stock(Good::Charcoal, 1.0);
            session.buildings.push(smelter);
            session.spawn_creature(&game.data, "salamander", Job::Smelter);
            if let Some(salamander) = session.creatures.last_mut() {
                salamander.x = spot.x as f32 + 0.5;
                salamander.y = spot.y as f32 + 0.5;
                salamander.task = Task::Smelting {
                    den: spot,
                    remaining: game.data.balance.smelt_batch_time_sec,
                };
            }
        }
    }
}
