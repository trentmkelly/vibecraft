//! Stateless Minecraft argument parsers used by the completion dispatcher.
//!
//! These return only parse success: completion needs the consumed UTF-16 range,
//! not the argument value. Cursor handling follows the Java argument classes;
//! the dispatcher's argument node restores the cursor when parsing fails.

use serde_json::Value;

use super::StringReader;

/// `ChatFormatting.getNames(true, false)` includes RESET as well as colors.
pub(super) const COLORS: [&str; 17] = [
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
    "reset",
];

/// `ColorArgument.parse` uses ChatFormatting's case-insensitive name cleanup,
/// which removes every character outside a-z before looking up the name.
pub(super) fn parse_color(reader: &mut StringReader<'_>) -> bool {
    let cleaned: String = reader
        .read_unquoted_string()
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_lowercase)
        .collect();
    COLORS.iter().any(|name| name.replace('_', "") == cleaned)
}

/// `SwizzleArgument.parse`: lowercase axes, each occurring at most once.
/// Java accepts an empty set; the dispatcher controls whether an argument node
/// is visited at an empty input position.
pub(super) fn parse_swizzle(reader: &mut StringReader<'_>) -> bool {
    let mut axes = 0;
    while let Some(unit) = reader.peek().filter(|unit| *unit != b' ' as u16) {
        reader.skip();
        let bit = match unit {
            120 => 1,
            121 => 2,
            122 => 4,
            _ => return false,
        };
        if axes & bit != 0 {
            return false;
        }
        axes |= bit;
    }
    true
}

/// `TimeArgument.parse`: readFloat followed by an unquoted, case-sensitive
/// unit; multiplication and rounding happen at Java float precision. Rust's
/// float-to-int cast saturates like Math.round's final narrowing conversion.
pub(super) fn parse_time(properties: &Value, reader: &mut StringReader<'_>) -> bool {
    let Some(value) = reader.read_float() else {
        return false;
    };
    let factor = match reader.read_unquoted_string().as_str() {
        "d" => 24000.0,
        "s" => 20.0,
        "t" | "" => 1.0,
        _ => return false,
    };
    // Math.round(float) rounds ties toward positive infinity, including -0.5.
    // Widen before adding 0.5 so a large integral float cannot round upward
    // merely because float addition loses the fractional increment.
    let ticks = (f64::from(value * factor) + 0.5).floor() as i32;
    ticks as i64 >= properties["min"].as_i64().unwrap_or(0)
}

/// `MinMaxBounds.Bounds.fromReader` leaves nonnumeric suffixes unread. A dot
/// belongs to a bound unless followed by another dot (the range separator).
pub(super) fn parse_range(reader: &mut StringReader<'_>, floating: bool) -> bool {
    let start = reader.cursor();
    let result = (|| {
        let min = read_bound(reader, floating)?;
        let max = if reader.can_read(2)
            && reader.input[reader.cursor()] == b'.' as u16
            && reader.input[reader.cursor() + 1] == b'.' as u16
        {
            reader.skip();
            reader.skip();
            read_bound(reader, floating)?
        } else {
            min
        };
        // `MinMaxBounds.Doubles/Ints.fromReader` rejects swapped bounds via
        // `Bounds.areSwapped()`. Java's `Double.compareTo` distinguishes
        // negative and positive zero, so use Rust's total ordering here rather
        // than `>` (which considers the two zeros equal).
        Some(
            (min.is_some() || max.is_some())
                && !matches!((min, max), (Some(a), Some(b)) if a.total_cmp(&b).is_gt()),
        )
    })();
    if result != Some(true) {
        reader.set_cursor(start);
        return false;
    }
    true
}

fn read_bound(reader: &mut StringReader<'_>, floating: bool) -> Option<Option<f64>> {
    let start = reader.cursor();
    while let Some(unit) = reader.peek() {
        let allowed = matches!(unit, 48..=57 | 45)
            || (unit == b'.' as u16
                && (!reader.can_read(2) || reader.input[reader.cursor() + 1] != b'.' as u16));
        if !allowed {
            break;
        }
        reader.skip();
    }
    if start == reader.cursor() {
        return Some(None);
    }
    let text = String::from_utf16_lossy(&reader.input[start..reader.cursor()]);
    if floating {
        text.parse::<f64>().ok().map(Some)
    } else {
        text.parse::<i32>().ok().map(|value| Some(f64::from(value)))
    }
}
