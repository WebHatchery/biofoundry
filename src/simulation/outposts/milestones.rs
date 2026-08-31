use crate::data::GameData;
use crate::state::GameSession;

/// Return the aggregate completed scouting hauls across all routes.
pub fn total_expeditions(session: &GameSession) -> u32 {
    session.outposts.iter().fold(0, |total, outpost| {
        total.saturating_add(outpost.expeditions_completed)
    })
}

/// Award the one-time Worm Road Charter when its haul target is met.
pub fn claim_outpost_charter(session: &mut GameSession, data: &GameData) -> bool {
    if session.outpost_charter_claimed
        || data.balance.outpost_charter_haul_goal == 0
        || total_expeditions(session) < data.balance.outpost_charter_haul_goal
    {
        return false;
    }
    session.outpost_charter_claimed = true;
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(data.balance.outpost_charter_reward_ingots);
    true
}

/// Return the current progress toward the next repeatable Worm Road Archive
/// page after the one-time Charter milestone has been claimed.
pub fn outpost_archive_progress(session: &GameSession, data: &GameData) -> u32 {
    let goal = data.balance.outpost_archive_haul_goal;
    if !session.outpost_charter_claimed || goal == 0 {
        return 0;
    }
    let post_charter_hauls =
        total_expeditions(session).saturating_sub(data.balance.outpost_charter_haul_goal);
    let claimed_hauls = session.outpost_archive_claims.saturating_mul(goal);
    post_charter_hauls.saturating_sub(claimed_hauls).min(goal)
}

/// Reward every newly completed repeatable Archive page after the Charter.
/// Claiming all outstanding pages in one tick keeps saves and long hitches
/// deterministic without dropping progress.
pub fn claim_outpost_archive(session: &mut GameSession, data: &GameData) -> u32 {
    let goal = data.balance.outpost_archive_haul_goal;
    if !session.outpost_charter_claimed
        || goal == 0
        || data.balance.outpost_archive_reward_ingots == 0
    {
        return 0;
    }
    let post_charter_hauls =
        total_expeditions(session).saturating_sub(data.balance.outpost_charter_haul_goal);
    let completed_pages = post_charter_hauls / goal;
    let pages = completed_pages.saturating_sub(session.outpost_archive_claims);
    if pages == 0 {
        return 0;
    }
    session.outpost_archive_claims = session.outpost_archive_claims.saturating_add(pages);
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(pages.saturating_mul(data.balance.outpost_archive_reward_ingots));
    pages
}

/// Return the live progress for the one-time multi-route Worm Road Relay.
/// The contract only appears after the first Archive page has been claimed.
pub fn outpost_relay_progress(session: &GameSession) -> (u32, u32) {
    let active_routes = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count() as u32;
    let completed_hauls = total_expeditions(session);
    (active_routes, completed_hauls)
}

/// Award the one-time Relay contract once the first Archive page has opened
/// the route ledger and both route-count and haul-count goals are met.
pub fn claim_outpost_relay(session: &mut GameSession, data: &GameData) -> bool {
    if session.outpost_relay_claimed
        || !session.worm_awake
        || session.outpost_archive_claims == 0
        || data.balance.outpost_relay_route_goal == 0
        || data.balance.outpost_relay_haul_goal == 0
        || data.balance.outpost_relay_reward_ingots == 0
    {
        return false;
    }
    let (active_routes, completed_hauls) = outpost_relay_progress(session);
    if active_routes < data.balance.outpost_relay_route_goal
        || completed_hauls < data.balance.outpost_relay_haul_goal
    {
        return false;
    }
    session.outpost_relay_claimed = true;
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(data.balance.outpost_relay_reward_ingots);
    true
}
