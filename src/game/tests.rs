use super::*;
use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::{ExpeditionCompletion, TransitCompletion, TransitDirection};
use crate::state::structures::BuildSite;
use crate::state::GameSession;

fn session() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn old_tutorial_index_does_not_skip_the_new_factory_lesson() {
    let (data, mut session) = session();
    session.tutorial_step = 4; // old Blacksmith/famine territory
    session.tutorial_built = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, 2);
}

#[test]
fn tutorial_migration_keeps_a_pending_farm_lesson_visible() {
    let (data, mut session) = session();
    session.tutorial_step = 1;
    session.tutorial_built = true;
    session.build_sites.push(BuildSite {
        kind: "farm".to_owned(),
        pos: session.spawn_tile(),
        ore_needed: data.buildings.get("farm").unwrap().cost_ore,
        ore_delivered: 0,
    });

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, 1);
}

#[test]
fn tutorial_migration_uses_completed_campaign_facts() {
    let (data, mut session) = session();
    session.tutorial_step = 6;
    session.tutorial_built = true;
    session.won = true;
    session.worm_awake = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, data.tutorial.len());
}

#[test]
fn tutorial_migration_keeps_unwitnessed_guard_lesson_visible() {
    let (data, mut session) = session();
    session.tutorial_built = true;
    session
        .economy
        .gear_stock
        .insert("iron_pickaxe".to_owned(), 1);
    session.won = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 3);

    session.creatures[0].job = Job::Guard;
    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 4);
}

#[test]
fn non_viable_save_notice_protects_the_last_checkpoint() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(
        non_viable_save_notice(&session, &data),
        Some("This warren cannot staff the Guard post. Load a safe save or start a new warren instead.")
    );
}

#[test]
fn viable_save_notice_stays_empty_for_a_recoverable_warren() {
    let (data, mut session) = session();
    session.won = true;

    assert_eq!(non_viable_save_notice(&session, &data), None);
}

#[test]
fn goal_modal_holds_the_simulation_until_the_report_is_dismissed() {
    let (data, mut session) = session();
    session.won = true;

    assert!(simulation_blocked_by_modal(&session, &data, false));

    session.victory_shown = true;
    session.factory_complete = true;
    assert!(simulation_blocked_by_modal(&session, &data, false));

    session.factory_shown = true;
    session.worm_awake = true;
    assert!(simulation_blocked_by_modal(&session, &data, false));

    session.worm_shown = true;
    assert!(!simulation_blocked_by_modal(&session, &data, false));
}

#[test]
fn field_guide_pauses_a_viable_warren_while_open() {
    let (data, session) = session();

    assert!(simulation_blocked_by_modal(&session, &data, true));
    assert!(!simulation_blocked_by_modal(&session, &data, false));
}

#[test]
fn non_viable_recovery_stops_the_remaining_specialists() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert!(simulation_blocked_by_modal(&session, &data, false));
}

#[test]
fn secure_threshold_notice_waits_for_the_guard_handoff() {
    let (_data, mut session) = session();

    assert_eq!(
        warren_secured_notice(&session),
        "The reserve gate is secure — assign a Guard to finish onboarding."
    );

    session.creatures[0].job = Job::Guard;
    assert_eq!(
        warren_secured_notice(&session),
        "The warren is secure — onboarding complete."
    );
}

#[test]
fn transit_completion_notice_names_a_crew_only_arrival() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToOutpost,
        cargo_units: 0,
        passenger_count: 2,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm reaches the outpost — crew delivered."
    );
}

#[test]
fn expedition_completion_notice_names_the_remote_yield_and_upkeep() {
    assert_eq!(
        format_expedition_completion(ExpeditionCompletion {
            outpost: macroquad_toolkit::grid::TilePos::new(4, 4),
            ore: 6,
            food_spent: 2,
        }),
        "Outpost haul · +6 ore / -2 food."
    );
}

#[test]
fn auto_return_notice_names_the_remote_team_outcome() {
    assert_eq!(
        auto_return_notice(),
        "Outpost hold full — cargo returning while scouts remain remote."
    );
}

#[test]
fn auto_resupply_notice_names_the_food_only_transit() {
    assert_eq!(
        auto_resupply_notice(),
        "Outpost scouts need food — a food-only resupply is on its way."
    );
}

#[test]
fn transit_failure_notice_points_to_route_recovery() {
    assert_eq!(
        transit_failure_notice(),
        "The worm route failed — tap the outpost, then reactivate the route before trying again."
    );
}

#[test]
fn transit_completion_notice_names_mixed_payloads() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToShrine,
        cargo_units: 3,
        passenger_count: 1,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm returns to the shrine — cargo and crew delivered."
    );
}

#[test]
fn unlock_notice_teaches_how_to_use_new_content() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Beetle Breeding Pit"),
        "Unlocked: Beetle Breeding Pit — available in Build & Dig after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Hobgoblin Brood"),
        "Unlocked: Hobgoblin Brood — breed at the Breeding Pit after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Slime Janitor"),
        "Unlocked: Slime Janitor — recruit from Jobs after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Bat Courier"),
        "Unlocked: Bat Courier — recruit from Jobs after onboarding."
    );
}

#[test]
fn unlock_notice_switches_to_visible_optional_controls_after_onboarding() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures[0].job = Job::Guard;

    assert_eq!(
        unlock_notice(&data, &session, "Beetle Breeding Pit"),
        "Unlocked: Beetle Breeding Pit — build Breeding Pit from Build & Dig."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Slime Janitor"),
        "Unlocked: Slime Janitor — recruit from Jobs."
    );
}

#[test]
fn unlock_notice_teaches_passive_benefits() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Hardened Guards"),
        "Unlocked: Hardened Guards — Guards deal +50% damage."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Preservation Techniques"),
        "Unlocked: Preservation Techniques — Farms hold +50% food."
    );
}

#[test]
fn unlock_notice_keeps_unknown_names_safe() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Future discovery"),
        "Unlocked: Future discovery"
    );
}
