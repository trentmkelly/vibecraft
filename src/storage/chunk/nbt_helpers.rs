use super::*;

pub(super) fn heightmap_fields(chunk: &LevelChunk) -> Vec<(String, Tag)> {
    chunk
        .heightmaps
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

pub(super) fn filter_saved_ticks_for_chunk(ticks: Vec<Tag>, pos: ChunkPos) -> Vec<Tag> {
    ticks
        .into_iter()
        .filter(|tick| saved_tick_belongs_to_chunk(tick, pos))
        .collect()
}

pub(super) fn compound_list_entries(entries: Vec<Tag>) -> Vec<Tag> {
    entries
        .into_iter()
        .filter(|entry| matches!(entry, Tag::Compound(_)))
        .collect()
}

pub(super) fn chunk_sections_from_list(entries: Vec<Tag>) -> Result<Vec<ChunkSection>, String> {
    entries
        .into_iter()
        .filter(|entry| matches!(entry, Tag::Compound(_)))
        .map(|entry| ChunkSection::from_nbt(&entry))
        .collect()
}

pub(super) fn post_processing_sections(entries: Vec<Tag>) -> Vec<Tag> {
    entries
        .into_iter()
        .map(|entry| match entry {
            Tag::List(offsets) => Tag::List(
                offsets
                    .into_iter()
                    .map(|offset| match offset {
                        Tag::Short(value) => Tag::Short(value),
                        _ => Tag::Short(0),
                    })
                    .collect(),
            ),
            _ => Tag::List(Vec::new()),
        })
        .collect()
}

pub(super) fn saved_tick_belongs_to_chunk(tick: &Tag, pos: ChunkPos) -> bool {
    let Ok(fields) = compound(tick) else {
        return false;
    };
    let Ok(x) = int_field(fields, "x") else {
        return false;
    };
    let Ok(z) = int_field(fields, "z") else {
        return false;
    };
    x.div_euclid(CHUNK_WIDTH) == pos.x && z.div_euclid(CHUNK_WIDTH) == pos.z
}

pub(super) fn compound(tag: &Tag) -> Result<&[(String, Tag)], String> {
    match tag {
        Tag::Compound(values) => Ok(values),
        _ => Err("expected NBT compound".to_string()),
    }
}

pub(super) fn field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a Tag, String> {
    compound
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("missing NBT field {name}"))
}

pub(super) fn optional_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
}

pub(super) fn byte_field(compound: &[(String, Tag)], name: &str) -> Result<i8, String> {
    match field(compound, name)? {
        Tag::Byte(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be a byte")),
    }
}

pub(super) fn int_field(compound: &[(String, Tag)], name: &str) -> Result<i32, String> {
    match field(compound, name)? {
        Tag::Int(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be an int")),
    }
}

pub(super) fn optional_int_field(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<i32>, String> {
    match optional_field(compound, name) {
        Some(Tag::Int(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be an int")),
        None => Ok(None),
    }
}

pub(super) fn optional_byte_field(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<i8>, String> {
    match optional_field(compound, name) {
        Some(Tag::Byte(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be a byte")),
        None => Ok(None),
    }
}

pub(super) fn long_field(compound: &[(String, Tag)], name: &str) -> Result<i64, String> {
    match field(compound, name)? {
        Tag::Long(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be a long")),
    }
}

pub(super) fn optional_long_field(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<i64>, String> {
    match optional_field(compound, name) {
        Some(Tag::Long(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be a long")),
        None => Ok(None),
    }
}

pub(super) fn string_field<'a>(
    compound: &'a [(String, Tag)],
    name: &str,
) -> Result<&'a str, String> {
    match field(compound, name)? {
        Tag::String(value) => Ok(value),
        _ => Err(format!("NBT field {name} must be a string")),
    }
}

pub(super) fn list_field<'a>(
    compound: &'a [(String, Tag)],
    name: &str,
) -> Result<&'a [Tag], String> {
    match field(compound, name)? {
        Tag::List(values) => Ok(values),
        _ => Err(format!("NBT field {name} must be a list")),
    }
}

pub(super) fn optional_list_field(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<Tag>>, String> {
    match optional_field(compound, name) {
        Some(Tag::List(values)) => Ok(Some(values.clone())),
        Some(_) => Err(format!("NBT field {name} must be a list")),
        None => Ok(None),
    }
}

pub(super) fn optional_compound_field<'a>(
    compound: &'a [(String, Tag)],
    name: &str,
) -> Result<Option<&'a [(String, Tag)]>, String> {
    match optional_field(compound, name) {
        Some(Tag::Compound(values)) => Ok(Some(values)),
        Some(_) => Err(format!("NBT field {name} must be a compound")),
        None => Ok(None),
    }
}

pub(super) fn optional_compound_tag(compound: &[(String, Tag)], name: &str) -> Option<Tag> {
    match optional_field(compound, name) {
        Some(Tag::Compound(values)) => Some(Tag::Compound(values.clone())),
        Some(_) | None => None,
    }
}

pub(super) fn optional_byte_array(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<i8>>, String> {
    match compound.iter().find(|(field_name, _)| field_name == name) {
        Some((_name, Tag::ByteArray(values))) => Ok(Some(values.clone())),
        Some((_name, _)) => Err(format!("NBT field {name} must be a byte array")),
        None => Ok(None),
    }
}

pub(super) fn optional_light_array(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<i8>>, String> {
    optional_byte_array(compound, name)?
        .map(|values| {
            if values.len() == LIGHT_DATA_LAYER_LENGTH {
                Ok(values)
            } else {
                Err(format!(
                    "DataLayer should be 2048 bytes not: {} for NBT field {name}",
                    values.len()
                ))
            }
        })
        .transpose()
}

pub(super) fn optional_long_array(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<i64>>, String> {
    match optional_field(compound, name) {
        Some(Tag::LongArray(values)) => Ok(Some(values.clone())),
        Some(_) => Err(format!("NBT field {name} must be a long array")),
        None => Ok(None),
    }
}

pub(super) fn optional_bool_field(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<bool>, String> {
    match optional_field(compound, name) {
        Some(Tag::Byte(value)) => Ok(Some(*value != 0)),
        Some(_) => Err(format!("NBT field {name} must be a byte boolean")),
        None => Ok(None),
    }
}

pub(super) fn chunk_status_field(compound: &[(String, Tag)]) -> Result<String, String> {
    let status = string_field(compound, "Status")?;
    if status.is_empty() {
        return Err("chunk Status cannot be empty".to_string());
    }
    Ok(chunk_status(status)
        .map(|status| status.id.to_string())
        .unwrap_or_else(|| "minecraft:empty".to_string()))
}

pub(super) fn optional_blending_data(compound: &[(String, Tag)]) -> Result<Option<Tag>, String> {
    let Some(tag) = optional_field(compound, "blending_data") else {
        return Ok(None);
    };
    validate_blending_data(tag)?;
    Ok(Some(tag.clone()))
}

pub(super) fn validate_blending_data(tag: &Tag) -> Result<(), String> {
    let compound = compound(tag)?;
    let min_section = int_field(compound, "min_section")?;
    let max_section = int_field(compound, "max_section")?;
    let heights = optional_double_list(compound, "heights")?;
    validate_blending_data_packed(BlendingDataPacked {
        min_section,
        max_section,
        heights: heights.as_deref(),
    })
}

pub(super) fn optional_double_list(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<f64>>, String> {
    let Some(tag) = optional_field(compound, name) else {
        return Ok(None);
    };
    let Tag::List(values) = tag else {
        return Err(format!("NBT field {name} must be a double list"));
    };
    values
        .iter()
        .map(|value| match value {
            Tag::Double(value) => Ok(*value),
            _ => Err(format!("NBT field {name} must be a double list")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub(super) fn optional_below_zero_retrogen(
    compound: &[(String, Tag)],
) -> Result<Option<Tag>, String> {
    let Some(tag) = optional_field(compound, "below_zero_retrogen") else {
        return Ok(None);
    };
    validate_below_zero_retrogen(tag)?;
    Ok(Some(tag.clone()))
}

pub(super) fn validate_below_zero_retrogen(tag: &Tag) -> Result<(), String> {
    let compound = compound(tag)?;
    let target_status = string_field(compound, "target_status")?;
    let target_status_name = target_status
        .strip_prefix("minecraft:")
        .unwrap_or(target_status);
    if target_status_name == "empty" {
        return Err("below_zero_retrogen target_status cannot be empty".to_string());
    }
    if chunk_status(target_status_name).is_none() {
        return Err(format!(
            "below_zero_retrogen target_status {target_status} is not a known chunk status"
        ));
    }
    optional_long_array(compound, "missing_bedrock")?;
    Ok(())
}

pub(super) fn below_zero_retrogen_target_status(tag: &Tag) -> Option<&'static str> {
    let compound = compound(tag).ok()?;
    let target_status = string_field(compound, "target_status").ok()?;
    chunk_status(target_status).map(|status| status.id)
}
