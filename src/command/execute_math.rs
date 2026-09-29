//! Java `Mth` / `Vec3` math used by `/execute` source rewriting.
//!
//! `CommandSourceStack.facing`, local (`^`) coordinates and `align` go through `Mth.sin`,
//! `Mth.cos`, `Mth.atan2` and `Mth.wrapDegrees`, which are table-driven approximations in
//! vanilla; they are ported here so rotations match the Java server bit for bit.

use super::*;
use std::sync::OnceLock;

/// `Mth.SIN` (`(float) Math.sin(i / 10430.378350470453)`).
fn sin_table() -> &'static [f32; 65536] {
    static TABLE: OnceLock<Box<[f32; 65536]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Box::new([0.0_f32; 65536]);
        for (index, slot) in table.iter_mut().enumerate() {
            *slot = (index as f64 / 10430.378350470453).sin() as f32;
        }
        table
    })
}

/// `Mth.sin(double)`.
pub(super) fn mth_sin(value: f64) -> f32 {
    sin_table()[((value * 10430.378350470453) as i64 & 65535) as usize]
}

/// `Mth.cos(double)`.
pub(super) fn mth_cos(value: f64) -> f32 {
    sin_table()[((value * 10430.378350470453 + 16384.0) as i64 & 65535) as usize]
}

/// `Mth.ASIN_TAB` / `Mth.COS_TAB` (257 entries indexed by `ind / 256.0`).
fn atan_tables() -> &'static ([f64; 257], [f64; 257]) {
    static TABLES: OnceLock<([f64; 257], [f64; 257])> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut asin_tab = [0.0; 257];
        let mut cos_tab = [0.0; 257];
        for index in 0..257 {
            let asin = (f64::from(index as u16) / 256.0).asin();
            cos_tab[index] = asin.cos();
            asin_tab[index] = asin;
        }
        (asin_tab, cos_tab)
    })
}

/// `Mth.fastInvSqrt(double)`.
fn fast_inv_sqrt(value: f64) -> f64 {
    let half = 0.5 * value;
    let bits = value.to_bits() as i64;
    let bits = 6910469410427058090_i64.wrapping_sub(bits >> 1);
    let value = f64::from_bits(bits as u64);
    value * (1.5 - half * value * value)
}

/// `Mth.atan2(double y, double x)`.
pub(super) fn mth_atan2(mut y: f64, mut x: f64) -> f64 {
    let d2 = x * x + y * y;
    if d2.is_nan() {
        return f64::NAN;
    }
    let neg_y = y < 0.0;
    if neg_y {
        y = -y;
    }
    let neg_x = x < 0.0;
    if neg_x {
        x = -x;
    }
    let steep = y > x;
    if steep {
        std::mem::swap(&mut x, &mut y);
    }
    let frac_bias = f64::from_bits(4805340802404319232);
    let rinv = fast_inv_sqrt(d2);
    x *= rinv;
    y *= rinv;
    let yp = frac_bias + y;
    let index = yp.to_bits() as i32 as usize;
    let (asin_tab, cos_tab) = atan_tables();
    let phi = asin_tab[index];
    let c_phi = cos_tab[index];
    let s_phi = yp - frac_bias;
    let sd = y * c_phi - x * s_phi;
    let d = (6.0 + sd * sd) * sd * 0.16666666666666666;
    let mut theta = phi + d;
    if steep {
        theta = std::f64::consts::FRAC_PI_2 - theta;
    }
    if neg_x {
        theta = std::f64::consts::PI - theta;
    }
    if neg_y {
        theta = -theta;
    }
    theta
}

/// `Mth.wrapDegrees(float)`.
pub(super) fn mth_wrap_degrees(angle: f32) -> f32 {
    let mut normalized = angle % 360.0;
    if normalized >= 180.0 {
        normalized -= 360.0;
    }
    if normalized < -180.0 {
        normalized += 360.0;
    }
    normalized
}

/// `Vec3.applyLocalCoordinatesToRotation(Vec2 rotation, Vec3 direction)`; `direction` is
/// `(left, up, forwards)` and `rotation` is `(x_rot = pitch, y_rot = yaw)`.
pub(super) fn apply_local_coordinates_to_rotation(
    x_rot: f32,
    y_rot: f32,
    left: f64,
    up: f64,
    forwards: f64,
) -> Vec3 {
    const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;
    let y_cos = mth_cos(f64::from((y_rot + 90.0) * DEG_TO_RAD));
    let y_sin = mth_sin(f64::from((y_rot + 90.0) * DEG_TO_RAD));
    let x_cos = mth_cos(f64::from(-x_rot * DEG_TO_RAD));
    let x_sin = mth_sin(f64::from(-x_rot * DEG_TO_RAD));
    let x_cos_up = mth_cos(f64::from((-x_rot + 90.0) * DEG_TO_RAD));
    let x_sin_up = mth_sin(f64::from((-x_rot + 90.0) * DEG_TO_RAD));
    let forwards_vec = [
        f64::from(y_cos * x_cos),
        f64::from(x_sin),
        f64::from(y_sin * x_cos),
    ];
    let up_vec = [
        f64::from(y_cos * x_cos_up),
        f64::from(x_sin_up),
        f64::from(y_sin * x_cos_up),
    ];
    // `forwards.cross(up).scale(-1.0)`
    let left_vec = [
        -(forwards_vec[1] * up_vec[2] - forwards_vec[2] * up_vec[1]),
        -(forwards_vec[2] * up_vec[0] - forwards_vec[0] * up_vec[2]),
        -(forwards_vec[0] * up_vec[1] - forwards_vec[1] * up_vec[0]),
    ];
    Vec3 {
        x: forwards_vec[0] * forwards + up_vec[0] * up + left_vec[0] * left,
        y: forwards_vec[1] * forwards + up_vec[1] * up + left_vec[1] * left,
        z: forwards_vec[2] * forwards + up_vec[2] * up + left_vec[2] * left,
    }
}

/// `DimensionType.coordinateScale()` of a vanilla dimension (`the_nether` is 8, all others 1).
pub(super) fn dimension_coordinate_scale(dimension: &str) -> f64 {
    if dimension == "minecraft:the_nether" {
        8.0
    } else {
        1.0
    }
}

/// `Mth.floor(double)`.
pub(super) fn mth_floor(value: f64) -> f64 {
    value.floor()
}
