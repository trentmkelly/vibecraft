//! Static (built-in) registries: `BuiltInRegistries`.
//!
//! The element sets and protocol ids come from the vanilla `registries.json` data
//! generator report vendored at `vanilla-data/reports/registries_26_1_2.json`
//! (`Registry.getId` for every `BuiltInRegistries` entry). They serve two purposes:
//! resolving holders that data-pack codecs reference (`SoundEvent.CODEC`, ...), and
//! providing the numeric ids that tag payloads (`TagNetworkSerialization`) use.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde_json::Value;

use crate::registry::Identifier;

const REGISTRIES_REPORT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vanilla-data/reports/registries_26_1_2.json"
));

/// One static registry: its elements in protocol-id order.
#[derive(Debug, Clone)]
pub struct BuiltinRegistry {
    key: Identifier,
    elements: Vec<Identifier>,
    ids: HashMap<Identifier, usize>,
}

impl BuiltinRegistry {
    /// The registry's own key, e.g. `minecraft:block`.
    pub fn key(&self) -> &Identifier {
        &self.key
    }

    /// Elements ordered by protocol id.
    pub fn elements(&self) -> &[Identifier] {
        &self.elements
    }

    /// `Registry.getId(Identifier)`.
    pub fn id_of(&self, element: &Identifier) -> Option<usize> {
        self.ids.get(element).copied()
    }
}

/// All static registries known to the report.
#[derive(Debug, Clone)]
pub struct BuiltinRegistries {
    registries: BTreeMap<Identifier, BuiltinRegistry>,
}

impl BuiltinRegistries {
    /// The vanilla 26.1.2 static registries.
    pub fn vanilla() -> Result<&'static Self, String> {
        static VANILLA: OnceLock<Result<BuiltinRegistries, String>> = OnceLock::new();
        VANILLA
            .get_or_init(|| Self::from_report(REGISTRIES_REPORT))
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Parses a `registries.json` report.
    pub fn from_report(report: &str) -> Result<Self, String> {
        let root: Value = serde_json::from_str(report)
            .map_err(|err| format!("registries report is not valid JSON: {err}"))?;
        let object = root
            .as_object()
            .ok_or_else(|| "registries report must be a JSON object".to_string())?;
        let mut registries = BTreeMap::new();
        for (name, body) in object {
            let key = Identifier::parse(name)
                .map_err(|err| format!("invalid registry name {name}: {err}"))?;
            let entries = body
                .get("entries")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("registry {name} has no entries object"))?;
            let mut by_id = Vec::with_capacity(entries.len());
            for (element, info) in entries {
                let id = Identifier::parse(element)
                    .map_err(|err| format!("invalid element {element} in {name}: {err}"))?;
                let protocol_id = info
                    .get("protocol_id")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| format!("{name}/{element} has no protocol_id"))?;
                by_id.push((protocol_id, id));
            }
            by_id.sort_by_key(|(protocol_id, _)| *protocol_id);
            for (expected, (actual, element)) in by_id.iter().enumerate() {
                if *actual != expected as u64 {
                    return Err(format!(
                        "registry {name} protocol ids are not contiguous at {element} \
                         (expected {expected}, found {actual})"
                    ));
                }
            }
            let elements: Vec<Identifier> = by_id.into_iter().map(|(_, id)| id).collect();
            let ids = elements
                .iter()
                .enumerate()
                .map(|(index, id)| (id.clone(), index))
                .collect();
            registries.insert(key.clone(), BuiltinRegistry { key, elements, ids });
        }
        Ok(Self { registries })
    }

    /// Looks a static registry up by key.
    pub fn get(&self, registry: &Identifier) -> Option<&BuiltinRegistry> {
        self.registries.get(registry)
    }

    /// Whether `registry` is a static registry.
    pub fn contains_registry(&self, registry: &Identifier) -> bool {
        self.registries.contains_key(registry)
    }

    /// Whether `registry` holds `element`.
    pub fn contains_element(&self, registry: &Identifier, element: &Identifier) -> bool {
        self.get(registry)
            .is_some_and(|found| found.id_of(element).is_some())
    }

    /// Every static registry, ordered by key.
    pub fn iter(&self) -> impl Iterator<Item = &BuiltinRegistry> {
        self.registries.values()
    }
}
