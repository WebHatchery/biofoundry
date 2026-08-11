use super::*;

#[test]
fn embedded_data_loads() {
    let data = GameData::load().unwrap();

    assert_eq!(data.config.game_name, "biofoundry");
    assert!(data.config.world_width >= 16);
    assert!(data.config.tile_size > 0.0);
}

#[test]
fn species_cross_references_hold() {
    let data = GameData::load().unwrap();

    // The two Phase 1 species must exist and be sane.
    let goblin = data.species.get("goblin").expect("goblin species");
    assert!(goblin.reassignable);
    assert!(goblin.food_per_min > 0.0);
    assert!(goblin.carry_capacity > 0);

    let beetle = data.species.get("beetle").expect("beetle species");
    assert!(!beetle.reassignable);
    assert!(
        beetle.carry_capacity >= goblin.carry_capacity * 5,
        "beetle must haul at least 5x a goblin (plan)"
    );
    assert!(beetle.food_per_min > goblin.food_per_min);
}

#[test]
fn equipment_loads_with_valid_job_affinities() {
    let data = GameData::load().unwrap();
    assert!(!data.equipment.is_empty(), "expected launch equipment set");
    let pick = data
        .equipment_def("iron_pickaxe")
        .expect("iron pickaxe exists");
    assert_eq!(pick.job, "miner");
    assert!(pick.cost_ingots > 0);
    assert!(pick.value > 1.0, "a pickaxe should be a speed multiplier");
    // Every item targets a real, gear-wearing job.
    for eq in &data.equipment {
        assert!(
            ["miner", "carrier", "smith", "guard"].contains(&eq.job.as_str()),
            "unknown job affinity {}",
            eq.job
        );
    }
}

#[test]
fn balance_values_are_playable() {
    let data = GameData::load().unwrap();
    let b = &data.balance;

    assert!(b.start_food > 0.0);
    assert!(b.start_miners + b.start_carriers + b.start_cooks >= 3);
    assert!(b.cook_batch_mushrooms > 0);
    assert!(b.cook_batch_food > 0.0);
    assert!(b.win_ore_delivered > 0);
    assert!(b.win_food_surplus > b.start_food);
    // Cooking must multiply calories, or the loop can never go positive.
    assert!(b.cook_batch_food / b.cook_batch_mushrooms as f32 > 1.0);
}
