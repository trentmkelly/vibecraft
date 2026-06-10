#![allow(dead_code)]

use crate::chat_component::Component;
use crate::storage::datafix::TARGET_DATA_VERSION;

use super::compound_tag::{compound_get, put_compound_entry, remove_compound_entry};
use super::snbt_printer;
use super::tag_parser::{parse_compound_fully, TagParserError};
use super::text_component_tag_visitor;
use super::Tag;

pub const SNBT_DATA_TAG: &str = "data";

// TODO(nbt-utils-block-state-registry): Port Java readBlockState/writeBlockState
// and writeFluidState once RustCraft has a canonical block/fluid registry-backed
// state model equivalent to HolderGetter<Block>, StateDefinition, and
// BuiltInRegistries.
// TODO(nbt-utils-dynamic-ops): Port the Dynamic<T>/ValueOutput overloads once
// NbtOps and the DFU Dynamic surface are complete.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedBlockState {
    pub name: String,
    pub properties: Vec<(String, String)>,
}

pub fn compare_nbt(
    expected: Option<&Tag>,
    actual: Option<&Tag>,
    partial_list_matches: bool,
) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    let Some(actual) = actual else {
        return false;
    };

    match (expected, actual) {
        (Tag::Compound(expected_values), Tag::Compound(actual_values)) => {
            if actual_values.len() < expected_values.len() {
                return false;
            }
            expected_values.iter().all(|(key, expected_tag)| {
                compare_nbt(
                    Some(expected_tag),
                    compound_get(actual_values, key),
                    partial_list_matches,
                )
            })
        }
        (Tag::List(expected_values), Tag::List(actual_values)) if partial_list_matches => {
            if expected_values.is_empty() {
                return actual_values.is_empty();
            }
            actual_values.len() >= expected_values.len()
                && expected_values.iter().all(|expected_tag| {
                    actual_values.iter().any(|actual_tag| {
                        compare_nbt(Some(expected_tag), Some(actual_tag), partial_list_matches)
                    })
                })
        }
        _ if std::mem::discriminant(expected) == std::mem::discriminant(actual) => {
            expected == actual
        }
        _ => false,
    }
}

pub fn pretty_print(tag: &Tag, with_binary_blobs: bool) -> String {
    let mut out = String::new();
    pretty_print_into(&mut out, tag, 0, with_binary_blobs);
    out
}

pub fn pretty_print_into(
    builder: &mut String,
    input: &Tag,
    indent_level: usize,
    with_binary_blobs: bool,
) {
    match input {
        Tag::End => {}
        Tag::Byte(_)
        | Tag::Short(_)
        | Tag::Int(_)
        | Tag::Long(_)
        | Tag::Float(_)
        | Tag::Double(_)
        | Tag::String(_) => builder.push_str(&input.to_snbt()),
        Tag::ByteArray(values) => {
            pretty_print_byte_array(builder, values, indent_level, with_binary_blobs)
        }
        Tag::IntArray(values) => {
            pretty_print_int_array(builder, values, indent_level, with_binary_blobs)
        }
        Tag::LongArray(values) => {
            pretty_print_long_array(builder, values, indent_level, with_binary_blobs)
        }
        Tag::List(values) => pretty_print_list(builder, values, indent_level, with_binary_blobs),
        Tag::Compound(values) => {
            pretty_print_compound(builder, values, indent_level, with_binary_blobs)
        }
    }
}

pub fn to_pretty_component(tag: &Tag) -> Component {
    text_component_tag_visitor::to_rich_text_component(tag, "", true)
}

pub fn structure_to_snbt(structure: &Tag) -> Result<String, String> {
    Ok(snbt_printer::to_pretty_snbt(&pack_structure_template(
        structure.clone(),
    )?))
}

pub fn snbt_to_structure(snbt: &str) -> Result<Tag, TagParserError> {
    parse_compound_fully(snbt).map(|tag| match unpack_structure_template(tag.clone()) {
        Ok(unpacked) => unpacked,
        Err(_) => tag,
    })
}

pub fn pack_structure_template(mut snbt: Tag) -> Result<Tag, String> {
    let Tag::Compound(values) = &mut snbt else {
        return Err("Expected structure compound".to_string());
    };

    let old_palettes = get_list(values, "palettes").map(<[_]>::to_vec);
    let palette = if let Some(palettes) = &old_palettes {
        palettes
            .first()
            .and_then(Tag::as_list)
            .unwrap_or(&[])
            .to_vec()
    } else {
        get_list(values, "palette")
            .map(<[_]>::to_vec)
            .unwrap_or_default()
    };
    let deflated_palette = palette
        .iter()
        .map(pack_block_state_tag)
        .collect::<Result<Vec<_>, _>>()?;
    let deflated_palette_tags = deflated_palette
        .iter()
        .cloned()
        .map(Tag::String)
        .collect::<Vec<_>>();
    put_compound_entry(values, "palette", Tag::List(deflated_palette_tags.clone()));

    if let Some(palettes) = old_palettes {
        let mut new_palettes = Vec::new();
        for old_palette in palettes.iter().filter_map(Tag::as_list) {
            let mut new_palette = Vec::new();
            for (index, old_state) in old_palette.iter().enumerate() {
                let key = deflated_palette.get(index).ok_or_else(|| {
                    format!("Palette index {index} missing from deflated palette")
                })?;
                put_compound_entry(
                    &mut new_palette,
                    key.clone(),
                    Tag::String(pack_block_state_tag(old_state)?),
                );
            }
            new_palettes.push(Tag::Compound(new_palette));
        }
        put_compound_entry(values, "palettes", Tag::List(new_palettes));
    }

    if let Some(entities) = get_list(values, "entities").map(<[_]>::to_vec) {
        let mut entity_compounds = entities
            .into_iter()
            .filter(|tag| matches!(tag, Tag::Compound(_)))
            .collect::<Vec<_>>();
        entity_compounds.sort_by(compare_entity_pos_yxz);
        put_compound_entry(values, "entities", Tag::List(entity_compounds));
    }

    let mut blocks = get_list(values, "blocks")
        .map(<[_]>::to_vec)
        .unwrap_or_default()
        .into_iter()
        .filter(|tag| matches!(tag, Tag::Compound(_)))
        .collect::<Vec<_>>();
    blocks.sort_by(compare_block_pos_yxz);
    for block in &mut blocks {
        let state_id = compound_entries(block)
            .and_then(|entries| compound_get(entries, "state"))
            .and_then(Tag::as_int)
            .unwrap_or(0);
        let state = deflated_palette
            .get(state_id.max(0) as usize)
            .cloned()
            .ok_or_else(|| format!("Palette state index {state_id} missing"))?;
        if let Some(entries) = compound_entries_mut(block) {
            put_compound_entry(entries, "state", Tag::String(state));
        }
    }
    put_compound_entry(values, SNBT_DATA_TAG, Tag::List(blocks));
    remove_compound_entry(values, "blocks");

    Ok(snbt)
}

pub fn unpack_structure_template(mut template: Tag) -> Result<Tag, String> {
    let Tag::Compound(values) = &mut template else {
        return Err("Expected structure compound".to_string());
    };

    let packed_palette = get_list(values, "palette")
        .map(<[_]>::to_vec)
        .unwrap_or_default();
    let mut palette = Vec::new();
    for tag in &packed_palette {
        let key = tag
            .as_string()
            .ok_or_else(|| "Packed palette entry must be a string".to_string())?;
        palette.push((key.to_string(), unpack_block_state(key)));
    }

    if let Some(old_palettes) = get_list(values, "palettes").map(<[_]>::to_vec) {
        let mut new_palettes = Vec::new();
        for old_palette in old_palettes.iter().filter_map(Tag::as_compound) {
            let mut new_palette = Vec::new();
            for (key, _) in &palette {
                let state_name = compound_get(old_palette, key)
                    .and_then(Tag::as_string)
                    .ok_or_else(|| format!("Palette key {key} missing"))?;
                new_palette.push(unpack_block_state(state_name));
            }
            new_palettes.push(Tag::List(new_palette));
        }
        put_compound_entry(values, "palettes", Tag::List(new_palettes));
        remove_compound_entry(values, "palette");
    } else {
        put_compound_entry(
            values,
            "palette",
            Tag::List(palette.iter().map(|(_, tag)| tag.clone()).collect()),
        );
    }

    if let Some(data) = get_list(values, SNBT_DATA_TAG).map(<[_]>::to_vec) {
        let mut blocks = data;
        for block in &mut blocks {
            let state_name = compound_entries(block)
                .and_then(|entries| compound_get(entries, "state"))
                .and_then(Tag::as_string)
                .ok_or_else(|| "Block state string missing".to_string())?;
            let state_id = palette
                .iter()
                .position(|(key, _)| key == state_name)
                .ok_or_else(|| format!("Entry {state_name} missing from palette"))?;
            if let Some(entries) = compound_entries_mut(block) {
                put_compound_entry(entries, "state", Tag::Int(state_id as i32));
            }
        }
        put_compound_entry(values, "blocks", Tag::List(blocks));
        remove_compound_entry(values, SNBT_DATA_TAG);
    }

    Ok(template)
}

pub fn pack_block_state(state: &PackedBlockState) -> String {
    let mut out = state.name.clone();
    if !state.properties.is_empty() {
        let mut properties = state.properties.clone();
        properties.sort_by(|left, right| left.0.cmp(&right.0));
        out.push('{');
        out.push_str(
            &properties
                .iter()
                .map(|(key, value)| format!("{key}:{value}"))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('}');
    }
    out
}

pub fn unpack_block_state(input: &str) -> Tag {
    let state = unpack_block_state_model(input);
    let mut entries = Vec::new();
    if !state.properties.is_empty() {
        let mut properties = Vec::new();
        for (key, value) in state.properties {
            put_compound_entry(&mut properties, key, Tag::String(value));
        }
        put_compound_entry(&mut entries, "Properties", Tag::Compound(properties));
    }
    put_compound_entry(&mut entries, "Name", Tag::String(state.name));
    Tag::Compound(entries)
}

pub fn unpack_block_state_model(input: &str) -> PackedBlockState {
    let Some(open_index) = input.find('{') else {
        return PackedBlockState {
            name: input.to_string(),
            properties: Vec::new(),
        };
    };
    let name = input[..open_index].to_string();
    let mut properties = Vec::new();
    if open_index + 2 <= input.len() {
        let close_index = input[open_index..]
            .find('}')
            .map(|index| open_index + index)
            .unwrap_or(input.len());
        for key_value in input[open_index + 1..close_index].split(',') {
            let mut parts = key_value.splitn(2, ':');
            let Some(key) = parts.next() else {
                continue;
            };
            let Some(value) = parts.next() else {
                continue;
            };
            properties.push((key.to_string(), value.to_string()));
        }
    }
    PackedBlockState { name, properties }
}

pub fn pack_block_state_tag(tag: &Tag) -> Result<String, String> {
    let entries = tag
        .as_compound()
        .ok_or_else(|| "Block state must be a compound".to_string())?;
    let name = compound_get(entries, "Name")
        .and_then(Tag::as_string)
        .ok_or_else(|| "Block state missing Name".to_string())?;
    let properties = compound_get(entries, "Properties")
        .and_then(Tag::as_compound)
        .map(|properties| {
            properties
                .iter()
                .map(|(key, value)| {
                    value
                        .as_string()
                        .map(|value| (key.clone(), value.to_string()))
                        .ok_or_else(|| format!("Property {key} must be a string"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(pack_block_state(&PackedBlockState {
        name: name.to_string(),
        properties,
    }))
}

pub fn add_current_data_version(tag: &mut Tag) -> bool {
    add_data_version(tag, TARGET_DATA_VERSION)
}

pub fn add_data_version(tag: &mut Tag, version: i32) -> bool {
    match tag {
        Tag::Compound(entries) => {
            put_compound_entry(entries, "DataVersion", Tag::Int(version));
            true
        }
        _ => false,
    }
}

pub fn get_data_version(tag: &Tag) -> i32 {
    get_data_version_or(tag, -1)
}

pub fn get_data_version_or(tag: &Tag, default_value: i32) -> i32 {
    tag.compound_get_int_or("DataVersion", default_value)
}

fn pretty_print_byte_array(
    builder: &mut String,
    values: &[i8],
    indent_level: usize,
    with_binary_blobs: bool,
) {
    indent(indent_level, builder);
    builder.push_str(&format!("byte[{}] {{\n", values.len()));
    if with_binary_blobs {
        indent(indent_level + 1, builder);
        for (index, value) in values.iter().enumerate() {
            append_array_separator(builder, index, indent_level);
            builder.push_str(&format!("0x{:02X}", *value as u8));
        }
    } else {
        indent(indent_level + 1, builder);
        builder.push_str(" // Skipped, supply withBinaryBlobs true");
    }
    builder.push('\n');
    indent(indent_level, builder);
    builder.push('}');
}

fn pretty_print_int_array(
    builder: &mut String,
    values: &[i32],
    indent_level: usize,
    with_binary_blobs: bool,
) {
    let width = values
        .iter()
        .map(|value| format!("{:X}", *value as u32).len())
        .max()
        .unwrap_or(0);
    indent(indent_level, builder);
    builder.push_str(&format!("int[{}] {{\n", values.len()));
    if with_binary_blobs {
        indent(indent_level + 1, builder);
        for (index, value) in values.iter().enumerate() {
            append_array_separator(builder, index, indent_level);
            builder.push_str(&format!("0x{:0width$X}", *value as u32, width = width));
        }
    } else {
        indent(indent_level + 1, builder);
        builder.push_str(" // Skipped, supply withBinaryBlobs true");
    }
    builder.push('\n');
    indent(indent_level, builder);
    builder.push('}');
}

fn pretty_print_long_array(
    builder: &mut String,
    values: &[i64],
    indent_level: usize,
    with_binary_blobs: bool,
) {
    let width = values
        .iter()
        .map(|value| format!("{:X}", *value as u64).len())
        .max()
        .unwrap_or(0);
    indent(indent_level, builder);
    builder.push_str(&format!("long[{}] {{\n", values.len()));
    if with_binary_blobs {
        indent(indent_level + 1, builder);
        for (index, value) in values.iter().enumerate() {
            append_array_separator(builder, index, indent_level);
            builder.push_str(&format!("0x{:0width$X}", *value as u64, width = width));
        }
    } else {
        indent(indent_level + 1, builder);
        builder.push_str(" // Skipped, supply withBinaryBlobs true");
    }
    builder.push('\n');
    indent(indent_level, builder);
    builder.push('}');
}

fn pretty_print_list(
    builder: &mut String,
    values: &[Tag],
    indent_level: usize,
    with_binary_blobs: bool,
) {
    indent(indent_level, builder);
    builder.push_str(&format!("list[{}] [", values.len()));
    if !values.is_empty() {
        builder.push('\n');
    }
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            builder.push_str(",\n");
        }
        indent(indent_level + 1, builder);
        pretty_print_into(builder, value, indent_level + 1, with_binary_blobs);
    }
    if !values.is_empty() {
        builder.push('\n');
    }
    indent(indent_level, builder);
    builder.push(']');
}

fn pretty_print_compound(
    builder: &mut String,
    values: &[(String, Tag)],
    indent_level: usize,
    with_binary_blobs: bool,
) {
    let mut entries = values.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    indent(indent_level, builder);
    builder.push('{');
    if builder
        .len()
        .saturating_sub(builder.rfind('\n').map_or(0, |index| index + 1))
        > 2 * (indent_level + 1)
    {
        builder.push('\n');
        indent(indent_level + 1, builder);
    }
    let padding_len = entries.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
    for (index, (key, value)) in entries.iter().enumerate() {
        if index != 0 {
            builder.push_str(",\n");
        }
        indent(indent_level + 1, builder);
        builder.push('"');
        builder.push_str(key);
        builder.push('"');
        for _ in 0..padding_len.saturating_sub(key.len()) {
            builder.push(' ');
        }
        builder.push_str(": ");
        pretty_print_into(builder, value, indent_level + 1, with_binary_blobs);
    }
    if !entries.is_empty() {
        builder.push('\n');
    }
    indent(indent_level, builder);
    builder.push('}');
}

fn indent(indent_level: usize, builder: &mut String) {
    let line_start = builder.rfind('\n').map_or(0, |index| index + 1);
    let line_len = builder.len().saturating_sub(line_start);
    for _ in line_len..(2 * indent_level) {
        builder.push(' ');
    }
}

fn append_array_separator(builder: &mut String, index: usize, indent_level: usize) {
    if index != 0 {
        builder.push(',');
    }
    if index.is_multiple_of(16) && index / 16 > 0 {
        builder.push('\n');
        indent(indent_level + 1, builder);
    } else if index != 0 {
        builder.push(' ');
    }
}

fn get_list<'a>(entries: &'a [(String, Tag)], name: &str) -> Option<&'a [Tag]> {
    compound_get(entries, name).and_then(Tag::as_list)
}

fn compound_entries(tag: &Tag) -> Option<&[(String, Tag)]> {
    tag.as_compound()
}

fn compound_entries_mut(tag: &mut Tag) -> Option<&mut Vec<(String, Tag)>> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn compare_block_pos_yxz(left: &Tag, right: &Tag) -> std::cmp::Ordering {
    position_ints(left).cmp(&position_ints(right))
}

fn compare_entity_pos_yxz(left: &Tag, right: &Tag) -> std::cmp::Ordering {
    position_doubles(left)
        .partial_cmp(&position_doubles(right))
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn position_ints(tag: &Tag) -> (i32, i32, i32) {
    let Some(pos) = compound_entries(tag)
        .and_then(|entries| compound_get(entries, "pos"))
        .and_then(Tag::as_list)
    else {
        return (i32::MAX, i32::MAX, i32::MAX);
    };
    (
        list_int_or(pos, 1, 0),
        list_int_or(pos, 0, 0),
        list_int_or(pos, 2, 0),
    )
}

fn position_doubles(tag: &Tag) -> (OrderedF64, OrderedF64, OrderedF64) {
    let Some(pos) = compound_entries(tag)
        .and_then(|entries| compound_get(entries, "pos"))
        .and_then(Tag::as_list)
    else {
        return (
            OrderedF64(f64::INFINITY),
            OrderedF64(f64::INFINITY),
            OrderedF64(f64::INFINITY),
        );
    };
    (
        OrderedF64(list_double_or(pos, 1, 0.0)),
        OrderedF64(list_double_or(pos, 0, 0.0)),
        OrderedF64(list_double_or(pos, 2, 0.0)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedF64(f64);

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

fn list_int_or(values: &[Tag], index: usize, default_value: i32) -> i32 {
    values
        .get(index)
        .and_then(Tag::as_int)
        .unwrap_or(default_value)
}

fn list_double_or(values: &[Tag], index: usize, default_value: f64) -> f64 {
    values
        .get(index)
        .and_then(Tag::as_double)
        .unwrap_or(default_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NBT_UTILS_JAVA: &str =
        include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtUtils.java");

    #[test]
    fn nbt_utils_compare_nbt_matches_java_partial_rules() {
        for sentinel in [
            "public static boolean compareNbt",
            "if (expected == null)",
            "if (!expected.getClass().equals(actual.getClass()))",
            "actualCompound.size() < expectedCompound.size()",
            "expected instanceof ListTag expectedList && partialListMatches",
        ] {
            assert!(
                NBT_UTILS_JAVA.contains(sentinel),
                "missing NbtUtils sentinel {sentinel}"
            );
        }

        assert!(compare_nbt(None, Some(&Tag::Int(1)), false));
        assert!(!compare_nbt(Some(&Tag::Int(1)), None, false));
        assert!(!compare_nbt(Some(&Tag::Int(1)), Some(&Tag::Long(1)), false));
        assert!(compare_nbt(
            Some(&Tag::Compound(vec![("a".to_string(), Tag::Int(1))])),
            Some(&Tag::Compound(vec![
                ("a".to_string(), Tag::Int(1)),
                ("b".to_string(), Tag::Int(2))
            ])),
            false
        ));
        assert!(compare_nbt(
            Some(&Tag::List(vec![Tag::String("x".to_string()), Tag::Int(2)])),
            Some(&Tag::List(vec![
                Tag::Int(2),
                Tag::String("x".to_string()),
                Tag::Byte(3)
            ])),
            true
        ));
        assert!(!compare_nbt(
            Some(&Tag::List(vec![Tag::String("x".to_string()), Tag::Int(2)])),
            Some(&Tag::List(vec![
                Tag::Int(2),
                Tag::String("x".to_string()),
                Tag::Byte(3)
            ])),
            false
        ));
    }

    #[test]
    fn nbt_utils_pretty_print_matches_java_layout_and_blob_switch() {
        for sentinel in [
            "public static String prettyPrint",
            "byte[",
            " // Skipped, supply withBinaryBlobs true",
            "String.format(Locale.ROOT, \"0x%02X\"",
            "Collections.sort(keys)",
            "private static StringBuilder indent",
        ] {
            assert!(
                NBT_UTILS_JAVA.contains(sentinel),
                "missing NbtUtils sentinel {sentinel}"
            );
        }

        assert_eq!(pretty_print(&Tag::Int(3), false), "3");
        assert_eq!(
            pretty_print(&Tag::ByteArray(vec![0, 15, -1]), false),
            "byte[3] {\n   // Skipped, supply withBinaryBlobs true\n}"
        );
        assert_eq!(
            pretty_print(&Tag::ByteArray(vec![0, 15, -1]), true),
            "byte[3] {\n  0x00, 0x0F, 0xFF\n}"
        );
        assert_eq!(
            pretty_print(
                &Tag::Compound(vec![
                    ("z".to_string(), Tag::Int(1)),
                    ("alpha".to_string(), Tag::String("two".to_string()))
                ]),
                false
            ),
            "{ \"alpha\": \"two\",\n  \"z\"    : 1\n}"
        );
    }

    #[test]
    fn nbt_utils_structure_pack_unpack_and_block_state_strings_match_java() {
        for sentinel in [
            "structureToSnbt",
            "snbtToStructure",
            "packStructureTemplate",
            "unpackStructureTemplate",
            "packBlockState",
            "unpackBlockState",
            "Comparator.comparing(tag -> tag.getList(\"pos\"), Comparators.emptiesLast(YXZ_LISTTAG_INT_COMPARATOR))",
        ] {
            assert!(NBT_UTILS_JAVA.contains(sentinel), "missing NbtUtils sentinel {sentinel}");
        }

        let state = Tag::Compound(vec![
            (
                "Name".to_string(),
                Tag::String("minecraft:stone".to_string()),
            ),
            (
                "Properties".to_string(),
                Tag::Compound(vec![
                    ("variant".to_string(), Tag::String("smooth".to_string())),
                    ("waterlogged".to_string(), Tag::String("false".to_string())),
                ]),
            ),
        ]);
        assert_eq!(
            pack_block_state_tag(&state).unwrap(),
            "minecraft:stone{variant:smooth,waterlogged:false}"
        );
        assert_eq!(
            unpack_block_state("minecraft:air"),
            Tag::Compound(vec![(
                "Name".to_string(),
                Tag::String("minecraft:air".to_string())
            )])
        );

        let structure = Tag::Compound(vec![
            ("palette".to_string(), Tag::List(vec![state.clone()])),
            (
                "blocks".to_string(),
                Tag::List(vec![Tag::Compound(vec![
                    ("state".to_string(), Tag::Int(0)),
                    (
                        "pos".to_string(),
                        Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]),
                    ),
                ])]),
            ),
        ]);
        let packed = pack_structure_template(structure).unwrap();
        assert_eq!(
            packed.compound_get("palette").unwrap(),
            &Tag::List(vec![Tag::String(
                "minecraft:stone{variant:smooth,waterlogged:false}".to_string()
            )])
        );
        assert!(packed.compound_get("blocks").is_none());
        assert!(packed.compound_get(SNBT_DATA_TAG).is_some());

        let unpacked = unpack_structure_template(packed).unwrap();
        assert!(unpacked.compound_get("data").is_none());
        assert!(unpacked.compound_get("blocks").is_some());
    }

    #[test]
    fn nbt_utils_data_version_helpers_match_java_current_version() {
        for sentinel in [
            "addCurrentDataVersion",
            "SharedConstants.getCurrentVersion().dataVersion().version()",
            "addDataVersion(final CompoundTag tag, final int version)",
            "getDataVersion(final CompoundTag tag, final int _default)",
        ] {
            assert!(
                NBT_UTILS_JAVA.contains(sentinel),
                "missing NbtUtils sentinel {sentinel}"
            );
        }

        let mut tag = Tag::Compound(Vec::new());
        assert_eq!(get_data_version(&tag), -1);
        assert!(add_current_data_version(&mut tag));
        assert_eq!(get_data_version(&tag), 4790);
        assert!(add_data_version(&mut tag, 123));
        assert_eq!(get_data_version_or(&tag, 7), 123);
        assert!(!add_data_version(&mut Tag::Int(1), 123));
    }
}
