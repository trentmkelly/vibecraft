//! `pack.mcmeta` parsing: `Pack.readPackMetadata` with the `pack`, `features` and
//! `overlays` sections (`PackMetadataSection`, `FeatureFlagsMetadataSection`,
//! `OverlayMetadataSection`) and the `filter` section (`ResourceFilterSection`).

use serde_json::{Map, Value};

use crate::registry::{FeatureFlagRegistry, FeatureFlagSet, Identifier};

use super::pack_format::{IntermediaryFormat, TOP_MINOR};
use super::{
    DataPackMetadata, PackCompatibility, PackFormat, PackFormatRange,
    LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT,
};

/// `Pack.readPackMetadata` for `PackType.SERVER_DATA`: parses `contents` and
/// classifies the pack against the current data-pack format.
///
/// An `Err` means Java would log a warning and drop the pack (`null` metadata):
/// unreadable JSON, a missing `pack` section, or an invalid `features`/`overlays`
/// section. A `pack` section whose formats fail validation falls back to
/// `PackMetadataSection.FALLBACK_TYPE` (description only), which yields
/// [`PackCompatibility::Unknown`].
pub fn parse_pack_metadata(contents: &str) -> Result<DataPackMetadata, String> {
    let root = parse_root(contents)?;
    let pack = root.get("pack").ok_or("missing pack metadata")?;
    let (description, supported_formats) = match parse_pack_section(pack) {
        Ok(section) => section,
        // `JsonParseException` on the typed section: retry with the fallback codec.
        Err(_) => (
            parse_fallback_section(pack)?,
            PackFormatRange {
                min: PackFormat {
                    major: TOP_MINOR,
                    minor: 0,
                },
                max: PackFormat {
                    major: TOP_MINOR,
                    minor: 0,
                },
            },
        ),
    };
    let current = PackFormat::current_server_data();
    Ok(DataPackMetadata {
        description,
        supported_formats,
        compatibility: PackCompatibility::for_version(supported_formats, current),
        requested_features: parse_features_section(root.get("features"))?,
        overlays: parse_overlays_section(root.get("overlays"), current)?,
    })
}

/// A top-level `pack.mcmeta` JSON object (`GsonHelper.parse`).
pub(super) fn parse_root(contents: &str) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(contents) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("pack.mcmeta is not a JSON object".to_string()),
        Err(error) => Err(format!("invalid pack.mcmeta: {error}")),
    }
}

/// `PackMetadataSection.codecForPackType(SERVER_DATA)`.
pub(super) fn parse_pack_section(pack: &Value) -> Result<(String, PackFormatRange), String> {
    let map = pack
        .as_object()
        .ok_or_else(|| format!("Not a map: {pack}"))?;
    let description = parse_description(map)?;
    let formats = IntermediaryFormat::from_pack(map)?.validate(
        LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT,
        true,
        false,
        "Pack",
        "supported_formats",
    )?;
    Ok((description, formats))
}

/// `PackMetadataSection.FALLBACK_CODEC`: only the description is read.
fn parse_fallback_section(pack: &Value) -> Result<String, String> {
    let map = pack
        .as_object()
        .ok_or_else(|| format!("Not a map: {pack}"))?;
    parse_description(map)
}

/// The `description` component (`ComponentSerialization.CODEC`): the plain text of
/// a literal, or the translation key of a translatable component.
fn parse_description(pack: &Map<String, Value>) -> Result<String, String> {
    let description = pack
        .get("description")
        .ok_or("No key description in pack section")?;
    component_text(description)
}

fn component_text(component: &Value) -> Result<String, String> {
    match component {
        Value::String(text) => Ok(text.clone()),
        Value::Array(parts) => parts
            .first()
            .ok_or_else(|| "Empty component list".to_string())
            .and_then(component_text),
        Value::Object(map) => ["text", "translate", "keybind", "selector"]
            .iter()
            .find_map(|key| map.get(*key).and_then(Value::as_str))
            .map(str::to_string)
            .or_else(|| (map.contains_key("score") || map.contains_key("nbt")).then(String::new))
            .ok_or_else(|| format!("Don't know how to turn {component} into a Component")),
        other => Ok(other.to_string()),
    }
}

/// `FeatureFlagsMetadataSection`: absent means no requested features.
fn parse_features_section(section: Option<&Value>) -> Result<FeatureFlagSet, String> {
    let Some(section) = section else {
        return Ok(FeatureFlagSet::empty());
    };
    let enabled = section
        .as_object()
        .and_then(|map| map.get("enabled"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("No key enabled in {section}"))?;
    let names = enabled
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| format!("Not a string: {entry}"))
                .and_then(Identifier::parse)
        })
        .collect::<Result<Vec<_>, _>>()?;
    FeatureFlagRegistry::main_26_1_2()?
        .resolve_names(&names)
        .map_err(|unknown| format!("Unknown feature ids: {unknown:?}"))
}

/// `OverlayMetadataSection.codecForPackType(SERVER_DATA)` followed by
/// `overlaysForVersion(current)`.
fn parse_overlays_section(
    section: Option<&Value>,
    current: PackFormat,
) -> Result<Vec<String>, String> {
    let Some(section) = section else {
        return Ok(Vec::new());
    };
    let entries = section
        .as_object()
        .and_then(|map| map.get("entries"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("No key entries in {section}"))?;
    let mut parsed = Vec::with_capacity(entries.len());
    for entry in entries {
        let map = entry
            .as_object()
            .ok_or_else(|| format!("Not a map: {entry}"))?;
        let directory = map
            .get("directory")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("No key directory in {entry}"))?;
        if directory.is_empty()
            || !directory
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        {
            return Err(format!("{directory} is not accepted directory name"));
        }
        parsed.push((
            IntermediaryFormat::from_overlay(map)?,
            directory.to_string(),
        ));
    }
    // `PackFormat.validateHolderList`.
    let min_version = parsed
        .iter()
        .map(|(format, _)| format.effective_min_major_version())
        .min()
        .unwrap_or(i32::MAX as u32);
    let mut overlays = Vec::new();
    for (format, directory) in parsed {
        if format.is_empty() {
            continue; // "Unknown or broken overlay entry" is logged and skipped.
        }
        let range = format.validate(
            LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT,
            false,
            min_version <= LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT,
            &format!("Overlay \"{directory}\""),
            "formats",
        )?;
        if range.min <= current && current <= range.max {
            overlays.push(directory);
        }
    }
    Ok(overlays)
}

/// `ResourceFilterSection`: `filter.block` patterns hiding resources of lower-priority
/// packs. Unset pattern parts match everything; set parts match with
/// `Pattern.asPredicate` (a partial match).
#[derive(Debug, Clone)]
pub struct ResourceFilter {
    patterns: Vec<(Option<regex::Regex>, Option<regex::Regex>)>,
}

impl ResourceFilter {
    /// `isNamespaceFiltered`.
    pub fn is_namespace_filtered(&self, namespace: &str) -> bool {
        self.patterns
            .iter()
            .any(|(pattern, _)| pattern.as_ref().is_none_or(|p| p.is_match(namespace)))
    }

    /// `isPathFiltered`.
    pub fn is_path_filtered(&self, path: &str) -> bool {
        self.patterns
            .iter()
            .any(|(_, pattern)| pattern.as_ref().is_none_or(|p| p.is_match(path)))
    }
}

/// Parses the `filter` section of raw `pack.mcmeta` text; `None` when the pack has no
/// (valid) `filter` section (Java logs the failure and ignores the filter).
pub fn parse_filter_section(contents: &str) -> Option<ResourceFilter> {
    let root = parse_root(contents).ok()?;
    let block = root.get("filter")?.as_object()?.get("block")?.as_array()?;
    let patterns = block
        .iter()
        .map(|entry| {
            let map = entry.as_object()?;
            let compile = |key: &str| -> Option<Option<regex::Regex>> {
                match map.get(key) {
                    None => Some(None),
                    Some(Value::String(text)) => regex::Regex::new(text).ok().map(Some),
                    Some(_) => None,
                }
            };
            Some((compile("namespace")?, compile("path")?))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ResourceFilter { patterns })
}
