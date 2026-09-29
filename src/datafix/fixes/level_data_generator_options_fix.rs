//! Port of `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix`: the
//! legacy string `generatorOptions` of flat and buffet worlds become structured
//! data.

use std::collections::HashMap;

use crate::datafix::dynamic::{compound, get_str, get_str_or, parse_java_int, set};
use crate::datafix::fix::Fix;
use crate::datafix::json::parse_lenient;
use crate::datafix::json_ops::json_to_nbt;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::level_data_generator_options_fix_map::MAP;
use super::write_and_read_fix::write_fix_and_read_fix;

/// `new LevelDataGeneratorOptionsFix(schema, changesType)`.
///
/// Where the Java code throws (unparseable buffet JSON) the level is left as it
/// was.
pub fn fix() -> Fix {
    write_fix_and_read_fix("LevelDataGeneratorOptionsFix", r::LEVEL, |level| {
        let options = get_str(level, "generatorOptions").map(str::to_string);
        let generator = get_str_or(level, "generatorName", "");
        if generator.eq_ignore_ascii_case("flat") {
            let converted = convert(options.as_deref().unwrap_or(""));
            set(level, "generatorOptions", converted);
        } else if generator.eq_ignore_ascii_case("buffet") {
            if let Some(Ok(json)) = options.as_deref().map(parse_lenient) {
                set(level, "generatorOptions", json_to_nbt(&json));
            }
        }
    })
}

/// `String.split(regex)` for a literal separator: trailing empty strings are
/// removed, unless the separator does not occur at all.
fn java_split(text: &str, separator: char) -> Vec<&str> {
    let mut parts: Vec<&str> = text.split(separator).collect();
    if parts.len() > 1 {
        while parts.last() == Some(&"") {
            parts.pop();
        }
    }
    parts
}

/// `getLayerInfoFromString`: `None` for an unparseable height.
fn layer_info(input: &str) -> Option<(i32, String)> {
    let parts: Vec<&str> = input.splitn(2, '*').collect();
    let height = if parts.len() == 2 {
        parse_java_int(parts[0])?
    } else {
        1
    };
    Some((height, parts[parts.len() - 1].to_string()))
}

/// `getLayersInfoFromString`: empty when any layer is invalid.
fn layers_info(input: &str) -> Vec<(i32, String)> {
    let mut result = Vec::new();
    for depth in java_split(input, ',') {
        match layer_info(depth) {
            Some(layer) => result.push(layer),
            None => return Vec::new(),
        }
    }
    result
}

/// Structure name to its options.
type Structures = HashMap<String, HashMap<String, String>>;

/// The `structures` part of a flat options string.
fn parse_structures(text: &str) -> Structures {
    let mut structures = Structures::new();
    for structure in java_split(&text.to_lowercase(), ',') {
        let separated: Vec<&str> = structure.splitn(2, '(').collect();
        if separated[0].is_empty() {
            continue;
        }
        let mut options = HashMap::new();
        if separated.len() > 1 && separated[1].ends_with(')') && separated[1].len() > 1 {
            let inner = &separated[1][..separated[1].len() - 1];
            for part in java_split(inner, ' ') {
                let split: Vec<&str> = part.splitn(2, '=').collect();
                if split.len() == 2 {
                    options.insert(split[0].to_string(), split[1].to_string());
                }
            }
        }
        structures.insert(separated[0].to_string(), options);
    }
    structures
}

/// `convert(flatOptionString, ops)`.
fn convert(flat_options: &str) -> Tag {
    let mut biome = "minecraft:plains".to_string();
    let mut structures = Structures::new();
    let layers;
    if flat_options.is_empty() {
        layers = vec![
            (1, "minecraft:bedrock".to_string()),
            (2, "minecraft:dirt".to_string()),
            (1, "minecraft:grass_block".to_string()),
        ];
        structures.insert("village".to_string(), HashMap::new());
    } else {
        let mut parts = flat_options.split(';');
        layers = layers_info(parts.next().unwrap_or(""));
        if !layers.is_empty() {
            if let Some(biome_id) = parts.next() {
                biome = MAP
                    .iter()
                    .find(|(id, _)| *id == biome_id)
                    .map_or("minecraft:plains", |(_, name)| *name)
                    .to_string();
            }
            match parts.next() {
                Some(text) => structures = parse_structures(text),
                None => {
                    structures.insert("village".to_string(), HashMap::new());
                }
            }
        }
    }
    let layers = layers
        .into_iter()
        .map(|(height, block)| {
            compound(vec![
                ("height", Tag::Int(height)),
                ("block", Tag::String(block)),
            ])
        })
        .collect();
    let structures = structures
        .into_iter()
        .map(|(name, options)| {
            let options = options
                .into_iter()
                .map(|(key, value)| (key, Tag::String(value)))
                .collect();
            (name, Tag::Compound(options))
        })
        .collect();
    compound(vec![
        ("layers", Tag::List(layers)),
        ("biome", Tag::String(biome)),
        ("structures", Tag::Compound(structures)),
    ])
}
