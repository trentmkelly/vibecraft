//! The server's live data-pack state: `PackRepository` + `WorldDataConfiguration`
//! plus the registries and resource manager loaded from the selected packs.
//!
//! - [`ServerResources::initialize`] is `WorldLoader.load`: the selected packs are
//!   opened, in selection order, into a [`ResourceManager`] and every data-driven
//!   registry is decoded from it (`RegistryDataLoader`).
//! - [`ServerResources::reload`] is `MinecraftServer.reloadResources`: the new packs'
//!   tags replace the tags of the existing registries (`TagLoader
//!   .loadTagsForExistingRegistries`); registry *elements* are not re-read (Java only
//!   applies datapack registry changes on restart). On success the selection and the
//!   world data configuration are updated and persisted; on failure nothing changes.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::log::log_warn;
use crate::recipe_system::RecipeManagerModel;
use crate::network::configuration::KnownPack;
use crate::registry::{FeatureFlagRegistry, FeatureFlagSet};
use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::registry_pipeline::datapack_content::DatapackContent;
use crate::registry_pipeline::resources::{PackResources, ResourceManager};
use crate::registry_pipeline::store::Registries;
use crate::registry_pipeline::{load_registries, loader};
use crate::resources::{
    DataPack, DataPackConfig, DataPackRepository, WorldDataConfiguration, VANILLA_PACK_ID,
};
use crate::storage::nbt::Tag;
use crate::storage::world::WorldLayout;

/// The registries and resources produced by one load or reload.
pub struct LoadedResources {
    /// The frozen registries (elements from startup, tags from the latest load).
    pub registries: Registries,
    /// Recipes, loot tables, advancements and functions of the selected packs
    /// (`ReloadableServerResources`).
    pub content: DatapackContent,
    manager: ResourceManager,
}

impl LoadedResources {
    /// `listPacks().knownPackInfo()`: the selected packs a client may already have.
    pub fn known_packs(&self) -> Vec<KnownPack> {
        self.manager.known_packs()
    }
}

/// Opens `ids` in order (`packsToEnable.map(getPack).filter(nonNull).map(Pack::open)`).
fn open_packs(
    repository: &DataPackRepository,
    ids: &[String],
) -> Result<Vec<Box<dyn PackResources>>, String> {
    ids.iter()
        .filter_map(|id| repository.pack(id))
        .map(|pack| pack.content.open(&pack.id, &pack.metadata))
        .collect()
}

/// `getSelectedPacks(packRepository, true)`.
fn selected_pack_config(repository: &DataPackRepository) -> DataPackConfig {
    let enabled = repository.selected_ids();
    let disabled = repository
        .available_ids()
        .into_iter()
        .filter(|id| !enabled.contains(id))
        .collect::<Vec<_>>();
    DataPackConfig::new(enabled, disabled)
}

struct State {
    repository: DataPackRepository,
    data_config: WorldDataConfiguration,
    world_root: PathBuf,
}

/// The server-lifetime data-pack state (`MinecraftServer.packRepository`,
/// `worldData.getDataConfiguration()` and `resources`).
pub struct ServerResources {
    state: Mutex<State>,
    current: RwLock<Arc<LoadedResources>>,
}

static SERVER_RESOURCES: OnceLock<ServerResources> = OnceLock::new();

/// A read-only view of the pack repository for command seeding.
#[derive(Debug, Clone, Default)]
pub struct PackListing {
    /// `PackRepository.getAvailableIds`.
    pub available: Vec<String>,
    /// `PackRepository.getSelectedIds`, lowest priority first.
    pub selected: Vec<String>,
    /// `worldData.getDataConfiguration().dataPacks().getDisabled()`.
    pub disabled: Vec<String>,
    /// Feature packs that request features (they cannot be disabled).
    pub feature_packs: Vec<String>,
    /// Packs requesting features the world does not enable.
    pub unavailable_feature_packs: Vec<String>,
}

impl ServerResources {
    /// `WorldLoader.load` for the packs selected in `repository`.
    pub fn new(
        repository: DataPackRepository,
        data_config: WorldDataConfiguration,
        world_root: &Path,
    ) -> Result<Self, String> {
        let loaded = Self::load(&repository)?;
        Ok(Self {
            state: Mutex::new(State {
                repository,
                data_config,
                world_root: world_root.to_path_buf(),
            }),
            current: RwLock::new(Arc::new(loaded)),
        })
    }

    /// [`Self::new`], installed as the process-wide server state; a second call
    /// returns the first instance.
    pub fn initialize(
        repository: DataPackRepository,
        data_config: WorldDataConfiguration,
        world_root: &Path,
    ) -> Result<&'static ServerResources, String> {
        let resources = Self::new(repository, data_config, world_root)?;
        Ok(SERVER_RESOURCES.get_or_init(|| resources))
    }

    /// The installed instance, when the server started through [`Self::initialize`].
    pub fn installed() -> Option<&'static ServerResources> {
        SERVER_RESOURCES.get()
    }

    fn load(repository: &DataPackRepository) -> Result<LoadedResources, String> {
        let manager = ResourceManager::new(open_packs(repository, &repository.selected_ids())?);
        let builtin = BuiltinRegistries::vanilla()?;
        let registries = load_registries(&manager, builtin)?;
        let content = DatapackContent::load(&manager, &registries)?;
        Ok(LoadedResources {
            registries,
            content,
            manager,
        })
    }

    /// The resources currently in force.
    pub fn current(&self) -> Arc<LoadedResources> {
        Arc::clone(&self.current.read().unwrap_or_else(|e| e.into_inner()))
    }

    /// `PackRepository` state for `/datapack` and `/reload`. Re-scans the world
    /// folder first (`packRepository.reload()`), like the commands do.
    pub fn list_packs(&self) -> PackListing {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.repository.reload();
        let enabled_features = state.data_config.enabled_features;
        let packs = state.repository.available_packs();
        PackListing {
            available: state.repository.available_ids(),
            selected: state.repository.selected_ids(),
            disabled: state.data_config.data_packs.disabled().to_vec(),
            feature_packs: packs
                .iter()
                .filter(|pack| is_feature_pack(pack))
                .map(|pack| pack.id.clone())
                .collect(),
            unavailable_feature_packs: packs
                .iter()
                .filter(|pack| !pack.requested_features.is_subset_of(enabled_features))
                .map(|pack| pack.id.clone())
                .collect(),
        }
    }

    /// The world `datapacks` folder (`LevelResource.DATAPACK_DIR`).
    pub fn datapack_directory(&self) -> PathBuf {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.world_root.join("datapacks")
    }

    /// `MinecraftServer.reloadResources(packsToEnable)`. Returns the new resources
    /// once the selection, configuration and tags are replaced; on error the previous
    /// state is untouched.
    pub fn reload(&self, packs_to_enable: &[String]) -> Result<Arc<LoadedResources>, String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let manager = ResourceManager::new(open_packs(&state.repository, packs_to_enable)?);
        let mut registries = self.current().registries.clone();
        let mut logged = Vec::new();
        loader::reload_tags(&mut registries, &manager, &mut logged);
        for line in logged {
            crate::log::log_error(&line);
        }

        // Everything that can fail is built before any server state changes: Java keeps
        // the old resources when the reload future fails.
        let content = DatapackContent::load(&manager, &registries)?;
        state
            .repository
            .set_selected(packs_to_enable.iter().map(String::as_str));
        let new_config = WorldDataConfiguration {
            data_packs: selected_pack_config(&state.repository),
            enabled_features: state.data_config.enabled_features,
        };
        state.data_config = new_config;
        if let Err(error) = persist_data_configuration(&state.world_root, &state.data_config) {
            log_warn(&format!(
                "Failed to save the data pack configuration: {error}"
            ));
        }
        let loaded = Arc::new(LoadedResources {
            registries,
            content,
            manager,
        });
        *self.current.write().unwrap_or_else(|e| e.into_inner()) = Arc::clone(&loaded);
        Ok(loaded)
    }
}

/// A feature pack that requests features (`!getRequestedFeatures().isEmpty() &&
/// getPackSource() == FEATURE`), which `/datapack disable` refuses to touch.
fn is_feature_pack(pack: &DataPack) -> bool {
    pack.source == crate::resources::PackSource::Feature
        && pack.requested_features != FeatureFlagSet::empty()
}

/// Stores `config` into `level.dat` (`worldData.setDataConfiguration`, which Java
/// writes with the next level save): replaces the `DataPacks` and `enabled_features`
/// fields of the `Data` compound and leaves everything else untouched. A world
/// without a `level.dat` yet is left for the level-data writer.
pub fn persist_data_configuration(
    world_root: &Path,
    config: &WorldDataConfiguration,
) -> Result<(), String> {
    let layout = WorldLayout::new(world_root);
    if !layout.level_dat().is_file() && !layout.level_dat_old().is_file() {
        return Ok(());
    }
    let mut root = layout
        .load_level_dat_with_backup()
        .map_err(|error| error.to_string())?;
    let registry = FeatureFlagRegistry::main_26_1_2()?;
    let Tag::Compound(configuration) = config.to_nbt(&registry) else {
        return Err("world data configuration is not a compound".to_string());
    };
    let Tag::Compound(root_fields) = &mut root else {
        return Err("level.dat root is not a compound".to_string());
    };
    let Some((_, Tag::Compound(data))) = root_fields.iter_mut().find(|(key, _)| key == "Data")
    else {
        return Err("level.dat has no Data compound".to_string());
    };
    data.retain(|(key, _)| {
        key != "DataPacks" && key != WorldDataConfiguration::ENABLED_FEATURES_ID
    });
    data.extend(configuration);
    layout
        .save_level_dat(&root)
        .map_err(|error| error.to_string())
}

/// The resources in force: the installed server state, or the vanilla-only
/// resources when running without one (unit tests, report generation).
pub fn active_resources() -> Result<Arc<LoadedResources>, String> {
    if let Some(resources) = ServerResources::installed() {
        return Ok(resources.current());
    }
    static VANILLA: OnceLock<Result<Arc<LoadedResources>, String>> = OnceLock::new();
    VANILLA
        .get_or_init(|| {
            let mut repository = DataPackRepository::new(crate::resources::builtin_packs());
            repository.set_selected([VANILLA_PACK_ID]);
            ServerResources::load(&repository).map(Arc::new)
        })
        .clone()
}

/// The recipe manager in force (`server.getRecipeManager()`).
pub fn active_recipe_manager() -> Result<Arc<RecipeManagerModel>, String> {
    Ok(Arc::clone(&active_resources()?.content.recipes))
}

/// [`active_recipe_manager`] for long-lived holders that may run without a server
/// (unit tests): `None` until the server state is installed.
pub fn installed_recipe_manager() -> Option<Arc<RecipeManagerModel>> {
    ServerResources::installed().map(|resources| Arc::clone(&resources.current().content.recipes))
}

/// `FeatureFlags.REGISTRY.toNames(worldData.enabledFeatures())`, the payload of the
/// configuration-phase `ClientboundUpdateEnabledFeaturesPacket`. Falls back to the
/// default (vanilla) flags when no server state is installed.
pub fn enabled_feature_names() -> Result<Vec<crate::registry::Identifier>, String> {
    let features = match ServerResources::installed() {
        Some(resources) => {
            let state = resources.state.lock().unwrap_or_else(|e| e.into_inner());
            state.data_config.enabled_features
        }
        None => crate::registry::feature_flags::default_flags_26_1_2(),
    };
    Ok(FeatureFlagRegistry::main_26_1_2()?.to_names(features))
}
