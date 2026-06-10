#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::base_entity::Vec3;
use crate::network::varint::{read_var_i32, write_var_i32};

const DATA_BITS_MASK: u64 = 0x7FFF;
const MAX_QUANTIZED_VALUE: f64 = 32766.0;
const SCALE_BITS_MASK: u64 = 3;
const CONTINUATION_FLAG: u64 = 4;
const X_OFFSET: u64 = 3;
const Y_OFFSET: u64 = 18;
const Z_OFFSET: u64 = 33;

pub const ABS_MAX_VALUE: f64 = 1.7179869183E10;
pub const ABS_MIN_VALUE: f64 = 3.051944088384301E-5;

pub fn has_continuation_bit(value: u8) -> bool {
    value & CONTINUATION_FLAG as u8 == CONTINUATION_FLAG as u8
}

pub fn read_lp_vec3<R: Read>(reader: &mut R) -> io::Result<Vec3> {
    let mut lowest = [0; 1];
    reader.read_exact(&mut lowest)?;
    let lowest = lowest[0];
    if lowest == 0 {
        return Ok(Vec3::ZERO);
    }

    let mut middle = [0; 1];
    reader.read_exact(&mut middle)?;
    let middle = middle[0] as u64;
    let mut highest = [0; 4];
    reader.read_exact(&mut highest)?;
    let highest = u32::from_be_bytes(highest) as u64;

    let buffer = (highest << 16) | (middle << 8) | u64::from(lowest);
    let mut scale = u64::from(lowest) & SCALE_BITS_MASK;
    if has_continuation_bit(lowest) {
        scale |= u64::from(read_var_i32(reader)? as u32) << 2;
    }
    let scale = scale as f64;

    Ok(Vec3 {
        x: unpack(buffer >> X_OFFSET) * scale,
        y: unpack(buffer >> Y_OFFSET) * scale,
        z: unpack(buffer >> Z_OFFSET) * scale,
    })
}

pub fn write_lp_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    let x = sanitize(value.x);
    let y = sanitize(value.y);
    let z = sanitize(value.z);
    let chessboard_length = x.abs().max(y.abs()).max(z.abs());
    if chessboard_length < ABS_MIN_VALUE {
        writer.write_all(&[0])?;
        return Ok(());
    }

    let scale = chessboard_length.ceil() as u64;
    let is_partial = (scale & SCALE_BITS_MASK) != scale;
    let markers = if is_partial {
        (scale & SCALE_BITS_MASK) | CONTINUATION_FLAG
    } else {
        scale
    };
    let buffer = markers
        | (pack(x / scale as f64) << X_OFFSET)
        | (pack(y / scale as f64) << Y_OFFSET)
        | (pack(z / scale as f64) << Z_OFFSET);

    writer.write_all(&[buffer as u8])?;
    writer.write_all(&[(buffer >> 8) as u8])?;
    writer.write_all(&((buffer >> 16) as u32).to_be_bytes())?;
    if is_partial {
        write_var_i32(writer, (scale >> 2) as i32)?;
    }
    Ok(())
}

pub fn encode_lp_vec3(value: Vec3) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    write_lp_vec3(&mut output, value)?;
    Ok(output)
}

fn sanitize(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-ABS_MAX_VALUE, ABS_MAX_VALUE)
    }
}

fn pack(value: f64) -> u64 {
    ((value * 0.5 + 0.5) * MAX_QUANTIZED_VALUE).round() as u64
}

fn unpack(value: u64) -> f64 {
    (value & DATA_BITS_MASK).min(MAX_QUANTIZED_VALUE as u64) as f64 * 2.0 / MAX_QUANTIZED_VALUE
        - 1.0
}

#[cfg(test)]
mod tests {
    use super::{encode_lp_vec3, has_continuation_bit, read_lp_vec3, ABS_MAX_VALUE, ABS_MIN_VALUE};
    use crate::base_entity::Vec3;
    use std::io::Cursor;

    const LP_VEC3_JAVA: &str =
        include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/LpVec3.java");

    fn assert_close(actual: Vec3, expected: Vec3, scale: f64) {
        let tolerance = scale / 32766.0 + 1.0e-9;
        assert!(
            (actual.x - expected.x).abs() <= tolerance,
            "x: actual={} expected={} tolerance={}",
            actual.x,
            expected.x,
            tolerance
        );
        assert!(
            (actual.y - expected.y).abs() <= tolerance,
            "y: actual={} expected={} tolerance={}",
            actual.y,
            expected.y,
            tolerance
        );
        assert!(
            (actual.z - expected.z).abs() <= tolerance,
            "z: actual={} expected={} tolerance={}",
            actual.z,
            expected.z,
            tolerance
        );
    }

    #[test]
    fn lp_vec3_matches_java_constants_and_zero_path() {
        for sentinel in [
            "private static final int DATA_BITS = 15;",
            "private static final int DATA_BITS_MASK = 32767;",
            "private static final double MAX_QUANTIZED_VALUE = 32766.0;",
            "private static final int CONTINUATION_FLAG = 4;",
            "public static final double ABS_MAX_VALUE = 1.7179869183E10;",
            "public static final double ABS_MIN_VALUE = 3.051944088384301E-5;",
            "return (in & 4) == 4;",
            "if (lowest == 0) {",
            "return Vec3.ZERO;",
        ] {
            assert!(
                LP_VEC3_JAVA.contains(sentinel),
                "missing LpVec3 sentinel {sentinel}"
            );
        }

        assert_eq!(ABS_MAX_VALUE, 1.7179869183E10);
        assert_eq!(ABS_MIN_VALUE, 3.051944088384301E-5);
        assert!(has_continuation_bit(4));
        assert!(!has_continuation_bit(3));

        let tiny = Vec3 {
            x: ABS_MIN_VALUE / 2.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(encode_lp_vec3(tiny).unwrap(), vec![0]);
        assert_eq!(read_lp_vec3(&mut Cursor::new([0])).unwrap(), Vec3::ZERO);
    }

    #[test]
    fn lp_vec3_round_trips_inline_and_continuation_scales() {
        for sentinel in [
            "long scale = lowest & 3;",
            "scale |= (VarInt.read(input) & 4294967295L) << 2;",
            "boolean isPartial = (scale & 3L) != scale;",
            "long markers = isPartial ? scale & 3L | 4L : scale;",
            "VarInt.write(output, (int)(scale >> 2));",
        ] {
            assert!(
                LP_VEC3_JAVA.contains(sentinel),
                "missing LpVec3 sentinel {sentinel}"
            );
        }

        let inline = Vec3 {
            x: 1.0,
            y: 0.0,
            z: -1.0,
        };
        let inline_encoded = encode_lp_vec3(inline).unwrap();
        assert_eq!(inline_encoded.len(), 6);
        assert!(!has_continuation_bit(inline_encoded[0]));
        assert_close(
            read_lp_vec3(&mut Cursor::new(inline_encoded)).unwrap(),
            inline,
            1.0,
        );

        let continued = Vec3 {
            x: 10.0,
            y: -5.0,
            z: 0.25,
        };
        let continued_encoded = encode_lp_vec3(continued).unwrap();
        assert_eq!(continued_encoded.len(), 7);
        assert!(has_continuation_bit(continued_encoded[0]));
        assert_eq!(continued_encoded[6], 2);
        assert_close(
            read_lp_vec3(&mut Cursor::new(continued_encoded)).unwrap(),
            continued,
            10.0,
        );
    }

    #[test]
    fn lp_vec3_sanitizes_nan_and_clamps_extreme_values() {
        for sentinel in [
            "return Double.isNaN(value) ? 0.0 : Math.clamp(value, -1.7179869183E10, 1.7179869183E10);",
            "return Math.round((value * 0.5 + 0.5) * 32766.0);",
            "return Math.min(value & 32767L, 32766.0) * 2.0 / 32766.0 - 1.0;",
        ] {
            assert!(
                LP_VEC3_JAVA.contains(sentinel),
                "missing LpVec3 sentinel {sentinel}"
            );
        }

        let sanitized = Vec3 {
            x: f64::NAN,
            y: ABS_MAX_VALUE * 2.0,
            z: -ABS_MAX_VALUE * 2.0,
        };
        let decoded = read_lp_vec3(&mut Cursor::new(encode_lp_vec3(sanitized).unwrap())).unwrap();
        assert_close(
            decoded,
            Vec3 {
                x: 0.0,
                y: ABS_MAX_VALUE,
                z: -ABS_MAX_VALUE,
            },
            ABS_MAX_VALUE,
        );
    }
}
