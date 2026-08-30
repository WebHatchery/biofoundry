//! The building inspection card: what the clicked node is doing right now,
//! plus its per-kind verbs (blacksmith production orders, pit breeding).

use crate::data::GameData;
use crate::state::creatures::Good;
#[cfg(test)]
use crate::state::creatures::Task;
#[cfg(test)]
use crate::state::outposts::{TransitDirection, WormTransit};
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::legibility::{shrine_waiting_for_food, shrine_waiting_for_ingots, BuildingStatus};
use crate::ui::{UiAction, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

mod breeding;
mod outpost;
mod status;
mod workstations;

use breeding::{breed_label, breeding_unlock_hint};
use outpost::{
    draw_compact_route_controls, draw_route_upgrade_controls, CompactRouteContext,
    RouteUpgradeContext,
};
pub(super) use status::inspect_status;
use status::{
    local_mine_staffed_at, local_mine_worker_at, mine_staffing_label,
    outpost_cargo_only_return_label, outpost_expedition_hint, outpost_has_loadable_payload,
    outpost_load_hint, outpost_return_label, transit_destination, transit_payload_line,
    waste_inspection_hint,
};
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
    top: f32,
    mouse: Vec2,
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) -> Option<Rect> {
    let building = session.building_at(pos)?;
    let def = data.buildings.get(&building.kind);
    let name = def.map(|d| d.name.as_str()).unwrap_or(&building.kind);

    let compact = super::panels::compact_top_bar(ui_scale);
    // Crafting and breeding are deliberate choices rather than high-frequency
    // chrome. Give their compact cards a little more vertical breathing room
    // so a player can target the action and read its benefit at 800×450.
    let (inspect_button_height, inspect_button_step) =
        inspection_button_metrics(&building.kind, compact);

    // The blacksmith panel carries the production-order queue and craft
    // buttons, and the breeding pit its breed buttons — both taller.
    let height = match building.kind.as_str() {
        "blacksmith" => 194.0 + data.equipment.len() as f32 * inspect_button_step,
        "breeding_pit" => {
            if compact {
                350.0
            } else {
                280.0
            }
        }
        "worm_shrine" => {
            if compact {
                250.0
            } else {
                240.0
            }
        }
        "outpost" => 510.0,
        _ => 152.0,
    };
    let panel = Rect::new(LOGICAL_WIDTH - 262.0, top, 250.0, height);
    draw_surface_with_title(
        panel,
        Some(name),
        &panel_style(),
        TextStyle::new(16.0, dark::TEXT_BRIGHT),
    );

    let x = panel.x + 14.0;
    let mut y = panel.y + 50.0;
    let outpost_button_height = if compact { 30.0 } else { 24.0 };
    let outpost_button_step = if compact { 34.0 } else { 26.0 };
    let line = |text: &str, color: Color, y: &mut f32| {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, color).params());
        *y += 20.0;
    };

    let (status, status_color) = inspect_status(session, data, building);
    line(&format!("Status · {status}"), status_color, &mut y);
    if status == "No valid route" {
        line("Dig a tunnel to reconnect this node", dark::WARNING, &mut y);
    }
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
                if hud_button(
                    Rect::new(x, y, bw, inspect_button_height),
                    &label,
                    queue_available,
                    mouse,
                ) {
                    actions.push(UiAction::QueueOrder(pos, eq.id.clone()));
                }
                y += inspect_button_step;
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
                if hud_button(
                    Rect::new(x, y, bw, inspect_button_height),
                    &label,
                    enabled,
                    mouse,
                ) {
                    actions.push(UiAction::Breed(id.to_owned()));
                }
                y += if compact { inspect_button_step } else { 28.0 };
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
                } else if shrine_waiting_for_food(session, data) {
                    "Waiting for food reserve"
                } else if shrine_waiting_for_ingots(session, data) {
                    "Waiting for ingot reserve"
                } else {
                    "Offering automatically"
                },
                if paused
                    || shrine_waiting_for_food(session, data)
                    || shrine_waiting_for_ingots(session, data)
                {
                    dark::WARNING
                } else {
                    dark::POSITIVE
                },
                &mut y,
            );
            if hud_button(
                Rect::new(x, y, panel.w - 28.0, inspect_button_height),
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
            let crew_cap = outpost
                .map(|route| crate::simulation::outposts::crew_capacity(route, data))
                .unwrap_or(data.balance.outpost_capacity);
            let crew_dispatch_limit = outpost.and_then(|route| route.crew_dispatch_limit);
            let storage_cap = outpost
                .map(|route| crate::simulation::outposts::storage_capacity(route, data))
                .unwrap_or(data.balance.outpost_storage_cap);
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
                    storage_cap,
                    crew,
                    crew_cap
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
            if active && session.worm_awake {
                if let Some(route) = outpost {
                    line(
                        &format!(
                            "Scouted ore {} · Hauls {}",
                            route.ore_scouted, route.expeditions_completed
                        ),
                        dark::TEXT_DIM,
                        &mut y,
                    );
                }
                let in_transit = session
                    .worm_transit
                    .as_ref()
                    .is_some_and(|transit| transit.outpost == pos);
                if !in_transit {
                    if let Some(expedition_hint) =
                        outpost.and_then(|route| outpost_expedition_hint(data, route))
                    {
                        line(&expedition_hint, dark::TEXT_DIM, &mut y);
                    }
                }
            }
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
                    Rect::new(x, y, panel.w - 28.0, outpost_button_height),
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
                y += if compact { 38.0 } else { 32.0 };
                if !active && (cargo > 0 || crew > 0) {
                    line("Reactivate route to return payload", dark::WARNING, &mut y);
                }
                if active && session.worm_awake {
                    let loadable_payload = outpost_has_loadable_payload(
                        session,
                        data,
                        cargo,
                        crew,
                        storage_cap,
                        crew_cap,
                        crew_dispatch_limit,
                    );
                    let return_label = outpost_return_label(cargo, crew);
                    if hud_button(
                        Rect::new(x, y, panel.w - 28.0, outpost_button_height),
                        &return_label,
                        cargo > 0 || crew > 0,
                        mouse,
                    ) {
                        actions.push(UiAction::TransitToShrine(pos));
                    }
                    y += if compact { outpost_button_step } else { 24.0 };
                    if cargo > 0
                        && crew > 0
                        && hud_button(
                            Rect::new(x, y, panel.w - 28.0, outpost_button_height),
                            &outpost_cargo_only_return_label(cargo),
                            true,
                            mouse,
                        )
                    {
                        actions.push(UiAction::TransitCargoToShrine(pos));
                    }
                    if cargo > 0 && crew > 0 {
                        y += if compact { outpost_button_step } else { 26.0 };
                    }
                    if compact {
                        draw_compact_route_controls(CompactRouteContext {
                            session,
                            data,
                            pos,
                            outpost,
                            panel,
                            x,
                            y: &mut y,
                            crew,
                            button_height: outpost_button_height,
                            button_step: outpost_button_step,
                            mouse,
                            actions,
                        });
                    } else {
                        let auto_return_label = outpost
                            .map(|route| route.auto_return_label())
                            .unwrap_or("Auto-return · Off");
                        if hud_button(
                            Rect::new(x, y, panel.w - 28.0, 24.0),
                            auto_return_label,
                            session.worm_transit.is_none(),
                            mouse,
                        ) {
                            actions.push(UiAction::ToggleOutpostAutoReturn(pos));
                        }
                        y += 26.0;
                        let auto_resupply_label = outpost
                            .map(|route| route.auto_resupply_label())
                            .unwrap_or("Auto-resupply · Off");
                        if hud_button(
                            Rect::new(x, y, panel.w - 28.0, 24.0),
                            auto_resupply_label,
                            session.worm_transit.is_none(),
                            mouse,
                        ) {
                            actions.push(UiAction::ToggleOutpostAutoResupply(pos));
                        }
                        y += 26.0;
                        let priority = outpost
                            .map(|route| route.cargo_priority.label())
                            .unwrap_or("Ore first");
                        if hud_button(
                            Rect::new(x, y, panel.w - 28.0, 24.0),
                            &format!("Load order · {priority}"),
                            session.worm_transit.is_none(),
                            mouse,
                        ) {
                            actions.push(UiAction::CycleOutpostCargo(pos));
                        }
                        y += 26.0;
                        let crew_label = outpost
                            .map(|route| {
                                route.crew_dispatch_label(
                                    crate::simulation::outposts::crew_capacity(route, data),
                                )
                            })
                            .unwrap_or_else(|| "Crew per run · Auto".to_owned());
                        if hud_button(
                            Rect::new(x, y, panel.w - 28.0, 24.0),
                            &crew_label,
                            session.worm_transit.is_none(),
                            mouse,
                        ) {
                            actions.push(UiAction::CycleOutpostCrew(pos));
                        }
                        y += 26.0;
                        draw_route_upgrade_controls(RouteUpgradeContext {
                            session,
                            data,
                            pos,
                            outpost,
                            x,
                            width: panel.w - 28.0,
                            y: &mut y,
                            button_height: 24.0,
                            button_step: 26.0,
                            mouse,
                            actions,
                        });
                        if crew > 0
                            && hud_button(
                                Rect::new(x, y, panel.w - 28.0, 24.0),
                                if outpost.is_some_and(|route| route.expedition_paused) {
                                    "Resume scouting"
                                } else {
                                    "Pause scouting"
                                },
                                session.worm_transit.is_none(),
                                mouse,
                            )
                        {
                            actions.push(UiAction::ToggleOutpostExpedition(pos));
                        }
                        y += 38.0;
                    }
                    if let Some(load_hint) =
                        outpost.and_then(|route| outpost_load_hint(session, data, route))
                    {
                        line(&load_hint, dark::TEXT_DIM, &mut y);
                    }
                    if hud_button(
                        Rect::new(x, y, panel.w - 28.0, outpost_button_height),
                        "Load outpost from warren",
                        loadable_payload,
                        mouse,
                    ) {
                        actions.push(UiAction::TransitToOutpost(pos));
                    }
                    y += 36.0;
                    if cargo == 0 && crew == 0 && !loadable_payload {
                        line("No payload ready at warren", dark::WARNING, &mut y);
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

fn inspection_button_metrics(kind: &str, compact: bool) -> (f32, f32) {
    if compact && matches!(kind, "blacksmith" | "breeding_pit") {
        (36.0, 40.0)
    } else if compact {
        (30.0, 34.0)
    } else {
        (24.0, 26.0)
    }
}

#[cfg(test)]
mod tests;
