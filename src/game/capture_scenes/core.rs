//! Core menu, tutorial, and campaign staging scenes.

use super::*;

pub(super) fn handles(scene: &str) -> bool {
    scene.starts_with("touch_audit_")
        || matches!(
            scene,
            "touch_audit"
                | "menu"
                | "new_warren_confirm"
                | "load_confirm"
                | "load_confirm_resolved"
                | "settings"
                | "factory"
                | "hud_food"
                | "hud_jobs"
                | "hud_build"
                | "hud_objective"
                | "tutorial_food"
                | "tutorial_factory"
                | "tutorial_blacksmith"
                | "tutorial_worm"
                | "victory"
                | "victory_factory"
        )
}

pub(super) fn begin(game: &mut Game, scene: &str) {
    match scene {
        "touch_audit" => {
            super::begin(game, "warren");
            game.arm_touch_target_audit();
        }
        scene if scene.starts_with("touch_audit_") => {
            let base_scene = &scene["touch_audit_".len()..];
            super::begin(game, base_scene);
            game.arm_touch_target_audit();
        }
        "menu" => game.transition(StateTransition::BackToMenu),
        "new_warren_confirm" => {
            game.transition(StateTransition::BackToMenu);
            game.save_exists = true;
            game.confirm_new_warren = true;
        }
        "load_confirm" => {
            game.transition(StateTransition::StartWarren);
            game.paused = true;
            game.confirm_load = true;
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
            }
        }
        "load_confirm_resolved" => {
            game.transition(StateTransition::StartWarren);
            game.confirm_load = true;
            let loaded = match &game.state {
                GameState::Warren(session) => session.as_ref().clone(),
                GameState::Menu => return,
            };
            game.install_loaded_session(loaded);
            game.notifications.success("Warren loaded.");
        }
        "settings" => {
            game.transition(StateTransition::BackToMenu);
            game.settings_open = true;
        }
        "factory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage a mid-build factory: banked ore, ghosts, digs.
                session.tutorial_dismissed = true;
                session.economy.ore_stock = 24;
                session.economy.food = 60.0;
                let spawn = session.spawn_tile();
                let mut spots: Vec<TilePos> = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .collect();
                spots.sort_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                for (kind, spot) in ["farm", "cook_pot"].iter().zip(spots.iter().skip(2)) {
                    simulation::try_place_build_site(session, &game.data, kind, *spot);
                }
                for mark in session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(_, t)| **t == Tile::Rock)
                    .map(|(pos, _)| pos)
                    .filter(|p| p.manhattan_distance(&spawn) <= 6)
                    .take(4)
                    .collect::<Vec<_>>()
                {
                    session.toggle_dig_mark(mark);
                }
                for _ in 0..900 {
                    simulation::tick(session, &game.data);
                }
            }
        }
        "hud_food" => {
            super::begin(game, "warren");
            game.hud_panel = Some(HudPanel::Food);
        }
        "hud_jobs" => {
            super::begin(game, "warren");
            game.hud_panel = Some(HudPanel::Jobs);
        }
        "hud_build" => {
            super::begin(game, "factory");
            game.hud_panel = Some(HudPanel::Build);
        }
        "hud_objective" => {
            super::begin(game, "warren");
            game.hud_panel = Some(HudPanel::Objective);
        }
        "tutorial_food" => {
            game.transition(StateTransition::StartWarren);
            game.hud_panel = Some(HudPanel::Tutorial);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_step = 1;
                session.economy.food = 36.0;
                // Show the lesson at the exact recovery handoff: a player-
                // placed Farm is waiting on ore while Food Grid is under
                // pressure, so the Objective can name both visible responses.
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .find(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos);
                if let Some(spot) = spot {
                    session.build_sites.push(BuildSite {
                        kind: "farm".to_owned(),
                        pos: spot,
                        ore_needed: 10,
                        ore_delivered: 0,
                    });
                }
            }
        }
        "tutorial_factory" => {
            game.transition(StateTransition::StartWarren);
            game.hud_panel = Some(HudPanel::Tutorial);
            if let GameState::Warren(session) = &mut game.state {
                // Hold on the factory lesson so its distinction between the
                // prebuilt Mine and the Mine build button is reviewable.
                session.tutorial_step = 2;
                session.tutorial_built = true;
                session.economy.food = 80.0;
                session.economy.ore_stock = 24;
            }
        }
        "tutorial_blacksmith" => {
            game.transition(StateTransition::StartWarren);
            game.hud_panel = Some(HudPanel::Tutorial);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_step = 2;
                session.tutorial_built = true;
                session.economy.food = 80.0;
                session.economy.ore_stock = 24;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("blacksmith", spot));
                    game.selected_building = Some(spot);
                }
            }
        }
        "tutorial_worm" => {
            super::begin(game, "shrine");
            game.hud_panel = Some(HudPanel::Tutorial);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = false;
                session.tutorial_step = 4;
            }
        }
        "victory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = game.data.balance.win_food_surplus;
                session.economy.ore_delivered_total = game.data.balance.win_ore_delivered;
                session.won = true;
            }
        }
        "victory_factory" => {
            super::begin(game, "victory");
            if let GameState::Warren(session) = &mut game.state {
                session.creatures[0].job = Job::Guard;
            }
        }
        _ => unreachable!("unhandled core capture scene: {scene}"),
    }
}
