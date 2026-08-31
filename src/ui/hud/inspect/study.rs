use crate::data::GameData;
use crate::simulation::wildlife;
use crate::state::GameSession;
use crate::ui::hud::requirements::unlock_requirement_progress;

pub(super) fn study_rate_per_min(session: &GameSession, data: &GameData) -> f32 {
    session.progress.specimens as f32 * data.balance.study_knowledge_per_specimen_min
}

pub(super) fn study_adaptation_line(session: &GameSession, data: &GameData) -> String {
    let Some(unlock) = data
        .unlocks
        .iter()
        .find(|unlock| unlock.id == "adaptive_haulers")
    else {
        return "No adaptation recorded".to_owned();
    };

    if session.unlocked.contains(&unlock.id) {
        let bonus = (wildlife::beetle_carry_capacity_multiplier(session, data) - 1.0) * 100.0;
        format!("Adapted haulers · +{bonus:.0}% carry")
    } else if let Some(requirement) = unlock_requirement_progress(session, data, &unlock.id) {
        format!("Next · {requirement}")
    } else {
        "Adaptation in progress".to_owned()
    }
}

#[cfg(test)]
mod tests;
