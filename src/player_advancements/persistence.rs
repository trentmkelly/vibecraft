//! The `players/advancements/<uuid>.json` file format (`PlayerAdvancements.Data.CODEC`
//! wrapped by `DataFixTypes.ADVANCEMENTS.wrapCodec`).
//!
//! The file is one JSON object: every key is an advancement id mapped to an
//! `AdvancementProgress` (`{"criteria": {name: "yyyy-MM-dd HH:mm:ss Z"}, "done": bool}`),
//! plus the `DataVersion` key that `DataFixTypes.wrapCodec` adds next to them.

use std::collections::BTreeMap;

use chrono::{DateTime, Local, TimeZone};

use crate::advancement_progress::JavaAdvancementProgressModel;
use crate::advancement_system::CriterionProgressModel;
use crate::registry::Identifier;
use crate::storage::datafix::{check_saved_tag_data_version, DataFixDecision, TARGET_DATA_VERSION};

/// `AdvancementProgress.OBTAINED_TIME_FORMAT` (`yyyy-MM-dd HH:mm:ss Z`).
const OBTAINED_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";
/// `PlayerAdvancements` passes this `defaultVersion` to `wrapCodec`: a file without a
/// `DataVersion` is treated as written by that (1.12) version.
const DEFAULT_DATA_VERSION: i32 = 1343;

/// Why a progress file could not be turned into progress data.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum LoadError {
    /// Malformed JSON or a value the codec rejects (Java `JsonParseException`).
    Parse(String),
    /// The file predates the current `DataVersion`; VibeCraft has no advancement
    /// data fixers, so it must be left untouched rather than misread.
    /// TODO(advancement-datafix): port the `DataFixTypes.ADVANCEMENTS` upgrades.
    NeedsDataFix { found: i32 },
}

/// A decoded progress file: `(id, progress)` pairs in `Data.forEach` order.
pub(super) type ProgressEntries = Vec<(Identifier, JavaAdvancementProgressModel)>;

/// `Data.CODEC` decode: `(id, progress)` pairs sorted like `Data.forEach`
/// (`Entry.comparingByValue()` - earliest first progress, unstarted last).
pub(super) fn parse_progress_file(json: &str) -> Result<ProgressEntries, LoadError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|err| LoadError::Parse(err.to_string()))?;
    let serde_json::Value::Object(mut entries) = value else {
        return Err(LoadError::Parse("Not a JSON object".to_string()));
    };
    let found = match entries.remove("DataVersion") {
        None => DEFAULT_DATA_VERSION,
        Some(version) => version
            .as_i64()
            .and_then(|version| i32::try_from(version).ok())
            .ok_or_else(|| LoadError::Parse("DataVersion is not an int".to_string()))?,
    };
    if let DataFixDecision::Blocked { .. } = check_saved_tag_data_version(found) {
        return Err(LoadError::NeedsDataFix { found });
    }
    let mut parsed = Vec::with_capacity(entries.len());
    for (key, progress) in entries {
        let id = Identifier::parse(&key).map_err(LoadError::Parse)?;
        parsed.push((id, parse_progress(&progress)?));
    }
    parsed.sort_by(|(_, left), (_, right)| left.java_compare_to(right));
    Ok(parsed)
}

/// `AdvancementProgress.CODEC`: `criteria` (optional map of obtained times) and a
/// `done` flag that only has to be a boolean when present.
fn parse_progress(value: &serde_json::Value) -> Result<JavaAdvancementProgressModel, LoadError> {
    let object = value
        .as_object()
        .ok_or_else(|| LoadError::Parse("Not a JSON object: advancement progress".to_string()))?;
    if object.get("done").is_some_and(|done| !done.is_boolean()) {
        return Err(LoadError::Parse("Not a boolean: done".to_string()));
    }
    let mut criteria = BTreeMap::new();
    if let Some(entries) = object.get("criteria") {
        let entries = entries
            .as_object()
            .ok_or_else(|| LoadError::Parse("Not a JSON object: criteria".to_string()))?;
        for (name, obtained) in entries {
            let obtained = obtained
                .as_str()
                .ok_or_else(|| LoadError::Parse(format!("Not a string: {name}")))?;
            let time = DateTime::parse_from_str(obtained, OBTAINED_TIME_FORMAT)
                .map_err(|err| LoadError::Parse(format!("Failed to parse {obtained}: {err}")))?;
            criteria.insert(
                name.clone(),
                CriterionProgressModel::from_obtained_epoch_millis(time.timestamp_millis()),
            );
        }
    }
    Ok(JavaAdvancementProgressModel::from_criteria(criteria))
}

/// `Data.CODEC` encode plus the `DataVersion` key, pretty printed like Gson.
pub(super) fn encode_progress_file<'a>(
    entries: impl IntoIterator<Item = (&'a Identifier, &'a JavaAdvancementProgressModel)>,
) -> String {
    let mut object = serde_json::Map::new();
    for (id, progress) in entries {
        object.insert(id.to_string(), encode_progress(progress));
    }
    object.insert(
        "DataVersion".to_string(),
        serde_json::Value::from(TARGET_DATA_VERSION),
    );
    // Serializing a `Value` cannot fail.
    serde_json::to_string_pretty(&serde_json::Value::Object(object)).unwrap_or_default()
}

/// `AdvancementProgress.CODEC` encode: only obtained criteria, in the system zone.
fn encode_progress(progress: &JavaAdvancementProgressModel) -> serde_json::Value {
    let criteria: serde_json::Map<String, serde_json::Value> = progress
        .get_completed_criteria()
        .into_iter()
        .filter_map(|name| {
            let millis = progress.get_criterion(&name)?.get_obtained_epoch_millis()?;
            let time = Local.timestamp_millis_opt(millis).single()?;
            Some((
                name,
                serde_json::Value::String(time.format(OBTAINED_TIME_FORMAT).to_string()),
            ))
        })
        .collect();
    serde_json::json!({ "criteria": criteria, "done": progress.is_done() })
}
