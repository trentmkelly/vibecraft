//! Recipe loading through the pack [`ResourceManager`]: `RecipeManager.prepare`
//! over every enabled data pack (not only the bundled vanilla directory).
//!
//! Java: `RecipeManager.prepare` runs `SimpleJsonResourceReloadListener.scanDirectory`
//! over `recipe/`. The `MultiPackResourceManager` hands back the highest-priority
//! copy of each id, so a datapack overrides a vanilla recipe by shipping a file with
//! the same id. A copy that fails to parse is logged and skipped, which is also how a
//! datapack removes a recipe (the override shadows the vanilla copy but produces no
//! recipe).

use std::collections::BTreeMap;

use super::*;
use crate::registry::Identifier;
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};

/// `FileToIdConverter.registry(Registries.RECIPE)`: the `recipe` directory.
const RECIPE_DIRECTORY: &str = "recipe";
/// The advancement directory scanned for recipe-unlocking advancements.
const ADVANCEMENT_DIRECTORY: &str = "advancement";

impl ItemTagMap {
    /// Builds a map from fully resolved item tags (`TagLoader.loadAndBuild` output for
    /// the item registry): tag id to its item ids. Ids are interned as `&'static str`
    /// like the rest of the recipe system.
    pub fn from_resolved_tags(tags: &BTreeMap<Identifier, Vec<Identifier>>) -> Self {
        let tags = tags
            .iter()
            .map(|(tag, items)| {
                let items = items
                    .iter()
                    .map(|item| &*Box::leak(item.to_string().into_boxed_str()))
                    .collect();
                (tag.to_string(), items)
            })
            .collect();
        Self { tags }
    }
}

/// `RecipeManager.prepare` + `RecipeMap.create` over `manager`, plus the recipe-book
/// unlock tables derived from the packs' recipe advancements. Problems Java logs
/// (`Parsing error loading recipe`) are appended to `problems` and the recipe skipped.
pub fn load_recipe_manager_from_resources(
    manager: &ResourceManager,
    tags: &ItemTagMap,
    problems: &mut Vec<String>,
) -> RecipeManagerModel {
    let converter = FileToIdConverter::json(RECIPE_DIRECTORY);
    let mut recipes = Vec::new();
    for (location, resource) in manager.list_matching_resources(&converter) {
        let Ok(id) = converter.file_to_id(&location) else {
            continue;
        };
        let loaded = resource
            .read_to_string()
            .map_err(|err| err.to_string())
            .and_then(|raw| {
                let recipe_id: &'static str = Box::leak(id.to_string().into_boxed_str());
                load_recipe_json(recipe_id, &raw, tags)
            });
        match loaded {
            Ok(holder) => recipes.push(holder),
            Err(err) => problems.push(format!(
                "Parsing error loading recipe {id} from {location} in data pack {}: {err}",
                resource.source_pack_id()
            )),
        }
    }

    let unlocks = load_recipe_unlocks_from_resources(manager, tags, problems);
    let mut recipe_manager = RecipeManagerModel::new(recipes);
    recipe_manager.set_acquisition_unlocks(unlocks.acquisition);
    recipe_manager.set_initial_unlocks(unlocks.initial);
    recipe_manager
}

/// The unlock tables of every advancement that rewards recipes. The bundled vanilla
/// pack ships no `advancement/` tree, so the decompiled vanilla recipe advancements
/// (when available at build time) form the base layer that pack advancements of the
/// same id replace.
fn load_recipe_unlocks_from_resources(
    manager: &ResourceManager,
    tags: &ItemTagMap,
    problems: &mut Vec<String>,
) -> LoadedRecipeUnlocks {
    let mut raw: BTreeMap<Identifier, (String, String)> = BTreeMap::new();
    if let Some(dir) = optional_decompiled_recipe_advancement_dir() {
        collect_fallback_advancements(&dir, &mut raw, problems);
    }
    let converter = FileToIdConverter::json(ADVANCEMENT_DIRECTORY);
    for (location, resource) in manager.list_matching_resources(&converter) {
        let Ok(id) = converter.file_to_id(&location) else {
            continue;
        };
        match resource.read_to_string() {
            Ok(text) => {
                raw.insert(id, (resource.source_pack_id().to_string(), text));
            }
            Err(err) => problems.push(format!("Couldn't read advancement {id}: {err}")),
        }
    }

    let mut unlocks = LoadedRecipeUnlocks::default();
    for (id, (pack, text)) in raw {
        let outcome = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|err| err.to_string())
            .and_then(|value| collect_recipe_unlocks(&value, tags, &mut unlocks));
        if let Err(err) = outcome {
            problems.push(format!(
                "Couldn't read recipe unlocks of advancement {id} in data pack {pack}: {err}"
            ));
        }
    }
    unlocks
}

/// Reads the vanilla recipe advancements from `dir` (`.../advancement/recipes`) into
/// `raw`, keyed by `minecraft:recipes/<relative path>`.
fn collect_fallback_advancements(
    dir: &std::path::Path,
    raw: &mut BTreeMap<Identifier, (String, String)>,
    problems: &mut Vec<String>,
) {
    let base = dir.parent().unwrap_or(dir);
    let paths = match advancement_paths(dir) {
        Ok(paths) => paths,
        Err(err) => {
            problems.push(err);
            return;
        }
    };
    for path in paths {
        let Ok(relative) = path.strip_prefix(base) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let Some(name) = relative.strip_suffix(".json") else {
            continue;
        };
        let Ok(id) = Identifier::parse(&format!("minecraft:{name}")) else {
            continue;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                raw.insert(id, ("vanilla".to_string(), text));
            }
            Err(err) => problems.push(format!("Couldn't read {}: {err}", path.display())),
        }
    }
}
