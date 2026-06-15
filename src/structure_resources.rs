#![allow(dead_code)]

use std::path::Path;

use crate::storage::nbt::{read_gzip_named_tag, Tag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureTemplateResource {
    pub data_version: i32,
    pub size: [i32; 3],
    pub block_count: usize,
    pub entity_count: usize,
    pub palette_count: usize,
    pub palette_state_count: usize,
}

pub fn load_structure_template_resource(
    path: impl AsRef<Path>,
) -> Result<StructureTemplateResource, String> {
    let file = std::fs::File::open(path.as_ref())
        .map_err(|err| format!("failed to open {}: {err}", path.as_ref().display()))?;
    let (_name, tag) = read_gzip_named_tag(file).map_err(|err| {
        format!(
            "failed to read {} as gzip NBT: {err}",
            path.as_ref().display()
        )
    })?;
    parse_structure_template_tag(&tag)
}

pub fn parse_structure_template_tag(tag: &Tag) -> Result<StructureTemplateResource, String> {
    let root = compound(tag, "root")?;
    let data_version = int_field(root, "DataVersion")?;
    let size = int_triplet(list_field(root, "size")?, "size")?;
    let blocks = list_field(root, "blocks")?;
    let entities = list_field(root, "entities")?;
    let (palette_count, palette_state_count) = parse_palettes(root)?;

    for (index, block) in blocks.iter().enumerate() {
        let block = compound(block, &format!("blocks[{index}]"))?;
        int_triplet(list_field(block, "pos")?, "block pos")?;
        let state = int_field(block, "state")?;
        if state < 0 || state as usize >= palette_state_count {
            return Err(format!(
                "blocks[{index}] state index {state} outside palette"
            ));
        }
        if let Some(nbt) = optional_field(block, "nbt") {
            compound(nbt, "block nbt")?;
        }
    }

    for (index, entity) in entities.iter().enumerate() {
        let entity = compound(entity, &format!("entities[{index}]"))?;
        double_triplet(list_field(entity, "pos")?, "entity pos")?;
        int_triplet(list_field(entity, "blockPos")?, "entity blockPos")?;
        if let Some(nbt) = optional_field(entity, "nbt") {
            compound(nbt, "entity nbt")?;
        }
    }

    Ok(StructureTemplateResource {
        data_version,
        size,
        block_count: blocks.len(),
        entity_count: entities.len(),
        palette_count,
        palette_state_count,
    })
}

fn parse_palettes(root: &[(String, Tag)]) -> Result<(usize, usize), String> {
    if let Some(palettes) = optional_field(root, "palettes") {
        let palettes = list(palettes, "palettes")?;
        let mut first_palette_len = None;
        for (palette_index, palette) in palettes.iter().enumerate() {
            let palette = list(palette, &format!("palettes[{palette_index}]"))?;
            validate_palette(palette)?;
            if let Some(expected) = first_palette_len {
                if palette.len() != expected {
                    return Err(
                        "all alternate palettes must match the first palette size".to_string()
                    );
                }
            } else {
                first_palette_len = Some(palette.len());
            }
        }
        Ok((palettes.len(), first_palette_len.unwrap_or(0)))
    } else {
        let palette = list_field(root, "palette")?;
        validate_palette(palette)?;
        Ok((1, palette.len()))
    }
}

fn validate_palette(palette: &[Tag]) -> Result<(), String> {
    for (index, state) in palette.iter().enumerate() {
        let state = compound(state, &format!("palette[{index}]"))?;
        string_field(state, "Name")?;
        if let Some(properties) = optional_field(state, "Properties") {
            compound(properties, "block state Properties")?;
        }
    }
    Ok(())
}

fn optional_field<'a>(compound: &'a [(String, Tag)], field: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find_map(|(name, tag)| (name == field).then_some(tag))
}

fn int_field(compound: &[(String, Tag)], field: &str) -> Result<i32, String> {
    match optional_field(compound, field) {
        Some(Tag::Int(value)) => Ok(*value),
        Some(_) => Err(format!("{field} must be an int")),
        None => Err(format!("{field} is required")),
    }
}

fn string_field<'a>(compound: &'a [(String, Tag)], field: &str) -> Result<&'a str, String> {
    match optional_field(compound, field) {
        Some(Tag::String(value)) => Ok(value),
        Some(_) => Err(format!("{field} must be a string")),
        None => Err(format!("{field} is required")),
    }
}

fn list_field<'a>(compound: &'a [(String, Tag)], field: &str) -> Result<&'a [Tag], String> {
    let tag = optional_field(compound, field).ok_or_else(|| format!("{field} is required"))?;
    list(tag, field)
}

fn list<'a>(tag: &'a Tag, field: &str) -> Result<&'a [Tag], String> {
    match tag {
        Tag::List(values) => Ok(values),
        _ => Err(format!("{field} must be a list")),
    }
}

fn compound<'a>(tag: &'a Tag, field: &str) -> Result<&'a [(String, Tag)], String> {
    match tag {
        Tag::Compound(values) => Ok(values),
        _ => Err(format!("{field} must be a compound")),
    }
}

fn int_triplet(values: &[Tag], field: &str) -> Result<[i32; 3], String> {
    if values.len() != 3 {
        return Err(format!("{field} must contain exactly three ints"));
    }
    let mut result = [0; 3];
    for (index, value) in values.iter().enumerate() {
        result[index] = match value {
            Tag::Int(value) => *value,
            _ => return Err(format!("{field}[{index}] must be an int")),
        };
    }
    Ok(result)
}

fn double_triplet(values: &[Tag], field: &str) -> Result<[f64; 3], String> {
    if values.len() != 3 {
        return Err(format!("{field} must contain exactly three doubles"));
    }
    let mut result = [0.0; 3];
    for (index, value) in values.iter().enumerate() {
        result[index] = match value {
            Tag::Double(value) => *value,
            _ => return Err(format!("{field}[{index}] must be a double")),
        };
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn structure_resources_decode_all_vanilla_templates() {
        let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
            unreachable!("test is gated on vibecraft_has_decompiled_sources");
        };
        let root = std::path::PathBuf::from(source_root)
        .join("data")
        .join("minecraft")
        .join("structure");
        let mut paths = Vec::new();
        collect_nbt_paths(&root, &mut paths);
        paths.sort();

        assert_eq!(paths.len(), 1202);
        let mut total_blocks = 0;
        let mut templates_with_entities = 0;
        let mut templates_with_alternate_palettes = 0;
        for path in &paths {
            let template = load_structure_template_resource(path).unwrap_or_else(|err| {
                panic!("{} failed to decode: {err}", path.display());
            });
            assert!(template.data_version > 0);
            assert!(template.palette_count >= 1);
            total_blocks += template.block_count;
            if template.entity_count > 0 {
                templates_with_entities += 1;
            }
            if template.palette_count > 1 {
                templates_with_alternate_palettes += 1;
            }
        }

        assert!(total_blocks > 100_000);
        assert!(templates_with_entities > 0);
        assert!(templates_with_alternate_palettes > 0);
    }

    #[test]
    fn structure_template_tag_validation_rejects_bad_state_indices() {
        let tag = Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(4000)),
            (
                "size".to_string(),
                Tag::List(vec![Tag::Int(1), Tag::Int(1), Tag::Int(1)]),
            ),
            (
                "palette".to_string(),
                Tag::List(vec![Tag::Compound(vec![(
                    "Name".to_string(),
                    Tag::String("minecraft:stone".to_string()),
                )])]),
            ),
            (
                "blocks".to_string(),
                Tag::List(vec![Tag::Compound(vec![
                    (
                        "pos".to_string(),
                        Tag::List(vec![Tag::Int(0), Tag::Int(0), Tag::Int(0)]),
                    ),
                    ("state".to_string(), Tag::Int(1)),
                ])]),
            ),
            ("entities".to_string(), Tag::List(Vec::new())),
        ]);

        assert!(parse_structure_template_tag(&tag).is_err());
    }

    fn collect_nbt_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collect_nbt_paths(&path, paths);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("nbt") {
                paths.push(path);
            }
        }
    }
}
