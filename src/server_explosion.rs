//! Java `net.minecraft.world.level.ServerExplosion` and
//! `ExplosionDamageCalculator`: the world-independent parts of an explosion.
//!
//! Everything here is a pure function of the block world ([`SurvivalWorld`])
//! and a random source, so it is unit-testable; the live server applies the
//! results to the level, its entities and its players
//! (`network::status::explosion_live`).

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{block_physics, state_physics_by_name, StateFluid};
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::collision_shape::Aabb;
use crate::entity_collision::clip_hits_collider;
use crate::entity_physics::ExplosionBlockInteraction;
use crate::game_event::calculate_exploded_positions;
use crate::network::play::Vec3;

/// `ServerExplosion.MAX_DROPS_PER_COMBINED_STACK`.
pub const MAX_DROPS_PER_COMBINED_STACK: i32 = 16;
/// `FlowingFluid.getExplosionResistance` of water and lava.
const FLUID_EXPLOSION_RESISTANCE: f32 = 100.0;

/// The `RandomSource` calls an explosion makes (`ServerLevel.random`).
pub trait ExplosionRandom {
    /// `RandomSource.nextFloat()`.
    fn next_float(&mut self) -> f32;
    /// `RandomSource.nextInt(bound)`.
    fn next_int(&mut self, bound: i32) -> i32;
    /// `RandomSource.nextDouble()`.
    fn next_double(&mut self) -> f64;
}

/// `Level.ExplosionInteraction`. Only `Tnt` has a live source so far; the
/// other kinds belong to creepers, beds/respawn anchors, wind charges and
/// `Level.explode` callers that are not ported yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LevelExplosionInteraction {
    None,
    Block,
    Mob,
    Tnt,
    Trigger,
}

/// The game rules `ServerLevel.explode` consults to pick the block interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExplosionRules {
    /// `GameRules.MOB_GRIEFING`.
    pub mob_griefing: bool,
    /// `GameRules.BLOCK_EXPLOSION_DROP_DECAY`.
    pub block_explosion_drop_decay: bool,
    /// `GameRules.MOB_EXPLOSION_DROP_DECAY`.
    pub mob_explosion_drop_decay: bool,
    /// `GameRules.TNT_EXPLOSION_DROP_DECAY`.
    pub tnt_explosion_drop_decay: bool,
    /// `GameRules.TNT_EXPLODES`.
    pub tnt_explodes: bool,
    /// `GameRules.BLOCK_DROPS` (`Block.popResource`).
    pub block_drops: bool,
}

impl Default for ExplosionRules {
    /// The vanilla defaults of `GameRules`.
    fn default() -> Self {
        Self {
            mob_griefing: true,
            block_explosion_drop_decay: true,
            mob_explosion_drop_decay: true,
            tnt_explosion_drop_decay: false,
            tnt_explodes: true,
            block_drops: true,
        }
    }
}

impl ExplosionRules {
    /// Reads the rules from the live game-rule store.
    pub fn read(rules: &crate::game_rules::LiveGameRules) -> Self {
        Self {
            mob_griefing: rules.bool("mob_griefing"),
            block_explosion_drop_decay: rules.bool("block_explosion_drop_decay"),
            mob_explosion_drop_decay: rules.bool("mob_explosion_drop_decay"),
            tnt_explosion_drop_decay: rules.bool("tnt_explosion_drop_decay"),
            tnt_explodes: rules.bool("tnt_explodes"),
            block_drops: rules.bool("block_drops"),
        }
    }
}

impl LevelExplosionInteraction {
    /// The `switch (interactionType)` of `ServerLevel.explode`.
    pub fn block_interaction(self, rules: &ExplosionRules) -> ExplosionBlockInteraction {
        let destroy = |decay: bool| {
            if decay {
                ExplosionBlockInteraction::DestroyWithDecay
            } else {
                ExplosionBlockInteraction::Destroy
            }
        };
        match self {
            Self::None => ExplosionBlockInteraction::Keep,
            Self::Block => destroy(rules.block_explosion_drop_decay),
            Self::Mob if rules.mob_griefing => destroy(rules.mob_explosion_drop_decay),
            Self::Mob => ExplosionBlockInteraction::Keep,
            Self::Tnt => destroy(rules.tnt_explosion_drop_decay),
            Self::Trigger => ExplosionBlockInteraction::TriggerBlock,
        }
    }
}

/// `Explosion.BlockInteraction.shouldAffectBlocklikeEntities()`.
pub fn interaction_affects_blocklike_entities(interaction: ExplosionBlockInteraction) -> bool {
    matches!(
        interaction,
        ExplosionBlockInteraction::Destroy | ExplosionBlockInteraction::DestroyWithDecay
    )
}

/// The `ExplosionDamageCalculator` variants an explosion uses for blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockCalculator {
    /// `ExplosionDamageCalculator` / `EntityBasedExplosionDamageCalculator`
    /// for a source that does not override the block hooks (`Entity`'s
    /// defaults return the resistance unchanged and allow every block).
    Default,
    /// `PrimedTnt.USED_PORTAL_DAMAGE_CALCULATOR`: nether portals are ignored.
    UsedPortal,
}

impl BlockCalculator {
    /// `getBlockExplosionResistance`: `None` (`Optional.empty()`) for air with
    /// no fluid, else the larger of block and fluid resistance.
    pub fn block_resistance(self, state: &BlockStateModel) -> Option<f32> {
        if self == Self::UsedPortal && state.registry_id == "minecraft:nether_portal" {
            return None;
        }
        let fluid = state_physics_by_name(&state.state_name())
            .map_or(StateFluid::Empty, |physics| physics.fluid);
        let fluid_resistance = match fluid {
            StateFluid::Empty => 0.0,
            StateFluid::Water { .. } | StateFluid::Lava { .. } => FLUID_EXPLOSION_RESISTANCE,
        };
        if state.is_air() && fluid == StateFluid::Empty {
            return None;
        }
        let block_resistance =
            block_physics(&state.registry_id).map_or(0.5, |physics| physics.explosion_resistance);
        Some(block_resistance.max(fluid_resistance))
    }

    /// `shouldBlockExplode`.
    pub fn should_block_explode(self, state: &BlockStateModel) -> bool {
        !(self == Self::UsedPortal && state.registry_id == "minecraft:nether_portal")
    }
}

/// `Level.isInWorldBounds(pos)` for the overworld.
pub fn is_in_world_bounds(pos: BlockPos) -> bool {
    pos.y >= crate::world::OVERWORLD_MIN_Y
        && pos.y < crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT
        && (-30_000_000..30_000_000).contains(&pos.x)
        && (-30_000_000..30_000_000).contains(&pos.z)
}

/// `ServerExplosion.calculateExplodedPositions`: the blocks the blast
/// destroys, in unspecified order.
pub fn calculate_exploded(
    world: &impl SurvivalWorld,
    center: Vec3,
    radius: f32,
    calculator: BlockCalculator,
    random: &mut impl ExplosionRandom,
) -> Vec<BlockPos> {
    calculate_exploded_positions(
        crate::entity_physics::Vec3 {
            x: center.x,
            y: center.y,
            z: center.z,
        },
        radius,
        |_| random.next_float(),
        is_in_world_bounds,
        |pos| calculator.block_resistance(&world.state_at(pos)),
        |pos, _power| calculator.should_block_explode(&world.state_at(pos)),
    )
    .into_iter()
    .map(|(x, y, z)| BlockPos { x, y, z })
    .collect()
}

/// `Util.shuffle(list, random)`: Fisher-Yates from the back.
pub fn shuffle<T>(list: &mut [T], random: &mut impl ExplosionRandom) {
    for i in (2..=list.len()).rev() {
        let j = random.next_int(i as i32) as usize;
        list.swap(i - 1, j);
    }
}

/// `ServerExplosion.getSeenPercent(center, entity)`: the fraction of sample
/// points of the entity's bounding box that have an unobstructed line to
/// `center`.
pub fn get_seen_percent(world: &impl SurvivalWorld, center: Vec3, bb: &Aabb) -> f32 {
    let xs = 1.0 / ((bb.max_x - bb.min_x) * 2.0 + 1.0);
    let ys = 1.0 / ((bb.max_y - bb.min_y) * 2.0 + 1.0);
    let zs = 1.0 / ((bb.max_z - bb.min_z) * 2.0 + 1.0);
    let x_offset = (1.0 - (1.0 / xs).floor() * xs) / 2.0;
    let z_offset = (1.0 - (1.0 / zs).floor() * zs) / 2.0;
    if xs < 0.0 || ys < 0.0 || zs < 0.0 {
        return 0.0;
    }
    let lerp = |t: f64, a: f64, b: f64| a + t * (b - a);
    let (mut hits, mut count) = (0_i32, 0_i32);
    let mut xx = 0.0;
    while xx <= 1.0 {
        let mut yy = 0.0;
        while yy <= 1.0 {
            let mut zz = 0.0;
            while zz <= 1.0 {
                let from = Vec3 {
                    x: lerp(xx, bb.min_x, bb.max_x) + x_offset,
                    y: lerp(yy, bb.min_y, bb.max_y),
                    z: lerp(zz, bb.min_z, bb.max_z) + z_offset,
                };
                if !clip_hits_collider(world, from, center) {
                    hits += 1;
                }
                count += 1;
                zz += zs;
            }
            yy += ys;
        }
        xx += xs;
    }
    hits as f32 / count as f32
}

/// What one explosion does to one entity (`ServerExplosion.hurtEntities`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityImpact {
    /// `ExplosionDamageCalculator.getEntityDamageAmount`.
    pub damage: f32,
    /// The `entity.push(knockback)` vector.
    pub knockback: Vec3,
}

/// The per-entity arithmetic of `hurtEntities`, given the entity's exposure.
pub struct EntityTarget {
    /// `Entity.position()` (distance is measured from the feet).
    pub position: Vec3,
    /// `PrimedTnt ? position() : getEyePosition()`.
    pub origin: Vec3,
    /// `LivingEntity.getAttributeValue(EXPLOSION_KNOCKBACK_RESISTANCE)`, else 0.
    pub knockback_resistance: f64,
}

/// `dist = sqrt(entity.distanceToSqr(center)) / doubleRadius`.
pub fn explosion_distance(center: Vec3, radius: f32, position: Vec3) -> f64 {
    let (dx, dy, dz) = (position.x - center.x, position.y - center.y, position.z - center.z);
    (dx * dx + dy * dy + dz * dz).sqrt() / f64::from(radius * 2.0)
}

/// Damage and knockback for an entity within range; `None` when it is out of
/// range (`dist > 1`). `should_damage` / `knockback_multiplier` come from the
/// `ExplosionDamageCalculator`.
pub fn entity_impact(
    center: Vec3,
    radius: f32,
    target: &EntityTarget,
    exposure: f32,
    knockback_multiplier: f32,
) -> Option<EntityImpact> {
    let double_radius = radius * 2.0;
    let dist = explosion_distance(center, radius, target.position);
    if dist > 1.0 {
        return None;
    }
    // `getEntityDamageAmount`.
    let pow = (1.0 - dist) * f64::from(exposure);
    let damage = ((pow * pow + pow) / 2.0 * 7.0 * f64::from(double_radius) + 1.0) as f32;
    let direction = normalize(Vec3 {
        x: target.origin.x - center.x,
        y: target.origin.y - center.y,
        z: target.origin.z - center.z,
    });
    let power = (1.0 - dist)
        * f64::from(exposure)
        * f64::from(knockback_multiplier)
        * (1.0 - target.knockback_resistance);
    Some(EntityImpact {
        damage,
        knockback: Vec3 {
            x: direction.x * power,
            y: direction.y * power,
            z: direction.z * power,
        },
    })
}

/// `Vec3.normalize()`.
fn normalize(v: Vec3) -> Vec3 {
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if length < 1.0E-5 {
        Vec3::ZERO
    } else {
        Vec3 {
            x: v.x / length,
            y: v.y / length,
            z: v.z / length,
        }
    }
}

/// `ServerExplosion.StackCollector`: a block drop combined with later drops
/// of the same item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectedStack {
    /// Position of the first block that dropped into this stack.
    pub pos: BlockPos,
    pub item: &'static str,
    pub count: i32,
}

/// `ItemStack.getMaxStackSize()` of a bare stack of `item`.
///
/// TODO(item-max-stack-table): shares the name-based table of `/give`; the
/// vanilla per-item `max_stack_size` components are not modelled for every
/// item yet.
fn max_stack_size(item: &str) -> i32 {
    crate::command::inventory_items::item_max_stack_size(item)
}

/// `ServerExplosion.addOrAppendStack(stacks, stack, pos)`.
pub fn add_or_append_stack(
    stacks: &mut Vec<CollectedStack>,
    item: &'static str,
    mut count: i32,
    pos: BlockPos,
) {
    let max = max_stack_size(item);
    for collector in stacks.iter_mut() {
        // `ItemEntity.areMergable`: same item and the sum fits one stack.
        if collector.item == item && collector.count + count <= max {
            // `ItemEntity.merge(stack, input, 16)`.
            let delta = (max.min(MAX_DROPS_PER_COMBINED_STACK) - collector.count).min(count);
            collector.count += delta;
            count -= delta;
        }
        if count <= 0 {
            return;
        }
    }
    stacks.push(CollectedStack { pos, item, count });
}

#[cfg(test)]
mod tests;
