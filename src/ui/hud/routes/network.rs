//! Compact network-summary presentation for the route ledger.

use super::{charter_summary, route_needs_attention};
use crate::data::GameData;
use crate::state::GameSession;

pub(super) fn compact_route_network_summary(
    session: &GameSession,
    data: &GameData,
) -> (String, String) {
    let active = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count();
    let held_cargo = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.cargo_total())
    });
    let remote_crew = session.outposts.iter().fold(0usize, |total, outpost| {
        total.saturating_add(outpost.crew.len())
    });
    let scouted_ore = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.ore_scouted)
    });
    let attention = session
        .outposts
        .iter()
        .filter(|outpost| route_needs_attention(session, outpost, data))
        .count();
    let cached_ingots = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.signal_cache_ingots)
    });
    let chorus_ingots = session.outposts.iter().fold(0u32, |total, outpost| {
        total.saturating_add(outpost.chorus_ingots)
    });
    let mut earnings = Vec::new();
    if cached_ingots > 0 {
        earnings.push(format!("Cache {cached_ingots}"));
    }
    if chorus_ingots > 0 {
        earnings.push(format!("Chorus {chorus_ingots}"));
    }
    let detail = [charter_summary(session, data), earnings.join(" · ")]
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    (
        format!(
            "Routes {} · Active {} · Cargo {} · Crew {} · Ore {} · Attention {}",
            session.outposts.len(),
            active,
            held_cargo,
            remote_crew,
            scouted_ore,
            attention
        ),
        detail,
    )
}
