//! Attachment-family `getStateForPlacement` ports (bells, chests, buttons,
//! levers, lanterns, cocoa): blocks that probe `canSurvive` across candidate
//! support faces.

use super::*;

/// Families that attach to a support face chosen via canSurvive probing.
pub(super) fn attached_placement(
    block_type: &str,
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> Option<PlacementOutcome> {
    match block_type {
        // BellBlock.
        "bell" => Some(bell_placement(state, context, world)),

        // CampfireBlock.
        "campfire" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            // Java CampfireBlock.isSmokeSource: hay bales only.
            let below = world.state_at(context.clicked_pos.relative(Direction::Down));
            let signal = below.registry_id == "minecraft:hay_block";
            Some(PlacementOutcome::Place(set(
                set(
                    set(
                        set(state, "waterlogged", bool_str(waterlogged)),
                        "signal_fire",
                        bool_str(signal),
                    ),
                    "lit",
                    bool_str(!waterlogged),
                ),
                "facing",
                direction_name(context.horizontal_direction()),
            )))
        }

        // FaceAttachedHorizontalDirectionalBlock (buttons, levers): first
        // looking direction that can attach.
        "button" | "lever" => Some(face_attached_placement(state, context, world)),

        // CocoaBlock: first horizontal looking direction that survives.
        "cocoa" => {
            for direction in context.block_place_nearest_looking_directions() {
                if is_horizontal(direction) {
                    let placed = set(state.clone(), "facing", direction_name(direction));
                    if can_survive(&placed, context.clicked_pos, world) {
                        return Some(PlacementOutcome::Place(placed));
                    }
                }
            }
            Some(PlacementOutcome::Reject)
        }

        // ChestBlock (and trapped/copper chests): single/left/right pairing.
        "chest" | "trapped_chest" => Some(chest_placement(state, context, world)),

        // LanternBlock: hanging when clicked face/looking suggests ceiling.
        "lantern" | "weathering_lantern" => {
            let waterlogged = replaced_by_source_water(world, context.clicked_pos);
            for direction in context.block_place_nearest_looking_directions() {
                if !is_horizontal(direction) {
                    let hanging = direction == Direction::Up;
                    let placed = set(
                        set(state.clone(), "hanging", bool_str(hanging)),
                        "waterlogged",
                        bool_str(waterlogged),
                    );
                    if can_survive(&placed, context.clicked_pos, world) {
                        return Some(PlacementOutcome::Place(placed));
                    }
                }
            }
            Some(PlacementOutcome::Reject)
        }

        _ => None,
    }
}

/// Java `BellBlock.getStateForPlacement`.
fn bell_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let face = context.clicked_face;
    let pos = context.clicked_pos;
    if !is_horizontal(face) {
        let placed = set(
            set(
                state.clone(),
                "attachment",
                if face == Direction::Down {
                    "ceiling"
                } else {
                    "floor"
                },
            ),
            "facing",
            direction_name(context.horizontal_direction()),
        );
        if can_survive(&placed, pos, world) {
            return PlacementOutcome::Place(placed);
        }
    } else {
        let axis_x = matches!(face, Direction::West | Direction::East);
        let double = if axis_x {
            face_sturdy_at(world, pos.relative(Direction::West), Direction::East)
                && face_sturdy_at(world, pos.relative(Direction::East), Direction::West)
        } else {
            face_sturdy_at(world, pos.relative(Direction::North), Direction::South)
                && face_sturdy_at(world, pos.relative(Direction::South), Direction::North)
        };
        let mut placed = set(
            set(state.clone(), "facing", direction_name(face.opposite())),
            "attachment",
            if double { "double_wall" } else { "single_wall" },
        );
        if can_survive(&placed, pos, world) {
            return PlacementOutcome::Place(placed);
        }
        let below_sturdy = face_sturdy_at(world, pos.relative(Direction::Down), Direction::Up);
        placed = set(
            placed,
            "attachment",
            if below_sturdy { "floor" } else { "ceiling" },
        );
        if can_survive(&placed, pos, world) {
            return PlacementOutcome::Place(placed);
        }
    }
    PlacementOutcome::Reject
}

pub(crate) fn face_sturdy_at(
    world: &impl PlacementWorld,
    pos: BlockPos,
    direction: Direction,
) -> bool {
    let state = world.state_at(pos);
    state_physics_by_name(&state.state_name()).is_some_and(|physics| {
        crate::block_properties::is_face_sturdy(
            physics,
            java_ordinal(direction),
            crate::block_properties::SupportType::Full,
        )
    })
}

pub(crate) fn java_ordinal(direction: Direction) -> usize {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

/// Java `FaceAttachedHorizontalDirectionalBlock.getStateForPlacement`.
fn face_attached_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    for direction in context.block_place_nearest_looking_directions() {
        let placed = if !is_horizontal(direction) {
            set(
                set(
                    state.clone(),
                    "face",
                    if direction == Direction::Up {
                        "ceiling"
                    } else {
                        "floor"
                    },
                ),
                "facing",
                direction_name(context.horizontal_direction()),
            )
        } else {
            set(
                set(state.clone(), "face", "wall"),
                "facing",
                direction_name(direction.opposite()),
            )
        };
        if can_survive(&placed, context.clicked_pos, world) {
            return PlacementOutcome::Place(placed);
        }
    }
    PlacementOutcome::Reject
}

/// Java `ChestBlock.getStateForPlacement`.
fn chest_placement(
    state: BlockStateModel,
    context: &PlaceContext,
    world: &impl PlacementWorld,
) -> PlacementOutcome {
    let mut chest_type = "single";
    let mut facing = context.horizontal_direction().opposite();
    let waterlogged = replaced_by_source_water(world, context.clicked_pos);
    let secondary = context.secondary_use_active;
    let face = context.clicked_face;

    if is_horizontal(face) && secondary {
        if let Some(partner_facing) =
            chest_partner_facing(&state, world, context.clicked_pos, face.opposite())
        {
            if axis_name(partner_facing) != axis_name(face) {
                facing = partner_facing;
                chest_type = if counter_clockwise(facing) == face.opposite() {
                    "right"
                } else {
                    "left"
                };
            }
        }
    }

    if chest_type == "single" && !secondary {
        // Java getChestType: a same-facing single chest clockwise pairs LEFT,
        // counter-clockwise pairs RIGHT.
        if chest_partner_facing(&state, world, context.clicked_pos, clockwise(facing))
            .is_some_and(|partner| partner == facing)
        {
            chest_type = "left";
        } else if chest_partner_facing(
            &state,
            world,
            context.clicked_pos,
            counter_clockwise(facing),
        )
        .is_some_and(|partner| partner == facing)
        {
            chest_type = "right";
        }
    }

    PlacementOutcome::Place(set(
        set(
            set(state, "facing", direction_name(facing)),
            "type",
            chest_type,
        ),
        "waterlogged",
        bool_str(waterlogged),
    ))
}

/// Java `ChestBlock.candidatePartnerFacing`: the facing of a SINGLE chest of
/// the same block at `pos` (relative direction applied by the caller).
fn chest_partner_facing(
    state: &BlockStateModel,
    world: &impl PlacementWorld,
    pos: BlockPos,
    direction: Direction,
) -> Option<Direction> {
    let neighbour = world.state_at(pos.relative(direction));
    if neighbour.registry_id == state.registry_id && neighbour.property("type") == Some("single") {
        neighbour.property("facing").and_then(direction_by_name)
    } else {
        None
    }
}
