use super::*;

#[test]
fn resonator_outpost_hint_exposes_the_shorter_survey_cycle() {
    let data = GameData::load().expect("embedded game data");
    let mut outpost = crate::state::outposts::Outpost::new(TilePos::new(4, 4));
    outpost.active = true;
    outpost.crew = vec![1, 2];
    outpost.cargo.insert(Good::CookedFood, 4);
    outpost.resonator_upgraded = true;
    outpost.expedition_progress = 10.0;

    assert_eq!(
        outpost_expedition_hint(&data, &outpost).as_deref(),
        Some("Expedition 50% · +6/-2 food · 20s")
    );
}

#[test]
fn deep_survey_outpost_hint_exposes_the_calibrated_yield() {
    let data = GameData::load().expect("embedded game data");
    let mut outpost = crate::state::outposts::Outpost::new(TilePos::new(4, 4));
    outpost.active = true;
    outpost.crew = vec![1, 2];
    outpost.cargo.insert(Good::CookedFood, 4);
    outpost.survey_upgraded = true;
    outpost.resonator_upgraded = true;
    outpost.deep_survey_upgraded = true;
    outpost.expedition_progress = 10.0;

    assert_eq!(
        outpost_expedition_hint(&data, &outpost).as_deref(),
        Some("Expedition 50% · +12/-2 food · 20s")
    );
}

#[test]
fn in_flight_payload_summary_names_cargo_and_crew() {
    let transit = WormTransit {
        outpost: TilePos::new(4, 4),
        direction: TransitDirection::ToShrine,
        remaining: 4.0,
        ore: 2,
        ingots: 3,
        food: 4.0,
        passengers: vec![7, 8],
    };

    assert_eq!(
        transit_payload_line(&transit),
        "In flight · 9 cargo · 2 crew"
    );
}
