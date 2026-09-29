//! Port of `net.minecraft.util.datafix.fixes.LevelFlatGeneratorInfoFix`.

use crate::datafix::dynamic::{get_mut, get_str_or, parse_java_int};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::block_state_data::get_tag;
use super::entity_block_state_fix::get_block_id;

const DEFAULT: &str = "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village";

/// `new LevelFlatGeneratorInfoFix(schema, changesType)`: legacy flat world
/// generator options are rewritten with flattened block names.
pub fn fix() -> Fix {
    Fix::everywhere(
        "LevelFlatGeneratorInfoFix",
        Target::Type(r::LEVEL),
        |level| {
            if !get_str_or(level, "generatorName", "").eq_ignore_ascii_case("flat") {
                return;
            }
            if let Some(Tag::String(options)) = get_mut(level, "generatorOptions") {
                if let Some(fixed) = fix_string(options) {
                    *options = fixed;
                }
            }
        },
    )
}

/// Guava `Splitter.on(sep).limit(n).split(..)`: at most `n` parts, the last one
/// holding the rest.
fn split_limit(text: &str, separator: char, limit: usize) -> Vec<&str> {
    text.splitn(limit, separator).collect()
}

/// `NumberUtils.toInt(str, default)`.
fn to_int(text: &str, default: i32) -> i32 {
    parse_java_int(text).unwrap_or(default)
}

/// `LevelFlatGeneratorInfoFix.fixString`; `None` where the Java code would throw
/// (a layer without a block name).
pub fn fix_string(generator_options: &str) -> Option<String> {
    if generator_options.is_empty() {
        return Some(DEFAULT.to_string());
    }
    let mut parts = split_limit(generator_options, ';', 5).into_iter();
    let first_part = parts.next()?;
    let (version, layer_info) = match parts.next() {
        Some(layer_info) => (to_int(first_part, 0), layer_info),
        None => (0, first_part),
    };
    if !(0..=3).contains(&version) {
        return Some(DEFAULT.to_string());
    }
    let height_separator = if version < 3 { 'x' } else { '*' };
    let mut layers = Vec::new();
    for layer_string in layer_info.split(',') {
        let height_parts = split_limit(layer_string, height_separator, 2);
        let (height, layer_type) = if height_parts.len() == 2 {
            (to_int(height_parts[0], 0), height_parts[1])
        } else {
            (1, height_parts[0])
        };
        let layer_parts = split_limit(layer_type, ':', 3);
        let name_index = usize::from(layer_parts[0] == "minecraft");
        let block_string = layer_parts.get(name_index)?;
        let block_id = if version == 3 {
            get_block_id(&format!("minecraft:{block_string}"))
        } else {
            to_int(block_string, 0)
        };
        let data = layer_parts
            .get(name_index + 1)
            .map_or(0, |data| to_int(data, 0));
        let name = match get_tag((block_id << 4) | data) {
            tag @ Tag::Compound(_) => get_str_or(&tag, "Name", "").to_string(),
            _ => String::new(),
        };
        let prefix = if height == 1 {
            String::new()
        } else {
            format!("{height}*")
        };
        layers.push(format!("{prefix}{name}"));
    }
    let mut result = layers.join(",");
    for part in parts {
        result.push(';');
        result.push_str(part);
    }
    Some(result)
}
