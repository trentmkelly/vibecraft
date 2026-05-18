#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::registry::{feature_flags, FeatureFlagRegistry, FeatureFlagSet, Identifier};

pub const VANILLA_PACK_ID: &str = "vanilla";
pub const SERVER_DATA_PACK_FORMAT_MAJOR: u32 = 101;
pub const SERVER_DATA_PACK_FORMAT_MINOR: u32 = 1;
pub const LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT: u32 = 81;
pub const VANILLA_PACK_MCMETA: &str = r#"{
  "pack": {
    "description": "dataPack.vanilla.description",
    "min_format": [101, 0],
    "max_format": [101, 1]
  },
  "features": {
    "enabled": ["minecraft:vanilla"]
  }
}"#;

const VANILLA_BUILTIN_RESOURCES: &[(&str, &str)] = &[
    ("pack.mcmeta", VANILLA_PACK_MCMETA),
    (
        "data/minecraft/tags/block/replaceable.json",
        r#"{"replace":false,"values":["minecraft:air"]}"#,
    ),
    (
        "data/minecraft/tags/item/logs.json",
        r#"{"replace":false,"values":["minecraft:oak_log"]}"#,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSource {
    Default,
    BuiltIn,
    Feature,
    World,
    Server,
}

impl PackSource {
    pub fn should_add_automatically(self) -> bool {
        !matches!(self, Self::Feature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPack {
    pub id: String,
    pub source: PackSource,
    pub requested_features: FeatureFlagSet,
    pub metadata: DataPackMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltInDataPack {
    id: String,
    resources: BTreeMap<String, String>,
    metadata: DataPackMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataPack {
    pub pack: DataPack,
    resources: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataResourceKind {
    Advancement,
    BannerPattern,
    ChatType,
    CatSoundVariant,
    CatVariant,
    DamageType,
    Dialog,
    DimensionType,
    Enchantment,
    EnchantmentProvider,
    FrogVariant,
    ChickenSoundVariant,
    ChickenVariant,
    CowSoundVariant,
    CowVariant,
    Instrument,
    JukeboxSong,
    LootTable,
    PaintingVariant,
    PigSoundVariant,
    PigVariant,
    Recipe,
    Structure,
    Tags,
    TestEnvironment,
    TestInstance,
    WolfSoundVariant,
    WolfVariant,
    Timeline,
    TradeSet,
    TrialSpawner,
    TrimMaterial,
    TrimPattern,
    VillagerTrade,
    WorldClock,
    Worldgen,
    ZombieNautilusVariant,
}

impl DataResourceKind {
    pub const ALL: &'static [Self] = &[
        Self::Advancement,
        Self::BannerPattern,
        Self::ChatType,
        Self::CatSoundVariant,
        Self::CatVariant,
        Self::DamageType,
        Self::Dialog,
        Self::DimensionType,
        Self::Enchantment,
        Self::EnchantmentProvider,
        Self::FrogVariant,
        Self::ChickenSoundVariant,
        Self::ChickenVariant,
        Self::CowSoundVariant,
        Self::CowVariant,
        Self::Instrument,
        Self::JukeboxSong,
        Self::LootTable,
        Self::PaintingVariant,
        Self::PigSoundVariant,
        Self::PigVariant,
        Self::Recipe,
        Self::Structure,
        Self::Tags,
        Self::TestEnvironment,
        Self::TestInstance,
        Self::WolfSoundVariant,
        Self::WolfVariant,
        Self::Timeline,
        Self::TradeSet,
        Self::TrialSpawner,
        Self::TrimMaterial,
        Self::TrimPattern,
        Self::VillagerTrade,
        Self::WorldClock,
        Self::Worldgen,
        Self::ZombieNautilusVariant,
    ];

    pub fn path_component(self) -> &'static str {
        match self {
            Self::Advancement => "advancement",
            Self::BannerPattern => "banner_pattern",
            Self::ChatType => "chat_type",
            Self::CatSoundVariant => "cat_sound_variant",
            Self::CatVariant => "cat_variant",
            Self::DamageType => "damage_type",
            Self::Dialog => "dialog",
            Self::DimensionType => "dimension_type",
            Self::Enchantment => "enchantment",
            Self::EnchantmentProvider => "enchantment_provider",
            Self::FrogVariant => "frog_variant",
            Self::ChickenSoundVariant => "chicken_sound_variant",
            Self::ChickenVariant => "chicken_variant",
            Self::CowSoundVariant => "cow_sound_variant",
            Self::CowVariant => "cow_variant",
            Self::Instrument => "instrument",
            Self::JukeboxSong => "jukebox_song",
            Self::LootTable => "loot_table",
            Self::PaintingVariant => "painting_variant",
            Self::PigSoundVariant => "pig_sound_variant",
            Self::PigVariant => "pig_variant",
            Self::Recipe => "recipe",
            Self::Structure => "structure",
            Self::Tags => "tags",
            Self::TestEnvironment => "test_environment",
            Self::TestInstance => "test_instance",
            Self::WolfSoundVariant => "wolf_sound_variant",
            Self::WolfVariant => "wolf_variant",
            Self::Timeline => "timeline",
            Self::TradeSet => "trade_set",
            Self::TrialSpawner => "trial_spawner",
            Self::TrimMaterial => "trim_material",
            Self::TrimPattern => "trim_pattern",
            Self::VillagerTrade => "villager_trade",
            Self::WorldClock => "world_clock",
            Self::Worldgen => "worldgen",
            Self::ZombieNautilusVariant => "zombie_nautilus_variant",
        }
    }

    pub fn from_path_component(component: &str) -> Option<Self> {
        Some(match component {
            "advancement" => Self::Advancement,
            "banner_pattern" => Self::BannerPattern,
            "chat_type" => Self::ChatType,
            "damage_type" => Self::DamageType,
            "dialog" => Self::Dialog,
            "dimension_type" => Self::DimensionType,
            "enchantment" => Self::Enchantment,
            "enchantment_provider" => Self::EnchantmentProvider,
            "cat_sound_variant" => Self::CatSoundVariant,
            "cat_variant" => Self::CatVariant,
            "frog_variant" => Self::FrogVariant,
            "chicken_sound_variant" => Self::ChickenSoundVariant,
            "chicken_variant" => Self::ChickenVariant,
            "cow_sound_variant" => Self::CowSoundVariant,
            "cow_variant" => Self::CowVariant,
            "instrument" => Self::Instrument,
            "jukebox_song" => Self::JukeboxSong,
            "loot_table" => Self::LootTable,
            "painting_variant" => Self::PaintingVariant,
            "pig_sound_variant" => Self::PigSoundVariant,
            "pig_variant" => Self::PigVariant,
            "recipe" => Self::Recipe,
            "structure" => Self::Structure,
            "tags" => Self::Tags,
            "test_environment" => Self::TestEnvironment,
            "test_instance" => Self::TestInstance,
            "wolf_sound_variant" => Self::WolfSoundVariant,
            "wolf_variant" => Self::WolfVariant,
            "timeline" => Self::Timeline,
            "trade_set" => Self::TradeSet,
            "trial_spawner" => Self::TrialSpawner,
            "trim_material" => Self::TrimMaterial,
            "trim_pattern" => Self::TrimPattern,
            "villager_trade" => Self::VillagerTrade,
            "world_clock" => Self::WorldClock,
            "worldgen" => Self::Worldgen,
            "zombie_nautilus_variant" => Self::ZombieNautilusVariant,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataResource {
    pub namespace: String,
    pub kind: DataResourceKind,
    pub id: String,
    pub contents: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataResourceIndex {
    resources: BTreeMap<(String, DataResourceKind, String), String>,
}

impl DataResourceIndex {
    pub fn from_resources<'a>(
        resources: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, String> {
        let mut index = Self::default();
        for (path, contents) in resources {
            if let Some(resource) = parse_data_resource_path(path, contents)? {
                index.resources.insert(
                    (resource.namespace, resource.kind, resource.id),
                    resource.contents,
                );
            }
        }
        Ok(index)
    }

    pub fn get(&self, namespace: &str, kind: DataResourceKind, id: &str) -> Option<&str> {
        self.resources
            .get(&(namespace.to_string(), kind, id.to_string()))
            .map(String::as_str)
    }

    pub fn list(&self, namespace: &str, kind: DataResourceKind) -> Vec<DataResource> {
        self.resources
            .iter()
            .filter_map(|((entry_namespace, entry_kind, id), contents)| {
                (entry_namespace == namespace && *entry_kind == kind).then(|| DataResource {
                    namespace: entry_namespace.clone(),
                    kind: *entry_kind,
                    id: id.clone(),
                    contents: contents.clone(),
                })
            })
            .collect()
    }
}

impl WorldDataPack {
    pub fn get(&self, path: &str) -> Option<&str> {
        self.resources.get(path).map(String::as_str)
    }

    pub fn list_prefix(&self, prefix: &str) -> Vec<&str> {
        self.resources
            .keys()
            .filter(|path| path.starts_with(prefix))
            .map(String::as_str)
            .collect()
    }

    pub fn data_resources(&self) -> Result<DataResourceIndex, String> {
        DataResourceIndex::from_resources(
            self.resources
                .iter()
                .map(|(path, contents)| (path.as_str(), contents.as_str())),
        )
    }
}

impl BuiltInDataPack {
    pub fn vanilla_26_1_2() -> Self {
        let resources = VANILLA_BUILTIN_RESOURCES
            .iter()
            .map(|(path, contents)| ((*path).to_string(), (*contents).to_string()))
            .collect::<BTreeMap<_, _>>();
        let metadata = parse_pack_metadata(VANILLA_PACK_MCMETA)
            .unwrap_or_else(|err| panic!("invalid bundled vanilla pack metadata: {err}"));
        Self {
            id: VANILLA_PACK_ID.to_string(),
            resources,
            metadata,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn metadata(&self) -> &DataPackMetadata {
        &self.metadata
    }

    pub fn as_data_pack(&self) -> DataPack {
        DataPack::new(self.id.clone(), PackSource::BuiltIn).with_metadata(self.metadata.clone())
    }

    pub fn get(&self, path: &str) -> Option<&str> {
        self.resources.get(path).map(String::as_str)
    }

    pub fn contains(&self, path: &str) -> bool {
        self.resources.contains_key(path)
    }

    pub fn list_prefix(&self, prefix: &str) -> Vec<&str> {
        self.resources
            .keys()
            .filter(|path| path.starts_with(prefix))
            .map(String::as_str)
            .collect()
    }

    pub fn namespaces(&self) -> Vec<&str> {
        let mut namespaces = BTreeSet::new();
        for path in self.resources.keys() {
            if let Some(rest) = path.strip_prefix("data/") {
                if let Some((namespace, _tail)) = rest.split_once('/') {
                    namespaces.insert(namespace);
                }
            }
        }
        namespaces.into_iter().collect()
    }

    pub fn data_resources(&self) -> Result<DataResourceIndex, String> {
        DataResourceIndex::from_resources(
            self.resources
                .iter()
                .map(|(path, contents)| (path.as_str(), contents.as_str())),
        )
    }
}

impl DataPack {
    pub fn new(id: impl Into<String>, source: PackSource) -> Self {
        Self {
            id: id.into(),
            source,
            requested_features: FeatureFlagSet::empty(),
            metadata: DataPackMetadata::default_26_1_2(),
        }
    }

    pub fn with_features(mut self, requested_features: FeatureFlagSet) -> Self {
        self.requested_features = requested_features;
        self
    }

    pub fn with_metadata(mut self, metadata: DataPackMetadata) -> Self {
        self.requested_features = metadata.requested_features;
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackMetadata {
    pub description: String,
    pub supported_formats: PackFormatRange,
    pub compatibility: PackCompatibility,
    pub requested_features: FeatureFlagSet,
}

impl DataPackMetadata {
    pub fn default_26_1_2() -> Self {
        Self {
            description: String::new(),
            supported_formats: PackFormatRange {
                min: PackFormat::current_server_data(),
                max: PackFormat {
                    major: SERVER_DATA_PACK_FORMAT_MAJOR,
                    minor: u32::MAX,
                },
            },
            compatibility: PackCompatibility::Compatible,
            requested_features: FeatureFlagSet::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackFormat {
    pub major: u32,
    pub minor: u32,
}

impl PackFormat {
    pub fn current_server_data() -> Self {
        Self {
            major: SERVER_DATA_PACK_FORMAT_MAJOR,
            minor: SERVER_DATA_PACK_FORMAT_MINOR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackFormatRange {
    pub min: PackFormat,
    pub max: PackFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackCompatibility {
    TooOld,
    TooNew,
    Unknown,
    Compatible,
}

impl PackCompatibility {
    pub fn is_compatible(self) -> bool {
        self == Self::Compatible
    }

    pub fn for_version(declared: PackFormatRange, current: PackFormat) -> Self {
        if declared.min.major == u32::MAX {
            Self::Unknown
        } else if declared.max < current {
            Self::TooOld
        } else if current < declared.min {
            Self::TooNew
        } else {
            Self::Compatible
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

impl DataPackConfig {
    pub fn default_26_1_2() -> Self {
        Self {
            enabled: vec![VANILLA_PACK_ID.to_string()],
            disabled: Vec::new(),
        }
    }

    pub fn from_properties(enabled: &str, disabled: &str) -> Self {
        Self {
            enabled: split_pack_list(enabled),
            disabled: split_pack_list(disabled),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataConfiguration {
    pub data_packs: DataPackConfig,
    pub enabled_features: FeatureFlagSet,
}

impl WorldDataConfiguration {
    pub fn default_26_1_2() -> Self {
        Self {
            data_packs: DataPackConfig::default_26_1_2(),
            enabled_features: feature_flags::default_flags_26_1_2(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataPackRepository {
    available: BTreeMap<String, DataPack>,
    selected: Vec<String>,
}

impl DataPackRepository {
    pub fn new(packs: impl IntoIterator<Item = DataPack>) -> Self {
        Self {
            available: packs
                .into_iter()
                .map(|pack| (pack.id.clone(), pack))
                .collect(),
            selected: Vec::new(),
        }
    }

    pub fn server_repository(datapack_dir: &Path) -> Result<Self, String> {
        let mut packs = vec![BuiltInDataPack::vanilla_26_1_2().as_data_pack()];

        packs.extend(
            load_world_data_packs(datapack_dir)?
                .into_iter()
                .map(|loaded| loaded.pack),
        );
        Ok(Self::new(packs))
    }

    pub fn is_available(&self, id: &str) -> bool {
        self.available.contains_key(id)
    }

    pub fn selected_ids(&self) -> Vec<String> {
        self.selected.clone()
    }

    pub fn available_ids(&self) -> Vec<String> {
        self.available.keys().cloned().collect()
    }

    pub fn selected_packs(&self) -> Vec<&DataPack> {
        self.selected
            .iter()
            .filter_map(|id| self.available.get(id))
            .collect()
    }

    pub fn priority_stack(&self) -> Vec<String> {
        self.selected.clone()
    }

    pub fn available_packs(&self) -> Vec<&DataPack> {
        self.available.values().collect()
    }

    pub fn set_selected<'a>(&mut self, selected: impl IntoIterator<Item = &'a str>) {
        let mut seen = BTreeSet::new();
        self.selected.clear();
        for id in selected {
            if self.is_available(id) && seen.insert(id.to_string()) {
                self.selected.push(id.to_string());
            }
        }
    }

    pub fn enable_pack_highest_priority(&mut self, id: &str) -> bool {
        if !self.is_available(id) {
            return false;
        }
        self.selected.retain(|selected| selected != id);
        self.selected.push(id.to_string());
        true
    }

    pub fn disable_pack(&mut self, id: &str) -> bool {
        let before = self.selected.len();
        self.selected.retain(|selected| selected != id);
        before != self.selected.len()
    }

    pub fn requested_feature_flags(&self) -> FeatureFlagSet {
        self.selected_packs()
            .into_iter()
            .fold(FeatureFlagSet::empty(), |features, pack| {
                features.join(pack.requested_features)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackConfigureOptions {
    pub init_mode: bool,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackReloadFailure {
    pub error: String,
    pub restored_enabled: Vec<String>,
}

impl PackReloadFailure {
    pub fn user_message(&self) -> String {
        format!(
            "Failed to reload data packs; keeping previous selection [{}]: {}",
            self.restored_enabled.join(","),
            self.error
        )
    }
}

pub fn configure_pack_repository(
    repository: &mut DataPackRepository,
    initial_data_config: &WorldDataConfiguration,
    options: PackConfigureOptions,
) -> WorldDataConfiguration {
    let forced_features = if options.init_mode {
        FeatureFlagSet::empty()
    } else {
        initial_data_config.enabled_features
    };
    let allowed_features = if options.init_mode {
        all_known_features_26_1_2()
    } else {
        initial_data_config.enabled_features
    };

    if options.safe_mode {
        return configure_repository_with_selection(
            repository,
            &[VANILLA_PACK_ID.to_string()],
            forced_features,
            false,
        );
    }

    let disabled: BTreeSet<&str> = initial_data_config
        .data_packs
        .disabled
        .iter()
        .map(String::as_str)
        .collect();
    let mut selected = Vec::new();
    let mut selected_lookup = BTreeSet::new();

    for id in &initial_data_config.data_packs.enabled {
        if repository.is_available(id) && selected_lookup.insert(id.clone()) {
            selected.push(id.clone());
        }
    }

    for pack in repository.available_packs() {
        if disabled.contains(pack.id.as_str()) {
            continue;
        }

        let is_selected = selected_lookup.contains(&pack.id);
        if !is_selected
            && pack.source.should_add_automatically()
            && pack.requested_features.is_subset_of(allowed_features)
        {
            selected.push(pack.id.clone());
            selected_lookup.insert(pack.id.clone());
        }

        if is_selected && !pack.requested_features.is_subset_of(allowed_features) {
            selected.retain(|id| id != &pack.id);
            selected_lookup.remove(&pack.id);
        }
    }

    if selected.is_empty() {
        selected.push(VANILLA_PACK_ID.to_string());
    }

    configure_repository_with_selection(repository, &selected, forced_features, true)
}

pub fn reload_pack_repository<F>(
    repository: &mut DataPackRepository,
    initial_data_config: &WorldDataConfiguration,
    options: PackConfigureOptions,
    validate_reload: F,
) -> Result<WorldDataConfiguration, String>
where
    F: FnOnce(&[&DataPack]) -> Result<(), String>,
{
    reload_pack_repository_with_report(repository, initial_data_config, options, validate_reload)
        .map_err(|failure| failure.error)
}

pub fn reload_pack_repository_with_report<F>(
    repository: &mut DataPackRepository,
    initial_data_config: &WorldDataConfiguration,
    options: PackConfigureOptions,
    validate_reload: F,
) -> Result<WorldDataConfiguration, PackReloadFailure>
where
    F: FnOnce(&[&DataPack]) -> Result<(), String>,
{
    let previous = repository.selected.clone();
    let configured = configure_pack_repository(repository, initial_data_config, options);
    let selected = repository.selected_packs();
    if let Err(err) = validate_reload(&selected) {
        repository.selected = previous;
        return Err(PackReloadFailure {
            error: err,
            restored_enabled: repository.selected.clone(),
        });
    }
    Ok(configured)
}

fn configure_repository_with_selection(
    repository: &mut DataPackRepository,
    selected: &[String],
    forced_features: FeatureFlagSet,
    disable_inactive: bool,
) -> WorldDataConfiguration {
    repository.set_selected(selected.iter().map(String::as_str));
    enable_forced_feature_packs(repository, forced_features);

    let enabled = repository.selected_ids();
    let enabled_lookup = enabled.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let disabled = if disable_inactive {
        repository
            .available_ids()
            .into_iter()
            .filter(|id| !enabled_lookup.contains(id.as_str()))
            .collect()
    } else {
        Vec::new()
    };

    WorldDataConfiguration {
        data_packs: DataPackConfig { enabled, disabled },
        enabled_features: repository.requested_feature_flags().join(forced_features),
    }
}

fn enable_forced_feature_packs(
    repository: &mut DataPackRepository,
    forced_features: FeatureFlagSet,
) {
    let mut missing_features = forced_features.subtract(repository.requested_feature_flags());
    if missing_features == FeatureFlagSet::empty() {
        return;
    }

    let mut selected = repository.selected_ids();
    let mut selected_lookup = selected.iter().cloned().collect::<BTreeSet<_>>();
    for pack in repository.available_packs() {
        if missing_features == FeatureFlagSet::empty() {
            break;
        }
        if pack.source == PackSource::Feature
            && pack.requested_features != FeatureFlagSet::empty()
            && pack.requested_features.intersects(missing_features)
            && pack.requested_features.is_subset_of(forced_features)
        {
            if selected_lookup.insert(pack.id.clone()) {
                selected.push(pack.id.clone());
            }
            missing_features = missing_features.subtract(pack.requested_features);
        }
    }

    repository.set_selected(selected.iter().map(String::as_str));
}

fn all_known_features_26_1_2() -> FeatureFlagSet {
    FeatureFlagSet::of(&[
        feature_flags::VANILLA,
        feature_flags::TRADE_REBALANCE,
        feature_flags::REDSTONE_EXPERIMENTS,
        feature_flags::MINECART_IMPROVEMENTS,
    ])
}

fn split_pack_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn load_world_data_packs(datapack_dir: &Path) -> Result<Vec<WorldDataPack>, String> {
    if !datapack_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    let entries = fs::read_dir(datapack_dir).map_err(|err| {
        format!(
            "Failed to read datapack directory '{}': {err}",
            datapack_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "Failed to read datapack entry in '{}': {err}",
                datapack_dir.display()
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "Failed to inspect datapack entry '{}': {err}",
                path.display()
            )
        })?;

        let id = if file_type.is_dir() {
            entry.file_name().to_string_lossy().into_owned()
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("zip")
        {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string()
        } else {
            continue;
        };

        if !is_valid_pack_id(&id) {
            continue;
        }

        let (metadata, resources) = if file_type.is_dir() {
            let metadata_path = path.join("pack.mcmeta");
            let contents = match fs::read_to_string(&metadata_path) {
                Ok(contents) => contents,
                Err(_) => continue,
            };
            let metadata = match parse_pack_metadata(&contents) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            (metadata, load_directory_pack_resources(&path)?)
        } else {
            continue;
        };

        if metadata.compatibility.is_compatible() {
            packs.push(WorldDataPack {
                pack: DataPack::new(format!("file/{id}"), PackSource::World)
                    .with_metadata(metadata),
                resources,
            });
        }
    }

    Ok(packs)
}

fn load_directory_pack_resources(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut resources = BTreeMap::new();
    load_directory_pack_resources_inner(root, root, &mut resources)?;
    Ok(resources)
}

fn load_directory_pack_resources_inner(
    root: &Path,
    current: &Path,
    resources: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|err| {
        format!(
            "Failed to read datapack directory '{}': {err}",
            current.display()
        )
    })? {
        let entry = entry.map_err(|err| {
            format!(
                "Failed to read datapack entry in '{}': {err}",
                current.display()
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "Failed to inspect datapack entry '{}': {err}",
                path.display()
            )
        })?;
        if file_type.is_dir() {
            load_directory_pack_resources_inner(root, &path, resources)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|err| {
                format!(
                    "Failed to relativize datapack path '{}' against '{}': {err}",
                    path.display(),
                    root.display()
                )
            })?;
            let resource_path = relative.to_string_lossy().replace('\\', "/");
            let contents = fs::read_to_string(&path).map_err(|err| {
                format!(
                    "Failed to read datapack resource '{}': {err}",
                    path.display()
                )
            })?;
            resources.insert(resource_path, contents);
        }
    }
    Ok(())
}

fn parse_data_resource_path(path: &str, contents: &str) -> Result<Option<DataResource>, String> {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 4 || parts[0] != "data" {
        return Ok(None);
    }
    let namespace = parts[1];
    Identifier::parse(namespace)?;
    let Some(kind) = DataResourceKind::from_path_component(parts[2]) else {
        return Ok(None);
    };
    let tail = parts[3..].join("/");
    let Some(id) = tail.strip_suffix(".json") else {
        return Ok(None);
    };
    if id.is_empty() {
        return Err(format!("empty data resource id in path {path}"));
    }
    Identifier::parse(&format!("{namespace}:{id}"))?;
    Ok(Some(DataResource {
        namespace: namespace.to_string(),
        kind,
        id: id.to_string(),
        contents: contents.to_string(),
    }))
}

fn is_valid_pack_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/'))
}

pub fn parse_pack_metadata(contents: &str) -> Result<DataPackMetadata, String> {
    let pack_object = object_slice(contents, "pack").ok_or("missing pack metadata")?;
    let description = string_field(pack_object, "description").unwrap_or_default();
    let supported_formats = parse_supported_formats(pack_object)?;
    let requested_features = if let Some(features_object) = object_slice(contents, "features") {
        parse_feature_flags(features_object)?
    } else {
        FeatureFlagSet::empty()
    };

    Ok(DataPackMetadata {
        description,
        supported_formats,
        compatibility: PackCompatibility::for_version(
            supported_formats,
            PackFormat::current_server_data(),
        ),
        requested_features,
    })
}

fn parse_supported_formats(pack_object: &str) -> Result<PackFormatRange, String> {
    if let (Some(min), Some(max)) = (
        pack_format_field(pack_object, "min_format"),
        pack_format_field(pack_object, "max_format"),
    ) {
        if min > max {
            return Err("min_format is greater than max_format".to_string());
        }
        if min.major <= LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT
            && !has_field(pack_object, "supported_formats")
        {
            return Err("supported_formats required for pre-minor pack formats".to_string());
        }
        return Ok(PackFormatRange { min, max });
    }

    if let Some(range) = int_range_field(pack_object, "supported_formats") {
        if range.max.major > LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT {
            return Err("old supported_formats cannot exceed last pre-minor format".to_string());
        }
        return Ok(range);
    }

    if let Some(pack_format) = int_field(pack_object, "pack_format") {
        if pack_format > LAST_PRE_MINOR_SERVER_DATA_PACK_FORMAT {
            return Err("new pack formats require min_format and max_format".to_string());
        }
        return Ok(PackFormatRange {
            min: PackFormat {
                major: pack_format,
                minor: 0,
            },
            max: PackFormat {
                major: pack_format,
                minor: 0,
            },
        });
    }

    Err("missing format version information".to_string())
}

fn parse_feature_flags(features_object: &str) -> Result<FeatureFlagSet, String> {
    let names = string_array_field(features_object, "enabled")
        .unwrap_or_default()
        .into_iter()
        .map(|name| Identifier::parse(&name))
        .collect::<Result<Vec<_>, _>>()?;
    FeatureFlagRegistry::main_26_1_2()
        .from_names(&names)
        .map_err(|unknown| format!("unknown feature flags: {unknown:?}"))
}

fn object_slice<'a>(contents: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = contents.find(&key)?;
    let start = contents[key_index + key.len()..].find('{')? + key_index + key.len();
    let end = matching_delimiter(contents, start, '{', '}')?;
    Some(&contents[start + 1..end])
}

fn has_field(contents: &str, field: &str) -> bool {
    contents.contains(&format!("\"{field}\""))
}

fn string_field(contents: &str, field: &str) -> Option<String> {
    let raw = field_value(contents, field)?;
    if raw.trim_start().starts_with('"') {
        parse_json_string(raw.trim_start()).map(|(value, _)| value)
    } else {
        Some(raw.trim().to_string())
    }
}

fn int_field(contents: &str, field: &str) -> Option<u32> {
    let raw = field_value(contents, field)?;
    parse_u32_prefix(raw.trim_start())
}

fn int_range_field(contents: &str, field: &str) -> Option<PackFormatRange> {
    let raw = field_value(contents, field)?.trim_start();
    if raw.starts_with('[') {
        let end = matching_delimiter(raw, 0, '[', ']')?;
        let values = raw[1..end]
            .split(',')
            .filter_map(|part| parse_u32_prefix(part.trim()))
            .collect::<Vec<_>>();
        match values.as_slice() {
            [one] => Some(PackFormatRange {
                min: PackFormat {
                    major: *one,
                    minor: 0,
                },
                max: PackFormat {
                    major: *one,
                    minor: 0,
                },
            }),
            [min, max] => Some(PackFormatRange {
                min: PackFormat {
                    major: *min,
                    minor: 0,
                },
                max: PackFormat {
                    major: *max,
                    minor: 0,
                },
            }),
            _ => None,
        }
    } else {
        int_field(contents, field).map(|value| PackFormatRange {
            min: PackFormat {
                major: value,
                minor: 0,
            },
            max: PackFormat {
                major: value,
                minor: 0,
            },
        })
    }
}

fn pack_format_field(contents: &str, field: &str) -> Option<PackFormat> {
    let raw = field_value(contents, field)?.trim_start();
    if raw.starts_with('[') {
        let end = matching_delimiter(raw, 0, '[', ']')?;
        let values = raw[1..end]
            .split(',')
            .filter_map(|part| parse_u32_prefix(part.trim()))
            .collect::<Vec<_>>();
        Some(PackFormat {
            major: *values.first()?,
            minor: *values.get(1).unwrap_or(&0),
        })
    } else {
        int_field(contents, field).map(|major| PackFormat { major, minor: 0 })
    }
}

fn string_array_field(contents: &str, field: &str) -> Option<Vec<String>> {
    let raw = field_value(contents, field)?.trim_start();
    if !raw.starts_with('[') {
        return None;
    }
    let end = matching_delimiter(raw, 0, '[', ']')?;
    let mut values = Vec::new();
    let mut rest = raw[1..end].trim_start();
    while !rest.is_empty() {
        if let Some((value, remaining)) = parse_json_string(rest) {
            values.push(value);
            rest = remaining.trim_start();
            if rest.starts_with(',') {
                rest = rest[1..].trim_start();
            } else {
                break;
            }
        } else {
            return None;
        }
    }
    Some(values)
}

fn field_value<'a>(contents: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = contents.find(&key)?;
    let after_key = &contents[key_index + key.len()..];
    let colon = after_key.find(':')?;
    Some(&after_key[colon + 1..])
}

fn matching_delimiter(contents: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in contents[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

fn parse_json_string(contents: &str) -> Option<(String, &str)> {
    let mut chars = contents.char_indices();
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    for (index, ch) in chars {
        if escaped {
            value.push(match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some((value, &contents[index + 1..]));
        } else {
            value.push(ch);
        }
    }
    None
}

fn parse_u32_prefix(contents: &str) -> Option<u32> {
    let digits = contents
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::feature_flags;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn decompiled_minecraft_data_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("decompiled-server-26.1.2")
            .join("data")
            .join("minecraft")
    }

    fn collect_decompiled_minecraft_json_paths() -> Vec<String> {
        let root = decompiled_minecraft_data_root();
        let mut paths = Vec::new();
        let mut dirs = vec![root.clone()];

        while let Some(current) = dirs.pop() {
            let read_dir = fs::read_dir(&current).expect("failed to read decompiled data dir");
            for entry in read_dir {
                let entry = entry.expect("failed to read decompiled data entry");
                let path = entry.path();

                if path.is_dir() {
                    if current == root && path.file_name().and_then(|name| name.to_str()) == Some("datapacks")
                    {
                        continue;
                    }
                    dirs.push(path);
                    continue;
                }

                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }

                let relative = path
                    .strip_prefix(&root)
                    .expect("decompiled data path should be under data/minecraft");
                let relative_path = relative
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("/");
                paths.push(format!("data/minecraft/{relative_path}"));
            }
        }

        paths
    }

    #[test]
    fn decompiled_minecraft_data_resource_kinds_match_data_folders() {
        let root = decompiled_minecraft_data_root();
        let mut observed_top_levels = root
            .read_dir()
            .expect("failed to read decompiled data root")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_str().map(ToString::to_string))
            .filter(|name| name != "datapacks")
            .collect::<Vec<_>>();
        observed_top_levels.sort();

        let known_top_levels = DataResourceKind::ALL
            .iter()
            .map(|kind| kind.path_component().to_string())
            .collect::<std::collections::BTreeSet<_>>();

        for observed in observed_top_levels {
            assert!(
                known_top_levels.contains(&observed),
                "unknown top-level minecraft data folder in decompiled tree: {observed}"
            );
        }
    }

    #[test]
    fn decompiled_minecraft_data_kind_counts_match_index() {
        let resource_paths = collect_decompiled_minecraft_json_paths();
        let mut expected_counts = std::collections::BTreeMap::<DataResourceKind, usize>::new();
        for kind in DataResourceKind::ALL {
            expected_counts.insert(*kind, 0);
        }

        for path in &resource_paths {
            let top_level = path
                .split('/')
                .nth(2)
                .expect("data path should include top-level resource folder");
            if let Some(kind) = DataResourceKind::from_path_component(top_level) {
                *expected_counts.entry(kind).or_insert(0) += 1;
            }
        }

        let resources = resource_paths
            .iter()
            .map(|path| (path.as_str(), "{}"))
            .collect::<Vec<_>>();
        let index = DataResourceIndex::from_resources(resources).unwrap();

        for kind in DataResourceKind::ALL {
            let expected = expected_counts.get(kind).copied().unwrap_or(0);
            let observed = index.list("minecraft", *kind).len();
            assert_eq!(
                expected,
                observed,
                "resource kind {} expected count mismatch",
                kind.path_component()
            );
        }
    }

    #[test]
    fn safe_mode_selects_only_vanilla_and_does_not_disable_world_packs() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("world_pack", PackSource::World),
            DataPack::new("server_pack", PackSource::Server),
        ]);
        let initial = WorldDataConfiguration {
            data_packs: DataPackConfig {
                enabled: vec![
                    VANILLA_PACK_ID.to_string(),
                    "world_pack".to_string(),
                    "server_pack".to_string(),
                ],
                disabled: Vec::new(),
            },
            enabled_features: feature_flags::default_flags_26_1_2(),
        };

        let configured = configure_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: true,
            },
        );

        assert_eq!(configured.data_packs.enabled, vec![VANILLA_PACK_ID]);
        assert!(configured.data_packs.disabled.is_empty());
        assert_eq!(repository.selected_ids(), vec![VANILLA_PACK_ID]);
    }

    #[test]
    fn loads_vanilla_builtin_datapack_from_bundled_resources() {
        let pack = BuiltInDataPack::vanilla_26_1_2();

        assert_eq!(pack.id(), VANILLA_PACK_ID);
        assert_eq!(pack.metadata().description, "dataPack.vanilla.description");
        assert_eq!(
            pack.metadata().requested_features,
            feature_flags::default_flags_26_1_2()
        );
        assert!(pack.contains("pack.mcmeta"));
        assert!(pack.contains("data/minecraft/tags/block/replaceable.json"));
        assert_eq!(pack.namespaces(), vec!["minecraft"]);
        assert_eq!(
            pack.list_prefix("data/minecraft/tags/")
                .into_iter()
                .collect::<Vec<_>>(),
            vec![
                "data/minecraft/tags/block/replaceable.json",
                "data/minecraft/tags/item/logs.json"
            ]
        );

        let repository = DataPackRepository::server_repository(Path::new("missing-datapacks"))
            .expect("builtin repository should not require a world datapack dir");
        let vanilla = repository
            .available_packs()
            .into_iter()
            .find(|candidate| candidate.id == VANILLA_PACK_ID)
            .expect("vanilla pack should be present");
        assert_eq!(vanilla.source, PackSource::BuiltIn);
        assert_eq!(
            vanilla.requested_features,
            feature_flags::default_flags_26_1_2()
        );
    }

    #[test]
    fn indexes_all_vanilla_data_resource_roots_by_kind() {
        let resources = DataResourceKind::ALL
            .iter()
            .map(|kind| {
                let id = if *kind == DataResourceKind::Tags {
                    "block/example"
                } else {
                    "example"
                };
                let path = format!("data/minecraft/{}/{id}.json", kind.path_component());
                (
                    path,
                    r#"{"loaded":true}"#.to_string(),
                    *kind,
                    id.to_string(),
                )
            })
            .collect::<Vec<_>>();
        let index = DataResourceIndex::from_resources(
            resources
                .iter()
                .map(|(path, contents, _kind, _id)| (path.as_str(), contents.as_str())),
        )
        .unwrap();

        for (_path, contents, kind, id) in resources {
            assert_eq!(
                index.get("minecraft", kind, &id),
                Some(contents.as_str()),
                "missing indexed resource kind {:?}",
                kind
            );
        }
    }

    #[test]
    fn normal_mode_keeps_enabled_packs_and_disables_inactive_available_packs() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("world_pack", PackSource::World),
            DataPack::new("disabled_pack", PackSource::World),
        ]);
        let initial = WorldDataConfiguration {
            data_packs: DataPackConfig {
                enabled: vec![VANILLA_PACK_ID.to_string(), "world_pack".to_string()],
                disabled: vec!["disabled_pack".to_string()],
            },
            enabled_features: feature_flags::default_flags_26_1_2(),
        };

        let configured = configure_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );

        assert_eq!(
            configured.data_packs.enabled,
            vec![VANILLA_PACK_ID, "world_pack"]
        );
        assert_eq!(configured.data_packs.disabled, vec!["disabled_pack"]);
    }

    #[test]
    fn normal_mode_auto_adds_new_world_packs() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("new_world_pack", PackSource::World),
        ]);
        let initial = WorldDataConfiguration::default_26_1_2();

        let configured = configure_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );

        assert_eq!(
            configured.data_packs.enabled,
            vec![VANILLA_PACK_ID, "new_world_pack"]
        );
        assert!(configured.data_packs.disabled.is_empty());
    }

    #[test]
    fn forced_features_enable_matching_feature_pack() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("feature/redstone", PackSource::Feature)
                .with_features(FeatureFlagSet::of(&[feature_flags::REDSTONE_EXPERIMENTS])),
        ]);
        let initial = WorldDataConfiguration {
            data_packs: DataPackConfig::default_26_1_2(),
            enabled_features: feature_flags::default_flags_26_1_2()
                .join(FeatureFlagSet::of(&[feature_flags::REDSTONE_EXPERIMENTS])),
        };

        let configured = configure_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );

        assert_eq!(
            configured.data_packs.enabled,
            vec![VANILLA_PACK_ID, "feature/redstone"]
        );
        assert!(configured
            .enabled_features
            .contains(feature_flags::REDSTONE_EXPERIMENTS));
    }

    #[test]
    fn pack_priority_enable_disable_and_reload_rollback_match_repository_rules() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("low", PackSource::World),
            DataPack::new("high", PackSource::World),
        ]);

        repository.set_selected([VANILLA_PACK_ID, "low"]);
        assert!(repository.enable_pack_highest_priority("high"));
        assert_eq!(
            repository.priority_stack(),
            vec![VANILLA_PACK_ID, "low", "high"]
        );
        assert!(repository.enable_pack_highest_priority("low"));
        assert_eq!(
            repository.priority_stack(),
            vec![VANILLA_PACK_ID, "high", "low"]
        );
        assert!(repository.disable_pack("high"));
        assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "low"]);
        assert!(!repository.enable_pack_highest_priority("missing"));

        let initial = WorldDataConfiguration {
            data_packs: DataPackConfig {
                enabled: vec![VANILLA_PACK_ID.to_string(), "high".to_string()],
                disabled: vec!["low".to_string()],
            },
            enabled_features: feature_flags::default_flags_26_1_2(),
        };
        let err = reload_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
            |packs| {
                assert_eq!(
                    packs
                        .iter()
                        .map(|pack| pack.id.as_str())
                        .collect::<Vec<_>>(),
                    vec![VANILLA_PACK_ID, "high"]
                );
                Err("reload failed".to_string())
            },
        )
        .unwrap_err();

        assert_eq!(err, "reload failed");
        assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "low"]);

        let configured = reload_pack_repository(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(configured.data_packs.enabled, vec![VANILLA_PACK_ID, "high"]);
        assert_eq!(configured.data_packs.disabled, vec!["low"]);
        assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "high"]);
    }

    #[test]
    fn reload_failure_report_is_user_facing_and_names_restored_selection() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
                .with_features(feature_flags::default_flags_26_1_2()),
            DataPack::new("kept", PackSource::World),
            DataPack::new("broken", PackSource::World),
        ]);
        repository.set_selected([VANILLA_PACK_ID, "kept"]);
        let initial = WorldDataConfiguration {
            data_packs: DataPackConfig {
                enabled: vec![VANILLA_PACK_ID.to_string(), "broken".to_string()],
                disabled: vec!["kept".to_string()],
            },
            enabled_features: feature_flags::default_flags_26_1_2(),
        };

        let failure = reload_pack_repository_with_report(
            &mut repository,
            &initial,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
            |_| Err("invalid tag entry in file/broken".to_string()),
        )
        .unwrap_err();

        assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "kept"]);
        assert_eq!(
            failure.restored_enabled,
            vec![VANILLA_PACK_ID.to_string(), "kept".to_string()]
        );
        assert_eq!(
            failure.user_message(),
            "Failed to reload data packs; keeping previous selection [vanilla,kept]: invalid tag entry in file/broken"
        );
    }

    #[test]
    fn parses_pack_metadata_and_detects_compatibility() {
        let metadata = parse_pack_metadata(
            r#"{
              "pack": {
                "description": "Test pack",
                "pack_format": 101,
                "min_format": [101, 0],
                "max_format": [101, 99]
              },
              "features": {
                "enabled": ["vanilla"]
              }
            }"#,
        )
        .unwrap();

        assert_eq!(metadata.description, "Test pack");
        assert_eq!(
            metadata.supported_formats.min,
            PackFormat {
                major: 101,
                minor: 0
            }
        );
        assert_eq!(metadata.compatibility, PackCompatibility::Compatible);
        assert!(metadata.requested_features.contains(feature_flags::VANILLA));

        let old = parse_pack_metadata(
            r#"{
              "pack": {
                "description": "Old pack",
                "pack_format": 81,
                "supported_formats": [80, 81]
              }
            }"#,
        )
        .unwrap();
        assert_eq!(old.compatibility, PackCompatibility::TooOld);

        assert!(
            parse_pack_metadata(r#"{"pack":{"description":"bad","pack_format":101}}"#).is_err()
        );
    }

    #[test]
    fn server_repository_discovers_compatible_directory_world_packs_with_metadata() {
        let temp_dir = std::env::temp_dir().join(format!(
            "rustcraft-packs-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let datapacks = temp_dir.join("datapacks");
        fs::create_dir_all(datapacks.join("dir_pack")).unwrap();
        fs::write(
            datapacks.join("dir_pack").join("pack.mcmeta"),
            r#"{
              "pack": {
                "description": "Directory pack",
                "pack_format": 101,
                "min_format": [101, 0],
                "max_format": [101, 99]
              }
            }"#,
        )
        .unwrap();
        fs::create_dir_all(datapacks.join("dir_pack").join("data/example/tags/item")).unwrap();
        fs::create_dir_all(
            datapacks
                .join("dir_pack")
                .join("data/minecraft/advancement"),
        )
        .unwrap();
        fs::write(
            datapacks
                .join("dir_pack")
                .join("data/example/tags/item/test_items.json"),
            r#"{"replace":false,"values":["minecraft:stick"]}"#,
        )
        .unwrap();
        fs::write(
            datapacks
                .join("dir_pack")
                .join("data/minecraft/advancement/root.json"),
            r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}}}"#,
        )
        .unwrap();
        fs::create_dir_all(datapacks.join("missing_meta")).unwrap();
        fs::create_dir_all(datapacks.join("old_pack")).unwrap();
        fs::write(
            datapacks.join("old_pack").join("pack.mcmeta"),
            r#"{"pack":{"description":"Old","pack_format":81,"supported_formats":[80,81]}}"#,
        )
        .unwrap();
        fs::write(datapacks.join("zip_pack.zip"), []).unwrap();
        fs::write(datapacks.join("notes.txt"), []).unwrap();

        let repository = DataPackRepository::server_repository(&datapacks).unwrap();
        let mut ids = repository.available_ids();
        ids.sort();

        assert_eq!(ids, vec!["file/dir_pack", VANILLA_PACK_ID]);

        let loaded = load_world_data_packs(&datapacks).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].pack.id, "file/dir_pack");
        assert_eq!(
            loaded[0].get("data/example/tags/item/test_items.json"),
            Some(r#"{"replace":false,"values":["minecraft:stick"]}"#)
        );
        assert_eq!(
            loaded[0].list_prefix("data/example/tags/"),
            vec!["data/example/tags/item/test_items.json"]
        );
        let index = loaded[0].data_resources().unwrap();
        assert_eq!(
            index.get("minecraft", DataResourceKind::Advancement, "root"),
            Some(r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}}}"#)
        );
        assert_eq!(
            index
                .list("minecraft", DataResourceKind::Advancement)
                .into_iter()
                .map(|resource| resource.id)
                .collect::<Vec<_>>(),
            vec!["root"]
        );

        fs::remove_dir_all(temp_dir).unwrap();
    }

}
