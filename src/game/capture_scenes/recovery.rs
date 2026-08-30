//! Capture fixtures for recovery states that are difficult to reach in a new game.

use super::Game;
use crate::state::structures::Building;
use crate::state::world::Tile;
use crate::state::{GameState, StateTransition};

/// Stage a pre-guard workstation in an isolated floor pocket.
pub(super) fn unreachable_workstation(game: &mut Game) {
    game.transition(StateTransition::StartWarren);
    if let GameState::Warren(session) = &mut game.state {
        session.tutorial_dismissed = true;
        let stockpile = session.stockpile_pos();
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                **tile == Tile::Floor
                    && *pos != stockpile
                    && pos.manhattan_distance(&stockpile) >= 3
                    && session.building_at(*pos).is_none()
            })
            .map(|(pos, _)| pos)
            .expect("a free workstation location");

        for neighbor in pos.neighbors_4way() {
            session.world.tiles.set(neighbor, Tile::Rock);
        }
        session.buildings.push(Building::new("cook_pot", pos));
        game.selected_building = Some(pos);
    }
}
