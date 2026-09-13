//! Compact labels for optional post-campaign support creatures.

use crate::data::GameData;

pub fn optional_specialist_label(species: &str, posted: bool) -> &'static str {
    match (species, posted) {
        ("slime_janitor", false) => "Slime · clean",
        ("slime_janitor", true) => "Slime · posted",
        ("bat_courier", false) => "Bat · 8 cargo",
        ("bat_courier", true) => "Bat · posted",
        _ => "Specialist",
    }
}

/// Add the ongoing supply draw to an available specialist action. The first
/// line keeps the familiar role/cost label; the second line makes the upkeep
/// visible before a player commits to the optional recruit.
pub fn optional_specialist_button_label(species: &str, posted: bool, data: &GameData) -> String {
    let label = optional_specialist_label(species, posted);
    if posted {
        return label.to_owned();
    }
    format!("{label}\n{}", optional_upkeep_label(species, data))
}

pub fn optional_support_label(species: &str, cost: u32, local_count: usize) -> String {
    match (species, local_count) {
        ("beetle", count) if count > 0 => format!("Beetle x{count} haul"),
        ("salamander", count) if count > 0 => format!("Salam x{count} forge"),
        ("beetle", _) => format!("Beetle haul ({cost})"),
        ("salamander", _) => format!("Salam. forge ({cost})"),
        _ => format!("{species} ({cost})"),
    }
}

/// Add the ongoing supply draw to a Beetle or Salamander action. Existing
/// local specialists keep their benefit label because the recruitment action
/// is no longer available for them.
pub fn optional_support_button_label(
    species: &str,
    cost: u32,
    local_count: usize,
    data: &GameData,
) -> String {
    let label = optional_support_label(species, cost, local_count);
    if local_count > 0 {
        return label;
    }
    format!("{label}\n{}", optional_upkeep_label(species, data))
}

pub fn optional_upkeep_label(species: &str, data: &GameData) -> String {
    if species == "salamander" {
        return format!(
            "{:.0} charcoal/batch",
            data.balance.smelt_batch_charcoal.max(0.0)
        );
    }
    data.species
        .get(species)
        .filter(|definition| definition.diet == "food" && definition.food_per_min > 0.0)
        .map(|definition| format!("+{:.0} food/min", definition.food_per_min))
        .unwrap_or_else(|| "check Food Grid".to_owned())
}
