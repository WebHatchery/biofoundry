//! Worm Shrine route validation and fixed-time cargo/crew transit.

use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::outposts::{
    CargoPriority, Outpost, TransitCompletion, TransitDirection, WormTransit,
};
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

pub fn activate_outpost(session: &mut GameSession, pos: TilePos) -> bool {
    if session.buildings_of("worm_shrine").next().is_none()
        || session.building_at(pos).is_none_or(|b| b.kind != "outpost")
    {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(outpost) = session.outposts.iter_mut().find(|o| o.pos == pos) else {
        return false;
    };
    let was_inactive = !outpost.active;
    outpost.active = !outpost.active;
    outpost.last_failure = None;
    if was_inactive {
        // The player has acknowledged the failed route and reopened it.
        // Clear the global banner too, otherwise the top bar reports a stale
        // failure until a new transit happens to launch.
        session.last_transit_failure = None;
    }
    true
}

pub fn start_to_outpost(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(session, data, pos, TransitDirection::ToOutpost)
}

pub fn start_to_shrine(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(session, data, pos, TransitDirection::ToShrine)
}

/// The cargo that the next outbound run will put in an Outpost hold.
///
/// This is intentionally a small value type shared by the route action and
/// its inspection preview, so the player sees the same result the simulation
/// will execute after tapping the load button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CargoLoad {
    pub ore: u32,
    pub ingots: u32,
    pub food: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpeditionState {
    Inactive,
    NoCrew,
    Paused,
    HoldFull,
    NeedsFood {
        required: u32,
        available: u32,
    },
    Scouting {
        progress_percent: u32,
        ore_yield: u32,
        food_cost: u32,
    },
}

impl CargoLoad {
    pub fn total(self) -> u32 {
        self.ore
            .saturating_add(self.ingots)
            .saturating_add(self.food)
    }
}

pub fn preview_outbound_cargo(
    outpost: &Outpost,
    capacity: u32,
    ore_available: u32,
    ingots_available: u32,
    food_available: u32,
) -> CargoLoad {
    let mut room = capacity.saturating_sub(outpost.cargo_total());
    let mut load = CargoLoad {
        ore: 0,
        ingots: 0,
        food: 0,
    };
    for kind in priority_order(outpost.cargo_priority) {
        let available = match kind {
            CargoKind::Ore => ore_available,
            CargoKind::Ingots => ingots_available,
            CargoKind::Food => food_available,
        };
        let take = available.min(room);
        match kind {
            CargoKind::Ore => load.ore = take,
            CargoKind::Ingots => load.ingots = take,
            CargoKind::Food => load.food = take,
        }
        room -= take;
    }
    load
}

pub fn expedition_state(outpost: &Outpost, data: &GameData) -> ExpeditionState {
    if !outpost.active {
        return ExpeditionState::Inactive;
    }
    let crew = outpost.crew.len() as u32;
    if crew == 0 {
        return ExpeditionState::NoCrew;
    }
    if outpost.expedition_paused {
        return ExpeditionState::Paused;
    }
    if outpost.cargo_total() >= data.balance.outpost_storage_cap {
        return ExpeditionState::HoldFull;
    }
    let food_required = crew.saturating_mul(data.balance.outpost_expedition_food_per_crew);
    let food_available = outpost.cargo.get(&Good::CookedFood).copied().unwrap_or(0);
    if food_available < food_required {
        return ExpeditionState::NeedsFood {
            required: food_required,
            available: food_available,
        };
    }
    let cycle = data.balance.outpost_expedition_cycle_sec.max(0.1);
    let progress_percent = (outpost.expedition_progress.max(0.0) / cycle * 100.0)
        .floor()
        .clamp(0.0, 100.0) as u32;
    ExpeditionState::Scouting {
        progress_percent,
        ore_yield: crew.saturating_mul(data.balance.outpost_expedition_ore_per_crew),
        food_cost: food_required,
    }
}

pub fn tick_expeditions(session: &mut GameSession, data: &GameData, dt: f32) {
    if !session.worm_awake {
        return;
    }
    let cycle = data.balance.outpost_expedition_cycle_sec.max(0.1);
    let food_per_crew = data.balance.outpost_expedition_food_per_crew;
    let ore_per_crew = data.balance.outpost_expedition_ore_per_crew;
    for outpost in &mut session.outposts {
        if !outpost.active || outpost.expedition_paused || outpost.crew.is_empty() {
            continue;
        }
        let crew = outpost.crew.len() as u32;
        let room = data
            .balance
            .outpost_storage_cap
            .saturating_sub(outpost.cargo_total());
        let food_cost = crew.saturating_mul(food_per_crew);
        let food_available = outpost.cargo.get(&Good::CookedFood).copied().unwrap_or(0);
        if room == 0 || food_available < food_cost {
            continue;
        }
        outpost.expedition_progress = (outpost.expedition_progress.max(0.0) + dt).min(cycle);
        if outpost.expedition_progress < cycle {
            continue;
        }
        take_cargo(outpost, Good::CookedFood, food_cost);
        add_cargo(
            outpost,
            Good::Ore,
            crew.saturating_mul(ore_per_crew).min(room),
        );
        outpost.expedition_progress -= cycle;
    }
}

fn start_transit(
    session: &mut GameSession,
    data: &GameData,
    pos: TilePos,
    direction: TransitDirection,
) -> bool {
    if !session.worm_awake
        || session.worm_transit.is_some()
        || session.buildings_of("worm_shrine").next().is_none()
    {
        return false;
    }
    let Some(outpost) = session.outposts.iter().find(|o| o.pos == pos) else {
        return false;
    };
    if !outpost.active {
        return false;
    }
    let cap = data.balance.outpost_storage_cap;
    let (ore, ingots, food) = match direction {
        TransitDirection::ToOutpost => {
            // Remote cargo is stored in whole units; leave any fractional
            // food behind instead of charging it and truncating it at arrival.
            let food_available = (session.economy.food - data.balance.worm_feed_reserve)
                .max(0.0)
                .floor() as u32;
            let load = preview_outbound_cargo(
                outpost,
                cap,
                session.economy.ore_stock,
                session.economy.ingots_stock,
                food_available,
            );
            (load.ore, load.ingots, load.food as f32)
        }
        TransitDirection::ToShrine => (
            *outpost.cargo.get(&Good::Ore).unwrap_or(&0),
            *outpost.cargo.get(&Good::Ingot).unwrap_or(&0),
            *outpost.cargo.get(&Good::CookedFood).unwrap_or(&0) as f32,
        ),
    };
    let passengers = match direction {
        TransitDirection::ToOutpost => session
            .creatures
            .iter()
            .filter(|c| {
                !c.is_remote() && c.carrying.is_none() && c.tile() == session.stockpile_pos()
            })
            .take(
                data.balance
                    .outpost_capacity
                    .saturating_sub(outpost.crew.len() as u32) as usize,
            )
            .map(|c| c.id)
            .collect(),
        TransitDirection::ToShrine => outpost.crew.clone(),
    };
    if ore == 0 && ingots == 0 && food <= 0.0 && passengers.is_empty() {
        return false;
    }

    match direction {
        TransitDirection::ToOutpost => {
            session.economy.ore_stock -= ore;
            session.economy.ingots_stock -= ingots;
            session.economy.food -= food;
        }
        TransitDirection::ToShrine => {
            if let Some(o) = session.outposts.iter_mut().find(|o| o.pos == pos) {
                take_cargo(o, Good::Ore, ore);
                take_cargo(o, Good::Ingot, ingots);
                take_cargo(o, Good::CookedFood, food as u32);
                o.crew.clear();
            }
        }
    }
    for creature in &mut session.creatures {
        if passengers.contains(&creature.id) {
            creature.remote_outpost = Some(pos);
            creature.clear_task();
        }
    }
    session.worm_transit = Some(WormTransit {
        outpost: pos,
        direction,
        remaining: data.balance.worm_transit_time_sec,
        ore,
        ingots,
        food,
        passengers,
    });
    session.last_transit_failure = None;
    true
}

#[derive(Clone, Copy)]
enum CargoKind {
    Ore,
    Ingots,
    Food,
}

fn priority_order(priority: CargoPriority) -> [CargoKind; 3] {
    match priority {
        CargoPriority::Ore => [CargoKind::Ore, CargoKind::Ingots, CargoKind::Food],
        CargoPriority::Ingots => [CargoKind::Ingots, CargoKind::Ore, CargoKind::Food],
        CargoPriority::Food => [CargoKind::Food, CargoKind::Ore, CargoKind::Ingots],
    }
}

pub fn tick_transit(
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
) -> Option<TransitCompletion> {
    let mut transit = session.worm_transit.take()?;
    if !session
        .outposts
        .iter()
        .any(|o| o.pos == transit.outpost && o.active)
    {
        recover_failed_transit(session, transit);
        return None;
    }
    transit.remaining -= dt;
    if transit.remaining > 0.0 {
        session.worm_transit = Some(transit);
        return None;
    }
    let target = transit.outpost;
    let direction = transit.direction;
    match transit.direction {
        TransitDirection::ToOutpost => {
            if let Some(outpost) = session.outposts.iter_mut().find(|o| o.pos == target) {
                add_cargo(outpost, Good::Ore, transit.ore);
                add_cargo(outpost, Good::Ingot, transit.ingots);
                add_cargo(outpost, Good::CookedFood, transit.food as u32);
                outpost.crew.extend(transit.passengers.iter().copied());
            }
            for creature in &mut session.creatures {
                if transit.passengers.contains(&creature.id) {
                    creature.x = target.x as f32 + 0.5;
                    creature.y = target.y as f32 + 0.5;
                    creature.remote_outpost = Some(target);
                    creature.clear_task();
                }
            }
            session.progress.courier_deliveries += 1;
        }
        TransitDirection::ToShrine => {
            session.economy.ore_stock += transit.ore;
            session.economy.ingots_stock += transit.ingots;
            session.economy.food += transit.food;
            let stock = session.stockpile_pos();
            for creature in &mut session.creatures {
                if transit.passengers.contains(&creature.id) {
                    creature.x = stock.x as f32 + 0.5;
                    creature.y = stock.y as f32 + 0.5;
                    creature.remote_outpost = None;
                    creature.clear_task();
                }
            }
        }
    }
    let _ = data;
    Some(TransitCompletion {
        direction,
        cargo_units: transit
            .ore
            .saturating_add(transit.ingots)
            .saturating_add(transit.food.max(0.0) as u32),
        passenger_count: transit.passengers.len(),
    })
}

fn take_cargo(outpost: &mut Outpost, good: Good, amount: u32) {
    let current = outpost.cargo.get(&good).copied().unwrap_or(0);
    if current <= amount {
        outpost.cargo.remove(&good);
    } else {
        outpost.cargo.insert(good, current - amount);
    }
}

fn add_cargo(outpost: &mut Outpost, good: Good, amount: u32) {
    if amount > 0 {
        *outpost.cargo.entry(good).or_insert(0) += amount;
    }
}

fn recover_failed_transit(session: &mut GameSession, transit: WormTransit) {
    if transit.direction == TransitDirection::ToOutpost {
        session.economy.ore_stock += transit.ore;
        session.economy.ingots_stock += transit.ingots;
        session.economy.food += transit.food;
        for creature in &mut session.creatures {
            if transit.passengers.contains(&creature.id) {
                creature.remote_outpost = None;
                creature.clear_task();
            }
        }
    } else if let Some(outpost) = session
        .outposts
        .iter_mut()
        .find(|o| o.pos == transit.outpost)
    {
        add_cargo(outpost, Good::Ore, transit.ore);
        add_cargo(outpost, Good::Ingot, transit.ingots);
        add_cargo(outpost, Good::CookedFood, transit.food as u32);
        outpost.crew.extend(transit.passengers.iter().copied());
    }
    if let Some(outpost) = session
        .outposts
        .iter_mut()
        .find(|o| o.pos == transit.outpost)
    {
        outpost.last_failure =
            Some("The worm route collapsed; cargo returned to safety.".to_owned());
    }
    session.last_transit_failure =
        Some("Transit failed because the outpost was inactive.".to_owned());
}
