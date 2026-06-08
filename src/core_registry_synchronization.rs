use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct IdentifierModel(String);

impl IdentifierModel {
    fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for IdentifierModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ResourceKeyModel {
    registry: IdentifierModel,
    location: IdentifierModel,
}

impl ResourceKeyModel {
    fn registry(location: &str) -> Self {
        Self {
            registry: IdentifierModel::new("minecraft:root"),
            location: IdentifierModel::new(location),
        }
    }

    fn element(registry: &ResourceKeyModel, location: &str) -> Self {
        Self {
            registry: registry.location.clone(),
            location: IdentifierModel::new(location),
        }
    }

    fn identifier(&self) -> &IdentifierModel {
        &self.location
    }
}

impl fmt::Display for ResourceKeyModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} / {}", self.registry, self.location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct KnownPackModel {
    namespace: &'static str,
    id: &'static str,
    version: &'static str,
}

impl KnownPackModel {
    fn new(namespace: &'static str, id: &'static str, version: &'static str) -> Self {
        Self {
            namespace,
            id,
            version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationInfoModel {
    known_pack_info: Option<KnownPackModel>,
}

impl RegistrationInfoModel {
    fn unknown_pack() -> Self {
        Self {
            known_pack_info: None,
        }
    }

    fn known_pack(pack: KnownPackModel) -> Self {
        Self {
            known_pack_info: Some(pack),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryElementModel {
    key: ResourceKeyModel,
    value: &'static str,
    registration_info: RegistrationInfoModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryDataModel {
    key: ResourceKeyModel,
    element_codec: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryModel {
    key: ResourceKeyModel,
    elements: Vec<RegistryElementModel>,
}

impl RegistryModel {
    fn new(key: ResourceKeyModel, elements: Vec<RegistryElementModel>) -> Self {
        Self { key, elements }
    }

    fn size(&self) -> usize {
        self.elements.len()
    }

    fn list_elements(&self) -> &[RegistryElementModel] {
        &self.elements
    }

    fn registration_info(&self, key: &ResourceKeyModel) -> Option<&RegistrationInfoModel> {
        self.elements
            .iter()
            .find(|element| &element.key == key)
            .map(|element| &element.registration_info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryEntryModel {
    key: ResourceKeyModel,
    value: RegistryModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryAccessModel {
    registries: BTreeMap<ResourceKeyModel, RegistryModel>,
}

impl RegistryAccessModel {
    fn new(registries: impl IntoIterator<Item = RegistryModel>) -> Self {
        Self {
            registries: registries
                .into_iter()
                .map(|registry| (registry.key.clone(), registry))
                .collect(),
        }
    }

    fn lookup(&self, key: &ResourceKeyModel) -> Option<&RegistryModel> {
        self.registries.get(key)
    }

    fn registries(&self) -> Vec<RegistryEntryModel> {
        self.registries
            .iter()
            .map(|(key, value)| RegistryEntryModel {
                key: key.clone(),
                value: value.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryLayerModel {
    Static,
    Worldgen,
    Dimensions,
    Reloadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayeredRegistryAccessModel {
    static_layer: RegistryAccessModel,
    worldgen_and_later: RegistryAccessModel,
}

impl LayeredRegistryAccessModel {
    fn new(static_layer: RegistryAccessModel, worldgen_and_later: RegistryAccessModel) -> Self {
        Self {
            static_layer,
            worldgen_and_later,
        }
    }

    fn get_layer(&self, layer: RegistryLayerModel) -> &RegistryAccessModel {
        match layer {
            RegistryLayerModel::Static => &self.static_layer,
            RegistryLayerModel::Worldgen
            | RegistryLayerModel::Dimensions
            | RegistryLayerModel::Reloadable => &self.worldgen_and_later,
        }
    }

    fn get_access_from(&self, layer: RegistryLayerModel) -> &RegistryAccessModel {
        match layer {
            RegistryLayerModel::Static => &self.static_layer,
            RegistryLayerModel::Worldgen
            | RegistryLayerModel::Dimensions
            | RegistryLayerModel::Reloadable => &self.worldgen_and_later,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackedRegistryEntryModel {
    id: IdentifierModel,
    data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackedRegistryEntryStreamCodecShape {
    first_field: &'static str,
    second_field: &'static str,
    second_field_codec: &'static str,
}

impl PackedRegistryEntryModel {
    fn stream_codec_shape() -> PackedRegistryEntryStreamCodecShape {
        PackedRegistryEntryStreamCodecShape {
            first_field: "Identifier.STREAM_CODEC -> id",
            second_field: "ByteBufCodecs.TAG.apply(ByteBufCodecs::optional) -> data",
            second_field_codec: "optional TAG",
        }
    }
}

fn synchronized_registries() -> Vec<RegistryDataModel> {
    [
        ("minecraft:worldgen/biome", "Biome.NETWORK_CODEC"),
        ("minecraft:chat_type", "ChatType.DIRECT_CODEC"),
        ("minecraft:trim_pattern", "TrimPattern.DIRECT_CODEC"),
        ("minecraft:trim_material", "TrimMaterial.DIRECT_CODEC"),
        ("minecraft:wolf_variant", "WolfVariant.NETWORK_CODEC"),
        (
            "minecraft:wolf_sound_variant",
            "WolfSoundVariant.NETWORK_CODEC",
        ),
        ("minecraft:pig_variant", "PigVariant.NETWORK_CODEC"),
        (
            "minecraft:pig_sound_variant",
            "PigSoundVariant.NETWORK_CODEC",
        ),
        ("minecraft:frog_variant", "FrogVariant.NETWORK_CODEC"),
        ("minecraft:cat_variant", "CatVariant.NETWORK_CODEC"),
        (
            "minecraft:cat_sound_variant",
            "CatSoundVariant.NETWORK_CODEC",
        ),
        (
            "minecraft:cow_sound_variant",
            "CowSoundVariant.DIRECT_CODEC",
        ),
        ("minecraft:cow_variant", "CowVariant.NETWORK_CODEC"),
        (
            "minecraft:chicken_sound_variant",
            "ChickenSoundVariant.DIRECT_CODEC",
        ),
        ("minecraft:chicken_variant", "ChickenVariant.NETWORK_CODEC"),
        (
            "minecraft:zombie_nautilus_variant",
            "ZombieNautilusVariant.NETWORK_CODEC",
        ),
        ("minecraft:painting_variant", "PaintingVariant.DIRECT_CODEC"),
        ("minecraft:dimension_type", "DimensionType.NETWORK_CODEC"),
        ("minecraft:damage_type", "DamageType.DIRECT_CODEC"),
        ("minecraft:banner_pattern", "BannerPattern.DIRECT_CODEC"),
        ("minecraft:enchantment", "Enchantment.DIRECT_CODEC"),
        ("minecraft:jukebox_song", "JukeboxSong.DIRECT_CODEC"),
        ("minecraft:instrument", "Instrument.DIRECT_CODEC"),
        (
            "minecraft:test_environment",
            "TestEnvironmentDefinition.DIRECT_CODEC",
        ),
        ("minecraft:test_instance", "GameTestInstance.DIRECT_CODEC"),
        ("minecraft:dialog", "Dialog.DIRECT_CODEC"),
        ("minecraft:world_clock", "WorldClock.DIRECT_CODEC"),
        ("minecraft:timeline", "Timeline.NETWORK_CODEC"),
    ]
    .into_iter()
    .map(|(key, element_codec)| RegistryDataModel {
        key: ResourceKeyModel::registry(key),
        element_codec,
    })
    .collect()
}

fn networkable_registries() -> BTreeSet<ResourceKeyModel> {
    synchronized_registries()
        .into_iter()
        .map(|registry_data| registry_data.key)
        .collect()
}

fn is_networkable(key: &ResourceKeyModel) -> bool {
    networkable_registries().contains(key)
}

fn pack_registries(
    registries: &RegistryAccessModel,
    client_known_packs: &BTreeSet<KnownPackModel>,
) -> Vec<(ResourceKeyModel, Vec<PackedRegistryEntryModel>)> {
    let mut output = Vec::new();
    for registry_data in synchronized_registries() {
        if let Some(registry) = registries.lookup(&registry_data.key) {
            output.push((
                registry.key.clone(),
                pack_registry(&registry_data, registry, client_known_packs),
            ));
        }
    }
    output
}

fn pack_registry(
    registry_data: &RegistryDataModel,
    registry: &RegistryModel,
    client_known_packs: &BTreeSet<KnownPackModel>,
) -> Vec<PackedRegistryEntryModel> {
    let mut packed_elements = Vec::with_capacity(registry.size());
    for element in registry.list_elements() {
        let can_skip_contents = registry
            .registration_info(&element.key)
            .and_then(|info| info.known_pack_info.as_ref())
            .is_some_and(|known_pack| client_known_packs.contains(known_pack));
        let data = if can_skip_contents {
            None
        } else {
            Some(format!(
                "{}::encodeStart({})",
                registry_data.element_codec, element.value
            ))
        };
        packed_elements.push(PackedRegistryEntryModel {
            id: element.key.identifier().clone(),
            data,
        });
    }
    packed_elements
}

fn owned_networkable_registries(access: &RegistryAccessModel) -> Vec<RegistryEntryModel> {
    access
        .registries()
        .into_iter()
        .filter(|entry| is_networkable(&entry.key))
        .collect()
}

fn networked_registries(registries: &LayeredRegistryAccessModel) -> Vec<RegistryEntryModel> {
    owned_networkable_registries(registries.get_access_from(RegistryLayerModel::Worldgen))
}

fn network_safe_registries(registries: &LayeredRegistryAccessModel) -> Vec<RegistryEntryModel> {
    let mut result = networked_registries(registries);
    result.extend(
        registries
            .get_layer(RegistryLayerModel::Static)
            .registries(),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry_key(id: &str) -> ResourceKeyModel {
        ResourceKeyModel::registry(id)
    }

    fn element(
        registry: &ResourceKeyModel,
        id: &'static str,
        value: &'static str,
    ) -> RegistryElementModel {
        RegistryElementModel {
            key: ResourceKeyModel::element(registry, id),
            value,
            registration_info: RegistrationInfoModel::unknown_pack(),
        }
    }

    fn known_element(
        registry: &ResourceKeyModel,
        id: &'static str,
        value: &'static str,
        pack: KnownPackModel,
    ) -> RegistryElementModel {
        RegistryElementModel {
            key: ResourceKeyModel::element(registry, id),
            value,
            registration_info: RegistrationInfoModel::known_pack(pack),
        }
    }

    #[test]
    fn synchronized_registry_manifest_matches_java_order_and_networkable_set() {
        let registries = synchronized_registries();
        assert_eq!(registries.len(), 28);
        assert_eq!(
            registries
                .iter()
                .map(|data| data.key.identifier().to_string())
                .collect::<Vec<_>>(),
            vec![
                "minecraft:worldgen/biome",
                "minecraft:chat_type",
                "minecraft:trim_pattern",
                "minecraft:trim_material",
                "minecraft:wolf_variant",
                "minecraft:wolf_sound_variant",
                "minecraft:pig_variant",
                "minecraft:pig_sound_variant",
                "minecraft:frog_variant",
                "minecraft:cat_variant",
                "minecraft:cat_sound_variant",
                "minecraft:cow_sound_variant",
                "minecraft:cow_variant",
                "minecraft:chicken_sound_variant",
                "minecraft:chicken_variant",
                "minecraft:zombie_nautilus_variant",
                "minecraft:painting_variant",
                "minecraft:dimension_type",
                "minecraft:damage_type",
                "minecraft:banner_pattern",
                "minecraft:enchantment",
                "minecraft:jukebox_song",
                "minecraft:instrument",
                "minecraft:test_environment",
                "minecraft:test_instance",
                "minecraft:dialog",
                "minecraft:world_clock",
                "minecraft:timeline",
            ]
        );
        assert_eq!(registries[0].element_codec, "Biome.NETWORK_CODEC");
        assert_eq!(registries[17].element_codec, "DimensionType.NETWORK_CODEC");
        assert_eq!(registries[27].element_codec, "Timeline.NETWORK_CODEC");
        assert!(is_networkable(&registry_key("minecraft:worldgen/biome")));
        assert!(!is_networkable(&registry_key(
            "minecraft:configured_feature"
        )));
    }

    #[test]
    fn pack_registries_visits_synchronized_registry_list_and_skips_missing_registries() {
        let biome_key = registry_key("minecraft:worldgen/biome");
        let chat_key = registry_key("minecraft:chat_type");
        let access = RegistryAccessModel::new([
            RegistryModel::new(
                chat_key.clone(),
                vec![element(&chat_key, "minecraft:chat", "chat-type")],
            ),
            RegistryModel::new(
                biome_key.clone(),
                vec![element(&biome_key, "minecraft:plains", "plains-biome")],
            ),
            RegistryModel::new(
                registry_key("minecraft:configured_feature"),
                vec![RegistryElementModel {
                    key: ResourceKeyModel::element(
                        &registry_key("minecraft:configured_feature"),
                        "minecraft:ore",
                    ),
                    value: "ore",
                    registration_info: RegistrationInfoModel::unknown_pack(),
                }],
            ),
        ]);

        let packed = pack_registries(&access, &BTreeSet::new());
        assert_eq!(
            packed
                .iter()
                .map(|(key, _)| key.identifier().to_string())
                .collect::<Vec<_>>(),
            vec!["minecraft:worldgen/biome", "minecraft:chat_type"]
        );
        assert_eq!(
            packed[0].1,
            vec![PackedRegistryEntryModel {
                id: IdentifierModel::new("minecraft:plains"),
                data: Some("Biome.NETWORK_CODEC::encodeStart(plains-biome)".to_string()),
            }]
        );
        assert_eq!(
            packed[1].1,
            vec![PackedRegistryEntryModel {
                id: IdentifierModel::new("minecraft:chat"),
                data: Some("ChatType.DIRECT_CODEC::encodeStart(chat-type)".to_string()),
            }]
        );
    }

    #[test]
    fn pack_registry_omits_data_when_client_knows_registered_pack() {
        let pack = KnownPackModel::new("minecraft", "vanilla", "26.1.2");
        let biome_key = registry_key("minecraft:worldgen/biome");
        let registry = RegistryModel::new(
            biome_key.clone(),
            vec![
                known_element(&biome_key, "minecraft:plains", "plains-biome", pack.clone()),
                element(&biome_key, "minecraft:desert", "desert-biome"),
            ],
        );
        let access = RegistryAccessModel::new([registry]);
        let packed = pack_registries(&access, &BTreeSet::from([pack]));

        assert_eq!(
            packed[0].1,
            vec![
                PackedRegistryEntryModel {
                    id: IdentifierModel::new("minecraft:plains"),
                    data: None,
                },
                PackedRegistryEntryModel {
                    id: IdentifierModel::new("minecraft:desert"),
                    data: Some("Biome.NETWORK_CODEC::encodeStart(desert-biome)".to_string()),
                },
            ]
        );
    }

    #[test]
    fn pack_registry_encodes_known_pack_content_when_client_does_not_advertise_it() {
        let server_pack = KnownPackModel::new("minecraft", "vanilla", "26.1.2");
        let client_pack = KnownPackModel::new("minecraft", "vanilla", "older");
        let dimension_key = registry_key("minecraft:dimension_type");
        let registry = RegistryModel::new(
            dimension_key.clone(),
            vec![known_element(
                &dimension_key,
                "minecraft:overworld",
                "overworld-dimension",
                server_pack,
            )],
        );
        let access = RegistryAccessModel::new([registry]);
        let packed = pack_registries(&access, &BTreeSet::from([client_pack]));

        assert_eq!(
            packed[0].1[0],
            PackedRegistryEntryModel {
                id: IdentifierModel::new("minecraft:overworld"),
                data: Some(
                    "DimensionType.NETWORK_CODEC::encodeStart(overworld-dimension)".to_string()
                ),
            }
        );
    }

    #[test]
    fn networked_and_network_safe_registry_streams_match_java_layer_rules() {
        let biome_key = registry_key("minecraft:worldgen/biome");
        let configured_feature_key = registry_key("minecraft:configured_feature");
        let root_key = registry_key("minecraft:root");
        let static_custom_key = registry_key("minecraft:static_custom");
        let layers = LayeredRegistryAccessModel::new(
            RegistryAccessModel::new([
                RegistryModel::new(root_key.clone(), Vec::new()),
                RegistryModel::new(static_custom_key.clone(), Vec::new()),
            ]),
            RegistryAccessModel::new([
                RegistryModel::new(biome_key.clone(), Vec::new()),
                RegistryModel::new(configured_feature_key, Vec::new()),
            ]),
        );

        assert_eq!(
            networked_registries(&layers)
                .into_iter()
                .map(|entry| entry.key)
                .collect::<Vec<_>>(),
            vec![biome_key]
        );
        assert_eq!(
            layers
                .get_access_from(RegistryLayerModel::Dimensions)
                .registries()
                .len(),
            2
        );
        assert_eq!(
            layers
                .get_access_from(RegistryLayerModel::Reloadable)
                .registries()
                .len(),
            2
        );
        assert_eq!(
            network_safe_registries(&layers)
                .into_iter()
                .map(|entry| entry.key)
                .collect::<Vec<_>>(),
            vec![
                registry_key("minecraft:worldgen/biome"),
                root_key,
                static_custom_key
            ]
        );
    }

    #[test]
    fn packed_registry_entry_stream_codec_shape_matches_java_composite_fields() {
        assert_eq!(
            PackedRegistryEntryModel::stream_codec_shape(),
            PackedRegistryEntryStreamCodecShape {
                first_field: "Identifier.STREAM_CODEC -> id",
                second_field: "ByteBufCodecs.TAG.apply(ByteBufCodecs::optional) -> data",
                second_field_codec: "optional TAG",
            }
        );
    }
}
