use crate::data::GameData;
use crate::simulation::wildlife;
use crate::state::GameSession;
use crate::ui::hud::requirements::unlock_requirement_progress;

pub(super) fn study_rate_per_min(session: &GameSession, data: &GameData) -> f32 {
    session.progress.specimens as f32 * data.balance.study_knowledge_per_specimen_min
}

pub(super) fn study_adaptation_line(session: &GameSession, data: &GameData) -> String {
    let Some(haulers) = data
        .unlocks
        .iter()
        .find(|unlock| unlock.id == "adaptive_haulers")
    else {
        return "No adaptation recorded".to_owned();
    };

    if !session.unlocked.contains(&haulers.id) {
        if let Some(requirement) = unlock_requirement_progress(session, data, &haulers.id) {
            return format!("Next · {requirement}");
        }
        return "Adaptation in progress".to_owned();
    }

    let hauler_bonus = (wildlife::beetle_carry_capacity_multiplier(session, data) - 1.0) * 100.0;
    let Some(brood) = data
        .unlocks
        .iter()
        .find(|unlock| unlock.id == "brood_memory")
    else {
        return format!("Adapted haulers · +{hauler_bonus:.0}% carry");
    };
    if !session.unlocked.contains(&brood.id) {
        let current = wildlife::counter_value(session, &brood.counter);
        return format!(
            "Haulers +{hauler_bonus:.0}% · next {current}/{} study",
            brood.threshold
        );
    }

    let hatch_bonus = if data.balance.breed_interval_sec > 0.0 {
        (1.0 - wildlife::breeding_interval_sec(session, data) / data.balance.breed_interval_sec)
            * 100.0
    } else {
        0.0
    };
    format!("Haulers +{hauler_bonus:.0}% · Brood +{hatch_bonus:.0}%")
}

#[cfg(test)]
mod tests;
