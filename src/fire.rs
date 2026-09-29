#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_update::{BlockPos, Direction};

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
    Extinguish,
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

/// `FireBlock.getIgniteOdds/getBurnOdds` for a state: the exact
/// `FireBlock.bootStrap` registry (see [`crate::fire_block`]); waterlogged
/// states have zero odds like Java.
pub fn flammability(state: &BlockStateModel) -> Option<Flammability> {
    if state.property("waterlogged") == Some("true") {
        return None;
    }
    crate::fire_block::flammable_block_odds(&state.registry_id).map(|(ignite_odds, burn_odds)| {
        Flammability {
            ignite_odds,
            burn_odds,
        }
    })
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
    // Java `Block.getExplosionResistance()` — block-level constant, exact for
    // every vanilla block; 0.5 only for names outside the vanilla registry.
    crate::block_properties::block_physics(&state.registry_id)
        .map(|physics| physics.explosion_resistance)
        .unwrap_or(0.5)
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

#[cfg(test)]
mod tests {
    use super::{
        blast_resistance, explosion_affects_block, explosion_entity_impact,
        explosion_neighbor_updates, extinguish_by_neighbor, flammability, prime_tnt,
        ExplosionInput, FireAction, TNT_FUSE_TICKS,
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
