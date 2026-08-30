//! Capture-only scenes for the post-campaign Outpost hold upgrade.

use super::super::Game;
use crate::simulation;
use crate::state::GameState;

pub(super) fn begin(game: &mut Game, scene: &str) {
    match scene {
        "endless_auto_return" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_return_cargo = true;
                }
            }
        }
        "endless_upgrade" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_upgrade_ingots;
            }
        }
        "endless_upgraded" => {
            begin(game, "endless_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Outpost hold expanded to {} slots.",
                    game.data.balance.outpost_upgraded_storage_cap
                ));
            }
        }
        _ => {}
    }
}
