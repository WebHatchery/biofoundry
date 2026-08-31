use super::*;
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
}
