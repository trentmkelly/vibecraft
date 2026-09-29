//! `TagLoader`: loads `tags/<registry path>/**/*.json` from every pack, merges
//! them (honouring `replace`), resolves nested tag references in dependency order
//! (`DependencySorter`), and reports unresolved references the way Java does.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::registry::{Identifier, TagEntry};
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};

/// `TagLoader.EntryWithSource`.
#[derive(Debug, Clone)]
pub struct EntryWithSource {
    entry: TagEntry,
    source: String,
}

impl std::fmt::Display for EntryWithSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} (from {})", self.entry, self.source)
    }
}

/// `TagLoader.ElementLookup`: does `id` resolve to an element of the registry?
/// `required` selects Java's registration getter (which always succeeds and binds
/// later) versus the read-only registry.
pub type ElementLookup<'a> = dyn FnMut(&Identifier, bool) -> bool + 'a;

/// Parses a `TagFile` (`values` list plus optional `replace`).
fn parse_tag_file(json: &Value) -> Result<(Vec<TagEntry>, bool), String> {
    let object = json
        .as_object()
        .ok_or_else(|| format!("Not a map: {json}"))?;
    let values = object
        .get("values")
        .ok_or_else(|| format!("No key values in MapLike[{json}]"))?
        .as_array()
        .ok_or_else(|| format!("Not a list: {}", object["values"]))?;
    let entries = values
        .iter()
        .map(TagEntry::from_json)
        .collect::<Result<Vec<_>, _>>()?;
    let replace = match object.get("replace") {
        None => false,
        Some(Value::Bool(flag)) => *flag,
        Some(other) => return Err(format!("Not a boolean: {other}")),
    };
    Ok((entries, replace))
}

/// `TagLoader.load`: gathers the merged entry lists per tag id.
fn load_entries(
    manager: &ResourceManager,
    directory: &str,
    errors: &mut Vec<String>,
) -> BTreeMap<Identifier, Vec<EntryWithSource>> {
    let converter = FileToIdConverter::json(directory);
    let mut builders: BTreeMap<Identifier, Vec<EntryWithSource>> = BTreeMap::new();
    for (location, stack) in manager.list_matching_resource_stacks(&converter) {
        let Ok(id) = converter.file_to_id(&location) else {
            continue;
        };
        for resource in stack {
            let source = resource.source_pack_id().to_string();
            let parsed = resource
                .read_to_string()
                .map_err(|err| err.to_string())
                .and_then(|text| {
                    serde_json::from_str::<Value>(&text).map_err(|err| err.to_string())
                })
                .and_then(|json| parse_tag_file(&json));
            match parsed {
                Ok((entries, replace)) => {
                    let contents = builders.entry(id.clone()).or_default();
                    if replace {
                        contents.clear();
                    }
                    contents.extend(entries.into_iter().map(|entry| EntryWithSource {
                        entry,
                        source: source.clone(),
                    }));
                }
                Err(err) => errors.push(format!(
                    "Couldn't read tag list {id} from {location} in data pack {source}: {err}"
                )),
            }
        }
    }
    builders
}

/// `DependencySorter.isCyclic`.
fn is_cyclic(
    dependencies: &BTreeMap<Identifier, BTreeSet<Identifier>>,
    from: &Identifier,
    to: &Identifier,
) -> bool {
    dependencies.get(to).is_some_and(|direct| {
        direct.contains(from)
            || direct
                .iter()
                .any(|next| is_cyclic(dependencies, from, next))
    })
}

/// `DependencySorter.orderByDependencies`: ids ordered so dependencies come first.
fn order_by_dependencies(contents: &BTreeMap<Identifier, Vec<EntryWithSource>>) -> Vec<Identifier> {
    let mut dependencies: BTreeMap<Identifier, BTreeSet<Identifier>> = BTreeMap::new();
    for optional_pass in [false, true] {
        for (id, entries) in contents {
            let mut found = Vec::new();
            for entry in entries {
                if optional_pass {
                    entry.entry.visit_optional_dependencies(&mut found);
                } else {
                    entry.entry.visit_required_dependencies(&mut found);
                }
            }
            for dependency in found {
                if !is_cyclic(&dependencies, id, &dependency) {
                    dependencies
                        .entry(id.clone())
                        .or_default()
                        .insert(dependency);
                }
            }
        }
    }

    fn visit(
        id: &Identifier,
        dependencies: &BTreeMap<Identifier, BTreeSet<Identifier>>,
        contents: &BTreeMap<Identifier, Vec<EntryWithSource>>,
        visited: &mut BTreeSet<Identifier>,
        ordered: &mut Vec<Identifier>,
    ) {
        if !visited.insert(id.clone()) {
            return;
        }
        if let Some(direct) = dependencies.get(id) {
            for dependency in direct {
                visit(dependency, dependencies, contents, visited, ordered);
            }
        }
        if contents.contains_key(id) {
            ordered.push(id.clone());
        }
    }

    let mut visited = BTreeSet::new();
    let mut ordered = Vec::with_capacity(contents.len());
    for id in contents.keys() {
        visit(id, &dependencies, contents, &mut visited, &mut ordered);
    }
    ordered
}

/// `TagLoader.build`: resolves every tag into its element ids (in encounter order,
/// duplicates removed). Tags with unresolved required references are reported to
/// `errors` and omitted, exactly like Java's `LOGGER.error` path.
fn build(
    contents: &BTreeMap<Identifier, Vec<EntryWithSource>>,
    lookup: &mut ElementLookup<'_>,
    errors: &mut Vec<String>,
) -> BTreeMap<Identifier, Vec<Identifier>> {
    let mut new_tags: BTreeMap<Identifier, Vec<Identifier>> = BTreeMap::new();
    for id in order_by_dependencies(contents) {
        let Some(entries) = contents.get(&id) else {
            continue;
        };
        let mut values: Vec<Identifier> = Vec::new();
        let mut seen: BTreeSet<Identifier> = BTreeSet::new();
        let mut missing: Vec<&EntryWithSource> = Vec::new();
        for entry in entries {
            let TagEntry {
                id: target,
                tag,
                required,
            } = &entry.entry;
            let resolved: Option<Vec<Identifier>> = if *tag {
                new_tags.get(target).cloned()
            } else if lookup(target, *required) {
                Some(vec![target.clone()])
            } else {
                None
            };
            match resolved {
                Some(elements) => {
                    for element in elements {
                        if seen.insert(element.clone()) {
                            values.push(element);
                        }
                    }
                }
                None if !*required => {}
                None => missing.push(entry),
            }
        }
        if missing.is_empty() {
            new_tags.insert(id, values);
        } else {
            errors.push(format!(
                "Couldn't load tag {id} as it is missing following references: {}",
                missing
                    .iter()
                    .map(|entry| entry.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    new_tags
}

/// `TagLoader.loadTagsForRegistry`: loads and resolves every tag under
/// `tags/<registry path>` for one registry.
pub fn load_tags_for_registry(
    manager: &ResourceManager,
    registry_path: &str,
    lookup: &mut ElementLookup<'_>,
    errors: &mut Vec<String>,
) -> BTreeMap<Identifier, Vec<Identifier>> {
    let directory = format!("tags/{registry_path}");
    let contents = load_entries(manager, &directory, errors);
    build(&contents, lookup, errors)
}
