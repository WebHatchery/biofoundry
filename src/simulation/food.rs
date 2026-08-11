//! The food grid: calorie ledger math, hunger (brownout), and starvation
//! (blackout → desertion). Food is treated exactly like electricity.

use crate::data::GameData;
use crate::state::creatures::Creature;
use crate::state::GameSession;

/// Total upkeep draw across all creatures, in food per minute.
pub fn consumption_per_min(session: &GameSession, data: &GameData) -> f32 {
    session
        .creatures
        .iter()
        .map(|c| {
            let base = data
                .species
                .get(&c.species)
                .map(|s| s.food_per_min)
                .unwrap_or(0.0);
            GameSession::upkeep_per_min(c, base, &data.balance)
        })
        .sum()
}

/// Drain the stockpile by total upkeep and update every creature's
/// satiation. Returns creatures that deserted this tick (already removed).
///
/// Diets: "food" eaters draw the shared stockpile; "charcoal" eaters
/// (salamanders) refill by consuming charcoal at their den when they
/// smelt, and only hunger slowly between meals.
pub fn tick_hunger(session: &mut GameSession, data: &GameData, dt: f32) -> Vec<Creature> {
    let consumption = consumption_per_min(session, data);
    let fed = session.economy.food > 0.0;
    session.economy.food = (session.economy.food - consumption / 60.0 * dt).max(0.0);

    let b = &data.balance;
    for creature in &mut session.creatures {
        let species = data.species.get(&creature.species);
        let eats_food = species.map(|s| s.diet == "food").unwrap_or(true);
        // Fed creatures knit wounds between fights.
        if creature.satiation > 0.66 {
            let max_hp = species.map(|s| s.max_hp).unwrap_or(20.0);
            creature.hp = (creature.hp + b.hp_regen_per_sec * dt).min(max_hp);
        }
        if eats_food {
            if fed {
                creature.satiation = (creature.satiation + dt / b.satiation_recover_sec).min(1.0);
                creature.starving_for = 0.0;
            } else {
                creature.satiation = (creature.satiation - dt / b.satiation_drain_sec).max(0.0);
                if creature.satiation <= 0.0 {
                    creature.starving_for += dt;
                }
            }
        } else {
            // Charcoal eaters: meals happen at the den (jobs.rs); here
            // they just get slowly hungrier.
            creature.satiation = (creature.satiation - dt / b.salamander_hunger_drain_sec).max(0.0);
            if creature.satiation <= 0.0 {
                creature.starving_for += dt;
            }
        }
    }

    let desert_after = b.desert_after_starving_sec;
    let mut deserters = Vec::new();
    let mut i = 0;
    while i < session.creatures.len() {
        if session.creatures[i].starving_for >= desert_after {
            deserters.push(session.creatures.remove(i));
        } else {
            i += 1;
        }
    }
    session.economy.deserted += deserters.len() as u32;
    deserters
}

#[cfg(test)]
mod tests;
