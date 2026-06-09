use super::*;

pub fn world_preset_from_overworld_generator(generator: &str) -> Option<&'static str> {
    match generator {
        "minecraft:flat" | "flat" => Some("minecraft:flat"),
        "minecraft:debug" | "debug" => Some("minecraft:debug_all_block_states"),
        "minecraft:noise" | "noise" => Some("minecraft:normal"),
        _ => None,
    }
}

pub fn world_preset_dimensions_in_order(preset: &WorldPresetEntry) -> [&'static str; 3] {
    [
        preset.overworld.dimension,
        preset.nether.dimension,
        preset.end.dimension,
    ]
}

pub fn validate_world_preset_dimensions(dimensions: &[&str]) -> Result<(), String> {
    if dimensions.contains(&"minecraft:overworld") {
        Ok(())
    } else {
        Err("Missing overworld dimension".to_string())
    }
}

pub fn parse_worldgen_settings_json(raw: &str) -> Result<WorldGenSettingsModel, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid WorldGenSettings JSON: {err}"))?;
    let object = json_object(&value, "WorldGenSettings")?;
    let seed = json_i64_field(object, "seed")?;
    let generate_structures = json_bool_field(object, "generate_features")
        .or_else(|_| json_bool_field(object, "generateStructures"))?;
    let generate_bonus_chest = json_bool_field(object, "bonus_chest")
        .or_else(|_| json_bool_field(object, "generateBonusChest"))?;
    let dimensions = parse_world_dimensions_value(json_required(object, "dimensions")?)?;

    Ok(WorldGenSettingsModel {
        seed,
        generate_structures,
        generate_bonus_chest,
        dimensions,
    })
}

pub fn parse_world_preset_json(raw: &str) -> Result<ParsedWorldPreset, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid world preset JSON: {err}"))?;
    let object = json_object(&value, "world preset")?;
    let dimensions = parse_world_dimensions_value(json_required(object, "dimensions")?)?;
    Ok(ParsedWorldPreset { dimensions })
}

pub fn load_worldgen_preset_registry(
    worldgen_root: impl AsRef<std::path::Path>,
) -> Result<ParsedWorldgenPresetRegistry, String> {
    let worldgen_root = worldgen_root.as_ref();
    Ok(ParsedWorldgenPresetRegistry {
        world_presets: load_world_preset_directory(&worldgen_root.join("world_preset"))?,
        flat_level_generator_presets: load_flat_level_generator_preset_directory(
            &worldgen_root.join("flat_level_generator_preset"),
        )?,
    })
}

pub fn load_template_pool_registry(
    template_pool_root: impl AsRef<std::path::Path>,
) -> Result<ParsedTemplatePoolRegistry, String> {
    let root = template_pool_root.as_ref();
    let mut pools = BTreeMap::new();
    load_template_pool_directory(root, root, &mut pools)?;
    Ok(ParsedTemplatePoolRegistry { pools })
}

fn load_template_pool_directory(
    root: &std::path::Path,
    directory: &std::path::Path,
    pools: &mut BTreeMap<String, ParsedJigsawTemplatePool>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|err| format!("failed to read template pool directory {directory:?}: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read template pool entry: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            load_template_pool_directory(root, &path, pools)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = worldgen_json_id(root, &path)?;
        let raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read template pool JSON {path:?}: {err}"))?;
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|err| format!("invalid template pool JSON {path:?}: {err}"))?;
        pools.insert(id, parse_template_pool_value(&value)?);
    }
    Ok(())
}

pub fn parse_template_pool_value(
    value: &serde_json::Value,
) -> Result<ParsedJigsawTemplatePool, String> {
    let object = json_object(value, "template pool")?;
    let fallback = json_string_field(object, "fallback")?.to_string();
    let elements = json_required(object, "elements")?
        .as_array()
        .ok_or_else(|| "template pool elements must be an array".to_string())?
        .iter()
        .map(|entry| {
            let entry = json_object(entry, "template pool element entry")?;
            let weight = json_i64_field(entry, "weight")?;
            let weight =
                i32::try_from(weight).map_err(|_| format!("weight {weight} overflows i32"))?;
            Ok(ParsedJigsawTemplatePoolEntry {
                element: parse_jigsaw_pool_element_value(json_required(entry, "element")?)?,
                weight,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(ParsedJigsawTemplatePool { fallback, elements })
}

fn parse_jigsaw_pool_element_value(
    value: &serde_json::Value,
) -> Result<ParsedJigsawPoolElement, String> {
    let object = json_object(value, "jigsaw pool element")?;
    let element_type = json_string_field(object, "element_type")?.to_string();
    let projection = object
        .get("projection")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let location = object
        .get("location")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let feature = object
        .get("feature")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let processors = object
        .get("processors")
        .map(parse_template_pool_processors)
        .transpose()?
        .unwrap_or_default();
    let children = object
        .get("elements")
        .and_then(|value| value.as_array())
        .map(|elements| {
            elements
                .iter()
                .map(parse_jigsaw_pool_element_value)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(ParsedJigsawPoolElement {
        element_type,
        projection,
        location,
        processors,
        feature,
        children,
    })
}

fn parse_template_pool_processors(value: &serde_json::Value) -> Result<Vec<String>, String> {
    if let Some(id) = value.as_str() {
        return Ok(vec![id.to_string()]);
    }
    let object = json_object(value, "template pool processors")?;
    object
        .get("processors")
        .and_then(|value| value.as_array())
        .map(|processors| {
            processors
                .iter()
                .map(|processor| {
                    processor
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "processor entries must be strings".to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

pub fn load_processor_list_registry(
    processor_list_root: impl AsRef<std::path::Path>,
) -> Result<ParsedProcessorListRegistry, String> {
    let root = processor_list_root.as_ref();
    let mut lists = BTreeMap::new();
    load_processor_list_directory(root, root, &mut lists)?;
    Ok(ParsedProcessorListRegistry { lists })
}

fn load_processor_list_directory(
    root: &std::path::Path,
    directory: &std::path::Path,
    lists: &mut BTreeMap<String, ParsedStructureProcessorList>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|err| format!("failed to read processor list directory {directory:?}: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read processor list entry: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            load_processor_list_directory(root, &path, lists)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = worldgen_json_id(root, &path)?;
        let raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read processor list JSON {path:?}: {err}"))?;
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|err| format!("invalid processor list JSON {path:?}: {err}"))?;
        lists.insert(id, parse_processor_list_value(&value)?);
    }
    Ok(())
}

pub fn parse_processor_list_value(
    value: &serde_json::Value,
) -> Result<ParsedStructureProcessorList, String> {
    let object = json_object(value, "processor list")?;
    let processors = json_required(object, "processors")?
        .as_array()
        .ok_or_else(|| "processor list processors must be an array".to_string())?
        .iter()
        .map(parse_structure_processor_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ParsedStructureProcessorList { processors })
}

fn parse_structure_processor_value(
    value: &serde_json::Value,
) -> Result<ParsedStructureProcessor, String> {
    let object = json_object(value, "structure processor")?;
    let processor_type = json_string_field(object, "processor_type")?.to_string();
    let rules = object
        .get("rules")
        .and_then(|value| value.as_array())
        .map(|rules| {
            rules
                .iter()
                .map(parse_structure_processor_rule_value)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let delegate = object
        .get("delegate")
        .map(parse_structure_processor_value)
        .transpose()?
        .map(Box::new);
    let limit = object
        .get("limit")
        .and_then(|value| value.as_i64())
        .map(|limit| i32::try_from(limit).map_err(|_| format!("limit {limit} overflows i32")))
        .transpose()?;
    let integrity = object
        .get("integrity")
        .map(|value| value.to_string().trim_matches('"').to_string());
    let rottable_blocks = object
        .get("rottable_blocks")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let cannot_replace = object
        .get("cannot_replace")
        .or_else(|| object.get("value"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    Ok(ParsedStructureProcessor {
        processor_type,
        rules,
        delegate,
        limit,
        integrity,
        rottable_blocks,
        cannot_replace,
    })
}

fn parse_structure_processor_rule_value(
    value: &serde_json::Value,
) -> Result<ParsedStructureProcessorRule, String> {
    let object = json_object(value, "structure processor rule")?;
    let input_predicate = json_object(
        json_required(object, "input_predicate")?,
        "structure processor input predicate",
    )?;
    let location_predicate = json_object(
        json_required(object, "location_predicate")?,
        "structure processor location predicate",
    )?;
    let output_state_name = object
        .get("output_state")
        .and_then(|value| value.as_object())
        .and_then(|state| state.get("Name"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let block_entity_modifier_type = object
        .get("block_entity_modifier")
        .and_then(|value| value.as_object())
        .and_then(|modifier| modifier.get("type"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    Ok(ParsedStructureProcessorRule {
        input_predicate_type: json_string_field(input_predicate, "predicate_type")?.to_string(),
        location_predicate_type: json_string_field(location_predicate, "predicate_type")?
            .to_string(),
        output_state_name,
        block_entity_modifier_type,
    })
}

fn load_world_preset_directory(
    directory: &std::path::Path,
) -> Result<BTreeMap<String, ParsedWorldPreset>, String> {
    load_json_directory(directory, parse_world_preset_json)
}

fn load_flat_level_generator_preset_directory(
    directory: &std::path::Path,
) -> Result<BTreeMap<String, ParsedFlatGeneratorSettings>, String> {
    load_json_directory(directory, |raw| {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|err| format!("invalid flat level generator preset JSON: {err}"))?;
        let object = json_object(&value, "flat level generator preset")?;
        parse_flat_generator_settings_value(json_required(object, "settings")?)
    })
}

fn load_json_directory<T>(
    directory: &std::path::Path,
    parse: impl Fn(&str) -> Result<T, String>,
) -> Result<BTreeMap<String, T>, String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|err| format!("failed to read {}: {err}", directory.display()))?;
    let mut loaded = BTreeMap::new();

    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("invalid JSON file name {}", path.display()))?;
        let id = format!("minecraft:{stem}");
        let raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        let parsed = parse(&raw).map_err(|err| format!("{}: {err}", path.display()))?;
        loaded.insert(id, parsed);
    }

    Ok(loaded)
}

fn worldgen_json_id(root: &std::path::Path, path: &std::path::Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|err| format!("failed to relativize {}: {err}", path.display()))?;
    let without_extension = relative.with_extension("");
    let id_path = without_extension
        .to_str()
        .ok_or_else(|| format!("invalid UTF-8 path {}", path.display()))?
        .replace('\\', "/");
    Ok(format!("minecraft:{id_path}"))
}

pub fn parse_world_dimensions_value(
    value: &serde_json::Value,
) -> Result<ParsedWorldDimensions, String> {
    let object = json_object(value, "dimensions")?;
    let mut stems = object
        .iter()
        .map(|(id, stem)| parse_level_stem_value(stem).map(|stem| (id.clone(), stem)))
        .collect::<Result<Vec<_>, _>>()?;
    stems.sort_by(|(left, _), (right, _)| {
        dimension_order_key(left).cmp(&dimension_order_key(right))
    });
    Ok(ParsedWorldDimensions { stems })
}

pub fn parse_level_stem_value(value: &serde_json::Value) -> Result<ParsedLevelStem, String> {
    let object = json_object(value, "level stem")?;
    let dimension_type = json_string_field(object, "type")?.to_string();
    let generator = parse_chunk_generator_value(json_required(object, "generator")?)?;
    Ok(ParsedLevelStem {
        dimension_type,
        generator,
    })
}

pub fn parse_chunk_generator_value(
    value: &serde_json::Value,
) -> Result<ParsedChunkGenerator, String> {
    let object = json_object(value, "chunk generator")?;
    match strip_minecraft(json_string_field(object, "type")?) {
        "noise" => Ok(ParsedChunkGenerator::Noise {
            biome_source: parse_biome_source_value(json_required(object, "biome_source")?)?,
            settings: json_string_field(object, "settings")?.to_string(),
        }),
        "flat" => Ok(ParsedChunkGenerator::Flat {
            settings: parse_flat_generator_settings_value(json_required(object, "settings")?)?,
        }),
        "debug" => Ok(ParsedChunkGenerator::Debug),
        other => Err(format!("unknown chunk generator type {other}")),
    }
}

pub fn parse_biome_source_value(value: &serde_json::Value) -> Result<ParsedBiomeSource, String> {
    if let Some(biome) = value.as_str() {
        return Ok(ParsedBiomeSource::Fixed {
            biome: biome.to_string(),
        });
    }
    let object = json_object(value, "biome source")?;
    match strip_minecraft(json_string_field(object, "type")?) {
        "multi_noise" => Ok(ParsedBiomeSource::MultiNoisePreset {
            preset: json_string_field(object, "preset")?.to_string(),
        }),
        "the_end" => Ok(ParsedBiomeSource::TheEnd),
        "fixed" => Ok(ParsedBiomeSource::Fixed {
            biome: json_string_field(object, "biome")?.to_string(),
        }),
        "checkerboard" => {
            let biomes = json_required(object, "biomes")?
                .as_array()
                .ok_or_else(|| "biomes must be an array".to_string())?
                .iter()
                .map(|biome| {
                    biome
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "checkerboard biome entries must be strings".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let scale = object.get("scale").and_then(|v| v.as_i64()).unwrap_or(2);
            Ok(ParsedBiomeSource::Checkerboard { biomes, scale })
        }
        other => Err(format!("unknown biome source type {other}")),
    }
}

pub fn parse_flat_generator_settings_value(
    value: &serde_json::Value,
) -> Result<ParsedFlatGeneratorSettings, String> {
    let object = json_object(value, "flat generator settings")?;
    let biome = json_string_field(object, "biome")?.to_string();
    let add_lakes = json_bool_field(object, "lakes")?;
    let decoration = json_bool_field(object, "features")?;
    let structure_overrides =
        parse_structure_overrides(json_required(object, "structure_overrides")?)?;
    let layers = json_required(object, "layers")?
        .as_array()
        .ok_or_else(|| "layers must be an array".to_string())?
        .iter()
        .map(|layer| {
            let layer = json_object(layer, "flat layer")?;
            let height = json_i64_field(layer, "height")?;
            let height =
                i32::try_from(height).map_err(|_| format!("height {height} overflows i32"))?;
            let block = json_string_field(layer, "block")?.to_string();
            Ok((height, block))
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(ParsedFlatGeneratorSettings {
        biome,
        structure_overrides,
        add_lakes,
        decoration,
        layers,
    })
}

fn parse_structure_overrides(value: &serde_json::Value) -> Result<Vec<String>, String> {
    if let Some(single) = value.as_str() {
        return Ok(vec![single.to_string()]);
    }
    value
        .as_array()
        .ok_or_else(|| "structure_overrides must be a string or array".to_string())?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "structure_overrides entries must be strings".to_string())
        })
        .collect()
}

fn dimension_order_key(id: &str) -> (u8, &str) {
    match id {
        "minecraft:overworld" => (0, id),
        "minecraft:the_nether" => (1, id),
        "minecraft:the_end" => (2, id),
        _ => (3, id),
    }
}

pub(super) fn strip_minecraft(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}

fn json_object<'a>(
    value: &'a serde_json::Value,
    context: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn json_required<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{field} is required"))
}

fn json_string_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    json_required(object, field)?
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))
}

fn json_i64_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i64, String> {
    json_required(object, field)?
        .as_i64()
        .ok_or_else(|| format!("{field} must be an integer"))
}

fn json_bool_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<bool, String> {
    json_required(object, field)?
        .as_bool()
        .ok_or_else(|| format!("{field} must be a boolean"))
}
