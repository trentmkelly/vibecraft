#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::chat_component::{Component, Style};
use crate::chat_formatting::ChatFormatting;
use crate::registry::{feature_flags, FeatureFlagRegistry, FeatureFlagSet, Identifier};

pub const VANILLA_PACK_ID: &str = "vanilla";
pub const CLIENT_RESOURCE_PACK_FORMAT_MAJOR: u32 = 84;
pub const CLIENT_RESOURCE_PACK_FORMAT_MINOR: u32 = 0;
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
            .filter(|((entry_namespace, entry_kind, _id), _contents)| {
                entry_namespace == namespace && *entry_kind == kind
            })
            .map(
                |((entry_namespace, entry_kind, id), contents)| DataResource {
                    namespace: entry_namespace.clone(),
                    kind: *entry_kind,
                    id: id.clone(),
                    contents: contents.clone(),
                },
            )
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
    pub const fn current_client_resources() -> Self {
        Self {
            major: CLIENT_RESOURCE_PACK_FORMAT_MAJOR,
            minor: CLIENT_RESOURCE_PACK_FORMAT_MINOR,
        }
    }

    pub const fn current_server_data() -> Self {
        Self {
            major: SERVER_DATA_PACK_FORMAT_MAJOR,
            minor: SERVER_DATA_PACK_FORMAT_MINOR,
        }
    }
}

impl std::fmt::Display for PackFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.minor == 0 {
            write!(formatter, "{}", self.major)
        } else {
            write!(formatter, "{}.{}", self.major, self.minor)
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
    pub const UNKNOWN_VERSION: u32 = u32::MAX;

    pub fn is_compatible(self) -> bool {
        self == Self::Compatible
    }

    pub fn for_version(declared: PackFormatRange, current: PackFormat) -> Self {
        if declared.min.major == Self::UNKNOWN_VERSION {
            Self::Unknown
        } else if declared.max < current {
            Self::TooOld
        } else if current < declared.min {
            Self::TooNew
        } else {
            Self::Compatible
        }
    }

    pub fn description(self) -> Component {
        Component::translatable(format!("pack.incompatible.{}", self.key()), Vec::new())
            .styled(Style::empty().with_legacy_color(Some(ChatFormatting::Gray)))
    }

    pub fn confirmation(self) -> Component {
        Component::translatable(
            format!("pack.incompatible.confirm.{}", self.key()),
            Vec::new(),
        )
    }

    fn key(self) -> &'static str {
        match self {
            Self::TooOld => "old",
            Self::TooNew => "new",
            Self::Unknown => "unknown",
            Self::Compatible => "compatible",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

impl DataPackConfig {
    /// Java `DataPackConfig(List<String>, List<String>)` takes immutable
    /// copies of both lists. Rust callers receive owned vectors, so this
    /// constructor performs the same boundary copy while retaining the
    /// existing field-based API used by repository selection.
    pub fn new<E, D, EI, DI>(enabled: EI, disabled: DI) -> Self
    where
        E: Into<String>,
        D: Into<String>,
        EI: IntoIterator<Item = E>,
        DI: IntoIterator<Item = D>,
    {
        Self {
            enabled: enabled.into_iter().map(Into::into).collect(),
            disabled: disabled.into_iter().map(Into::into).collect(),
        }
    }

    pub fn default_26_1_2() -> Self {
        Self::new([VANILLA_PACK_ID], std::iter::empty::<&str>())
    }

    pub fn from_properties(enabled: &str, disabled: &str) -> Self {
        Self::new(split_pack_list(enabled), split_pack_list(disabled))
    }

    pub fn enabled(&self) -> &[String] {
        &self.enabled
    }

    pub fn disabled(&self) -> &[String] {
        &self.disabled
    }

    /// Encodes Java `DataPackConfig.CODEC`'s exact field names and order.
    pub fn to_json(&self) -> Result<String, String> {
        let enabled = serde_json::to_string(&self.enabled).map_err(|error| error.to_string())?;
        let disabled = serde_json::to_string(&self.disabled).map_err(|error| error.to_string())?;
        Ok(format!(r#"{{"Enabled":{enabled},"Disabled":{disabled}}}"#))
    }

    /// Decodes the record codec shape used by level data and datapack
    /// configuration files. Unknown fields are ignored like Mojang's codec.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|error| format!("invalid DataPackConfig JSON: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "DataPackConfig must be a JSON object".to_string())?;
        let read_list = |name: &str| -> Result<Vec<String>, String> {
            object
                .get(name)
                .ok_or_else(|| format!("DataPackConfig missing {name}"))?
                .as_array()
                .ok_or_else(|| format!("DataPackConfig {name} must be an array"))?
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| format!("DataPackConfig {name} entries must be strings"))
                })
                .collect()
        };
        Ok(Self::new(read_list("Enabled")?, read_list("Disabled")?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataConfiguration {
    pub data_packs: DataPackConfig,
    pub enabled_features: FeatureFlagSet,
}

impl WorldDataConfiguration {
    pub const ENABLED_FEATURES_ID: &'static str = "enabled_features";

    pub fn default_26_1_2() -> Self {
        Self {
            data_packs: DataPackConfig::default_26_1_2(),
            enabled_features: feature_flags::default_flags_26_1_2(),
        }
    }

    /// Java `WorldDataConfiguration#expandFeatures` joins the supplied flags
    /// without changing the selected datapacks.
    pub fn expand_features(&self, new_enabled_features: FeatureFlagSet) -> Self {
        Self {
            data_packs: self.data_packs.clone(),
            enabled_features: self.enabled_features.join(new_enabled_features),
        }
    }

    /// Encodes the Java record codec's optional `DataPacks` and
    /// `enabled_features` fields using the server's feature registry names.
    pub fn to_json(&self, registry: &FeatureFlagRegistry) -> Result<String, String> {
        let data_packs = self.data_packs.to_json()?;
        let features = registry
            .to_names(self.enabled_features)
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();
        let features = serde_json::to_string(&features).map_err(|error| error.to_string())?;
        Ok(format!(r#"{{"DataPacks":{data_packs},"enabled_features":{features}}}"#))
    }

    /// Decodes the Java record codec shape. Missing optional fields receive
    /// `DEFAULT` values, while present fields are validated by their codecs.
    pub fn from_json(raw: &str, registry: &FeatureFlagRegistry) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|error| format!("invalid WorldDataConfiguration JSON: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "WorldDataConfiguration must be a JSON object".to_string())?;
        let data_packs = object
            .get("DataPacks")
            .map(|value| DataPackConfig::from_json(&value.to_string()))
            .transpose()?
            .unwrap_or_else(DataPackConfig::default_26_1_2);
        let enabled_features = match object.get(Self::ENABLED_FEATURES_ID) {
            Some(value) => {
                let values = value
                    .as_array()
                    .ok_or_else(|| "enabled_features must be an array".to_string())?;
                let names = values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .ok_or_else(|| "enabled_features entries must be strings".to_string())
                            .and_then(Identifier::parse)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                registry
                    .resolve_names(&names)
                    .map_err(|unknown| format!("unknown feature flags: {unknown:?}"))?
            }
            None => feature_flags::default_flags_26_1_2(),
        };
        Ok(Self {
            data_packs,
            enabled_features,
        })
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
    value.split(',').map(|entry| entry.trim().to_string()).collect()
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

mod metadata_parser;
pub use metadata_parser::parse_pack_metadata;

#[cfg(test)]
mod tests;
