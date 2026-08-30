//! The standing HUD chrome: the top bar, the calorie balance meter, the
//! job assignment panel, the build/dig tools, and the tutorial card.

mod food;
mod specialists;
mod top_bar;
mod workforce;

pub(super) use food::draw_food_grid_panel;
use specialists::{optional_specialist_label, optional_support_label};
pub(super) use top_bar::{
    compact_food_recovery_hint, compact_raid_defense_hint, compact_top_bar,
    condensed_food_recovery_hint, condensed_raid_defense_hint, reassignable_job_count,
};
use workforce::{engineer_status_label, workforce_capacity_label, workforce_pressure_label};

use crate::data::GameData;
use crate::simulation::{self, food as simulation_food};
use crate::state::creatures::Job;
use crate::state::GameSession;
use crate::ui::hud::requirements::unlock_requirement_progress;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::hud::HudSprites;
use crate::ui::legibility::advanced_systems_unlocked;
use crate::ui::{UiAction, UiMode, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, format_mmss};

const OBJECTIVE_PANEL: Rect = Rect::new(548.0, 72.0, 370.0, 128.0);
const TUTORIAL_PANEL_HEIGHT: f32 = 168.0;
const TUTORIAL_BODY_HEIGHT: f32 = 94.0;
const LOCKED_TOOL_MARKER: &str = "[L]";

#[derive(Debug, Clone, Copy)]
pub(super) struct TopBarState<'a> {
    pub(super) paused: bool,
    pub(super) checkpoint_warning: Option<&'a str>,
}

pub(super) fn draw_top_bar(
    session: &GameSession,
    data: &GameData,
    bar: Rect,
    mouse: Vec2,
    ui_scale: f32,
    top_bar_state: TopBarState<'_>,
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

    let compact = compact_top_bar(ui_scale);
    if let Some(warning) = top_bar_state.checkpoint_warning {
        draw_ui_text_ex(
            warning,
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::NEGATIVE).params(),
        );
    } else if top_bar_state.paused {
        draw_ui_text_ex(
            if compact {
                "PAUSED — tap Resume"
            } else {
                "PAUSED — tap Resume to continue"
            },
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::WARNING).params(),
        );
    } else if let Some(transit) = &session.worm_transit {
        let transit_label = if compact {
            format!("WORM TRANSIT — {:.0}s", transit.remaining.max(0.0))
        } else {
            format!(
                "WORM TRANSIT — {:.0}s remaining",
                transit.remaining.max(0.0)
            )
        };
        draw_ui_text_ex(
            &transit_label,
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::POSITIVE).params(),
        );
    } else if session.last_transit_failure.is_some() {
        draw_ui_text_ex(
            if compact {
                "ROUTE FAILED — tap Outpost"
            } else {
                "WORM ROUTE FAILED — inspect the outpost"
            },
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::NEGATIVE).params(),
        );
    } else if session.raid_active {
        let food_warning = simulation_food::time_to_empty_seconds(session, data)
            .filter(|seconds| *seconds <= data.balance.food_warning_sec);
        let food_suffix = if session.economy.food <= 0.0 {
            " · FAMINE".to_owned()
        } else if let Some(seconds) = food_warning {
            if compact {
                format!(" · FOOD {}", format_mmss(seconds))
            } else {
                format!(" · FOOD IN {}", format_mmss(seconds))
            }
        } else {
            String::new()
        };
        let raid_label = if compact {
            format!(
                "RAID — {}{}",
                condensed_raid_defense_hint(session, data),
                food_suffix
            )
        } else {
            format!(
                "RAID — gnarls are after the larder! {}{}",
                compact_raid_defense_hint(session, data),
                food_suffix
            )
        };
        draw_ui_text_ex(
            &raid_label,
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::NEGATIVE).params(),
        );
    } else if session.economy.food <= 0.0 {
        draw_ui_text_ex(
            if compact {
                "FAMINE — workers slowing"
            } else {
                "FAMINE — workers are slowing down"
            },
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(18.0, dark::NEGATIVE).params(),
        );
    } else if let Some(seconds) = simulation_food::time_to_empty_seconds(session, data)
        .filter(|seconds| *seconds <= data.balance.food_warning_sec)
    {
        if session.raid_in <= data.balance.raid_warning_sec {
            let warning_label = if compact {
                format!(
                    "FOOD {} · RAID {} · {}",
                    format_mmss(seconds),
                    format_mmss(session.raid_in.max(0.0)),
                    condensed_raid_defense_hint(session, data)
                )
            } else {
                format!(
                    "FOOD IN {} · RAID IN {} · {}",
                    format_mmss(seconds),
                    format_mmss(session.raid_in.max(0.0)),
                    compact_raid_defense_hint(session, data)
                )
            };
            draw_ui_text_ex(
                &warning_label,
                bar.x + 380.0,
                bar.y + 31.0,
                TextStyle::new(15.0, dark::WARNING).params(),
            );
        } else {
            let warning_label = if compact {
                format!(
                    "FOOD {} · {}",
                    format_mmss(seconds),
                    condensed_food_recovery_hint(session, data)
                )
            } else {
                format!(
                    "FOOD IN {} · {}",
                    format_mmss(seconds),
                    compact_food_recovery_hint(session, data)
                )
            };
            draw_ui_text_ex(
                &warning_label,
                bar.x + 380.0,
                bar.y + 31.0,
                TextStyle::new(15.0, dark::WARNING).params(),
            );
        }
    } else if session.raid_in <= data.balance.raid_warning_sec {
        let warning_label = if compact {
            format!(
                "RAID {} · {}",
                format_mmss(session.raid_in.max(0.0)),
                condensed_raid_defense_hint(session, data)
            )
        } else {
            format!(
                "RAID IN {} — {}",
                format_mmss(session.raid_in.max(0.0)),
                compact_raid_defense_hint(session, data)
            )
        };
        draw_ui_text_ex(
            &warning_label,
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::WARNING).params(),
        );
    } else {
        draw_ui_text_ex(
            "Drag map · +/− zoom · tap Help",
            bar.x + 380.0,
            bar.y + 31.0,
            TextStyle::new(15.0, dark::TEXT_DIM).params(),
        );
    }

    let chrome_y = if compact { bar.y + 4.0 } else { bar.y + 8.0 };
    let chrome_height = if compact { 40.0 } else { 32.0 };
    if hud_button(
        Rect::new(bar.right() - 96.0, chrome_y, 84.0, chrome_height),
        "Menu",
        true,
        mouse,
    ) {
        actions.push(UiAction::BackToMenu);
    }
    if hud_button(
        Rect::new(bar.right() - 512.0, chrome_y, 74.0, chrome_height),
        if top_bar_state.paused {
            "Resume"
        } else {
            "Pause"
        },
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
        Rect::new(bar.right() - 254.0, chrome_y, 74.0, chrome_height),
        "Save",
        true,
        mouse,
    ) {
        actions.push(UiAction::Save);
    }
    if hud_button(
        Rect::new(bar.right() - 176.0, chrome_y, 74.0, chrome_height),
        "Load",
        true,
        mouse,
    ) {
        actions.push(UiAction::Load);
    }
    if hud_button(
        Rect::new(bar.right() - 438.0, chrome_y, 74.0, chrome_height),
        "Help",
        true,
        mouse,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
    if session.worm_awake
        && !session.outposts.is_empty()
        && hud_button(
            Rect::new(bar.right() - 606.0, chrome_y, 74.0, chrome_height),
            "Routes",
            true,
            mouse,
        )
    {
        actions.push(UiAction::ToggleRoutes);
    }
}

pub(super) fn draw_jobs_panel(
    session: &GameSession,
    data: &GameData,
    sprites: &HudSprites,
    panel: Rect,
    mouse: Vec2,
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) {
    draw_surface_with_title(
        panel,
        Some("Jobs"),
        &panel_style(),
        TextStyle::new(17.0, dark::TEXT),
    );

    let idle_reassignable = reassignable_job_count(session, data, Job::Idle);
    let x = panel.x + 14.0;
    let mut y = panel.y + 44.0;
    let compact = compact_top_bar(ui_scale);
    let job_button_height = if compact { 30.0 } else { 26.0 };
    let job_row_step = if compact { 36.0 } else { 32.0 };

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
            Rect::new(x + 130.0, y, 34.0, job_button_height),
            "-",
            reassignable > 0,
            mouse,
        ) {
            actions.push(UiAction::Unassign(job));
        }
        if hud_button(
            Rect::new(x + 172.0, y, 34.0, job_button_height),
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
        y += job_row_step;
    }

    sprites.draw_job(Job::Idle, vec2(x + 9.0, y + 13.0));
    draw_ui_text_ex(
        &workforce_capacity_label(session, data),
        x + 22.0,
        y + 18.0,
        TextStyle::new(16.0, dark::TEXT_DIM).params(),
    );
    let workforce_pressure = workforce_pressure_label(session, data);
    if let Some(pressure) = workforce_pressure.as_deref() {
        draw_ui_text_ex(
            pressure,
            x + 22.0,
            y + 36.0,
            TextStyle::new(13.0, dark::WARNING).params(),
        );
    }
    y += if workforce_pressure.is_some() {
        46.0
    } else {
        28.0
    };

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
    let local_beetles = session
        .creatures
        .iter()
        .filter(|c| !c.is_remote() && c.species == "beetle")
        .count();
    let local_salamanders = session
        .creatures
        .iter()
        .filter(|c| !c.is_remote() && c.species == "salamander")
        .count();
    let half = (panel.w - 36.0) / 2.0;
    let specialist_height = if compact { 34.0 } else { 30.0 };
    let specialist_step = if compact { 38.0 } else { 34.0 };
    if hud_button(
        Rect::new(x, y, half, specialist_height),
        &optional_support_label("beetle", data.balance.beetle_ore_cost, local_beetles),
        session.economy.ore_stock >= data.balance.beetle_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractBeetle);
    }
    let has_den = session.buildings_of("smelter").next().is_some();
    if hud_button(
        Rect::new(x + half + 8.0, y, half, specialist_height),
        &optional_support_label(
            "salamander",
            data.balance.salamander_ore_cost,
            local_salamanders,
        ),
        has_den && session.economy.ore_stock >= data.balance.salamander_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractSalamander);
    }
    y += specialist_step;
    if hud_button(
        Rect::new(x, y, half, specialist_height),
        optional_specialist_label("slime_janitor", janitors > 0),
        session.unlocked.contains("slime_janitor") && janitors == 0,
        mouse,
    ) {
        actions.push(UiAction::AttractSlimeJanitor);
    }
    if hud_button(
        Rect::new(x + half + 8.0, y, half, specialist_height),
        optional_specialist_label("bat_courier", couriers > 0),
        session.unlocked.contains("bat_courier") && couriers == 0,
        mouse,
    ) {
        actions.push(UiAction::AttractBatCourier);
    }
    y += specialist_step;
    let engineers = session
        .creatures
        .iter()
        .filter(|c| c.species == "engineer")
        .count();
    let local_engineers = session
        .creatures
        .iter()
        .filter(|c| !c.is_remote() && c.species == "engineer")
        .count();
    draw_ui_text_ex(
        &format!(
            "{} · Outposts {}/{}",
            engineer_status_label(local_engineers, engineers),
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
            if let Some(requirement) = unlock_requirement_progress(session, data, unlock_id) {
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

#[cfg(test)]
mod tests;

pub(super) fn draw_tools_panel(
    session: &GameSession,
    data: &GameData,
    panel: Rect,
    mode: &UiMode,
    mouse: Vec2,
    ui_scale: f32,
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
    let compact = compact_top_bar(ui_scale);
    let tool_button_height = if compact { 30.0 } else { 22.0 };
    let tool_row_step = if compact { 34.0 } else { 26.0 };

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
                format!("{}{short} ({})", active_tool_marker(active), def.cost_ore)
            } else {
                format!("{short} {LOCKED_TOOL_MARKER}")
            };
            if !unlocked {
                if let Some(unlock_id) = def.requires_unlock.as_deref() {
                    if let Some(requirement) = unlock_requirement_progress(session, data, unlock_id)
                    {
                        locked_requirements.push(format!("{short} → {requirement}"));
                    }
                }
            }
            let bx = x + (cell + 8.0) * i as f32;
            if hud_button(
                Rect::new(bx, y, cell, tool_button_height),
                &label,
                unlocked,
                mouse,
            ) {
                actions.push(UiAction::SetMode(UiMode::Build((*id).clone())));
            }
        }
        y += tool_row_step;
    }

    let dig_active = *mode == UiMode::Dig;
    let dig_label = format!("{}Dig", active_tool_marker(dig_active));
    if hud_button(
        Rect::new(x, y, cell, tool_button_height),
        &dig_label,
        true,
        mouse,
    ) {
        actions.push(UiAction::SetMode(UiMode::Dig));
    }
    // Show pending construction so hauling progress is visible.
    if !session.build_sites.is_empty() {
        let pending: u32 = session.build_sites.iter().map(|s| s.remaining()).sum();
        draw_ui_text_ex(
            &construction_progress_label(session.build_sites.len(), pending),
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

fn active_tool_marker(active: bool) -> &'static str {
    if active {
        "> "
    } else {
        ""
    }
}

fn construction_progress_label(site_count: usize, ore_remaining: u32) -> String {
    let noun = if site_count == 1 { "site" } else { "sites" };
    format!("Build {site_count} {noun} · {ore_remaining} ore left")
}

fn is_core_building(id: &str) -> bool {
    matches!(
        id,
        "blacksmith" | "cook_pot" | "farm" | "mine" | "worm_shrine"
    )
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
    ui_scale: f32,
    actions: &mut Vec<UiAction>,
) -> Option<Rect> {
    let step = crate::tutorial::current_step(session, data)?;
    let (done, total) = crate::tutorial::progress(session, data);

    let panel = Rect::new(LOGICAL_WIDTH - 342.0, 72.0, 330.0, TUTORIAL_PANEL_HEIGHT);
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
        TUTORIAL_BODY_HEIGHT,
        14.0,
        4.0,
        dark::TEXT,
    );

    let skip_height = if compact_top_bar(ui_scale) {
        30.0
    } else {
        22.0
    };
    if hud_button(
        Rect::new(
            panel.right() - 78.0,
            panel.bottom() - 30.0,
            64.0,
            skip_height,
        ),
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
