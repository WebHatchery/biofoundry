use super::*;
use crate::data::GameData;
use crate::state::structures::Building;
use crate::state::GameSession;

fn session_with_outpost() -> (GameData, GameSession) {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 270);
    let shrine = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable shrine location");
    let outpost = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| *pos != shrine && tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("worm_shrine", shrine));
    session.buildings.push(Building::new("outpost", outpost));
    session.ensure_outpost(outpost);
    (data, session)
}

#[test]
fn loaded_survey_rig_requires_both_logistics_expansions() {
    let (data, mut session) = session_with_outpost();
    session.outposts[0].survey_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("a survey rig cannot exist before the hold and camp upgrades");

    assert!(error.contains("expanded hold and camp"), "{error}");
}

#[test]
fn loaded_resonance_beacon_requires_a_survey_rig() {
    let (data, mut session) = session_with_outpost();
    session.outposts[0].resonator_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("a resonance beacon cannot exist before the survey rig");

    assert!(error.contains("survey rig"), "{error}");
}

#[test]
fn loaded_deep_survey_requires_a_resonance_beacon() {
    let (data, mut session) = session_with_outpost();
    session.outpost_charter_claimed = true;
    session.outposts[0].deep_survey_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("deep survey cannot exist before the resonance beacon");

    assert!(error.contains("resonance beacon"), "{error}");
}

#[test]
fn loaded_deep_survey_requires_the_worm_road_charter() {
    let (data, mut session) = session_with_outpost();
    session.outposts[0].storage_upgraded = true;
    session.outposts[0].crew_upgraded = true;
    session.outposts[0].survey_upgraded = true;
    session.outposts[0].resonator_upgraded = true;
    session.outposts[0].deep_survey_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("deep survey cannot exist before the Charter");

    assert!(error.contains("Worm Road Charter"), "{error}");
}

#[test]
fn loaded_signal_cache_requires_deep_survey() {
    let (data, mut session) = session_with_outpost();
    session.outpost_relay_claimed = true;
    session.outposts[0].signal_cache_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("Signal Cache cannot exist before Deep Survey");

    assert!(error.contains("deep survey"), "{error}");
}

#[test]
fn loaded_signal_cache_requires_the_worm_road_relay() {
    let (data, mut session) = session_with_outpost();
    session.outpost_charter_claimed = true;
    session.outposts[0].storage_upgraded = true;
    session.outposts[0].crew_upgraded = true;
    session.outposts[0].survey_upgraded = true;
    session.outposts[0].resonator_upgraded = true;
    session.outposts[0].deep_survey_upgraded = true;
    session.outposts[0].signal_cache_upgraded = true;

    let error = validate_loaded_session(&session, &data)
        .expect_err("Signal Cache cannot exist before the Relay");

    assert!(error.contains("Worm Road Relay"), "{error}");
}
