//! Repeatable rewards for keeping a complete Wormsong route in motion.

use super::specialists::route_concord_active;
use crate::data::GameData;
use crate::state::outposts::Outpost;
use crate::state::GameSession;

/// Record one haul completed by a route whose Concord crew is actively
/// singing. The counter is separate from ordinary expedition totals so an
/// Encore cannot be earned from pre-Concord work.
pub fn record_concord_haul(session: &mut GameSession, data: &GameData, outpost: &Outpost) {
    if route_concord_active(session, data, outpost) {
        session.outpost_concord_hauls = session.outpost_concord_hauls.saturating_add(1);
    }
}

/// Return progress toward the next repeatable Wormsong Encore.
pub fn outpost_encore_progress(session: &GameSession, data: &GameData) -> u32 {
    let goal = data.balance.outpost_encore_haul_goal;
    if !session.outpost_circuit_claimed || goal == 0 {
        return 0;
    }
    let claimed_hauls = session.outpost_encore_claims.saturating_mul(goal);
    session
        .outpost_concord_hauls
        .saturating_sub(claimed_hauls)
        .min(goal)
}

/// Reward every newly completed Encore after the Wormsong Circuit is live.
pub fn claim_outpost_encore(session: &mut GameSession, data: &GameData) -> u32 {
    let goal = data.balance.outpost_encore_haul_goal;
    if !session.worm_awake
        || !session.outpost_circuit_claimed
        || goal == 0
        || data.balance.outpost_encore_reward_ingots == 0
    {
        return 0;
    }
    let completed = session.outpost_concord_hauls / goal;
    let encores = completed.saturating_sub(session.outpost_encore_claims);
    if encores == 0 {
        return 0;
    }
    session.outpost_encore_claims = session.outpost_encore_claims.saturating_add(encores);
    session.economy.ingots_stock = session
        .economy
        .ingots_stock
        .saturating_add(encores.saturating_mul(data.balance.outpost_encore_reward_ingots));
    encores
}
