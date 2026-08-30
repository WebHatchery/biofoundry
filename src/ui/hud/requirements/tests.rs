use super::*;
use crate::data::GameData;
use crate::state::GameSession;

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

#[test]
fn locked_gates_show_live_progress_from_the_simulation_counter() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.progress.waste_generated = 4.75;
    session.economy.ore_delivered_total = 12;

    assert_eq!(
        unlock_requirement_progress(&session, &data, "slime_janitor").as_deref(),
        Some("spoil 10 food (4/10)")
    );
    assert_eq!(
        unlock_requirement_progress(&session, &data, "bat_courier").as_deref(),
        Some("deliver 75 ore (12/75)")
    );
}
