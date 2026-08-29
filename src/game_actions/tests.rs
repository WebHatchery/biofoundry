use super::*;

#[test]
fn recruitment_notice_exposes_food_eater_upkeep() {
    let data = GameData::load().expect("embedded game data");

    assert_eq!(
        recruitment_notice(&data, "beetle", "A beetle hauler joins the warren"),
        "A beetle hauler joins the warren — carries 5× a goblin load; Upkeep +5.0 food/min."
    );
}

#[test]
fn recruitment_notice_does_not_claim_food_for_charcoal_eaters() {
    let data = GameData::load().expect("embedded game data");

    assert_eq!(
        recruitment_notice(&data, "salamander", "A salamander joins the den"),
        "A salamander joins the den — feeds the Smelter Den."
    );
}

#[test]
fn recruitment_notice_explains_optional_cleaners_and_couriers() {
    let data = GameData::load().expect("embedded game data");

    assert!(recruitment_notice(&data, "slime_janitor", "A slime joins")
        .contains("cleans spoiled stores"));
    assert!(recruitment_notice(&data, "bat_courier", "A bat joins").contains("carries 8 at a time"));
}
