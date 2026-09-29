//! `Advancement.CODEC` as a strict JSON decoder/re-encoder.
//!
//! Java: `net.minecraft.advancements.Advancement.CODEC`, `Criterion.CODEC`
//! (`dispatchOptionalValue("trigger", "conditions", ...)`), `AdvancementRequirements.CODEC`,
//! `AdvancementRewards.CODEC` and `DisplayInfo.CODEC`. Decoding follows DFU semantics
//! (unknown fields are errors, defaults are re-encoded away); the per-trigger `conditions`
//! payload is handled by [`ConditionCodec`].
//!
//! The runtime model (`AdvancementDefinition`) keeps only what the server needs; this codec
//! keeps the *whole* document so that `encode(decode(x))` can be compared with `x`.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::advancement_condition_schema::{BuiltinLookup, ConditionCodec, Rec, RegistryLookup};
use crate::advancement_criteria::AdvancementRequirementsModel;
use crate::registry::Identifier;

/// Decoder for whole advancement documents.
pub(crate) struct AdvancementCodec<'a> {
    conditions: ConditionCodec<'a>,
}

impl<'a> AdvancementCodec<'a> {
    /// Creates a codec resolving registry references through `lookup`.
    pub(crate) fn new(lookup: &'a dyn RegistryLookup) -> Self {
        Self { conditions: ConditionCodec::new(lookup) }
    }

    /// Decodes `document` and returns the canonical JSON Java would encode the resulting
    /// `Advancement` to. Errors carry a field path.
    pub(crate) fn round_trip(&self, document: &Value) -> Result<Value, String> {
        let object = document.as_object().ok_or("advancement must be a JSON object")?;
        if let Some(key) = object.keys().find(|key| !TOP_LEVEL_FIELDS.contains(&key.as_str())) {
            return Err(format!("unknown field `{key}`"));
        }
        let mut out = Map::new();
        if let Some(parent) = object.get("parent") {
            let raw = parent.as_str().ok_or("parent: expected an id string")?;
            out.insert("parent".to_string(), Value::from(Identifier::parse(raw)?.to_string()));
        }
        self.insert_record(&mut out, object, "display", Rec::Display, false)?;
        self.insert_record(&mut out, object, "rewards", Rec::Rewards, true)?;
        let criteria = self.criteria(object.get("criteria"))?;
        let names = criteria.keys().cloned().collect::<BTreeSet<_>>();
        out.insert("criteria".to_string(), Value::Object(criteria));
        out.insert("requirements".to_string(), Self::requirements(object.get("requirements"), &names)?);
        if let Some(telemetry) = object.get("sends_telemetry_event") {
            let flag = telemetry.as_bool().ok_or("sends_telemetry_event: expected a boolean")?;
            if flag {
                out.insert("sends_telemetry_event".to_string(), Value::Bool(true));
            }
        }
        Ok(Value::Object(out))
    }

    /// Decodes an optional record field; `omit_empty` drops it when it equals the default.
    fn insert_record(
        &self,
        out: &mut Map<String, Value>,
        object: &Map<String, Value>,
        name: &str,
        rec: Rec,
        omit_empty: bool,
    ) -> Result<(), String> {
        let Some(raw) = object.get(name) else {
            return Ok(());
        };
        let encoded = self.conditions.decode_rec(rec, raw).map_err(|err| format!("{name}: {err}"))?;
        if !(omit_empty && encoded.as_object().is_some_and(Map::is_empty)) {
            out.insert(name.to_string(), encoded);
        }
        Ok(())
    }

    /// `CRITERIA_CODEC`: a non-empty map of `Criterion`.
    fn criteria(&self, value: Option<&Value>) -> Result<Map<String, Value>, String> {
        let object = value
            .ok_or("missing required field `criteria`")?
            .as_object()
            .ok_or("criteria: expected a JSON object")?;
        if object.is_empty() {
            return Err("criteria: Advancement criteria cannot be empty".to_string());
        }
        let mut out = Map::new();
        for (name, criterion) in object {
            let encoded = self.criterion(criterion).map_err(|err| format!("criteria.{name}: {err}"))?;
            out.insert(name.clone(), encoded);
        }
        Ok(out)
    }

    /// `Criterion.CODEC`: `{trigger, conditions?}` dispatched on the trigger type.
    fn criterion(&self, value: &Value) -> Result<Value, String> {
        let object = value.as_object().ok_or("expected a JSON object")?;
        if let Some(key) = object.keys().find(|key| !matches!(key.as_str(), "trigger" | "conditions")) {
            return Err(format!("unknown field `{key}`"));
        }
        let trigger = object
            .get("trigger")
            .ok_or("missing required field `trigger`")?
            .as_str()
            .ok_or("trigger: expected an id string")?;
        let trigger = Identifier::parse(trigger)?;
        let conditions = self
            .conditions
            .conditions(&trigger, object.get("conditions"))
            .map_err(|err| format!("conditions: {err}"))?;
        let mut out = Map::new();
        out.insert("trigger".to_string(), Value::from(trigger.to_string()));
        if let Some(conditions) = conditions {
            out.insert("conditions".to_string(), conditions);
        }
        Ok(Value::Object(out))
    }

    /// `AdvancementRequirements.CODEC` plus `Advancement.validate`; an absent field means
    /// `AdvancementRequirements.allOf(criteria.keySet())`.
    fn requirements(value: Option<&Value>, criteria: &BTreeSet<String>) -> Result<Value, String> {
        let groups = match value {
            None => criteria.iter().map(|name| vec![name.clone()]).collect(),
            Some(value) => {
                let mut groups = Vec::new();
                for group in value.as_array().ok_or("requirements: expected a JSON list")? {
                    let items = group.as_array().ok_or("requirements: expected a list of lists")?;
                    groups.push(
                        items
                            .iter()
                            .map(|item| {
                                item.as_str()
                                    .map(ToString::to_string)
                                    .ok_or_else(|| "requirements: expected strings".to_string())
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                groups
            }
        };
        let model = AdvancementRequirementsModel::new(groups);
        model.validate(criteria).map_err(|err| format!("requirements: {err}"))?;
        Ok(Value::Array(
            model
                .requirements()
                .iter()
                .map(|group| Value::Array(group.iter().cloned().map(Value::String).collect()))
                .collect(),
        ))
    }
}

/// Fields `Advancement.CODEC` accepts.
const TOP_LEVEL_FIELDS: [&str; 6] =
    ["parent", "display", "rewards", "criteria", "requirements", "sends_telemetry_event"];

/// Strictly validates `document` against `Advancement.CODEC` using only the static
/// registries, for the live loader (`AdvancementDefinition::from_json`).
pub(crate) fn validate_builtin(id: &str, document: &Value) -> Result<(), String> {
    AdvancementCodec::new(&BuiltinLookup)
        .round_trip(document)
        .map(drop)
        .map_err(|err| format!("invalid advancement {id}: {err}"))
}
