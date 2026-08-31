//! Player-facing text for simulation events.

use crate::state::outposts::ExpeditionCompletion;

pub(crate) fn format_expedition_completion(completion: ExpeditionCompletion) -> String {
    if completion.ingots > 0 {
        format!(
            "Outpost haul · +{} ore · +{} ingot{} / -{} food.",
            completion.ore,
            completion.ingots,
            if completion.ingots == 1 { "" } else { "s" },
            completion.food_spent
        )
    } else {
        format!(
            "Outpost haul · +{} ore / -{} food.",
            completion.ore, completion.food_spent
        )
    }
}
