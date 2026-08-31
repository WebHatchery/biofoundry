//! Capture scene for the one-time four-role Wormsong Concord.

use super::super::super::Game;
use super::remote_specialists;
use crate::state::GameState;

pub(super) fn begin(game: &mut Game) {
    remote_specialists::begin(game);
    game.routes_open = true;
    if let GameState::Warren(session) = &mut game.state {
        session.outpost_concord_claimed = true;
        session.economy.ingots_stock = game.data.balance.outpost_concord_reward_ingots;
        game.notifications.success(format!(
            "Wormsong Concord · +{} ingots · all four roles on the road.",
            game.data.balance.outpost_concord_reward_ingots
        ));
    }
}
