//! The standing HUD chrome: the top bar, the calorie balance meter, the
//! job assignment panel, the build/dig tools, and the tutorial card.

use crate::data::GameData;
use crate::simulation::{self, food};
use crate::state::creatures::Job;
use crate::state::GameSession;
use crate::ui::hud::requirements::unlock_requirement;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::hud::HudSprites;
use crate::ui::{UiAction, UiMode, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, format_mmss};

const OBJECTIVE_PANEL: Rect = Rect::new(548.0, 72.0, 370.0, 128.0);

pub(super) fn draw_top_bar(
    session: &GameSession,
    data: &GameData,
    bar: Rect,
    mouse: Vec2,
    paused: bool,
    actions: &mut Vec<UiAction>,
) {
    draw_surface(
        bar,
        &SurfaceStyle::new(Color::new(0.07, 0.08, 0.10, 0.94))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.55)),
    );

    draw_ui_text_ex(
        "Biofoundry — Warren",
        bar.x + 16.0,
        bar.y + 31.0,
        TextStyle::new(22.0, dark::TEXT_BRIGHT).params(),
    );

    let seconds = simulation::sim_seconds(session);
    draw_ui_text_ex(
        &format_mmss(seconds),
        bar.x + 290.0,
        bar.y + 31.0,
        TextStyle::new(18.0, dark::TEXT).params(),
    );

    if paused {
        draw_ui_text_ex(
            "PAUSED — tap Resume to continue",
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::WARNING).params(),
        );
    } else if let Some(transit) = &session.worm_transit {
        draw_ui_text_ex(
            &format!(
                "WORM TRANSIT — {:.0}s remaining",
                transit.remaining.max(0.0)
            ),
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::POSITIVE).params(),
        );
    } else if session.last_transit_failure.is_some() {
        draw_ui_text_ex(
            "WORM ROUTE FAILED — inspect the outpost",
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::NEGATIVE).params(),
        );
    } else if session.raid_active {
        let food_warning = food::time_to_empty_seconds(session, data)
            .filter(|seconds| *seconds <= data.balance.food_warning_sec);
        let food_suffix = if session.economy.food <= 0.0 {
            " · FAMINE".to_owned()
        } else if let Some(seconds) = food_warning {
            format!(" · FOOD IN {}", format_mmss(seconds))
        } else {
            String::new()
        };
        draw_ui_text_ex(
            &format!(
                "RAID — gnarls are after the larder! {}{}",
                compact_raid_defense_hint(session, data),
                food_suffix
            ),
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::NEGATIVE).params(),
        );
    } else if session.economy.food <= 0.0 {
        draw_ui_text_ex(
            "FAMINE — workers are slowing down",
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::NEGATIVE).params(),
        );
    } else if let Some(seconds) = food::time_to_empty_seconds(session, data)
        .filter(|seconds| *seconds <= data.balance.food_warning_sec)
    {
        if session.raid_in <= data.balance.raid_warning_sec {
            draw_ui_text_ex(
                &format!(
                    "FOOD IN {} · RAID IN {} · {}",
                    format_mmss(seconds),
                    format_mmss(session.raid_in.max(0.0)),
                    compact_raid_defense_hint(session, data)
                ),
                bar.x + 380.0,
                bar.y + 31.0,
                TextStyle::new(15.0, dark::WARNING).params(),
            );
        } else {
            draw_ui_text_ex(
                &format!(
                    "FOOD IN {} · {}",
                    format_mmss(seconds),
                    compact_food_recovery_hint(session, data)
                ),
                bar.x + 380.0,
                bar.y + 31.0,
                TextStyle::new(15.0, dark::WARNING).params(),
            );
        }
    } else if session.raid_in <= data.balance.raid_warning_sec {
        draw_ui_text_ex(
            &format!(
                "RAID IN {} — {}",
                format_mmss(session.raid_in.max(0.0)),
                compact_raid_defense_hint(session, data)
            ),
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::WARNING).params(),
        );
    } else {
        draw_ui_text_ex(
            "Drag map · tap +/− to zoom · Save / Load / Menu",
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::TEXT_DIM).params(),
        );
    }

    if hud_button(
        Rect::new(bar.right() - 96.0, bar.y + 8.0, 84.0, 32.0),
        "Menu",
        true,
        mouse,
    ) {
        actions.push(UiAction::BackToMenu);
    }
    if hud_button(
        Rect::new(bar.right() - 512.0, bar.y + 8.0, 74.0, 32.0),
        if paused { "Resume" } else { "Pause" },
        true,
        mouse,
    ) {
        actions.push(UiAction::TogglePause);
    }
    if hud_button(
        Rect::new(bar.right() - 354.0, bar.y + 4.0, 40.0, 40.0),
        "−",
        true,
        mouse,
    ) {
        actions.push(UiAction::ZoomCamera(-1));
    }
    if hud_button(
        Rect::new(bar.right() - 310.0, bar.y + 4.0, 40.0, 40.0),
        "+",
        true,
        mouse,
    ) {
        actions.push(UiAction::ZoomCamera(1));
    }
    if hud_button(
        Rect::new(bar.right() - 254.0, bar.y + 8.0, 74.0, 32.0),
        "Save",
        true,
        mouse,
    ) {
        actions.push(UiAction::Save);
    }
    if hud_button(
        Rect::new(bar.right() - 176.0, bar.y + 8.0, 74.0, 32.0),
        "Load",
        true,
        mouse,
    ) {
        actions.push(UiAction::Load);
    }
    if hud_button(
        Rect::new(bar.right() - 438.0, bar.y + 8.0, 74.0, 32.0),
        "Help",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
}

/// The calorie balance meter — production, consumption, and stockpile,
/// exactly like a power graph.
pub(super) fn draw_food_grid_panel(session: &GameSession, data: &GameData, panel: Rect) {
    draw_surface_with_title(
        panel,
        Some("Food Grid"),
        &panel_style(),
        TextStyle::new(17.0, dark::TEXT),
    );

    let production = session.economy.production_ema_per_min.max(0.0);
    let consumption = food::consumption_per_min(session, data);
    let net = production - consumption;
    let x = panel.x + 14.0;
    let mut y = panel.y + 50.0;

    draw_ui_text_ex(
        &format!("Production  +{production:.1}/min"),
        x,
        y,
        TextStyle::new(15.0, dark::POSITIVE).params(),
    );
    y += 21.0;
    draw_ui_text_ex(
        &format!("Upkeep      -{consumption:.1}/min"),
        x,
        y,
        TextStyle::new(15.0, dark::NEGATIVE).params(),
    );
    y += 21.0;
    let net_color = if net >= 0.0 {
        dark::POSITIVE
    } else {
        dark::NEGATIVE
    };
    draw_ui_text_ex(
        &format!("Net         {net:+.1}/min"),
        x,
        y,
        TextStyle::new(15.0, net_color).params(),
    );
    y += 15.0;

    meter(
        Rect::new(x, y, panel.w - 28.0, 18.0),
        session.economy.food,
        data.balance.win_food_surplus,
        if session.economy.food > 15.0 {
            dark::POSITIVE
        } else {
            dark::NEGATIVE
        },
        Some(&format!(
            "Food {:.0}/{:.0}",
            session.economy.food, data.balance.win_food_surplus
        )),
    );
    y += 30.0;

    let (forecast, forecast_color) = match food::time_to_empty_seconds(session, data) {
        Some(seconds) => (
            format!("Forecast: empty in {}", format_mmss(seconds)),
            if seconds <= 120.0 {
                dark::WARNING
            } else {
                dark::NEGATIVE
            },
        ),
        None if net >= 0.0 => ("Forecast: reserve rising".to_owned(), dark::POSITIVE),
        None => ("Forecast: reserve empty".to_owned(), dark::NEGATIVE),
    };
    draw_ui_text_ex(
        &forecast,
        x,
        y,
        TextStyle::new(14.0, forecast_color).params(),
    );
    y += 20.0;

    // Chain throughput + haul pressure: the food grid generalised to a
    // factory dashboard (plan §Phase 9).
    let hauls = crate::ui::legibility::pending_hauls(session);
    draw_ui_text_ex(
        &format!(
            "Ore +{:.0}/m · Ingots +{:.0}/m · Hauls {hauls}",
            session.economy.ore_ema_per_min.max(0.0),
            session.economy.ingot_ema_per_min.max(0.0),
        ),
        x,
        y,
        TextStyle::new(14.0, dark::TEXT_DIM).params(),
    );
    y += 20.0;
    draw_ui_text_ex(
        &format!(
            "Ore banked {} · delivered {}/{}",
            session.economy.ore_stock,
            session.economy.ore_delivered_total,
            data.balance.win_ore_delivered
        ),
        x,
        y,
        TextStyle::new(14.0, dark::TEXT).params(),
    );
}

pub(super) fn draw_jobs_panel(
    session: &GameSession,
    data: &GameData,
    sprites: &HudSprites,
    panel: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_surface_with_title(
        panel,
        Some("Jobs"),
        &panel_style(),
        TextStyle::new(17.0, dark::TEXT),
    );

    let idle = session.job_count(Job::Idle);
    let idle_reassignable = reassignable_job_count(session, data, Job::Idle);
    let x = panel.x + 14.0;
    let mut y = panel.y + 44.0;

    let raid_warning = session.raid_active || session.raid_in <= data.balance.raid_warning_sec;
    for job in [Job::Miner, Job::Carrier, Job::Cook, Job::Smith, Job::Guard] {
        let count = session.job_count(job);
        if job == Job::Guard && raid_warning {
            draw_surface(
                Rect::new(x - 6.0, y - 3.0, panel.w - 20.0, 32.0),
                &SurfaceStyle::new(Color::new(0.24, 0.12, 0.08, 0.32))
                    .with_border(1.0, dark::WARNING),
            );
        }
        sprites.draw_job(job, vec2(x + 9.0, y + 13.0));
        draw_ui_text_ex(
            &format!("{} {count}", job.label()),
            x + 22.0,
            y + 19.0,
            TextStyle::new(16.0, dark::TEXT).params(),
        );
        let reassignable = reassignable_job_count(session, data, job);
        if hud_button(
            Rect::new(x + 130.0, y, 34.0, 26.0),
            "-",
            reassignable > 0,
            mouse,
        ) {
            actions.push(UiAction::Unassign(job));
        }
        if hud_button(
            Rect::new(x + 172.0, y, 34.0, 26.0),
            "+",
            idle_reassignable > 0,
            mouse,
        ) {
            actions.push(UiAction::Assign(job));
        }
        if job == Job::Guard && raid_warning {
            draw_ui_text_ex(
                "RAID",
                x + 84.0,
                y + 18.0,
                TextStyle::new(11.0, dark::WARNING).params(),
            );
        }
        y += 32.0;
    }

    sprites.draw_job(Job::Idle, vec2(x + 9.0, y + 13.0));
    draw_ui_text_ex(
        &format!("Idle {idle}"),
        x + 22.0,
        y + 18.0,
        TextStyle::new(16.0, dark::TEXT_DIM).params(),
    );
    y += 28.0;

    if !advanced_systems_unlocked(session) {
        draw_ui_text_ex(
            "Advanced systems unlock after Secure the warren.",
            x,
            y + 18.0,
            TextStyle::new(13.0, dark::TEXT_DIM).params(),
        );
        return;
    }

    let janitors = session
        .creatures
        .iter()
        .filter(|c| c.species == "slime_janitor")
        .count();
    let couriers = session
        .creatures
        .iter()
        .filter(|c| c.species == "bat_courier")
        .count();
    let half = (panel.w - 36.0) / 2.0;
    if hud_button(
        Rect::new(x, y, half, 30.0),
        &format!("Beetle ×5 ({})", data.balance.beetle_ore_cost),
        session.economy.ore_stock >= data.balance.beetle_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractBeetle);
    }
    let has_den = session.buildings_of("smelter").next().is_some();
    if hud_button(
        Rect::new(x + half + 8.0, y, half, 30.0),
        &format!("Salam. bulk ({})", data.balance.salamander_ore_cost),
        has_den && session.economy.ore_stock >= data.balance.salamander_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractSalamander);
    }
    y += 34.0;
    if hud_button(
        Rect::new(x, y, half, 30.0),
        optional_specialist_label("slime_janitor", janitors > 0),
        session.unlocked.contains("slime_janitor") && janitors == 0,
        mouse,
    ) {
        actions.push(UiAction::AttractSlimeJanitor);
    }
    if hud_button(
        Rect::new(x + half + 8.0, y, half, 30.0),
        optional_specialist_label("bat_courier", couriers > 0),
        session.unlocked.contains("bat_courier") && couriers == 0,
        mouse,
    ) {
        actions.push(UiAction::AttractBatCourier);
    }
    y += 32.0;
    draw_ui_text_ex(
        &format!(
            "Engineer {} · Mine +25% · Outposts {}/{}",
            session
                .creatures
                .iter()
                .filter(|c| c.species == "engineer")
                .count(),
            session.outposts.iter().filter(|o| o.active).count(),
            session.outposts.len()
        ),
        x,
        y + 18.0,
        TextStyle::new(13.0, dark::TEXT_DIM).params(),
    );

    let mut locked_actions = Vec::new();
    if !has_den {
        locked_actions.push("Salamander → build a Smelter Den".to_owned());
    }
    for (label, unlock_id) in [("Slime", "slime_janitor"), ("Bat", "bat_courier")] {
        if !session.unlocked.contains(unlock_id) {
            if let Some(requirement) = unlock_requirement(data, unlock_id) {
                locked_actions.push(format!("{label} → {requirement}"));
            }
        }
    }
    if !locked_actions.is_empty() {
        draw_ui_text_ex(
            "Locked actions",
            x,
            y + 36.0,
            TextStyle::new(12.0, dark::TEXT_DIM).params(),
        );
        for (index, requirement) in locked_actions.iter().enumerate() {
            draw_ui_text_ex(
                requirement,
                x,
                y + 52.0 + index as f32 * 16.0,
                TextStyle::new(12.0, dark::WARNING).params(),
            );
        }
    }
}

fn optional_specialist_label(species: &str, posted: bool) -> &'static str {
    match (species, posted) {
        ("slime_janitor", false) => "Slime · waste",
        ("slime_janitor", true) => "Slime · posted",
        ("bat_courier", false) => "Bat ×8",
        ("bat_courier", true) => "Bat · posted",
        _ => "Specialist",
    }
}

/// Keep a combined food/raid banner short while naming the visible Guard
/// control that resolves the incoming threat.
pub(super) fn compact_raid_defense_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Guard) > 0 {
        "guards on watch".to_owned()
    } else if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Guard".to_owned()
    } else {
        [Job::Miner, Job::Carrier, Job::Cook, Job::Smith]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap − {}, then + Guard", job.label()))
            .unwrap_or_else(|| "free a worker, then + Guard".to_owned())
    }
}

/// Shorten the opening response enough to share the top bar with its buttons.
/// The full control names remain in the tutorial card beside the banner.
pub(super) fn compact_food_recovery_hint(session: &GameSession, data: &GameData) -> String {
    if reassignable_job_count(session, data, Job::Idle) > 0 {
        "tap + Carrier or Cook".to_owned()
    } else {
        [Job::Miner, Job::Smith, Job::Guard]
            .into_iter()
            .find(|job| reassignable_job_count(session, data, *job) > 0)
            .map(|job| format!("tap − {}, then + Carrier", job.label()))
            .unwrap_or_else(|| "free a worker, then + Carrier".to_owned())
    }
}

fn reassignable_job_count(session: &GameSession, data: &GameData, job: Job) -> usize {
    session
        .creatures
        .iter()
        .filter(|creature| {
            !creature.is_remote()
                && creature.job == job
                && data
                    .species
                    .get(&creature.species)
                    .is_some_and(|species| species.reassignable)
        })
        .count()
}

#[cfg(test)]
mod tests;

pub(super) fn draw_tools_panel(
    session: &GameSession,
    data: &GameData,
    panel: Rect,
    mode: &UiMode,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_surface_with_title(
        panel,
        Some("Build & Dig"),
        &panel_style(),
        TextStyle::new(17.0, dark::TEXT),
    );

    let x = panel.x + 14.0;
    let mut y = panel.y + 28.0;
    let w = panel.w - 28.0;
    let cell = (w - 16.0) / 3.0;

    // Build buttons, two per row: label is the short name + cost. Locked
    // kinds stay visible but disabled (progression is discoverable).
    let mut defs: Vec<_> = data
        .buildings
        .iter()
        .filter(|(id, d)| {
            d.buildable && (advanced_systems_unlocked(session) || is_core_building(id))
        })
        .collect();
    defs.sort_by(|a, b| a.0.cmp(b.0));
    let mut locked_requirements = Vec::new();
    for row in defs.chunks(3) {
        for (i, (id, def)) in row.iter().enumerate() {
            let active = *mode == UiMode::Build((*id).clone());
            let unlocked = session.building_unlocked(def);
            let short = def.name.split_whitespace().last().unwrap_or(&def.name);
            let label = if unlocked {
                format!(
                    "{}{short} ({})",
                    if active { "▶ " } else { "" },
                    def.cost_ore
                )
            } else {
                format!("{short} 🔒")
            };
            if !unlocked {
                if let Some(unlock_id) = def.requires_unlock.as_deref() {
                    if let Some(requirement) = unlock_requirement(data, unlock_id) {
                        locked_requirements.push(format!("{short} → {requirement}"));
                    }
                }
            }
            let bx = x + (cell + 8.0) * i as f32;
            if hud_button(Rect::new(bx, y, cell, 22.0), &label, unlocked, mouse) {
                actions.push(UiAction::SetMode(UiMode::Build((*id).clone())));
            }
        }
        y += 26.0;
    }

    let dig_active = *mode == UiMode::Dig;
    let dig_label = if dig_active { "▶ Dig" } else { "Dig" };
    if hud_button(Rect::new(x, y, cell, 22.0), dig_label, true, mouse) {
        actions.push(UiAction::SetMode(UiMode::Dig));
    }
    // Show pending construction so hauling progress is visible.
    if !session.build_sites.is_empty() {
        let pending: u32 = session.build_sites.iter().map(|s| s.remaining()).sum();
        draw_ui_text_ex(
            &format!("{} site(s) · {} ore", session.build_sites.len(), pending),
            x + cell + 8.0,
            y + 16.0,
            TextStyle::new(13.0, dark::TEXT_DIM).params(),
        );
    }
    if !locked_requirements.is_empty() {
        draw_ui_text_ex(
            "Locked gates",
            x,
            y + 36.0,
            TextStyle::new(12.0, dark::TEXT_DIM).params(),
        );
        for (index, requirement) in locked_requirements.iter().enumerate() {
            draw_ui_text_ex(
                requirement,
                x,
                y + 52.0 + index as f32 * 16.0,
                TextStyle::new(12.0, dark::WARNING).params(),
            );
        }
    }
}

fn is_core_building(id: &str) -> bool {
    matches!(
        id,
        "blacksmith" | "cook_pot" | "farm" | "mine" | "worm_shrine"
    )
}

fn advanced_systems_unlocked(session: &GameSession) -> bool {
    (session.won && session.job_count(Job::Guard) > 0) || session.worm_awake
}

/// The persistent campaign card keeps the next milestone visible even after
/// the tutorial is skipped or completed.
pub(super) fn draw_objective_panel(session: &GameSession, data: &GameData) -> Rect {
    let objective = super::objective::CampaignObjective::current(session, data);
    let title = format!("Objective · {}", objective.title);
    draw_surface_with_title(
        OBJECTIVE_PANEL,
        Some(&title),
        &panel_style(),
        TextStyle::new(15.0, dark::TEXT_BRIGHT),
    );

    draw_ui_text_ex(
        &objective.progress,
        OBJECTIVE_PANEL.x + 14.0,
        OBJECTIVE_PANEL.y + 57.0,
        TextStyle::new(
            14.0,
            if objective.complete {
                dark::POSITIVE
            } else {
                dark::TEXT
            },
        )
        .params(),
    );
    let meter_color = if objective.complete {
        dark::POSITIVE
    } else {
        dark::ACCENT
    };
    meter(
        Rect::new(
            OBJECTIVE_PANEL.x + 14.0,
            OBJECTIVE_PANEL.y + 68.0,
            OBJECTIVE_PANEL.w - 28.0,
            12.0,
        ),
        objective.ratio.clamp(0.0, 1.0),
        1.0,
        meter_color,
        None,
    );
    draw_text_block(
        &objective.next,
        OBJECTIVE_PANEL.x + 14.0,
        OBJECTIVE_PANEL.y + 94.0,
        OBJECTIVE_PANEL.w - 28.0,
        34.0,
        13.0,
        3.0,
        dark::TEXT_DIM,
    );

    OBJECTIVE_PANEL
}

/// The tutorial card, top-right: current step, progress chip, and a skip
/// button. Returns its rect while visible (for pointer-over-UI checks).
pub(super) fn draw_tutorial_panel(
    session: &GameSession,
    data: &GameData,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) -> Option<Rect> {
    let step = crate::tutorial::current_step(session, data)?;
    let (done, total) = crate::tutorial::progress(session, data);

    let panel = Rect::new(LOGICAL_WIDTH - 342.0, 72.0, 330.0, 128.0);
    draw_surface_with_title(
        panel,
        Some(&format!("Tutorial {}/{} — {}", done + 1, total, step.title)),
        &panel_style(),
        TextStyle::new(15.0, dark::TEXT_BRIGHT),
    );

    let body = tutorial_body(step, session, data);
    draw_text_block(
        &body,
        panel.x + 14.0,
        panel.y + 42.0,
        panel.w - 28.0,
        56.0,
        14.0,
        4.0,
        dark::TEXT,
    );

    if hud_button(
        Rect::new(panel.right() - 78.0, panel.bottom() - 30.0, 64.0, 22.0),
        "Skip",
        true,
        mouse,
    ) {
        actions.push(UiAction::SkipTutorial);
    }

    Some(panel)
}

fn tutorial_body(
    step: &crate::data::TutorialStepDef,
    session: &GameSession,
    data: &GameData,
) -> String {
    match step.id.as_str() {
        "food" => format!(
            "Read Food Grid: keep Production above Upkeep. Tap Farm, then open floor. Wait for Farm construction. If food pressure rises, {}.",
            super::objective::job_assignment_action_hint(
                session,
                data,
                Job::Carrier,
                &[Job::Miner, Job::Smith, Job::Guard],
            )
        ),
        "factory" => format!(
            "Tap the existing Mine to read its rate. Place a Blacksmith. To staff it, {}. Tap Blacksmith, then Iron Pickaxe; the miner equips it and the Mine speeds up.",
            super::objective::job_assignment_action_hint(
                session,
                data,
                Job::Smith,
                &[Job::Miner, Job::Carrier, Job::Cook, Job::Guard],
            )
        ),
        "secure" => format!(
            "Deliver 50 ore and hold 100 food in Objective. Before a raid, {}. This secures the warren and ends onboarding; next, forge 20 ingots and raise the Shrine.",
            super::objective::security_handoff_action_hint(session, data)
        ),
        _ => step.body.clone(),
    }
}
