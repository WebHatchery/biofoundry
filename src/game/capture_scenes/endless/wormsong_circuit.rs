//! Capture scene for the two-route Wormsong Circuit.

use super::super::super::Game;
use super::remote_specialists;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;
use crate::state::GameState;
pub(super) fn begin(game: &mut Game) {
    remote_specialists::begin(game);
    game.routes_open = true;
    if let GameState::Warren(session) = &mut game.state {
        let Some(first_pos) = session.outposts.first().map(|route| route.pos) else {
            return;
        };
        let second_pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable() && session.can_place_building(*pos) && *pos != first_pos
            })
            .map(|(pos, _)| pos);
        let Some(second_pos) = second_pos else {
            return;
        };
        session.buildings.push(Building::new("outpost", second_pos));
        session.ensure_outpost(second_pos);
        session.creatures.clear();
        for route in &mut session.outposts {
            route.active = true;
            route.crew.clear();
            route.storage_upgraded = true;
            route.crew_upgraded = true;
            route.resonator_upgraded = true;
            route.signal_cache_upgraded = true;
            route.expedition_paused = false;
            route.cargo.clear();
            route.cargo.insert(Good::CookedFood, 4);
            route.expedition_progress = 8.0;
        }
        let specialist_gear = [
            (Job::Carrier, "wormsong_harness"),
            (Job::Miner, "wormsong_drill"),
            (Job::Smith, "wormsong_smiths_hammer"),
            (Job::Guard, "wormsong_guard_blade"),
        ];
        for route_pos in [first_pos, second_pos] {
            let mut crew = Vec::new();
            for (job, equipment) in specialist_gear {
                session.spawn_creature(&game.data, "goblin", job);
                let creature = session.creatures.last_mut().expect("specialist spawned");
                creature.equipment = Some(equipment.to_owned());
                creature.remote_outpost = Some(route_pos);
                creature.x = route_pos.x as f32 + 0.5;
                creature.y = route_pos.y as f32 + 0.5;
                creature.clear_task();
                crew.push(creature.id);
            }
            let route = session
                .outposts
                .iter_mut()
                .find(|route| route.pos == route_pos)
                .expect("circuit route exists");
            route.crew = crew;
            route.cargo.insert(Good::Ore, 6);
            route.cargo.insert(Good::Ingot, 2);
        }
        session.outpost_concord_claimed = true;
        session.outpost_circuit_claimed = false;
        session.economy.ingots_stock = 0;
        game.selected_building = Some(first_pos);
        game.focus_camera_on_tile(first_pos);
    }
}
