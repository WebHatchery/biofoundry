use super::*;
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
    assert!(!outpost.auto_return_cargo);
    assert_eq!(outpost.auto_return_label(), "Auto-return · Off");
    assert!(!outpost.auto_resupply_food);
    assert_eq!(outpost.auto_resupply_label(), "Auto-resupply · Off");
    assert_eq!(outpost.crew_dispatch_label(4), "Crew per run · Auto");
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
