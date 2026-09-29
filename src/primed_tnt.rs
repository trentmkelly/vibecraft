//! Java `net.minecraft.world.entity.item.PrimedTnt`: the lit TNT entity.
//!
//! The entity simulation is pure: [`PrimedTntEntity::tick`] advances physics
//! against a [`SurvivalWorld`] and reports whether the fuse ran out. The live
//! server drives it from the shared world tick and performs the explosion.

use crate::block_properties::block_physics;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::collision_shape::Aabb;
use crate::damage_type::DamageEntityRef;
use crate::entity_collision::{
    aabb_offset, collide_bounding_box, find_supporting_block,
};
use crate::entity_fluid::{
    update_fluid_interaction, LAVA_CURRENT_SCALE, WATER_CURRENT_SCALE,
};
use crate::network::play::Vec3;
use crate::server_entity_sync::EntitySync;

/// `PrimedTnt.DEFAULT_FUSE_TIME`.
pub const DEFAULT_FUSE_TIME: i32 = 80;
/// `PrimedTnt.DEFAULT_EXPLOSION_POWER`.
pub const DEFAULT_EXPLOSION_POWER: f32 = 4.0;
/// `EntityType.TNT` `sized(0.98F, 0.98F)`.
pub const TNT_SIZE: f64 = 0.98_f32 as f64;
/// `EntityType.TNT` `eyeHeight(0.15F)`.
pub const TNT_EYE_HEIGHT: f64 = 0.15_f32 as f64;
/// `EntityType.TNT` `updateInterval(10)`.
pub const TNT_UPDATE_INTERVAL: i32 = 10;
/// `PrimedTnt.getDefaultGravity`.
const GRAVITY: f64 = 0.04;
/// `Mth.equal` tolerance.
const EQUAL_EPSILON: f64 = 1.0E-5;
/// The block state `PrimedTnt.DEFAULT_BLOCK_STATE` (`Blocks.TNT.defaultBlockState()`).
pub const DEFAULT_BLOCK_STATE: &str = "minecraft:tnt[unstable=false]";

/// Result of one [`PrimedTntEntity::tick`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TntTick {
    /// The fuse has not run out.
    Burning,
    /// `fuse <= 0`: the entity is discarded and `explode()` runs.
    Detonate,
}

/// A lit TNT block (`PrimedTnt`).
#[derive(Clone, Debug, PartialEq)]
pub struct PrimedTntEntity {
    /// `Entity.getId()`.
    pub entity_id: i32,
    /// `Entity.position()`.
    pub pos: Vec3,
    /// `Entity.getDeltaMovement()`.
    pub delta: Vec3,
    /// `PrimedTnt.DATA_FUSE_ID`.
    pub fuse: i32,
    /// `PrimedTnt.explosionPower`.
    pub explosion_power: f32,
    /// `PrimedTnt.owner` (the living entity that lit it).
    pub owner: Option<DamageEntityRef>,
    /// `PrimedTnt.DATA_BLOCK_STATE_ID`.
    pub block_state: String,
    /// `PrimedTnt.usedPortal`.
    pub used_portal: bool,
    /// `Entity.onGround()`.
    pub on_ground: bool,
    /// `Entity.needsSync`, set by [`Self::push`] and consumed by the tracker.
    pub needs_sync: bool,
    /// `Entity.mainSupportingBlockPos`.
    main_supporting_block: Option<BlockPos>,
    /// `Entity.onGroundNoBlocks`.
    on_ground_no_blocks: bool,
    /// Whether the spawn packets were broadcast yet.
    pub announced: bool,
    /// `TntBlock.prime` plays `SoundEvents.TNT_PRIMED` when the entity spawns.
    pub play_prime_sound: bool,
    /// `ServerEntity` bookkeeping, created when the entity is first announced.
    pub sync: Option<EntitySync>,
}

impl PrimedTntEntity {
    /// `new PrimedTnt(level, x, y, z, owner)`; `random_angle` is
    /// `level.getRandom().nextDouble()`.
    pub fn new(
        entity_id: i32,
        (x, y, z): (f64, f64, f64),
        owner: Option<DamageEntityRef>,
        random_angle: f64,
    ) -> Self {
        let rot = random_angle * f64::from(std::f32::consts::PI * 2.0);
        Self {
            entity_id,
            pos: Vec3 { x, y, z },
            delta: Vec3 {
                x: -rot.sin() * 0.02,
                y: f64::from(0.2_f32),
                z: -rot.cos() * 0.02,
            },
            fuse: DEFAULT_FUSE_TIME,
            explosion_power: DEFAULT_EXPLOSION_POWER,
            owner,
            block_state: DEFAULT_BLOCK_STATE.to_string(),
            used_portal: false,
            on_ground: false,
            needs_sync: false,
            main_supporting_block: None,
            on_ground_no_blocks: false,
            announced: false,
            play_prime_sound: false,
            sync: None,
        }
    }

    /// `Entity.getBoundingBox()` for the current position.
    pub fn bounding_box(&self) -> Aabb {
        let half = TNT_SIZE / 2.0;
        Aabb::new(
            self.pos.x - half,
            self.pos.y,
            self.pos.z - half,
            self.pos.x + half,
            self.pos.y + TNT_SIZE,
            self.pos.z + half,
        )
    }

    /// `Entity.getY(0.0625)`: the explosion centre height.
    pub fn y_at(&self, fraction: f64) -> f64 {
        self.pos.y + TNT_SIZE * fraction
    }

    /// `Entity.push(Vec3)`: adds to the motion (ignored when non-finite).
    pub fn push(&mut self, impulse: Vec3) {
        if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
            self.delta.x += impulse.x;
            self.delta.y += impulse.y;
            self.delta.z += impulse.z;
            self.needs_sync = true;
        }
    }

    /// `Entity.blockPosition()`.
    pub fn block_position(&self) -> BlockPos {
        BlockPos {
            x: self.pos.x.floor() as i32,
            y: self.pos.y.floor() as i32,
            z: self.pos.z.floor() as i32,
        }
    }

    /// `Entity.getOnPos(offset)`.
    fn on_pos(&self, world: &impl SurvivalWorld, offset: f64) -> BlockPos {
        if let Some(supporting) = self.main_supporting_block {
            if offset <= f64::from(1.0E-5_f32) {
                return supporting;
            }
            let state = world.state_at(supporting);
            let tags = |tag: &str| crate::block_tags::block_tag_contains(tag, &state.registry_id);
            let keep = (offset <= 0.5 && tags("fences")) || tags("walls") || tags("fence_gates");
            return if keep {
                supporting
            } else {
                BlockPos {
                    y: (self.pos.y - offset).floor() as i32,
                    ..supporting
                }
            };
        }
        BlockPos {
            x: self.pos.x.floor() as i32,
            y: (self.pos.y - offset).floor() as i32,
            z: self.pos.z.floor() as i32,
        }
    }

    /// `Entity.checkSupportingBlock(onGround, movement)`.
    fn check_supporting_block(
        &mut self,
        world: &impl SurvivalWorld,
        on_ground: bool,
        movement: Vec3,
    ) {
        if !on_ground {
            self.on_ground_no_blocks = false;
            self.main_supporting_block = None;
            return;
        }
        let bb = self.bounding_box();
        let test_area = Aabb::new(bb.min_x, bb.min_y - 1.0E-6, bb.min_z, bb.max_x, bb.min_y, bb.max_z);
        let mut supporting = find_supporting_block(world, &test_area, self.pos);
        if supporting.is_some() || self.on_ground_no_blocks {
            self.main_supporting_block = supporting;
        } else {
            let shifted = aabb_offset(test_area, -movement.x, 0.0, -movement.z);
            supporting = find_supporting_block(world, &shifted, self.pos);
            self.main_supporting_block = supporting;
        }
        self.on_ground_no_blocks = supporting.is_none();
    }

    /// `Block.updateEntityMovementAfterFallOn` for the block under the entity.
    fn update_movement_after_fall_on(&mut self, on_block: &str) {
        if on_block == "minecraft:slime_block" {
            // SlimeBlock.bounceUp: non-living entities keep 0.8 of the speed.
            if self.delta.y < 0.0 {
                self.delta.y = -self.delta.y * 0.8;
            }
        } else if on_block.ends_with("_bed") {
            // BedBlock.bounceUp.
            if self.delta.y < 0.0 {
                self.delta.y = -self.delta.y * f64::from(0.66_f32) * 0.8;
            }
        } else {
            // Block.updateEntityMovementAfterFallOn.
            self.delta.y = 0.0;
        }
    }

    /// `Entity.getBlockSpeedFactor`.
    fn block_speed_factor(&self, world: &impl SurvivalWorld) -> f32 {
        let factor_of = |pos: BlockPos| {
            let state = world.state_at(pos);
            let factor =
                block_physics(&state.registry_id).map_or(1.0, |physics| physics.speed_factor);
            (state, factor)
        };
        let (state, here) = factor_of(self.block_position());
        if state.registry_id != "minecraft:water" && state.registry_id != "minecraft:bubble_column" {
            if here == 1.0 {
                factor_of(self.on_pos(world, f64::from(0.500_001_f32))).1
            } else {
                here
            }
        } else {
            here
        }
    }

    /// `Entity.move(MoverType.SELF, delta)` for an entity with no step height.
    fn move_entity(&mut self, world: &impl SurvivalWorld, delta: Vec3) {
        let movement = collide_bounding_box(world, &self.bounding_box(), delta);
        let movement_sqr = movement.x * movement.x + movement.y * movement.y + movement.z * movement.z;
        let delta_sqr = delta.x * delta.x + delta.y * delta.y + delta.z * delta.z;
        if movement_sqr > 1.0E-7 || delta_sqr - movement_sqr < 1.0E-7 {
            self.pos = Vec3 {
                x: self.pos.x + movement.x,
                y: self.pos.y + movement.y,
                z: self.pos.z + movement.z,
            };
        }
        let x_collision = (delta.x - movement.x).abs() >= EQUAL_EPSILON;
        let z_collision = (delta.z - movement.z).abs() >= EQUAL_EPSILON;
        let horizontal_collision = x_collision || z_collision;
        // `isLocalInstanceAuthoritative()` holds for every server-side entity.
        let vertical_collision_below = delta.y != movement.y && delta.y < 0.0;
        self.on_ground = vertical_collision_below;
        self.check_supporting_block(world, vertical_collision_below, movement);
        let effect_pos = self.on_pos(world, f64::from(0.2_f32));
        let effect_block = world.state_at(effect_pos).registry_id;
        if horizontal_collision {
            if x_collision {
                self.delta.x = 0.0;
            }
            if z_collision {
                self.delta.z = 0.0;
            }
        }
        if delta.y != movement.y {
            self.update_movement_after_fall_on(&effect_block);
        }
        // TODO(entity-movement-emission): PrimedTnt.getMovementEmission is NONE,
        // so no step sounds/vibrations are produced.
        let speed_factor = f64::from(self.block_speed_factor(world));
        self.delta.x *= speed_factor;
        self.delta.z *= speed_factor;
    }

    /// `Entity.updateFluidInteraction` for a fluid-pushed entity.
    fn update_fluid_interaction(&mut self, world: &impl SurvivalWorld) {
        let eye_y = self.pos.y + TNT_EYE_HEIGHT;
        let block = self.block_position();
        let interaction = update_fluid_interaction(
            world,
            &self.bounding_box(),
            (block.x, block.z),
            eye_y,
            false,
        );
        if interaction.water.in_fluid() {
            if let Some(impulse) = interaction.water.current_impulse(self.delta, WATER_CURRENT_SCALE) {
                self.add_delta_movement(impulse);
            }
        }
        if interaction.lava.in_fluid() {
            if let Some(impulse) = interaction.lava.current_impulse(self.delta, LAVA_CURRENT_SCALE) {
                self.add_delta_movement(impulse);
            }
        }
    }

    /// `Entity.addDeltaMovement`.
    fn add_delta_movement(&mut self, impulse: Vec3) {
        self.delta.x += impulse.x;
        self.delta.y += impulse.y;
        self.delta.z += impulse.z;
    }

    /// `Entity.mainSupportingBlockPos`, exposed for tests.
    #[cfg(test)]
    pub fn supporting_block(&self) -> Option<BlockPos> {
        self.main_supporting_block
    }

    /// `PrimedTnt.tick()`.
    ///
    /// TODO(entity-portals): `handlePortal` (nether/end portal travel) has no
    /// dimension travel to drive.
    /// TODO(entity-inside-block-effects): `applyEffectsFromBlocks` (cobweb and
    /// powder-snow slowdown, bubble columns, portals) is not ported.
    pub fn tick(&mut self, world: &impl SurvivalWorld) -> TntTick {
        // applyGravity
        self.delta.y -= GRAVITY;
        self.move_entity(world, self.delta);
        self.delta = Vec3 {
            x: self.delta.x * 0.98,
            y: self.delta.y * 0.98,
            z: self.delta.z * 0.98,
        };
        if self.on_ground {
            self.delta = Vec3 {
                x: self.delta.x * 0.7,
                y: self.delta.y * -0.5,
                z: self.delta.z * 0.7,
            };
        }
        self.fuse -= 1;
        if self.fuse <= 0 {
            return TntTick::Detonate;
        }
        self.update_fluid_interaction(world);
        TntTick::Burning
    }
}

#[cfg(test)]
mod tests;
