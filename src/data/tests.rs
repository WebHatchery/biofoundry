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

    let slime = data.species.get("slime_janitor").expect("slime janitor");
    assert!(!slime.reassignable);
    let bat = data.species.get("bat_courier").expect("bat courier");
    assert!(bat.carry_capacity > beetle.carry_capacity);
    let engineer = data.species.get("engineer").expect("engineer");
    assert!(engineer.work_mult > 1.0);
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
    let drill = data
        .equipment_def("wormbone_drill")
        .expect("Wormbone Drill exists");
    assert_eq!(drill.requires_unlock.as_deref(), Some("outpost_charter"));
    assert!(
        drill.value > pick.value,
        "the Charter reward should improve mining"
    );
    let frame = data
        .equipment_def("wormbone_hauling_frame")
        .expect("Wormbone Hauling Frame exists");
    let hauling_frame = data
        .equipment_def("hauling_frame")
        .expect("hauling frame exists");
    assert_eq!(frame.job, "carrier");
    assert_eq!(frame.requires_unlock.as_deref(), Some("outpost_charter"));
    assert!(
        frame.value > hauling_frame.value,
        "the Charter reward should improve hauling"
    );
    let hammer = data
        .equipment_def("wormbone_smiths_hammer")
        .expect("Wormbone Smith's Hammer exists");
    let smiths_hammer = data
        .equipment_def("smiths_hammer")
        .expect("smith's hammer exists");
    assert_eq!(hammer.job, "smith");
    assert_eq!(hammer.requires_unlock.as_deref(), Some("outpost_charter"));
    assert!(
        hammer.value < smiths_hammer.value,
        "the Charter reward should shorten smithing time"
    );
    let blade = data
        .equipment_def("wormbone_guard_blade")
        .expect("Wormbone Guard Blade exists");
    let guard_blade = data
        .equipment_def("guard_blade")
        .expect("guard blade exists");
    assert_eq!(blade.job, "guard");
    assert_eq!(blade.requires_unlock.as_deref(), Some("outpost_charter"));
    assert!(
        blade.value > guard_blade.value,
        "the Charter reward should improve guard damage"
    );
    let wayfinder = data
        .equipment_def("archive_wayfinder")
        .expect("Archive Wayfinder exists");
    assert_eq!(wayfinder.job, "carrier");
    assert_eq!(
        wayfinder.requires_unlock.as_deref(),
        Some("archive_wayfinder")
    );
    assert!(
        wayfinder.value > frame.value,
        "the first Archive page should improve hauling"
    );
    let resonance_tools = [
        ("wormsong_harness", "carrier", 6.0),
        ("wormsong_drill", "miner", 2.5),
        ("wormsong_smiths_hammer", "smith", 0.5),
        ("wormsong_guard_blade", "guard", 2.5),
    ];
    for (id, job, value) in resonance_tools {
        let tool = data.equipment_def(id).expect("Wormsong tool exists");
        assert_eq!(tool.job, job);
        assert_eq!(tool.value, value);
        assert_eq!(tool.requires_unlock.as_deref(), Some("resonance_forging"));
    }
    assert_eq!(
        data.equipment_def("wormsong_harness")
            .unwrap()
            .remote_storage_bonus,
        2
    );
    assert_eq!(
        data.equipment_def("wormsong_drill")
            .unwrap()
            .remote_ore_bonus,
        2
    );
    assert_eq!(
        data.equipment_def("wormsong_smiths_hammer")
            .unwrap()
            .remote_ingot_bonus,
        1
    );
    assert_eq!(
        data.equipment_def("wormsong_guard_blade")
            .unwrap()
            .remote_cycle_reduction,
        4.0
    );
    assert!(
        data.equipment_def("wormsong_harness").unwrap().value > wayfinder.value,
        "the first Muster should improve the carrier tier"
    );
    assert!(data.unlocks.iter().any(
        |unlock| unlock.id == "resonance_forging" && unlock.counter == "outpost_muster_claims"
    ));
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
    assert!(b.raw_recipe_multiplier > 0.0);
    assert!(b.cooked_recipe_multiplier > 0.0);
    assert!(b.waste_storage_cap > 0.0);
    assert!(b.outpost_capacity > 0);
    assert!(b.outpost_upgrade_ingots > 0);
    assert!(b.outpost_upgraded_storage_cap > b.outpost_storage_cap);
    assert!(b.outpost_crew_upgrade_ingots > 0);
    assert!(b.outpost_upgraded_capacity > b.outpost_capacity);
    assert!(b.outpost_survey_upgrade_ingots > b.outpost_crew_upgrade_ingots);
    assert!(b.outpost_upgraded_ore_per_crew > b.outpost_expedition_ore_per_crew);
    assert!(b.outpost_resonator_upgrade_ingots > b.outpost_survey_upgrade_ingots);
    assert!(b.outpost_resonator_cycle_sec > 0.0);
    assert!(b.outpost_resonator_cycle_sec < b.outpost_expedition_cycle_sec);
    assert!(b.outpost_deep_survey_upgrade_ingots > b.outpost_resonator_upgrade_ingots);
    assert!(b.outpost_deep_survey_ore_per_crew > b.outpost_upgraded_ore_per_crew);
    assert!(b.outpost_charter_haul_goal > 0);
    assert!(b.outpost_charter_reward_ingots > 0);
    assert!(b.outpost_archive_haul_goal > 0);
    assert!(b.outpost_archive_reward_ingots > 0);
    assert!(b.outpost_relay_route_goal >= 2);
    assert!(b.outpost_relay_haul_goal > 0);
    assert!(b.outpost_relay_reward_ingots > 0);
    assert!(b.outpost_convoy_route_goal > b.outpost_relay_route_goal);
    assert!(b.outpost_convoy_haul_goal > b.outpost_relay_haul_goal);
    assert!(b.outpost_convoy_reward_ingots > b.outpost_relay_reward_ingots);
    assert!(b.outpost_muster_route_goal > b.outpost_convoy_route_goal);
    assert!(b.outpost_muster_haul_goal > b.outpost_convoy_haul_goal);
    assert!(b.outpost_muster_reward_ingots > b.outpost_convoy_reward_ingots);
    assert_eq!(b.outpost_concord_role_goal, 4);
    assert!(b.outpost_concord_reward_ingots > b.outpost_muster_reward_ingots);
    assert_eq!(b.outpost_concord_ore_bonus, 1);
    assert_eq!(b.outpost_concord_cycle_reduction, 2.0);
    assert!(b.outpost_circuit_route_goal >= 2);
    assert!(b.outpost_circuit_reward_ingots > b.outpost_concord_reward_ingots);
    assert!(b.outpost_signal_cache_upgrade_ingots > b.outpost_relay_reward_ingots);
    assert!(b.outpost_signal_cache_ingots_per_haul > 0);
    assert!(b.outpost_waypoint_upgrade_ingots > b.outpost_signal_cache_upgrade_ingots);
    assert!(b.outpost_waypoint_transit_time_sec > 0.0);
    assert!(b.outpost_waypoint_transit_time_sec < b.worm_transit_time_sec);
    assert!(b.win_ore_delivered > 0);
    assert!(b.win_food_surplus > b.start_food);
    assert!(b.food_warning_sec > 0.0);
    assert!(b.raid_warning_sec > 0.0 && b.raid_warning_sec < b.raid_first_sec);
    assert!(b.worm_food_per_min > 0.0);
    assert!(b.worm_food_per_offering > 0.0);
    assert!(b.worm_awaken_ingots > 0);
    assert!(b.outpost_expedition_cycle_sec > 0.0);
    assert!(b.outpost_expedition_food_per_crew > 0);
    assert!(b.outpost_expedition_ore_per_crew > 0);
    assert!(
        b.worm_awaken_at >= b.worm_food_per_offering * b.worm_awaken_ingots as f32,
        "the food goal must fund every required ingot offering"
    );
    // Cooking must multiply calories, or the loop can never go positive.
    assert!(b.cook_batch_food / b.cook_batch_mushrooms as f32 > 1.0);
}

#[test]
fn content_validation_rejects_broken_references() {
    let mut data = GameData::load().unwrap();
    data.unlocks[0].building = Some("missing_building".to_owned());

    let error = data
        .validate()
        .expect_err("broken content should be rejected");
    assert!(error.contains("missing_building"));
}

#[test]
fn content_validation_rejects_broken_equipment_unlocks() {
    let mut data = GameData::load().unwrap();
    data.equipment[0].requires_unlock = Some("missing_unlock".to_owned());

    let error = data
        .validate()
        .expect_err("broken equipment gates should be rejected");
    assert!(error.contains("missing_unlock"));
}

#[test]
fn content_validation_rejects_a_disabled_signal_cache() {
    let mut data = GameData::load().unwrap();
    data.balance.outpost_signal_cache_ingots_per_haul = 0;

    let error = data
        .validate()
        .expect_err("a zero-payload Signal Cache should be rejected");
    assert!(error.contains("Signal Cache"));
}

#[test]
fn content_validation_rejects_a_disabled_convoy_contract() {
    let mut data = GameData::load().unwrap();
    data.balance.outpost_convoy_reward_ingots = 0;

    let error = data
        .validate()
        .expect_err("a zero-reward Convoy should be rejected");
    assert!(error.contains("convoy"));
}

#[test]
fn content_validation_rejects_a_muster_that_does_not_raise_the_network_bar() {
    let mut data = GameData::load().unwrap();
    data.balance.outpost_muster_route_goal = data.balance.outpost_convoy_route_goal;

    let error = data
        .validate()
        .expect_err("a Muster should raise the route requirement");
    assert!(error.contains("Muster"));
}
