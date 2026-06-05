#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_metadata::representative_state_definition;
use crate::block_update::{BlockPos, Direction};

pub const FIRE_TICK_DELAY: i32 = 30;
pub const TNT_FUSE_TICKS: i32 = 80;
pub const EXPLOSION_RAY_ATTENUATION: f32 = 0.225;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flammability {
    pub ignite_odds: u8,
    pub burn_odds: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FireAction {
    Stay,
    Schedule { delay: i32 },
    Extinguish,
    Ignite(BlockStateModel),
    Burn(BlockStateModel),
    PrimeTnt { fuse_ticks: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionInput {
    pub pos: BlockPos,
    pub block: BlockStateModel,
    pub distance: f32,
    pub exposure: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionBlockResult {
    pub pos: BlockPos,
    pub destroyed: bool,
    pub remaining_power: f32,
    pub drops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionEntityImpact {
    pub damage: f32,
    pub knockback: f32,
}

pub fn flammability(state: &BlockStateModel) -> Option<Flammability> {
    let id = state.registry_id.as_str();
    if id == "minecraft:tnt" {
        Some(Flammability {
            ignite_odds: 15,
            burn_odds: 100,
        })
    } else if id.ends_with("_leaves") || id.ends_with("_wool") {
        Some(Flammability {
            ignite_odds: 30,
            burn_odds: 60,
        })
    } else if id.contains("planks")
        || id.ends_with("_slab")
        || id.ends_with("_stairs")
        || id.ends_with("_fence")
        || id.ends_with("_fence_gate")
        || id.ends_with("_roots")
    {
        Some(Flammability {
            ignite_odds: 5,
            burn_odds: 20,
        })
    } else if id.ends_with("_log")
        || id.ends_with("_wood")
        || id.contains("bamboo_block")
        || id == "minecraft:coal_block"
    {
        Some(Flammability {
            ignite_odds: 5,
            burn_odds: 5,
        })
    } else if id == "minecraft:vine" || id == "minecraft:target" {
        Some(Flammability {
            ignite_odds: 15,
            burn_odds: if id == "minecraft:vine" { 100 } else { 20 },
        })
    } else if id.ends_with("_carpet")
        || id.ends_with("_grass")
        || id.ends_with("_flower")
        || id == "minecraft:hay_block"
        || id == "minecraft:bookshelf"
    {
        Some(Flammability {
            ignite_odds: 60,
            burn_odds: if id == "minecraft:hay_block" { 20 } else { 100 },
        })
    } else {
        None
    }
}

pub fn fire_tick(
    fire_state: &BlockStateModel,
    raining_at_pos: bool,
    has_support_or_flammable_neighbor: bool,
) -> FireAction {
    if raining_at_pos {
        return FireAction::Extinguish;
    }
    if !has_support_or_flammable_neighbor {
        return FireAction::Extinguish;
    }

    let age = fire_age(fire_state);
    FireAction::Schedule {
        delay: FIRE_TICK_DELAY + i32::from(age.min(15)),
    }
}

pub fn try_spread_fire(
    target: &BlockStateModel,
    fire_age: u8,
    difficulty_id: u8,
    random_roll: u8,
) -> FireAction {
    let Some(flammable) = flammability(target) else {
        return FireAction::Stay;
    };
    if target.registry_id == "minecraft:tnt" {
        return FireAction::PrimeTnt {
            fuse_ticks: TNT_FUSE_TICKS,
        };
    }

    let odds = ((u16::from(flammable.ignite_odds) + 40 + u16::from(difficulty_id) * 7)
        / u16::from(fire_age.saturating_add(30)))
    .min(u16::from(u8::MAX)) as u8;
    if random_roll < odds {
        FireAction::Ignite(BlockStateModel::new("minecraft:fire").with_property("age", "0"))
    } else if random_roll < flammable.burn_odds {
        FireAction::Burn(BlockStateModel::air())
    } else {
        FireAction::Stay
    }
}

pub fn extinguish_by_neighbor(state: &BlockStateModel, neighbor: &BlockStateModel) -> FireAction {
    if state.registry_id == "minecraft:fire"
        && matches!(
            neighbor.registry_id.as_str(),
            "minecraft:water" | "minecraft:powder_snow"
        )
    {
        FireAction::Extinguish
    } else {
        FireAction::Stay
    }
}

pub fn prime_tnt(source: &str) -> FireAction {
    match source {
        "fire" | "redstone" | "explosion" | "flint_and_steel" | "fire_charge" => {
            FireAction::PrimeTnt {
                fuse_ticks: TNT_FUSE_TICKS,
            }
        }
        _ => FireAction::Stay,
    }
}

pub fn blast_resistance(state: &BlockStateModel) -> f32 {
    representative_state_definition(&state.registry_id)
        .map(|definition| definition.physical.explosion_resistance)
        .unwrap_or_else(|| match state.registry_id.as_str() {
            "minecraft:obsidian" => 1200.0,
            "minecraft:bedrock" | "minecraft:end_portal_frame" => 3_600_000.0,
            "minecraft:tnt" => 0.0,
            "minecraft:air" => 0.0,
            _ => 0.5,
        })
}

/// Legacy single-block explosion check. The production explosion path now uses
/// the faithful `game_event::calculate_exploded_positions` ray-march (per-ray
/// RNG intensity + cumulative resistance); `plan_server_explosion` consumes its
/// destroyed-position set. This helper (no random intensity, only the current
/// block's resistance) is retained for the unit tests that exercise
/// `blast_resistance`.
pub fn explosion_affects_block(input: ExplosionInput, initial_power: f32) -> ExplosionBlockResult {
    let resistance = blast_resistance(&input.block);
    let remaining_power =
        initial_power - input.distance * EXPLOSION_RAY_ATTENUATION - (resistance + 0.3) * 0.3;
    let destroyed = !input.block.is_air() && remaining_power > 0.0;
    ExplosionBlockResult {
        pos: input.pos,
        destroyed,
        remaining_power,
        drops: if destroyed {
            vec![input.block.registry_id]
        } else {
            Vec::new()
        },
    }
}

pub fn explosion_entity_impact(
    power: f32,
    distance_ratio: f32,
    exposure: f32,
    protection: f32,
) -> ExplosionEntityImpact {
    let impact = (1.0 - distance_ratio.clamp(0.0, 1.0)) * exposure.clamp(0.0, 1.0);
    let raw_damage = ((impact * impact + impact) / 2.0 * 7.0 * power * 2.0 + 1.0).max(0.0);
    let damage = (raw_damage * (1.0 - protection.clamp(0.0, 1.0))).max(0.0);
    ExplosionEntityImpact {
        damage,
        knockback: impact,
    }
}

pub fn explosion_neighbor_updates(pos: BlockPos) -> [BlockPos; 6] {
    [
        pos.relative(Direction::West),
        pos.relative(Direction::East),
        pos.relative(Direction::Down),
        pos.relative(Direction::Up),
        pos.relative(Direction::North),
        pos.relative(Direction::South),
    ]
}

fn fire_age(state: &BlockStateModel) -> u8 {
    state
        .property("age")
        .and_then(|age| age.parse::<u8>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        blast_resistance, explosion_affects_block, explosion_entity_impact,
        explosion_neighbor_updates, extinguish_by_neighbor, fire_tick, flammability, prime_tnt,
        try_spread_fire, ExplosionInput, FireAction, FIRE_TICK_DELAY, TNT_FUSE_TICKS,
    };
    use crate::block_behavior::BlockStateModel;
    use crate::block_update::{BlockPos, Direction};

    #[test]
    fn flammability_matches_representative_vanilla_tables() {
        assert_eq!(
            flammability(&BlockStateModel::new("minecraft:oak_planks"))
                .unwrap()
                .burn_odds,
            20
        );
        assert_eq!(
            flammability(&BlockStateModel::new("minecraft:oak_log"))
                .unwrap()
                .burn_odds,
            5
        );
        assert_eq!(
            flammability(&BlockStateModel::new("minecraft:oak_leaves"))
                .unwrap()
                .ignite_odds,
            30
        );
        assert_eq!(
            flammability(&BlockStateModel::new("minecraft:tnt"))
                .unwrap()
                .burn_odds,
            100
        );
        assert!(flammability(&BlockStateModel::new("minecraft:stone")).is_none());
    }

    #[test]
    fn fire_tick_extinguishes_without_support_or_in_rain_otherwise_reschedules() {
        let fire = BlockStateModel::new("minecraft:fire").with_property("age", "4");
        assert_eq!(fire_tick(&fire, true, true), FireAction::Extinguish);
        assert_eq!(fire_tick(&fire, false, false), FireAction::Extinguish);
        assert_eq!(
            fire_tick(&fire, false, true),
            FireAction::Schedule {
                delay: FIRE_TICK_DELAY + 4
            }
        );
    }

    #[test]
    fn fire_spread_ignites_burns_or_primes_tnt() {
        assert_eq!(
            try_spread_fire(&BlockStateModel::new("minecraft:oak_planks"), 0, 2, 0),
            FireAction::Ignite(BlockStateModel::new("minecraft:fire").with_property("age", "0"))
        );
        assert_eq!(
            try_spread_fire(&BlockStateModel::new("minecraft:oak_leaves"), 15, 0, 50),
            FireAction::Burn(BlockStateModel::air())
        );
        assert_eq!(
            try_spread_fire(&BlockStateModel::new("minecraft:tnt"), 0, 0, 99),
            FireAction::PrimeTnt {
                fuse_ticks: TNT_FUSE_TICKS
            }
        );
        assert_eq!(
            try_spread_fire(&BlockStateModel::new("minecraft:stone"), 0, 0, 0),
            FireAction::Stay
        );
    }

    #[test]
    fn water_and_powder_snow_extinguish_fire() {
        let fire = BlockStateModel::new("minecraft:fire");
        assert_eq!(
            extinguish_by_neighbor(&fire, &BlockStateModel::new("minecraft:water")),
            FireAction::Extinguish
        );
        assert_eq!(
            extinguish_by_neighbor(&fire, &BlockStateModel::new("minecraft:stone")),
            FireAction::Stay
        );
    }

    #[test]
    fn tnt_primes_from_vanilla_sources() {
        assert_eq!(
            prime_tnt("redstone"),
            FireAction::PrimeTnt {
                fuse_ticks: TNT_FUSE_TICKS
            }
        );
        assert_eq!(
            prime_tnt("explosion"),
            FireAction::PrimeTnt {
                fuse_ticks: TNT_FUSE_TICKS
            }
        );
        assert_eq!(prime_tnt("hand"), FireAction::Stay);
    }

    #[test]
    fn blast_resistance_uses_metadata_and_special_hard_blocks() {
        assert!(blast_resistance(&BlockStateModel::new("minecraft:stone")) > 0.0);
        assert_eq!(
            blast_resistance(&BlockStateModel::new("minecraft:obsidian")),
            1200.0
        );
        assert_eq!(
            blast_resistance(&BlockStateModel::new("minecraft:bedrock")),
            3_600_000.0
        );
    }

    #[test]
    fn explosion_destroys_low_resistance_blocks_and_keeps_high_resistance_blocks() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let sand = explosion_affects_block(
            ExplosionInput {
                pos,
                block: BlockStateModel::new("minecraft:sand"),
                distance: 0.0,
                exposure: 1.0,
            },
            4.0,
        );
        assert!(sand.destroyed);
        assert_eq!(sand.drops, vec!["minecraft:sand"]);

        let bedrock = explosion_affects_block(
            ExplosionInput {
                pos,
                block: BlockStateModel::new("minecraft:bedrock"),
                distance: 0.0,
                exposure: 1.0,
            },
            4.0,
        );
        assert!(!bedrock.destroyed);
    }

    #[test]
    fn explosion_entity_damage_scales_by_distance_exposure_and_protection() {
        let full = explosion_entity_impact(4.0, 0.0, 1.0, 0.0);
        let protected = explosion_entity_impact(4.0, 0.0, 1.0, 0.5);
        let far = explosion_entity_impact(4.0, 1.0, 1.0, 0.0);
        assert!(full.damage > protected.damage);
        assert!(protected.damage > far.damage);
        assert_eq!(far.knockback, 0.0);
    }

    #[test]
    fn explosion_neighbor_updates_cover_all_sides() {
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        assert_eq!(
            explosion_neighbor_updates(pos),
            [
                pos.relative(Direction::West),
                pos.relative(Direction::East),
                pos.relative(Direction::Down),
                pos.relative(Direction::Up),
                pos.relative(Direction::North),
                pos.relative(Direction::South),
            ]
        );
    }
}
