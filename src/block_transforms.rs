//! Exact Java `BlockState.rotate` / `BlockState.mirror` for every block,
//! replacing the facing-only approximation in `block_behavior`.
//!
//! Java implements these per class, but the per-property behavior is uniform
//! across the registry: horizontal `facing` rotates through
//! `Rotation.rotate(Direction)`, `axis` swaps X/Z on quarter turns,
//! four-direction connection properties permute (fences, panes, walls with
//! their `none/low/tall` sides, vines, mushroom caps, chorus, tripwire),
//! 16-segment `rotation` follows `Rotation.rotate(int, 16)` /
//! `Mirror.mirror(int, 16)`, and `FrontAndTop` orientations rotate their
//! horizontal component. Class-specific overrides — rail shape tables, the
//! stair-shape mirror swaps, and `DoorBlock.mirror`'s hinge cycle — are
//! dispatched on the official block-type key.

#![allow(dead_code)]

use crate::block_behavior::{BlockStateModel, Mirror, Rotation};
use crate::block_placement::{direction_by_name, direction_name};
use crate::block_states::block_state_entry;
use crate::block_update::Direction;

/// Java `Rotation.rotate(Direction)` (no-op on the Y axis).
pub fn rotate_direction(direction: Direction, rotation: Rotation) -> Direction {
    if matches!(direction, Direction::Up | Direction::Down) {
        return direction;
    }
    match rotation {
        Rotation::None => direction,
        Rotation::Clockwise90 => clockwise(direction),
        Rotation::Clockwise180 => direction.opposite(),
        Rotation::CounterClockwise90 => clockwise(direction).opposite(),
    }
}

/// Java `Mirror.mirror(Direction)`.
pub fn mirror_direction(direction: Direction, mirror: Mirror) -> Direction {
    let on_x = matches!(direction, Direction::West | Direction::East);
    let on_z = matches!(direction, Direction::North | Direction::South);
    match mirror {
        Mirror::FrontBack if on_x => direction.opposite(),
        Mirror::LeftRight if on_z => direction.opposite(),
        _ => direction,
    }
}

/// Java `Mirror.getRotation(Direction)`: the 180-degree turn a mirror implies
/// for a block facing along the mirrored axis.
fn mirror_rotation(direction: Direction, mirror: Mirror) -> Rotation {
    let on_x = matches!(direction, Direction::West | Direction::East);
    let on_z = matches!(direction, Direction::North | Direction::South);
    if (mirror == Mirror::LeftRight && on_z) || (mirror == Mirror::FrontBack && on_x) {
        Rotation::Clockwise180
    } else {
        Rotation::None
    }
}

fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        other => other,
    }
}

/// Java `Rotation.rotate(int, int)` for segment properties.
fn rotate_segments(rotation: Rotation, value: i32, steps: i32) -> i32 {
    match rotation {
        Rotation::Clockwise90 => (value + steps / 4) % steps,
        Rotation::Clockwise180 => (value + steps / 2) % steps,
        Rotation::CounterClockwise90 => (value + steps * 3 / 4) % steps,
        Rotation::None => value,
    }
}

/// Java `Mirror.mirror(int, int)` for segment properties.
fn mirror_segments(mirror: Mirror, value: i32, steps: i32) -> i32 {
    let half = steps / 2;
    let corrected = if value > half { value - steps } else { value };
    match mirror {
        Mirror::LeftRight => (half - corrected + steps) % steps,
        Mirror::FrontBack => (steps - corrected) % steps,
        Mirror::None => value,
    }
}

/// Java `BlockState.rotate(Rotation)`.
pub fn rotate_state(state: &BlockStateModel, rotation: Rotation) -> BlockStateModel {
    if rotation == Rotation::None {
        return state.clone();
    }
    let block_type = block_state_entry(&state.registry_id)
        .map(|entry| entry.block_type)
        .unwrap_or("");

    // Rails use the explicit shape tables.
    if matches!(block_type, "rail" | "powered_rail" | "detector_rail") {
        if let Some(shape) = state.property("shape") {
            return state
                .clone()
                .try_set_property("shape", rotate_rail_shape(shape, rotation));
        }
    }

    let mut result = state.clone();
    // FrontAndTop orientations (jigsaw, crafter): rotate horizontal parts.
    if let Some(orientation) = state.property("orientation") {
        let rotated = rotate_orientation(orientation, rotation);
        return result.try_set_property("orientation", rotated);
    }
    if let Some(facing) = state.property("facing").and_then(direction_by_name) {
        result =
            result.try_set_property("facing", direction_name(rotate_direction(facing, rotation)));
    }
    if let Some(axis) = state.property("axis") {
        if matches!(
            rotation,
            Rotation::Clockwise90 | Rotation::CounterClockwise90
        ) {
            let swapped = match axis {
                "x" => "z",
                "z" => "x",
                other => other,
            };
            result = result.try_set_property("axis", swapped);
        }
    }
    if let Some(value) = state
        .property("rotation")
        .and_then(|raw| raw.parse::<i32>().ok())
    {
        result =
            result.try_set_property("rotation", rotate_segments(rotation, value, 16).to_string());
    }
    if has_horizontal_sides(state) {
        result = permute_sides(&result, |direction| {
            // The NEW value of `direction` comes from the side that rotates
            // ONTO it: rotate by the inverse.
            rotate_direction(direction, inverse(rotation))
        });
    }
    result
}

/// Java `BlockState.mirror(Mirror)`.
pub fn mirror_state(state: &BlockStateModel, mirror: Mirror) -> BlockStateModel {
    if mirror == Mirror::None {
        return state.clone();
    }
    let block_type = block_state_entry(&state.registry_id)
        .map(|entry| entry.block_type)
        .unwrap_or("");

    if matches!(block_type, "rail" | "powered_rail" | "detector_rail") {
        if let Some(shape) = state.property("shape") {
            return state
                .clone()
                .try_set_property("shape", mirror_rail_shape(shape, mirror));
        }
    }

    // DoorBlock.mirror: rotate by the implied 180 turn, then cycle the hinge.
    if matches!(block_type, "door" | "weathering_copper_door") {
        if let Some(facing) = state.property("facing").and_then(direction_by_name) {
            let turn = mirror_rotation(facing, mirror);
            if turn == Rotation::None {
                return state.clone();
            }
            let rotated = rotate_state(state, turn);
            let hinge = if rotated.property("hinge") == Some("left") {
                "right"
            } else {
                "left"
            };
            return rotated.try_set_property("hinge", hinge);
        }
    }

    // StairBlock.mirror: the implied 180 turn swaps the corner handedness.
    if matches!(block_type, "stair" | "weathering_copper_stair") {
        if let Some(facing) = state.property("facing").and_then(direction_by_name) {
            if mirror_rotation(facing, mirror) == Rotation::Clockwise180 {
                let rotated = rotate_state(state, Rotation::Clockwise180);
                let shape = match rotated.property("shape") {
                    Some("outer_left") => "outer_right",
                    Some("outer_right") => "outer_left",
                    Some("inner_left") => "inner_right",
                    Some("inner_right") => "inner_left",
                    _ => "straight",
                };
                return rotated.try_set_property("shape", shape);
            }
            return state.clone();
        }
    }

    let mut result = state.clone();
    if let Some(facing) = state.property("facing").and_then(direction_by_name) {
        result =
            result.try_set_property("facing", direction_name(mirror_direction(facing, mirror)));
    }
    if let Some(value) = state
        .property("rotation")
        .and_then(|raw| raw.parse::<i32>().ok())
    {
        result =
            result.try_set_property("rotation", mirror_segments(mirror, value, 16).to_string());
    }
    if has_horizontal_sides(state) {
        // Mirrors swap one opposing side pair; the mapping is involutive.
        result = permute_sides(&result, |direction| mirror_direction(direction, mirror));
    }
    result
}

fn inverse(rotation: Rotation) -> Rotation {
    match rotation {
        Rotation::Clockwise90 => Rotation::CounterClockwise90,
        Rotation::CounterClockwise90 => Rotation::Clockwise90,
        other => other,
    }
}

fn has_horizontal_sides(state: &BlockStateModel) -> bool {
    ["north", "south", "west", "east"]
        .iter()
        .all(|side| state.has_property(side))
}

/// Reassigns the four horizontal side properties: the new value of side `d`
/// comes from side `source(d)`.
fn permute_sides(
    state: &BlockStateModel,
    source: impl Fn(Direction) -> Direction,
) -> BlockStateModel {
    let mut result = state.clone();
    for direction in [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        let from = source(direction);
        if let Some(value) = state.property(direction_name(from)) {
            result = result.try_set_property(direction_name(direction), value);
        }
    }
    result
}

/// Java `FrontAndTop` rotation: both component directions rotate.
fn rotate_orientation(orientation: &str, rotation: Rotation) -> String {
    let Some((front_raw, top_raw)) = orientation.split_once('_') else {
        return orientation.to_string();
    };
    let rotate_part = |part: &str| -> String {
        direction_by_name(part)
            .map(|direction| direction_name(rotate_direction(direction, rotation)).to_string())
            .unwrap_or_else(|| part.to_string())
    };
    format!("{}_{}", rotate_part(front_raw), rotate_part(top_raw))
}

/// Java `BaseRailBlock.rotate(RailShape, Rotation)`.
fn rotate_rail_shape(shape: &str, rotation: Rotation) -> &'static str {
    let table: &[(&str, &str)] = match rotation {
        Rotation::Clockwise180 => &[
            ("ascending_east", "ascending_west"),
            ("ascending_west", "ascending_east"),
            ("ascending_north", "ascending_south"),
            ("ascending_south", "ascending_north"),
            ("north_south", "north_south"),
            ("east_west", "east_west"),
            ("south_east", "north_west"),
            ("south_west", "north_east"),
            ("north_west", "south_east"),
            ("north_east", "south_west"),
        ],
        Rotation::CounterClockwise90 => &[
            ("ascending_east", "ascending_north"),
            ("ascending_west", "ascending_south"),
            ("ascending_north", "ascending_west"),
            ("ascending_south", "ascending_east"),
            ("north_south", "east_west"),
            ("east_west", "north_south"),
            ("south_east", "north_east"),
            ("south_west", "south_east"),
            ("north_west", "south_west"),
            ("north_east", "north_west"),
        ],
        Rotation::Clockwise90 => &[
            ("ascending_east", "ascending_south"),
            ("ascending_west", "ascending_north"),
            ("ascending_north", "ascending_east"),
            ("ascending_south", "ascending_west"),
            ("north_south", "east_west"),
            ("east_west", "north_south"),
            ("south_east", "south_west"),
            ("south_west", "north_west"),
            ("north_west", "north_east"),
            ("north_east", "south_east"),
        ],
        Rotation::None => return lookup_identity(shape),
    };
    table
        .iter()
        .find(|(from, _)| *from == shape)
        .map(|(_, to)| *to)
        .unwrap_or_else(|| lookup_identity(shape))
}

/// Java `BaseRailBlock.mirror(RailShape, Mirror)`.
fn mirror_rail_shape(shape: &str, mirror: Mirror) -> &'static str {
    let table: &[(&str, &str)] = match mirror {
        Mirror::LeftRight => &[
            ("ascending_north", "ascending_south"),
            ("ascending_south", "ascending_north"),
            ("south_east", "north_east"),
            ("south_west", "north_west"),
            ("north_west", "south_west"),
            ("north_east", "south_east"),
        ],
        Mirror::FrontBack => &[
            ("ascending_east", "ascending_west"),
            ("ascending_west", "ascending_east"),
            ("south_east", "south_west"),
            ("south_west", "south_east"),
            ("north_west", "north_east"),
            ("north_east", "north_west"),
        ],
        Mirror::None => return lookup_identity(shape),
    };
    table
        .iter()
        .find(|(from, _)| *from == shape)
        .map(|(_, to)| *to)
        .unwrap_or_else(|| lookup_identity(shape))
}

fn lookup_identity(shape: &str) -> &'static str {
    match shape {
        "ascending_east" => "ascending_east",
        "ascending_west" => "ascending_west",
        "ascending_north" => "ascending_north",
        "ascending_south" => "ascending_south",
        "east_west" => "east_west",
        "south_east" => "south_east",
        "south_west" => "south_west",
        "north_west" => "north_west",
        "north_east" => "north_east",
        _ => "north_south",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str) -> BlockStateModel {
        BlockStateModel::default_for(id).unwrap_or_else(|| panic!("unknown block {id}"))
    }

    #[test]
    fn facing_axis_and_segment_properties_transform_like_java() {
        // Horizontal facing rotates; UP/DOWN facings do not.
        let furnace = block("minecraft:furnace"); // facing north
        assert_eq!(
            rotate_state(&furnace, Rotation::Clockwise90).property("facing"),
            Some("east")
        );
        assert_eq!(
            rotate_state(&furnace, Rotation::CounterClockwise90).property("facing"),
            Some("west")
        );
        let hopper_down = block("minecraft:hopper"); // facing down
        assert_eq!(
            rotate_state(&hopper_down, Rotation::Clockwise90).property("facing"),
            Some("down")
        );

        // Mirrors flip only the matching axis.
        assert_eq!(
            mirror_state(&furnace, Mirror::LeftRight).property("facing"),
            Some("south")
        );
        assert_eq!(
            mirror_state(&furnace, Mirror::FrontBack).property("facing"),
            Some("north")
        );

        // Pillar axes swap on quarter turns only.
        let log = block("minecraft:oak_log").try_set_property("axis", "x");
        assert_eq!(
            rotate_state(&log, Rotation::Clockwise90).property("axis"),
            Some("z")
        );
        assert_eq!(
            rotate_state(&log, Rotation::Clockwise180).property("axis"),
            Some("x")
        );

        // 16-segment rotations.
        let banner = block("minecraft:white_banner").try_set_property("rotation", "1");
        assert_eq!(
            rotate_state(&banner, Rotation::Clockwise90).property("rotation"),
            Some("5")
        );
        assert_eq!(
            mirror_state(&banner, Mirror::FrontBack).property("rotation"),
            Some("15")
        );
        assert_eq!(
            mirror_state(&banner, Mirror::LeftRight).property("rotation"),
            Some("7")
        );
    }

    #[test]
    fn side_connection_properties_permute_like_java() {
        let fence = block("minecraft:oak_fence")
            .try_set_property("north", "true")
            .try_set_property("east", "true");
        let rotated = rotate_state(&fence, Rotation::Clockwise90);
        assert_eq!(rotated.property("east"), Some("true"));
        assert_eq!(rotated.property("south"), Some("true"));
        assert_eq!(rotated.property("north"), Some("false"));

        let mirrored = mirror_state(&fence, Mirror::LeftRight);
        assert_eq!(mirrored.property("south"), Some("true"));
        assert_eq!(mirrored.property("east"), Some("true"));
        assert_eq!(mirrored.property("north"), Some("false"));

        // Wall sides carry their low/tall values through the permutation.
        let wall = block("minecraft:cobblestone_wall").try_set_property("north", "tall");
        let rotated = rotate_state(&wall, Rotation::Clockwise180);
        assert_eq!(rotated.property("south"), Some("tall"));
        assert_eq!(rotated.property("north"), Some("none"));
    }

    #[test]
    fn rails_doors_and_stairs_use_their_class_tables() {
        let rail = block("minecraft:rail").try_set_property("shape", "south_east");
        assert_eq!(
            rotate_state(&rail, Rotation::Clockwise90).property("shape"),
            Some("south_west")
        );
        assert_eq!(
            mirror_state(&rail, Mirror::LeftRight).property("shape"),
            Some("north_east")
        );
        let ascending =
            block("minecraft:powered_rail").try_set_property("shape", "ascending_north");
        assert_eq!(
            rotate_state(&ascending, Rotation::Clockwise90).property("shape"),
            Some("ascending_east")
        );

        // Doors cycle their hinge under an affecting mirror.
        let door = block("minecraft:oak_door"); // facing east by default? use explicit
        let door = door.try_set_property("facing", "north");
        let mirrored = mirror_state(&door, Mirror::LeftRight);
        assert_eq!(mirrored.property("facing"), Some("south"));
        assert_eq!(mirrored.property("hinge"), Some("right"));
        // A non-affecting mirror leaves the door alone.
        let untouched = mirror_state(&door, Mirror::FrontBack);
        assert_eq!(untouched, door);

        // Stairs swap corner handedness under an affecting mirror.
        let stairs = block("minecraft:oak_stairs")
            .try_set_property("facing", "north")
            .try_set_property("shape", "outer_left");
        let mirrored = mirror_state(&stairs, Mirror::LeftRight);
        assert_eq!(mirrored.property("facing"), Some("south"));
        assert_eq!(mirrored.property("shape"), Some("outer_right"));

        // Jigsaw orientations rotate their horizontal components.
        let jigsaw = block("minecraft:jigsaw").try_set_property("orientation", "north_up");
        assert_eq!(
            rotate_state(&jigsaw, Rotation::Clockwise90).property("orientation"),
            Some("east_up")
        );
    }
}
