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

pub(super) fn breeding_unlock_hint(
    session: &GameSession,
    data: &GameData,
    unlock: &str,
) -> Option<String> {
    unlock_requirement_progress(session, data, unlock)
        .map(|requirement| format!("Next: {requirement}"))
}
