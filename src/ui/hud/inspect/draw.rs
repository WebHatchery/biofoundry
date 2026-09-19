//! Rendering entry point for the selected building inspection card.

use super::*;

#[path = "draw_outpost.rs"]
mod draw_outpost;
use draw_outpost::draw_outpost_details;

pub(crate) fn draw_inspect_panel(
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

    let compact = crate::ui::hud::panels::compact_top_bar(ui_scale);
    // Crafting and breeding are deliberate choices rather than high-frequency
    // chrome. Give their compact cards a little more vertical breathing room
    // so a player can target the action and read its benefit at 800×450.
    let (inspect_button_height, inspect_button_step) =
        inspection_button_metrics(&building.kind, compact, data.equipment.len());

    let layout = InspectionLayout::new(
        &building.kind,
        compact,
        data.equipment.len(),
        inspect_button_step,
        top,
    );
    let panel = layout.panel;
    let compact_outpost = layout.compact_outpost;
    if compact_outpost {
        // This sheet replaces the compact HUD rather than floating above it.
        // Give it an opaque backing so world labels and the bottom status
        // legend cannot show through its controls.
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, dark::BACKGROUND);
    }
    draw_surface_with_title(
        panel,
        Some(name),
        &panel_style(),
        TextStyle::new(16.0, dark::TEXT_BRIGHT),
    );
    if compact_outpost {
        // The sheet covers the underlying HUD, so its Close action must be
        // visible before the route controls take over the compact canvas.
        macroquad_toolkit::ui::occlude(panel);
        if hud_button(
            Rect::new(panel.right() - 92.0, panel.y, 72.0, 72.0),
            "Close",
            true,
            mouse,
        ) {
            actions.push(UiAction::ClearSelection);
        }
    }

    let x = layout.content_x;
    let mut y = layout.content_y;
    let outpost_button_height = layout.outpost_button_height;
    let outpost_button_step = layout.outpost_button_step;
    let line_step = layout.line_step;
    let line = |text: &str, color: Color, y: &mut f32| {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, color).params());
        *y += line_step;
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
        "material_stockpile" => {
            let good = building.accepted_good();
            let copy_id = match good {
                Good::Wood => "storage.wood",
                Good::Charcoal => "storage.charcoal",
                _ => "storage.mushrooms",
            };
            if let Some(storage) = crate::simulation::storage::definition(building, data) {
                line(
                    &format!(
                        "{} {:.0}/{}",
                        data.message(copy_id),
                        building.stock(good),
                        storage.capacity
                    ),
                    dark::TEXT,
                    &mut y,
                );
                line(
                    &data
                        .message("storage.range")
                        .replace("{radius}", &storage.supply_radius.to_string()),
                    dark::TEXT_DIM,
                    &mut y,
                );
                line(data.message("storage.supply"), dark::TEXT_DIM, &mut y);
                let empty = building.stocks.values().sum::<f32>() < 1.0;
                if hud_button(
                    Rect::new(x, y, panel.w - 28.0, inspect_button_height),
                    data.message("storage.change"),
                    empty,
                    mouse,
                ) {
                    actions.push(UiAction::CycleStorageGood(pos));
                }
                y += inspect_button_step + 14.0;
                line(
                    data.message(if empty {
                        "storage.cycle"
                    } else {
                        "storage.empty_first"
                    }),
                    dark::TEXT_DIM,
                    &mut y,
                );
            }
        }
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
        "rest_hollow" => {
            draw_rest_hollow_inspection(data, x, &mut y);
        }
        "study_pen" => {
            line(
                &format!("Specimens housed · {}", session.progress.specimens),
                dark::TEXT,
                &mut y,
            );
            line(
                &format!(
                    "Study +{:.1}/min · {:.1} observed",
                    study_rate_per_min(session, data),
                    session.progress.knowledge
                ),
                dark::TEXT,
                &mut y,
            );
            line(
                &study_adaptation_line(session, data),
                if session.unlocked.contains("adaptive_haulers") {
                    dark::POSITIVE
                } else {
                    dark::WARNING
                },
                &mut y,
            );
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
            draw_blacksmith_inspection(BlacksmithInspection {
                session,
                data,
                building,
                pos,
                x,
                y: &mut y,
                panel_width: panel.w,
                mouse,
                compact,
                button_height: inspect_button_height,
                button_step: inspect_button_step,
                line_step,
                actions,
            });
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
            line(
                &format!(
                    "Hatch cycle {:.0}s",
                    crate::simulation::wildlife::breeding_interval_sec(session, data)
                ),
                dark::TEXT,
                &mut y,
            );
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
                    breed_button_label(id, name, cost, data)
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
            draw_outpost_details(
                session,
                data,
                pos,
                panel,
                mouse,
                compact,
                compact_outpost,
                outpost_button_height,
                outpost_button_step,
                line_step,
                x,
                &mut y,
                actions,
            );
        }
        _ => {
            line("Tap empty ground to deselect", dark::TEXT_DIM, &mut y);
        }
    }

    Some(panel)
}
