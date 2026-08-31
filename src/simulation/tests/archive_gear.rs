//! Coverage for the Archive Wayfinder progression payoff.

use crate::simulation::{self, outposts};
use crate::state::creatures::Job;
use crate::state::structures::Building;

#[test]
fn first_archive_page_unlocks_wayfinder_after_a_save_roundtrip() {
    let (data, mut session, _) = super::novel::active_outpost(171);
    session.outpost_charter_claimed = true;
    session.outposts[0].expeditions_completed = data
        .balance
        .outpost_charter_haul_goal
        .saturating_add(data.balance.outpost_archive_haul_goal);

    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 1);
    assert_eq!(session.outpost_archive_claims, 1);
    assert!(!session.unlocked.contains("archive_wayfinder"));

    let encoded = serde_json::to_string(&session).expect("Archive page should serialize");
    let mut restored = serde_json::from_str(&encoded).expect("Archive page should deserialize");
    let report = simulation::tick(&mut restored, &data);

    assert!(report
        .wild
        .unlocked
        .iter()
        .any(|name| name == "Archive Wayfinder"));
    let wayfinder = data
        .equipment_def("archive_wayfinder")
        .expect("Archive Wayfinder data");
    assert!(restored.equipment_unlocked(wayfinder));
}

#[test]
fn archive_wayfinder_replaces_a_weaker_carrier_tool() {
    let (data, mut session, _) = super::novel::active_outpost(172);
    let stockpile = session.stockpile_pos();
    let carrier_index = session
        .creatures
        .iter()
        .position(|creature| creature.job == Job::Carrier)
        .expect("the active-outpost fixture has a carrier");
    {
        let carrier = &mut session.creatures[carrier_index];
        carrier.equipment = Some("wormbone_hauling_frame".to_owned());
        carrier.x = stockpile.x as f32 + 0.5;
        carrier.y = stockpile.y as f32 + 0.5;
        carrier.clear_task();
    }
    session.unlocked.insert("archive_wayfinder".to_owned());
    session
        .economy
        .gear_stock
        .insert("archive_wayfinder".to_owned(), 1);

    simulation::tick(&mut session, &data);

    let carrier = &session.creatures[carrier_index];
    assert_eq!(carrier.equipment.as_deref(), Some("archive_wayfinder"));
    assert_eq!(
        session.economy.gear_stock.get("wormbone_hauling_frame"),
        Some(&1)
    );
}

#[test]
fn first_muster_unlocks_and_replaces_the_archive_carrier_tool() {
    let (data, mut session, _) = super::novel::active_outpost(181);
    let carrier_index = session
        .creatures
        .iter()
        .position(|creature| creature.job == Job::Carrier)
        .expect("the active-outpost fixture has a carrier");
    session.outpost_muster_claims = 1;
    session.unlocked.insert("archive_wayfinder".to_owned());
    session
        .economy
        .gear_stock
        .insert("wormsong_harness".to_owned(), 1);

    let report = simulation::tick(&mut session, &data);

    assert!(report
        .wild
        .unlocked
        .iter()
        .any(|name| name == "Resonance Forging"));
    assert!(session.unlocked.contains("resonance_forging"));
    assert_eq!(session.creatures[carrier_index].equipment, None);
    let stockpile = session.stockpile_pos();
    let carrier = &mut session.creatures[carrier_index];
    carrier.x = stockpile.x as f32 + 0.5;
    carrier.y = stockpile.y as f32 + 0.5;
    carrier.clear_task();
    simulation::tick(&mut session, &data);
    let carrier = &session.creatures[carrier_index];
    assert_eq!(carrier.equipment.as_deref(), Some("wormsong_harness"));
}

#[test]
fn resonance_tools_replace_weaker_specialist_tools() {
    let (data, mut session, _) = super::novel::active_outpost(182);
    session.unlocked.insert("resonance_forging".to_owned());
    let stockpile = session.stockpile_pos();
    session.creatures.clear();
    for (species, job, weaker, stronger) in [
        ("goblin", Job::Miner, "wormbone_drill", "wormsong_drill"),
        (
            "goblin",
            Job::Smith,
            "wormbone_smiths_hammer",
            "wormsong_smiths_hammer",
        ),
        (
            "goblin",
            Job::Guard,
            "wormbone_guard_blade",
            "wormsong_guard_blade",
        ),
    ] {
        session.spawn_creature(&data, species, job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(weaker.to_owned());
        creature.x = stockpile.x as f32 + 0.5;
        creature.y = stockpile.y as f32 + 0.5;
        session.economy.gear_stock.insert(stronger.to_owned(), 1);
    }

    simulation::tick(&mut session, &data);

    for (job, stronger, weaker) in [
        (Job::Miner, "wormsong_drill", "wormbone_drill"),
        (
            Job::Smith,
            "wormsong_smiths_hammer",
            "wormbone_smiths_hammer",
        ),
        (Job::Guard, "wormsong_guard_blade", "wormbone_guard_blade"),
    ] {
        let creature = session
            .creatures
            .iter()
            .find(|creature| creature.job == job)
            .expect("specialist remains present");
        assert_eq!(creature.equipment.as_deref(), Some(stronger));
        assert_eq!(session.economy.gear_stock.get(weaker), Some(&1));
    }
}

#[test]
fn relay_contract_needs_two_active_routes_and_rewards_once() {
    let (data, mut session, _) = super::novel::active_outpost(173);
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 1;
    session.economy.ingots_stock = 0;

    let first_pos = session.outposts[0].pos;
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| {
            *pos != first_pos && tile.walkable() && session.can_place_building(*pos)
        })
        .map(|(pos, _)| pos)
        .expect("the relay test needs a second outpost site");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);

    let first_haul_goal = data.balance.outpost_relay_haul_goal / 2;
    session.outposts[0].expeditions_completed = first_haul_goal;
    session.outposts[1].expeditions_completed = data
        .balance
        .outpost_relay_haul_goal
        .saturating_sub(first_haul_goal);
    session.outposts[1].active = false;

    assert!(!outposts::claim_outpost_relay(&mut session, &data));
    session.outposts[1].active = true;

    assert!(outposts::claim_outpost_relay(&mut session, &data));
    assert!(session.outpost_relay_claimed);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_relay_reward_ingots
    );
    assert!(!outposts::claim_outpost_relay(&mut session, &data));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_relay_reward_ingots
    );
}
