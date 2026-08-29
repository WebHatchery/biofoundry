//! Player-facing prerequisite phrases for locked optional actions.

use crate::data::GameData;

/// Translate a data-driven unlock into a concise action the player can take.
pub(super) fn unlock_requirement(data: &GameData, id: &str) -> Option<String> {
    let unlock = data.unlocks.iter().find(|candidate| candidate.id == id)?;
    let phrase = match unlock.counter.as_str() {
        "beetles_captured" => format!("capture {} beetles", unlock.threshold),
        "raids_survived" => format!("survive {} raid", unlock.threshold),
        "famines_survived" => format!("survive {} famine", unlock.threshold),
        "specimens" => format!("house {} specimens", unlock.threshold),
        "knowledge" => format!("gain {} study", unlock.threshold),
        "waste_processed" => format!("process {} waste", unlock.threshold),
        "courier_deliveries" => format!("complete {} courier deliveries", unlock.threshold),
        "ingots_forged" => format!("forge {} ingots", unlock.threshold),
        "ore_delivered_total" => format!("deliver {} ore", unlock.threshold),
        _ => return None,
    };
    Some(phrase)
}

#[cfg(test)]
mod tests;
