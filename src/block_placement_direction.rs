use crate::block_update::Direction;

pub(super) fn direction_step(direction: Direction) -> (f64, f64, f64) {
    match direction {
        Direction::West => (-1.0, 0.0, 0.0),
        Direction::East => (1.0, 0.0, 0.0),
        Direction::Down => (0.0, -1.0, 0.0),
        Direction::Up => (0.0, 1.0, 0.0),
        Direction::North => (0.0, 0.0, -1.0),
        Direction::South => (0.0, 0.0, 1.0),
    }
}

pub(super) fn directional_rotation(direction: Direction) -> f32 {
    match direction {
        Direction::South => 0.0,
        Direction::West => 90.0,
        Direction::North | Direction::Up | Direction::Down => 180.0,
        Direction::East => 270.0,
    }
}

/// Java `Direction.fromYRot(double)`.
pub(super) fn direction_from_y_rot(y_rot: f64) -> Direction {
    match ((y_rot / 90.0 + 0.5).floor() as i64 & 3) as u8 {
        0 => Direction::South,
        1 => Direction::West,
        2 => Direction::North,
        _ => Direction::East,
    }
}

/// Java `Direction.orderedByNearest(entity)`.
pub(super) fn ordered_by_nearest(pitch_degrees: f32, yaw_degrees: f32) -> [Direction; 6] {
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
pub(super) fn rotation_segment(degrees: f32) -> i32 {
    (((degrees + 360.0) / 22.5 + 0.5).floor() as i32) & 15
}
