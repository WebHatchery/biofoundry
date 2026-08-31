use super::*;

#[test]
fn study_readout_explains_the_next_adaptation() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 51);
    session.progress.specimens = 2;
    session.progress.knowledge = 7.8;

    assert_eq!(study_rate_per_min(&session, &data), 2.0);
    assert_eq!(
        study_adaptation_line(&session, &data),
        "Next · gain 12 study (7/12)"
    );
}

#[test]
fn study_readout_reports_the_adapted_haul_bonus() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 52);
    session.unlocked.insert("adaptive_haulers".to_owned());

    assert_eq!(
        study_adaptation_line(&session, &data),
        "Adapted haulers · +20% carry"
    );
}
