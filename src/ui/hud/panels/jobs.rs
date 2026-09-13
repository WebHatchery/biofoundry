//! Job assignment panel rendering and its touch targets.

use super::*;

pub(crate) fn draw_jobs_panel(
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
    let compact = compact_top_bar(ui_scale);
    let mut y = panel.y + if compact { 40.0 } else { 44.0 };
    let job_button_height = if compact { 72.0 } else { 26.0 };
    let job_row_step = if compact { 72.0 } else { 32.0 };

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
        sprites.draw_job(
            job,
            vec2(
                x + 9.0,
                y + if compact {
                    job_button_height * 0.5
                } else {
                    13.0
                },
            ),
        );
        draw_ui_text_ex(
            &format!("{} {count}", job.label()),
            x + 22.0,
            y + if compact { 43.0 } else { 19.0 },
            TextStyle::new(16.0, dark::TEXT).params(),
        );
        let reassignable = reassignable_job_count(session, data, job);
        if hud_button(
            Rect::new(
                x + if compact { 93.0 } else { 130.0 },
                y,
                if compact { 72.0 } else { 34.0 },
                job_button_height,
            ),
            "-",
            reassignable > 0,
            mouse,
        ) {
            actions.push(UiAction::Unassign(job));
        }
        if hud_button(
            Rect::new(
                x + if compact { 167.0 } else { 172.0 },
                y,
                if compact { 72.0 } else { 34.0 },
                job_button_height,
            ),
            "+",
            idle_reassignable > 0,
            mouse,
        ) {
            actions.push(UiAction::Assign(job));
        }
        if job == Job::Guard && raid_warning && !compact {
            draw_ui_text_ex(
                "RAID",
                x + 84.0,
                y + 18.0,
                TextStyle::new(11.0, dark::WARNING).params(),
            );
        }
        y += job_row_step;
    }

    sprites.draw_job(
        Job::Idle,
        vec2(x + 9.0, y + if compact { 35.0 } else { 13.0 }),
    );
    draw_ui_text_ex(
        &workforce_capacity_label(session, data),
        x + 22.0,
        y + if compact { 41.0 } else { 18.0 },
        TextStyle::new(16.0, dark::TEXT_DIM).params(),
    );
    let workforce_pressure = workforce_pressure_label(session, data);
    if let Some(pressure) = workforce_pressure.as_deref() {
        draw_ui_text_ex(
            pressure,
            x + 22.0,
            y + if compact { 59.0 } else { 36.0 },
            TextStyle::new(13.0, dark::WARNING).params(),
        );
    }
    y += if workforce_pressure.is_some() {
        if compact {
            70.0
        } else {
            46.0
        }
    } else {
        if compact {
            58.0
        } else {
            28.0
        }
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
        &optional_support_button_label("beetle", data.balance.beetle_ore_cost, local_beetles, data),
        session.economy.ore_stock >= data.balance.beetle_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractBeetle);
    }
    let has_den = session.buildings_of("smelter").next().is_some();
    if hud_button(
        Rect::new(x + half + 8.0, y, half, specialist_height),
        &optional_support_button_label(
            "salamander",
            data.balance.salamander_ore_cost,
            local_salamanders,
            data,
        ),
        has_den && session.economy.ore_stock >= data.balance.salamander_ore_cost,
        mouse,
    ) {
        actions.push(UiAction::AttractSalamander);
    }
    y += specialist_step;
    if hud_button(
        Rect::new(x, y, half, specialist_height),
        &optional_specialist_button_label("slime_janitor", janitors > 0, data),
        session.unlocked.contains("slime_janitor") && janitors == 0,
        mouse,
    ) {
        actions.push(UiAction::AttractSlimeJanitor);
    }
    if hud_button(
        Rect::new(x + half + 8.0, y, half, specialist_height),
        &optional_specialist_button_label("bat_courier", couriers > 0, data),
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
