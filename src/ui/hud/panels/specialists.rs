//! Compact labels for optional post-campaign support creatures.

pub(super) fn optional_specialist_label(species: &str, posted: bool) -> &'static str {
    match (species, posted) {
        ("slime_janitor", false) => "Slime · clean",
        ("slime_janitor", true) => "Slime · posted",
        ("bat_courier", false) => "Bat · 8 cargo",
        ("bat_courier", true) => "Bat · posted",
        _ => "Specialist",
    }
}

pub(super) fn optional_support_label(species: &str, cost: u32, local_count: usize) -> String {
    match (species, local_count) {
        ("beetle", count) if count > 0 => format!("Beetle x{count} haul"),
        ("salamander", count) if count > 0 => format!("Salam x{count} forge"),
        ("beetle", _) => format!("Beetle haul ({cost})"),
        ("salamander", _) => format!("Salam. forge ({cost})"),
        _ => format!("{species} ({cost})"),
    }
}
