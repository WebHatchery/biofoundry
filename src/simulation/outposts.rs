//! Worm Shrine route validation and fixed-time cargo/crew transit.

use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::outposts::{Outpost, TransitDirection, WormTransit};
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

pub fn activate_outpost(session: &mut GameSession, pos: TilePos) -> bool {
    if session.buildings_of("worm_shrine").next().is_none()
        || session.building_at(pos).is_none_or(|b| b.kind != "outpost")
    {
        return false;
    }
    session.ensure_outpost(pos);
    session
        .outposts
        .iter_mut()
        .find(|o| o.pos == pos)
        .map(|o| {
            o.active = !o.active;
            o.last_failure = None;
            true
        })
        .unwrap_or(false)
}

pub fn start_to_outpost(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(session, data, pos, TransitDirection::ToOutpost)
}

pub fn start_to_shrine(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    start_transit(session, data, pos, TransitDirection::ToShrine)
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
            let used = outpost.cargo_total();
            let room = cap.saturating_sub(used);
            let food = (session.economy.food - data.balance.worm_feed_reserve)
                .max(0.0)
                .min(room as f32);
            let ore = session.economy.ore_stock.min(room);
            let room_after_ore = room.saturating_sub(ore);
            let ingots = session.economy.ingots_stock.min(room_after_ore);
            let room_after_ingots = room_after_ore.saturating_sub(ingots);
            let food = food.min(room_after_ingots as f32);
            (ore, ingots, food)
        }
        TransitDirection::ToShrine => (
            *outpost.cargo.get(&Good::Ore).unwrap_or(&0),
            *outpost.cargo.get(&Good::Ingot).unwrap_or(&0),
            *outpost.cargo.get(&Good::CookedFood).unwrap_or(&0) as f32,
        ),
    };
    if ore == 0 && ingots == 0 && food <= 0.0 {
        return false;
    }

    let passengers = match direction {
        TransitDirection::ToOutpost => session
            .creatures
            .iter()
            .filter(|c| c.tile() == session.stockpile_pos())
            .take(data.balance.outpost_capacity as usize)
            .map(|c| c.id)
            .collect(),
        TransitDirection::ToShrine => outpost.crew.clone(),
    };

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

pub fn tick_transit(session: &mut GameSession, data: &GameData, dt: f32) {
    let Some(mut transit) = session.worm_transit.take() else {
        return;
    };
    if !session
        .outposts
        .iter()
        .any(|o| o.pos == transit.outpost && o.active)
    {
        recover_failed_transit(session, transit);
        return;
    }
    transit.remaining -= dt;
    if transit.remaining > 0.0 {
        session.worm_transit = Some(transit);
        return;
    }
    let target = transit.outpost;
    match transit.direction {
        TransitDirection::ToOutpost => {
            if let Some(outpost) = session.outposts.iter_mut().find(|o| o.pos == target) {
                outpost.cargo.insert(Good::Ore, transit.ore);
                outpost.cargo.insert(Good::Ingot, transit.ingots);
                outpost.cargo.insert(Good::CookedFood, transit.food as u32);
                outpost.crew.extend(transit.passengers.iter().copied());
            }
            for creature in &mut session.creatures {
                if transit.passengers.contains(&creature.id) {
                    creature.x = target.x as f32 + 0.5;
                    creature.y = target.y as f32 + 0.5;
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
                    creature.clear_task();
                }
            }
        }
    }
    let _ = data;
}

fn take_cargo(outpost: &mut Outpost, good: Good, amount: u32) {
    let current = outpost.cargo.get(&good).copied().unwrap_or(0);
    if current <= amount {
        outpost.cargo.remove(&good);
    } else {
        outpost.cargo.insert(good, current - amount);
    }
}

fn recover_failed_transit(session: &mut GameSession, transit: WormTransit) {
    if transit.direction == TransitDirection::ToOutpost {
        session.economy.ore_stock += transit.ore;
        session.economy.ingots_stock += transit.ingots;
        session.economy.food += transit.food;
    } else if let Some(outpost) = session
        .outposts
        .iter_mut()
        .find(|o| o.pos == transit.outpost)
    {
        outpost
            .cargo
            .entry(Good::Ore)
            .and_modify(|n| *n += transit.ore)
            .or_insert(transit.ore);
        outpost
            .cargo
            .entry(Good::Ingot)
            .and_modify(|n| *n += transit.ingots)
            .or_insert(transit.ingots);
        outpost
            .cargo
            .entry(Good::CookedFood)
            .and_modify(|n| *n += transit.food as u32)
            .or_insert(transit.food as u32);
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
