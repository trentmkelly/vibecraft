#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use crate::worldgen::{parse_flat_generator_settings_value, parse_world_preset_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorldgenResourceKind {
    Biome,
    ConfiguredCarver,
    ConfiguredFeature,
    DensityFunction,
    FlatLevelGeneratorPreset,
    MultiNoiseBiomeSourceParameterList,
    Noise,
    NoiseSettings,
    PlacedFeature,
    ProcessorList,
    Structure,
    StructureSet,
    TemplatePool,
    WorldPreset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenResourceSummary {
    pub kind: WorldgenResourceKind,
    pub key: String,
    pub top_level_keys: Vec<String>,
}

pub fn load_worldgen_resource(
    root: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<WorldgenResourceSummary, String> {
    let root = root.as_ref();
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let relative = path
        .strip_prefix(root)
        .map_err(|err| format!("{} is not under {}: {err}", path.display(), root.display()))?;
    let kind = worldgen_kind(relative)?;
    let key = resource_key(relative)?;
    parse_worldgen_resource(kind, key, &raw)
}

pub fn parse_worldgen_resource(
    kind: WorldgenResourceKind,
    key: String,
    raw: &str,
) -> Result<WorldgenResourceSummary, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid worldgen JSON: {err}"))?;
    let Some(object) = value.as_object() else {
        if kind == WorldgenResourceKind::DensityFunction && value.as_f64().is_some() {
            return Ok(WorldgenResourceSummary {
                kind,
                key,
                top_level_keys: Vec::new(),
            });
        }
        return Err("worldgen resource must be a JSON object".to_string());
    };
    validate_worldgen_object(kind, object)?;
    let mut top_level_keys = object.keys().cloned().collect::<Vec<_>>();
    top_level_keys.sort();
    Ok(WorldgenResourceSummary {
        kind,
        key,
        top_level_keys,
    })
}

fn validate_worldgen_object(
    kind: WorldgenResourceKind,
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    match kind {
        WorldgenResourceKind::Biome => {
            require_number(object, "temperature")?;
            require_number(object, "downfall")?;
            require_object(object, "effects")?;
            require_object(object, "spawners")?;
            require_array(object, "features")?;
            require_value(object, "carvers")?;
        }
        WorldgenResourceKind::ConfiguredCarver | WorldgenResourceKind::ConfiguredFeature => {
            require_string(object, "type")?;
            require_object(object, "config")?;
        }
        WorldgenResourceKind::DensityFunction => {
            require_string(object, "type")?;
        }
        WorldgenResourceKind::FlatLevelGeneratorPreset => {
            require_value(object, "display")?;
            parse_flat_generator_settings_value(require_object_value(object, "settings")?)?;
        }
        WorldgenResourceKind::MultiNoiseBiomeSourceParameterList => {
            if !object.contains_key("preset") && !object.contains_key("parameters") {
                return Err("multi-noise parameter list requires preset or parameters".to_string());
            }
        }
        WorldgenResourceKind::Noise => {
            require_i64(object, "firstOctave")?;
            require_array(object, "amplitudes")?;
        }
        WorldgenResourceKind::NoiseSettings => {
            require_object(object, "noise")?;
            require_object(object, "noise_router")?;
            require_object(object, "default_block")?;
            require_object(object, "default_fluid")?;
        }
        WorldgenResourceKind::PlacedFeature => {
            require_value(object, "feature")?;
            require_array(object, "placement")?;
        }
        WorldgenResourceKind::ProcessorList => {
            require_array(object, "processors")?;
        }
        WorldgenResourceKind::Structure => {
            require_string(object, "type")?;
            require_value(object, "biomes")?;
            require_string(object, "step")?;
            require_object(object, "spawn_overrides")?;
        }
        WorldgenResourceKind::StructureSet => {
            require_array(object, "structures")?;
            require_object(object, "placement")?;
        }
        WorldgenResourceKind::TemplatePool => {
            require_value(object, "fallback")?;
            require_array(object, "elements")?;
        }
        WorldgenResourceKind::WorldPreset => {
            require_object(object, "dimensions")?;
            let raw = serde_json::Value::Object(object.clone());
            parse_world_preset_json(&raw.to_string())?;
        }
    }
    Ok(())
}

fn worldgen_kind(relative: &Path) -> Result<WorldgenResourceKind, String> {
    let Some(first) = relative.components().next() else {
        return Err("worldgen path is empty".to_string());
    };
    let name = first.as_os_str().to_string_lossy();
    match name.as_ref() {
        "biome" => Ok(WorldgenResourceKind::Biome),
        "configured_carver" => Ok(WorldgenResourceKind::ConfiguredCarver),
        "configured_feature" => Ok(WorldgenResourceKind::ConfiguredFeature),
        "density_function" => Ok(WorldgenResourceKind::DensityFunction),
        "flat_level_generator_preset" => Ok(WorldgenResourceKind::FlatLevelGeneratorPreset),
        "multi_noise_biome_source_parameter_list" => {
            Ok(WorldgenResourceKind::MultiNoiseBiomeSourceParameterList)
        }
        "noise" => Ok(WorldgenResourceKind::Noise),
        "noise_settings" => Ok(WorldgenResourceKind::NoiseSettings),
        "placed_feature" => Ok(WorldgenResourceKind::PlacedFeature),
        "processor_list" => Ok(WorldgenResourceKind::ProcessorList),
        "structure" => Ok(WorldgenResourceKind::Structure),
        "structure_set" => Ok(WorldgenResourceKind::StructureSet),
        "template_pool" => Ok(WorldgenResourceKind::TemplatePool),
        "world_preset" => Ok(WorldgenResourceKind::WorldPreset),
        other => Err(format!("unknown worldgen resource kind {other}")),
    }
}

fn resource_key(relative: &Path) -> Result<String, String> {
    let without_extension = relative.with_extension("");
    let parts = without_extension
        .components()
        .skip(1)
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err("worldgen resource is missing id path".to_string());
    }
    Ok(format!("minecraft:{}", parts.join("/")))
}

fn require_value<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{field} is required"))
}

fn require_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    require_value(object, field)?
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))
}

fn require_object<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    require_object_value(object, field)?
        .as_object()
        .ok_or_else(|| format!("{field} must be an object"))
}

fn require_object_value<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    require_value(object, field)
}

fn require_array<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a Vec<serde_json::Value>, String> {
    require_value(object, field)?
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))
}

fn require_number(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<f64, String> {
    require_value(object, field)?
        .as_f64()
        .ok_or_else(|| format!("{field} must be numeric"))
}

fn require_i64(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i64, String> {
    require_value(object, field)?
        .as_i64()
        .ok_or_else(|| format!("{field} must be an integer"))
}

pub fn summarize_worldgen_resources(
    root: impl AsRef<Path>,
) -> Result<Vec<WorldgenResourceSummary>, String> {
    let root = root.as_ref();
    let mut paths = Vec::new();
    collect_json_paths(root, &mut paths).map_err(|err| err.to_string())?;
    paths.sort();
    paths
        .iter()
        .map(|path| {
            load_worldgen_resource(root, path).map_err(|err| format!("{}: {err}", path.display()))
        })
        .collect()
}

fn collect_json_paths(root: &Path, paths: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn count_by_kind(
    resources: &[WorldgenResourceSummary],
) -> BTreeMap<WorldgenResourceKind, usize> {
    let mut counts = BTreeMap::new();
    for resource in resources {
        *counts.entry(resource.kind).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worldgen_resources_decode_all_vanilla_subregistries() {
        let root = std::path::Path::new(env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT"))
            .join("data")
            .join("minecraft")
            .join("worldgen");
        let resources = summarize_worldgen_resources(&root).unwrap();
        let counts = count_by_kind(&resources);

        assert_eq!(resources.len(), 951);
        assert_eq!(counts[&WorldgenResourceKind::Biome], 65);
        assert_eq!(counts[&WorldgenResourceKind::ConfiguredCarver], 4);
        assert_eq!(counts[&WorldgenResourceKind::ConfiguredFeature], 221);
        assert_eq!(counts[&WorldgenResourceKind::DensityFunction], 35);
        assert_eq!(counts[&WorldgenResourceKind::FlatLevelGeneratorPreset], 9);
        assert_eq!(
            counts[&WorldgenResourceKind::MultiNoiseBiomeSourceParameterList],
            2
        );
        assert_eq!(counts[&WorldgenResourceKind::Noise], 62);
        assert_eq!(counts[&WorldgenResourceKind::NoiseSettings], 7);
        assert_eq!(counts[&WorldgenResourceKind::PlacedFeature], 258);
        assert_eq!(counts[&WorldgenResourceKind::ProcessorList], 40);
        assert_eq!(counts[&WorldgenResourceKind::Structure], 34);
        assert_eq!(counts[&WorldgenResourceKind::StructureSet], 20);
        assert_eq!(counts[&WorldgenResourceKind::TemplatePool], 188);
        assert_eq!(counts[&WorldgenResourceKind::WorldPreset], 6);

        assert!(resources.iter().any(|resource| {
            resource.kind == WorldgenResourceKind::WorldPreset && resource.key == "minecraft:normal"
        }));
        assert!(resources.iter().any(|resource| {
            resource.kind == WorldgenResourceKind::ConfiguredFeature
                && resource.key == "minecraft:pale_oak_creaking"
        }));
        assert!(resources.iter().any(|resource| {
            resource.kind == WorldgenResourceKind::TemplatePool
                && resource.key == "minecraft:trial_chambers/chamber/end"
        }));
    }

    #[test]
    fn worldgen_resource_validation_rejects_missing_codec_fields() {
        assert!(parse_worldgen_resource(
            WorldgenResourceKind::ConfiguredFeature,
            "minecraft:bad".to_string(),
            r#"{"type":"minecraft:tree"}"#,
        )
        .is_err());
        assert!(parse_worldgen_resource(
            WorldgenResourceKind::Noise,
            "minecraft:bad".to_string(),
            r#"{"amplitudes":[1.0]}"#,
        )
        .is_err());
        assert!(parse_worldgen_resource(
            WorldgenResourceKind::WorldPreset,
            "minecraft:bad".to_string(),
            r#"{"foo":{}}"#,
        )
        .is_err());
    }
}
