use biofoundry::ui::hud::panels::food::*;

#[test]
fn resource_ledger_names_spendable_ore_and_ingots() {
    assert_eq!(resource_bank_line(12, 7), "Ore banked 12 · ingots 7");
}
