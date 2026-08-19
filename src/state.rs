//! Game state machine and the live warren session.
//!
//! Only one `GameState` is active at a time; states signal changes by
//! returning a `StateTransition`, which `Game::transition` applies
//! explicitly. Simulation state lives in `GameSession` and is only mutated
//! by `simulation` services and dispatched `UiAction`s.

pub mod creatures;
pub mod outposts;
pub mod serde_helpers;
pub mod structures;
pub mod wildlife;
pub mod world;

use crate::data::{Balance, GameData};
use creatures::{Creature, Job};
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::rng::SeededRng;
use outposts::{Outpost, WormTransit};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use structures::{BuildSite, Building};
use world::{Tile, WorldMap};

/// Which screen is running. Session data is owned by the active state.
pub enum GameState {
    Menu,
    Warren(Box<GameSession>),
}

/// Explicit state changes returned by state updates / UI dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransition {
    StartWarren,
    BackToMenu,
}

/// Global resource counters (the "battery" side of the food grid).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economy {
    /// Cooked food stockpile — the electricity of the warren.
    pub food: f32,
    /// Raw ingredients aggregated from farms and cook pots for the food
    /// variety readout. Mushroom stock remains physically stored per node.
    #[serde(default)]
    pub raw_food: f32,
    /// Cooked-food alias for new saves; `food` remains the compatible battery
    /// field used by the original simulation.
    #[serde(default)]
    pub cooked_food: f32,
    #[serde(default)]
    pub waste: f32,
    #[serde(default)]
    pub waste_processed: f32,
    /// Ore sitting at the stockpile, spendable on construction/upgrades.
    pub ore_stock: u32,
    /// Lifetime ore delivered (win condition counter; never spent).
    pub ore_delivered_total: u32,
    /// Lifetime ingots forged by the Blacksmith and Smelter Den (the
    /// extended-goal counter; never spent). `metal` alias migrates
    /// pre-Phase-7 saves.
    #[serde(alias = "metal")]
    pub ingots_forged: u32,
    /// Ingots banked at the stockpile, spendable on equipment (Phase 8).
    #[serde(default)]
    pub ingots_stock: u32,
    /// Crafted equipment waiting at the stockpile for a matching creature
    /// to pick up (item id → count).
    #[serde(default)]
    pub gear_stock: HashMap<String, u32>,
    /// Creatures lost to starvation desertion (blackout consequence).
    pub deserted: u32,
    /// Workers killed defending the warren.
    pub killed: u32,
    /// Smoothed food production rate (per minute) for the calorie meter —
    /// cooking lands in bursts, so the HUD shows a moving average.
    pub production_ema_per_min: f32,
    /// Smoothed ore-to-stockpile rate (per minute) — the factory dashboard's
    /// extraction throughput.
    #[serde(default)]
    pub ore_ema_per_min: f32,
    /// Smoothed ingots-forged rate (per minute) — processing throughput.
    #[serde(default)]
    pub ingot_ema_per_min: f32,
}

/// The live simulation: world map plus everything that ticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub world: WorldMap,
    pub rng: SeededRng,
    /// Completed fixed-timestep simulation ticks.
    pub tick: u64,
    pub buildings: Vec<Building>,
    pub build_sites: Vec<BuildSite>,
    /// Rock tiles the player marked for digging.
    pub dig_marks: HashSet<TilePos>,
    pub economy: Economy,
    pub creatures: Vec<Creature>,
    pub next_creature_id: u32,
    /// Wild mushroom patches: seconds until regrown (0 = harvestable).
    #[serde(with = "serde_helpers::tile_key_map")]
    pub patch_regrow: HashMap<TilePos, f32>,
    /// Sporewood groves: seconds until regrown (0 = harvestable).
    #[serde(with = "serde_helpers::tile_key_map")]
    pub sporewood_regrow: HashMap<TilePos, f32>,
    /// Ore remaining per vein tile; mined-out veins open into floor.
    #[serde(with = "serde_helpers::tile_key_map")]
    pub vein_ore: HashMap<TilePos, u32>,
    pub won: bool,
    pub victory_shown: bool,
    /// Extended goal: the forges made `win2_ingots` ingots.
    pub factory_complete: bool,
    pub factory_shown: bool,
    /// Fauna the player didn't hire (wild beetles, raiders).
    pub wilds: Vec<wildlife::WildCreature>,
    pub next_wild_id: u32,
    /// Progression counters + granted unlock ids (`unlocks.json`).
    pub progress: wildlife::Progress,
    pub unlocked: HashSet<String>,
    /// Wildlife/raid/breeding timers (seconds remaining).
    pub wild_spawn_in: f32,
    pub raid_in: f32,
    pub raids_launched: u32,
    pub raid_active: bool,
    pub breed_in: f32,
    /// Blackout episode tracker for the famines_survived counter.
    pub famine_active: bool,
    /// Food offered at the Worm Shrine so far.
    pub worm_fed: f32,
    #[serde(default)]
    pub worm_ingots_fed: u32,
    #[serde(default)]
    pub worm_feeding_paused: bool,
    /// The campaign monument: the Colossal Worm has awakened.
    pub worm_awake: bool,
    pub worm_shown: bool,
    /// Next tutorial step index (== tutorial length when finished).
    pub tutorial_step: usize,
    /// The player skipped the tutorial outright.
    pub tutorial_dismissed: bool,
    /// Action flags the tutorial watches for.
    pub tutorial_reassigned: bool,
    pub tutorial_built: bool,
    #[serde(default)]
    pub outposts: Vec<Outpost>,
    #[serde(default)]
    pub worm_transit: Option<WormTransit>,
    #[serde(default)]
    pub last_transit_failure: Option<String>,
}

impl GameSession {
    pub fn new(data: &GameData, seed: u64) -> Self {
        let config = &data.config;
        let balance = &data.balance;
        let mut rng = SeededRng::new(seed);
        let world = WorldMap::generate(config.world_width, config.world_height, &mut rng);

        let buildings = starting_buildings(&world, balance);
        let patch_regrow = world
            .tiles
            .iter_with_pos()
            .filter(|(_, t)| **t == Tile::MushroomPatch)
            .map(|(pos, _)| (pos, 0.0))
            .collect();
        let sporewood_regrow = world
            .tiles
            .iter_with_pos()
            .filter(|(_, t)| **t == Tile::Sporewood)
            .map(|(pos, _)| (pos, 0.0))
            .collect();
        let vein_ore = world
            .tiles
            .iter_with_pos()
            .filter(|(_, t)| **t == Tile::OreVein)
            .map(|(pos, _)| (pos, balance.vein_ore_yield))
            .collect();

        let mut session = Self {
            world,
            rng,
            tick: 0,
            buildings,
            build_sites: Vec::new(),
            dig_marks: HashSet::new(),
            economy: Economy {
                food: balance.start_food,
                raw_food: 0.0,
                cooked_food: balance.start_food,
                waste: 0.0,
                waste_processed: 0.0,
                ore_stock: 0,
                ore_delivered_total: 0,
                ingots_forged: 0,
                ingots_stock: 0,
                gear_stock: HashMap::new(),
                deserted: 0,
                killed: 0,
                production_ema_per_min: 0.0,
                ore_ema_per_min: 0.0,
                ingot_ema_per_min: 0.0,
            },
            creatures: Vec::new(),
            next_creature_id: 1,
            patch_regrow,
            sporewood_regrow,
            vein_ore,
            won: false,
            victory_shown: false,
            factory_complete: false,
            factory_shown: false,
            wilds: Vec::new(),
            next_wild_id: 1,
            progress: wildlife::Progress::default(),
            unlocked: HashSet::new(),
            wild_spawn_in: balance.wild_beetle_spawn_sec,
            raid_in: balance.raid_first_sec,
            raids_launched: 0,
            raid_active: false,
            breed_in: balance.breed_interval_sec,
            famine_active: false,
            worm_fed: 0.0,
            worm_ingots_fed: 0,
            worm_feeding_paused: false,
            worm_awake: false,
            worm_shown: false,
            tutorial_step: 0,
            tutorial_dismissed: false,
            tutorial_reassigned: false,
            tutorial_built: false,
            outposts: Vec::new(),
            worm_transit: None,
            last_transit_failure: None,
        };

        for _ in 0..balance.start_miners {
            session.spawn_creature(data, "goblin", Job::Miner);
        }
        for _ in 0..balance.start_carriers {
            session.spawn_creature(data, "goblin", Job::Carrier);
        }
        for _ in 0..balance.start_cooks {
            session.spawn_creature(data, "goblin", Job::Cook);
        }

        session
    }

    pub fn spawn_creature(&mut self, data: &GameData, species: &str, job: Job) {
        let id = self.next_creature_id;
        self.next_creature_id += 1;
        let mut creature = Creature::new(id, species, job, self.spawn_tile());
        creature.hp = data.species.get(species).map(|s| s.max_hp).unwrap_or(10.0);
        self.creatures.push(creature);
    }

    pub fn spawn_tile(&self) -> TilePos {
        self.world.spawn
    }

    /// The (single) stockpile position — ore deliveries land here.
    pub fn stockpile_pos(&self) -> TilePos {
        self.buildings
            .iter()
            .find(|b| b.kind == "stockpile")
            .map(|b| b.pos)
            .unwrap_or(self.world.spawn)
    }

    pub fn buildings_of<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Building> + 'a {
        self.buildings.iter().filter(move |b| b.kind == kind)
    }

    pub fn building_at(&self, pos: TilePos) -> Option<&Building> {
        self.buildings.iter().find(|b| b.pos == pos)
    }

    pub fn building_at_mut(&mut self, pos: TilePos) -> Option<&mut Building> {
        self.buildings.iter_mut().find(|b| b.pos == pos)
    }

    pub fn site_at(&self, pos: TilePos) -> Option<&BuildSite> {
        self.build_sites.iter().find(|s| s.pos == pos)
    }

    /// Whether a ghost can go here: open walkable floor, nothing else on it.
    pub fn can_place_building(&self, pos: TilePos) -> bool {
        self.world.tiles.get(pos).is_some_and(|t| *t == Tile::Floor)
            && self.building_at(pos).is_none()
            && self.site_at(pos).is_none()
    }

    /// Placement rules for a specific building kind. The Mine additionally
    /// demands an adjacent ore vein to exploit (plan §3).
    pub fn can_place_kind(&self, kind: &str, pos: TilePos) -> bool {
        if !self.can_place_building(pos) {
            return false;
        }
        if kind == "mine" {
            return self.adjacent_ore_vein(pos).is_some();
        }
        if kind == "outpost" {
            return self.buildings_of("worm_shrine").next().is_some()
                && self.outposts.iter().all(|o| o.pos != pos);
        }
        true
    }

    /// Number of usable tiles, plus the rooms remote outposts provide.
    pub fn usable_warren_capacity(&self, data: &GameData) -> usize {
        let floor_tiles = self
            .world
            .tiles
            .iter_with_pos()
            .filter(|(_, t)| t.walkable())
            .count() as f32;
        let local = (floor_tiles / data.balance.capacity_tiles_per_creature).floor() as usize;
        local
            + self.outposts.iter().filter(|o| o.active).count()
                * data.balance.outpost_capacity as usize
    }

    pub fn overcrowding_ratio(&self, data: &GameData) -> f32 {
        let capacity = self.usable_warren_capacity(data).max(1) as f32;
        self.creatures.len() as f32 / capacity
    }

    pub fn ensure_outpost(&mut self, pos: TilePos) {
        if self.outposts.iter().all(|o| o.pos != pos) {
            self.outposts.push(Outpost::new(pos));
        }
    }

    /// A 4-neighbour ore-vein tile of `pos`, if any (Mine placement).
    pub fn adjacent_ore_vein(&self, pos: TilePos) -> Option<TilePos> {
        pos.neighbors_4way()
            .into_iter()
            .find(|n| self.world.tiles.get(*n) == Some(&Tile::OreVein))
    }

    /// Toggle a dig designation on rock (plain or ore vein).
    pub fn toggle_dig_mark(&mut self, pos: TilePos) -> bool {
        let diggable = self
            .world
            .tiles
            .get(pos)
            .is_some_and(|t| matches!(t, Tile::Rock | Tile::OreVein));
        if !diggable {
            return false;
        }
        if !self.dig_marks.remove(&pos) {
            self.dig_marks.insert(pos);
        }
        true
    }

    pub fn job_count(&self, job: Job) -> usize {
        self.creatures.iter().filter(|c| c.job == job).count()
    }

    /// Move one reassignable creature from `from` to `to`. Returns success.
    pub fn reassign(
        &mut self,
        from: Job,
        to: Job,
        species_reassignable: impl Fn(&str) -> bool,
    ) -> bool {
        if from == to {
            return false;
        }
        let Some(creature) = self
            .creatures
            .iter_mut()
            .find(|c| c.job == from && species_reassignable(&c.species))
        else {
            return false;
        };
        creature.job = to;
        creature.clear_task();
        self.tutorial_reassigned = true;
        true
    }

    /// Per-minute upkeep draw for one creature (idle draws reduced rate,
    /// cooks and guards draw more — the plan's tier-0 table).
    pub fn upkeep_per_min(creature: &Creature, base: f32, balance: &Balance) -> f32 {
        match creature.job {
            Job::Idle => base * balance.idle_upkeep_factor,
            Job::Cook | Job::Smith => base * balance.cook_upkeep_factor,
            Job::Guard => base * balance.guard_upkeep_factor,
            _ => base,
        }
    }

    /// Whether this building kind may currently be placed (unlock gates).
    pub fn building_unlocked(&self, def: &crate::data::BuildingDef) -> bool {
        def.requires_unlock
            .as_ref()
            .map(|id| self.unlocked.contains(id))
            .unwrap_or(true)
    }
}

/// Pre-place the starting stockpile, cook pot, farm, and a working Mine on
/// reachable open floor. The pot sits next to spawn; the farm sits a real
/// haul away (that walk is what makes carrier throughput a meaningful
/// number); the Mine sits on the nearest vein-adjacent floor so extraction
/// → haul → stockpile is alive at second zero (plan §3).
fn starting_buildings(world: &WorldMap, balance: &Balance) -> Vec<Building> {
    let spawn = world.spawn;
    let reachable = world
        .tiles
        .flood_fill(spawn, false, |_, t: &Tile| t.walkable());
    let mut floors_by_distance: Vec<TilePos> = world
        .tiles
        .iter_with_pos()
        .filter(|(pos, t)| **t == Tile::Floor && *pos != spawn && reachable.contains(pos))
        .map(|(pos, _)| pos)
        .collect();
    floors_by_distance.sort_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));

    let cook_pot = floors_by_distance.first().copied().unwrap_or(spawn);
    let farm = floors_by_distance
        .iter()
        .find(|p| p.manhattan_distance(&spawn) >= balance.farm_min_distance && **p != cook_pot)
        .copied()
        // Fall back to the farthest reachable floor on cramped maps.
        .or_else(|| floors_by_distance.last().copied())
        .unwrap_or(spawn);

    let mut buildings = vec![
        Building::new("stockpile", spawn),
        Building::new("cook_pot", cook_pot),
        Building::new("farm", farm),
    ];

    // The prebuilt Mine: nearest reachable floor beside an ore vein, so the
    // extraction loop is already ticking beside the warren.
    if let Some(mine) = floors_by_distance.iter().copied().find(|p| {
        *p != cook_pot
            && *p != farm
            && p.neighbors_4way()
                .iter()
                .any(|n| world.tiles.get(*n) == Some(&Tile::OreVein))
    }) {
        buildings.push(Building::mine(mine, balance.mine_reserve));
    }

    buildings
}

#[cfg(test)]
mod tests;
