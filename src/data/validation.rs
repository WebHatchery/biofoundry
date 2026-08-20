//! Semantic validation for the embedded content tables.

use super::{GameData, TutorialDone};
use std::collections::HashSet;

const WORKSTATION_JOBS: &[&str] = &["miner", "smith"];
const UNLOCK_COUNTERS: &[&str] = &[
    "beetles_captured",
    "raids_survived",
    "famines_survived",
    "specimens",
    "knowledge",
    "waste_processed",
    "courier_deliveries",
    "ingots_forged",
    "ore_delivered_total",
];

pub(super) fn validate(data: &GameData) -> Result<(), String> {
    validate_config(data)?;
    validate_registry_ids(data)?;
    validate_building_references(data)?;
    validate_unlocks(data)?;
    validate_equipment(data)?;
    validate_tutorial(data)?;

    for required_species in ["goblin"] {
        if !data.species.contains(required_species) {
            return Err(format!(
                "species data is missing required id '{required_species}'"
            ));
        }
    }
    for required_building in ["stockpile", "cook_pot", "farm"] {
        if !data.buildings.contains(required_building) {
            return Err(format!(
                "building data is missing required id '{required_building}'"
            ));
        }
    }
    Ok(())
}

fn validate_config(data: &GameData) -> Result<(), String> {
    let config = &data.config;
    if config.game_name.trim().is_empty() || config.save_slot.trim().is_empty() {
        return Err("game config requires a game name and save slot".to_owned());
    }
    if config.world_width == 0 || config.world_height == 0 || config.tile_size <= 0.0 {
        return Err("game config must define a non-empty world and positive tile size".to_owned());
    }
    if !config.tile_size.is_finite() {
        return Err("game config tile size must be finite".to_owned());
    }
    Ok(())
}

fn validate_registry_ids(data: &GameData) -> Result<(), String> {
    for (key, species) in data.species.iter() {
        if key.trim().is_empty() || species.id != key.as_str() {
            return Err(format!(
                "species registry key '{key}' does not match its id"
            ));
        }
    }
    for (key, building) in data.buildings.iter() {
        if key.trim().is_empty() || building.id != key.as_str() {
            return Err(format!(
                "building registry key '{key}' does not match its id"
            ));
        }
    }
    Ok(())
}

fn validate_building_references(data: &GameData) -> Result<(), String> {
    for building in data.buildings.iter().map(|(_, building)| building) {
        if let Some(unlock) = &building.requires_unlock {
            if !data.unlocks.iter().any(|candidate| candidate.id == *unlock) {
                return Err(format!(
                    "building '{}' references missing unlock '{unlock}'",
                    building.id
                ));
            }
        }
        if let Some(workstation) = &building.workstation {
            if workstation.job.trim().is_empty()
                || !WORKSTATION_JOBS.contains(&workstation.job.as_str())
                || workstation.slots == 0
            {
                return Err(format!(
                    "building '{}' has an invalid workstation job or slot count",
                    building.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_unlocks(data: &GameData) -> Result<(), String> {
    let mut ids = HashSet::new();
    for unlock in &data.unlocks {
        if unlock.id.trim().is_empty() || !ids.insert(&unlock.id) {
            return Err(format!("unlock ids must be unique: '{}'", unlock.id));
        }
        if unlock.threshold == 0 {
            return Err(format!(
                "unlock '{}' must have a positive threshold",
                unlock.id
            ));
        }
        if !UNLOCK_COUNTERS.contains(&unlock.counter.as_str()) {
            return Err(format!(
                "unlock '{}' references unknown counter '{}'",
                unlock.id, unlock.counter
            ));
        }
        match unlock.effect.as_str() {
            "unlock_building" if unlock.building.is_none() => {
                return Err(format!(
                    "building unlock '{}' must name a building",
                    unlock.id
                ));
            }
            "guard_dps_mult" | "farm_cap_mult" | "unlock_creature" if unlock.building.is_some() => {
                return Err(format!(
                    "non-building unlock '{}' must not name a building",
                    unlock.id
                ));
            }
            "unlock_building" | "guard_dps_mult" | "farm_cap_mult" | "unlock_creature" => {}
            effect => {
                return Err(format!(
                    "unlock '{}' has unknown effect '{effect}'",
                    unlock.id
                ));
            }
        }
        if let Some(building) = &unlock.building {
            if !data.buildings.contains(building) {
                return Err(format!(
                    "unlock '{}' references missing building '{building}'",
                    unlock.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_equipment(data: &GameData) -> Result<(), String> {
    let mut ids = HashSet::new();
    for equipment in &data.equipment {
        if equipment.id.trim().is_empty() || !ids.insert(&equipment.id) {
            return Err(format!("equipment ids must be unique: '{}'", equipment.id));
        }
        if equipment.job.trim().is_empty() || equipment.effect.trim().is_empty() {
            return Err(format!(
                "equipment '{}' must define a job and effect",
                equipment.id
            ));
        }
        if equipment.cost_ingots == 0 || !equipment.value.is_finite() || equipment.value <= 0.0 {
            return Err(format!(
                "equipment '{}' must have a positive cost and finite value",
                equipment.id
            ));
        }
    }
    Ok(())
}

fn validate_tutorial(data: &GameData) -> Result<(), String> {
    let mut ids = HashSet::new();
    for step in &data.tutorial {
        if step.id.trim().is_empty() || !ids.insert(&step.id) {
            return Err(format!("tutorial step ids must be unique: '{}'", step.id));
        }
        match &step.done {
            TutorialDone::BuildingPlaced { building } => {
                if !data.buildings.contains(building) {
                    return Err(format!(
                        "tutorial step '{}' references missing building '{building}'",
                        step.id
                    ));
                }
            }
            TutorialDone::GearCrafted { item } => {
                if data.equipment_def(item).is_none() {
                    return Err(format!(
                        "tutorial step '{}' references missing equipment '{item}'",
                        step.id
                    ));
                }
            }
            TutorialDone::FamineRecovered { value } if !value.is_finite() || *value < 0.0 => {
                return Err(format!(
                    "tutorial step '{}' has an invalid famine recovery value",
                    step.id
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
