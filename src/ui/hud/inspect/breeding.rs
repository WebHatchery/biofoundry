//! Labels and prerequisite hints for the Breeding Pit inspection card.

use crate::data::GameData;
use crate::state::GameSession;
use crate::ui::hud::requirements::unlock_requirement_progress;

pub(super) fn breed_label(id: &str, name: &str, cost: u32, data: &GameData) -> String {
    match id {
        "hobgoblin" => {
            let work = data
                .species
                .get(id)
                .map(|species| species.work_mult)
                .unwrap_or(2.0);
            format!("{name} · ×{work:.0} work ({cost})")
        }
        "overseer" => {
            let aura = ((data.balance.overseer_aura_mult - 1.0) * 100.0).round();
            format!("{name} · aura +{aura:.0}% ({cost})")
        }
        "engineer" => {
            let mine = data
                .species
                .get(id)
                .map(|species| (species.work_mult - 1.0) * 100.0)
                .unwrap_or(25.0);
            format!("{name} · Mine +{mine:.0}% ({cost})")
        }
        _ => format!("{name} ({cost} ingots)"),
    }
}

/// Add the ongoing food draw to an available bred-specialist action. The
/// benefit and one-time ingot price stay on the first line; the second line
/// makes the continuing cost visible before the player commits.
pub(super) fn breed_button_label(id: &str, name: &str, cost: u32, data: &GameData) -> String {
    let label = breed_label(id, name, cost, data);
    let Some(species) = data.species.get(id) else {
        return label;
    };
    if species.diet != "food" || species.food_per_min <= 0.0 {
        return label;
    }
    let heading = label
        .rsplit_once(" (")
        .map(|(heading, _)| heading)
        .unwrap_or(label.as_str());
    format!(
        "{heading}\n{cost} ingots · +{:.1} food/min",
        species.food_per_min
    )
}

pub(super) fn breeding_unlock_hint(
    session: &GameSession,
    data: &GameData,
    unlock: &str,
) -> Option<String> {
    unlock_requirement_progress(session, data, unlock)
        .map(|requirement| format!("Next: {requirement}"))
}
