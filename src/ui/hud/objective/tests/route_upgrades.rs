use super::*;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;

#[test]
fn completed_objective_guides_the_route_upgrade_ladder_when_funded() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].active = true;

    session.economy.ingots_stock = data.balance.outpost_upgrade_ingots;
    assert!(CampaignObjective::current(&session, &data)
        .next
        .contains("Expand hold"));

    session.outposts[0].storage_upgraded = true;
    session.economy.ingots_stock = data.balance.outpost_crew_upgrade_ingots;
    assert!(CampaignObjective::current(&session, &data)
        .next
        .contains("Expand camp"));

    session.outposts[0].crew_upgraded = true;
    session.economy.ingots_stock = data.balance.outpost_survey_upgrade_ingots;
    assert!(CampaignObjective::current(&session, &data)
        .next
        .contains("Install survey"));

    session.outposts[0].survey_upgraded = true;
    session.economy.ingots_stock = data.balance.outpost_resonator_upgrade_ingots;
    let objective = CampaignObjective::current(&session, &data);
    assert!(objective.next.contains("Tune beacon"));
    assert!(objective.progress.contains("Upgrades 3"));

    session.outposts[0].resonator_upgraded = true;
    session.outpost_charter_claimed = true;
    session.economy.ingots_stock = data.balance.outpost_deep_survey_upgrade_ingots;
    let objective = CampaignObjective::current(&session, &data);
    assert!(objective.next.contains("Calibrate deep survey"));
    assert!(objective.progress.contains("Upgrades 4"));

    session.outposts[0].deep_survey_upgraded = true;
    assert!(CampaignObjective::current(&session, &data)
        .progress
        .contains("Upgrades 5"));

    session.outpost_relay_claimed = true;
    session.economy.ingots_stock = data.balance.outpost_signal_cache_upgrade_ingots;
    assert!(CampaignObjective::current(&session, &data)
        .next
        .contains("Install signal cache"));

    session.outposts[0].signal_cache_upgraded = true;
    assert!(CampaignObjective::current(&session, &data)
        .progress
        .contains("Upgrades 6"));

    session.outpost_convoy_claims = 1;
    session.economy.ingots_stock = data.balance.outpost_waypoint_upgrade_ingots;
    assert!(CampaignObjective::current(&session, &data)
        .next
        .contains("Install Worm Road waypoint"));

    session.outposts[0].waypoint_upgraded = true;
    assert!(CampaignObjective::current(&session, &data)
        .progress
        .contains("Upgrades 7"));
}

#[test]
fn completed_objective_guides_the_next_network_muster_after_convoy() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 1;
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a route location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].active = true;
    session.outposts[0].crew.push(1);
    session.outposts[0].cargo.insert(Good::CookedFood, 1);
    session.outposts[0].expeditions_completed =
        data.balance.outpost_relay_haul_goal + data.balance.outpost_convoy_haul_goal + 5;

    let objective = CampaignObjective::current(&session, &data);

    assert!(
        objective.next.contains("Worm Road Muster"),
        "{}",
        objective.next
    );
    assert!(objective.next.contains("1/4 active routes"));
    assert!(objective.next.contains("5/16 hauls"));
}

#[test]
fn completed_objective_guides_the_wormsong_concord_after_muster() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 1;
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    session.outpost_muster_claims = 1;
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].active = true;
    let mut crew = Vec::new();
    for (job, equipment) in [
        (Job::Carrier, "wormsong_harness"),
        (Job::Miner, "wormsong_drill"),
        (Job::Smith, "wormsong_smiths_hammer"),
    ] {
        session.spawn_creature(&data, "goblin", job);
        let creature = session.creatures.last_mut().expect("specialist spawned");
        creature.equipment = Some(equipment.to_owned());
        creature.remote_outpost = Some(pos);
        crew.push(creature.id);
    }
    session.outposts[0].crew = crew;
    session.outposts[0].cargo.insert(Good::CookedFood, 4);

    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.next.contains("Wormsong"));
    assert!(objective.next.contains("3/4 roles"));
    assert!(objective.next.contains("station a Wormsong"));
}
