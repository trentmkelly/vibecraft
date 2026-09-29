//! `new Dynamic(JsonOps.INSTANCE, json).convert(NbtOps.INSTANCE)`: the conversion
//! of a parsed Gson element to NBT (`JsonOps.convertTo`).

use crate::storage::nbt::Tag;

use super::json::Json;

/// `JsonOps.convertTo(NbtOps.INSTANCE, json)`.
pub fn json_to_nbt(json: &Json) -> Tag {
    match json {
        Json::Null => Tag::End,
        Json::Bool(value) => Tag::Byte(i8::from(*value)),
        Json::String(text) => Tag::String(text.clone()),
        Json::Number(text) => number_to_nbt(text),
        Json::Array(items) => Tag::List(items.iter().map(json_to_nbt).collect()),
        Json::Object(entries) => Tag::Compound(
            entries
                .iter()
                .map(|(name, value)| (name.clone(), json_to_nbt(value)))
                .collect(),
        ),
    }
}

/// The number branch of `JsonOps.convertTo`: an exactly integral `BigDecimal`
/// becomes the narrowest of byte/short/int/long, anything else a float when that
/// is lossless and a double otherwise.
fn number_to_nbt(text: &str) -> Tag {
    if let Some(long) = exact_long(text) {
        return if let Ok(byte) = i8::try_from(long) {
            Tag::Byte(byte)
        } else if let Ok(short) = i16::try_from(long) {
            Tag::Short(short)
        } else if let Ok(int) = i32::try_from(long) {
            Tag::Int(int)
        } else {
            Tag::Long(long)
        };
    }
    let double: f64 = text.parse().unwrap_or(f64::NAN);
    let float = double as f32;
    if f64::from(float) == double {
        Tag::Float(float)
    } else {
        Tag::Double(double)
    }
}

/// `new BigDecimal(text).longValueExact()`, `None` where it would throw
/// `ArithmeticException` (a fractional part or a value outside `long`).
fn exact_long(text: &str) -> Option<i64> {
    let (negative, unsigned) = match text.as_bytes().first()? {
        b'-' => (true, &text[1..]),
        b'+' => (false, &text[1..]),
        _ => (false, text),
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (&unsigned[..at], unsigned[at + 1..].parse::<i64>().ok()?),
        None => (unsigned, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits: String = format!("{whole}{fraction}");
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // value = digits * 10^(exponent - fraction.len())
    let scale = exponent - i64::try_from(fraction.len()).ok()?;
    if digits.bytes().all(|b| b == b'0') {
        return Some(0);
    }
    if scale < 0 {
        let drop = usize::try_from(-scale).ok()?;
        if drop > digits.len() || !digits[digits.len() - drop..].bytes().all(|b| b == b'0') {
            return None;
        }
        digits.truncate(digits.len() - drop);
    } else {
        if scale > 19 {
            return None;
        }
        digits.push_str(&"0".repeat(usize::try_from(scale).ok()?));
    }
    let magnitude: i128 = digits.parse().ok()?;
    i64::try_from(if negative { -magnitude } else { magnitude }).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integral_numbers_take_the_narrowest_integer_type() {
        assert_eq!(number_to_nbt("1"), Tag::Byte(1));
        assert_eq!(number_to_nbt("1.0"), Tag::Byte(1));
        assert_eq!(number_to_nbt("1e2"), Tag::Byte(100));
        assert_eq!(number_to_nbt("300"), Tag::Short(300));
        assert_eq!(number_to_nbt("-70000"), Tag::Int(-70000));
        assert_eq!(number_to_nbt("5000000000"), Tag::Long(5_000_000_000));
    }

    #[test]
    fn fractional_numbers_are_floats_when_lossless() {
        assert_eq!(number_to_nbt("0.5"), Tag::Float(0.5));
        assert_eq!(number_to_nbt("0.1"), Tag::Double(0.1));
        assert_eq!(number_to_nbt("1e30"), Tag::Double(1e30));
    }
}
