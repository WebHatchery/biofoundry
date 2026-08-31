//! Coverage for the optional three-route Wormsong Chorus.

use super::active_outpost;
use crate::data::GameData;
use crate::simulation::outposts;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

fn add_complete_route(data: &GameData, session: &mut GameSession, route_pos: TilePos) {
    let mut crew = Vec::new();
    for (job, equipment) in [
        (Job::Carrier, "wormsong_harness"),
        (Job::Miner, "wormsong_drill"),
        (Job::Smith, "wormsong_smiths_hammer"),
        (Job::Guard, "wormsong_guard_blade"),
    ] {
        session.spawn_creature(data, "goblin", job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(equipment.to_owned());
        creature.remote_outpost = Some(route_pos);
        crew.push(creature.id);
    }
    let route = session
        .outposts
        .iter_mut()
        .find(|route| route.pos == route_pos)
        .expect("route exists");
    route.active = true;
    route.crew = crew;
}

#[test]
fn chorus_requires_three_complete_routes_and_pays_once() {
    let (data, mut session, first_pos) = active_outpost(189);
    session.outpost_circuit_claimed = true;
    session.economy.ingots_stock = 4;
    session.creatures.clear();

    let mut route_positions = vec![first_pos];
    for _ in 0..2 {
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable()
                    && session.can_place_building(*pos)
                    && !route_positions.contains(pos)
            })
            .map(|(pos, _)| pos)
            .expect("a route location");
        session.buildings.push(Building::new("outpost", pos));
        session.ensure_outpost(pos);
        route_positions.push(pos);
    }

    for route_pos in route_positions.iter().copied() {
        add_complete_route(&data, &mut session, route_pos);
    }

    assert_eq!(
        outposts::outpost_chorus_progress(&session, &data),
        (data.balance.outpost_chorus_route_goal, 3)
    );
    assert!(outposts::claim_outpost_chorus(&mut session, &data));
    assert_eq!(
        session.economy.ingots_stock,
        4 + data.balance.outpost_chorus_reward_ingots
    );
    assert!(!outposts::claim_outpost_chorus(&mut session, &data));
    assert_eq!(
        outposts::route_chorus_ingots(&session, &data, &session.outposts[0]),
        data.balance.outpost_chorus_ingots_per_haul
    );

    session.outposts[2].crew.pop();
    assert_eq!(outposts::outpost_chorus_progress(&session, &data).0, 2);
    assert_eq!(
        outposts::route_chorus_ingots(&session, &data, &session.outposts[0]),
        0
    );
}

#[test]
fn chorus_ingots_are_recorded_separately_from_signal_cache_ingots() {
    let (data, mut session, first_pos) = active_outpost(190);
    session.outpost_circuit_claimed = true;
    session.outpost_chorus_claimed = true;
    session.creatures.clear();

    let mut route_positions = vec![first_pos];
    for _ in 0..2 {
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable()
                    && session.can_place_building(*pos)
                    && !route_positions.contains(pos)
            })
            .map(|(pos, _)| pos)
            .expect("a route location");
        session.buildings.push(Building::new("outpost", pos));
        session.ensure_outpost(pos);
        route_positions.push(pos);
    }
    for route_pos in route_positions.iter().copied() {
        add_complete_route(&data, &mut session, route_pos);
    }

    let route_snapshot = {
        let route = &mut session.outposts[0];
        route.active = true;
        route.storage_upgraded = true;
        route.cargo.insert(Good::CookedFood, 4);
        route.clone()
    };
    let cycle = outposts::route_expedition_cycle_sec(&session, &data, &route_snapshot);
    session.outposts[0].expedition_progress = cycle;

    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(completed.len(), 1);
    assert_eq!(
        completed[0].ingots,
        data.balance.outpost_chorus_ingots_per_haul
    );
    assert_eq!(session.outposts[0].chorus_ingots, completed[0].ingots);
    assert_eq!(session.outposts[0].signal_cache_ingots, 0);
}
