use biofoundry::state::outposts::*;
use macroquad_toolkit::grid::TilePos;

#[test]
fn new_outpost_defaults_to_ore_first_loading() {
    let outpost = Outpost::new(TilePos::new(4, 4));

    assert_eq!(outpost.cargo_priority, CargoPriority::Ore);
    assert_eq!(outpost.cargo_priority.label(), "Ore first");
    assert!(!outpost.expedition_paused);
    assert_eq!(outpost.crew_dispatch_limit, None);
    assert!(!outpost.storage_upgraded);
    assert!(!outpost.crew_upgraded);
    assert!(!outpost.survey_upgraded);
    assert!(!outpost.resonator_upgraded);
    assert!(!outpost.deep_survey_upgraded);
    assert!(!outpost.signal_cache_upgraded);
    assert_eq!(outpost.signal_cache_ingots, 0);
    assert_eq!(outpost.chorus_ingots, 0);
    assert!(!outpost.waypoint_upgraded);
    assert!(!outpost.auto_return_cargo);
    assert_eq!(outpost.auto_return_label(), "Auto-return · Off");
    assert!(!outpost.auto_resupply_food);
    assert_eq!(outpost.auto_resupply_label(), "Auto-resupply · Off");
    assert!(!outpost.auto_load);
    assert_eq!(outpost.auto_load_label(), "Auto-load · Off");
    assert_eq!(outpost.crew_dispatch_label(4), "Crew per run · Auto");
}

#[test]
fn automatic_route_priority_cycles_through_the_shared_worm_order() {
    assert_eq!(AutoRoutePriority::default(), AutoRoutePriority::Return);
    assert_eq!(AutoRoutePriority::Return.label(), "Return first");
    assert_eq!(
        AutoRoutePriority::Return.next(),
        AutoRoutePriority::Resupply
    );
    assert_eq!(AutoRoutePriority::Resupply.label(), "Food first");
    assert_eq!(AutoRoutePriority::Resupply.next(), AutoRoutePriority::Load);
    assert_eq!(AutoRoutePriority::Load.label(), "Load first");
    assert_eq!(AutoRoutePriority::Load.next(), AutoRoutePriority::Return);
    assert_eq!(
        AutoRoutePriority::Load.order(),
        [
            AutoRoutePriority::Load,
            AutoRoutePriority::Return,
            AutoRoutePriority::Resupply
        ]
    );
}

#[test]
fn crew_capacity_expansion_uses_the_larger_configured_capacity() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    assert_eq!(outpost.crew_capacity(4, 6), 4);

    outpost.upgrade_crew_capacity();
    assert_eq!(outpost.crew_capacity(4, 6), 6);
    assert_eq!(outpost.crew_capacity(6, 4), 6);
}

#[test]
fn survey_rig_uses_the_larger_configured_yield() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    assert_eq!(outpost.survey_ore_per_crew(3, 4), 3);

    outpost.upgrade_survey();
    assert_eq!(outpost.survey_ore_per_crew(3, 4), 4);
    assert_eq!(outpost.survey_ore_per_crew(4, 3), 4);
}

#[test]
fn resonance_beacon_uses_the_shorter_configured_cycle() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    assert_eq!(outpost.expedition_cycle_sec(30.0, 20.0), 30.0);

    outpost.upgrade_resonator();
    assert_eq!(outpost.expedition_cycle_sec(30.0, 20.0), 20.0);
    assert_eq!(outpost.expedition_cycle_sec(20.0, 30.0), 20.0);
}

#[test]
fn older_outpost_saves_default_the_survey_rig_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("survey_upgraded");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.survey_upgraded);
}

#[test]
fn older_outpost_saves_default_the_resonance_beacon_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("resonator_upgraded");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.resonator_upgraded);
}

#[test]
fn older_outpost_saves_default_deep_survey_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("deep_survey_upgraded");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.deep_survey_upgraded);
}

#[test]
fn older_outpost_saves_default_signal_cache_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("signal_cache_upgraded");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("signal_cache_ingots");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.signal_cache_upgraded);
    assert_eq!(restored.signal_cache_ingots, 0);
}

#[test]
fn older_outpost_saves_default_waypoint_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("waypoint_upgraded");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.waypoint_upgraded);
}

#[test]
fn signal_cache_earnings_survive_a_save_roundtrip() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    outpost.signal_cache_upgraded = true;
    outpost.signal_cache_ingots = 7;

    let encoded = serde_json::to_string(&outpost).expect("serialize outpost");
    let restored: Outpost = serde_json::from_str(&encoded).expect("restore outpost");

    assert!(restored.signal_cache_upgraded);
    assert_eq!(restored.signal_cache_ingots, 7);
}

#[test]
fn older_outpost_saves_default_chorus_earnings_to_zero() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("chorus_ingots");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert_eq!(restored.chorus_ingots, 0);
}

#[test]
fn waypoint_upgrade_survives_a_save_roundtrip() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    outpost.waypoint_upgraded = true;

    let encoded = serde_json::to_string(&outpost).expect("serialize outpost");
    let restored: Outpost = serde_json::from_str(&encoded).expect("restore outpost");

    assert!(restored.waypoint_upgraded);
}

#[test]
fn cargo_priority_cycles_in_a_predictable_loop() {
    assert_eq!(CargoPriority::Ore.next(), CargoPriority::Ingots);
    assert_eq!(CargoPriority::Ingots.next(), CargoPriority::Food);
    assert_eq!(CargoPriority::Food.next(), CargoPriority::Ore);

    let mut outpost = Outpost::new(TilePos::new(4, 4));
    outpost.cycle_cargo_priority();
    outpost.cycle_cargo_priority();
    assert_eq!(outpost.cargo_priority, CargoPriority::Food);

    outpost.toggle_expedition();
    assert!(outpost.expedition_paused);
    outpost.toggle_expedition();
    assert!(!outpost.expedition_paused);
}

#[test]
fn crew_dispatch_cycles_from_cargo_only_to_auto() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));

    outpost.cycle_crew_dispatch(4);
    assert_eq!(outpost.crew_dispatch_label(4), "Crew per run · Cargo only");
    outpost.cycle_crew_dispatch(4);
    assert_eq!(outpost.crew_dispatch_label(4), "Crew per run · 1/4");
    outpost.cycle_crew_dispatch(4);
    outpost.cycle_crew_dispatch(4);
    outpost.cycle_crew_dispatch(4);
    outpost.cycle_crew_dispatch(4);
    assert_eq!(outpost.crew_dispatch_limit, None);
    assert_eq!(outpost.crew_dispatch_count(3), 3);
}

#[test]
fn auto_return_toggle_names_the_cargo_only_policy() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));

    outpost.toggle_auto_return();
    assert!(outpost.auto_return_cargo);
    assert_eq!(outpost.auto_return_label(), "Auto-return · Cargo only");

    outpost.toggle_auto_return();
    assert!(!outpost.auto_return_cargo);
    assert_eq!(outpost.auto_return_label(), "Auto-return · Off");
}

#[test]
fn auto_resupply_toggle_names_the_food_only_policy() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));

    outpost.toggle_auto_resupply();
    assert!(outpost.auto_resupply_food);
    assert_eq!(outpost.auto_resupply_label(), "Auto-resupply · Food only");

    outpost.toggle_auto_resupply();
    assert!(!outpost.auto_resupply_food);
    assert_eq!(outpost.auto_resupply_label(), "Auto-resupply · Off");
}

#[test]
fn auto_load_toggle_names_the_standard_dispatch_policy() {
    let mut outpost = Outpost::new(TilePos::new(4, 4));

    outpost.toggle_auto_load();
    assert!(outpost.auto_load);
    assert_eq!(outpost.auto_load_label(), "Auto-load · Cargo + crew");

    outpost.toggle_auto_load();
    assert!(!outpost.auto_load);
    assert_eq!(outpost.auto_load_label(), "Auto-load · Off");
}

#[test]
fn older_outpost_saves_default_auto_load_to_off() {
    let outpost = Outpost::new(TilePos::new(4, 4));
    let mut value = serde_json::to_value(outpost).expect("serialize outpost");
    value
        .as_object_mut()
        .expect("outpost serializes as an object")
        .remove("auto_load");

    let restored: Outpost = serde_json::from_value(value).expect("restore legacy outpost");
    assert!(!restored.auto_load);
}
