//! Per-block `canSurvive` rules, ported 1:1 from every Java
//! `BlockBehaviour.canSurvive` override in 26.1.2 (the `BlockBehaviour` default
//! is `true`).
//!
//! Dispatch is on the official block-type key
//! ([`crate::block_states::BlockStateEntryData::block_type`]), so every block
//! routes to the rule of the Java class that actually owns it. Support
//! primitives come from the probed tables: [`crate::block_properties::is_face_sturdy`]
//! covers `isFaceSturdy`/`canSupportCenter`/`canSupportRigidBlock`, voxel-shape
//! face coverage is computed from the interned shapes, and the data-driven
//! `supports_*` tags resolve through [`crate::block_tags`].
//!
//! World access goes through [`SurvivalWorld`] so the rules are usable from the
//! live block-update pipeline, worldgen, and tests alike. Positions are absolute
//! world coordinates.

#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{
    is_face_sturdy, shape, state_physics_by_name, StateFluid, StatePhysics, SupportType,
};
use crate::block_states::block_state_entry;
use crate::block_tags::block_tag_contains;
use crate::block_update::{BlockPos, Direction};

/// World access needed by survival checks.
pub trait SurvivalWorld {
    /// The block state at `pos`; unloaded positions return air like Java's
    /// empty-chunk fallbacks during neighbor updates.
    fn state_at(&self, pos: BlockPos) -> BlockStateModel;

    /// Java `LevelReader.getRawBrightness(pos, 0)` — max of sky light (with no
    /// darkening) and block light. Used by crops, pitcher crops, and mushrooms.
    fn raw_brightness(&self, pos: BlockPos) -> i32;
}

/// Java `Direction.ordinal()` for the lighting/support tables (DOWN, UP, NORTH,
/// SOUTH, WEST, EAST); `block_update::Direction` declares its variants in
/// neighbor-update order instead.
fn java_ordinal(direction: Direction) -> usize {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

fn physics(state: &BlockStateModel) -> Option<&'static StatePhysics> {
    state_physics_by_name(&state.state_name())
}

fn is_tag(state: &BlockStateModel, tag: &str) -> bool {
    block_tag_contains(tag, &state.registry_id)
}

fn is_solid(state: &BlockStateModel) -> bool {
    physics(state).is_some_and(|physics| physics.is_solid)
}

fn fluid(state: &BlockStateModel) -> StateFluid {
    physics(state).map_or(StateFluid::Empty, |physics| physics.fluid)
}

/// Java `BlockState.isFaceSturdy(level, pos, direction)` (SupportType.FULL).
fn face_sturdy(state: &BlockStateModel, direction: Direction) -> bool {
    face_sturdy_typed(state, direction, SupportType::Full)
}

fn face_sturdy_typed(
    state: &BlockStateModel,
    direction: Direction,
    support_type: SupportType,
) -> bool {
    physics(state)
        .is_some_and(|physics| is_face_sturdy(physics, java_ordinal(direction), support_type))
}

/// Java `Block.canSupportCenter(level, pos, direction)`: the
/// `unstable_bottom_center` carve-out only applies to DOWN faces.
fn can_support_center(state: &BlockStateModel, direction: Direction) -> bool {
    if direction == Direction::Down && is_tag(state, "unstable_bottom_center") {
        return false;
    }
    face_sturdy_typed(state, direction, SupportType::Center)
}

/// Java `Block.canSupportRigidBlock(level, below)`.
fn can_support_rigid_block(below: &BlockStateModel) -> bool {
    face_sturdy_typed(below, Direction::Up, SupportType::Rigid)
}

/// Java `Block.isFaceFull(shape, direction)` over interned shape boxes: the
/// boxes touching the face plane must cover the full unit face. Exact for
/// axis-aligned box unions via 1D strip sweep over compressed coordinates.
fn shape_face_full(shape_index: u16, direction: Direction) -> bool {
    face_rectangles(shape_index, direction)
        .is_some_and(|rectangles| covers_unit_square(&rectangles))
}

/// Whether the face slice of a shape is non-empty (used by SeaPickleBlock).
fn shape_face_nonempty(shape_index: u16, direction: Direction) -> bool {
    face_rectangles(shape_index, direction).is_some_and(|rectangles| !rectangles.is_empty())
}

/// 2D rectangles `[min_a, min_b, max_a, max_b]` of the boxes touching the face
/// plane of `direction`, or `None` for an empty shape.
fn face_rectangles(shape_index: u16, direction: Direction) -> Option<Vec<[f64; 4]>> {
    let boxes = shape(shape_index);
    if boxes.is_empty() {
        return None;
    }
    // Axis index in [min_x, min_y, min_z, max_x, max_y, max_z] and the face
    // plane coordinate on that axis.
    let (axis, plane) = match direction {
        Direction::Down => (1, 0.0),
        Direction::Up => (1, 1.0),
        Direction::North => (2, 0.0),
        Direction::South => (2, 1.0),
        Direction::West => (0, 0.0),
        Direction::East => (0, 1.0),
    };
    let coordinate = |aabb: &[f64; 6]| {
        if plane == 0.0 {
            aabb[axis]
        } else {
            aabb[axis + 3]
        }
    };
    let (a_axis, b_axis) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    const EPSILON: f64 = 1.0e-7;
    Some(
        boxes
            .iter()
            .filter(|aabb| (coordinate(aabb) - plane).abs() < EPSILON)
            .map(|aabb| {
                [
                    aabb[a_axis],
                    aabb[b_axis],
                    aabb[a_axis + 3],
                    aabb[b_axis + 3],
                ]
            })
            .collect(),
    )
}

/// Whether a union of axis-aligned rectangles covers `[0,1] x [0,1]`, allowing
/// the same 1e-7 fuzz Java's `DoubleMath.fuzzyEquals` joins use.
fn covers_unit_square(rectangles: &[[f64; 4]]) -> bool {
    const EPSILON: f64 = 1.0e-7;
    let mut xs: Vec<f64> = vec![0.0, 1.0];
    for rectangle in rectangles {
        xs.push(rectangle[0].clamp(0.0, 1.0));
        xs.push(rectangle[2].clamp(0.0, 1.0));
    }
    xs.sort_by(|left, right| left.total_cmp(right));
    xs.dedup();
    for window in xs.windows(2) {
        let (start, end) = (window[0], window[1]);
        if end - start < EPSILON {
            continue;
        }
        let middle = (start + end) / 2.0;
        // Collect the b-axis intervals of rectangles spanning this strip and
        // check they cover [0, 1].
        let mut intervals: Vec<(f64, f64)> = rectangles
            .iter()
            .filter(|rectangle| rectangle[0] <= middle && middle <= rectangle[2])
            .map(|rectangle| (rectangle[1], rectangle[3]))
            .collect();
        intervals.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut covered = 0.0_f64;
        for (low, high) in intervals {
            if low > covered + EPSILON {
                return false;
            }
            covered = covered.max(high);
        }
        if covered < 1.0 - EPSILON {
            return false;
        }
    }
    true
}

/// Java `MultifaceBlock.canAttachTo(level, direction, neighbourPos, state)`:
/// the neighbour's support shape OR collision shape must present a full face
/// back toward the attaching block.
pub(crate) fn multiface_can_attach_to(
    neighbour: &BlockStateModel,
    direction_towards: Direction,
) -> bool {
    let Some(neighbour_physics) = physics(neighbour) else {
        return false;
    };
    let back = direction_towards.opposite();
    shape_face_full(neighbour_physics.support_shape, back)
        || shape_face_full(neighbour_physics.collision_shape, back)
}

/// Java `FaceAttachedHorizontalDirectionalBlock.canAttach(level, pos, direction)`.
fn face_attached_can_attach(
    world: &impl SurvivalWorld,
    pos: BlockPos,
    direction: Direction,
) -> bool {
    let relative = pos.relative(direction);
    face_sturdy(&world.state_at(relative), direction.opposite())
}

fn facing_property(state: &BlockStateModel) -> Option<Direction> {
    match state.property("facing")? {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

/// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection`: the
/// direction the block points away from its supporting face.
fn face_attached_connected_direction(state: &BlockStateModel) -> Direction {
    match state.property("face") {
        Some("ceiling") => Direction::Down,
        Some("floor") => Direction::Up,
        _ => facing_property(state).unwrap_or(Direction::North),
    }
}

/// Java `BlockState.canSurvive(level, pos)`, dispatched on the owning block
/// type. Unknown blocks (non-vanilla ids) survive like the Java default.
pub fn can_survive(state: &BlockStateModel, pos: BlockPos, world: &impl SurvivalWorld) -> bool {
    let Some(entry) = block_state_entry(&state.registry_id) else {
        return true;
    };
    let block_type = entry.block_type;
    attachment_can_survive(block_type, state, pos, world)
        .or_else(|| plant_can_survive(block_type, state, pos, world))
        .or_else(|| misc_can_survive(block_type, state, pos, world))
        .unwrap_or(true)
}

/// Support/attachment canSurvive families (torches, rails, signs, plates, ...).
/// One arm per Java class; length is inherent to the catalog.
#[allow(clippy::too_many_lines)]
fn attachment_can_survive(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> Option<bool> {
    let below = || world.state_at(pos.relative(Direction::Down));
    Some(match block_type {
        // BaseTorchBlock + CandleBlock: canSupportCenter(below, UP).
        "torch" | "redstone_torch" | "candle" => can_support_center(&below(), Direction::Up),

        // WallTorchBlock / RedstoneWallTorchBlock: facing-back face sturdy.
        "wall_torch" | "redstone_wall_torch" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            face_sturdy(&world.state_at(pos.relative(facing.opposite())), facing)
        }

        // AmethystClusterBlock: attached along FACING.
        "amethyst_cluster" => {
            let facing = facing_property(state).unwrap_or(Direction::Up);
            face_sturdy(&world.state_at(pos.relative(facing.opposite())), facing)
        }

        // Standing banners/signs, cake, candle cake: below isSolid.
        "banner" | "standing_sign" | "cake" | "candle_cake" => is_solid(&below()),

        // Wall banners/signs: block behind FACING isSolid.
        "wall_banner" | "wall_sign" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            is_solid(&world.state_at(pos.relative(facing.opposite())))
        }

        // BaseCoralPlantTypeBlock family + LeafLitterBlock: below sturdy UP.
        "base_coral_plant" | "base_coral_fan" | "coral" | "coral_fan" | "coral_plant"
        | "leaf_litter" => face_sturdy(&below(), Direction::Up),

        // Wall coral fans: attached along FACING.
        "base_coral_wall_fan" | "coral_wall_fan" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            face_sturdy(&world.state_at(pos.relative(facing.opposite())), facing)
        }

        // Pressure plates: rigid OR center support below.
        "pressure_plate" | "weighted_pressure_plate" => {
            let below_state = below();
            can_support_rigid_block(&below_state) || can_support_center(&below_state, Direction::Up)
        }

        // Rails: rigid support below.
        "rail" | "powered_rail" | "detector_rail" => can_support_rigid_block(&below()),

        // DiodeBlock (repeater/comparator): below UP face RIGID-sturdy.
        "repeater" | "comparator" => face_sturdy_typed(&below(), Direction::Up, SupportType::Rigid),

        // RedStoneWireBlock: full-sturdy top OR a hopper below.
        "redstone_wire" => {
            let below_state = below();
            face_sturdy(&below_state, Direction::Up)
                || below_state.registry_id == "minecraft:hopper"
        }

        // BellBlock.
        "bell" => {
            let attachment = state.property("attachment").unwrap_or("floor");
            let facing = facing_property(state).unwrap_or(Direction::North);
            // getConnectedDirection: floor -> UP, ceiling -> DOWN, walls -> facing.
            let connected = match attachment {
                "floor" => Direction::Up,
                "ceiling" => Direction::Down,
                _ => facing,
            };
            let connection_direction = connected.opposite();
            if connection_direction == Direction::Up {
                can_support_center(
                    &world.state_at(pos.relative(Direction::Up)),
                    Direction::Down,
                )
            } else {
                face_attached_can_attach(world, pos, connection_direction)
            }
        }

        // CeilingHangingSignBlock: above DOWN face CENTER-sturdy.
        "ceiling_hanging_sign" => face_sturdy_typed(
            &world.state_at(pos.relative(Direction::Up)),
            Direction::Down,
            SupportType::Center,
        ),

        // CocoaBlock: the log it hangs on must support cocoa.
        "cocoa" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            is_tag(&world.state_at(pos.relative(facing)), "supports_cocoa")
        }

        // FaceAttachedHorizontalDirectionalBlock (buttons, levers).
        "button" | "lever" => face_attached_can_attach(
            world,
            pos,
            face_attached_connected_direction(state).opposite(),
        ),

        // GrindstoneBlock overrides canSurvive to a constant true.
        "grindstone" => true,

        // HangingRootsBlock: above DOWN face sturdy.
        "hanging_roots" => face_sturdy(
            &world.state_at(pos.relative(Direction::Up)),
            Direction::Down,
        ),

        // LadderBlock: behind face sturdy along FACING.
        "ladder" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            face_sturdy(&world.state_at(pos.relative(facing.opposite())), facing)
        }

        // LanternBlock + weathering variants: center support on the attach side.
        "lantern" | "weathering_lantern" => {
            let connected = if state.property("hanging") == Some("true") {
                Direction::Up
            } else {
                Direction::Down
            };
            can_support_center(
                &world.state_at(pos.relative(connected)),
                connected.opposite(),
            )
        }

        // TripWireHookBlock: attached horizontally to a sturdy face.
        "trip_wire_hook" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            if matches!(facing, Direction::Up | Direction::Down) {
                return Some(false);
            }
            face_sturdy(&world.state_at(pos.relative(facing.opposite())), facing)
        }

        // PistonHeadBlock: base piston (or moving piston) behind, same facing.
        "piston_head" => {
            let facing = facing_property(state).unwrap_or(Direction::North);
            let base = world.state_at(pos.relative(facing.opposite()));
            let fitting = match state.property("type") {
                Some("sticky") => base.registry_id == "minecraft:sticky_piston",
                _ => base.registry_id == "minecraft:piston",
            } && base.property("extended") == Some("true")
                && facing_property(&base) == Some(facing);
            fitting
                || base.registry_id == "minecraft:moving_piston"
                    && facing_property(&base) == Some(facing)
        }

        // PointedDripstoneBlock: sturdy along the opposite of tip direction,
        // or chained to another dripstone.
        "pointed_dripstone" => {
            let tip = match state.property("vertical_direction") {
                Some("down") => Direction::Down,
                _ => Direction::Up,
            };
            let attached = world.state_at(pos.relative(tip.opposite()));
            face_sturdy(&attached, tip)
                || attached.registry_id == state.registry_id
                    && attached.property("vertical_direction")
                        == state.property("vertical_direction")
        }

        _ => return None,
    })
}

/// Plant and vegetation canSurvive families.
#[allow(clippy::too_many_lines)]
fn plant_can_survive(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> Option<bool> {
    let below = || world.state_at(pos.relative(Direction::Down));
    let above = || world.state_at(pos.relative(Direction::Up));
    Some(match block_type {
        // Bamboo: below must be in supports_bamboo.
        "bamboo_sapling" | "bamboo_stalk" => is_tag(&below(), "supports_bamboo"),

        // Big dripleaf and stem.
        "big_dripleaf" => {
            let below_state = below();
            below_state.registry_id == state.registry_id
                || below_state.registry_id == "minecraft:big_dripleaf_stem"
                || is_tag(&below_state, "supports_big_dripleaf")
        }
        "big_dripleaf_stem" => {
            let below_state = below();
            let above_state = above();
            (below_state.registry_id == state.registry_id
                || is_tag(&below_state, "supports_big_dripleaf"))
                && (above_state.registry_id == state.registry_id
                    || above_state.registry_id == "minecraft:big_dripleaf")
        }

        // CactusBlock.
        "cactus" => {
            for direction in HORIZONTAL {
                let neighbour = world.state_at(pos.relative(direction));
                if is_solid(&neighbour) || matches!(fluid(&neighbour), StateFluid::Lava { .. }) {
                    return Some(false);
                }
            }
            let below_state = below();
            (below_state.registry_id == state.registry_id
                || is_tag(&below_state, "supports_cactus"))
                && !physics(&above()).is_some_and(|physics| physics.liquid)
        }

        // ChorusFlowerBlock.
        "chorus_flower" => chorus_flower_can_survive(state, pos, world),

        // ChorusPlantBlock.
        "chorus_plant" => chorus_plant_can_survive(state, pos, world),

        // CropBlock family: sufficient light + supports_crops ground.
        "crop" | "carrot" | "potato" | "beetroot" | "torchflower_crop" => {
            world.raw_brightness(pos) >= 8 && is_tag(&below(), "supports_crops")
        }

        // PitcherCropBlock: lower half needs light; halves follow the
        // double-plant rule with supports_crops ground.
        "pitcher_crop" => {
            if state.property("half") != Some("upper") && world.raw_brightness(pos) < 8 {
                return Some(false);
            }
            if state.property("half") == Some("upper") {
                let below_state = below();
                below_state.registry_id == state.registry_id
                    && below_state.property("half") == Some("lower")
            } else {
                is_tag(&below(), "supports_crops")
            }
        }

        // DoublePlantBlock family: upper halves point at their lower half;
        // lower halves use the vegetation rule.
        "double_plant" | "tall_flower" => {
            if state.property("half") == Some("upper") {
                let below_state = below();
                below_state.registry_id == state.registry_id
                    && below_state.property("half") == Some("lower")
            } else {
                is_tag(&below(), "supports_vegetation")
            }
        }
        "tall_seagrass" => {
            if state.property("half") == Some("upper") {
                let below_state = below();
                below_state.registry_id == state.registry_id
                    && below_state.property("half") == Some("lower")
            } else {
                // Lower half: the seagrass ground rule + a full water fluid
                // (Java fluidState.is(WATER) && fluidState.isFull()).
                let below_state = below();
                face_sturdy(&below_state, Direction::Up)
                    && !is_tag(&below_state, "cannot_support_seagrass")
                    && matches!(fluid(state), StateFluid::Water { amount: 8, .. })
            }
        }

        // GrowingPlantBlock families: kelp/twisting/cave vines grow UP
        // (attach below); weeping vines grow DOWN (attach above).
        "kelp"
        | "kelp_plant"
        | "twisting_vines"
        | "twisting_vines_plant"
        | "cave_vines"
        | "cave_vines_plant"
        | "weeping_vines"
        | "weeping_vines_plant" => growing_plant_can_survive(block_type, pos, world),

        // HangingMossBlock: attaches to a full face above or itself.
        "hanging_moss" => {
            let above_state = above();
            above_state.registry_id == state.registry_id
                || multiface_can_attach_to(&above_state, Direction::Up)
        }

        // MangrovePropaguleBlock: hanging from mangrove leaves or planted.
        "mangrove_propagule" => {
            if state.property("hanging") == Some("true") {
                is_tag(&above(), "supports_hanging_mangrove_propagule")
            } else {
                is_tag(&below(), "supports_mangrove_propagule")
            }
        }

        // MossyCarpetBlock.
        "mossy_carpet" => {
            let below_state = below();
            if state.property("base") == Some("true") {
                !physics(&below_state).is_some_and(|physics| physics.is_air)
            } else {
                below_state.registry_id == state.registry_id
                    && below_state.property("base") == Some("true")
            }
        }

        // MushroomBlock: dark enough (or override tag) on solid-render ground.
        "mushroom" => {
            let below_state = below();
            if is_tag(&below_state, "overrides_mushroom_light_requirement") {
                return Some(true);
            }
            world.raw_brightness(pos) < 13
                && physics(&below_state).is_some_and(|physics| physics.is_solid_render)
        }

        // SmallDripleafBlock: double-plant upper rule, else special ground.
        "small_dripleaf" => {
            if state.property("half") == Some("upper") {
                let below_state = below();
                below_state.registry_id == state.registry_id
                    && below_state.property("half") == Some("lower")
            } else {
                let below_state = below();
                is_tag(&below_state, "supports_small_dripleaf")
                    || matches!(fluid(&above()), StateFluid::Water { source: true, .. })
                        && is_tag(&below_state, "supports_vegetation")
            }
        }

        // SugarCaneBlock.
        "sugar_cane" => sugar_cane_can_survive(state, pos, world),

        // VegetationBlock family: ground must support the plant family.
        // NOTE: type `tall_grass` is the SINGLE-block TallGrassBlock
        // (short_grass/fern); the two-tall plants are type `double_plant`.
        // GrassBlock (type `grass`) has no canSurvive override.
        "tall_grass" | "bush" | "flower" | "flower_bed" | "cactus_flower" | "eyeblossom"
        | "firefly_bush" | "sapling" | "azalea" | "lily_pad" | "mangrove_roots"
        | "sweet_berry_bush" | "attached_stem" => {
            if block_type == "lily_pad" && fluid(state) != StateFluid::Empty {
                return Some(false);
            }
            vegetation_can_survive(block_type, state, &below())
        }
        "dry_vegetation" | "short_dry_grass" | "tall_dry_grass" => {
            is_tag(&below(), "supports_dry_vegetation")
        }
        "nether_roots" | "nether_sprouts" | "nether_fungus" | "nether_wart" => {
            nether_vegetation_can_survive(state, &below())
        }
        "seagrass" => {
            // SeagrassBlock.mayPlaceOn: isFaceSturdy(UP) && !cannot_support_seagrass.
            let below_state = below();
            face_sturdy(&below_state, Direction::Up)
                && !is_tag(&below_state, "cannot_support_seagrass")
        }
        "stem" => {
            // StemBlock.stemSupportBlocks: supports_{melon,pumpkin}_stem.
            let tag = if state.registry_id == "minecraft:melon_stem" {
                "supports_melon_stem"
            } else {
                "supports_pumpkin_stem"
            };
            is_tag(&below(), tag)
        }
        "wither_rose" => is_tag(&below(), "supports_wither_rose"),

        // VineBlock: at least one attached face remains valid.
        "vine" => vine_can_survive(state, pos, world),

        _ => return None,
    })
}

/// Remaining canSurvive overrides (fluids, snow, scaffolding, doors, ...).
fn misc_can_survive(
    block_type: &str,
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> Option<bool> {
    let below = || world.state_at(pos.relative(Direction::Down));
    let above = || world.state_at(pos.relative(Direction::Up));
    Some(match block_type {
        // BubbleColumnBlock.
        "bubble_column" => {
            let below_state = below();
            below_state.registry_id == state.registry_id
                || is_tag(&below_state, "enables_bubble_column_push_up")
                || is_tag(&below_state, "enables_bubble_column_drag_down")
        }

        // Carpets: anything non-air below.
        "carpet" | "wool_carpet" => !physics(&below()).is_some_and(|physics| physics.is_air),

        // DirtPathBlock / FarmlandBlock: checks happen against the block above.
        "dirt_path" => {
            let above_state = above();
            !is_solid(&above_state) || is_tag(&above_state, "fence_gates")
        }
        "farmland" => {
            let above_state = above();
            !is_solid(&above_state) || is_tag(&above_state, "maintains_farmland")
        }

        // DoorBlock (incl. weathering copper doors).
        "door" | "weathering_copper_door" => {
            if state.property("half") == Some("lower") {
                face_sturdy(&below(), Direction::Up)
            } else {
                below().registry_id == state.registry_id
            }
        }

        // FireBlock: below face sturdy UP, or any neighbour burnable
        // (Java canBurn = getIgniteOdds(state) > 0; the flammability registry
        // is modelled by crate::fire::flammability).
        "fire" => {
            let below_state = below();
            if face_sturdy(&below_state, Direction::Up) {
                return Some(true);
            }
            ALL_DIRECTIONS.iter().any(|direction| {
                crate::fire::flammability(&world.state_at(pos.relative(*direction)))
                    .is_some_and(|flammable| flammable.ignite_odds > 0)
            })
        }
        "soul_fire" => is_tag(&below(), "soul_fire_base_blocks"),

        // FrogspawnBlock: source water below, no fluid in its own cell.
        "frogspawn" => {
            let below_state = below();
            let supports = matches!(fluid(&below_state), StateFluid::Water { source: true, .. })
                || is_tag(&below_state, "supports_frogspawn");
            supports && fluid(state) == StateFluid::Empty
        }

        // MultifaceBlock (glow lichen, sculk vein).
        "multiface" | "glow_lichen" | "sculk_vein" => multiface_can_survive(state, pos, world),

        // ScaffoldingBlock: survives while its distance property is < 7.
        "scaffolding" => state
            .property("distance")
            .and_then(|distance| distance.parse::<i32>().ok())
            .is_some_and(|distance| distance < 7),

        // SeaPickleBlock: below presents an upward face.
        "sea_pickle" => {
            let below_state = below();
            physics(&below_state).is_some_and(|below_physics| {
                shape_face_nonempty(below_physics.collision_shape, Direction::Up)
            }) || face_sturdy(&below_state, Direction::Up)
        }

        // SnowLayerBlock.
        "snow_layer" => {
            let below_state = below();
            if is_tag(&below_state, "cannot_support_snow_layer") {
                return Some(false);
            }
            if is_tag(&below_state, "support_override_snow_layer") {
                return Some(true);
            }
            physics(&below_state).is_some_and(|below_physics| {
                shape_face_full(below_physics.collision_shape, Direction::Up)
            }) || below_state.registry_id == state.registry_id
                && below_state.property("layers") == Some("8")
        }

        // SporeBlossomBlock: hangs from a center-supporting ceiling, dry cell.
        "spore_blossom" => {
            can_support_center(&above(), Direction::Down) && fluid(state) == StateFluid::Empty
        }

        _ => return None,
    })
}

/// Java `ChorusFlowerBlock.canSurvive`.
fn chorus_flower_can_survive(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> bool {
    let below = world.state_at(pos.relative(Direction::Down));
    if below.registry_id == "minecraft:chorus_plant" || is_tag(&below, "supports_chorus_flower") {
        return true;
    }
    if !physics(&below).is_some_and(|physics| physics.is_air) {
        return false;
    }
    let mut one_neighbour = false;
    for direction in HORIZONTAL {
        let neighbour = world.state_at(pos.relative(direction));
        if neighbour.registry_id == "minecraft:chorus_plant" {
            if one_neighbour {
                return false;
            }
            one_neighbour = true;
        } else if !physics(&neighbour).is_some_and(|physics| physics.is_air) {
            return false;
        }
    }
    let _ = state;
    one_neighbour
}

/// Java `ChorusPlantBlock.canSurvive`.
fn chorus_plant_can_survive(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> bool {
    let below = world.state_at(pos.relative(Direction::Down));
    let above = world.state_at(pos.relative(Direction::Up));
    let blocked_above_and_below = !physics(&above).is_some_and(|physics| physics.is_air)
        && !physics(&below).is_some_and(|physics| physics.is_air);
    for direction in HORIZONTAL {
        let neighbour_pos = pos.relative(direction);
        let neighbour = world.state_at(neighbour_pos);
        if neighbour.registry_id == state.registry_id {
            if blocked_above_and_below {
                return false;
            }
            let neighbour_below = world.state_at(neighbour_pos.relative(Direction::Down));
            if neighbour_below.registry_id == state.registry_id
                || is_tag(&neighbour_below, "supports_chorus_plant")
            {
                return true;
            }
        }
    }
    below.registry_id == state.registry_id || is_tag(&below, "supports_chorus_plant")
}

/// Java `GrowingPlantBlock.canSurvive` for the four growing-plant families.
fn growing_plant_can_survive(block_type: &str, pos: BlockPos, world: &impl SurvivalWorld) -> bool {
    let grows_up = !block_type.starts_with("weeping");
    let attached_pos = if grows_up {
        pos.relative(Direction::Down)
    } else {
        pos.relative(Direction::Up)
    };
    let attached = world.state_at(attached_pos);
    let family: &[&str] = match block_type {
        "kelp" | "kelp_plant" => &["minecraft:kelp", "minecraft:kelp_plant"],
        "twisting_vines" | "twisting_vines_plant" => {
            &["minecraft:twisting_vines", "minecraft:twisting_vines_plant"]
        }
        "cave_vines" | "cave_vines_plant" => {
            &["minecraft:cave_vines", "minecraft:cave_vines_plant"]
        }
        _ => &["minecraft:weeping_vines", "minecraft:weeping_vines_plant"],
    };
    if family.contains(&attached.registry_id.as_str()) {
        return true;
    }
    // KelpPlantBlock.canAttachTo rejects magma; others accept any sturdy face
    // along the growth direction.
    if block_type.starts_with("kelp") && attached.registry_id == "minecraft:magma_block" {
        return false;
    }
    let growth_direction = if grows_up {
        Direction::Up
    } else {
        Direction::Down
    };
    face_sturdy(&attached, growth_direction)
}

/// Java `MultifaceBlock.canSurvive`: every set face must attach to a full face.
fn multiface_can_survive(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> bool {
    let mut has_face = false;
    for direction in ALL_DIRECTIONS {
        let face_name = match direction {
            Direction::Down => "down",
            Direction::Up => "up",
            Direction::North => "north",
            Direction::South => "south",
            Direction::West => "west",
            Direction::East => "east",
        };
        if state.property(face_name) == Some("true") {
            if !multiface_can_attach_to(&world.state_at(pos.relative(direction)), direction) {
                return false;
            }
            has_face = true;
        }
    }
    has_face
}

/// Java `SugarCaneBlock.canSurvive`.
fn sugar_cane_can_survive(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl SurvivalWorld,
) -> bool {
    let below = world.state_at(pos.relative(Direction::Down));
    if below.registry_id == state.registry_id {
        return true;
    }
    if is_tag(&below, "supports_sugar_cane") {
        let below_pos = pos.relative(Direction::Down);
        for direction in HORIZONTAL {
            let adjacent = world.state_at(below_pos.relative(direction));
            if matches!(fluid(&adjacent), StateFluid::Water { .. })
                || is_tag(&adjacent, "supports_sugar_cane_adjacently")
            {
                return true;
            }
        }
    }
    false
}

fn vegetation_can_survive(
    block_type: &str,
    state: &BlockStateModel,
    below: &BlockStateModel,
) -> bool {
    match block_type {
        // AzaleaBlock.mayPlaceOn: supports_azalea.
        "azalea" => is_tag(below, "supports_azalea"),
        // CactusFlowerBlock.mayPlaceOn: override tag or CENTER-sturdy top.
        "cactus_flower" => {
            is_tag(below, "support_override_cactus_flower")
                || face_sturdy_typed(below, Direction::Up, SupportType::Center)
        }
        // LilyPadBlock.mayPlaceOn: source water (the supports_lily_pad fluid
        // tag is exactly minecraft:water) or a supports_lily_pad block below,
        // and no fluid in the lily pad's own cell.
        "lily_pad" => {
            is_tag(below, "supports_lily_pad")
                || matches!(
                    physics(below).map_or(StateFluid::Empty, |physics| physics.fluid),
                    StateFluid::Water { source: true, .. }
                )
        }
        // AttachedStemBlock: the stem ground rule still applies.
        "attached_stem" => {
            let tag = if state.registry_id.contains("melon") {
                "supports_melon_stem"
            } else {
                "supports_pumpkin_stem"
            };
            is_tag(below, tag)
        }
        _ => is_tag(below, "supports_vegetation"),
    }
}

fn nether_vegetation_can_survive(state: &BlockStateModel, below: &BlockStateModel) -> bool {
    match state.registry_id.as_str() {
        "minecraft:crimson_roots" => is_tag(below, "supports_crimson_roots"),
        "minecraft:warped_roots" => is_tag(below, "supports_warped_roots"),
        "minecraft:crimson_fungus" => is_tag(below, "supports_crimson_fungus"),
        "minecraft:warped_fungus" => is_tag(below, "supports_warped_fungus"),
        "minecraft:nether_sprouts" => is_tag(below, "supports_nether_sprouts"),
        "minecraft:nether_wart" => is_tag(below, "supports_nether_wart"),
        _ => is_tag(below, "supports_vegetation"),
    }
}

fn vine_can_survive(state: &BlockStateModel, pos: BlockPos, world: &impl SurvivalWorld) -> bool {
    // Java VineBlock.getUpdatedState: UP face survives if the block above can
    // attach; side faces survive if the side block supports attachment, or the
    // vine one block up has that side face set / can spread there.
    let above_pos = pos.relative(Direction::Up);
    let above = world.state_at(above_pos);
    let mut has_face = false;
    for (face_name, direction) in [
        ("up", Direction::Up),
        ("north", Direction::North),
        ("south", Direction::South),
        ("west", Direction::West),
        ("east", Direction::East),
    ] {
        if state.property(face_name) != Some("true") {
            continue;
        }
        let supported = if direction == Direction::Up {
            multiface_can_attach_to(&above, Direction::Up)
        } else {
            multiface_can_attach_to(&world.state_at(pos.relative(direction)), direction)
                || above.registry_id == state.registry_id
                    && above.property(face_name) == Some("true")
        };
        if supported {
            has_face = true;
        }
    }
    has_face
}

#[cfg(test)]
mod tests;
