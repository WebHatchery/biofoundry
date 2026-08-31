//! Capture scenes for repeatable Wormsong Encore rewards.

use super::super::super::Game;
use super::wormsong_circuit;
use crate::simulation;
use crate::state::GameState;

pub(super) fn begin(game: &mut Game) {
    wormsong_circuit::begin(game);
    game.notifications.clear();
    game.routes_open = true;
    game.paused = true;
    if let GameState::Warren(session) = &mut game.state {
        session.outpost_circuit_claimed = true;
        session.outpost_concord_hauls =
            game.data.balance.outpost_encore_haul_goal.saturating_sub(1);
        session.outpost_encore_claims = 0;
        session.economy.ingots_stock = 0;
    }
}

pub(super) fn award(game: &mut Game) {
    begin(game);
    if let GameState::Warren(session) = &mut game.state {
        session.outpost_concord_hauls = session.outpost_concord_hauls.saturating_add(1);
        let awarded = simulation::outposts::claim_outpost_encore(session, &game.data);
        if awarded > 0 {
            game.notifications.success(format!(
                "Wormsong Encore · +{} ingots · boosted haul replayed.",
                awarded.saturating_mul(game.data.balance.outpost_encore_reward_ingots)
            ));
        }
    }
}
