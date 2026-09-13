//! Integrity checks for sessions crossing the save/load boundary.

use super::*;

pub fn validate_loaded_session(session: &GameSession, data: &GameData) -> Result<(), String> {
    let tiles = &session.world.tiles;
    let expected_cells = tiles
        .width
        .checked_mul(tiles.height)
        .ok_or_else(|| "world dimensions overflowed".to_owned())?;
    if tiles.width == 0 || tiles.height == 0 || tiles.data().len() != expected_cells {
        return Err("world grid dimensions do not match its stored tiles".to_owned());
    }
    if !session.world.spawn.in_bounds(tiles.width, tiles.height) {
        return Err("world spawn is outside the stored map".to_owned());
    }
    validate_outpost_milestones(session, data)?;

    let mut occupied = HashSet::new();
    for building in &session.buildings {
        if data.buildings.get(&building.kind).is_none() {
            return Err(format!("unknown building id {:?}", building.kind));
        }
        validate_walkable_position(
            session,
            building.pos,
            &format!("building {:?}", building.kind),
        )?;
        if !occupied.insert(building.pos) {
            return Err(format!("multiple buildings occupy {:?}", building.pos));
        }
        validate_nonnegative_finite(building.reserve, "building reserve")?;
        validate_nonnegative_finite(building.waste, "building waste")?;
        for (good, amount) in &building.stocks {
            validate_nonnegative_finite(*amount, &format!("building stock {good:?}"))?;
        }
        for order in &building.orders {
            let Some(equipment) = data.equipment_def(order) else {
                return Err(format!("unknown equipment order id {order:?}"));
            };
            if !session.equipment_unlocked(equipment) {
                return Err(format!("locked equipment order id {order:?}"));
            }
        }
    }
    for equipment in session.economy.gear_stock.keys() {
        let Some(definition) = data.equipment_def(equipment) else {
            return Err(format!("unknown stored equipment id {equipment:?}"));
        };
        if !session.equipment_unlocked(definition) {
            return Err(format!("locked stored equipment id {equipment:?}"));
        }
    }

    for site in &session.build_sites {
        if data.buildings.get(&site.kind).is_none() {
            return Err(format!("unknown construction id {:?}", site.kind));
        }
        validate_walkable_position(session, site.pos, "construction site")?;
        if !occupied.insert(site.pos) {
            return Err(format!(
                "construction overlaps an occupied tile {:?}",
                site.pos
            ));
        }
        if site.ore_needed == 0 || site.ore_delivered > site.ore_needed {
            return Err(format!("invalid construction progress at {:?}", site.pos));
        }
    }

    for pos in &session.dig_marks {
        if !pos.in_bounds(tiles.width, tiles.height)
            || !session.world.tiles.get(*pos).is_some_and(|tile| {
                matches!(
                    tile,
                    crate::state::world::Tile::Rock | crate::state::world::Tile::OreVein
                )
            })
        {
            return Err(format!("invalid dig designation at {pos:?}"));
        }
    }
    for (pos, remaining) in &session.patch_regrow {
        validate_map_timer(
            *pos,
            *remaining,
            tiles.width,
            tiles.height,
            "mushroom regrow",
        )?;
    }
    for (pos, remaining) in &session.sporewood_regrow {
        validate_map_timer(
            *pos,
            *remaining,
            tiles.width,
            tiles.height,
            "sporewood regrow",
        )?;
    }
    for pos in session.vein_ore.keys() {
        if !pos.in_bounds(tiles.width, tiles.height) {
            return Err(format!("ore vein state is outside the map at {pos:?}"));
        }
    }

    validate_unique_ids(
        session.creatures.iter().map(|creature| creature.id),
        session.next_creature_id,
        "creature",
    )?;
    for creature in &session.creatures {
        if data.species.get(&creature.species).is_none() {
            return Err(format!("unknown creature species {:?}", creature.species));
        }
        validate_actor_position(session, creature.x, creature.y, "creature")?;
        validate_nonnegative_finite(creature.starving_for, "creature starvation timer")?;
        validate_nonnegative_finite(creature.hp, "creature health")?;
        validate_nonnegative_finite(creature.morale_stress_for, "creature morale timer")?;
        if !creature.satiation.is_finite() || !creature.morale.is_finite() {
            return Err("creature wellbeing contains a non-finite value".to_owned());
        }
        if let Some(equipment) = &creature.equipment {
            let Some(definition) = data.equipment_def(equipment) else {
                return Err(format!("unknown equipped item id {equipment:?}"));
            };
            if !session.equipment_unlocked(definition) {
                return Err(format!("locked equipped item id {equipment:?}"));
            }
        }
        validate_task_positions(session, &creature.task, &creature.path)?;
        if let Some((_, amount)) = creature.carrying {
            if amount == 0 {
                return Err(format!("creature {} carries an empty load", creature.id));
            }
        }
    }

    validate_unique_ids(
        session.wilds.iter().map(|wild| wild.id),
        session.next_wild_id,
        "wild creature",
    )?;
    for wild in &session.wilds {
        if data.species.get(&wild.species).is_none() {
            return Err(format!("unknown wild species {:?}", wild.species));
        }
        validate_actor_position(session, wild.x, wild.y, "wild creature")?;
        validate_nonnegative_finite(wild.hp, "wild creature health")?;
        validate_wild_behavior(session, &wild.behavior)?;
        for path_pos in &wild.path {
            validate_map_position(*path_pos, tiles.width, tiles.height, "wild creature path")?;
        }
    }

    let creature_ids: HashSet<u32> = session
        .creatures
        .iter()
        .map(|creature| creature.id)
        .collect();
    let mut claimed_outpost_crew = HashSet::new();
    let mut outpost_crew_counts = Vec::with_capacity(session.outposts.len());
    for outpost in &session.outposts {
        validate_walkable_position(session, outpost.pos, "outpost")?;
        if session
            .outposts
            .iter()
            .filter(|other| other.pos == outpost.pos)
            .count()
            > 1
        {
            return Err(format!("multiple outpost records occupy {:?}", outpost.pos));
        }
        if !session
            .buildings
            .iter()
            .any(|building| building.pos == outpost.pos && building.kind == "outpost")
        {
            return Err(format!(
                "outpost record has no matching building at {:?}",
                outpost.pos
            ));
        }
        if outpost.survey_upgraded && (!outpost.storage_upgraded || !outpost.crew_upgraded) {
            return Err(format!(
                "outpost survey rig lacks its expanded hold and camp at {:?}",
                outpost.pos
            ));
        }
        if outpost.resonator_upgraded && !outpost.survey_upgraded {
            return Err(format!(
                "outpost resonance beacon lacks its survey rig at {:?}",
                outpost.pos
            ));
        }
        if outpost.deep_survey_upgraded && !outpost.resonator_upgraded {
            return Err(format!(
                "outpost deep survey lacks its resonance beacon at {:?}",
                outpost.pos
            ));
        }
        if outpost.deep_survey_upgraded && !session.outpost_charter_claimed {
            return Err(format!(
                "outpost deep survey lacks the Worm Road Charter at {:?}",
                outpost.pos
            ));
        }
        if outpost.signal_cache_upgraded && !outpost.deep_survey_upgraded {
            return Err(format!(
                "outpost Signal Cache lacks deep survey at {:?}",
                outpost.pos
            ));
        }
        if outpost.signal_cache_upgraded && !session.outpost_relay_claimed {
            return Err(format!(
                "outpost Signal Cache lacks the Worm Road Relay at {:?}",
                outpost.pos
            ));
        }
        if outpost.waypoint_upgraded && !outpost.signal_cache_upgraded {
            return Err(format!(
                "outpost Worm Road Waypoint lacks its Signal Cache at {:?}",
                outpost.pos
            ));
        }
        if outpost.waypoint_upgraded && session.outpost_convoy_claims == 0 {
            return Err(format!(
                "outpost Worm Road Waypoint lacks a cleared Worm Road Convoy at {:?}",
                outpost.pos
            ));
        }
        validate_outpost_cargo(session, outpost, data)?;
        validate_nonnegative_finite(outpost.expedition_progress, "outpost expedition progress")?;

        let crew_count = outpost
            .crew
            .iter()
            .filter(|id| creature_ids.contains(id) && claimed_outpost_crew.insert(**id))
            .count();
        let capacity = crate::simulation::outposts::crew_capacity(outpost, data) as usize;
        if crew_count > capacity {
            return Err(format!(
                "outpost crew exceeds its {}-creature capacity at {:?}",
                capacity, outpost.pos
            ));
        }
        outpost_crew_counts.push((outpost.pos, crew_count));
    }
    if let Some(transit) = &session.worm_transit {
        if !session.worm_awake {
            return Err("worm transit exists before the worm awakens".to_owned());
        }
        let Some(outpost) = session
            .outposts
            .iter()
            .find(|outpost| outpost.pos == transit.outpost)
        else {
            return Err(format!(
                "transit targets an unknown outpost {:?}",
                transit.outpost
            ));
        };
        let capacity = crate::simulation::outposts::route_storage_capacity(session, data, outpost);
        let cargo_without_food = transit
            .ore
            .checked_add(transit.ingots)
            .ok_or_else(|| "worm transit cargo total overflows".to_owned())?;
        if cargo_without_food > capacity || transit.food > (capacity - cargo_without_food) as f32 {
            return Err(format!(
                "worm transit cargo exceeds the {:?} outpost hold",
                transit.outpost
            ));
        }
        if transit.direction == TransitDirection::ToOutpost {
            let stored_cargo = outpost
                .cargo
                .values()
                .try_fold(0u32, |total, amount| total.checked_add(*amount))
                .ok_or_else(|| format!("outpost cargo total overflows at {:?}", outpost.pos))?;
            let combined_cargo = stored_cargo
                .checked_add(cargo_without_food)
                .ok_or_else(|| {
                    format!(
                        "worm transit cargo total overflows at {:?}",
                        transit.outpost
                    )
                })?;
            if combined_cargo > capacity || transit.food > (capacity - combined_cargo) as f32 {
                return Err(format!(
                    "worm transit cargo exceeds the {:?} outpost hold when combined with stored cargo",
                    transit.outpost
                ));
            }

            let arriving_crew = transit
                .passengers
                .iter()
                .filter(|id| {
                    let id = **id;
                    creature_ids.contains(&id)
                        && !outpost.crew.contains(&id)
                        && !claimed_outpost_crew.contains(&id)
                })
                .copied()
                .collect::<HashSet<_>>()
                .len();
            let stationed_crew = outpost_crew_counts
                .iter()
                .find(|(pos, _)| *pos == outpost.pos)
                .map(|(_, count)| *count)
                .unwrap_or(0);
            let crew_capacity = crate::simulation::outposts::crew_capacity(outpost, data);
            if stationed_crew + arriving_crew > crew_capacity as usize {
                return Err(format!(
                    "worm transit crew exceeds the {:?} outpost capacity",
                    transit.outpost
                ));
            }
        }
        validate_nonnegative_finite(transit.remaining, "worm transit timer")?;
        validate_nonnegative_finite(transit.food, "worm transit food")?;
    }

    for (name, value) in [
        ("food", session.economy.food),
        ("raw food", session.economy.raw_food),
        ("cooked food", session.economy.cooked_food),
        ("waste", session.economy.waste),
        ("processed waste", session.economy.waste_processed),
        (
            "food production rate",
            session.economy.production_ema_per_min,
        ),
        ("ore production rate", session.economy.ore_ema_per_min),
        ("ingot production rate", session.economy.ingot_ema_per_min),
        ("worm fed food", session.worm_fed),
    ] {
        validate_nonnegative_finite(value, name)?;
    }
    for (name, value) in [
        ("wild spawn timer", session.wild_spawn_in),
        ("raid timer", session.raid_in),
        ("breeding timer", session.breed_in),
        ("progress knowledge", session.progress.knowledge),
        ("progress waste generated", session.progress.waste_generated),
    ] {
        validate_nonnegative_finite(value, name)?;
    }
    Ok(())
}

pub fn validate_loaded_session_boundary(
    session: GameSession,
    data: &GameData,
) -> Result<GameSession, String> {
    validate_loaded_session(&session, data)?;
    Ok(session)
}
