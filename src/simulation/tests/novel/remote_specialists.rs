//! Coverage for Wormsong equipment carried into remote expeditions.

use super::active_outpost;
use crate::simulation::outposts;
use crate::state::creatures::{Good, Job};

#[test]
fn stationed_wormsong_specialists_improve_a_real_expedition() {
    let (data, mut session, outpost_pos) = active_outpost(183);
    session.unlocked.insert("resonance_forging".to_owned());
    session.creatures.clear();

    let specialist_gear = [
        (Job::Carrier, "wormsong_harness"),
        (Job::Miner, "wormsong_drill"),
        (Job::Smith, "wormsong_smiths_hammer"),
        (Job::Guard, "wormsong_guard_blade"),
    ];
    let mut crew = Vec::new();
    for (job, equipment) in specialist_gear {
        session.spawn_creature(&data, "goblin", job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(equipment.to_owned());
        creature.remote_outpost = Some(outpost_pos);
        crew.push(creature.id);
    }

    {
        let route = &mut session.outposts[0];
        route.storage_upgraded = true;
        route.signal_cache_upgraded = true;
        route.crew = crew;
        route.cargo.insert(Good::CookedFood, 4);
    }
    let route_snapshot = session.outposts[0].clone();
    let cycle = outposts::route_expedition_cycle_sec(&session, &data, &route_snapshot);
    session.outposts[0].expedition_progress = cycle;

    let bonus = outposts::wormsong_route_bonus(&session, &data, &route_snapshot);
    assert_eq!(bonus.storage_slots, 2);
    assert_eq!(bonus.ore, 2);
    assert_eq!(bonus.ingots, 1);
    assert_eq!(bonus.cycle_reduction, 4.0);
    assert_eq!(
        outposts::route_storage_capacity(&session, &data, &route_snapshot),
        22
    );
    assert_eq!(
        outposts::route_expedition_cycle_sec(&session, &data, &route_snapshot),
        26.0
    );
    assert_eq!(
        outposts::route_expedition_ore(&session, &data, &route_snapshot),
        14
    );
    assert_eq!(
        outposts::route_signal_cache_ingots(&session, &data, &route_snapshot),
        2
    );
    assert_eq!(
        outposts::route_bonus_summary(&session, &data, &route_snapshot, false).as_deref(),
        Some("Wormsong crew · hold +2 · ore +2 · cache +1 · cycle -4s")
    );

    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].ore, 14);
    assert_eq!(completed[0].ingots, 2);
    assert_eq!(completed[0].food_spent, 4);
    assert_eq!(session.outposts[0].ore_scouted, 14);
    assert_eq!(session.outpost_concord_hauls, 0);
}

#[test]
fn route_bonus_is_empty_when_stationed_crew_has_legacy_tools() {
    let (data, mut session, outpost_pos) = active_outpost(184);
    let carrier = session
        .creatures
        .iter_mut()
        .find(|creature| creature.job == Job::Carrier)
        .expect("active route fixture has a carrier");
    carrier.equipment = Some("wormbone_hauling_frame".to_owned());
    carrier.remote_outpost = Some(outpost_pos);
    session.outposts[0].crew = vec![carrier.id];

    let bonus = outposts::wormsong_route_bonus(&session, &data, &session.outposts[0]);

    assert!(bonus.is_empty());
    assert_eq!(
        outposts::route_storage_capacity(&session, &data, &session.outposts[0]),
        outposts::storage_capacity(&session.outposts[0], &data)
    );
}

#[test]
fn wormsong_concord_requires_each_role_on_an_active_route() {
    let (data, mut session, outpost_pos) = active_outpost(185);
    session.outpost_muster_claims = 1;
    session.economy.ingots_stock = 3;
    session.creatures.clear();

    let specialist_gear = [
        (Job::Carrier, "wormsong_harness"),
        (Job::Miner, "wormsong_drill"),
        (Job::Smith, "wormsong_smiths_hammer"),
        (Job::Guard, "wormsong_guard_blade"),
    ];
    let mut crew = Vec::new();
    for (job, equipment) in specialist_gear {
        session.spawn_creature(&data, "goblin", job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(equipment.to_owned());
        creature.remote_outpost = Some(outpost_pos);
        crew.push(creature.id);
    }
    session.outposts[0].crew = crew[..3].to_vec();

    assert_eq!(outposts::outpost_concord_progress(&session, &data), (3, 1));
    assert!(!outposts::claim_outpost_concord(&mut session, &data));
    assert!(!session.outpost_concord_claimed);

    session.outposts[0].crew.push(crew[3]);
    assert_eq!(outposts::outpost_concord_progress(&session, &data), (4, 1));
    assert!(outposts::claim_outpost_concord(&mut session, &data));
    assert!(session.outpost_concord_claimed);
    assert_eq!(
        session.economy.ingots_stock,
        3 + data.balance.outpost_concord_reward_ingots
    );
    assert!(!outposts::claim_outpost_concord(&mut session, &data));

    session.outpost_concord_claimed = false;
    session.outposts[0].active = false;
    assert_eq!(outposts::outpost_concord_progress(&session, &data), (0, 0));
}

#[test]
fn claimed_concord_gives_a_complete_route_an_ongoing_song_bonus() {
    let (data, mut session, outpost_pos) = active_outpost(186);
    session.outpost_concord_claimed = true;
    session.creatures.clear();

    let specialist_gear = [
        (Job::Carrier, "wormsong_harness"),
        (Job::Miner, "wormsong_drill"),
        (Job::Smith, "wormsong_smiths_hammer"),
        (Job::Guard, "wormsong_guard_blade"),
    ];
    let mut crew = Vec::new();
    for (job, equipment) in specialist_gear {
        session.spawn_creature(&data, "goblin", job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(equipment.to_owned());
        creature.remote_outpost = Some(outpost_pos);
        crew.push(creature.id);
    }
    {
        let route = &mut session.outposts[0];
        route.crew = crew;
        route.storage_upgraded = true;
        route.cargo.insert(Good::CookedFood, 4);
    }
    let route_snapshot = session.outposts[0].clone();

    assert_eq!(
        outposts::route_expedition_cycle_sec(&session, &data, &route_snapshot),
        24.0
    );
    assert_eq!(
        outposts::route_expedition_ore(&session, &data, &route_snapshot),
        15
    );
    assert_eq!(
        outposts::route_bonus_summary(&session, &data, &route_snapshot, true).as_deref(),
        Some("Wormsong · hold+2 ore+2 cache+1ready cycle-4s concord+1ore/-2s")
    );

    session.outposts[0].expedition_progress = 24.0;
    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].ore, 15);
    assert_eq!(session.outpost_concord_hauls, 1);
    session.outposts[0].crew.clear();
    assert_eq!(
        outposts::route_expedition_ore(&session, &data, &session.outposts[0]),
        0
    );
}

#[test]
fn wormsong_circuit_requires_two_complete_route_crews_and_pays_once() {
    let (data, mut session, first_pos) = active_outpost(187);
    session.outpost_concord_claimed = true;
    session.economy.ingots_stock = 5;
    session.creatures.clear();
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| {
            tile.walkable() && session.can_place_building(*pos) && *pos != first_pos
        })
        .map(|(pos, _)| pos)
        .expect("a second route location");
    session
        .buildings
        .push(crate::state::structures::Building::new(
            "outpost", second_pos,
        ));
    session.ensure_outpost(second_pos);

    for route_pos in [first_pos, second_pos] {
        let specialist_gear = [
            (Job::Carrier, "wormsong_harness"),
            (Job::Miner, "wormsong_drill"),
            (Job::Smith, "wormsong_smiths_hammer"),
            (Job::Guard, "wormsong_guard_blade"),
        ];
        let mut crew = Vec::new();
        for (job, equipment) in specialist_gear {
            session.spawn_creature(&data, "goblin", job);
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

    assert_eq!(outposts::outpost_circuit_progress(&session, &data), (2, 2));
    assert!(outposts::claim_outpost_circuit(&mut session, &data));
    assert_eq!(
        session.economy.ingots_stock,
        5 + data.balance.outpost_circuit_reward_ingots
    );
    assert!(!outposts::claim_outpost_circuit(&mut session, &data));

    session.outposts[1].crew.pop();
    assert_eq!(outposts::outpost_circuit_progress(&session, &data), (1, 2));
}

#[test]
fn wormsong_encore_pays_for_boosted_hauls_and_preserves_partial_progress() {
    let (data, mut session, _) = active_outpost(188);
    session.outpost_circuit_claimed = true;
    session.outpost_concord_hauls = data.balance.outpost_encore_haul_goal + 1;
    session.economy.ingots_stock = 6;

    assert_eq!(
        outposts::outpost_encore_progress(&session, &data),
        data.balance.outpost_encore_haul_goal
    );
    assert_eq!(outposts::claim_outpost_encore(&mut session, &data), 1);
    assert_eq!(session.outpost_encore_claims, 1);
    assert_eq!(
        session.economy.ingots_stock,
        6 + data.balance.outpost_encore_reward_ingots
    );
    assert_eq!(outposts::outpost_encore_progress(&session, &data), 1);

    session.outpost_concord_hauls += data.balance.outpost_encore_haul_goal - 1;
    assert_eq!(outposts::claim_outpost_encore(&mut session, &data), 1);
    assert_eq!(session.outpost_encore_claims, 2);
}
