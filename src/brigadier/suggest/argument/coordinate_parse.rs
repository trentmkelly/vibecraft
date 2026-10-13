//! Exact cursor-consuming coordinate grammars used by Minecraft's command
//! coordinate argument types.
//!
//! This mirrors `WorldCoordinate`, `WorldCoordinates`, and
//! `LocalCoordinates`, plus the small argument-specific wrappers. Parsing is
//! intentionally prefix based: a successful argument leaves any following
//! text for Brigadier to parse as the next node.

use super::StringReader;

/// Coordinate argument parser whose Java `parse` method is being mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    /// `Vec3Argument`: three world doubles or three local doubles.
    Vec3,
    /// `BlockPosArgument`: three world integers or three local doubles.
    BlockPos,
    /// `Vec2Argument`: two world doubles.
    Vec2,
    /// `ColumnPosArgument`: two world integers.
    ColumnPos,
    /// `RotationArgument`: two world doubles, parsed in Y then X order.
    Rotation,
}

/// Parse one coordinate argument and preserve the reader cursor as Java does.
///
/// A `false` result represents a thrown `CommandSyntaxException`; callers
/// only need the parser's success and cursor behavior for completion checks.
pub(super) fn parse(reader: &mut StringReader<'_>, kind: Kind) -> bool {
    match kind {
        Kind::Vec3 | Kind::BlockPos => {
            if reader.peek() == Some(b'^' as u16) {
                parse_local_coordinates(reader)
            } else {
                parse_world_coordinates(reader, 3, kind == Kind::BlockPos)
            }
        }
        Kind::Vec2 => parse_two_world_coordinates(reader, false),
        Kind::ColumnPos => parse_two_world_coordinates(reader, true),
        Kind::Rotation => parse_rotation(reader),
    }
}

/// `WorldCoordinates.parseInt` and `parseDouble`.
fn parse_world_coordinates(reader: &mut StringReader<'_>, count: usize, integral: bool) -> bool {
    let start = reader.cursor();
    for index in 0..count {
        if index > 0 {
            if reader.peek() != Some(b' ' as u16) {
                // The complete vector is required. Java resets the reader to
                // the beginning when a separating space is absent. A missing
                // scalar after a present separator keeps its own error cursor.
                reader.set_cursor(start);
                return false;
            }
            reader.skip();
        }

        if !parse_world_coordinate(reader, integral) {
            return false;
        }
    }
    true
}

/// `WorldCoordinate.parseDouble` / `parseInt` grammar and cursor behavior.
fn parse_world_coordinate(reader: &mut StringReader<'_>, integral: bool) -> bool {
    if reader.peek() == Some(b'^' as u16) || !reader.can_read(1) {
        return false;
    }

    let relative = reader.peek() == Some(b'~' as u16);
    if relative {
        reader.skip();
    }

    if reader.can_read(1) && reader.peek() != Some(b' ' as u16) {
        if integral && !relative {
            reader.read_int().is_some()
        } else {
            reader.read_double().is_some()
        }
    } else {
        // Bare `~` and coordinates immediately followed by a separator are
        // zero-valued coordinates in the Java parser.
        true
    }
}

/// `Vec2Argument` and `ColumnPosArgument` wrappers around two world values.
fn parse_two_world_coordinates(reader: &mut StringReader<'_>, integral: bool) -> bool {
    let start = reader.cursor();
    if !reader.can_read(1) {
        return false;
    }
    if !parse_world_coordinate(reader, integral) {
        return false;
    }
    if reader.peek() != Some(b' ' as u16) {
        // The 2-D argument wrappers restore the initial cursor only when the
        // required second-coordinate separator is absent.
        reader.set_cursor(start);
        return false;
    }
    reader.skip();
    parse_world_coordinate(reader, integral)
}

/// `RotationArgument.parse`: yaw (Y) followed by pitch (X), both doubles.
fn parse_rotation(reader: &mut StringReader<'_>) -> bool {
    let start = reader.cursor();
    if !reader.can_read(1) {
        return false;
    }
    if !parse_world_coordinate(reader, false) {
        return false;
    }
    if reader.peek() != Some(b' ' as u16) {
        reader.set_cursor(start);
        return false;
    }
    reader.skip();
    parse_world_coordinate(reader, false)
}

/// `LocalCoordinates.parse`.
fn parse_local_coordinates(reader: &mut StringReader<'_>) -> bool {
    let start = reader.cursor();
    for index in 0..3 {
        if index > 0 {
            if reader.peek() != Some(b' ' as u16) {
                reader.set_cursor(start);
                return false;
            }
            reader.skip();
        }

        if !reader.can_read(1) {
            return false;
        }
        if reader.peek() != Some(b'^' as u16) {
            // LocalCoordinates resets to the full vector's start when an axis
            // lacks the local prefix, reporting a mixed-coordinate error.
            reader.set_cursor(start);
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

#[cfg(test)]
mod tests {
    use super::{parse, Kind};
    use crate::brigadier::suggest::argument::StringReader;

    fn parse_at(text: &str, cursor: usize, kind: Kind) -> (bool, usize) {
        let input: Vec<u16> = text.encode_utf16().collect();
        let mut reader = StringReader::new(&input, cursor);
        let success = parse(&mut reader, kind);
        (success, reader.cursor())
    }

    #[test]
    fn world_coordinates_leave_suffix_for_the_next_argument() {
        assert_eq!(parse_at("go 1 2 3tail", 3, Kind::Vec3), (true, 8));
        assert_eq!(parse_at("go ~ ~ ~3tail", 3, Kind::BlockPos), (true, 9));
        assert_eq!(parse_at("go 1 2", 3, Kind::Vec3), (false, 3));
        assert_eq!(parse_at("go 1 2 ", 3, Kind::Vec3), (false, 7));
    }

    #[test]
    fn coordinate_kinds_apply_their_world_number_grammar() {
        assert_eq!(parse_at("~1.5 ~ ~-2.25", 0, Kind::BlockPos), (true, 13));
        assert_eq!(parse_at("1.5 2 3", 0, Kind::BlockPos), (false, 0));
        assert_eq!(parse_at("1.5 ~-2", 0, Kind::Vec2), (true, 7));
        assert_eq!(parse_at("1.5 2", 0, Kind::ColumnPos), (false, 0));
        assert_eq!(parse_at("~-5 ~5tail", 0, Kind::Rotation), (true, 6));
    }

    #[test]
    fn local_coordinates_require_three_consistent_axes() {
        assert_eq!(parse_at("^ ^1 ^-5tail", 0, Kind::Vec3), (true, 8));
        assert_eq!(parse_at("^ ^ ^", 0, Kind::BlockPos), (true, 5));
        assert_eq!(parse_at("^ ^ ~", 0, Kind::Vec3), (false, 0));
        assert_eq!(parse_at("^ ^", 0, Kind::Vec3), (false, 0));
        assert_eq!(parse_at("^ ^ ", 0, Kind::Vec3), (false, 4));
        assert_eq!(parse_at("^ ^", 0, Kind::Vec2), (false, 0));
    }

    #[test]
    fn local_number_errors_keep_the_cursor_after_the_prefix() {
        assert_eq!(parse_at("^1 ^- ^", 0, Kind::Vec3), (false, 4));
        assert_eq!(parse_at("^ ^ ^", 2, Kind::Vec3), (false, 2));
    }
}
