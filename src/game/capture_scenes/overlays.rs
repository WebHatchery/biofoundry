//! Capture scenes for modal help and recovery surfaces.

use crate::game::Game;
use crate::state::{GameState, StateTransition};

pub(super) fn help(game: &mut Game) {
    game.transition(StateTransition::StartWarren);
    game.help_open = true;
    if let GameState::Warren(session) = &mut game.state {
        session.tutorial_dismissed = true;
    }
}

pub(super) fn event_log(game: &mut Game) {
    help(game);
    game.event_log_open = true;
    game.notifications.success("The food grid is stable again.");
    game.notifications
        .info("Iron Pickaxe queued at the Blacksmith.");
    game.notifications
        .warning("Food is low — tap + Carrier or Cook.");
    game.notifications.danger(
        "The worm route failed — tap the outpost, then reactivate the route before trying again.",
    );
    game.notifications
        .success("The Colossal Worm awakens — the endless route is open.");
}

pub(super) fn event_log_older(game: &mut Game) {
    event_log(game);
    for notice in [
        "Raid held — the warren still stands.",
        "The Mine is waiting for a carrier.",
        "Farm construction completed.",
        "The food forecast is recovering.",
        "A new route note was saved.",
        "The Blacksmith needs a Smith.",
        "A captured beetle was housed.",
    ] {
        game.notifications.info(notice);
    }
    game.event_log_page = 1;
}

pub(super) fn pause(game: &mut Game) {
    game.transition(StateTransition::StartWarren);
    game.paused = true;
    if let GameState::Warren(session) = &mut game.state {
        session.tutorial_dismissed = true;
        session.economy.food = 72.0;
    }
}

pub(super) fn collapse(game: &mut Game) {
    game.transition(StateTransition::StartWarren);
    if let GameState::Warren(session) = &mut game.state {
        session.tutorial_dismissed = true;
        session.creatures.clear();
    }
}
