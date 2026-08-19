//! Engineers are dedicated workstation specialists. They automatically take
//! an open Mine post, keeping the bred role useful without manual reassignment.

use super::miner::{tick_miner, MineClaims};
use super::routing::send_to;
use crate::data::GameData;
use crate::state::creatures::{Creature, Task};
use crate::state::GameSession;

pub(super) fn tick_engineer(
    creature: &mut Creature,
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
    claims: &mut MineClaims,
    work_boost: f32,
) {
    if creature.task == Task::Idle {
        if let Some(mine) = session
            .buildings_of("mine")
            .filter(|b| b.reserve > 0.0)
            .min_by_key(|b| (b.pos.manhattan_distance(&creature.tile()), b.pos.x, b.pos.y))
            .map(|b| b.pos)
        {
            send_to(creature, session, mine, Task::GoMine(mine));
            return;
        }
    }
    tick_miner(creature, session, data, dt, claims, work_boost);
}
