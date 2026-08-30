//! Shared inspection status and route-state derivations.

use crate::data::GameData;
use crate::state::creatures::{Creature, Job, Task};
use crate::state::outposts::{Outpost, TransitDirection, WormTransit};
use crate::state::structures::Building;
use crate::state::GameSession;
use crate::ui::legibility::{advanced_systems_unlocked, BuildingStatus};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;

pub(super) fn local_mine_worker_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote() && matches!(&creature.task, Task::WorkMine(p) if *p == pos)
}

pub(super) fn mine_staffing_label(staffed: usize, slots: u32, deposit_exhausted: bool) -> String {
    if deposit_exhausted {
        "Deposit exhausted".to_owned()
    } else if staffed == 0 {
        "No mine worker — stopped".to_owned()
    } else {
        format!("Mine staff {staffed}/{slots}")
    }
}

pub(super) fn waste_inspection_hint(
    session: &GameSession,
    data: &GameData,
    building: &Building,
) -> String {
    let janitor_present = session
        .creatures
        .iter()
        .any(|c| !c.is_remote() && c.species == "slime_janitor");
    let action = if janitor_present {
        "Slime Janitor cleans it".to_owned()
    } else if !advanced_systems_unlocked(session) {
        "Secure warren first".to_owned()
    } else if session.unlocked.contains("slime_janitor") {
        "Attract Slime".to_owned()
    } else if let Some(unlock) = data
        .unlocks
        .iter()
        .find(|candidate| candidate.id == "slime_janitor")
    {
        format!("Spoil {} food to unlock Slime", unlock.threshold)
    } else {
        "Unlock Slime Janitor".to_owned()
    };
    format!("Waste {:.1} · {action}", building.waste)
}

pub(super) fn local_mine_staffed_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote()
        && (creature.job == Job::Miner || creature.job == Job::Engineer)
        && match &creature.task {
            Task::WorkMine(p) | Task::GoMine(p) => *p == pos,
            _ => creature.tile() == pos,
        }
}

pub(super) fn outpost_return_label(cargo: u32, crew: usize) -> String {
    match (cargo > 0, crew > 0) {
        (true, true) => format!("Send {cargo} cargo + {crew} crew to shrine"),
        (true, false) => format!("Send {cargo} cargo to shrine"),
        (false, true) => format!("Send {crew} crew to shrine"),
        (false, false) => "No cargo or crew to return".to_owned(),
    }
}

pub(super) fn outpost_cargo_only_return_label(cargo: u32) -> String {
    format!("Send {cargo} cargo · keep crew")
}

pub(super) fn outpost_load_hint(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
) -> Option<String> {
    let food_available = (session.economy.food - data.balance.worm_feed_reserve)
        .max(0.0)
        .floor() as u32;
    let load = crate::simulation::outposts::preview_outbound_cargo(
        outpost,
        crate::simulation::outposts::storage_capacity(outpost, data),
        session.economy.ore_stock,
        session.economy.ingots_stock,
        food_available,
    );
    if load.total() > 0 {
        Some(format!(
            "Load · Ore {} · Ingots {} · Food {}",
            load.ore, load.ingots, load.food
        ))
    } else if outpost.cargo_total() >= crate::simulation::outposts::storage_capacity(outpost, data)
    {
        Some("Hold full · return to shrine".to_owned())
    } else {
        None
    }
}

pub(super) fn outpost_expedition_hint(data: &GameData, outpost: &Outpost) -> Option<String> {
    match crate::simulation::outposts::expedition_state(outpost, data) {
        crate::simulation::outposts::ExpeditionState::Inactive => None,
        crate::simulation::outposts::ExpeditionState::NoCrew => Some("Need scout crew".to_owned()),
        crate::simulation::outposts::ExpeditionState::Paused => {
            Some("Expedition paused · player paused".to_owned())
        }
        crate::simulation::outposts::ExpeditionState::HoldFull => {
            Some("Expedition paused · hold full".to_owned())
        }
        crate::simulation::outposts::ExpeditionState::NeedsFood {
            required,
            available,
        } => Some(format!(
            "Expedition paused · need {} food",
            required.saturating_sub(available)
        )),
        crate::simulation::outposts::ExpeditionState::Scouting {
            progress_percent,
            ore_yield,
            food_cost,
        } => Some(format!(
            "Expedition {progress_percent}% · +{ore_yield} ore / -{food_cost} food"
        )),
    }
}

/// Give every inspected building the same first-read answer: is it working,
/// stalled, paused, or on a route that needs attention?
pub(super) fn inspect_status(
    session: &GameSession,
    data: &GameData,
    building: &Building,
) -> (&'static str, Color) {
    if building.kind == "worm_shrine" {
        return worm_shrine_status(session, data);
    }
    if building.kind == "outpost" {
        if let Some(outpost) = session.outposts.iter().find(|o| o.pos == building.pos) {
            if session
                .worm_transit
                .as_ref()
                .is_some_and(|transit| transit.outpost == building.pos)
            {
                return ("In transit", dark::POSITIVE);
            }
            if outpost.last_failure.is_some() {
                return ("Route failed", dark::NEGATIVE);
            }
            if !outpost.active {
                if outpost.cargo_total() > 0 || !outpost.crew.is_empty() {
                    return ("Payload held · route inactive", dark::WARNING);
                }
                return ("Route inactive", dark::WARNING);
            }
            if session.worm_awake {
                match crate::simulation::outposts::expedition_state(outpost, data) {
                    crate::simulation::outposts::ExpeditionState::Paused => {
                        return ("Scouting paused", dark::WARNING);
                    }
                    crate::simulation::outposts::ExpeditionState::NeedsFood { .. } => {
                        return ("Needs scout food", dark::WARNING);
                    }
                    crate::simulation::outposts::ExpeditionState::HoldFull => {
                        return ("Outpost hold full", dark::NEGATIVE);
                    }
                    crate::simulation::outposts::ExpeditionState::Inactive
                    | crate::simulation::outposts::ExpeditionState::NoCrew
                    | crate::simulation::outposts::ExpeditionState::Scouting { .. } => {}
                }
            }
            if !session.worm_awake {
                return ("Route active", dark::POSITIVE);
            }
            if outpost.cargo_total() > 0 || !outpost.crew.is_empty() {
                return ("Payload ready", dark::POSITIVE);
            }
            if outpost_has_loadable_payload(
                session,
                data,
                0,
                0,
                crate::simulation::outposts::storage_capacity(outpost, data),
                outpost.crew_dispatch_limit,
            ) {
                return ("Ready to load", dark::POSITIVE);
            } else {
                return ("Awaiting payload", dark::WARNING);
            }
        }
    }
    match crate::ui::legibility::building_status(session, data, building) {
        Some(status) => (
            status.label(),
            match status {
                BuildingStatus::Exhausted | BuildingStatus::OutputFull => dark::NEGATIVE,
                _ => dark::WARNING,
            },
        ),
        None => ("Working", dark::POSITIVE),
    }
}

fn worm_shrine_status(session: &GameSession, data: &GameData) -> (&'static str, Color) {
    if session.worm_awake {
        ("Awakened", dark::POSITIVE)
    } else if session.worm_feeding_paused {
        ("Paused — reserve protected", dark::WARNING)
    } else if worm_waiting_for_food(session, data) {
        ("Waiting for food reserve", dark::WARNING)
    } else if worm_waiting_for_ingots(session, data) {
        ("Waiting for ingot reserve", dark::WARNING)
    } else {
        ("Working", dark::POSITIVE)
    }
}

pub(super) fn transit_destination(direction: TransitDirection) -> &'static str {
    match direction {
        TransitDirection::ToOutpost => "outpost",
        TransitDirection::ToShrine => "shrine",
    }
}

pub(super) fn transit_payload_line(transit: &WormTransit) -> String {
    let cargo = transit
        .ore
        .saturating_add(transit.ingots)
        .saturating_add(transit.food.max(0.0) as u32);
    format!(
        "In flight · {cargo} cargo · {} crew",
        transit.passengers.len()
    )
}

pub(super) fn outpost_has_loadable_payload(
    session: &GameSession,
    data: &GameData,
    cargo: u32,
    crew: usize,
    capacity: u32,
    crew_dispatch_limit: Option<u32>,
) -> bool {
    let crew_ready = crew < data.balance.outpost_capacity as usize
        && crew_dispatch_limit != Some(0)
        && session
            .creatures
            .iter()
            .any(|c| !c.is_remote() && c.carrying.is_none() && c.tile() == session.stockpile_pos());
    outpost_has_loadable_cargo(session, data, cargo, capacity) || crew_ready
}

fn outpost_has_loadable_cargo(
    session: &GameSession,
    data: &GameData,
    cargo: u32,
    capacity: u32,
) -> bool {
    if cargo >= capacity {
        return false;
    }
    session.economy.ore_stock > 0
        || session.economy.ingots_stock > 0
        || session.economy.food - data.balance.worm_feed_reserve >= 1.0
}

pub(super) fn worm_waiting_for_food(session: &GameSession, data: &GameData) -> bool {
    session.worm_fed < data.balance.worm_awaken_at
        && session.economy.food <= data.balance.worm_feed_reserve
}

pub(super) fn worm_waiting_for_ingots(session: &GameSession, data: &GameData) -> bool {
    if session.worm_fed >= data.balance.worm_awaken_at {
        return session.worm_ingots_fed < data.balance.worm_awaken_ingots
            && session.economy.ingots_stock <= data.balance.worm_ingot_reserve;
    }
    let food_per_offering = if data.balance.worm_food_per_offering > 0.0 {
        data.balance.worm_food_per_offering
    } else {
        data.balance.worm_awaken_at / data.balance.worm_awaken_ingots.max(1) as f32
    };
    let completed_offerings = (session.worm_fed / food_per_offering.max(1.0)).floor() as u32;
    let desired_ingots = completed_offerings * data.balance.worm_ingots_per_offering;
    session.worm_ingots_fed < desired_ingots
        && session.economy.ingots_stock <= data.balance.worm_ingot_reserve
}
