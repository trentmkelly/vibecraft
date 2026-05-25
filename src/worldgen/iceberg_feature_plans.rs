use super::*;

pub struct IcebergShapeRolls {
    pub snow_roll: f64,
    pub angle_roll: f64,
    pub ellipse_a_roll: i32,
    pub ellipse_c_roll: i32,
    pub ellipse_roll: f64,
    pub height_roll: i32,
    pub tall_roll: f64,
    pub tall_extra_roll: i32,
    pub underwater_roll: i32,
    pub width_plus_roll: i32,
    pub width_minus_roll: i32,
}

pub fn iceberg_shape_model(rolls: IcebergShapeRolls) -> IcebergShapeModel {
    let is_ellipse = rolls.ellipse_roll > 0.7;
    let mut over_water_height = if is_ellipse {
        rolls.height_roll.rem_euclid(6) + 6
    } else {
        rolls.height_roll.rem_euclid(15) + 3
    };
    if !is_ellipse && rolls.tall_roll > 0.9 {
        over_water_height += rolls.tall_extra_roll.rem_euclid(19) + 7;
    }
    IcebergShapeModel {
        snow_on_top: rolls.snow_roll > 0.7,
        shape_angle: rolls.angle_roll * 2.0 * std::f64::consts::PI,
        shape_ellipse_a: 11 - rolls.ellipse_a_roll.rem_euclid(5),
        shape_ellipse_c: 3 + rolls.ellipse_c_roll.rem_euclid(3),
        is_ellipse,
        over_water_height,
        under_water_height: (over_water_height + rolls.underwater_roll.rem_euclid(11)).min(18),
        width: (over_water_height + rolls.width_plus_roll.rem_euclid(7)
            - rolls.width_minus_roll.rem_euclid(5))
        .min(11),
    }
}

pub fn iceberg_ellipse_c(y_off: i32, height: i32, shape_ellipse_c: i32) -> i32 {
    if y_off > 0 && height - y_off <= 3 {
        shape_ellipse_c - (4 - (height - y_off))
    } else {
        shape_ellipse_c
    }
}

pub fn iceberg_signed_distance_circle(
    xo: i32,
    zo: i32,
    origin: BlockPos,
    radius: i32,
    float_roll: f32,
) -> f64 {
    let off = 10.0 * f64::from(float_roll.clamp(0.2, 0.8)) / f64::from(radius.max(1));
    let dx = f64::from(xo - origin.x);
    let dz = f64::from(zo - origin.z);
    off + dx.powi(2) + dz.powi(2) - f64::from(radius).powi(2)
}

pub fn iceberg_signed_distance_ellipse(
    xo: i32,
    zo: i32,
    origin: BlockPos,
    a: i32,
    c: i32,
    angle: f64,
) -> f64 {
    let dx = f64::from(xo - origin.x);
    let dz = f64::from(zo - origin.z);
    ((dx * angle.cos() - dz * angle.sin()) / f64::from(a.max(1))).powi(2)
        + ((dx * angle.sin() + dz * angle.cos()) / f64::from(c.max(1))).powi(2)
        - 1.0
}

pub fn iceberg_height_radius_round(
    y_off: i32,
    height: i32,
    width: i32,
    float_roll: f32,
    tall_height_roll: i32,
    tall_y_roll: i32,
) -> i32 {
    let k = 3.5 - float_roll;
    let mut effective_y = y_off;
    let mut scale = (1.0 - (y_off as f32).powi(2) / (height as f32 * k)) * width as f32;
    if height > 15 + tall_height_roll.rem_euclid(5) {
        if y_off < 3 + tall_y_roll.rem_euclid(6) {
            effective_y = y_off / 2;
        }
        scale = (1.0 - effective_y as f32 / (height as f32 * k * 0.4)) * width as f32;
    }
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_height_radius_ellipse(y_off: i32, height: i32, width: i32) -> i32 {
    let scale = (1.0 - (y_off as f32).powi(2) / height as f32) * width as f32;
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_height_radius_steep(y_off: i32, height: i32, width: i32, float_roll: f32) -> i32 {
    let k = 1.0 + float_roll / 2.0;
    let scale = (1.0 - y_off as f32 / (height as f32 * k)) * width as f32;
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_set_block_action(
    current_state: &str,
    h_diff: i32,
    height: i32,
    is_ellipse: bool,
    snow_on_top: bool,
    snow_height_roll: i32,
    ellipse_skip_roll: f64,
) -> IcebergBlockAction {
    if !matches!(
        current_state,
        "minecraft:air" | "minecraft:snow_block" | "minecraft:ice" | "minecraft:water"
    ) {
        return IcebergBlockAction::Keep;
    }
    let randomness = !is_ellipse || ellipse_skip_roll > 0.05;
    let divisor = if is_ellipse { 3 } else { 2 };
    let snow_limit =
        snow_height_roll.rem_euclid((height / divisor).max(1)) as f64 + f64::from(height) * 0.6;
    if snow_on_top
        && current_state != "minecraft:water"
        && f64::from(h_diff) <= snow_limit
        && randomness
    {
        IcebergBlockAction::SnowBlock
    } else {
        IcebergBlockAction::MainBlock
    }
}

pub fn iceberg_should_skip_surface_noise(
    signed_distance: f64,
    is_ellipse: bool,
    roll: f64,
) -> bool {
    let compare_val = if is_ellipse { -0.5 } else { -6.0 };
    signed_distance > compare_val && roll > 0.9
}

pub fn iceberg_carve_action(current_state: &str, under_water: bool) -> IcebergBlockAction {
    if matches!(
        current_state,
        "minecraft:packed_ice" | "minecraft:snow_block" | "minecraft:blue_ice"
    ) {
        if under_water {
            IcebergBlockAction::Water
        } else {
            IcebergBlockAction::Air
        }
    } else {
        IcebergBlockAction::Keep
    }
}

pub fn iceberg_smooth_action(
    current_state: &str,
    below_is_air: bool,
    horizontal_non_iceberg_neighbors: i32,
) -> IcebergBlockAction {
    if (below_is_air
        && matches!(
            current_state,
            "minecraft:packed_ice"
                | "minecraft:snow_block"
                | "minecraft:blue_ice"
                | "minecraft:snow"
        ))
        || (horizontal_non_iceberg_neighbors >= 3
            && matches!(
                current_state,
                "minecraft:packed_ice" | "minecraft:snow_block" | "minecraft:blue_ice"
            ))
    {
        IcebergBlockAction::Air
    } else {
        IcebergBlockAction::Keep
    }
}
