//! The reloadable data-pack content that sits next to the registries: recipes, loot
//! tables, advancements and functions (Java `ReloadableServerResources`).
//!
//! Java builds all of it from one `MultiPackResourceManager` in
//! `ReloadableServerResources.loadResources` (loot data through
//! `ReloadableServerRegistries.reload`; the `RecipeManager`, `ServerFunctionLibrary`
//! and `ServerAdvancementManager` reload listeners afterwards) and swaps the finished
//! bundle in only when every listener succeeded. [`DatapackContent::load`] is that
//! bundle: it never mutates server state, so a reload that fails leaves the previous
//! content in force.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::advancement_system::AdvancementDefinition;
use crate::log::{log_error, log_info, log_warn};
use crate::loot_system::{decode_loot_table, LootTable};
use crate::recipe_system::{load_recipe_manager_from_resources, ItemTagMap, RecipeManagerModel};
use crate::registry::Identifier;
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};
use crate::registry_pipeline::store::Registries;
use crate::registry_pipeline::tags::load_tags_for_registry;
use crate::server_advancement_manager::ServerAdvancementManagerModel;
use crate::server_function_library::ServerFunctionLibrary;

/// `Registries.elementsDirPath(Registries.LOOT_TABLE)`.
const LOOT_TABLE_DIRECTORY: &str = "loot_table";
/// `Registries.elementsDirPath(Registries.ADVANCEMENT)`.
const ADVANCEMENT_DIRECTORY: &str = "advancement";

/// The loot tables the enabled packs define (`LootDataType.TABLE` registry).
///
/// Java's `ReloadableServerRegistries.Holder.getLootTable` answers `LootTable.EMPTY`
/// for an id that no pack defines. The bundled vanilla pack does not ship loot table
/// JSON yet (vanilla drops still come from built-in Rust tables), so this holds only
/// the tables packs define; callers fall back to the built-in table for anything
/// absent. TODO(vanilla-loot-table-json): bundle vanilla `loot_table/` so this holder
/// becomes the single source, as in Java.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LootTables {
    tables: BTreeMap<Identifier, LootTable>,
}

impl LootTables {
    /// The pack-defined table for `id` (`minecraft:blocks/stone`).
    pub fn get(&self, id: &str) -> Option<&LootTable> {
        Identifier::parse(id)
            .ok()
            .and_then(|id| self.tables.get(&id))
    }

    /// `scheduleRegistryLoad` for `LootDataType.TABLE`: the highest-priority copy of
    /// every `loot_table/**.json`; tables that fail to decode are reported to
    /// `problems` and skipped.
    fn load(manager: &ResourceManager, problems: &mut Vec<String>) -> Self {
        let converter = FileToIdConverter::json(LOOT_TABLE_DIRECTORY);
        let mut tables = BTreeMap::new();
        for (location, resource) in manager.list_matching_resources(&converter) {
            let Ok(id) = converter.file_to_id(&location) else {
                continue;
            };
            let decoded = resource
                .read_to_string()
                .map_err(|err| err.to_string())
                .and_then(|raw| decode_loot_table(&raw));
            match decoded {
                Ok(table) => {
                    tables.insert(id, table);
                }
                Err(err) => problems.push(format!(
                    "Couldn't parse element loot_table/{id} from {location} in data pack {}: {err}",
                    resource.source_pack_id()
                )),
            }
        }
        Self { tables }
    }
}

/// Everything `ReloadableServerResources` holds besides the registries.
#[derive(Debug, Clone)]
pub struct DatapackContent {
    /// `RecipeManager`: recipes and recipe-book unlock tables.
    pub recipes: Arc<RecipeManagerModel>,
    /// `ReloadableServerRegistries.Holder`'s loot tables.
    pub loot_tables: LootTables,
    /// `ServerAdvancementManager`: the advancement tree. Shared (not copied) with each
    /// player's `PlayerAdvancements`, which detects a `/reload` by pointer identity.
    pub advancements: Arc<ServerAdvancementManagerModel>,
    /// `ServerFunctionLibrary`: functions and function tags. `/function` and
    /// `execute ... function` read it; `minecraft:load`/`minecraft:tick` and
    /// `/schedule function` need a live function tick.
    /// TODO(function-tick-live): drive `ServerFunctionManager` from the server tick.
    pub functions: ServerFunctionLibrary,
}

impl DatapackContent {
    /// Loads every reloadable component from `manager`, in the order Java runs them:
    /// loot data, then recipes, the function library and advancements. Problems the
    /// components log and recover from are written to the server log.
    pub fn load(manager: &ResourceManager, registries: &Registries) -> Result<Self, String> {
        let mut problems = Vec::new();
        let loot_tables = LootTables::load(manager, &mut problems);
        let recipes = Arc::new(load_recipes(manager, registries, &mut problems)?);
        let functions = ServerFunctionLibrary::from_resource_manager(manager);
        let advancements = load_advancements(manager, &mut problems);

        for problem in problems.iter().chain(functions.errors()) {
            log_error(problem);
        }
        for warning in advancements.validation_warnings() {
            log_warn(warning);
        }
        // `RecipeManager.apply`: `LOGGER.info("Loaded {} recipes", ...)`.
        log_info(&format!(
            "Loaded {} recipes",
            recipes.recipe_map().values().len()
        ));
        Ok(Self {
            recipes,
            loot_tables,
            advancements: Arc::new(advancements),
            functions,
        })
    }
}

/// Item tags resolved against the item registry, then the recipes that use them.
fn load_recipes(
    manager: &ResourceManager,
    registries: &Registries,
    problems: &mut Vec<String>,
) -> Result<RecipeManagerModel, String> {
    let item_key = Identifier::parse("minecraft:item")?;
    let items = registries
        .lookup(&item_key)
        .ok_or_else(|| "the item registry is not loaded".to_string())?;
    let mut known = |id: &Identifier, _required: bool| items.contains(id);
    let tags = load_tags_for_registry(manager, "item", &mut known, problems);
    let tags = ItemTagMap::from_resolved_tags(&tags);
    Ok(load_recipe_manager_from_resources(manager, &tags, problems))
}

/// `ServerAdvancementManager.prepare` + `apply`: every `advancement/**.json`, the
/// highest-priority copy per id; documents that fail to decode are reported and
/// skipped.
fn load_advancements(
    manager: &ResourceManager,
    problems: &mut Vec<String>,
) -> ServerAdvancementManagerModel {
    let converter = FileToIdConverter::json(ADVANCEMENT_DIRECTORY);
    let mut definitions: BTreeMap<Identifier, AdvancementDefinition> = BTreeMap::new();
    for (location, resource) in manager.list_matching_resources(&converter) {
        let Ok(id) = converter.file_to_id(&location) else {
            continue;
        };
        let decoded = resource
            .read_to_string()
            .map_err(|err| err.to_string())
            .and_then(|raw| AdvancementDefinition::from_json(&id.to_string(), &raw));
        match decoded {
            Ok(definition) => {
                definitions.insert(id, definition);
            }
            Err(err) => problems.push(format!(
                "Couldn't parse element {ADVANCEMENT_DIRECTORY}/{id} from {location} in data pack {}: {err}",
                resource.source_pack_id()
            )),
        }
    }
    let mut advancements = ServerAdvancementManagerModel::default();
    advancements.apply(definitions);
    advancements
}
