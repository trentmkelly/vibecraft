//! Plant, aquatic, and multiface `getStateForPlacement` ports: the final
//! catalog batch (bamboo, corals, sculk sensors, hanging signs, chorus,
//! double plants, multiface spreads, vines, growing-plant heads, dripleaves,
//! pointed dripstone, and friends).

use super::attached::{face_sturdy_at, java_ordinal};
use super::*;
use crate::block_properties::{is_face_sturdy, SupportType};
use crate::block_survival::multiface_can_attach_to;
use crate::block_tags::block_tag_contains;

/// One arm per Java class; length is inherent to the catalog.
#[allow(clippy::too_many_lines)]
pub(super) fn plant_placement(
    block_type: &str,
    block_id: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    let pos = context.clicked_pos;
    let placed = match block_type {
        // BambooStalkBlock.
        "bamboo_stalk" => {
            if fluid_at(world, pos) != StateFluid::Empty {
                return Some(PlacementOutcome::Reject);
            }
            let below = world.state_at(pos.relative(Direction::Down));
            if !block_tag_contains("supports_bamboo", &below.registry_id) {
                return Some(PlacementOutcome::Reject);
            }
            if below.registry_id == "minecraft:bamboo_sapling" {
                set(state, "age", "0")
            } else if below.registry_id == "minecraft:bamboo" {
                let age = below
                    .property("age")
                    .and_then(|value| value.parse::<i32>().ok())
                    .unwrap_or(0);
                set(state, "age", if age > 0 { "1" } else { "0" })
            } else {
                let above = world.state_at(pos.relative(Direction::Up));
                if above.registry_id == "minecraft:bamboo" {
                    set(state, "age", above.property("age").unwrap_or("0"))
                } else {
                    default_state("minecraft:bamboo_sapling")
                }
            }
        }

        // BaseCoralPlantTypeBlock family: waterlogged iff a full water fluid.
        "base_coral_plant" | "base_coral_fan" | "coral" | "coral_fan" | "coral_plant" => {
            let full_water = matches!(fluid_at(world, pos), StateFluid::Water { amount: 8, .. });
            set(state, "waterlogged", bool_str(full_water))
        }

        // Coral wall fans: full-water waterlog + looking-direction probe.
        "base_coral_wall_fan" | "coral_wall_fan" => {
            let full_water = matches!(fluid_at(world, pos), StateFluid::Water { amount: 8, .. });
            let base = set(state, "waterlogged", bool_str(full_water));
            for direction in context.block_place_nearest_looking_directions() {
                if is_horizontal(direction) {
                    let placed = set(base.clone(), "facing", direction_name(direction.opposite()));
                    if can_survive(&placed, pos, world) {
                        return Some(PlacementOutcome::Place(placed));
                    }
                }
            }
            return Some(PlacementOutcome::Reject);
        }

        // Sculk sensors and shrieker: waterlogged sample; the calibrated
        // sensor also faces the player.
        "sculk_sensor" | "sculk_shrieker" => set(
            state,
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),
        "calibrated_sculk_sensor" => set(
            set(
                state,
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            ),
            "facing",
            direction_name(context.horizontal_direction()),
        ),

        // CeilingHangingSignBlock.
        "ceiling_hanging_sign" => {
            return Some(ceiling_hanging_sign_placement(state, context, world));
        }
        // WallHangingSignBlock.
        "wall_hanging_sign" => {
            return Some(wall_hanging_sign_placement(state, context, world));
        }

        // ChiseledBookShelfBlock / CopperGolemStatueBlock / DriedGhastBlock.
        "chiseled_book_shelf" => set(
            state,
            "facing",
            direction_name(context.horizontal_direction().opposite()),
        ),
        "copper_golem_statue" | "weathering_copper_golem_statue" => set(
            set(
                state,
                "facing",
                direction_name(context.horizontal_direction().opposite()),
            ),
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),
        "dried_ghast" => set(
            set(
                state,
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            ),
            "facing",
            direction_name(context.horizontal_direction().opposite()),
        ),

        // ChorusPlantBlock.getStateWithConnections.
        "chorus_plant" => {
            let mut placed = state;
            for direction in [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                let connects = neighbour.registry_id == block_id
                    || neighbour.registry_id == "minecraft:chorus_flower"
                    || direction == Direction::Down
                        && block_tag_contains("supports_chorus_plant", &neighbour.registry_id);
                placed = set(placed, direction_name(direction), bool_str(connects));
            }
            placed
        }

        // DoublePlantBlock (incl. tall flowers): lower half placed here, the
        // caller writes the upper half like Java setPlacedBy.
        "double_plant" | "tall_flower" => {
            let above = world.state_at(pos.relative(Direction::Up));
            let above_replaceable = state_physics_by_name(&above.state_name())
                .is_some_and(|physics| physics.replaceable);
            let max_y = crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1;
            if pos.y >= max_y || !above_replaceable {
                return Some(PlacementOutcome::Reject);
            }
            state
        }
        // TallSeagrassBlock: additionally needs full water above.
        "tall_seagrass" => {
            let above = world.state_at(pos.relative(Direction::Up));
            let above_replaceable = state_physics_by_name(&above.state_name())
                .is_some_and(|physics| physics.replaceable);
            let above_full_water = matches!(
                fluid_at(world, pos.relative(Direction::Up)),
                StateFluid::Water { amount: 8, .. }
            );
            if !above_replaceable || !above_full_water {
                return Some(PlacementOutcome::Reject);
            }
            state
        }

        // MultifaceBlock (glow lichen, sculk vein, resin clump): first looking
        // direction with a valid attachment; merges into an existing block.
        "multiface" | "glow_lichen" | "sculk_vein" => {
            let existing = world.state_at(pos);
            let merging = existing.registry_id == block_id;
            for direction in context.block_place_nearest_looking_directions() {
                let face = direction_name(direction);
                if merging && existing.property(face) == Some("true") {
                    continue;
                }
                if !multiface_can_attach_to(&world.state_at(pos.relative(direction)), direction) {
                    continue;
                }
                let base = if merging {
                    existing.clone()
                } else if replaced_by_source_water(world, pos) && state.has_property("waterlogged")
                {
                    set(state.clone(), "waterlogged", "true")
                } else {
                    state.clone()
                };
                return Some(PlacementOutcome::Place(set(base, face, "true")));
            }
            return Some(PlacementOutcome::Reject);
        }

        // VineBlock.
        "vine" => {
            let existing = world.state_at(pos);
            let merging = existing.registry_id == block_id;
            let base = if merging { existing.clone() } else { state };
            for direction in context.block_place_nearest_looking_directions() {
                if direction == Direction::Down {
                    continue;
                }
                let face = direction_name(direction);
                let occupied = merging && existing.property(face) == Some("true");
                if !occupied && vine_can_support_at_face(world, pos, direction) {
                    return Some(PlacementOutcome::Place(set(base, face, "true")));
                }
            }
            return if merging {
                Some(PlacementOutcome::Place(base))
            } else {
                Some(PlacementOutcome::Reject)
            };
        }

        // GrowingPlantHeadBlock: random AGE roll; cave vines never start with
        // berries.
        "kelp" | "twisting_vines" | "weeping_vines" | "cave_vines" => {
            set(state, "age", (context.random_age_roll % 25).to_string())
        }

        // HangingRootsBlock.
        "hanging_roots" => set(
            state,
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),

        // HugeMushroomBlock: faces away from same-block neighbours.
        "huge_mushroom" => {
            let mut placed = state;
            for direction in [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let neighbour = world.state_at(pos.relative(direction));
                placed = set(
                    placed,
                    direction_name(direction),
                    bool_str(neighbour.registry_id != block_id),
                );
            }
            placed
        }

        // MangrovePropaguleBlock: planted with AGE 4.
        "mangrove_propagule" => {
            let placed = set(
                state,
                "waterlogged",
                bool_str(replaced_by_source_water(world, pos)),
            );
            set(placed, "age", "4")
        }

        // PitcherCropBlock places its default lower state.
        "pitcher_crop" => state,

        // ShelfBlock.
        "shelf" => set(
            set(
                set(
                    state,
                    "facing",
                    direction_name(context.horizontal_direction().opposite()),
                ),
                "powered",
                bool_str(world.has_neighbor_signal(pos)),
            ),
            "waterlogged",
            bool_str(replaced_by_source_water(world, pos)),
        ),

        // SmallDripleafBlock: lower half facing away, waterlog copied.
        "small_dripleaf" => {
            let watered = matches!(fluid_at(world, pos), StateFluid::Water { source: true, .. });
            set(
                set(
                    state,
                    "facing",
                    direction_name(context.horizontal_direction().opposite()),
                ),
                "waterlogged",
                bool_str(watered),
            )
        }

        // BigDripleafBlock: inherits the stem's facing when stacking.
        "big_dripleaf" => {
            let below = world.state_at(pos.relative(Direction::Down));
            let part = matches!(
                below.registry_id.as_str(),
                "minecraft:big_dripleaf" | "minecraft:big_dripleaf_stem"
            );
            let facing = if part {
                below.property("facing").unwrap_or("north").to_string()
            } else {
                direction_name(context.horizontal_direction().opposite()).to_string()
            };
            set(
                set(
                    state,
                    "waterlogged",
                    bool_str(matches!(
                        fluid_at(world, pos),
                        StateFluid::Water { source: true, .. }
                    )),
                ),
                "facing",
                facing,
            )
        }

        // PointedDripstoneBlock.
        "pointed_dripstone" => {
            return Some(pointed_dripstone_placement(state, context, world));
        }

        _ => return None,
    };
    Some(PlacementOutcome::Place(placed))
}

/// Java `VineBlock.canSupportAtFace`: UP uses the block above, sides need a
/// full attachment face.
fn vine_can_support_at_face(
    world: &impl PlacementWorld,
    pos: BlockPos,
    direction: Direction,
) -> bool {
    if direction == Direction::Down {
        return false;
    }
    multiface_can_attach_to(&world.state_at(pos.relative(direction)), direction)
}

/// Java `CeilingHangingSignBlock.getStateForPlacement`.
fn ceiling_hanging_sign_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let pos = context.clicked_pos;
    let above_pos = pos.relative(Direction::Up);
    let above = world.state_at(above_pos);
    let below_hanging_sign = block_tag_contains("all_hanging_signs", &above.registry_id);
    let direction = direction_from_y_rot(f64::from(context.player_yaw));
    let above_face_full = state_physics_by_name(&above.state_name()).is_some_and(|physics| {
        crate::block_properties::shape_face_rectangles(physics.collision_shape, 0).is_some_and(
            |rectangles| {
                crate::block_properties::rectangles_cover_region(&rectangles, [0.0, 0.0, 1.0, 1.0])
            },
        )
    });
    let mut attached = !above_face_full || context.secondary_use_active;
    if below_hanging_sign && !context.secondary_use_active {
        if let Some(above_facing) = above.property("facing").and_then(direction_by_name) {
            if axis_name(above_facing) == axis_name(direction) {
                attached = false;
            }
        } else if let Some(rotation) = above
            .property("rotation")
            .and_then(|value| value.parse::<i32>().ok())
        {
            // RotationSegment.convertToDirection: only cardinal segments map.
            let above_direction = match rotation {
                0 => Some(Direction::South),
                4 => Some(Direction::West),
                8 => Some(Direction::North),
                12 => Some(Direction::East),
                _ => None,
            };
            if above_direction.is_some_and(|other| axis_name(other) == axis_name(direction)) {
                attached = false;
            }
        }
    }
    let rotation = if attached {
        rotation_segment(context.player_yaw + 180.0)
    } else {
        // convertToSegment(direction.getOpposite()).
        rotation_segment(match direction.opposite() {
            Direction::South => 0.0,
            Direction::West => 90.0,
            Direction::North => 180.0,
            _ => 270.0,
        })
    };
    PlacementOutcome::Place(set(
        set(
            set(state, "attached", bool_str(attached)),
            "rotation",
            rotation.to_string(),
        ),
        "waterlogged",
        bool_str(replaced_by_source_water(world, pos)),
    ))
}

/// Java `WallHangingSignBlock.getStateForPlacement` + `canPlace`.
fn wall_hanging_sign_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let pos = context.clicked_pos;
    for direction in context.block_place_nearest_looking_directions() {
        if is_horizontal(direction) && axis_name(direction) != axis_name(context.clicked_face) {
            let facing = direction.opposite();
            let placed = set(state.clone(), "facing", direction_name(facing));
            if wall_hanging_sign_can_place(&placed, pos, world) {
                return PlacementOutcome::Place(set(
                    placed,
                    "waterlogged",
                    bool_str(replaced_by_source_water(world, pos)),
                ));
            }
        }
    }
    PlacementOutcome::Reject
}

/// Java `WallHangingSignBlock.canPlace`: a sturdy attachment on either end of
/// the sign bar.
fn wall_hanging_sign_can_place(
    state: &BlockStateModel,
    pos: BlockPos,
    world: &impl PlacementWorld,
) -> bool {
    let Some(facing) = state.property("facing").and_then(direction_by_name) else {
        return false;
    };
    let clockwise_direction = clockwise(facing);
    let counter_direction = counter_clockwise(facing);
    let attaches = |attach_pos: BlockPos, attach_face: Direction| {
        let attach_state = world.state_at(attach_pos);
        if block_tag_contains("all_hanging_signs", &attach_state.registry_id) {
            return true;
        }
        state_physics_by_name(&attach_state.state_name()).is_some_and(|physics| {
            is_face_sturdy(physics, java_ordinal(attach_face), SupportType::Full)
        })
    };
    attaches(pos.relative(clockwise_direction), counter_direction)
        || attaches(pos.relative(counter_direction), clockwise_direction)
}

/// Java `PointedDripstoneBlock.getStateForPlacement`.
fn pointed_dripstone_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let pos = context.clicked_pos;
    let default_tip = context.nearest_looking_vertical_direction().opposite();
    let valid = |tip: Direction| {
        let behind_pos = pos.relative(tip.opposite());
        let behind = world.state_at(behind_pos);
        face_sturdy_at(world, behind_pos, tip)
            || behind.registry_id == "minecraft:pointed_dripstone"
                && behind.property("vertical_direction") == Some(direction_name(tip))
    };
    let tip = if valid(default_tip) {
        default_tip
    } else if valid(default_tip.opposite()) {
        default_tip.opposite()
    } else {
        return PlacementOutcome::Reject;
    };

    let merge = !context.secondary_use_active;
    let is_dripstone_with = |candidate: &BlockStateModel, direction: Direction| {
        candidate.registry_id == "minecraft:pointed_dripstone"
            && candidate.property("vertical_direction") == Some(direction_name(direction))
    };
    let in_front = world.state_at(pos.relative(tip));
    let thickness = if is_dripstone_with(&in_front, tip.opposite()) {
        if !merge && in_front.property("thickness") != Some("tip_merge") {
            "tip"
        } else {
            "tip_merge"
        }
    } else if !is_dripstone_with(&in_front, tip) {
        "tip"
    } else {
        let in_front_thickness = in_front.property("thickness").unwrap_or("tip");
        if in_front_thickness != "tip" && in_front_thickness != "tip_merge" {
            let behind = world.state_at(pos.relative(tip.opposite()));
            if !is_dripstone_with(&behind, tip) {
                "base"
            } else {
                "middle"
            }
        } else {
            "frustum"
        }
    };
    PlacementOutcome::Place(set(
        set(
            set(state, "vertical_direction", direction_name(tip)),
            "thickness",
            thickness,
        ),
        "waterlogged",
        bool_str(replaced_by_source_water(world, pos)),
    ))
}
