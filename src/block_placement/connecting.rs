//! Connection-scanning and remaining `getStateForPlacement` families: fences,
//! panes/bars, walls, fence gates, tripwire, rails, skulls, fire, leaves,
//! scaffolding, snowy dirt, concrete powder, and the simple waterlogged or
//! survival-dependent placements.

use super::*;
use crate::block_properties::{
    is_face_sturdy, rectangles_cover_region, shape_face_rectangles, SupportType,
};
use crate::block_tags::block_tag_contains;

/// One arm per Java class; length is inherent to the catalog.
#[allow(clippy::too_many_lines)]
pub(super) fn connecting_placement(
    block_type: &str,
    block_id: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let pos = context.clicked_pos;
    let placed = match block_type {
        // FenceBlock: per-side connectsTo + waterlogged.
        "fence" => {
            let mut placed = state;
            for direction in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let neighbour_pos = pos.relative(direction);
                let neighbour = world.state_at(neighbour_pos);
                let connects = fence_connects_to(block_id, &neighbour, direction.opposite(), world);
                placed = set(placed, direction_name(direction), bool_str(connects));
            }
            set(
                placed,
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            )
        }

        // IronBarsBlock (panes, bars): attachsTo = face sturdy or same family.
        "iron_bars" | "stained_glass_pane" | "weathering_copper_bar" => {
            let mut placed = state;
            for direction in [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                let attaches = pane_attaches_to(&neighbour, direction.opposite());
                placed = set(placed, direction_name(direction), bool_str(attaches));
            }
            set(
                placed,
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            )
        }

        // WallBlock: low/tall arms from the above block's DOWN face coverage,
        // plus the post rule.
        "wall" => return Some(wall_placement(state, context, world)),

        // FenceGateBlock.
        "fence_gate" => {
            let powered = world.has_neighbor_signal(pos);
            let facing = context.horizontal_direction();
            let along_x = matches!(facing, Direction::West | Direction::East);
            let in_wall = if along_x {
                is_wall(&world.state_at(pos.relative(Direction::North)))
                    || is_wall(&world.state_at(pos.relative(Direction::South)))
            } else {
                is_wall(&world.state_at(pos.relative(Direction::West)))
                    || is_wall(&world.state_at(pos.relative(Direction::East)))
            };
            set(
                set(
                    set(
                        set(state, "facing", direction_name(facing)),
                        "open",
                        bool_str(powered),
                    ),
                    "powered",
                    bool_str(powered),
                ),
                "in_wall",
                bool_str(in_wall),
            )
        }

        // TripWireBlock: connect to hooks facing back or other tripwire.
        "tripwire" => {
            let mut placed = state;
            for direction in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                let connects = tripwire_connects_to(&neighbour, direction);
                placed = set(placed, direction_name(direction), bool_str(connects));
            }
            placed
        }

        // BaseRailBlock: straight shape along the player's axis.
        "rail" | "powered_rail" | "detector_rail" => {
            let east_west = matches!(
                context.horizontal_direction(),
                Direction::East | Direction::West
            );
            set(
                set(
                    state,
                    "shape",
                    if east_west {
                        "east_west"
                    } else {
                        "north_south"
                    },
                ),
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            )
        }

        // SkullBlock family: 16-segment rotation (no +180 unlike banners) and
        // the AbstractSkullBlock powered sample.
        "skull" | "player_head" | "wither_skull" => set(
            set(
                state,
                "rotation",
                rotation_segment(context.player_yaw).to_string(),
            ),
            "powered",
            bool_str(world.has_neighbor_signal(pos)),
        ),
        // WallSkullBlock family: first horizontal looking direction whose
        // backing block is NOT replaceable.
        "wall_skull" | "player_wall_head" | "wither_wall_skull" | "piglinwallskull" => {
            let powered = world.has_neighbor_signal(pos);
            for direction in context.nearest_looking_directions() {
                if is_horizontal(direction) {
                    let placed = set(
                        set(
                            state.clone(),
                            "facing",
                            direction_name(direction.opposite()),
                        ),
                        "powered",
                        bool_str(powered),
                    );
                    let behind = world.state_at(pos.relative(direction));
                    let replaceable = state_physics_by_name(&behind.state_name())
                        .is_some_and(|physics| physics.replaceable);
                    if !replaceable {
                        return Some(PlacementOutcome::Place(placed));
                    }
                }
            }
            return Some(PlacementOutcome::Reject);
        }

        // BaseFireBlock.getState: soul fire over soul-fire bases, else fire
        // with per-direction faces when the ground cannot hold it.
        "fire" | "soul_fire" => {
            let below = world.state_at(pos.relative(Direction::Down));
            if block_tag_contains("soul_fire_base_blocks", &below.registry_id) {
                return Some(PlacementOutcome::Place(default_state(
                    "minecraft:soul_fire",
                )));
            }
            let fire = default_state("minecraft:fire");
            let below_burns = burns(&below);
            let below_sturdy = face_sturdy_at(world, pos.relative(Direction::Down), Direction::Up);
            if below_burns || below_sturdy {
                return Some(PlacementOutcome::Place(fire));
            }
            let mut placed = fire;
            for (face_name, direction) in [
                ("up", Direction::Up),
                ("north", Direction::North),
                ("south", Direction::South),
                ("west", Direction::West),
                ("east", Direction::East),
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                placed = set(placed, face_name, bool_str(burns(&neighbour)));
            }
            placed
        }

        // LeavesBlock: placed persistent, distance recomputed from neighbors.
        "leaves" | "mangrove_leaves" | "tinted_particle_leaves" | "untinted_particle_leaves" => {
            let placed = set(
                set(state, "persistent", "true"),
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            );
            let mut distance = 7;
            for direction in [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                distance = distance.min(leaves_distance_at(&neighbour) + 1);
                if distance == 1 {
                    break;
                }
            }
            set(placed, "distance", distance.to_string())
        }

        // DirtPathBlock / FarmlandBlock: convert to dirt when they cannot
        // survive at the placement position.
        "dirt_path" | "farmland" => {
            if !can_survive(&state, pos, world) {
                return Some(PlacementOutcome::Place(default_state("minecraft:dirt")));
            }
            state
        }

        // ConcretePowderBlock: solidify on placement into/near water.
        "concrete_powder" => {
            if concrete_should_solidify(world, pos) {
                let concrete_id = state.registry_id.replace("_powder", "");
                return Some(PlacementOutcome::Place(default_state(&concrete_id)));
            }
            state
        }

        // ConduitBlock: waterlogged when the replaced fluid is full water.
        "conduit" => {
            let full_water = matches!(fluid_at(world, pos), StateFluid::Water { amount: 8, .. });
            set(state, "waterlogged", bool_str(full_water))
        }

        // SeagrassBlock: requires a full water fluid, else rejects.
        "seagrass" => {
            if matches!(fluid_at(world, pos), StateFluid::Water { amount: 8, .. }) {
                state
            } else {
                return Some(PlacementOutcome::Reject);
            }
        }

        // HeavyCoreBlock / DecoratedPotBlock.
        "heavy_core" => set(
            state,
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),
        "decorated_pot" => set(
            set(
                set(
                    state,
                    "facing",
                    direction_name(context.horizontal_direction()),
                ),
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            ),
            "cracked",
            "false",
        ),

        // CrafterBlock: ORIENTATION from front + top, TRIGGERED sample.
        "crafter" => {
            let front = context.nearest_looking_direction().opposite();
            let top = match front {
                Direction::Down => context.horizontal_direction().opposite(),
                Direction::Up => context.horizontal_direction(),
                _ => Direction::Up,
            };
            set(
                set(state, "orientation", front_and_top(front, top)),
                "triggered",
                bool_str(world.has_neighbor_signal(pos)),
            )
        }

        // CommandBlock / VaultBlock.
        "command" => set(
            state,
            "facing",
            direction_name(context.nearest_looking_direction().opposite()),
        ),
        "vault" => set(
            state,
            "facing",
            direction_name(context.horizontal_direction().opposite()),
        ),

        // JigsawBlock: ORIENTATION from clicked face + top rule.
        "jigsaw" => {
            let front = context.clicked_face;
            let top = if is_horizontal(front) {
                Direction::Up
            } else {
                context.horizontal_direction().opposite()
            };
            set(state, "orientation", front_and_top(front, top))
        }

        // ScaffoldingBlock: computed distance + bottom flag.
        "scaffolding" => {
            let distance = scaffolding_distance(world, pos);
            let bottom = scaffolding_is_bottom(world, pos, distance);
            set(
                set(
                    set(state, "distance", distance.to_string()),
                    "bottom",
                    bool_str(bottom),
                ),
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            )
        }

        // SnowyBlock (grass/podzol/mycelium): SNOWY from the block above.
        "grass" | "snowy_dirt" | "mycelium" | "nylium" => {
            if state.has_property("snowy") {
                let above = world.state_at(pos.relative(Direction::Up));
                set(
                    state,
                    "snowy",
                    bool_str(block_tag_contains("snow", &above.registry_id)),
                )
            } else {
                state
            }
        }

        // MangroveRootsBlock / TripWireHookBlock.
        "mangrove_roots" => set(
            state,
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),
        "trip_wire_hook" => {
            let base = set(set(state, "powered", "false"), "attached", "false");
            for direction in context.nearest_looking_directions() {
                if is_horizontal(direction) {
                    let placed = set(base.clone(), "facing", direction_name(direction.opposite()));
                    if can_survive(&placed, pos, world) {
                        return Some(PlacementOutcome::Place(placed));
                    }
                }
            }
            return Some(PlacementOutcome::Reject);
        }

        // MossyCarpetBlock: base placement computes the side arms.
        "mossy_carpet" => return Some(mossy_carpet_placement(state, context, world)),

        _ => return None,
    };
    Some(PlacementOutcome::Place(placed))
}

/// Java `FenceBlock.connectsTo` + `isSameFence`.
pub(crate) fn fence_connects_to(
    own_id: &str,
    neighbour: &BlockStateModel,
    direction: Direction,
    world: &impl PlacementWorld,
) -> bool {
    let _ = world;
    let same_fence = block_tag_contains("fences", &neighbour.registry_id)
        && block_tag_contains("wooden_fences", &neighbour.registry_id)
            == block_tag_contains("wooden_fences", own_id);
    let gate = is_fence_gate(neighbour) && fence_gate_connects(neighbour, direction);
    let face_solid = neighbour_face_sturdy(neighbour, direction);
    !is_exception_for_connection(neighbour) && face_solid || same_fence || gate
}

/// Java `IronBarsBlock.attachsTo`: sturdy face, another pane, or a wall.
pub(crate) fn pane_attaches_to(neighbour: &BlockStateModel, direction: Direction) -> bool {
    let face_solid = neighbour_face_sturdy(neighbour, direction);
    let pane = is_pane(neighbour);
    let wall = block_tag_contains("walls", &neighbour.registry_id);
    !is_exception_for_connection(neighbour) && face_solid || pane || wall
}

fn neighbour_face_sturdy(neighbour: &BlockStateModel, direction: Direction) -> bool {
    state_physics_by_name(&neighbour.state_name()).is_some_and(|physics| {
        is_face_sturdy(
            physics,
            super::attached::java_ordinal(direction),
            SupportType::Full,
        )
    })
}

/// Java `Block.isExceptionForConnection`.
fn is_exception_for_connection(state: &BlockStateModel) -> bool {
    let leaves = block_state_entry(&state.registry_id).is_some_and(|entry| {
        matches!(
            entry.block_type,
            "leaves" | "mangrove_leaves" | "tinted_particle_leaves" | "untinted_particle_leaves"
        )
    });
    leaves
        || matches!(
            state.registry_id.as_str(),
            "minecraft:barrier"
                | "minecraft:carved_pumpkin"
                | "minecraft:jack_o_lantern"
                | "minecraft:melon"
                | "minecraft:pumpkin"
        )
        || block_tag_contains("shulker_boxes", &state.registry_id)
}

fn is_fence_gate(state: &BlockStateModel) -> bool {
    block_state_entry(&state.registry_id).is_some_and(|entry| entry.block_type == "fence_gate")
}

fn is_pane(state: &BlockStateModel) -> bool {
    block_state_entry(&state.registry_id).is_some_and(|entry| {
        matches!(
            entry.block_type,
            "iron_bars" | "stained_glass_pane" | "weathering_copper_bar"
        )
    })
}

/// Java `FenceGateBlock.connectsToDirection`: the gate's facing axis is
/// perpendicular to the connection direction.
fn fence_gate_connects(state: &BlockStateModel, direction: Direction) -> bool {
    state
        .property("facing")
        .and_then(direction_by_name)
        .is_some_and(|facing| axis_name(facing) == axis_name(clockwise(direction)))
}

fn is_wall(state: &BlockStateModel) -> bool {
    block_tag_contains("walls", &state.registry_id)
}

/// Java `TripWireBlock.shouldConnectTo`.
pub(crate) fn tripwire_connects_to(neighbour: &BlockStateModel, direction: Direction) -> bool {
    if neighbour.registry_id == "minecraft:tripwire_hook" {
        neighbour.property("facing") == Some(direction_name(direction.opposite()))
    } else {
        neighbour.registry_id == "minecraft:tripwire"
    }
}

/// Java `FireBlock.canBurn` via the flammability registry model.
fn burns(state: &BlockStateModel) -> bool {
    crate::fire::flammability(state).is_some_and(|flammable| flammable.ignite_odds > 0)
}

/// Java `LeavesBlock.getDistanceAt`.
pub(crate) fn leaves_distance_at(state: &BlockStateModel) -> i32 {
    if block_tag_contains("prevents_nearby_leaf_decay", &state.registry_id) {
        return 0;
    }
    state
        .property("distance")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(7)
}

/// Java `ConcretePowderBlock.shouldSolidify` = canSolidify(replaced) ||
/// touchesLiquid.
pub(crate) fn concrete_should_solidify(world: &impl PlacementWorld, pos: BlockPos) -> bool {
    let water = |state: &BlockStateModel| {
        matches!(
            state_physics_by_name(&state.state_name())
                .map_or(StateFluid::Empty, |physics| physics.fluid),
            StateFluid::Water { .. }
        )
    };
    if water(&world.state_at(pos)) {
        return true;
    }
    for direction in [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        // Java skips DOWN unless the powder cell itself solidifies, then
        // requires the neighbouring water face not be sealed off.
        if direction == Direction::Down {
            continue;
        }
        let neighbour_pos = pos.relative(direction);
        let neighbour = world.state_at(neighbour_pos);
        if water(&neighbour) && !neighbour_face_sturdy(&neighbour, direction.opposite()) {
            return true;
        }
    }
    false
}

/// Java `ScaffoldingBlock.getDistance`.
fn scaffolding_distance(world: &impl PlacementWorld, pos: BlockPos) -> i32 {
    let below_pos = pos.relative(Direction::Down);
    let below = world.state_at(below_pos);
    let mut distance = 7;
    if below.registry_id == "minecraft:scaffolding" {
        distance = below
            .property("distance")
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(7);
    } else if face_sturdy_at(world, below_pos, Direction::Up) {
        return 0;
    }
    for direction in [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        let neighbour = world.state_at(pos.relative(direction));
        if neighbour.registry_id == "minecraft:scaffolding" {
            let neighbour_distance = neighbour
                .property("distance")
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(7);
            distance = distance.min(neighbour_distance + 1);
            if distance == 1 {
                break;
            }
        }
    }
    distance
}

/// Java `ScaffoldingBlock.isBottom`.
fn scaffolding_is_bottom(world: &impl PlacementWorld, pos: BlockPos, distance: i32) -> bool {
    let below = world.state_at(pos.relative(Direction::Down));
    distance > 0
        && below.registry_id != "minecraft:scaffolding"
        && !face_sturdy_at(world, pos.relative(Direction::Down), Direction::Up)
}

/// Java `FrontAndTop.fromFrontAndTop` property value name.
fn front_and_top(front: Direction, top: Direction) -> String {
    if is_horizontal(front) {
        // Horizontal fronts pair with TOP=UP as `<front>_up`.
        format!("{}_up", direction_name(front))
    } else {
        format!("{}_{}", direction_name(front), direction_name(top))
    }
}

/// Java `MossyCarpetBlock.getUpdatedState(default, level, pos, true)`.
pub(crate) fn mossy_carpet_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let pos = context.clicked_pos;
    let mut placed = state.clone();
    let above = world.state_at(pos.relative(Direction::Up));
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let face = direction_name(direction);
        let supported = crate::block_survival::multiface_can_attach_to(
            &world.state_at(pos.relative(direction)),
            direction,
        );
        let mut side = if supported { "low" } else { "none" };
        if side == "low"
            && above.registry_id == state.registry_id
            && above.property(face).is_some_and(|value| value != "none")
            && above.property("base") == Some("false")
        {
            side = "tall";
        }
        placed = set(placed, face, side);
    }
    PlacementOutcome::Place(placed)
}

/// Java `WallBlock.getStateForPlacement` + `updateShape`/`updateSides`.
pub(crate) fn wall_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let pos = context.clicked_pos;
    let above_pos = pos.relative(Direction::Up);
    let above = world.state_at(above_pos);
    let above_rectangles = state_physics_by_name(&above.state_name())
        .and_then(|physics| shape_face_rectangles(physics.collision_shape, 0))
        .unwrap_or_default();

    // isCovered(aboveShape, test) = the test region is inside the above
    // block's DOWN collision face. Wall test regions ((x, z), /16 coords):
    // post = [7,7,9,9]; arms run from each edge to z=9 (rotated per side).
    let covered = |region: [f64; 4]| rectangles_cover_region(&above_rectangles, region);
    const POST: [f64; 4] = [7.0 / 16.0, 7.0 / 16.0, 9.0 / 16.0, 9.0 / 16.0];
    let arm_region = |direction: Direction| -> [f64; 4] {
        match direction {
            Direction::North => [7.0 / 16.0, 0.0, 9.0 / 16.0, 9.0 / 16.0],
            Direction::South => [7.0 / 16.0, 7.0 / 16.0, 9.0 / 16.0, 1.0],
            Direction::West => [0.0, 7.0 / 16.0, 9.0 / 16.0, 9.0 / 16.0],
            Direction::East => [7.0 / 16.0, 7.0 / 16.0, 1.0, 9.0 / 16.0],
            _ => POST,
        }
    };

    let mut placed = set(
        state.clone(),
        "waterlogged",
        bool_str(replaced_by_source_water(world, pos)),
    );
    let mut connections = [false; 4];
    let sides = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    for (slot, direction) in sides.iter().enumerate() {
        let neighbour = world.state_at(pos.relative(*direction));
        connections[slot] = wall_connects_to(&neighbour, direction.opposite());
        let value = if connections[slot] {
            if covered(arm_region(*direction)) {
                "tall"
            } else {
                "low"
            }
        } else {
            "none"
        };
        placed = set(placed, direction_name(*direction), value);
    }

    // shouldRaisePost.
    let up = {
        let above_is_wall_post = is_wall(&above) && above.property("up") == Some("true");
        if above_is_wall_post {
            true
        } else {
            let side_value =
                |direction: Direction| placed.property(direction_name(direction)).unwrap_or("none");
            let north_none = side_value(Direction::North) == "none";
            let south_none = side_value(Direction::South) == "none";
            let west_none = side_value(Direction::West) == "none";
            let east_none = side_value(Direction::East) == "none";
            let has_corner = (north_none && south_none && west_none && east_none)
                || north_none != south_none
                || west_none != east_none;
            if has_corner {
                true
            } else {
                let tall_through = (side_value(Direction::North) == "tall"
                    && side_value(Direction::South) == "tall")
                    || (side_value(Direction::East) == "tall"
                        && side_value(Direction::West) == "tall");
                if tall_through {
                    false
                } else {
                    block_tag_contains("wall_post_override", &above.registry_id) || covered(POST)
                }
            }
        }
    };
    PlacementOutcome::Place(set(placed, "up", bool_str(up)))
}

/// Java `WallBlock.connectsTo`.
fn wall_connects_to(neighbour: &BlockStateModel, direction: Direction) -> bool {
    let face_solid = neighbour_face_sturdy(neighbour, direction);
    let gate = is_fence_gate(neighbour) && fence_gate_connects(neighbour, direction);
    block_tag_contains("walls", &neighbour.registry_id)
        || !is_exception_for_connection(neighbour) && face_solid
        || is_pane(neighbour)
        || gate
}
