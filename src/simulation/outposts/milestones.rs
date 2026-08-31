use crate::data::GameData;
use crate::state::GameSession;

use super::specialists::wormsong_route_bonus;

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

/// Return the current progress for the repeatable multi-route Worm Road
/// Convoy after Relay has been claimed. Hauls are measured from the Relay
/// baseline so old saves and replayed routes remain deterministic.
pub fn outpost_convoy_progress(session: &GameSession, data: &GameData) -> (u32, u32) {
    let active_routes = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count() as u32;
    let goal = data.balance.outpost_convoy_haul_goal;
    let post_relay_hauls =
        total_expeditions(session).saturating_sub(data.balance.outpost_relay_haul_goal);
    let claimed_hauls = session.outpost_convoy_claims.saturating_mul(goal);
    (
        active_routes,
        post_relay_hauls.saturating_sub(claimed_hauls).min(goal),
    )
}

/// Reward every newly completed Convoy contract once Relay is live and the
/// network can sustain the required number of active routes.
pub fn claim_outpost_convoy(session: &mut GameSession, data: &GameData) -> u32 {
    let route_goal = data.balance.outpost_convoy_route_goal;
    let haul_goal = data.balance.outpost_convoy_haul_goal;
    if !session.worm_awake
        || !session.outpost_relay_claimed
        || route_goal == 0
        || haul_goal == 0
        || data.balance.outpost_convoy_reward_ingots == 0
    {
        return 0;
    }
    let (active_routes, _) = outpost_convoy_progress(session, data);
    if active_routes < route_goal {
        return 0;
    }
    let post_relay_hauls =
        total_expeditions(session).saturating_sub(data.balance.outpost_relay_haul_goal);
    let completed_contracts = post_relay_hauls / haul_goal;
    let contracts = completed_contracts.saturating_sub(session.outpost_convoy_claims);
    if contracts == 0 {
        return 0;
    }
    session.outpost_convoy_claims = session.outpost_convoy_claims.saturating_add(contracts);
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(contracts.saturating_mul(data.balance.outpost_convoy_reward_ingots));
    contracts
}

/// Return progress for the repeatable post-Convoy Worm Road Muster. The
/// first cleared Convoy establishes the baseline; later scouting hauls count
/// toward Muster pages even when another Convoy is cleared in parallel.
pub fn outpost_muster_progress(session: &GameSession, data: &GameData) -> (u32, u32) {
    let active_routes = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count() as u32;
    let goal = data.balance.outpost_muster_haul_goal;
    if !session.outpost_relay_claimed || session.outpost_convoy_claims == 0 || goal == 0 {
        return (active_routes, 0);
    }
    let post_convoy_hauls = total_expeditions(session)
        .saturating_sub(data.balance.outpost_relay_haul_goal)
        .saturating_sub(data.balance.outpost_convoy_haul_goal);
    let claimed_hauls = session.outpost_muster_claims.saturating_mul(goal);
    (
        active_routes,
        post_convoy_hauls.saturating_sub(claimed_hauls).min(goal),
    )
}

/// Reward every newly completed Worm Road Muster once the network can hold
/// the required number of active routes. Muster progress is intentionally
/// independent from the repeatable Convoy claim counter.
pub fn claim_outpost_muster(session: &mut GameSession, data: &GameData) -> u32 {
    let route_goal = data.balance.outpost_muster_route_goal;
    let haul_goal = data.balance.outpost_muster_haul_goal;
    if !session.worm_awake
        || !session.outpost_relay_claimed
        || session.outpost_convoy_claims == 0
        || route_goal == 0
        || haul_goal == 0
        || data.balance.outpost_muster_reward_ingots == 0
    {
        return 0;
    }
    let (active_routes, _) = outpost_muster_progress(session, data);
    if active_routes < route_goal {
        return 0;
    }
    let post_convoy_hauls = total_expeditions(session)
        .saturating_sub(data.balance.outpost_relay_haul_goal)
        .saturating_sub(data.balance.outpost_convoy_haul_goal);
    let completed_musters = post_convoy_hauls / haul_goal;
    let musters = completed_musters.saturating_sub(session.outpost_muster_claims);
    if musters == 0 {
        return 0;
    }
    session.outpost_muster_claims = session.outpost_muster_claims.saturating_add(musters);
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(musters.saturating_mul(data.balance.outpost_muster_reward_ingots));
    musters
}

/// Count distinct Wormsong roles currently stationed on active routes.
/// Duplicate kits add throughput, but a Concord needs one carrier, miner,
/// smith, and guard represented somewhere on the live Worm Road.
pub fn outpost_concord_progress(session: &GameSession, data: &GameData) -> (u32, u32) {
    let active_routes = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count() as u32;
    if !session.worm_awake || session.outpost_muster_claims == 0 {
        return (0, active_routes);
    }
    let mut roles = [false; 4];
    for outpost in session.outposts.iter().filter(|outpost| outpost.active) {
        let bonus = wormsong_route_bonus(session, data, outpost);
        roles[0] |= bonus.carriers > 0;
        roles[1] |= bonus.miners > 0;
        roles[2] |= bonus.smiths > 0;
        roles[3] |= bonus.guards > 0;
    }
    (
        roles.into_iter().filter(|present| *present).count() as u32,
        active_routes,
    )
}

/// Award the one-time Wormsong Concord once every remote specialist role is
/// represented on an active route after the Muster has been held.
pub fn claim_outpost_concord(session: &mut GameSession, data: &GameData) -> bool {
    if session.outpost_concord_claimed
        || !session.worm_awake
        || session.outpost_muster_claims == 0
        || data.balance.outpost_concord_role_goal == 0
        || data.balance.outpost_concord_reward_ingots == 0
    {
        return false;
    }
    let (roles, _) = outpost_concord_progress(session, data);
    if roles < data.balance.outpost_concord_role_goal {
        return false;
    }
    session.outpost_concord_claimed = true;
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(data.balance.outpost_concord_reward_ingots);
    true
}

/// Count active routes carrying a complete Wormsong crew for the one-time
/// Circuit contract. Unlike Concord's network-wide role count, each route
/// must hold all four voices at once.
pub fn outpost_circuit_progress(session: &GameSession, data: &GameData) -> (u32, u32) {
    let active_routes = session
        .outposts
        .iter()
        .filter(|outpost| outpost.active)
        .count() as u32;
    if !session.worm_awake || !session.outpost_concord_claimed {
        return (0, active_routes);
    }
    let complete_routes = session
        .outposts
        .iter()
        .filter(|outpost| {
            outpost.active && wormsong_route_bonus(session, data, outpost).has_all_roles()
        })
        .count() as u32;
    (complete_routes, active_routes)
}

/// Award the one-time Wormsong Circuit after two active routes each carry a
/// complete four-role crew.
pub fn claim_outpost_circuit(session: &mut GameSession, data: &GameData) -> bool {
    if session.outpost_circuit_claimed
        || !session.worm_awake
        || !session.outpost_concord_claimed
        || data.balance.outpost_circuit_route_goal < 2
        || data.balance.outpost_circuit_reward_ingots == 0
    {
        return false;
    }
    let (complete_routes, _) = outpost_circuit_progress(session, data);
    if complete_routes < data.balance.outpost_circuit_route_goal {
        return false;
    }
    session.outpost_circuit_claimed = true;
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(data.balance.outpost_circuit_reward_ingots);
    true
}
