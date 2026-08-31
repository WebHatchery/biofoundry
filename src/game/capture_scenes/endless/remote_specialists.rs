//! Capture scene for Wormsong tools functioning as a remote expedition crew.

use super::super::super::Game;
use crate::state::creatures::{Good, Job};
use crate::state::GameState;

pub(super) fn begin(game: &mut Game) {
    super::super::begin(game, "endless");
    game.notifications.clear();
    game.paused = true;
    if let GameState::Warren(session) = &mut game.state {
        session.outpost_charter_claimed = true;
        session.outpost_relay_claimed = true;
        session.outpost_convoy_claims = 1;
        session.outpost_muster_claims = 1;
        session.unlocked.insert("resonance_forging".to_owned());
        let Some(outpost_pos) = session.outposts.last().map(|route| route.pos) else {
            return;
        };
        session.creatures.clear();
        let specialist_gear = [
            (Job::Carrier, "wormsong_harness"),
            (Job::Miner, "wormsong_drill"),
            (Job::Smith, "wormsong_smiths_hammer"),
            (Job::Guard, "wormsong_guard_blade"),
        ];
        let mut crew = Vec::new();
        for (job, equipment) in specialist_gear {
            session.spawn_creature(&game.data, "goblin", job);
            let creature = session.creatures.last_mut().expect("specialist spawned");
            creature.equipment = Some(equipment.to_owned());
            creature.remote_outpost = Some(outpost_pos);
            creature.x = outpost_pos.x as f32 + 0.5;
            creature.y = outpost_pos.y as f32 + 0.5;
            creature.clear_task();
            crew.push(creature.id);
        }
        let route = session.outposts.last_mut().expect("capture route exists");
        route.active = true;
        route.storage_upgraded = true;
        route.crew_upgraded = true;
        route.resonator_upgraded = true;
        route.signal_cache_upgraded = true;
        route.crew = crew;
        route.cargo.clear();
        route.cargo.insert(Good::Ore, 6);
        route.cargo.insert(Good::Ingot, 2);
        route.cargo.insert(Good::CookedFood, 4);
        route.expeditions_completed = 8;
        route.ore_scouted = 112;
        route.signal_cache_ingots = 3;
        route.expedition_progress = 12.0;
        game.selected_building = Some(outpost_pos);
        game.focus_camera_on_tile(outpost_pos);
    }
}
