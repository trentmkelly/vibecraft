//! `java.util.UUID` helpers used by the UUID related fixes.

/// A UUID as its two 64-bit halves (`getMostSignificantBits` /
/// `getLeastSignificantBits`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaUuid {
    /// Most significant 64 bits.
    pub most: i64,
    /// Least significant 64 bits.
    pub least: i64,
}

/// `Long.parseLong(s, begin, end, 16)`: optional sign, hex digits only.
fn parse_hex_long(text: &str) -> Option<i64> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let magnitude = i128::from_str_radix(digits, 16).ok()?;
    let value = if negative { -magnitude } else { magnitude };
    i64::try_from(value).ok()
}

impl JavaUuid {
    /// `UUID.fromString`: accepts the lenient dash-separated form Java accepts.
    /// Returns `None` where Java throws `IllegalArgumentException`.
    pub fn from_string(name: &str) -> Option<Self> {
        if name.len() > 36 {
            return None;
        }
        let dash1 = name.find('-')?;
        let dash2 = name[dash1 + 1..].find('-').map(|i| i + dash1 + 1)?;
        let dash3 = name[dash2 + 1..].find('-').map(|i| i + dash2 + 1)?;
        let dash4 = name[dash3 + 1..].find('-').map(|i| i + dash3 + 1)?;
        if name[dash4 + 1..].contains('-') {
            return None;
        }
        let mut most = parse_hex_long(&name[..dash1])? & 0xffff_ffff;
        most <<= 16;
        most |= parse_hex_long(&name[dash1 + 1..dash2])? & 0xffff;
        most <<= 16;
        most |= parse_hex_long(&name[dash2 + 1..dash3])? & 0xffff;
        let mut least = parse_hex_long(&name[dash3 + 1..dash4])? & 0xffff;
        least <<= 48;
        least |= parse_hex_long(&name[dash4 + 1..])? & 0xffff_ffff_ffff;
        Some(Self { most, least })
    }
}
