use biofoundry::data::GameData;
use biofoundry::state::structures::Building;
use biofoundry::state::GameSession;
use biofoundry::ui::hud::inspect::study::*;

#[test]
fn study_readout_explains_the_next_adaptation() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 51);
    session.progress.specimens = 2;
    session.progress.knowledge = 7.8;
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable study pen location");
    session.buildings.push(Building::new("study_pen", pos));

    assert_eq!(study_rate_per_min(&session, &data), 2.0);
    assert_eq!(
        study_adaptation_line(&session, &data),
        "Next · gain 12 study (7/12)"
    );
}

#[test]
fn study_readout_points_from_haulers_to_the_next_adaptation() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 52);
    session.unlocked.insert("adaptive_haulers".to_owned());

    assert_eq!(
        study_adaptation_line(&session, &data),
        "Haulers +20% · next 0/24 study"
    );
}

#[test]
fn study_readout_reports_both_adaptations_when_mastered() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 53);
    session.progress.knowledge = 24.0;
    session.unlocked.insert("adaptive_haulers".to_owned());
    session.unlocked.insert("brood_memory".to_owned());

    assert_eq!(
        study_adaptation_line(&session, &data),
        "Haulers +20% · Brood +20%"
    );
}
