//! The building inspection card: what the clicked node is doing right now,
//! plus its per-kind verbs (blacksmith production orders, pit breeding).

use crate::data::GameData;
use crate::state::creatures::{Creature, Good, Job, Task};
use crate::state::outposts::{TransitDirection, WormTransit};
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::legibility::{advanced_systems_unlocked, BuildingStatus};
use crate::ui::{UiAction, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

mod breeding;
mod workstations;

use breeding::{breed_label, breeding_unlock_hint};
use workstations::{
    blacksmith_input_hint, blacksmith_queue_available, cook_pot_input_hint, kiln_input_hint,
    local_smelter_staffed_at, local_smelter_worker_at, local_smith_staffed_at,
    local_smith_worker_at, smelter_input_hint,
};

const LOCKED_SPECIALIST_MARKER: &str = "[L]";

/// First-pass building inspection (plan §Phase 6): what a clicked building
/// is doing right now. Phase 9 grows this into the full legibility layer.
/// Returns its rect while a building is selected.
pub(super) fn draw_inspect_panel(
    session: &GameSession,
    data: &GameData,
    pos: TilePos,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) -> Option<Rect> {
    let building = session.building_at(pos)?;
    let def = data.buildings.get(&building.kind);
    let name = def.map(|d| d.name.as_str()).unwrap_or(&building.kind);

    // The blacksmith panel carries the production-order queue and craft
    // buttons, and the breeding pit its breed buttons — both taller.
    let height = match building.kind.as_str() {
        "blacksmith" => 194.0 + data.equipment.len() as f32 * 26.0,
        "breeding_pit" => 280.0,
        "worm_shrine" => 240.0,
        "outpost" => 270.0,
        _ => 152.0,
    };
    let panel = Rect::new(LOGICAL_WIDTH - 262.0, 210.0, 250.0, height);
    draw_surface_with_title(
        panel,
        Some(name),
        &panel_style(),
        TextStyle::new(16.0, dark::TEXT_BRIGHT),
    );

    let x = panel.x + 14.0;
    let mut y = panel.y + 50.0;
    let line = |text: &str, color: Color, y: &mut f32| {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, color).params());
        *y += 20.0;
    };

    let (status, status_color) = inspect_status(session, data, building);
    line(&format!("Status · {status}"), status_color, &mut y);
    if matches!(
        crate::ui::legibility::building_status(session, data, building),
        Some(BuildingStatus::InputStarved)
    ) {
        let input_hint = match building.kind.as_str() {
            "blacksmith" => Some(blacksmith_input_hint(building, data)),
            "cook_pot" => Some(cook_pot_input_hint(building, data)),
            "kiln" => Some(kiln_input_hint()),
            "smelter" => Some(smelter_input_hint(building, data)),
            _ => None,
        };
        if let Some(input_hint) = input_hint {
            line(&input_hint, dark::WARNING, &mut y);
        }
    }
    if building.waste > 0.0 && building.kind != "feeding_trough" {
        line(
            &waste_inspection_hint(session, data, building),
            dark::WARNING,
            &mut y,
        );
    }

    match building.kind.as_str() {
        "mine" => {
            let staffed = session
                .creatures
                .iter()
                .filter(|c| local_mine_staffed_at(c, pos))
                .count();
            let slots = def
                .and_then(|d| d.workstation.as_ref())
                .map(|w| w.slots)
                .unwrap_or(0);
            // Effective rate folds in each stationed miner's Iron Pickaxe.
            let base_rate: f32 = session
                .creatures
                .iter()
                .filter(|c| local_mine_worker_at(c, pos))
                .map(|c| {
                    data.balance.mine_ore_per_min
                        * crate::ui::legibility::work_multiplier(c, session, data)
                })
                .sum();
            let rate: f32 = session
                .creatures
                .iter()
                .filter(|c| local_mine_worker_at(c, pos))
                .map(|c| {
                    let pickaxe = c
                        .equipment
                        .as_deref()
                        .and_then(|id| data.equipment_def(id))
                        .filter(|e| e.effect == "mine_speed_mult")
                        .map(|e| e.value)
                        .unwrap_or(1.0);
                    // Species strength and any Overseer aura fold in too.
                    data.balance.mine_ore_per_min
                        * pickaxe
                        * crate::ui::legibility::work_multiplier(c, session, data)
                })
                .sum();
            let deposit_exhausted = building.reserve <= 0.0;
            let worker_txt = mine_staffing_label(staffed, slots, deposit_exhausted);
            let worker_col = if deposit_exhausted {
                dark::NEGATIVE
            } else if staffed == 0 {
                dark::WARNING
            } else {
                dark::POSITIVE
            };
            line(&worker_txt, worker_col, &mut y);
            let rate_text = if rate > base_rate + 0.01 {
                format!("Ore  +{rate:.0}/min  (base +{base_rate:.0})")
            } else {
                format!("Ore  +{rate:.0}/min")
            };
            line(&rate_text, dark::TEXT, &mut y);
            line(
                &format!(
                    "Buffer {:.0}/{:.0}",
                    building.stock(Good::Ore),
                    data.balance.mine_buffer_cap
                ),
                dark::TEXT,
                &mut y,
            );
            line(
                &format!("Reserve {:.0}", building.reserve.max(0.0)),
                dark::TEXT_DIM,
                &mut y,
            );
        }
        "farm" => {
            line(
                &format!(
                    "Mushrooms {:.0}/{:.0}",
                    building.stock(Good::Mushroom),
                    crate::simulation::wildlife::farm_cap(session, data)
                ),
                dark::TEXT,
                &mut y,
            );
            line("Carriers haul to the Cook Pot", dark::TEXT_DIM, &mut y);
        }
        "cook_pot" => {
            line(
                &format!("Mushrooms {:.0}", building.stock(Good::Mushroom)),
                dark::TEXT,
                &mut y,
            );
            line("Cooks turn mushrooms → stew", dark::TEXT_DIM, &mut y);
        }
        "feeding_trough" => {
            line(
                &format!(
                    "Cooked food {:.1}/{:.1}",
                    building.stock(Good::CookedFood),
                    data.balance.trough_food_cap
                ),
                dark::TEXT,
                &mut y,
            );
            line(
                &format!(
                    "Waste {:.1} · Slime janitor distributes food",
                    building.waste
                ),
                if building.waste > 0.0 {
                    dark::WARNING
                } else {
                    dark::TEXT_DIM
                },
                &mut y,
            );
        }
        "blacksmith" => {
            let working = session
                .creatures
                .iter()
                .any(|c| local_smith_worker_at(c, pos));
            let staffed = session
                .creatures
                .iter()
                .any(|c| local_smith_staffed_at(c, pos));
            line(
                if working {
                    "Smith at work"
                } else if staffed {
                    "Smith stationed"
                } else {
                    "No smith — idle"
                },
                if staffed {
                    dark::POSITIVE
                } else {
                    dark::WARNING
                },
                &mut y,
            );
            let queue_available = blacksmith_queue_available(building, data);
            line(
                &format!(
                    "Ore {:.0}  Ingots {:.0}  Queue {}/{}",
                    building.stock(Good::Ore),
                    building.stock(Good::Ingot),
                    building.orders.len(),
                    data.balance.order_queue_size,
                ),
                dark::TEXT,
                &mut y,
            );
            if !queue_available {
                line("Queue full · finish orders first", dark::WARNING, &mut y);
            }
            // Production orders: one craft button per equipment item. A
            // queued count and how many are already banked ride in the label.
            y += 2.0;
            let bw = panel.w - 28.0;
            for eq in &data.equipment {
                let banked = session.economy.gear_stock.get(&eq.id).copied().unwrap_or(0);
                let queued = building.orders.iter().filter(|o| **o == eq.id).count();
                let mut label = format!("{} ({})", eq.name, eq.cost_ingots);
                if queued > 0 {
                    label.push_str(&format!("  ·{queued} queued"));
                } else if banked > 0 {
                    label.push_str(&format!("  ·{banked} ready"));
                }
                if hud_button(Rect::new(x, y, bw, 22.0), &label, queue_available, mouse) {
                    actions.push(UiAction::QueueOrder(pos, eq.id.clone()));
                }
                y += 26.0;
            }
        }
        "kiln" => {
            line(
                &format!(
                    "Wood {:.0}  Charcoal {:.0}",
                    building.stock(Good::Wood),
                    building.stock(Good::Charcoal)
                ),
                dark::TEXT,
                &mut y,
            );
        }
        "smelter" => {
            let working = session
                .creatures
                .iter()
                .any(|c| local_smelter_worker_at(c, pos));
            let staffed = session
                .creatures
                .iter()
                .any(|c| local_smelter_staffed_at(c, pos));
            line(
                if working {
                    "Salamander at work"
                } else if staffed {
                    "Salamander stationed"
                } else {
                    "No salamander — idle"
                },
                if staffed {
                    dark::POSITIVE
                } else {
                    dark::WARNING
                },
                &mut y,
            );
            line(
                &format!(
                    "Ore {:.0}  Charcoal {:.0}",
                    building.stock(Good::Ore),
                    building.stock(Good::Charcoal)
                ),
                dark::TEXT,
                &mut y,
            );
        }
        "breeding_pit" => {
            line("Beetles hatch here", dark::TEXT_DIM, &mut y);
            line("Specialists cost banked ingots", dark::TEXT_DIM, &mut y);
            // Evolution line: breed heavyweight workers once forged ingots
            // unlock them. Buttons stay visible, disabled until then.
            let bw = panel.w - 28.0;
            y += 2.0;
            let has_overseer = session.creatures.iter().any(|c| c.species == "overseer");
            for (id, unlock, cost, blocked) in [
                (
                    "hobgoblin",
                    "hobgoblin",
                    data.balance.hobgoblin_ingot_cost,
                    false,
                ),
                (
                    "overseer",
                    "overseer",
                    data.balance.overseer_ingot_cost,
                    has_overseer,
                ),
                (
                    "engineer",
                    "engineer",
                    data.balance.engineer_ingot_cost,
                    session.creatures.iter().any(|c| c.species == "engineer"),
                ),
            ] {
                let unlocked = session.unlocked.contains(unlock);
                let name = data.species.get(id).map(|s| s.name.as_str()).unwrap_or(id);
                let label = if !unlocked {
                    format!("{name} {LOCKED_SPECIALIST_MARKER}")
                } else if blocked {
                    format!("{name} — posted")
                } else {
                    breed_label(id, name, cost, data)
                };
                let enabled = unlocked && !blocked && session.economy.ingots_stock >= cost;
                if hud_button(Rect::new(x, y, bw, 24.0), &label, enabled, mouse) {
                    actions.push(UiAction::Breed(id.to_owned()));
                }
                y += 28.0;
                if !unlocked {
                    if let Some(hint) = breeding_unlock_hint(session, data, unlock) {
                        line(&hint, dark::WARNING, &mut y);
                    }
                }
            }
        }
        "worm_shrine" => {
            let paused = session.worm_feeding_paused;
            let food_remaining = (data.balance.worm_awaken_at - session.worm_fed).max(0.0);
            let ingots_remaining = data
                .balance
                .worm_awaken_ingots
                .saturating_sub(session.worm_ingots_fed);
            line(
                &format!(
                    "Food {:.0}/{:.0} · Ingots {}/{}",
                    session.worm_fed,
                    data.balance.worm_awaken_at,
                    session.worm_ingots_fed,
                    data.balance.worm_awaken_ingots
                ),
                dark::TEXT,
                &mut y,
            );
            line(
                &format!(
                    "Remaining {:.0} food · {} ingots",
                    food_remaining, ingots_remaining
                ),
                dark::TEXT_DIM,
                &mut y,
            );
            let minimum_feed_seconds = if data.balance.worm_food_per_min > 0.0 {
                food_remaining / data.balance.worm_food_per_min * 60.0
            } else {
                0.0
            };
            line(
                &format!(
                    "Food draw {:.0}/min · minimum {}",
                    data.balance.worm_food_per_min,
                    format_mmss(minimum_feed_seconds)
                ),
                dark::TEXT_DIM,
                &mut y,
            );
            line(
                if session.worm_awake {
                    "The Colossal Worm is awake"
                } else if paused {
                    "Offerings paused — reserve protected"
                } else if worm_waiting_for_food(session, data) {
                    "Waiting for food reserve"
                } else if worm_waiting_for_ingots(session, data) {
                    "Waiting for ingot reserve"
                } else {
                    "Offering automatically"
                },
                if paused
                    || worm_waiting_for_food(session, data)
                    || worm_waiting_for_ingots(session, data)
                {
                    dark::WARNING
                } else {
                    dark::POSITIVE
                },
                &mut y,
            );
            if hud_button(
                Rect::new(x, y, panel.w - 28.0, 24.0),
                if paused {
                    "Resume offerings"
                } else {
                    "Pause offerings"
                },
                !session.worm_awake,
                mouse,
            ) {
                actions.push(UiAction::ToggleShrineFeeding(pos));
            }
        }
        "outpost" => {
            let outpost = session.outposts.iter().find(|o| o.pos == pos);
            let active = outpost.is_some_and(|o| o.active);
            let cargo = outpost.map(|o| o.cargo_total()).unwrap_or(0);
            let crew = outpost.map(|o| o.crew.len()).unwrap_or(0);
            let cargo_ore = outpost
                .and_then(|o| o.cargo.get(&Good::Ore))
                .copied()
                .unwrap_or(0);
            let cargo_ingots = outpost
                .and_then(|o| o.cargo.get(&Good::Ingot))
                .copied()
                .unwrap_or(0);
            let cargo_food = outpost
                .and_then(|o| o.cargo.get(&Good::CookedFood))
                .copied()
                .unwrap_or(0);
            line(
                &format!(
                    "{} · Cargo {}/{} · Crew {}/{}",
                    if active { "Active" } else { "Inactive" },
                    cargo,
                    data.balance.outpost_storage_cap,
                    crew,
                    data.balance.outpost_capacity
                ),
                if active {
                    dark::POSITIVE
                } else {
                    dark::WARNING
                },
                &mut y,
            );
            line(
                &format!(
                    "Ore {} · Ingots {} · Food {}",
                    cargo_ore, cargo_ingots, cargo_food
                ),
                dark::TEXT_DIM,
                &mut y,
            );
            if let Some(transit) = session.worm_transit.as_ref() {
                if transit.outpost == pos {
                    line(&transit_payload_line(transit), dark::POSITIVE, &mut y);
                }
                let route = if transit.outpost == pos {
                    transit_destination(transit.direction)
                } else {
                    "another outpost"
                };
                line(
                    &format!(
                        "Transit to {route} · {:.0}s remaining",
                        transit.remaining.max(0.0)
                    ),
                    dark::POSITIVE,
                    &mut y,
                );
                line("Wait for the worm to arrive", dark::TEXT_DIM, &mut y);
            } else {
                if hud_button(
                    Rect::new(x, y, panel.w - 28.0, 24.0),
                    if active {
                        "Deactivate route"
                    } else {
                        "Activate route"
                    },
                    true,
                    mouse,
                ) {
                    actions.push(UiAction::ActivateOutpost(pos));
                }
                // Leave a full text-line gap before recovery copy so the
                // baseline cannot crowd the button's lower border.
                y += 36.0;
                if !active && (cargo > 0 || crew > 0) {
                    line("Reactivate route to return payload", dark::WARNING, &mut y);
                }
                if active && session.worm_awake {
                    let loadable_payload = outpost_has_loadable_payload(session, data, cargo, crew);
                    let return_label = outpost_return_label(cargo, crew);
                    if hud_button(
                        Rect::new(x, y, panel.w - 28.0, 24.0),
                        &return_label,
                        cargo > 0 || crew > 0,
                        mouse,
                    ) {
                        actions.push(UiAction::TransitToShrine(pos));
                    }
                    y += 28.0;
                    if hud_button(
                        Rect::new(x, y, panel.w - 28.0, 24.0),
                        "Load outpost from warren",
                        loadable_payload,
                        mouse,
                    ) {
                        actions.push(UiAction::TransitToOutpost(pos));
                    }
                    y += 36.0;
                    if cargo == 0 && crew == 0 && !loadable_payload {
                        line(
                            "No cargo or crew ready at the warren",
                            dark::WARNING,
                            &mut y,
                        );
                    }
                } else if active {
                    line("Awaiting the worm's awakening", dark::TEXT_DIM, &mut y);
                }
            }
            if let Some(failure) = outpost.and_then(|o| o.last_failure.as_deref()) {
                draw_text_block(
                    failure,
                    x,
                    y,
                    panel.w - 28.0,
                    38.0,
                    13.0,
                    3.0,
                    dark::NEGATIVE,
                );
            }
        }
        _ => {
            line("Tap empty ground to deselect", dark::TEXT_DIM, &mut y);
        }
    }

    Some(panel)
}

fn local_mine_worker_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote() && matches!(&creature.task, Task::WorkMine(p) if *p == pos)
}

fn mine_staffing_label(staffed: usize, slots: u32, deposit_exhausted: bool) -> String {
    if deposit_exhausted {
        "Deposit exhausted".to_owned()
    } else if staffed == 0 {
        "No mine worker — stopped".to_owned()
    } else {
        format!("Mine staff {staffed}/{slots}")
    }
}

fn waste_inspection_hint(
    session: &GameSession,
    data: &GameData,
    building: &crate::state::structures::Building,
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

fn local_mine_staffed_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote()
        && (creature.job == Job::Miner || creature.job == Job::Engineer)
        && match &creature.task {
            Task::WorkMine(p) | Task::GoMine(p) => *p == pos,
            _ => creature.tile() == pos,
        }
}

fn outpost_return_label(cargo: u32, crew: usize) -> String {
    match (cargo > 0, crew > 0) {
        (true, true) => format!("Send {cargo} cargo + {crew} crew to shrine"),
        (true, false) => format!("Send {cargo} cargo to shrine"),
        (false, true) => format!("Send {crew} crew to shrine"),
        (false, false) => "No cargo or crew to return".to_owned(),
    }
}

/// Give every inspected building the same first-read answer: is it working,
/// stalled, paused, or on a route that needs attention?
fn inspect_status(
    session: &GameSession,
    data: &GameData,
    building: &crate::state::structures::Building,
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
            if !session.worm_awake {
                return ("Route active", dark::POSITIVE);
            }
            if outpost.cargo_total() > 0 || !outpost.crew.is_empty() {
                return ("Payload ready", dark::POSITIVE);
            }
            if outpost_has_loadable_payload(session, data, 0, 0) {
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

fn transit_destination(direction: TransitDirection) -> &'static str {
    match direction {
        TransitDirection::ToOutpost => "outpost",
        TransitDirection::ToShrine => "shrine",
    }
}

fn transit_payload_line(transit: &WormTransit) -> String {
    let cargo = transit
        .ore
        .saturating_add(transit.ingots)
        .saturating_add(transit.food.max(0.0) as u32);
    format!(
        "In flight · {cargo} cargo · {} crew",
        transit.passengers.len()
    )
}

fn outpost_has_loadable_payload(
    session: &GameSession,
    data: &GameData,
    cargo: u32,
    crew: usize,
) -> bool {
    let crew_ready = crew < data.balance.outpost_capacity as usize
        && session
            .creatures
            .iter()
            .any(|c| !c.is_remote() && c.carrying.is_none() && c.tile() == session.stockpile_pos());
    outpost_has_loadable_cargo(session, data, cargo) || crew_ready
}

fn outpost_has_loadable_cargo(session: &GameSession, data: &GameData, cargo: u32) -> bool {
    if cargo >= data.balance.outpost_storage_cap {
        return false;
    }
    session.economy.ore_stock > 0
        || session.economy.ingots_stock > 0
        || session.economy.food - data.balance.worm_feed_reserve >= 1.0
}

fn worm_waiting_for_food(session: &GameSession, data: &GameData) -> bool {
    session.worm_fed < data.balance.worm_awaken_at
        && session.economy.food <= data.balance.worm_feed_reserve
}

fn worm_waiting_for_ingots(session: &GameSession, data: &GameData) -> bool {
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

#[cfg(test)]
mod tests;
