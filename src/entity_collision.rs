//! Entity-versus-block collision and ray clipping.
//!
//! Ports the parts of Java `Entity.move` / `Entity.collide`, `Shapes.collide`
//! and `BlockGetter.clip` / `traverseBlocks` that an entity without step
//! height (`maxUpStep() == 0`, e.g. `PrimedTnt`) and the explosion exposure
//! ray cast (`ServerExplosion.getSeenPercent`, `ClipContext.Block.COLLIDER`)
//! need. Block collision shapes come from the authoritative per-state shape
//! tables in [`crate::block_properties`].

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{shape, state_physics_by_name};
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::collision_shape::Aabb;
use crate::network::play::Vec3;

/// `Shapes.collide` / `VoxelShape.collideX` distance tolerance.
const COLLISION_EPSILON: f64 = 1.0E-7;

/// The collision boxes of `state` at `pos` (`BlockState.getCollisionShape`
/// moved to the block position).
///
/// TODO(block-shape-offset): `BlockBehaviour.getOffset` (bamboo, pointed
/// dripstone) is not applied; the shape tables carry the un-offset shape.
pub fn block_collision_boxes(state: &BlockStateModel, pos: BlockPos) -> Vec<Aabb> {
    let Some(physics) = state_physics_by_name(&state.state_name()) else {
        return Vec::new();
    };
    shape(physics.collision_shape)
        .iter()
        .map(|b| {
            Aabb::new(
                f64::from(pos.x) + b[0],
                f64::from(pos.y) + b[1],
                f64::from(pos.z) + b[2],
                f64::from(pos.x) + b[3],
                f64::from(pos.y) + b[4],
                f64::from(pos.z) + b[5],
            )
        })
        .collect()
}

/// `AABB.move(x, y, z)`.
pub fn aabb_offset(bb: Aabb, dx: f64, dy: f64, dz: f64) -> Aabb {
    Aabb::new(
        bb.min_x + dx,
        bb.min_y + dy,
        bb.min_z + dz,
        bb.max_x + dx,
        bb.max_y + dy,
        bb.max_z + dz,
    )
}

/// `AABB.expandTowards(movement)`: stretches the box along the movement.
pub fn aabb_expand_towards(bb: Aabb, movement: Vec3) -> Aabb {
    let (mut min_x, mut min_y, mut min_z) = (bb.min_x, bb.min_y, bb.min_z);
    let (mut max_x, mut max_y, mut max_z) = (bb.max_x, bb.max_y, bb.max_z);
    if movement.x < 0.0 {
        min_x += movement.x;
    } else if movement.x > 0.0 {
        max_x += movement.x;
    }
    if movement.y < 0.0 {
        min_y += movement.y;
    } else if movement.y > 0.0 {
        max_y += movement.y;
    }
    if movement.z < 0.0 {
        min_z += movement.z;
    } else if movement.z > 0.0 {
        max_z += movement.z;
    }
    Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z)
}

/// `AABB.intersects(AABB)`.
pub fn aabb_intersects(a: &Aabb, b: &Aabb) -> bool {
    a.min_x < b.max_x
        && a.max_x > b.min_x
        && a.min_y < b.max_y
        && a.max_y > b.min_y
        && a.min_z < b.max_z
        && a.max_z > b.min_z
}

/// Every collision box of the blocks that could touch `region`
/// (`Level.getBlockCollisions`; the cell range is widened by one block like
/// `BlockCollisions` so large shapes such as fences are found).
fn collect_block_colliders(world: &impl SurvivalWorld, region: &Aabb) -> Vec<Aabb> {
    let lo = |v: f64| (v - COLLISION_EPSILON).floor() as i32 - 1;
    let hi = |v: f64| (v + COLLISION_EPSILON).floor() as i32 + 1;
    let mut boxes = Vec::new();
    for x in lo(region.min_x)..=hi(region.max_x) {
        for y in lo(region.min_y)..=hi(region.max_y) {
            for z in lo(region.min_z)..=hi(region.max_z) {
                let pos = BlockPos { x, y, z };
                let state = world.state_at(pos);
                if !state.is_air() {
                    boxes.extend(block_collision_boxes(&state, pos));
                }
            }
        }
    }
    boxes
}

/// `Level.findSupportingBlock(entity, box)`: among the blocks whose collision
/// shape intersects `test_area`, the one whose centre is nearest to
/// `entity_pos` (ties resolve to the greater `BlockPos.compareTo`, i.e. Y then
/// Z then X).
pub fn find_supporting_block(
    world: &impl SurvivalWorld,
    test_area: &Aabb,
    entity_pos: Vec3,
) -> Option<BlockPos> {
    let lo = |v: f64| (v - COLLISION_EPSILON).floor() as i32 - 1;
    let hi = |v: f64| (v + COLLISION_EPSILON).floor() as i32 + 1;
    let mut best: Option<(BlockPos, f64)> = None;
    for x in lo(test_area.min_x)..=hi(test_area.max_x) {
        for y in lo(test_area.min_y)..=hi(test_area.max_y) {
            for z in lo(test_area.min_z)..=hi(test_area.max_z) {
                let pos = BlockPos { x, y, z };
                let state = world.state_at(pos);
                if state.is_air()
                    || !block_collision_boxes(&state, pos)
                        .iter()
                        .any(|b| aabb_intersects(b, test_area))
                {
                    continue;
                }
                let dx = f64::from(x) + 0.5 - entity_pos.x;
                let dy = f64::from(y) + 0.5 - entity_pos.y;
                let dz = f64::from(z) + 0.5 - entity_pos.z;
                let dist = dx * dx + dy * dy + dz * dz;
                let better = match best {
                    None => true,
                    Some((current, best_dist)) => {
                        dist < best_dist
                            || (dist == best_dist
                                && (current.y, current.z, current.x) < (y, z, x))
                    }
                };
                if better {
                    best = Some((pos, dist));
                }
            }
        }
    }
    best.map(|(pos, _)| pos)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

fn axis_bounds(bb: &Aabb, axis: Axis) -> (f64, f64) {
    match axis {
        Axis::X => (bb.min_x, bb.max_x),
        Axis::Y => (bb.min_y, bb.max_y),
        Axis::Z => (bb.min_z, bb.max_z),
    }
}

fn others(axis: Axis) -> [Axis; 2] {
    match axis {
        Axis::X => [Axis::Y, Axis::Z],
        Axis::Y => [Axis::Z, Axis::X],
        Axis::Z => [Axis::X, Axis::Y],
    }
}

/// `VoxelShape.collide(axis, moving, distance)` for one box: the distance
/// `moving` may travel along `axis` before touching `collider`.
fn collide_box(axis: Axis, moving: &Aabb, collider: &Aabb, mut distance: f64) -> f64 {
    if distance.abs() < COLLISION_EPSILON {
        return 0.0;
    }
    for other in others(axis) {
        let (m_min, m_max) = axis_bounds(moving, other);
        let (c_min, c_max) = axis_bounds(collider, other);
        if !(m_min + COLLISION_EPSILON < c_max && m_max - COLLISION_EPSILON > c_min) {
            return distance;
        }
    }
    let (m_min, m_max) = axis_bounds(moving, axis);
    let (c_min, c_max) = axis_bounds(collider, axis);
    if distance > 0.0 {
        let gap = c_min - m_max;
        if gap >= -COLLISION_EPSILON {
            distance = distance.min(gap);
        }
    } else {
        let gap = c_max - m_min;
        if gap <= COLLISION_EPSILON {
            distance = distance.max(gap);
        }
    }
    distance
}

/// `Shapes.collide(axis, moving, shapes, distance)`.
fn collide_axis(axis: Axis, moving: &Aabb, colliders: &[Aabb], mut distance: f64) -> f64 {
    for collider in colliders {
        if distance.abs() < COLLISION_EPSILON {
            return 0.0;
        }
        distance = collide_box(axis, moving, collider, distance);
    }
    distance
}

/// `Entity.collideWithShapes`: resolves the movement axis by axis in
/// `Direction.axisStepOrder` (Y first, then the larger horizontal axis).
fn collide_with_shapes(movement: Vec3, bb: &Aabb, colliders: &[Aabb]) -> Vec3 {
    if colliders.is_empty() {
        return movement;
    }
    let order = if movement.x.abs() < movement.z.abs() {
        [Axis::Y, Axis::Z, Axis::X]
    } else {
        [Axis::Y, Axis::X, Axis::Z]
    };
    let mut resolved = Vec3::ZERO;
    for axis in order {
        let wanted = match axis {
            Axis::X => movement.x,
            Axis::Y => movement.y,
            Axis::Z => movement.z,
        };
        if wanted != 0.0 {
            let moved = aabb_offset(*bb, resolved.x, resolved.y, resolved.z);
            let allowed = collide_axis(axis, &moved, colliders, wanted);
            match axis {
                Axis::X => resolved.x = allowed,
                Axis::Y => resolved.y = allowed,
                Axis::Z => resolved.z = allowed,
            }
        }
    }
    resolved
}

/// `Entity.collide(movement)` for an entity with `maxUpStep() == 0` and no
/// entity colliders: the movement clipped by the block collision shapes
/// (`Entity.collideBoundingBox`).
///
/// TODO(entity-colliders): `Level.getEntityCollisions` (boats, shulkers and
/// other `canBeCollidedWith` entities) and the world-border collision shape
/// are not modelled.
pub fn collide_bounding_box(world: &impl SurvivalWorld, bb: &Aabb, movement: Vec3) -> Vec3 {
    if movement.x * movement.x + movement.y * movement.y + movement.z * movement.z == 0.0 {
        return movement;
    }
    let colliders = collect_block_colliders(world, &aabb_expand_towards(*bb, movement));
    collide_with_shapes(movement, bb, &colliders)
}

/// `AABB.clip(Iterable<AABB>, from, to, pos)` reduced to hit / no hit: the
/// scale at which the segment first enters any of `boxes`, mirroring
/// `AABB.getDirection` / `clipPoint`.
fn clip_boxes_hit(boxes: &[Aabb], from: Vec3, to: Vec3) -> bool {
    let (dx, dy, dz) = (to.x - from.x, to.y - from.y, to.z - from.z);
    let mut scale = 1.0_f64;
    let mut hit = false;
    for b in boxes {
        let mut clip = |da: f64,
                        db: f64,
                        dc: f64,
                        point: f64,
                        (min_b, max_b): (f64, f64),
                        (min_c, max_c): (f64, f64),
                        (from_a, from_b, from_c): (f64, f64, f64)| {
            let s = (point - from_a) / da;
            let pb = from_b + s * db;
            let pc = from_c + s * dc;
            if 0.0 < s
                && s < scale
                && min_b - 1.0E-7 < pb
                && pb < max_b + 1.0E-7
                && min_c - 1.0E-7 < pc
                && pc < max_c + 1.0E-7
            {
                scale = s;
                hit = true;
            }
        };
        if dx > 1.0E-7 {
            clip(dx, dy, dz, b.min_x, (b.min_y, b.max_y), (b.min_z, b.max_z), (from.x, from.y, from.z));
        } else if dx < -1.0E-7 {
            clip(dx, dy, dz, b.max_x, (b.min_y, b.max_y), (b.min_z, b.max_z), (from.x, from.y, from.z));
        }
        if dy > 1.0E-7 {
            clip(dy, dz, dx, b.min_y, (b.min_z, b.max_z), (b.min_x, b.max_x), (from.y, from.z, from.x));
        } else if dy < -1.0E-7 {
            clip(dy, dz, dx, b.max_y, (b.min_z, b.max_z), (b.min_x, b.max_x), (from.y, from.z, from.x));
        }
        if dz > 1.0E-7 {
            clip(dz, dx, dy, b.min_z, (b.min_x, b.max_x), (b.min_y, b.max_y), (from.z, from.x, from.y));
        } else if dz < -1.0E-7 {
            clip(dz, dx, dy, b.max_z, (b.min_x, b.max_x), (b.min_y, b.max_y), (from.z, from.x, from.y));
        }
    }
    hit
}

/// `VoxelShape.clip(from, to, pos) != null` for the collision shape at `pos`.
fn shape_clip_hits(boxes: &[Aabb], from: Vec3, to: Vec3) -> bool {
    if boxes.is_empty() {
        return false;
    }
    let diff = Vec3 {
        x: to.x - from.x,
        y: to.y - from.y,
        z: to.z - from.z,
    };
    if diff.x * diff.x + diff.y * diff.y + diff.z * diff.z < 1.0E-7 {
        return false;
    }
    // `isFullWide(testPoint)`: the start of the segment is already inside.
    let test = Vec3 {
        x: from.x + diff.x * 0.001,
        y: from.y + diff.y * 0.001,
        z: from.z + diff.z * 0.001,
    };
    let inside = boxes.iter().any(|b| {
        b.min_x <= test.x
            && test.x < b.max_x
            && b.min_y <= test.y
            && test.y < b.max_y
            && b.min_z <= test.z
            && test.z < b.max_z
    });
    inside || clip_boxes_hit(boxes, from, to)
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

fn frac(v: f64) -> f64 {
    v - v.floor()
}

fn sign(v: f64) -> i32 {
    if v == 0.0 {
        0
    } else if v > 0.0 {
        1
    } else {
        -1
    }
}

/// `level.clip(new ClipContext(from, to, COLLIDER, NONE, entity)).getType()
/// != MISS`: walks the blocks along the segment exactly like
/// `BlockGetter.traverseBlocks` and tests each collision shape.
pub fn clip_hits_collider(world: &impl SurvivalWorld, from: Vec3, to: Vec3) -> bool {
    if from == to {
        return false;
    }
    let block_hits = |x: i32, y: i32, z: i32| {
        let pos = BlockPos { x, y, z };
        let state = world.state_at(pos);
        !state.is_air() && shape_clip_hits(&block_collision_boxes(&state, pos), from, to)
    };
    let to_x = lerp(-1.0E-7, to.x, from.x);
    let to_y = lerp(-1.0E-7, to.y, from.y);
    let to_z = lerp(-1.0E-7, to.z, from.z);
    let from_x = lerp(-1.0E-7, from.x, to.x);
    let from_y = lerp(-1.0E-7, from.y, to.y);
    let from_z = lerp(-1.0E-7, from.z, to.z);
    let (mut bx, mut by, mut bz) = (
        from_x.floor() as i32,
        from_y.floor() as i32,
        from_z.floor() as i32,
    );
    if block_hits(bx, by, bz) {
        return true;
    }
    let (dx, dy, dz) = (to_x - from_x, to_y - from_y, to_z - from_z);
    let (sx, sy, sz) = (sign(dx), sign(dy), sign(dz));
    let t_delta = |s: i32, d: f64| if s == 0 { f64::MAX } else { f64::from(s) / d };
    let (t_dx, t_dy, t_dz) = (t_delta(sx, dx), t_delta(sy, dy), t_delta(sz, dz));
    let start = |s: i32, t_d: f64, f: f64| t_d * if s > 0 { 1.0 - frac(f) } else { frac(f) };
    let (mut tx, mut ty, mut tz) = (
        start(sx, t_dx, from_x),
        start(sy, t_dy, from_y),
        start(sz, t_dz, from_z),
    );
    while tx <= 1.0 || ty <= 1.0 || tz <= 1.0 {
        if tx < ty {
            if tx < tz {
                bx += sx;
                tx += t_dx;
            } else {
                bz += sz;
                tz += t_dz;
            }
        } else if ty < tz {
            by += sy;
            ty += t_dy;
        } else {
            bz += sz;
            tz += t_dz;
        }
        if block_hits(bx, by, bz) {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub(crate) mod test_world;
#[cfg(test)]
mod tests;
