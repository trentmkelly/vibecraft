//! Respawn position resolution.
//!
//! Java references: `ServerPlayer.findRespawnPositionAndUseSpawnBlock` /
//! `findRespawnAndUseSpawnBlock`, `BedBlock.findStandUpPosition`,
//! `RespawnAnchorBlock.findStandUpPosition` and `DismountHelper.findSafeDismountLocation`.
//!
//! The logic is pure over a [`RespawnWorld`] so it can be unit-tested against a small
//! in-memory block map; the live server implements the trait over its chunk cache.

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{shape, state_physics_by_name};
use crate::block_tags::block_tag_contains;
use crate::block_update::{BlockPos, Direction};

/// `EntityType.PLAYER` dimensions (`sized(0.6, 1.8)`).
const PLAYER_WIDTH: f64 = 0.6;
const PLAYER_HEIGHT: f64 = 1.8;

/// `ServerPlayer.RespawnConfig`: the stored respawn block plus the `forced` flag.
///
/// The dimension is kept next to the config by the caller (`RespawnData.dimension`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RespawnConfig {
    pub pos: BlockPos,
    pub yaw: f32,
    pub pitch: f32,
    pub forced: bool,
}

/// `ServerPlayer.RespawnPosAngle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RespawnPosAngle {
    pub position: (f64, f64, f64),
    pub yaw: f32,
    pub pitch: f32,
}

/// The world queries respawn resolution needs (`ServerLevel` in Java).
pub trait RespawnWorld {
    /// `Level.getBlockState(pos)`.
    fn block_state(&self, pos: BlockPos) -> BlockStateModel;
    /// `EnvironmentAttributes.RESPAWN_ANCHOR_WORKS` at `pos`.
    fn respawn_anchor_works(&self, pos: BlockPos) -> bool;
    /// `EnvironmentAttributes.BED_RULE` at `pos`, `.canSetSpawn(level)`.
    fn bed_can_set_spawn(&self, pos: BlockPos) -> bool;
    /// `level.setBlock(pos, state, 3)` — used to spend a respawn anchor charge.
    fn set_block_state(&mut self, pos: BlockPos, state: BlockStateModel);
}

/// `ServerPlayer.findRespawnAndUseSpawnBlock(level, config, consumeSpawnBlock)`.
/// `None` corresponds to `Optional.empty()` (the spawn block is missing/obstructed).
pub fn find_respawn_and_use_spawn_block(
    world: &mut impl RespawnWorld,
    config: RespawnConfig,
    consume_spawn_block: bool,
) -> Option<RespawnPosAngle> {
    let pos = config.pos;
    let block_state = world.block_state(pos);
    let block = block_state.registry_id.as_str();

    if block == "minecraft:respawn_anchor"
        && (config.forced || anchor_charge(&block_state) > 0)
        && world.respawn_anchor_works(pos)
    {
        let stand_up = anchor_stand_up_position(world, pos);
        if !config.forced && consume_spawn_block && stand_up.is_some() {
            let charge = anchor_charge(&block_state) - 1;
            let spent = block_state.with_property("charge", charge.to_string());
            world.set_block_state(pos, spent);
        }
        return stand_up.map(|position| pos_angle_looking_at(position, pos, 0.0));
    }

    if block.ends_with("_bed") && world.bed_can_set_spawn(pos) {
        let facing = block_state
            .properties
            .get("facing")
            .and_then(|value| direction_by_name(value))
            .unwrap_or(Direction::North);
        return bed_stand_up_position(world, pos, facing, config.yaw)
            .map(|position| pos_angle_looking_at(position, pos, 0.0));
    }

    if !config.forced {
        return None;
    }

    // Block.isPossibleToRespawnInThis: neither solid nor liquid, for feet and head block.
    let free_bottom = possible_to_respawn_in(&block_state);
    let free_top = possible_to_respawn_in(&world.block_state(pos.relative(Direction::Up)));
    (free_bottom && free_top).then_some(RespawnPosAngle {
        position: (
            f64::from(pos.x) + 0.5,
            f64::from(pos.y) + 0.1,
            f64::from(pos.z) + 0.5,
        ),
        yaw: config.yaw,
        pitch: config.pitch,
    })
}

/// `Block.isPossibleToRespawnInThis(state)`: `!state.isSolid() && !state.liquid()`.
fn possible_to_respawn_in(state: &BlockStateModel) -> bool {
    state_physics_by_name(&state.state_name())
        .is_some_and(|physics| !physics.is_solid && !physics.liquid)
}

fn anchor_charge(state: &BlockStateModel) -> i32 {
    state
        .properties
        .get("charge")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// `ServerPlayer.RespawnPosAngle.of(position, lookAtBlockPos, pitch)`.
fn pos_angle_looking_at(position: (f64, f64, f64), look_at: BlockPos, pitch: f32) -> RespawnPosAngle {
    // Vec3.atBottomCenterOf(lookAt).subtract(position).normalize(), then atan2(z, x).
    let dx = f64::from(look_at.x) + 0.5 - position.0;
    let dz = f64::from(look_at.z) + 0.5 - position.2;
    let dy = f64::from(look_at.y) - position.1;
    let length = (dx * dx + dy * dy + dz * dz).sqrt();
    let (nx, nz) = if length < 1.0E-5 { (0.0, 0.0) } else { (dx / length, dz / length) };
    let degrees = f64::from(nz.atan2(nx) as f32 * 180.0 / std::f32::consts::PI) - 90.0;
    RespawnPosAngle {
        position,
        yaw: wrap_degrees(degrees) as f32,
        pitch,
    }
}

/// `Mth.wrapDegrees(double)`.
fn wrap_degrees(degrees: f64) -> f64 {
    let mut wrapped = degrees % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

fn direction_by_name(name: &str) -> Option<Direction> {
    Some(match name {
        "north" => Direction::North,
        "south" => Direction::South,
        "west" => Direction::West,
        "east" => Direction::East,
        _ => return None,
    })
}

fn step(direction: Direction) -> (i32, i32) {
    match direction {
        Direction::North => (0, -1),
        Direction::South => (0, 1),
        Direction::West => (-1, 0),
        Direction::East => (1, 0),
        Direction::Up | Direction::Down => (0, 0),
    }
}

/// `Direction.getClockWise()` for horizontal directions.
fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        other => other,
    }
}

/// `Direction.isFacingAngle(yAngle)`.
fn is_facing_angle(direction: Direction, y_angle: f32) -> bool {
    let radians = y_angle * (std::f32::consts::PI / 180.0);
    let dx = -radians.sin();
    let dz = radians.cos();
    let (step_x, step_z) = step(direction);
    step_x as f32 * dx + step_z as f32 * dz > 0.0
}

/// `BedBlock.findStandUpPosition(PLAYER, level, pos, forward, yaw)`.
pub fn bed_stand_up_position(
    world: &impl RespawnWorld,
    pos: BlockPos,
    forward: Direction,
    yaw: f32,
) -> Option<(f64, f64, f64)> {
    let right = clockwise(forward);
    let side = if is_facing_angle(right, yaw) { right.opposite() } else { right };
    let below = world.block_state(pos.relative(Direction::Down));
    if below.registry_id.ends_with("_bed") {
        return bunk_bed_stand_up_position(world, pos, forward, side);
    }
    let offsets = bed_stand_up_offsets(forward, side);
    stand_up_at_offsets(world, pos, &offsets, true)
        .or_else(|| stand_up_at_offsets(world, pos, &offsets, false))
}

/// `BedBlock.findBunkBedStandUpPosition`.
fn bunk_bed_stand_up_position(
    world: &impl RespawnWorld,
    pos: BlockPos,
    forward: Direction,
    side: Direction,
) -> Option<(f64, f64, f64)> {
    let surround = bed_surround_offsets(forward, side);
    let above = bed_above_offsets(forward);
    let below = pos.relative(Direction::Down);
    stand_up_at_offsets(world, pos, &surround, true)
        .or_else(|| stand_up_at_offsets(world, below, &surround, true))
        .or_else(|| stand_up_at_offsets(world, pos, &above, true))
        .or_else(|| stand_up_at_offsets(world, pos, &surround, false))
        .or_else(|| stand_up_at_offsets(world, below, &surround, false))
        .or_else(|| stand_up_at_offsets(world, pos, &above, false))
}

/// `BedBlock.bedStandUpOffsets` = surround offsets followed by the "above" offsets.
fn bed_stand_up_offsets(forward: Direction, side: Direction) -> Vec<(i32, i32)> {
    let mut offsets = bed_surround_offsets(forward, side);
    offsets.extend(bed_above_offsets(forward));
    offsets
}

fn bed_surround_offsets(forward: Direction, side: Direction) -> Vec<(i32, i32)> {
    let (sx, sz) = step(side);
    let (fx, fz) = step(forward);
    vec![
        (sx, sz),
        (sx - fx, sz - fz),
        (sx - fx * 2, sz - fz * 2),
        (-fx * 2, -fz * 2),
        (-sx - fx * 2, -sz - fz * 2),
        (-sx - fx, -sz - fz),
        (-sx, -sz),
        (-sx + fx, -sz + fz),
        (fx, fz),
        (sx + fx, sz + fz),
    ]
}

fn bed_above_offsets(forward: Direction) -> Vec<(i32, i32)> {
    let (fx, fz) = step(forward);
    vec![(0, 0), (-fx, -fz)]
}

fn stand_up_at_offsets(
    world: &impl RespawnWorld,
    pos: BlockPos,
    offsets: &[(i32, i32)],
    check_dangerous: bool,
) -> Option<(f64, f64, f64)> {
    offsets.iter().find_map(|(dx, dz)| {
        let candidate = BlockPos { x: pos.x + dx, y: pos.y, z: pos.z + dz };
        find_safe_dismount_location(world, candidate, check_dangerous)
    })
}

/// `RespawnAnchorBlock.findStandUpPosition(PLAYER, level, pos)`: the safe pass first, then
/// the dangerous-allowed pass over `RESPAWN_OFFSETS`.
fn anchor_stand_up_position(world: &impl RespawnWorld, pos: BlockPos) -> Option<(f64, f64, f64)> {
    anchor_stand_up_pass(world, pos, true).or_else(|| anchor_stand_up_pass(world, pos, false))
}

fn anchor_stand_up_pass(
    world: &impl RespawnWorld,
    pos: BlockPos,
    check_dangerous: bool,
) -> Option<(f64, f64, f64)> {
    anchor_respawn_offsets().into_iter().find_map(|(dx, dy, dz)| {
        let candidate = BlockPos { x: pos.x + dx, y: pos.y + dy, z: pos.z + dz };
        find_safe_dismount_location(world, candidate, check_dangerous)
    })
}

/// `RespawnAnchorBlock.RESPAWN_OFFSETS`: the horizontal ring, the ring one lower, the ring
/// one higher, then straight up.
fn anchor_respawn_offsets() -> Vec<(i32, i32, i32)> {
    // RESPAWN_HORIZONTAL_OFFSETS = [(0,-1),(-1,0),(0,1),(1,0),(-1,-1),(1,-1),(-1,1),(1,1)]
    const RING: [(i32, i32); 8] =
        [(0, -1), (-1, 0), (0, 1), (1, 0), (-1, -1), (1, -1), (-1, 1), (1, 1)];
    let mut offsets: Vec<(i32, i32, i32)> = RING.iter().map(|(x, z)| (*x, 0, *z)).collect();
    offsets.extend(RING.iter().map(|(x, z)| (*x, -1, *z)));
    offsets.extend(RING.iter().map(|(x, z)| (*x, 1, *z)));
    offsets.push((0, 1, 0));
    offsets
}

/// `DismountHelper.findSafeDismountLocation(PLAYER, level, pos, checkDangerous)`.
pub fn find_safe_dismount_location(
    world: &impl RespawnWorld,
    pos: BlockPos,
    check_dangerous: bool,
) -> Option<(f64, f64, f64)> {
    let state = world.block_state(pos);
    if check_dangerous && is_block_dangerous(&state) {
        return None;
    }
    let floor_height = block_floor_height(
        &non_climbable_shape(&state),
        || non_climbable_shape(&world.block_state(pos.relative(Direction::Down))),
    );
    if floor_height.is_infinite() || floor_height >= 1.0 {
        return None;
    }
    if check_dangerous
        && floor_height <= 0.0
        && is_block_dangerous(&world.block_state(pos.relative(Direction::Down)))
    {
        return None;
    }
    let position = (
        f64::from(pos.x) + 0.5,
        f64::from(pos.y) + floor_height,
        f64::from(pos.z) + 0.5,
    );
    if collides_with_blocks(world, position) {
        return None;
    }
    let above = world.block_state(pos.relative(Direction::Up));
    if invalid_spawn_inside(&state) || invalid_spawn_inside(&above) {
        return None;
    }
    // World border containment is not modelled for respawn candidates yet.
    Some(position)
}

/// `BlockGetter.getBlockFloorHeight(blockShape, belowShape)`.
fn block_floor_height(block_shape: &[[f64; 6]], below_shape: impl FnOnce() -> Vec<[f64; 6]>) -> f64 {
    if !block_shape.is_empty() {
        return max_y(block_shape);
    }
    let below_floor = max_y(&below_shape());
    if below_floor >= 1.0 {
        below_floor - 1.0
    } else {
        f64::NEG_INFINITY
    }
}

/// `VoxelShape.max(Axis.Y)` (negative infinity for the empty shape).
fn max_y(boxes: &[[f64; 6]]) -> f64 {
    boxes.iter().map(|b| b[4]).fold(f64::NEG_INFINITY, f64::max)
}

/// `DismountHelper.nonClimbableShape`: climbables and open trapdoors have no collision.
fn non_climbable_shape(state: &BlockStateModel) -> Vec<[f64; 6]> {
    let id = state.registry_id.as_str();
    let open_trapdoor = id.ends_with("_trapdoor")
        && state.properties.get("open").is_some_and(|open| open == "true");
    if block_tag_contains("climbable", id) || open_trapdoor {
        return Vec::new();
    }
    collision_boxes(state)
}

fn collision_boxes(state: &BlockStateModel) -> Vec<[f64; 6]> {
    state_physics_by_name(&state.state_name())
        .map(|physics| shape(physics.collision_shape).to_vec())
        .unwrap_or_default()
}

/// Whether any block collision box overlaps the player's bounding box at `feet`
/// (`level.getBlockCollisions(null, aabb)` with a non-empty shape).
fn collides_with_blocks(world: &impl RespawnWorld, feet: (f64, f64, f64)) -> bool {
    let half = PLAYER_WIDTH / 2.0;
    let min = (feet.0 - half, feet.1, feet.2 - half);
    let max = (feet.0 + half, feet.1 + PLAYER_HEIGHT, feet.2 + half);
    // Blocks below `feet.1` can still reach up (fences are 1.5 tall), so start one lower.
    let (x0, x1) = (min.0.floor() as i32, max.0.floor() as i32);
    let (y0, y1) = (min.1.floor() as i32 - 1, max.1.floor() as i32);
    let (z0, z1) = (min.2.floor() as i32, max.2.floor() as i32);
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let state = world.block_state(BlockPos { x, y, z });
                for b in collision_boxes(&state) {
                    let overlaps = min.0 < f64::from(x) + b[3]
                        && max.0 > f64::from(x) + b[0]
                        && min.1 < f64::from(y) + b[4]
                        && max.1 > f64::from(y) + b[1]
                        && min.2 < f64::from(z) + b[5]
                        && max.2 > f64::from(z) + b[2];
                    if overlaps {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// `BlockTags.INVALID_SPAWN_INSIDE`.
fn invalid_spawn_inside(state: &BlockStateModel) -> bool {
    matches!(state.registry_id.as_str(), "minecraft:end_portal" | "minecraft:end_gateway")
}

/// `EntityType.isBlockDangerous(state)` for the player (`immuneTo` is empty, not fire immune).
fn is_block_dangerous(state: &BlockStateModel) -> bool {
    let id = state.registry_id.as_str();
    if is_burning_block(state) {
        return true;
    }
    matches!(
        id,
        "minecraft:wither_rose"
            | "minecraft:sweet_berry_bush"
            | "minecraft:cactus"
            | "minecraft:powder_snow"
    )
}

/// `NodeEvaluator.isBurningBlock`.
fn is_burning_block(state: &BlockStateModel) -> bool {
    let id = state.registry_id.as_str();
    block_tag_contains("fire", id)
        || matches!(id, "minecraft:lava" | "minecraft:magma_block" | "minecraft:lava_cauldron")
        || (block_tag_contains("campfires", id)
            && state.properties.get("lit").is_some_and(|lit| lit == "true"))
}

#[cfg(test)]
mod tests;
