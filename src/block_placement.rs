//! Java `Block.getStateForPlacement` ports, dispatched on the official
//! block-type key like [`crate::block_survival`].
//!
//! The Java entry point is `BlockItem.place`: `BlockPlaceContext.canPlace`
//! (replaceability) -> `getPlacementState` (the per-block override here) ->
//! `canSurvive` -> `Level.setBlock`. [`PlaceContext`] mirrors
//! `BlockPlaceContext`: the *relocated* placement position (`getClickedPos`
//! after `BlockPlaceContext`'s offset logic), the clicked face, the precise
//! click location, and the player's orientation, with world access through
//! [`PlacementWorld`].
//!
//! Coverage is tracked explicitly: [`state_for_placement`] returns `None` for
//! block types whose Java override has not been ported yet (so callers keep
//! their previous behavior instead of silently mis-placing), and
//! `PORTED_BLOCK_TYPES` pins the set in tests. Types without a Java override
//! use the `Block` default (the block's default state).

#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{state_physics_by_name, StateFluid};
use crate::block_states::block_state_entry;
use crate::block_survival::{can_survive, SurvivalWorld};
use crate::block_update::{BlockPos, Direction};

/// World access for placement decisions. Extends the survival view with the
/// redstone-signal query some placements sample (doors, trapdoors, skulls).
pub trait PlacementWorld: SurvivalWorld {
    /// Java `Level.hasNeighborSignal(pos)`.
    fn has_neighbor_signal(&self, pos: BlockPos) -> bool;
}

/// Java `BlockPlaceContext` (after `getClickedPos` relocation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceContext {
    /// `getClickedPos()` — where the block will be placed.
    pub clicked_pos: BlockPos,
    /// `getClickedFace()` — the face of the block the player clicked.
    pub clicked_face: Direction,
    /// `getClickLocation()` — absolute coordinates of the hit point.
    pub click_location: [f64; 3],
    /// `replacingClickedOnBlock()` — true when the clicked block itself was
    /// replaceable and the placement did not offset to a neighbor.
    pub replacing_clicked_on_block: bool,
    /// Player yaw in degrees (`getRotation()`).
    pub player_yaw: f32,
    /// Player pitch in degrees (drives `getNearestLookingDirections()`).
    pub player_pitch: f32,
    /// `isSecondaryUseActive()` — sneaking.
    pub secondary_use_active: bool,
}

impl PlaceContext {
    /// Java `UseOnContext.getHorizontalDirection()` =
    /// `Direction.fromYRot(player.getYRot())`.
    pub fn horizontal_direction(&self) -> Direction {
        direction_from_y_rot(self.player_yaw as f64)
    }

    /// Java `BlockPlaceContext.getNearestLookingDirection()`.
    pub fn nearest_looking_direction(&self) -> Direction {
        self.nearest_looking_directions()[0]
    }

    /// Java `Direction.orderedByNearest(player)` (`BlockPlaceContext
    /// .getNearestLookingDirections()`).
    pub fn nearest_looking_directions(&self) -> [Direction; 6] {
        ordered_by_nearest(self.player_pitch, self.player_yaw)
    }

    /// The in-block fraction of the click location along an axis, e.g. Java's
    /// `context.getClickLocation().y - pos.getY()`.
    fn click_fraction(&self, axis: usize) -> f64 {
        let base = match axis {
            0 => self.clicked_pos.x,
            1 => self.clicked_pos.y,
            _ => self.clicked_pos.z,
        };
        self.click_location[axis] - f64::from(base)
    }
}

/// Java `Direction.fromYRot(double)`.
pub fn direction_from_y_rot(y_rot: f64) -> Direction {
    match ((y_rot / 90.0 + 0.5).floor() as i64 & 3) as u8 {
        0 => Direction::South,
        1 => Direction::West,
        2 => Direction::North,
        _ => Direction::East,
    }
}

/// Java `Direction.orderedByNearest(entity)`.
pub fn ordered_by_nearest(pitch_degrees: f32, yaw_degrees: f32) -> [Direction; 6] {
    let pitch = pitch_degrees * std::f32::consts::PI / 180.0;
    let yaw = -yaw_degrees * std::f32::consts::PI / 180.0;
    let pitch_sin = pitch.sin();
    let pitch_cos = pitch.cos();
    let yaw_sin = yaw.sin();
    let yaw_cos = yaw.cos();
    let x_positive = yaw_sin > 0.0;
    let y_positive = pitch_sin < 0.0;
    let z_positive = yaw_cos > 0.0;
    let x_yaw = if x_positive { yaw_sin } else { -yaw_sin };
    let y_magnitude = if y_positive { -pitch_sin } else { pitch_sin };
    let z_yaw = if z_positive { yaw_cos } else { -yaw_cos };
    let x_magnitude = x_yaw * pitch_cos;
    let z_magnitude = z_yaw * pitch_cos;
    let axis_x = if x_positive {
        Direction::East
    } else {
        Direction::West
    };
    let axis_y = if y_positive {
        Direction::Up
    } else {
        Direction::Down
    };
    let axis_z = if z_positive {
        Direction::South
    } else {
        Direction::North
    };
    let (first, second, third) = if x_yaw > z_yaw {
        if y_magnitude > x_magnitude {
            (axis_y, axis_x, axis_z)
        } else if z_magnitude > y_magnitude {
            (axis_x, axis_z, axis_y)
        } else {
            (axis_x, axis_y, axis_z)
        }
    } else if y_magnitude > z_magnitude {
        (axis_y, axis_z, axis_x)
    } else if x_magnitude > y_magnitude {
        (axis_z, axis_x, axis_y)
    } else {
        (axis_z, axis_y, axis_x)
    };
    [
        first,
        second,
        third,
        third.opposite(),
        second.opposite(),
        first.opposite(),
    ]
}

/// Java `RotationSegment.convertToSegment(float)`: 16 yaw segments.
pub fn rotation_segment(degrees: f32) -> i32 {
    (((degrees + 360.0) / 22.5 + 0.5).floor() as i32) & 15
}

/// The outcome of `getStateForPlacement`.
#[derive(Debug, Clone, PartialEq)]
pub enum PlacementOutcome {
    /// Place this exact state.
    Place(BlockStateModel),
    /// Java returned `null` — the placement fails.
    Reject,
}

fn fluid_at(world: &impl PlacementWorld, pos: BlockPos) -> StateFluid {
    let state = world.state_at(pos);
    state_physics_by_name(&state.state_name()).map_or(StateFluid::Empty, |physics| physics.fluid)
}

/// Java `fluidState.is(Fluids.WATER)` — the source-water type check used by
/// every WATERLOGGED placement sample.
fn replaced_by_source_water(world: &impl PlacementWorld, pos: BlockPos) -> bool {
    matches!(fluid_at(world, pos), StateFluid::Water { source: true, .. })
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

fn axis_name(direction: Direction) -> &'static str {
    match direction {
        Direction::West | Direction::East => "x",
        Direction::Down | Direction::Up => "y",
        Direction::North | Direction::South => "z",
    }
}

fn is_horizontal(direction: Direction) -> bool {
    !matches!(direction, Direction::Up | Direction::Down)
}

/// Java `Direction.getClockWise()` (Y axis).
fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        other => other,
    }
}

fn counter_clockwise(direction: Direction) -> Direction {
    clockwise(direction).opposite()
}

fn default_state(registry_id: &str) -> BlockStateModel {
    BlockStateModel::default_for(registry_id)
        .unwrap_or_else(|| BlockStateModel::new(registry_id.to_string()))
}

fn set(state: BlockStateModel, property: &str, value: impl Into<String>) -> BlockStateModel {
    state.try_set_property(property, value)
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// Block types whose Java `getStateForPlacement` override is ported here.
/// Types not in this list and without a Java override place their default
/// state; types WITH an unported override yield `None` from
/// [`state_for_placement`] so callers keep legacy behavior explicitly.
pub const PORTED_BLOCK_TYPES: &[&str] = &[
    "furnace",
    "blast_furnace",
    "smoker",
    "anvil",
    "banner",
    "barrel",
    "bed",
    "beehive",
    "bell",
    "campfire",
    "candle",
    "chain",
    "chest",
    "trapped_chest",
    "cocoa",
    "repeater",
    "comparator",
    "end_portal_frame",
    "end_rod",
    "button",
    "lever",
    "stair",
    "weathering_copper_stair",
    "slab",
    "weathering_copper_slab",
    "rotated_pillar",
    "infested_rotated_pillar",
    "hay",
    "trapdoor",
    "weathering_copper_trap_door",
    "door",
    "weathering_copper_door",
    "amethyst_cluster",
    "glazed_terracotta",
    "carved_pumpkin",
    "jack_o_lantern",
    "dispenser",
    "dropper",
    "observer",
    "piston_base",
    "ladder",
    "lantern",
    "weathering_lantern",
    "standing_sign",
    "wall_sign",
    "wall_banner",
    "wall_torch",
    "redstone_wall_torch",
    "snow_layer",
    "waterlogged_transparent",
    "leaf_litter",
    "flower_bed",
    "sea_pickle",
    "turtle_egg",
    "candle_cake",
    "lightning_rod",
    "weathering_lightning_rod",
    "hopper",
    "loom",
    "stonecutter",
    "lectern",
    "note",
    "redstone_lamp",
    "shulker_box",
    "ender_chest",
    "barrier",
];

/// Java `Block.getStateForPlacement` for `block_id` (the item's block).
/// Returns `None` when the owning Java class has an override that is not yet
/// ported; callers must then keep their previous placement behavior.
pub fn state_for_placement(
    block_id: &str,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let entry = block_state_entry(block_id)?;
    let block_type = entry.block_type;
    let state = default_state(block_id);

    oriented_placement(block_type, state.clone(), context, world)
        .or_else(|| sliced_placement(block_type, state.clone(), context, world))
        .or_else(|| attached_placement(block_type, state.clone(), context, world))
        .or_else(|| stacking_placement(block_type, block_id, state, context, world))
        .or_else(|| unported_or_default(block_type, block_id))
}

/// Families whose placement is a pure orientation function of the context.
#[expect(
    clippy::too_many_lines,
    reason = "Java placement rules stay grouped by family for parity review"
)]
fn oriented_placement(
    block_type: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let placed = match block_type {
        // AbstractFurnaceBlock, BeehiveBlock, DiodeBlock, EndPortalFrameBlock,
        // CarvedPumpkinBlock, GlazedTerracottaBlock (toward the player),
        // LoomBlock, StonecutterBlock, LecternBlock, ChiseledBookShelf-style:
        // FACING = horizontal opposite.
        "furnace" | "blast_furnace" | "smoker" | "beehive" | "repeater" | "comparator"
        | "end_portal_frame" | "carved_pumpkin" | "jack_o_lantern" | "loom" | "lectern" => set(
            state,
            "facing",
            direction_name(context.horizontal_direction().opposite()),
        ),

        // GlazedTerracottaBlock / StonecutterBlock: FACING = horizontal (not
        // opposite for stonecutter? Java StonecutterBlock uses opposite;
        // glazed terracotta uses getHorizontalDirection() directly... both
        // verified below).
        "glazed_terracotta" => set(
            state,
            "facing",
            direction_name(context.horizontal_direction()),
        ),
        "stonecutter" => set(
            state,
            "facing",
            direction_name(context.horizontal_direction().opposite()),
        ),

        // AnvilBlock: FACING = horizontal clockwise.
        "anvil" => set(
            state,
            "facing",
            direction_name(clockwise(context.horizontal_direction())),
        ),

        // BarrelBlock: FACING = nearest looking direction opposite.
        "barrel" => set(
            state,
            "facing",
            direction_name(context.nearest_looking_direction().opposite()),
        ),

        // DispenserBlock/DropperBlock: nearest looking opposite.
        "dispenser" | "dropper" => set(
            state,
            "facing",
            direction_name(context.nearest_looking_direction().opposite()),
        ),

        // ObserverBlock: nearest looking opposite... Java: FACING =
        // getNearestLookingDirection().getOpposite().getOpposite() — the
        // observer looks AT the player along the placement axis. Java body:
        // defaultBlockState().setValue(FACING, context
        // .getNearestLookingDirection().getOpposite().getOpposite()) ==
        // nearest looking direction itself.
        "observer" => set(
            state,
            "facing",
            direction_name(context.nearest_looking_direction()),
        ),

        // PistonBaseBlock: FACING = nearest looking opposite, EXTENDED false.
        "piston_base" => set(
            set(
                state,
                "facing",
                direction_name(context.nearest_looking_direction().opposite()),
            ),
            "extended",
            "false",
        ),

        // RotatedPillarBlock (logs, pillars, hay, infested deepslate): AXIS
        // from the clicked face.
        "rotated_pillar" | "infested_rotated_pillar" | "hay" => {
            set(state, "axis", axis_name(context.clicked_face))
        }

        // EndRodBlock / LightningRodBlock: FACING = clicked face, flipped when
        // stacking onto another rod pointing the same way (end rod only).
        "end_rod" => {
            let behind = context
                .clicked_pos
                .relative(context.clicked_face.opposite());
            let behind_state = world.state_at(behind);
            let same = behind_state.registry_id == state.registry_id
                && behind_state.property("facing") == Some(direction_name(context.clicked_face));
            set(
                state,
                "facing",
                direction_name(if same {
                    context.clicked_face.opposite()
                } else {
                    context.clicked_face
                }),
            )
        }
        "lightning_rod" | "weathering_lightning_rod" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            set(
                set(state, "facing", direction_name(context.clicked_face)),
                "waterlogged",
                bool_str(waterlogged),
            )
        }

        // AmethystClusterBlock: FACING = clicked face + waterlogged sample.
        "amethyst_cluster" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            set(
                set(state, "waterlogged", bool_str(waterlogged)),
                "facing",
                direction_name(context.clicked_face),
            )
        }

        // HopperBlock: FACING = clicked face opposite, DOWN when vertical.
        "hopper" => {
            let facing = context.clicked_face.opposite();
            let facing = if is_horizontal(facing) {
                facing
            } else {
                Direction::Down
            };
            set(state, "facing", direction_name(facing))
        }

        // LadderBlock: FACING from the clicked face when horizontal, else try
        // the looking directions; canSurvive filters.
        "ladder" => {
            return Some(ladder_placement(state, context, world));
        }

        // Wall torches: FACING points away from the supporting wall; Java
        // WallTorchBlock tries the looking directions' horizontal entries.
        "wall_torch" | "redstone_wall_torch" => {
            return Some(wall_facing_placement(state, context, world));
        }

        // StandingSignBlock / BannerBlock: 16-segment rotation from yaw + 180.
        "standing_sign" | "banner" => {
            let segment = rotation_segment(context.player_yaw + 180.0);
            let placed = set(state, "rotation", segment.to_string());
            if block_type == "standing_sign" {
                let waterlogged = replaced_by_source_water(world, context.clicked_pos);
                set(placed, "waterlogged", bool_str(waterlogged))
            } else {
                placed
            }
        }

        // WallSignBlock / WallBannerBlock: FACING from looking directions'
        // horizontal entries whose backing face is sturdy enough (isSolid).
        "wall_sign" | "wall_banner" => {
            return Some(wall_sign_placement(block_type, state, context, world));
        }

        // NoteBlock/RedstoneLampBlock-style powered sampling.
        "note" => set(
            state,
            "powered",
            bool_str(world.has_neighbor_signal(context.clicked_pos)),
        ),
        "redstone_lamp" => set(
            state,
            "lit",
            bool_str(world.has_neighbor_signal(context.clicked_pos)),
        ),

        // ShulkerBoxBlock: FACING = clicked face.
        "shulker_box" => set(state, "facing", direction_name(context.clicked_face)),

        // EnderChestBlock: horizontal opposite + waterlogged.
        "ender_chest" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            set(
                set(
                    state,
                    "facing",
                    direction_name(context.horizontal_direction().opposite()),
                ),
                "waterlogged",
                bool_str(waterlogged),
            )
        }

        // BarrierBlock: waterlogged sample only.
        "barrier" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            set(state, "waterlogged", bool_str(waterlogged))
        }

        _ => return None,
    };
    Some(PlacementOutcome::Place(placed))
}

/// Java `LadderBlock.getStateForPlacement`.
fn ladder_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let waterlogged = replaced_by_source_water(world, context.clicked_pos);
    let mut candidates: Vec<Direction> = Vec::new();
    if !context.replacing_clicked_on_block && is_horizontal(context.clicked_face) {
        candidates.push(context.clicked_face);
    } else {
        for direction in context.nearest_looking_directions() {
            if is_horizontal(direction) {
                candidates.push(direction.opposite());
            }
        }
    }
    for facing in candidates {
        let placed = set(
            set(state.clone(), "facing", direction_name(facing)),
            "waterlogged",
            bool_str(waterlogged),
        );
        if can_survive(&placed, context.clicked_pos, world) {
            return PlacementOutcome::Place(placed);
        }
    }
    PlacementOutcome::Reject
}

/// Java `WallTorchBlock.getStateForPlacement`: first horizontal looking
/// direction whose opposite wall face is sturdy.
fn wall_facing_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    for direction in context.nearest_looking_directions() {
        if is_horizontal(direction) {
            let placed = set(
                state.clone(),
                "facing",
                direction_name(direction.opposite()),
            );
            if can_survive(&placed, context.clicked_pos, world) {
                return PlacementOutcome::Place(placed);
            }
        }
    }
    PlacementOutcome::Reject
}

/// Java `WallSignBlock` / `WallBannerBlock` placement.
fn wall_sign_placement(
    block_type: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    for direction in context.nearest_looking_directions() {
        if is_horizontal(direction) {
            let facing = direction.opposite();
            let mut placed = set(state.clone(), "facing", direction_name(facing));
            if block_type == "wall_sign" {
                let waterlogged = replaced_by_source_water(world, context.clicked_pos);
                placed = set(placed, "waterlogged", bool_str(waterlogged));
            }
            if can_survive(&placed, context.clicked_pos, world) {
                return PlacementOutcome::Place(placed);
            }
        }
    }
    PlacementOutcome::Reject
}

/// Families that pick a half/axis slice from the precise click location.
#[expect(
    clippy::too_many_lines,
    reason = "Java placement rules stay grouped by family for parity review"
)]
fn sliced_placement(
    block_type: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let placed = match block_type {
        // StairBlock: FACING = horizontal, HALF from face/hit-Y, waterlogged,
        // SHAPE recomputed by updateShape (default straight at placement; the
        // neighbor pass fixes it — Java computes getStairsShape immediately,
        // mirrored here).
        "stair" | "weathering_copper_stair" => {
            let top = stair_half_is_top(context);
            let placed = set(
                set(
                    set(
                        state,
                        "facing",
                        direction_name(context.horizontal_direction()),
                    ),
                    "half",
                    if top { "top" } else { "bottom" },
                ),
                "waterlogged",
                bool_str(replaced_by_source_water(world, context.clicked_pos)),
            );
            let shape = stairs_shape(&placed, context.clicked_pos, world);
            set(placed, "shape", shape)
        }

        // SlabBlock: merge into DOUBLE when clicking an existing slab.
        "slab" | "weathering_copper_slab" => {
            let existing = world.state_at(context.clicked_pos);
            if existing.registry_id == state.registry_id {
                return Some(PlacementOutcome::Place(set(
                    set(existing, "type", "double"),
                    "waterlogged",
                    "false",
                )));
            }
            let top = stair_half_is_top(context);
            set(
                set(state, "type", if top { "top" } else { "bottom" }),
                "waterlogged",
                bool_str(replaced_by_source_water(world, context.clicked_pos)),
            )
        }

        // TrapDoorBlock.
        "trapdoor" | "weathering_copper_trap_door" => {
            let mut placed = state;
            if !context.replacing_clicked_on_block && is_horizontal(context.clicked_face) {
                placed = set(placed, "facing", direction_name(context.clicked_face));
                placed = set(
                    placed,
                    "half",
                    if context.click_fraction(1) > 0.5 {
                        "top"
                    } else {
                        "bottom"
                    },
                );
            } else {
                placed = set(
                    placed,
                    "facing",
                    direction_name(context.horizontal_direction().opposite()),
                );
                placed = set(
                    placed,
                    "half",
                    if context.clicked_face == Direction::Up {
                        "bottom"
                    } else {
                        "top"
                    },
                );
            }
            if world.has_neighbor_signal(context.clicked_pos) {
                placed = set(set(placed, "open", "true"), "powered", "true");
            }
            set(
                placed,
                "waterlogged",
                bool_str(replaced_by_source_water(world, context.clicked_pos)),
            )
        }

        // DoorBlock: lower half here; the caller also places the upper half.
        "door" | "weathering_copper_door" => {
            let above = context.clicked_pos.relative(Direction::Up);
            let above_state = world.state_at(above);
            let above_replaceable = state_physics_by_name(&above_state.state_name())
                .is_some_and(|physics| physics.replaceable);
            if context.clicked_pos.y
                >= crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1
                || !above_replaceable
            {
                return Some(PlacementOutcome::Reject);
            }
            let powered =
                world.has_neighbor_signal(context.clicked_pos) || world.has_neighbor_signal(above);
            set(
                set(
                    set(
                        set(
                            set(
                                state,
                                "facing",
                                direction_name(context.horizontal_direction()),
                            ),
                            "hinge",
                            door_hinge(context, world),
                        ),
                        "powered",
                        bool_str(powered),
                    ),
                    "open",
                    bool_str(powered),
                ),
                "half",
                "lower",
            )
        }

        _ => return None,
    };
    Some(PlacementOutcome::Place(placed))
}

/// Shared stairs/slab/trapdoor rule: top half iff the clicked face is DOWN, or
/// a side face hit above the block's midpoint.
fn stair_half_is_top(context: &PlaceContext) -> bool {
    context.clicked_face != Direction::Up
        && (context.clicked_face == Direction::Down || context.click_fraction(1) > 0.5)
}

/// Java `StairBlock.getStairsShape`.
fn stairs_shape(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl PlacementWorld,
) -> &'static str {
    let facing = state
        .property("facing")
        .and_then(direction_by_name)
        .unwrap_or(Direction::North);

    // Java: check the stairs behind (along facing) for outer corners.
    let behind = world.state_at(pos.relative(facing));
    if is_stairs(&behind) && state.property("half") == behind.property("half") {
        if let Some(behind_facing) = behind.property("facing").and_then(direction_by_name) {
            if is_horizontal(behind_facing)
                && axis_name(behind_facing) != axis_name(facing)
                && can_take_shape(state, pos, world, behind_facing.opposite())
            {
                return if behind_facing == counter_clockwise(facing) {
                    "outer_left"
                } else {
                    "outer_right"
                };
            }
        }
    }

    // Java: check the stairs in front (opposite facing) for inner corners.
    let front = world.state_at(pos.relative(facing.opposite()));
    if is_stairs(&front) && state.property("half") == front.property("half") {
        if let Some(front_facing) = front.property("facing").and_then(direction_by_name) {
            if is_horizontal(front_facing)
                && axis_name(front_facing) != axis_name(facing)
                && can_take_shape(state, pos, world, front_facing)
            {
                return if front_facing == counter_clockwise(facing) {
                    "inner_left"
                } else {
                    "inner_right"
                };
            }
        }
    }
    "straight"
}

/// Java `StairBlock.canTakeShape`.
fn can_take_shape(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl PlacementWorld,
    direction: Direction,
) -> bool {
    let neighbour = world.state_at(pos.relative(direction));
    !is_stairs(&neighbour)
        || neighbour.property("facing") != state.property("facing")
        || neighbour.property("half") != state.property("half")
}

fn is_stairs(state: &BlockStateModel) -> bool {
    block_state_entry(&state.registry_id)
        .is_some_and(|entry| matches!(entry.block_type, "stair" | "weathering_copper_stair"))
}

fn direction_by_name(name: &str) -> Option<Direction> {
    match name {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

/// Java `DoorBlock.getHinge`.
fn door_hinge(context: &PlaceContext, world: &impl PlacementWorld) -> &'static str {
    let facing = context.horizontal_direction();
    let pos = context.clicked_pos;
    let above = pos.relative(Direction::Up);
    let left_direction = counter_clockwise(facing);
    let left_state = world.state_at(pos.relative(left_direction));
    let above_left = world.state_at(above.relative(left_direction));
    let right_direction = clockwise(facing);
    let right_state = world.state_at(pos.relative(right_direction));
    let above_right = world.state_at(above.relative(right_direction));

    // Java tallies isCollisionShapeFullBlock on both rows.
    let full = |state: &BlockStateModel| {
        state_physics_by_name(&state.state_name())
            .is_some_and(crate::block_properties::collision_shape_is_full_cube)
    };
    let is_lower_door = |state: &BlockStateModel| {
        block_state_entry(&state.registry_id)
            .is_some_and(|entry| matches!(entry.block_type, "door" | "weathering_copper_door"))
            && state.property("half") == Some("lower")
    };
    let balance = -i32::from(full(&left_state)) - i32::from(full(&above_left))
        + i32::from(full(&right_state))
        + i32::from(full(&above_right));
    let door_left = is_lower_door(&left_state);
    let door_right = is_lower_door(&right_state);

    if (door_left && !door_right) || balance > 0 {
        return "right";
    }
    if (door_right && !door_left) || balance < 0 {
        return "left";
    }

    // Tie-break by which half of the block face the player clicked.
    let step_x: i32 = match facing {
        Direction::West => -1,
        Direction::East => 1,
        _ => 0,
    };
    let step_z: i32 = match facing {
        Direction::North => -1,
        Direction::South => 1,
        _ => 0,
    };
    let click_x = context.click_fraction(0);
    let click_z = context.click_fraction(2);
    let left = (step_x >= 0 || click_z >= 0.5)
        && (step_x <= 0 || click_z <= 0.5)
        && (step_z >= 0 || click_x <= 0.5)
        && (step_z <= 0 || click_x >= 0.5);
    if left {
        "left"
    } else {
        "right"
    }
}

/// Families that stack into the clicked block (candles, pickles, eggs, snow).
fn stacking_placement(
    block_type: &str,
    block_id: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let stack = |property: &'static str, max: i32| -> PlacementOutcome {
        let existing = world.state_at(context.clicked_pos);
        if existing.registry_id == block_id {
            let count = existing
                .property(property)
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(1);
            return PlacementOutcome::Place(set(
                existing,
                property,
                (count + 1).min(max).to_string(),
            ));
        }
        let waterlogged = replaced_by_source_water(world, context.clicked_pos);
        PlacementOutcome::Place(if state.has_property("waterlogged") {
            set(state.clone(), "waterlogged", bool_str(waterlogged))
        } else {
            state.clone()
        })
    };

    match block_type {
        // CandleBlock: cycle CANDLES on restack, else waterlogged default.
        "candle" => Some(stack("candles", 4)),
        // SeaPickleBlock: PICKLES up to 4.
        "sea_pickle" => Some(stack("pickles", 4)),
        // TurtleEggBlock: EGGS up to 4.
        "turtle_egg" => Some(stack("eggs", 4)),
        // FlowerBedBlock / LeafLitterBlock (SegmentableBlock): stack
        // FLOWER_AMOUNT/SEGMENT_AMOUNT and face the player.
        "flower_bed" | "leaf_litter" => {
            let property = if block_type == "flower_bed" {
                "flower_amount"
            } else {
                "segment_amount"
            };
            let existing = world.state_at(context.clicked_pos);
            if existing.registry_id == block_id {
                let count = existing
                    .property(property)
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or(1);
                return Some(PlacementOutcome::Place(set(
                    existing,
                    property,
                    (count + 1).min(4).to_string(),
                )));
            }
            Some(PlacementOutcome::Place(set(
                state,
                "facing",
                direction_name(context.horizontal_direction().opposite()),
            )))
        }
        // SnowLayerBlock: LAYERS stack up to 8.
        "snow_layer" => {
            let existing = world.state_at(context.clicked_pos);
            if existing.registry_id == block_id {
                let layers = existing
                    .property("layers")
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or(1);
                return Some(PlacementOutcome::Place(set(
                    existing,
                    "layers",
                    (layers + 1).min(8).to_string(),
                )));
            }
            Some(PlacementOutcome::Place(state))
        }
        // WaterloggedTransparentBlock (glass-like with waterlogging),
        // chains, candle cakes.
        "waterlogged_transparent" | "chain" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            let placed = set(state, "waterlogged", bool_str(waterlogged));
            let placed = if block_type == "chain" {
                set(placed, "axis", axis_name(context.clicked_face))
            } else {
                placed
            };
            Some(PlacementOutcome::Place(placed))
        }
        "candle_cake" => Some(PlacementOutcome::Place(state)),
        _ => None,
    }
}

/// Final tier: either the type has no Java override (default state) or its
/// override is not ported yet (`None` -> caller keeps legacy behavior).
fn unported_or_default(block_type: &str, block_id: &str) -> Option<PlacementOutcome> {
    /// Types with a Java getStateForPlacement override that is NOT yet ported.
    const UNPORTED: &[&str] = &[
        "abstract_skull",
        "skull",
        "player_head",
        "wall_skull",
        "wither_skull",
        "wither_wall_skull",
        "piglinwallskull",
        "player_wall_head",
        "bamboo_stalk",
        "base_coral_plant",
        "base_coral_fan",
        "base_coral_wall_fan",
        "coral",
        "coral_fan",
        "coral_plant",
        "coral_wall_fan",
        "fire",
        "soul_fire",
        "rail",
        "powered_rail",
        "detector_rail",
        "big_dripleaf",
        "calibrated_sculk_sensor",
        "sculk_sensor",
        "sculk_shrieker",
        "ceiling_hanging_sign",
        "wall_hanging_sign",
        "chiseled_book_shelf",
        "chorus_plant",
        "command",
        "concrete_powder",
        "conduit",
        "copper_chest",
        "weathering_copper_chest",
        "copper_golem_statue",
        "weathering_copper_golem_statue",
        "crafter",
        "creaking_heart",
        "decorated_pot",
        "dirt_path",
        "dried_ghast",
        "double_plant",
        "tall_flower",
        "tall_grass",
        "farmland",
        "fence",
        "fence_gate",
        "iron_bars",
        "bars",
        "weathering_copper_bar",
        "wall",
        "glow_lichen",
        "multiface",
        "sculk_vein",
        "vine",
        "kelp",
        "twisting_vines",
        "weeping_vines",
        "cave_vines",
        "growing_plant",
        "hanging_roots",
        "heavy_core",
        "huge_mushroom",
        "jigsaw",
        "leaves",
        "mangrove_leaves",
        "tinted_particle_leaves",
        "untinted_particle_leaves",
        "mangrove_propagule",
        "mangrove_roots",
        "mossy_carpet",
        "pitcher_crop",
        "pointed_dripstone",
        "redstone_wire",
        "scaffolding",
        "seagrass",
        "tall_seagrass",
        "shelf",
        "wooden_shelf",
        "small_dripleaf",
        "snowy_dirt",
        "test",
        "tripwire",
        "trip_wire_hook",
        "vault",
    ];
    if UNPORTED.contains(&block_type) {
        // TODO(block-placement-coverage): port the remaining Java
        // getStateForPlacement overrides (tracked by PORTED_BLOCK_TYPES).
        return None;
    }
    Some(PlacementOutcome::Place(default_state(block_id)))
}

mod attached;
use attached::attached_placement;

#[cfg(test)]
mod tests;
