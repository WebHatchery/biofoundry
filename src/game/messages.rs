//! Player-facing notices and small game-loop policy helpers.

use super::*;

pub fn report_touch_target_audit() {
    let mut report = String::new();
    if !macroquad_toolkit::ui::neighbours_warm() {
        report.push_str("touch targets: no settled controls were recorded\n");
        emit_touch_target_audit_report(&report);
        return;
    }

    if let Some((width, worst)) = macroquad_toolkit::ui::smallest_touchable_width(ui::LOGICAL_WIDTH)
    {
        report.push_str(&format!(
            "touch targets: need a {:.0}px-wide window; worst is {}\n",
            width, worst
        ));
    }
    for (side, label) in macroquad_toolkit::ui::undersized_targets() {
        report.push_str(&format!("touch targets: drawn {:.0}px — {}\n", side, label));
    }
    for (a, b, area) in macroquad_toolkit::ui::overlapping_targets() {
        report.push_str(&format!(
            "touch targets: {} and {} overlap by {:.0}px² once grown\n",
            a, b, area
        ));
    }
    emit_touch_target_audit_report(&report);
}

pub fn emit_touch_target_audit_report(report: &str) {
    print!("{report}");
    if let Ok(path) = std::env::var("BIOFOUNDRY_TOUCH_AUDIT_REPORT") {
        if let Err(error) = std::fs::write(&path, report) {
            eprintln!("touch targets: could not write {path}: {error}");
        }
    }
}

pub fn tile_world_center(tile: TilePos, tile_size: f32) -> Option<Vec2> {
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return None;
    }
    let (x, y) = tile.to_f32();
    Some(vec2((x + 0.5) * tile_size, (y + 0.5) * tile_size))
}

pub fn simulation_blocked_by_modal(
    session: &GameSession,
    data: &GameData,
    help_open: bool,
    routes_open: bool,
    confirm_load: bool,
) -> bool {
    help_open
        || routes_open
        || confirm_load
        || (session.won && !session.victory_shown)
        || (session.factory_complete && !session.factory_shown)
        || (session.worm_awake && !session.worm_shown)
        || (!session.worm_awake && session.is_non_viable(data))
}

pub fn clear_replacement_confirmations(confirm_new_warren: &mut bool, confirm_load: &mut bool) {
    *confirm_new_warren = false;
    *confirm_load = false;
}

pub fn progression_reaches_safe_beat(report: &simulation::TickReport) -> bool {
    report.wild.raid_survived
        || report.wild.captured > 0
        || !report.wild.unlocked.is_empty()
        || report.wild.bred_beetle
        || report.outpost_relay_awarded
        || report.outpost_convoy_awarded > 0
        || report.outpost_muster_awarded > 0
        || report.outpost_concord_awarded
        || report.outpost_circuit_awarded
        || report.outpost_encore_awarded > 0
        || report.outpost_chorus_awarded
        || report.auto_load_started.is_some()
}

pub fn transit_completion_notice(completion: TransitCompletion) -> &'static str {
    let payload = match (completion.cargo_units > 0, completion.passenger_count > 0) {
        (true, true) => "cargo and crew delivered",
        (true, false) => "cargo delivered",
        (false, true) => "crew delivered",
        (false, false) => "route complete",
    };
    match completion.direction {
        TransitDirection::ToOutpost => match payload {
            "cargo and crew delivered" => {
                "The worm reaches the outpost — cargo and crew delivered."
            }
            "cargo delivered" => "The worm reaches the outpost — cargo delivered.",
            "crew delivered" => "The worm reaches the outpost — crew delivered.",
            _ => "The worm reaches the outpost — route complete.",
        },
        TransitDirection::ToShrine => match payload {
            "cargo and crew delivered" => {
                "The worm returns to the shrine — cargo and crew delivered."
            }
            "cargo delivered" => "The worm returns to the shrine — cargo delivered.",
            "crew delivered" => "The worm returns to the shrine — crew delivered.",
            _ => "The worm returns to the shrine — route complete.",
        },
    }
}

pub fn auto_return_notice() -> &'static str {
    "Outpost hold full — cargo returning while scouts remain remote."
}

pub fn auto_resupply_notice() -> &'static str {
    "Outpost scouts need food — a food-only resupply is on its way."
}

pub fn auto_load_notice() -> &'static str {
    "Auto-load departed — cargo and available scouts are on the worm road."
}

pub fn transit_failure_notice() -> &'static str {
    "The worm route failed — tap the outpost, then reactivate the route before trying again."
}

pub fn warren_secured_notice(session: &GameSession) -> &'static str {
    if session.job_count(Job::Guard) > 0 {
        "The warren is secure — onboarding complete."
    } else {
        "The reserve gate is secure — assign a Guard to finish onboarding."
    }
}

pub fn unlock_notice(data: &GameData, session: &GameSession, name: &str) -> String {
    let Some(unlock) = data.unlocks.iter().find(|unlock| unlock.name == name) else {
        return format!("Unlocked: {name}");
    };

    let detail = match unlock.effect.as_str() {
        "unlock_building" => unlock
            .building
            .as_deref()
            .and_then(|id| data.buildings.get(id).map(|building| (id, building)))
            .map(|(id, building)| {
                if advanced_building_hidden(session, id) {
                    "available in Build & Dig after onboarding".to_owned()
                } else {
                    format!("tap {} in Build & Dig", building.name)
                }
            })
            .unwrap_or_else(|| "available in Build & Dig".to_owned()),
        "unlock_creature" => {
            let route = match unlock.id.as_str() {
                "slime_janitor" => "tap Slime in Jobs".to_owned(),
                "bat_courier" => "tap Bat in Jobs".to_owned(),
                _ => data
                    .species
                    .get(&unlock.id)
                    .map(|species| format!("tap {} in the Breeding Pit", species.name))
                    .unwrap_or_else(|| "tap the specialist button in the Breeding Pit".to_owned()),
            };
            if crate::ui::legibility::advanced_systems_unlocked(session) {
                route
            } else {
                format!("{route} after onboarding")
            }
        }
        "guard_dps_mult" => format!("Guards deal +{:.0}% damage", (unlock.value - 1.0) * 100.0),
        "farm_cap_mult" => format!("Farms hold +{:.0}% food", (unlock.value - 1.0) * 100.0),
        "beetle_carry_mult" => {
            format!("Beetle Haulers carry +{:.0}%", (unlock.value - 1.0) * 100.0)
        }
        "breed_interval_mult" => format!(
            "Breeding Pits hatch {:.0}% sooner",
            (1.0 - unlock.value) * 100.0
        ),
        "unlock_equipment" => format!("tap {} at the Blacksmith", unlock.name),
        _ => unlock.description.trim_end_matches('.').to_owned(),
    };

    format!("Unlocked: {} — {detail}.", unlock.name)
}

pub fn advanced_building_hidden(session: &GameSession, building_id: &str) -> bool {
    !matches!(
        building_id,
        "blacksmith" | "cook_pot" | "farm" | "mine" | "worm_shrine"
    ) && !crate::ui::legibility::advanced_systems_unlocked(session)
}
