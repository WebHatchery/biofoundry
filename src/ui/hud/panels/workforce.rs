use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::GameSession;

pub(super) fn engineer_status_label(local: usize, total: usize) -> String {
    if local > 0 {
        format!("Engineer {local} local · Mine +25%")
    } else if total > 0 {
        format!("Engineer 0 local · {total} posted")
    } else {
        "Engineer 0 · no local bonus".to_owned()
    }
}

pub(super) fn workforce_capacity_label(session: &GameSession, data: &GameData) -> String {
    format!(
        "Idle {} · Local {}/{}",
        session.job_count(Job::Idle),
        session.local_creature_count(),
        session.local_warren_capacity(data)
    )
}

pub(super) fn workforce_pressure_label(session: &GameSession, data: &GameData) -> Option<String> {
    let capacity = session.local_warren_capacity(data);
    let local = session.local_creature_count();
    if local == 0 {
        return None;
    }

    let morale = session
        .creatures
        .iter()
        .filter(|creature| !creature.is_remote())
        .map(|creature| creature.morale.clamp(0.0, 1.0))
        .sum::<f32>()
        / local as f32;
    let morale_recovering = session.creatures.iter().any(|creature| {
        !creature.is_remote() && (creature.morale < 0.995 || creature.morale_stress_for > 0.0)
    });
    if local <= capacity && !morale_recovering {
        return None;
    }

    if local <= capacity {
        return Some(format!(
            "Wellbeing recovering · morale {:.0}% · work speed returns",
            morale * 100.0
        ));
    }

    let work_penalty =
        ((session.overcrowding_ratio(data) - 1.0) * data.balance.overcrowding_work_penalty * 100.0)
            .clamp(0.0, 65.0);
    Some(format!(
        "Crowded · work −{work_penalty:.0}% · morale {:.0}% · tap Dig",
        morale * 100.0
    ))
}
