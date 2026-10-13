//! Lexical parsers for Minecraft's small, stateless command arguments.
//!
//! The command tree can tell us the parser id and the input text, but it does
//! not carry live registry contents. Resource keys and function identifiers
//! therefore receive only the syntax checks performed by `Identifier.read`;
//! registry-backed existence checks are intentionally outside this module.

use super::StringReader;

pub(super) const HEIGHTMAPS: [&str; 4] = [
    "world_surface",
    "ocean_floor",
    "motion_blocking",
    "motion_blocking_no_leaves",
];

pub(super) const OPERATIONS: [&str; 9] = ["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"];

pub(super) const HEX_COLOR_EXAMPLES: [&str; 2] = ["F00", "FF0000"];

/// Parse a scalar Minecraft argument by the exact token grammar used by its
/// Java `ArgumentType`. `None` means this module does not own `parser`.
pub(super) fn parse(parser: &str, reader: &mut StringReader<'_>) -> Option<bool> {
    let parsed = match parser {
        "minecraft:resource_location" | "minecraft:resource_key" => parse_identifier(reader, false),
        "minecraft:resource_or_tag_key" => parse_identifier(reader, true),
        "minecraft:function" => parse_function(reader),
        "minecraft:uuid" => parse_uuid(reader),
        "minecraft:angle" => parse_angle(reader),
        "minecraft:gamemode" => parse_game_mode(reader),
        "minecraft:entity_anchor" => parse_entity_anchor(reader),
        "minecraft:heightmap" => parse_heightmap(reader),
        "minecraft:hex_color" => parse_hex_color(reader),
        "minecraft:operation" => parse_operation(reader),
        _ => return None,
    };
    Some(parsed)
}

/// Implements `Identifier.read`: consume the allowed identifier run, then
/// validate its namespace and path. The Java reader restores the start cursor
/// on an invalid identifier. For resource-or-tag keys, `#` is part of the
/// grammar and the reset includes that marker.
fn parse_identifier(reader: &mut StringReader<'_>, allow_tag: bool) -> bool {
    let start = reader.cursor();
    if allow_tag && reader.peek() == Some(b'#' as u16) {
        reader.skip();
    }
    let identifier_start = reader.cursor();
    while reader.peek().is_some_and(is_identifier_character) {
        reader.skip();
    }

    let token = String::from_utf16_lossy(&reader.input[identifier_start..reader.cursor()]);
    let valid = identifier_is_valid(&token);
    if !valid {
        reader.set_cursor(start);
    }
    valid
}

fn is_identifier_character(unit: u16) -> bool {
    matches!(unit, 0x30..=0x39 | 0x61..=0x7a)
        || unit == b'_' as u16
        || unit == b':' as u16
        || unit == b'/' as u16
        || unit == b'.' as u16
        || unit == b'-' as u16
}

fn identifier_is_valid(identifier: &str) -> bool {
    let (namespace, path) = match identifier.split_once(':') {
        Some((namespace, path)) => (namespace, path),
        None => ("minecraft", identifier),
    };
    namespace != ".."
        && namespace.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
        })
        && path.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.' | b'/')
        })
}

fn parse_function(reader: &mut StringReader<'_>) -> bool {
    if reader.peek() == Some(b'#' as u16) {
        reader.skip();
    }
    // FunctionArgument delegates both function and tag forms to Identifier.read.
    // For a bad id Identifier.read restores to immediately after '#', matching
    // the enclosing ArgumentType's exception behavior.
    parse_identifier(reader, false)
}

fn parse_uuid(reader: &mut StringReader<'_>) -> bool {
    let start = reader.cursor();
    while reader.peek().is_some_and(is_uuid_token_character) {
        reader.skip();
    }
    let token = String::from_utf16_lossy(&reader.input[start..reader.cursor()]);
    if !uuid_from_string_compatible(&token) {
        reader.set_cursor(start);
        return false;
    }
    true
}

fn is_uuid_token_character(unit: u16) -> bool {
    unit == b'-' as u16 || matches!(unit, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
}

/// Java's `UUID.fromString` accepts noncanonical component widths. Its fallback
/// parser permits at most 36 characters, splits into five nonempty groups, and
/// parses each with signed-positive `Long.parseLong(..., 16)` before masking
/// and shifting. This is intentionally more permissive than the canonical
/// 8-4-4-4-12 spelling but rejects a field whose value exceeds `i64::MAX`.
fn uuid_from_string_compatible(token: &str) -> bool {
    if token.len() > 36 {
        return false;
    }
    token.split('-').count() == 5
        && token
            .split('-')
            .all(|group| !group.is_empty() && i64::from_str_radix(group, 16).is_ok())
}

fn parse_angle(reader: &mut StringReader<'_>) -> bool {
    if !reader.can_read(1) {
        return false;
    }
    if reader.peek() == Some(b'~' as u16) {
        reader.skip();
    }

    if reader.can_read(1) && reader.peek() != Some(b' ' as u16) {
        return reader.read_float().is_some_and(f32::is_finite);
    }
    true
}

fn parse_game_mode(reader: &mut StringReader<'_>) -> bool {
    let name = reader.read_unquoted_string();
    matches!(
        name.as_str(),
        "survival" | "creative" | "adventure" | "spectator"
    )
}

fn parse_entity_anchor(reader: &mut StringReader<'_>) -> bool {
    matches!(reader.read_unquoted_string().as_str(), "feet" | "eyes")
}

fn parse_heightmap(reader: &mut StringReader<'_>) -> bool {
    let name = reader.read_unquoted_string();
    HEIGHTMAPS.contains(&name.as_str())
}

fn parse_hex_color(reader: &mut StringReader<'_>) -> bool {
    let value = reader.read_unquoted_string();
    match value.as_bytes() {
        [a, b, c] => is_hex_digit(*a) && is_hex_digit(*b) && is_hex_digit(*c),
        [r1, r2, g1, g2, b1, b2] => {
            java_hex_byte(*r1, *r2) && java_hex_byte(*g1, *g2) && java_hex_byte(*b1, *b2)
        }
        _ => false,
    }
}

fn is_hex_digit(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}

/// `Integer.parseInt(chunk, 16)` accepts an optional sign in a two-character
/// channel, as long as at least one radix-16 digit follows it.
fn java_hex_byte(sign_or_digit: u8, digit: u8) -> bool {
    if sign_or_digit == b'+' || sign_or_digit == b'-' {
        is_hex_digit(digit)
    } else {
        is_hex_digit(sign_or_digit) && is_hex_digit(digit)
    }
}

fn parse_operation(reader: &mut StringReader<'_>) -> bool {
    let start = reader.cursor();
    while reader.can_read(1) && reader.peek() != Some(b' ' as u16) {
        reader.skip();
    }
    let operation = String::from_utf16_lossy(&reader.input[start..reader.cursor()]);
    OPERATIONS.contains(&operation.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(parser: &str, input: &str) -> (bool, usize) {
        let units: Vec<u16> = input.encode_utf16().collect();
        let mut reader = StringReader::new(&units, 0);
        (parse(parser, &mut reader).unwrap(), reader.cursor())
    }

    #[test]
    fn identifiers_validate_java_namespace_and_path_grammar() {
        assert_eq!(run("minecraft:resource_location", "stone rest"), (true, 5));
        assert_eq!(
            run("minecraft:resource_location", "mod:path/file rest"),
            (true, 13)
        );
        // Identifier.read accepts the empty token as the empty default-path id;
        // a later parser/command validation step reports the unread suffix.
        assert_eq!(run("minecraft:resource_location", "UPPER"), (true, 0));
        assert_eq!(
            run("minecraft:resource_location", "mod:bad:path"),
            (false, 0)
        );
        assert_eq!(
            run("minecraft:resource_or_tag_key", "#mod:tag rest"),
            (true, 8)
        );
        assert_eq!(
            run("minecraft:resource_or_tag_key", "#bad:path:more"),
            (false, 0)
        );
    }

    #[test]
    fn function_parses_optional_tag_prefix_and_identifier() {
        assert_eq!(run("minecraft:function", "mod:tick rest"), (true, 8));
        assert_eq!(run("minecraft:function", "#mod:tick rest"), (true, 9));
        assert_eq!(run("minecraft:function", "#Bad:tick"), (true, 1));
        assert_eq!(run("minecraft:function", "#bad:path:more"), (false, 1));
    }

    #[test]
    fn uuid_accepts_java_short_components_and_only_consumes_hex_prefix() {
        assert_eq!(run("minecraft:uuid", "1-1-1-1-1 rest"), (true, 9));
        assert_eq!(
            run(
                "minecraft:uuid",
                "dd12be42-52a9-4a91-a8a1-11c01849e498!tail"
            ),
            (true, 36)
        );
        assert_eq!(run("minecraft:uuid", "1-1-1-1"), (false, 0));
        assert_eq!(run("minecraft:uuid", "1-1-1-1-1-1"), (false, 0));
        assert_eq!(
            run("minecraft:uuid", "8000000000000000-1-1-1-1"),
            (false, 0)
        );
        assert_eq!(
            run("minecraft:uuid", "1-1-1-1-1111111111111111111111111111111"),
            (false, 0)
        );
    }

    #[test]
    fn angle_and_enum_arguments_follow_reader_cursor_rules() {
        assert_eq!(run("minecraft:angle", "~ -5 rest"), (true, 1));
        assert_eq!(run("minecraft:angle", "1e2"), (true, 1));
        assert_eq!(run("minecraft:gamemode", "creative!"), (true, 8));
        assert_eq!(run("minecraft:entity_anchor", "eyes rest"), (true, 4));
        assert_eq!(run("minecraft:heightmap", "world_surface rest"), (true, 13));
        assert_eq!(
            run("minecraft:heightmap", "WORLD_SURFACE rest"),
            (false, 13)
        );
    }

    #[test]
    fn hex_color_and_scoreboard_operation_use_exact_tokens() {
        assert_eq!(run("minecraft:hex_color", "F00 rest"), (true, 3));
        assert_eq!(run("minecraft:hex_color", "FF-1+0"), (true, 6));
        assert_eq!(run("minecraft:hex_color", "GGG"), (false, 3));
        assert_eq!(run("minecraft:operation", ">< rest"), (true, 2));
        assert_eq!(run("minecraft:operation", "+=tail"), (false, 6));
    }

    #[test]
    fn unknown_parser_is_left_for_another_module() {
        let units = "value".encode_utf16().collect::<Vec<_>>();
        let mut reader = StringReader::new(&units, 0);
        assert_eq!(parse("minecraft:unknown", &mut reader), None);
        assert_eq!(reader.cursor(), 0);
    }
}
