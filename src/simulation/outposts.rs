//! Worm Shrine route validation and fixed-time cargo/crew transit.

use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::outposts::{
    CargoPriority, ExpeditionCompletion, Outpost, TransitCompletion, TransitDirection, WormTransit,
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
        // failure until a new transit happens to launch. Keep it when another
        // Outpost still carries an unresolved failure, though.
        sync_transit_failure_banner(session);
    }
    true
}

/// Reconcile the persisted per-route failure records with the global warning
/// shown by the HUD. This also repairs saves written while a route failure was
/// being recovered or clears a stale banner after the last failure is gone.
pub fn sync_transit_failure_banner(session: &mut GameSession) {
    if session
        .outposts
        .iter()
        .any(|outpost| outpost.last_failure.is_some())
    {
        if session.last_transit_failure.is_none() {
            session.last_transit_failure =
                Some("Transit failed because an outpost was inactive.".to_owned());
        }
    } else {
        session.last_transit_failure = None;
    }
}

pub fn start_to_outpost(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(
        session,
        data,
        pos,
        TransitDirection::ToOutpost,
        TransitPlan::Standard,
    )
}

pub fn start_to_shrine(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(
        session,
        data,
        pos,
        TransitDirection::ToShrine,
        TransitPlan::Standard,
    )
}

/// Start a return trip that unloads the remote hold but leaves stationed
/// scouts at the Outpost for another expedition cycle.
pub fn start_cargo_to_shrine(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(
        session,
        data,
        pos,
        TransitDirection::ToShrine,
        TransitPlan::CargoOnly,
    )
}

/// Return the selected Outpost's current remote cargo capacity.
pub fn storage_capacity(outpost: &Outpost, data: &GameData) -> u32 {
    if outpost.storage_upgraded {
        data.balance
            .outpost_upgraded_storage_cap
            .max(data.balance.outpost_storage_cap)
    } else {
        data.balance.outpost_storage_cap
    }
}

/// Buy the one-time remote hold expansion for an active awakened route.
pub fn upgrade_outpost(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    if !session.worm_awake
        || session.worm_transit.is_some()
        || session
            .building_at(pos)
            .is_none_or(|building| building.kind != "outpost")
    {
        return false;
    }
    session.ensure_outpost(pos);
    let Some(index) = session
        .outposts
        .iter()
        .position(|outpost| outpost.pos == pos)
    else {
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
    if outpost.cargo_total() >= storage_capacity(outpost, data) {
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

pub fn tick_expeditions(
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
) -> Vec<ExpeditionCompletion> {
    if !session.worm_awake {
        return Vec::new();
    }
    let cycle = data.balance.outpost_expedition_cycle_sec.max(0.1);
    let food_per_crew = data.balance.outpost_expedition_food_per_crew;
    let ore_per_crew = data.balance.outpost_expedition_ore_per_crew;
    let mut completed = Vec::new();
    for outpost in &mut session.outposts {
        if !outpost.active || outpost.expedition_paused || outpost.crew.is_empty() {
            continue;
        }
        let crew = outpost.crew.len() as u32;
        let room = storage_capacity(outpost, data).saturating_sub(outpost.cargo_total());
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
        let ore = crew.saturating_mul(ore_per_crew).min(room);
        add_cargo(outpost, Good::Ore, ore);
        outpost.expedition_progress -= cycle;
        outpost.expeditions_completed = outpost.expeditions_completed.saturating_add(1);
        outpost.ore_scouted = outpost.ore_scouted.saturating_add(ore);
        completed.push(ExpeditionCompletion {
            outpost: outpost.pos,
            ore,
            food_spent: food_cost,
        });
    }
    completed
}

/// Start one opt-in cargo-only return when an active route has filled its
/// remote hold. The global worm can carry one transit at a time, so routes are
/// considered in their persisted order and the next full route waits its turn.
pub fn start_auto_return_if_full(session: &mut GameSession, data: &GameData) -> Option<TilePos> {
    if session.worm_transit.is_some() {
        return None;
    }
    let index = next_auto_route_index(session, |outpost| {
        outpost.active
            && outpost.auto_return_cargo
            && outpost.cargo_total() >= storage_capacity(outpost, data)
    })?;
    let pos = session.outposts[index].pos;
    if start_transit(
        session,
        data,
        pos,
        TransitDirection::ToShrine,
        TransitPlan::AutoCargoReturn,
    ) {
        advance_auto_route_cursor(session, index);
        Some(pos)
    } else {
        None
    }
}

/// Start one opt-in food-only trip when stationed scouts need provisions.
/// Manual pause is respected so a player who is protecting a route does not
/// have food pulled from the warren unexpectedly.
pub fn start_auto_resupply_if_needed(
    session: &mut GameSession,
    data: &GameData,
) -> Option<TilePos> {
    if session.worm_transit.is_some() {
        return None;
    }
    let index = next_auto_route_index(session, |outpost| {
        let food_required = (outpost.crew.len() as u32)
            .saturating_mul(data.balance.outpost_expedition_food_per_crew);
        let food_available = outpost.cargo.get(&Good::CookedFood).copied().unwrap_or(0);
        let food_ready_at_warren = (session.economy.food - data.balance.worm_feed_reserve)
            .max(0.0)
            .floor() as u32;
        outpost.active
            && outpost.auto_resupply_food
            && !outpost.expedition_paused
            && !outpost.crew.is_empty()
            && outpost.cargo_total() < storage_capacity(outpost, data)
            && food_available < food_required
            && food_ready_at_warren > 0
    })?;
    let pos = session.outposts[index].pos;
    if start_transit(
        session,
        data,
        pos,
        TransitDirection::ToOutpost,
        TransitPlan::FoodResupply,
    ) {
        advance_auto_route_cursor(session, index);
        Some(pos)
    } else {
        None
    }
}

fn next_auto_route_index<F>(session: &GameSession, mut eligible: F) -> Option<usize>
where
    F: FnMut(&Outpost) -> bool,
{
    let route_count = session.outposts.len();
    if route_count == 0 {
        return None;
    }
    let start = session.auto_route_cursor % route_count;
    (0..route_count)
        .map(|offset| (start + offset) % route_count)
        .find(|&index| eligible(&session.outposts[index]))
}

fn advance_auto_route_cursor(session: &mut GameSession, served_index: usize) {
    if !session.outposts.is_empty() {
        session.auto_route_cursor = (served_index + 1) % session.outposts.len();
    }
}

#[derive(Clone, Copy)]
enum TransitPlan {
    Standard,
    CargoOnly,
    AutoCargoReturn,
    FoodResupply,
}

fn start_transit(
    session: &mut GameSession,
    data: &GameData,
    pos: TilePos,
    direction: TransitDirection,
    plan: TransitPlan,
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
    let cap = storage_capacity(outpost, data);
    let (ore, ingots, food) = match (direction, plan) {
        (TransitDirection::ToOutpost, TransitPlan::FoodResupply) => {
            let room = cap.saturating_sub(outpost.cargo_total());
            let food_available = (session.economy.food - data.balance.worm_feed_reserve)
                .max(0.0)
                .floor() as u32;
            (0, 0, food_available.min(room) as f32)
        }
        (TransitDirection::ToOutpost, _) => {
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
        (TransitDirection::ToShrine, TransitPlan::AutoCargoReturn) => {
            let food = *outpost.cargo.get(&Good::CookedFood).unwrap_or(&0);
            let keep_food = (outpost.crew.len() as u32)
                .saturating_mul(data.balance.outpost_expedition_food_per_crew);
            (
                *outpost.cargo.get(&Good::Ore).unwrap_or(&0),
                *outpost.cargo.get(&Good::Ingot).unwrap_or(&0),
                food.saturating_sub(keep_food) as f32,
            )
        }
        (TransitDirection::ToShrine, _) => (
            *outpost.cargo.get(&Good::Ore).unwrap_or(&0),
            *outpost.cargo.get(&Good::Ingot).unwrap_or(&0),
            *outpost.cargo.get(&Good::CookedFood).unwrap_or(&0) as f32,
        ),
    };
    let passengers = match (direction, plan) {
        (TransitDirection::ToOutpost, TransitPlan::FoodResupply) => Vec::new(),
        (TransitDirection::ToOutpost, _) => session
            .creatures
            .iter()
            .filter(|c| {
                !c.is_remote() && c.carrying.is_none() && c.tile() == session.stockpile_pos()
            })
            .take(
                outpost.crew_dispatch_count(
                    data.balance
                        .outpost_capacity
                        .saturating_sub(outpost.crew.len() as u32),
                ) as usize,
            )
            .map(|c| c.id)
            .collect(),
        (TransitDirection::ToShrine, TransitPlan::Standard) => outpost.crew.clone(),
        (TransitDirection::ToShrine, _) => Vec::new(),
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
                o.crew.retain(|id| !passengers.contains(id));
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
    // Starting a healthy route must not hide another Outpost's unresolved
    // failure from the global HUD banner.
    sync_transit_failure_banner(session);
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
