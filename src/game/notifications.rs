//! Player-facing text for simulation events.

use crate::audio::{Audio, Sfx};
use crate::data::GameData;
use crate::simulation::TickReport;
use crate::state::outposts::ExpeditionCompletion;
use macroquad_toolkit::notifications::NotificationManager;

/// Announce route milestones and return whether the tick should be checkpointed.
pub(super) fn announce_outpost_milestones(
    report: &TickReport,
    data: &GameData,
    notifications: &mut NotificationManager,
    audio: &mut Audio,
) -> bool {
    let mut awarded = false;
    if report.outpost_charter_awarded {
        awarded = true;
        notifications.success(format!(
            "Worm Road Charter · +{} ingots · Wormbone Drill unlocked.",
            data.balance.outpost_charter_reward_ingots,
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_archive_awarded > 0 {
        awarded = true;
        let reward = report
            .outpost_archive_awarded
            .saturating_mul(data.balance.outpost_archive_reward_ingots);
        notifications.success(format!(
            "Worm Road Archive · +{} ingots · {} page{} logged.",
            reward,
            report.outpost_archive_awarded,
            if report.outpost_archive_awarded == 1 {
                ""
            } else {
                "s"
            }
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_relay_awarded {
        awarded = true;
        notifications.success(format!(
            "Worm Road Relay · +{} ingots · twin routes linked.",
            data.balance.outpost_relay_reward_ingots,
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_convoy_awarded > 0 {
        awarded = true;
        let reward = report
            .outpost_convoy_awarded
            .saturating_mul(data.balance.outpost_convoy_reward_ingots);
        notifications.success(format!(
            "Worm Road Convoy · +{} ingots · {} contract{} cleared.",
            reward,
            report.outpost_convoy_awarded,
            if report.outpost_convoy_awarded == 1 {
                ""
            } else {
                "s"
            }
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_muster_awarded > 0 {
        awarded = true;
        let reward = report
            .outpost_muster_awarded
            .saturating_mul(data.balance.outpost_muster_reward_ingots);
        notifications.success(format!(
            "Worm Road Muster · +{} ingots · {} network muster{} held.",
            reward,
            report.outpost_muster_awarded,
            if report.outpost_muster_awarded == 1 {
                ""
            } else {
                "s"
            }
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_concord_awarded {
        awarded = true;
        notifications.success(format!(
            "Wormsong Concord · +{} ingots · all four roles on the road.",
            data.balance.outpost_concord_reward_ingots,
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_circuit_awarded {
        awarded = true;
        notifications.success(format!(
            "Wormsong Circuit · +{} ingots · two complete crews linked.",
            data.balance.outpost_circuit_reward_ingots,
        ));
        audio.play(Sfx::Complete);
    }
    if report.outpost_encore_awarded > 0 {
        awarded = true;
        let reward = report
            .outpost_encore_awarded
            .saturating_mul(data.balance.outpost_encore_reward_ingots);
        notifications.success(format!(
            "Wormsong Encore · +{} ingots · {} boosted haul{} replayed.",
            reward,
            report.outpost_encore_awarded,
            if report.outpost_encore_awarded == 1 {
                ""
            } else {
                "s"
            }
        ));
        audio.play(Sfx::Complete);
    }
    awarded
}

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
