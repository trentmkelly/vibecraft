//! `RegistryDataLoader.load`: decodes registry elements and tags from a
//! [`ResourceManager`], freezes the registries and validates them, collecting
//! errors the way Java's `loadingErrors` map does.

use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;
use crate::registry_pipeline::builtin::{BuiltinRegistries, BuiltinRegistry};
use crate::registry_pipeline::codec::{Codec, CodecContext, References};
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};
use crate::registry_pipeline::store::{MappedRegistry, RegistrationInfo};
use crate::registry_pipeline::tags::load_tags_for_registry;
use crate::resource_registry_data_loader::{
    RegistryDataLoaderRegistryData, RegistryDataLoaderValidatorKind,
};

/// `ResourceKey.registry()` of a registry key: `minecraft:root`.
const ROOT_REGISTRY: &str = "minecraft:root";

/// The key of a loading error: `(ResourceKey.registry(), ResourceKey.identifier())`.
pub type ErrorKey = (Identifier, Identifier);

/// One entry of Java's `loadingErrors` map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadingError {
    /// `Exception.getMessage()`.
    pub message: String,
    /// The wrapped cause printed by the full log (`Caused by: ...`).
    pub cause: Option<String>,
}

/// `IllegalStateException("Failed to load registries due to errors")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadError {
    /// Every error, keyed like Java's map.
    pub errors: BTreeMap<ErrorKey, LoadingError>,
    /// Non-fatal problems logged while loading (`LOGGER.error` in `TagLoader`).
    pub logged: Vec<String>,
}

impl RegistryLoadError {
    /// `createReportWithBriefInfo`: `\n\t\t<registry>/<id>: <message>` per error,
    /// ordered by `ERROR_KEY_COMPARATOR`.
    pub fn brief_details(&self) -> String {
        self.errors
            .iter()
            .map(|((registry, id), error)| format!("\n\t\t{registry}/{id}: {}", error.message))
            .collect()
    }

    /// `printFullDetailsToLog`: errors grouped by registry then element.
    pub fn full_details(&self) -> String {
        let mut by_registry: BTreeMap<&Identifier, Vec<(&Identifier, &LoadingError)>> =
            BTreeMap::new();
        for ((registry, id), error) in &self.errors {
            by_registry.entry(registry).or_default().push((id, error));
        }
        let mut out = String::new();
        for (registry, elements) in by_registry {
            out.push_str(&format!("> Errors in registry {registry}:\n"));
            for (id, error) in elements {
                out.push_str(&format!(">> Errors in element {id}:\n{}\n", error.message));
                if let Some(cause) = &error.cause {
                    out.push_str(&format!("Caused by: {cause}\n"));
                }
            }
        }
        out
    }
}

impl std::fmt::Display for RegistryLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Registry Loading: Failed to load registries due to errors\nErrors:{}",
            self.brief_details()
        )
    }
}

impl std::error::Error for RegistryLoadError {}

/// The two codecs of one registry.
#[derive(Clone)]
pub struct ElementCodecs {
    /// The codec elements are decoded with when loaded from a pack.
    pub direct: Codec,
    /// The codec elements are encoded with for the network.
    pub network: Codec,
}

/// A registry to load (`RegistryDataLoader.RegistryData` with its resolved codecs).
pub struct LoadTask<'a> {
    /// The static table entry.
    pub data: &'a RegistryDataLoaderRegistryData,
    /// The resolved codecs.
    pub codecs: ElementCodecs,
}

fn registry_key(data: &RegistryDataLoaderRegistryData) -> Result<Identifier, String> {
    Identifier::parse(data.key).map_err(|err| format!("invalid registry key {}: {err}", data.key))
}

/// `Registries.elementsDirPath`: the registry key's path.
fn registry_path(key: &Identifier) -> &str {
    key.path()
}

/// `ResourceKey.toString` for a registry key.
fn registry_key_string(key: &Identifier) -> String {
    format!("ResourceKey[{ROOT_REGISTRY} / {key}]")
}

struct LoadedRegistry {
    registry: MappedRegistry,
    validator: RegistryDataLoaderValidatorKind,
}

fn root() -> Identifier {
    // `minecraft:root` is a valid identifier by construction.
    Identifier::parse(ROOT_REGISTRY).unwrap_or_else(|_| unreachable_root())
}

#[cold]
fn unreachable_root() -> Identifier {
    panic!("the constant {ROOT_REGISTRY} must be a valid identifier")
}

/// The result of a successful [`load`].
#[derive(Debug)]
pub struct LoadedRegistries {
    /// The frozen registries, in task order.
    pub registries: Vec<MappedRegistry>,
    /// Non-fatal problems Java reports with `LOGGER.error` (tags that failed to
    /// build because of missing references, unreadable tag files).
    pub logged: Vec<String>,
}

/// Loads `tasks` from `manager` (`RegistryDataLoader.load(resourceManager, ...)`).
///
/// Registries are returned frozen, in task order. Element ids follow the sorted
/// resource-file order Java registers in.
pub fn load(
    manager: &ResourceManager,
    builtin: &BuiltinRegistries,
    tasks: &[LoadTask<'_>],
) -> Result<LoadedRegistries, RegistryLoadError> {
    let mut errors: BTreeMap<ErrorKey, LoadingError> = BTreeMap::new();
    let mut logged: Vec<String> = Vec::new();

    let mut keys = Vec::with_capacity(tasks.len());
    let mut loading = BTreeSet::new();
    for task in tasks {
        match registry_key(task.data) {
            Ok(key) => {
                loading.insert(key.clone());
                keys.push(key);
            }
            Err(message) => {
                errors.insert(
                    (root(), root()),
                    LoadingError {
                        message,
                        cause: None,
                    },
                );
                return Err(RegistryLoadError { errors, logged });
            }
        }
    }

    let ctx = CodecContext::new(builtin, &loading);
    let mut loaded: Vec<LoadedRegistry> = Vec::with_capacity(tasks.len());
    for (task, key) in tasks.iter().zip(&keys) {
        let mut registry = MappedRegistry::new(key.clone());
        load_elements(manager, task, key, &ctx, &mut registry, &mut errors);
        loaded.push(LoadedRegistry {
            registry,
            validator: task.data.validator,
        });
    }

    // Tags are bound after every registry's elements exist, and required element
    // lookups record references exactly like `ConcurrentHolderGetter`.
    let mut tag_element_refs: BTreeSet<(Identifier, Identifier)> = BTreeSet::new();
    for loaded_registry in &mut loaded {
        bind_tags(
            manager,
            &mut loaded_registry.registry,
            &mut tag_element_refs,
            &mut logged,
        );
    }

    let mut references = ctx.take_references();
    references.elements.extend(tag_element_refs);
    let mut frozen = Vec::with_capacity(loaded.len());
    for loaded_registry in loaded {
        let LoadedRegistry {
            mut registry,
            validator,
        } = loaded_registry;
        if freeze(&mut registry, &references, &mut errors) {
            frozen.push((registry, validator));
        }
    }
    if !errors.is_empty() {
        return Err(RegistryLoadError { errors, logged });
    }

    for (registry, validator) in &frozen {
        validate(registry, *validator, &mut errors);
    }
    if !errors.is_empty() {
        return Err(RegistryLoadError { errors, logged });
    }
    Ok(LoadedRegistries {
        registries: frozen.into_iter().map(|(registry, _)| registry).collect(),
        logged,
    })
}

/// `ResourceManagerRegistryLoadTask.load` (element half).
fn load_elements(
    manager: &ResourceManager,
    task: &LoadTask<'_>,
    key: &Identifier,
    ctx: &CodecContext<'_>,
    registry: &mut MappedRegistry,
    errors: &mut BTreeMap<ErrorKey, LoadingError>,
) {
    let converter = FileToIdConverter::json(registry_path(key));
    for (file, resource) in manager.list_matching_resources(&converter) {
        let Ok(element_key) = converter.file_to_id(&file) else {
            continue;
        };
        let pack = resource.source_pack_id().to_string();
        let parsed = resource
            .read_to_string()
            .map_err(|err| err.to_string())
            .and_then(|text| {
                serde_json::from_str::<serde_json::Value>(&text).map_err(|err| err.to_string())
            })
            .and_then(|json| task.codecs.direct.parse(&json, ctx).map(|_| json));
        match parsed {
            Ok(json) => {
                let info = RegistrationInfo::for_pack(resource.known_pack_info().cloned());
                if let Err(message) = registry.register(element_key.clone(), json, info) {
                    errors.insert(
                        (key.clone(), element_key),
                        LoadingError {
                            message,
                            cause: None,
                        },
                    );
                }
            }
            Err(cause) => {
                errors.insert(
                    (key.clone(), element_key.clone()),
                    LoadingError {
                        message: format!("Failed to parse {element_key} from pack {pack}"),
                        cause: Some(cause),
                    },
                );
            }
        }
    }
}

/// `TagLoader.loadTagsForRegistry` + `registerTags` for one loaded registry.
fn bind_tags(
    manager: &ResourceManager,
    registry: &mut MappedRegistry,
    element_refs: &mut BTreeSet<(Identifier, Identifier)>,
    logged: &mut Vec<String>,
) {
    let key = registry.key().clone();
    let known: BTreeSet<Identifier> = registry.elements().iter().map(|e| e.key.clone()).collect();
    let mut lookup = |id: &Identifier, required: bool| {
        if required {
            // The registration getter hands out a reference even for unknown ids;
            // freezing reports the unbound ones.
            element_refs.insert((key.clone(), id.clone()));
            true
        } else {
            known.contains(id)
        }
    };
    let tags = load_tags_for_registry(manager, registry_path(&key), &mut lookup, logged);
    for (tag, elements) in tags {
        let ids = elements
            .iter()
            .filter_map(|element| registry.id_of(element))
            .collect();
        registry.bind_tag(tag, ids);
    }
}

/// `MappedRegistry.freeze` checks: unbound values and unbound tags.
fn freeze(
    registry: &mut MappedRegistry,
    references: &References,
    errors: &mut BTreeMap<ErrorKey, LoadingError>,
) -> bool {
    let key = registry.key().clone();
    let unbound_values: BTreeSet<&Identifier> = references
        .elements
        .iter()
        .filter(|(target, id)| *target == key && !registry.contains(id))
        .map(|(_, id)| id)
        .collect();
    let unbound_tags: BTreeSet<&Identifier> = references
        .tags
        .iter()
        .filter(|(target, tag)| *target == key && !registry.tags().contains_key(tag))
        .map(|(_, tag)| tag)
        .collect();
    let describe_list = |ids: &BTreeSet<&Identifier>| {
        format!(
            "[{}]",
            ids.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let message = if !unbound_values.is_empty() {
        Some(format!(
            "Unbound values in registry {}: {}",
            registry_key_string(&key),
            describe_list(&unbound_values)
        ))
    } else if !unbound_tags.is_empty() {
        Some(format!(
            "Unbound tags in registry {}: {}",
            registry_key_string(&key),
            describe_list(&unbound_tags)
        ))
    } else {
        None
    };
    if let Some(message) = message {
        errors.insert(
            (root(), key),
            LoadingError {
                message,
                cause: None,
            },
        );
        return false;
    }
    registry.freeze();
    true
}

/// `RegistryValidator.validate`.
fn validate(
    registry: &MappedRegistry,
    validator: RegistryDataLoaderValidatorKind,
    errors: &mut BTreeMap<ErrorKey, LoadingError>,
) {
    match validator {
        RegistryDataLoaderValidatorKind::None => {}
        RegistryDataLoaderValidatorKind::NonEmpty => {
            if registry.is_empty() {
                errors.insert(
                    (root(), registry.key().clone()),
                    LoadingError {
                        message: format!("Registry must be non-empty: {}", registry.key()),
                        cause: None,
                    },
                );
            }
        }
        RegistryDataLoaderValidatorKind::TimelineValidateRegistry => {
            validate_timeline_markers(registry, errors);
        }
    }
}

/// `Timeline.validateRegistry`: a time marker may be defined once per clock.
fn validate_timeline_markers(
    registry: &MappedRegistry,
    errors: &mut BTreeMap<ErrorKey, LoadingError>,
) {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for element in registry.elements() {
        let Some(clock) = element
            .json
            .get("clock")
            .and_then(serde_json::Value::as_str)
            .and_then(|text| Identifier::parse(text).ok())
        else {
            continue;
        };
        let Some(markers) = element
            .json
            .get("time_markers")
            .and_then(serde_json::Value::as_object)
        else {
            continue;
        };
        for marker in markers.keys() {
            let Ok(marker_id) = Identifier::parse(marker) else {
                continue;
            };
            if !seen.insert((clock.to_string(), marker_id.to_string())) {
                errors.insert(
                    (registry.key().clone(), element.key.clone()),
                    LoadingError {
                        message: format!(
                            "ResourceKey[minecraft:clock_time_marker / {marker_id}] was defined multiple times in {clock}"
                        ),
                        cause: None,
                    },
                );
            }
        }
    }
}

/// Builds the static registry layer: every built-in registry with its tags loaded
/// through `TagLoader.loadTagsForExistingRegistries` (`fromFrozenRegistry`, so
/// required and optional elements are both checked against the registry).
pub fn build_static_layer(
    manager: &ResourceManager,
    builtin: &BuiltinRegistries,
    logged: &mut Vec<String>,
) -> Vec<MappedRegistry> {
    builtin
        .iter()
        .map(|registry| static_registry(manager, registry, logged))
        .collect()
}

fn static_registry(
    manager: &ResourceManager,
    builtin: &BuiltinRegistry,
    logged: &mut Vec<String>,
) -> MappedRegistry {
    let mut registry = MappedRegistry::new(builtin.key().clone());
    for element in builtin.elements() {
        // Element keys are unique by construction of the report.
        let _ = registry.register(
            element.clone(),
            serde_json::Value::Null,
            RegistrationInfo::built_in(),
        );
    }
    let mut lookup = |id: &Identifier, _required: bool| builtin.id_of(id).is_some();
    let tags = load_tags_for_registry(manager, registry_path(builtin.key()), &mut lookup, logged);
    for (tag, elements) in tags {
        let ids = elements
            .iter()
            .filter_map(|element| builtin.id_of(element))
            .collect();
        registry.bind_tag(tag, ids);
    }
    registry.freeze();
    registry
}
