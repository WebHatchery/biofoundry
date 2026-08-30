//! Persisted remote logistics state for the Worm Shrine's outposts.

use crate::state::creatures::Good;
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The order in which an outbound worm route fills its limited cargo hold.
/// The default preserves the original route behavior: ore, then ingots, then
/// food above the reserve.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CargoPriority {
    #[default]
    Ore,
    Ingots,
    Food,
}

impl CargoPriority {
    pub fn next(self) -> Self {
        match self {
            Self::Ore => Self::Ingots,
            Self::Ingots => Self::Food,
            Self::Food => Self::Ore,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ore => "Ore first",
            Self::Ingots => "Ingots first",
            Self::Food => "Food first",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outpost {
    pub pos: TilePos,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub cargo_priority: CargoPriority,
    #[serde(default)]
    pub cargo: HashMap<Good, u32>,
    #[serde(default)]
    pub crew: Vec<u32>,
    /// Whether remote scouting is paused while the route remains active.
    #[serde(default)]
    pub expedition_paused: bool,
    /// Seconds accumulated toward the next remote scouting haul.
    #[serde(default)]
    pub expedition_progress: f32,
    /// Number of completed scouting hauls at this remote route.
    #[serde(default)]
    pub expeditions_completed: u32,
    /// Lifetime ore returned by this route's scouting expeditions.
    #[serde(default)]
    pub ore_scouted: u32,
    #[serde(default)]
    pub last_failure: Option<String>,
}

impl Outpost {
    pub fn new(pos: TilePos) -> Self {
        Self {
            pos,
            active: false,
            cargo_priority: CargoPriority::default(),
            cargo: HashMap::new(),
            crew: Vec::new(),
            expedition_paused: false,
            expedition_progress: 0.0,
            expeditions_completed: 0,
            ore_scouted: 0,
            last_failure: None,
        }
    }

    pub fn cargo_total(&self) -> u32 {
        self.cargo.values().sum()
    }

    pub fn cycle_cargo_priority(&mut self) {
        self.cargo_priority = self.cargo_priority.next();
    }

    pub fn toggle_expedition(&mut self) {
        self.expedition_paused = !self.expedition_paused;
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitDirection {
    ToOutpost,
    ToShrine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitCompletion {
    pub direction: TransitDirection,
    pub cargo_units: u32,
    pub passenger_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpeditionCompletion {
    pub outpost: TilePos,
    pub ore: u32,
    pub food_spent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormTransit {
    pub outpost: TilePos,
    pub direction: TransitDirection,
    pub remaining: f32,
    pub ore: u32,
    pub ingots: u32,
    pub food: f32,
    #[serde(default)]
    pub passengers: Vec<u32>,
}
