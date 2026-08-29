use super::*;
use crate::state::outposts::TransitDirection;

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

#[test]
fn transit_departure_notice_names_each_destination_without_assuming_payload() {
    assert_eq!(
        transit_departure_notice(TransitDirection::ToOutpost),
        "The worm begins its journey to the outpost."
    );
    assert_eq!(
        transit_departure_notice(TransitDirection::ToShrine),
        "The worm begins its journey to the shrine."
    );
}

#[test]
fn outpost_activation_notice_names_the_resulting_route_state() {
    assert_eq!(
        outpost_activation_notice(true),
        "The worm route is now active."
    );
    assert_eq!(
        outpost_activation_notice(false),
        "The worm route is now inactive."
    );
}
