use super::*;

#[test]
fn recruitment_notice_exposes_food_eater_upkeep() {
    let data = GameData::load().expect("embedded game data");

    assert_eq!(
        recruitment_notice(&data, "beetle", "A beetle hauler joins the warren"),
        "A beetle hauler joins the warren — Upkeep +5.0 food/min."
    );
}

#[test]
fn recruitment_notice_does_not_claim_food_for_charcoal_eaters() {
    let data = GameData::load().expect("embedded game data");

    assert_eq!(
        recruitment_notice(&data, "salamander", "A salamander joins the den"),
        "A salamander joins the den."
    );
}
