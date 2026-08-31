//! Player-facing prerequisite phrases for locked optional actions.

use crate::data::GameData;
use crate::simulation::wildlife;
use crate::state::GameSession;

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
        "waste_generated" => format!("spoil {} food", unlock.threshold),
        "courier_deliveries" => format!("complete {} courier deliveries", unlock.threshold),
        "ingots_forged" => format!("forge {} ingots", unlock.threshold),
        "ore_delivered_total" => format!("deliver {} ore", unlock.threshold),
        "outpost_archive_claims" => {
            let noun = if unlock.threshold == 1 {
                "Archive page"
            } else {
                "Archive pages"
            };
            format!("log {} {noun}", unlock.threshold)
        }
        "outpost_muster_claims" => {
            let noun = if unlock.threshold == 1 {
                "Worm Road Muster"
            } else {
                "Worm Road Musters"
            };
            format!("hold {} {noun}", unlock.threshold)
        }
        _ => return None,
    };
    Some(phrase)
}

/// Translate a data-driven unlock into an actionable phrase with live
/// progress, using the same counter lookup that grants the unlock.
pub(super) fn unlock_requirement_progress(
    session: &GameSession,
    data: &GameData,
    id: &str,
) -> Option<String> {
    let unlock = data.unlocks.iter().find(|candidate| candidate.id == id)?;
    let phrase = unlock_requirement(data, id)?;
    let current = wildlife::counter_value(session, &unlock.counter);
    Some(format!("{phrase} ({current}/{})", unlock.threshold))
}

#[cfg(test)]
mod tests;
