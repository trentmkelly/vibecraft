//! Entity-versus-fluid interaction: Java `EntityFluidInteraction` and
//! `FlowingFluid.getFlow`.
//!
//! Tracks how deep an entity's bounding box is submerged in water and lava
//! and accumulates the fluid currents that push it
//! (`Entity.updateFluidInteraction`).

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{is_face_sturdy, state_physics_by_name, StateFluid, SupportType};
use crate::block_survival::SurvivalWorld;
use crate::block_update::{BlockPos, Direction};
use crate::collision_shape::Aabb;
use crate::network::play::Vec3;

/// `Entity.updateFluidInteraction` water current scale (`applyCurrentTo(WATER, this, 0.014)`).
pub const WATER_CURRENT_SCALE: f64 = 0.014;
/// Lava current scale without `EnvironmentAttributes.FAST_LAVA`
/// (`0.0023333333333333335`).
pub const LAVA_CURRENT_SCALE: f64 = 0.007 / 3.0;

/// The two fluid tags an entity tracks (`FluidTags.WATER` / `FluidTags.LAVA`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FluidTag {
    Water,
    Lava,
}

/// `BlockState.getFluidState()` reduced to what the entity interaction reads.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FluidHere {
    tag: FluidTag,
    /// `FluidState.getAmount()` (8 for a source).
    amount: u8,
    /// `FlowingFluid.FALLING`.
    falling: bool,
}

impl FluidHere {
    /// `FluidState.getOwnHeight()`: `amount / 9`.
    fn own_height(self) -> f32 {
        f32::from(self.amount) / 9.0
    }
}

fn fluid_of(state: &BlockStateModel) -> Option<FluidHere> {
    let physics = state_physics_by_name(&state.state_name())?;
    let falling = state
        .property("level")
        .and_then(|level| level.parse::<u8>().ok())
        .is_some_and(|level| level >= 8);
    match physics.fluid {
        StateFluid::Empty => None,
        StateFluid::Water { amount, .. } => Some(FluidHere {
            tag: FluidTag::Water,
            amount,
            falling,
        }),
        StateFluid::Lava { amount, .. } => Some(FluidHere {
            tag: FluidTag::Lava,
            amount,
            falling,
        }),
    }
}

fn fluid_at(world: &impl SurvivalWorld, pos: BlockPos) -> Option<FluidHere> {
    fluid_of(&world.state_at(pos))
}

/// `FluidState.getHeight(level, pos)`: a full block when the same fluid sits
/// above, else the own height.
fn fluid_height_at(world: &impl SurvivalWorld, pos: BlockPos, fluid: FluidHere) -> f32 {
    let above = fluid_at(world, pos.relative(Direction::Up));
    if above.is_some_and(|above| above.tag == fluid.tag) {
        1.0
    } else {
        fluid.own_height()
    }
}

/// `FlowingFluid.isSolidFace`.
fn is_solid_face(
    world: &impl SurvivalWorld,
    pos: BlockPos,
    direction: Direction,
    tag: FluidTag,
) -> bool {
    let state = world.state_at(pos);
    if fluid_of(&state).is_some_and(|fluid| fluid.tag == tag) {
        return false;
    }
    if direction == Direction::Up {
        return true;
    }
    if matches!(state.registry_id.as_str(), "minecraft:ice" | "minecraft:frosted_ice") {
        return false;
    }
    state_physics_by_name(&state.state_name())
        .is_some_and(|physics| is_face_sturdy(physics, direction_ordinal(direction), SupportType::Full))
}

/// Java `Direction` ordinal (DOWN, UP, NORTH, SOUTH, WEST, EAST).
fn direction_ordinal(direction: Direction) -> usize {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

const HORIZONTAL: [(Direction, i32, i32); 4] = [
    (Direction::North, 0, -1),
    (Direction::East, 1, 0),
    (Direction::South, 0, 1),
    (Direction::West, -1, 0),
];

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

/// `FlowingFluid.getFlow(level, pos, fluidState)` for the fluid at `pos`.
fn flow_at(world: &impl SurvivalWorld, pos: BlockPos, fluid: FluidHere) -> Vec3 {
    let affects_flow = |neighbour: Option<FluidHere>| neighbour.is_none_or(|n| n.tag == fluid.tag);
    let (mut flow_x, mut flow_z) = (0.0_f64, 0.0_f64);
    for (direction, step_x, step_z) in HORIZONTAL {
        let neighbour_pos = pos.relative(direction);
        let neighbour = fluid_at(world, neighbour_pos);
        if !affects_flow(neighbour) {
            continue;
        }
        let mut neighbour_height = neighbour.map_or(0.0, FluidHere::own_height);
        let mut distance = 0.0_f32;
        if neighbour_height == 0.0 {
            let blocks_motion = state_physics_by_name(&world.state_at(neighbour_pos).state_name())
                .is_some_and(|physics| physics.blocks_motion);
            if !blocks_motion {
                let below = fluid_at(world, neighbour_pos.relative(Direction::Down));
                if affects_flow(below) {
                    neighbour_height = below.map_or(0.0, FluidHere::own_height);
                    if neighbour_height > 0.0 {
                        distance = fluid.own_height() - (neighbour_height - 0.888_888_9);
                    }
                }
            }
        } else if neighbour_height > 0.0 {
            distance = fluid.own_height() - neighbour_height;
        }
        if distance != 0.0 {
            flow_x += f64::from(step_x) * f64::from(distance);
            flow_z += f64::from(step_z) * f64::from(distance);
        }
    }
    let mut flow = Vec3 {
        x: flow_x,
        y: 0.0,
        z: flow_z,
    };
    if fluid.falling {
        for (direction, _, _) in HORIZONTAL {
            let neighbour_pos = pos.relative(direction);
            if is_solid_face(world, neighbour_pos, direction, fluid.tag)
                || is_solid_face(world, neighbour_pos.relative(Direction::Up), direction, fluid.tag)
            {
                let n = normalize(flow);
                flow = Vec3 {
                    x: n.x,
                    y: n.y - 6.0,
                    z: n.z,
                };
                break;
            }
        }
    }
    normalize(flow)
}

/// `EntityFluidInteraction.Tracker`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidTracker {
    /// Depth of the box's bottom inside the fluid.
    pub height: f64,
    pub eyes_inside: bool,
    accumulated_current: Vec3,
    current_count: u32,
}

impl Default for FluidTracker {
    fn default() -> Self {
        Self {
            height: 0.0,
            eyes_inside: false,
            accumulated_current: Vec3::ZERO,
            current_count: 0,
        }
    }
}

impl FluidTracker {
    /// `EntityFluidInteraction.isInFluid`.
    pub fn in_fluid(&self) -> bool {
        self.height > 0.0
    }

    /// `Tracker.applyCurrentTo(entity, scale)` for a non-player entity: the
    /// motion impulse to add to `delta_movement`, or `None` when there is no
    /// current.
    pub fn current_impulse(&self, delta_movement: Vec3, scale: f64) -> Option<Vec3> {
        let a = self.accumulated_current;
        if self.current_count == 0 || a.x * a.x + a.y * a.y + a.z * a.z < f64::from(1.0E-5_f32) {
            return None;
        }
        let n = normalize(a);
        let mut impulse = Vec3 {
            x: n.x * scale,
            y: n.y * scale,
            z: n.z * scale,
        };
        let length = (impulse.x * impulse.x + impulse.y * impulse.y + impulse.z * impulse.z).sqrt();
        if delta_movement.x.abs() < 0.003 && delta_movement.z.abs() < 0.003 && length < 0.0045 {
            let n = normalize(impulse);
            impulse = Vec3 {
                x: n.x * 0.0045,
                y: n.y * 0.0045,
                z: n.z * 0.0045,
            };
        }
        Some(impulse)
    }
}

/// Both trackers after one `EntityFluidInteraction.update`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FluidInteraction {
    pub water: FluidTracker,
    pub lava: FluidTracker,
}

/// `EntityFluidInteraction.update(entity, ignoreCurrent)` for an entity with
/// bounding box `bb`, block column `(block_x, block_z)` and eye height `eye_y`.
pub fn update_fluid_interaction(
    world: &impl SurvivalWorld,
    bb: &Aabb,
    (block_x, block_z): (i32, i32),
    eye_y: f64,
    ignore_current: bool,
) -> FluidInteraction {
    let mut interaction = FluidInteraction::default();
    // `getFluidInteractionBox`: the bounding box deflated by 0.001.
    let (min_x, min_y, min_z) = (bb.min_x + 0.001, bb.min_y + 0.001, bb.min_z + 0.001);
    let (max_x, max_y, max_z) = (bb.max_x - 0.001, bb.max_y - 0.001, bb.max_z - 0.001);
    let (x0, y0, z0) = (min_x.floor() as i32, min_y.floor() as i32, min_z.floor() as i32);
    let (x1, y1, z1) = (
        max_x.ceil() as i32 - 1,
        max_y.ceil() as i32 - 1,
        max_z.ceil() as i32 - 1,
    );
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let pos = BlockPos { x, y, z };
                let Some(fluid) = fluid_at(world, pos) else {
                    continue;
                };
                let fluid_bottom = f64::from(y);
                let fluid_top = fluid_bottom + f64::from(fluid_height_at(world, pos, fluid));
                if fluid_top < min_y {
                    continue;
                }
                let tracker = match fluid.tag {
                    FluidTag::Water => &mut interaction.water,
                    FluidTag::Lava => &mut interaction.lava,
                };
                if x == block_x && z == block_z && eye_y >= fluid_bottom && eye_y <= fluid_top {
                    tracker.eyes_inside = true;
                }
                tracker.height = tracker.height.max(fluid_top - bb.min_y);
                if !ignore_current {
                    let mut flow = flow_at(world, pos, fluid);
                    if tracker.height < 0.4 {
                        flow = Vec3 {
                            x: flow.x * tracker.height,
                            y: flow.y * tracker.height,
                            z: flow.z * tracker.height,
                        };
                    }
                    tracker.accumulated_current = Vec3 {
                        x: tracker.accumulated_current.x + flow.x,
                        y: tracker.accumulated_current.y + flow.y,
                        z: tracker.accumulated_current.z + flow.z,
                    };
                    tracker.current_count += 1;
                }
            }
        }
    }
    interaction
}

#[cfg(test)]
mod tests;
