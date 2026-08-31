//! Coverage for the Worm Road Charter milestone.

use crate::simulation::{self, outposts};
use crate::state::creatures::{Good, Job, Task};
use crate::state::structures::Building;

#[test]
fn charter_awards_ingots_after_the_configured_number_of_hauls() {
    let (data, mut session, _) = super::novel::active_outpost(164);
    let goal = data.balance.outpost_charter_haul_goal;
    let drill = data
        .equipment_def("wormbone_drill")
        .expect("Wormbone Drill data");
    assert!(!session.equipment_unlocked(drill));
    session.economy.ingots_stock = 0;
    for route in &mut session.outposts {
        route.expeditions_completed = goal.saturating_sub(1);
    }

    assert!(!outposts::claim_outpost_charter(&mut session, &data));
    session.outposts[0].expeditions_completed = goal;

    assert!(outposts::claim_outpost_charter(&mut session, &data));
    assert!(session.outpost_charter_claimed);
    assert!(session.equipment_unlocked(drill));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_charter_reward_ingots
    );
    assert!(!outposts::claim_outpost_charter(&mut session, &data));
}

#[test]
fn charter_claims_from_a_real_expedition_completion() {
    let (data, mut session, _) = super::novel::active_outpost(165);
    let goal = data.balance.outpost_charter_haul_goal;
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    session.economy.ingots_stock = 0;
    let route = &mut session.outposts[0];
    route.storage_upgraded = true;
    route.crew_upgraded = true;
    route.survey_upgraded = true;
    route.resonator_upgraded = true;
    route.crew = crew;
    route.cargo.insert(Good::CookedFood, 2);
    route.expeditions_completed = goal.saturating_sub(1);
    route.expedition_progress = outposts::expedition_cycle_sec(route, &data) - simulation::SIM_DT;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.expedition_completed.len(), 1);
    assert!(report.outpost_charter_awarded);
    assert!(session.outpost_charter_claimed);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_charter_reward_ingots
    );
}

#[test]
fn charter_hauling_frame_replaces_a_weaker_carrier_tool() {
    let (data, mut session, _) = super::novel::active_outpost(166);
    session.outpost_charter_claimed = true;
    let carrier = session
        .creatures
        .iter_mut()
        .find(|creature| creature.job == crate::state::creatures::Job::Carrier)
        .expect("the active-outpost fixture has a carrier");
    carrier.equipment = Some("hauling_frame".to_owned());
    session
        .economy
        .gear_stock
        .insert("wormbone_hauling_frame".to_owned(), 1);

    simulation::tick(&mut session, &data);

    let carrier = session
        .creatures
        .iter()
        .find(|creature| creature.job == crate::state::creatures::Job::Carrier)
        .unwrap();
    assert_eq!(carrier.equipment.as_deref(), Some("wormbone_hauling_frame"));
    assert_eq!(session.economy.gear_stock.get("hauling_frame"), Some(&1));
}

#[test]
fn charter_hammer_shortens_a_blacksmith_craft() {
    let (data, mut session, _) = super::novel::active_outpost(167);
    session.outpost_charter_claimed = true;
    let spot = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable blacksmith location");
    session.buildings.push(Building::new("blacksmith", spot));
    session.economy.ingots_stock = 2;
    session.spawn_creature(&data, "goblin", Job::Smith);
    let smith_id = session.creatures.last().unwrap().id;
    session
        .economy
        .gear_stock
        .insert("wormbone_smiths_hammer".to_owned(), 1);

    simulation::tick(&mut session, &data);
    let smith = session
        .creatures
        .iter_mut()
        .find(|creature| creature.id == smith_id)
        .unwrap();
    assert_eq!(smith.equipment.as_deref(), Some("wormbone_smiths_hammer"));
    smith.x = spot.x as f32 + 0.5;
    smith.y = spot.y as f32 + 0.5;
    smith.clear_task();
    assert_eq!(
        session.queue_equipment_order(
            &data,
            spot,
            "iron_pickaxe".to_owned(),
            data.balance.order_queue_size,
        ),
        Some(2)
    );

    simulation::tick(&mut session, &data);

    let smith = session
        .creatures
        .iter()
        .find(|creature| creature.id == smith_id)
        .unwrap();
    match &smith.task {
        Task::Crafting {
            item, remaining, ..
        } => {
            assert_eq!(item, "iron_pickaxe");
            assert!((*remaining - data.balance.gear_craft_time_sec * 0.6).abs() < 0.001);
        }
        task => panic!("expected a shortened craft task, got {task:?}"),
    }
}

#[test]
fn charter_blade_replaces_a_weaker_guard_tool() {
    let (data, mut session, _) = super::novel::active_outpost(168);
    session.outpost_charter_claimed = true;
    let guard = session
        .creatures
        .first_mut()
        .expect("the fixture has a worker");
    guard.job = Job::Guard;
    guard.equipment = Some("guard_blade".to_owned());
    session
        .economy
        .gear_stock
        .insert("wormbone_guard_blade".to_owned(), 1);

    simulation::tick(&mut session, &data);

    let guard = session
        .creatures
        .iter()
        .find(|creature| creature.job == Job::Guard)
        .unwrap();
    assert_eq!(guard.equipment.as_deref(), Some("wormbone_guard_blade"));
    assert_eq!(session.economy.gear_stock.get("guard_blade"), Some(&1));
}

#[test]
fn deep_survey_requires_the_claimed_charter_and_resonator() {
    let (data, mut session, outpost_pos) = super::novel::active_outpost(169);
    session.economy.ingots_stock = data.balance.outpost_deep_survey_upgrade_ingots;
    session.outposts[0].resonator_upgraded = true;

    assert!(!outposts::upgrade_outpost_deep_survey(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_deep_survey_upgrade_ingots
    );

    session.outpost_charter_claimed = true;
    assert!(outposts::upgrade_outpost_deep_survey(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(session.economy.ingots_stock, 0);
    assert!(session.outposts[0].deep_survey_upgraded);
    assert_eq!(
        outposts::ore_per_crew(&session.outposts[0], &data),
        data.balance.outpost_deep_survey_ore_per_crew
    );
    assert!(!outposts::upgrade_outpost_deep_survey(
        &mut session,
        &data,
        outpost_pos
    ));
}

#[test]
fn deep_survey_increases_a_real_expedition_yield() {
    let (data, mut session, _outpost_pos) = super::novel::active_outpost(170);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    let route = &mut session.outposts[0];
    route.storage_upgraded = true;
    route.crew_upgraded = true;
    route.survey_upgraded = true;
    route.resonator_upgraded = true;
    route.deep_survey_upgraded = true;
    route.crew = crew;
    route.cargo.insert(Good::CookedFood, 2);
    route.expedition_progress = outposts::expedition_cycle_sec(route, &data);

    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(
        completed[0].ore,
        2 * data.balance.outpost_deep_survey_ore_per_crew
    );
    assert_eq!(session.outposts[0].ore_scouted, completed[0].ore);
}

#[test]
fn signal_cache_requires_relay_and_pays_for_each_real_haul() {
    let (data, mut session, outpost_pos) = super::novel::active_outpost(174);
    session.economy.ingots_stock = data.balance.outpost_signal_cache_upgrade_ingots;
    let route = &mut session.outposts[0];
    route.storage_upgraded = true;
    route.crew_upgraded = true;
    route.survey_upgraded = true;
    route.resonator_upgraded = true;
    route.deep_survey_upgraded = true;
    route.active = true;

    assert!(!outposts::upgrade_outpost_signal_cache(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_signal_cache_upgrade_ingots
    );

    session.outpost_relay_claimed = true;
    assert!(outposts::upgrade_outpost_signal_cache(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(session.economy.ingots_stock, 0);

    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    let route = &mut session.outposts[0];
    route.crew = crew;
    route.cargo.insert(Good::CookedFood, 2);
    route.expedition_progress = outposts::expedition_cycle_sec(route, &data);

    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(completed.len(), 1);
    assert_eq!(
        completed[0].ore,
        2 * data.balance.outpost_deep_survey_ore_per_crew
    );
    assert_eq!(
        completed[0].ingots,
        data.balance.outpost_signal_cache_ingots_per_haul
    );
    assert_eq!(
        session.outposts[0].cargo.get(&Good::Ingot),
        Some(&data.balance.outpost_signal_cache_ingots_per_haul)
    );
}
