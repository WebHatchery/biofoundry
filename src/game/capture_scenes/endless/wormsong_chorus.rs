//! Capture scenes for the optional three-route Wormsong Chorus.

use super::super::super::{format_expedition_completion, Game};
use super::wormsong_circuit;
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;
use crate::state::GameState;

const SPECIALIST_GEAR: [(Job, &str); 4] = [
    (Job::Carrier, "wormsong_harness"),
    (Job::Miner, "wormsong_drill"),
    (Job::Smith, "wormsong_smiths_hammer"),
    (Job::Guard, "wormsong_guard_blade"),
];

pub(super) fn begin(game: &mut Game) {
    wormsong_circuit::begin(game);
    game.notifications.clear();
    game.routes_open = true;
    game.paused = true;
    if let GameState::Warren(session) = &mut game.state {
        let Some(first_pos) = session.outposts.first().map(|route| route.pos) else {
            return;
        };
        let third_pos = session
            .world
            .tiles
            .iter_with_pos()
            .filter(|(pos, _)| session.can_place_building(*pos))
            .map(|(pos, _)| pos)
            .max_by_key(|pos| (pos.manhattan_distance(&session.spawn_tile()), pos.x, pos.y));
        let Some(third_pos) = third_pos else {
            return;
        };
        session.buildings.push(Building::new("outpost", third_pos));
        session.ensure_outpost(third_pos);
        let third = session.outposts.last_mut().expect("third route exists");
        third.active = true;
        third.storage_upgraded = true;
        third.crew_upgraded = true;
        third.resonator_upgraded = true;
        third.signal_cache_upgraded = true;
        third.cargo.insert(Good::CookedFood, 4);
        session.outpost_circuit_claimed = true;
        session.outpost_chorus_claimed = false;
        session.economy.ingots_stock = 0;
        game.selected_building = Some(first_pos);
        game.focus_camera_on_tile(first_pos);
    }
}

fn complete_third_route(game: &mut Game) {
    if let GameState::Warren(session) = &mut game.state {
        let Some(route_pos) = session.outposts.last().map(|route| route.pos) else {
            return;
        };
        let mut crew = Vec::new();
        for (job, equipment) in SPECIALIST_GEAR {
            session.spawn_creature(&game.data, "goblin", job);
            let creature = session.creatures.last_mut().expect("specialist spawned");
            creature.equipment = Some(equipment.to_owned());
            creature.remote_outpost = Some(route_pos);
            creature.x = route_pos.x as f32 + 0.5;
            creature.y = route_pos.y as f32 + 0.5;
            creature.clear_task();
            crew.push(creature.id);
        }
        let route = session.outposts.last_mut().expect("third route exists");
        route.crew = crew;
        route.cargo.clear();
        route.cargo.insert(Good::CookedFood, 4);
    }
}

pub(super) fn award(game: &mut Game) {
    begin(game);
    complete_third_route(game);
    if let GameState::Warren(session) = &mut game.state {
        if simulation::outposts::claim_outpost_chorus(session, &game.data) {
            game.notifications.success(format!(
                "Chorus · +{} ingots · 3 routes linked.",
                game.data.balance.outpost_chorus_reward_ingots
            ));
        }
    }
}

pub(super) fn haul(game: &mut Game) {
    award(game);
    game.routes_open = false;
    if let GameState::Warren(session) = &mut game.state {
        let Some(route_snapshot) = session.outposts.last().cloned() else {
            return;
        };
        let cycle =
            simulation::outposts::route_expedition_cycle_sec(session, &game.data, &route_snapshot);
        if let Some(route) = session.outposts.last_mut() {
            route.expedition_progress = cycle;
        }
        let completed = simulation::outposts::tick_expeditions(session, &game.data, 0.0);
        for completion in completed {
            game.notifications
                .info(format_expedition_completion(completion));
        }
        game.selected_building = Some(route_snapshot.pos);
        game.focus_camera_on_tile(route_snapshot.pos);
    }
}
