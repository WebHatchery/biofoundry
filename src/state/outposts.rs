//! Persisted remote logistics state for the Worm Shrine's outposts.

use crate::state::creatures::Good;
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outpost {
    pub pos: TilePos,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub cargo: HashMap<Good, u32>,
    #[serde(default)]
    pub crew: Vec<u32>,
    #[serde(default)]
    pub last_failure: Option<String>,
}

impl Outpost {
    pub fn new(pos: TilePos) -> Self {
        Self {
            pos,
            active: false,
            cargo: HashMap::new(),
            crew: Vec::new(),
            last_failure: None,
        }
    }

    pub fn cargo_total(&self) -> u32 {
        self.cargo.values().sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitDirection {
    ToOutpost,
    ToShrine,
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
