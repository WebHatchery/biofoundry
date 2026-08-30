//! Engineers are dedicated workstation specialists. They automatically take
//! an open Mine post, keeping the bred role useful without manual reassignment.

use super::miner::{nearest_open_mine, tick_miner, MineClaims};
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
        if let Some(mine) = nearest_open_mine(creature, session, data, claims) {
            send_to(creature, session, mine, Task::GoMine(mine));
            if matches!(creature.task, Task::GoMine(target) if target == mine) {
                // Reserve the slot immediately so later workers in this tick
                // cannot choose the same post before the engineer arrives.
                *claims.entry(mine).or_insert(0) += 1;
                return;
            }
        }
    }
    tick_miner(creature, session, data, dt, claims, work_boost);
}
