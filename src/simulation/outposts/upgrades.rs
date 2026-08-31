//! One-time route upgrades and their prerequisite ladder.

use crate::data::GameData;
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

/// Buy the one-time remote hold expansion for an active awakened route.
pub fn upgrade_outpost(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    if !route_upgrade_target_is_valid(session, pos) {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    if !session.outposts[index].active
        || session.outposts[index].storage_upgraded
        || session.economy.ingots_stock < data.balance.outpost_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_upgrade_ingots;
    session.outposts[index].upgrade_storage();
    true
}

/// Buy the one-time remote camp expansion for an active awakened route.
pub fn upgrade_outpost_crew(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    if !route_upgrade_target_is_valid(session, pos) {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    if !session.outposts[index].active
        || session.outposts[index].crew_upgraded
        || session.economy.ingots_stock < data.balance.outpost_crew_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_crew_upgrade_ingots;
    session.outposts[index].upgrade_crew_capacity();
    true
}

/// Install the one-time survey rig after the route's hold and camp are ready.
pub fn upgrade_outpost_survey(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    if !route_upgrade_target_is_valid(session, pos) {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    let outpost = &session.outposts[index];
    if !outpost.active
        || !outpost.storage_upgraded
        || !outpost.crew_upgraded
        || outpost.survey_upgraded
        || session.economy.ingots_stock < data.balance.outpost_survey_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_survey_upgrade_ingots;
    session.outposts[index].upgrade_survey();
    true
}

/// Install the one-time resonance beacon after the route's survey rig is ready.
pub fn upgrade_outpost_resonator(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    if !route_upgrade_target_is_valid(session, pos) {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    let outpost = &session.outposts[index];
    if !outpost.active
        || !outpost.survey_upgraded
        || outpost.resonator_upgraded
        || session.economy.ingots_stock < data.balance.outpost_resonator_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_resonator_upgrade_ingots;
    session.outposts[index].upgrade_resonator();
    true
}

/// Calibrate the one-time Charter-gated deep survey after the route's beacon
/// is ready, increasing the ore returned by each remote scout.
pub fn upgrade_outpost_deep_survey(
    session: &mut GameSession,
    data: &GameData,
    pos: TilePos,
) -> bool {
    if !route_upgrade_target_is_valid(session, pos) || !session.outpost_charter_claimed {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    let outpost = &session.outposts[index];
    if !outpost.active
        || !outpost.resonator_upgraded
        || outpost.deep_survey_upgraded
        || session.economy.ingots_stock < data.balance.outpost_deep_survey_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_deep_survey_upgrade_ingots;
    session.outposts[index].upgrade_deep_survey();
    true
}

/// Install the post-Relay Signal Cache, which adds a small ingot payload to
/// each later scouting haul after the full survey ladder is complete.
pub fn upgrade_outpost_signal_cache(
    session: &mut GameSession,
    data: &GameData,
    pos: TilePos,
) -> bool {
    if !route_upgrade_target_is_valid(session, pos)
        || !session.outpost_relay_claimed
        || data.balance.outpost_signal_cache_ingots_per_haul == 0
    {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = route_index(session, pos) else {
        return false;
    };
    let outpost = &session.outposts[index];
    if !outpost.active
        || !outpost.deep_survey_upgraded
        || outpost.signal_cache_upgraded
        || session.economy.ingots_stock < data.balance.outpost_signal_cache_upgrade_ingots
    {
        return false;
    }
    session.economy.ingots_stock -= data.balance.outpost_signal_cache_upgrade_ingots;
    session.outposts[index].upgrade_signal_cache();
    true
}

fn route_upgrade_target_is_valid(session: &GameSession, pos: TilePos) -> bool {
    session.worm_awake
        && session.worm_transit.is_none()
        && session
            .building_at(pos)
            .is_some_and(|building| building.kind == "outpost")
}

fn route_index(session: &GameSession, pos: TilePos) -> Option<usize> {
    session
        .outposts
        .iter()
        .position(|outpost| outpost.pos == pos)
}
