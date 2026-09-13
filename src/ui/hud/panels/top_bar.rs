//! Compact and desktop top-bar status copy for food, raids, and workforce.

use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::GameSession;

/// Compact population readout for the quiet top-left corner of the map HUD.
/// Local room and idle workers are the two values that most often require a
/// decision; remote workers only appear after the Warren has outposts.
pub fn population_stats(session: &GameSession, data: &GameData) -> String {
    let local = session.local_creature_count();
    let capacity = session.local_warren_capacity(data);
    let idle = session.job_count(Job::Idle);
    let remote = session
        .creatures
        .iter()
        .filter(|creature| creature.is_remote())
        .count();
    if remote > 0 {
        format!("POP {local}/{capacity} · IDLE {idle} · REMOTE {remote}")
    } else {
        format!("POP {local}/{capacity} · IDLE {idle}")
    }
}

/// Keep a combined food/raid banner short while naming the visible Guard
/// control that resolves the incoming threat.
pub fn compact_raid_defense_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Guard) > 0 {
        "guards on watch".to_owned()
    } else if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Guard".to_owned()
    } else {
        [Job::Miner, Job::Carrier, Job::Cook, Job::Smith]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap − {}, then + Guard", job.label()))
            .unwrap_or_else(|| "free a worker, then + Guard".to_owned())
    }
}

/// Shorten the opening response enough to share the top bar with its buttons.
/// The full control names remain in the tutorial card beside the banner.
pub fn compact_food_recovery_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Carrier or Cook".to_owned()
    } else {
        [Job::Miner, Job::Smith, Job::Guard]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap − {}, then + Carrier", job.label()))
            .unwrap_or_else(|| "free a worker, then + Carrier".to_owned())
    }
}

/// Compress alert guidance when the fixed HUD is letterboxed into a narrow
/// browser canvas. The Jobs panel and tutorial card still carry the full
/// wording; these banners only need to identify the next visible control.
pub fn condensed_raid_defense_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Guard) > 0 {
        "guards ready".to_owned()
    } else if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Guard".to_owned()
    } else {
        [Job::Miner, Job::Carrier, Job::Cook, Job::Smith]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap −{}→+Guard", job.label()))
            .unwrap_or_else(|| "free worker→+Guard".to_owned())
    }
}

pub fn condensed_food_recovery_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Carrier/Cook".to_owned()
    } else {
        [Job::Miner, Job::Smith, Job::Guard]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap −{}→+Carrier", job.label()))
            .unwrap_or_else(|| "free worker→+Carrier".to_owned())
    }
}

pub fn compact_top_bar(ui_scale: f32) -> bool {
    ui_scale.is_finite() && ui_scale < 0.9
}

pub fn reassignable_job_count(session: &GameSession, data: &GameData, job: Job) -> usize {
    session
        .creatures
        .iter()
        .filter(|creature| {
            !creature.is_remote()
                && creature.job == job
                && data
                    .species
                    .get(&creature.species)
                    .is_some_and(|species| species.reassignable)
        })
        .count()
}
