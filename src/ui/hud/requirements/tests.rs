use super::*;
use crate::data::GameData;

#[test]
fn unlocks_explain_the_action_and_threshold() {
    let data = GameData::load().unwrap();

    assert_eq!(
        unlock_requirement(&data, "worm_shrine").as_deref(),
        Some("forge 20 ingots")
    );
    assert_eq!(
        unlock_requirement(&data, "breeding_pit").as_deref(),
        Some("capture 2 beetles")
    );
    assert_eq!(
        unlock_requirement(&data, "slime_janitor").as_deref(),
        Some("spoil 10 food")
    );
}
