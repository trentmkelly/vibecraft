//! Coordinate argument suggestions: `SharedSuggestionProvider.suggestCoordinates`
//! and `suggest2DCoordinates`, driven by `Vec3Argument`, `BlockPosArgument`,
//! `Vec2Argument` and `ColumnPosArgument.listSuggestions`.
//!
//! A `CommandSourceStack` offers only `TextCoordinates.DEFAULT_GLOBAL` (`~ ~ ~`)
//! as its absolute/relevant coordinates; a leading `^` selects
//! `DEFAULT_LOCAL` (`^ ^ ^`). Candidates are kept only when the full
//! coordinate text parses with the argument's own parser (`Commands.createValidator`).

use super::{suggest_matching, StringReader, SuggestionsBuilder};

/// Which coordinate argument is being completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `Vec3Argument`: three doubles.
    Vec3,
    /// `BlockPosArgument`: three ints.
    BlockPos,
    /// `Vec2Argument`: two doubles.
    Vec2,
    /// `ColumnPosArgument`: two ints.
    ColumnPos,
}

impl Kind {
    fn is_3d(self) -> bool {
        matches!(self, Self::Vec3 | Self::BlockPos)
    }

    fn integral(self) -> bool {
        matches!(self, Self::BlockPos | Self::ColumnPos)
    }
}

/// `listSuggestions` for a coordinate argument.
pub(crate) fn suggest(builder: &mut SuggestionsBuilder<'_>, kind: Kind) {
    let remainder = builder.remaining();
    let (x, y, z) = if remainder.starts_with('^') {
        ("^", "^", "^")
    } else {
        ("~", "~", "~")
    };
    let valid = |text: &str| parses(text, kind);
    let mut result: Vec<String> = Vec::new();
    if kind.is_3d() {
        if remainder.is_empty() {
            let full = format!("{x} {y} {z}");
            if valid(&full) {
                result.extend([x.to_owned(), format!("{x} {y}"), full]);
            }
        } else {
            let fields = java_split_spaces(&remainder);
            if fields.len() == 1 {
                let full = format!("{} {y} {z}", fields[0]);
                if valid(&full) {
                    result.push(format!("{} {y}", fields[0]));
                    result.push(full);
                }
            } else if fields.len() == 2 {
                let full = format!("{} {} {z}", fields[0], fields[1]);
                if valid(&full) {
                    result.push(full);
                }
            }
        }
    } else if remainder.is_empty() {
        let full = format!("{x} {z}");
        if valid(&full) {
            result.extend([x.to_owned(), full]);
        }
    } else {
        let fields = java_split_spaces(&remainder);
        if fields.len() == 1 {
            let full = format!("{} {z}", fields[0]);
            if valid(&full) {
                result.push(full);
            }
        }
    }
    suggest_matching(result.iter().map(String::as_str), builder);
}

/// `String.split(" ")`: trailing empty strings are dropped.
fn java_split_spaces(text: &str) -> Vec<&str> {
    let mut fields: Vec<&str> = text.split(' ').collect();
    while fields.last() == Some(&"") {
        fields.pop();
    }
    fields
}

/// `Commands.createValidator(this::parse)`: does the argument parse `text`?
fn parses(text: &str, kind: Kind) -> bool {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut reader = StringReader::new(&units, 0);
    if reader.peek() == Some(b'^' as u16) {
        return parse_local(&mut reader, kind);
    }
    let count = if kind.is_3d() { 3 } else { 2 };
    for index in 0..count {
        if index > 0 {
            if reader.peek() != Some(b' ' as u16) {
                return false; // ERROR_NOT_COMPLETE
            }
            reader.skip();
        }
        if !parse_world_coordinate(&mut reader, kind.integral()) {
            return false;
        }
    }
    true
}

/// `WorldCoordinate.parseDouble` / `parseInt`.
fn parse_world_coordinate(reader: &mut StringReader<'_>, integral: bool) -> bool {
    if reader.peek() == Some(b'^' as u16) || !reader.can_read(1) {
        return false; // mixed types / expected number
    }
    let relative = reader.peek() == Some(b'~' as u16);
    if relative {
        reader.skip();
    }
    if reader.can_read(1) && reader.peek() != Some(b' ' as u16) {
        if integral && !relative {
            return reader.read_int().is_some();
        }
        return reader.read_double().is_some();
    }
    true
}

/// `LocalCoordinates.parse`: exactly `^`-prefixed values for every axis
/// (`Vec3`/`BlockPos` take three, the 2D arguments reject local coordinates
/// through `WorldCoordinates`/`Vec2` parsing).
fn parse_local(reader: &mut StringReader<'_>, kind: Kind) -> bool {
    if !kind.is_3d() {
        return false;
    }
    for index in 0..3 {
        if index > 0 {
            if reader.peek() != Some(b' ' as u16) {
                return false;
            }
            reader.skip();
        }
        if reader.peek() != Some(b'^' as u16) {
            return false;
        }
        reader.skip();
        if reader.can_read(1)
            && reader.peek() != Some(b' ' as u16)
            && reader.read_double().is_none()
        {
            return false;
        }
    }
    true
}
