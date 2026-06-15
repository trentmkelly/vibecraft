#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryDataLoaderValidatorKind {
    None,
    NonEmpty,
    TimelineValidateRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryDataLoaderRegistryData {
    pub key: &'static str,
    pub element_codec: &'static str,
    pub validator: RegistryDataLoaderValidatorKind,
}

impl RegistryDataLoaderRegistryData {
    pub const fn new(key: &'static str, element_codec: &'static str) -> Self {
        Self {
            key,
            element_codec,
            validator: RegistryDataLoaderValidatorKind::None,
        }
    }

    pub const fn with_validator(
        key: &'static str,
        element_codec: &'static str,
        validator: RegistryDataLoaderValidatorKind,
    ) -> Self {
        Self {
            key,
            element_codec,
            validator,
        }
    }

    pub fn run_with_arguments(&self) -> (&'static str, &'static str) {
        (self.key, self.element_codec)
    }
}

pub const WORLDGEN_REGISTRIES: &[RegistryDataLoaderRegistryData] = &[
    RegistryDataLoaderRegistryData::new("minecraft:dimension_type", "DimensionType.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:worldgen/biome", "Biome.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:chat_type", "ChatType.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/configured_carver",
        "ConfiguredWorldCarver.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/configured_feature",
        "ConfiguredFeature.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/placed_feature",
        "PlacedFeature.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:worldgen/structure", "Structure.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/structure_set",
        "StructureSet.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/processor_list",
        "StructureProcessorType.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/template_pool",
        "StructureTemplatePool.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/noise_settings",
        "NoiseGeneratorSettings.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/noise",
        "NormalNoise.NoiseParameters.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/density_function",
        "DensityFunction.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/world_preset",
        "WorldPreset.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/flat_level_generator_preset",
        "FlatLevelGeneratorPreset.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:trim_pattern", "TrimPattern.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:trim_material", "TrimMaterial.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:trial_spawner_config",
        "TrialSpawnerConfig.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:wolf_variant",
        "WolfVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:wolf_sound_variant",
        "WolfSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:pig_variant",
        "PigVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:pig_sound_variant",
        "PigSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:frog_variant",
        "FrogVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cat_variant",
        "CatVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cat_sound_variant",
        "CatSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cow_variant",
        "CowVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cow_sound_variant",
        "CowSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:chicken_variant",
        "ChickenVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:chicken_sound_variant",
        "ChickenSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:zombie_nautilus_variant",
        "ZombieNautilusVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:painting_variant",
        "PaintingVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::new("minecraft:damage_type", "DamageType.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        "MultiNoiseBiomeSourceParameterList.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:banner_pattern", "BannerPattern.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:enchantment", "Enchantment.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:enchantment_provider",
        "EnchantmentProvider.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:jukebox_song", "JukeboxSong.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:instrument", "Instrument.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:test_environment",
        "TestEnvironmentDefinition.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:test_instance",
        "GameTestInstance.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:dialog", "Dialog.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:world_clock", "WorldClock.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:timeline",
        "Timeline.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::TimelineValidateRegistry,
    ),
    RegistryDataLoaderRegistryData::new("minecraft:villager_trade", "VillagerTrade.CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:trade_set", "TradeSet.CODEC"),
];

pub const DIMENSION_REGISTRIES: &[RegistryDataLoaderRegistryData] =
    &[RegistryDataLoaderRegistryData::new(
        "minecraft:dimension",
        "LevelStem.CODEC",
    )];

pub const SYNCHRONIZED_REGISTRIES: &[RegistryDataLoaderRegistryData] = &[
    RegistryDataLoaderRegistryData::new("minecraft:worldgen/biome", "Biome.NETWORK_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:chat_type", "ChatType.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:trim_pattern", "TrimPattern.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:trim_material", "TrimMaterial.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:wolf_variant",
        "WolfVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:wolf_sound_variant",
        "WolfSoundVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:pig_variant",
        "PigVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:pig_sound_variant",
        "PigSoundVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:frog_variant",
        "FrogVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cat_variant",
        "CatVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cat_sound_variant",
        "CatSoundVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cow_sound_variant",
        "CowSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:cow_variant",
        "CowVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:chicken_sound_variant",
        "ChickenSoundVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:chicken_variant",
        "ChickenVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:zombie_nautilus_variant",
        "ZombieNautilusVariant.NETWORK_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::with_validator(
        "minecraft:painting_variant",
        "PaintingVariant.DIRECT_CODEC",
        RegistryDataLoaderValidatorKind::NonEmpty,
    ),
    RegistryDataLoaderRegistryData::new("minecraft:dimension_type", "DimensionType.NETWORK_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:damage_type", "DamageType.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:banner_pattern", "BannerPattern.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:enchantment", "Enchantment.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:jukebox_song", "JukeboxSong.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:instrument", "Instrument.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new(
        "minecraft:test_environment",
        "TestEnvironmentDefinition.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new(
        "minecraft:test_instance",
        "GameTestInstance.DIRECT_CODEC",
    ),
    RegistryDataLoaderRegistryData::new("minecraft:dialog", "Dialog.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:world_clock", "WorldClock.DIRECT_CODEC"),
    RegistryDataLoaderRegistryData::new("minecraft:timeline", "Timeline.NETWORK_CODEC"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryDataLoaderFactoryKind {
    ResourceManager,
    Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadTaskStub {
    pub registry: &'static str,
    pub load_error: Option<&'static str>,
    pub freeze_error: Option<&'static str>,
    pub validate_error: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryDataLoadOutcome {
    pub factory_kind: RegistryDataLoaderFactoryKind,
    pub context_registries: Vec<&'static str>,
    pub task_registries: Vec<&'static str>,
    pub operations: Vec<&'static str>,
    pub frozen_registries: Vec<&'static str>,
    pub final_registries: Vec<&'static str>,
    pub loading_errors: BTreeMap<&'static str, &'static str>,
    pub report: Option<String>,
}

pub fn load_from_resource_manager(
    context_registries: &[&'static str],
    tasks: &[RegistryLoadTaskStub],
) -> RegistryDataLoadOutcome {
    load_with_factory(
        RegistryDataLoaderFactoryKind::ResourceManager,
        context_registries,
        tasks,
    )
}

pub fn load_from_network(
    context_registries: &[&'static str],
    tasks: &[RegistryLoadTaskStub],
) -> RegistryDataLoadOutcome {
    load_with_factory(
        RegistryDataLoaderFactoryKind::Network,
        context_registries,
        tasks,
    )
}

pub fn load_with_factory(
    factory_kind: RegistryDataLoaderFactoryKind,
    context_registries: &[&'static str],
    tasks: &[RegistryLoadTaskStub],
) -> RegistryDataLoadOutcome {
    let mut loading_errors = BTreeMap::new();
    let mut operations = match factory_kind {
        RegistryDataLoaderFactoryKind::ResourceManager => {
            vec!["ResourceManagerRegistryLoadTask(Lifecycle.stable())"]
        }
        RegistryDataLoaderFactoryKind::Network => {
            vec!["NetworkRegistryLoadTask(Lifecycle.stable())"]
        }
    };
    operations.push("createContext(contextRegistries, loadTasks)");
    operations.push("load each task with contextAndNewRegistries");

    for task in tasks {
        if let Some(error) = task.load_error {
            loading_errors.insert(task.registry, error);
        }
    }

    operations.push("CompletableFuture.allOf(loadCompletions)");
    let mut frozen_registries = Vec::new();
    for task in tasks {
        if let Some(error) = task.freeze_error {
            loading_errors.insert(task.registry, error);
        } else {
            frozen_registries.push(task.registry);
        }
    }
    operations.push("freezeRegistry");
    if !loading_errors.is_empty() {
        let report = create_report_with_brief_info(&loading_errors);
        operations.push("logErrors");
        return RegistryDataLoadOutcome {
            factory_kind,
            context_registries: context_registries.to_vec(),
            task_registries: tasks.iter().map(|task| task.registry).collect(),
            operations,
            frozen_registries,
            final_registries: Vec::new(),
            loading_errors,
            report: Some(report),
        };
    }

    let mut final_registries = Vec::new();
    for task in tasks {
        if let Some(error) = task.validate_error {
            loading_errors.insert(task.registry, error);
        } else if frozen_registries.contains(&task.registry) {
            final_registries.push(task.registry);
        }
    }
    operations.push("validateRegistry");
    if !loading_errors.is_empty() {
        let report = create_report_with_brief_info(&loading_errors);
        operations.push("logErrors");
        return RegistryDataLoadOutcome {
            factory_kind,
            context_registries: context_registries.to_vec(),
            task_registries: tasks.iter().map(|task| task.registry).collect(),
            operations,
            frozen_registries,
            final_registries: Vec::new(),
            loading_errors,
            report: Some(report),
        };
    }

    operations.push("new RegistryAccess.ImmutableRegistryAccess(registries).freeze()");
    RegistryDataLoadOutcome {
        factory_kind,
        context_registries: context_registries.to_vec(),
        task_registries: tasks.iter().map(|task| task.registry).collect(),
        operations,
        frozen_registries,
        final_registries,
        loading_errors,
        report: None,
    }
}

pub fn create_context_lookup_order(
    context_registries: &[&'static str],
    load_tasks: &[RegistryLoadTaskStub],
) -> Vec<&'static str> {
    context_registries
        .iter()
        .copied()
        .chain(load_tasks.iter().map(|task| task.registry))
        .collect()
}

pub fn create_report_with_brief_info(errors: &BTreeMap<&'static str, &'static str>) -> String {
    let mut details = String::from("Failed to load registries due to errors");
    for (key, error) in errors {
        details.push_str("\n\t\t");
        details.push_str(key);
        details.push_str(": ");
        details.push_str(error);
    }
    details
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_DATA_LOADER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryDataLoader.java");

    #[test]
    fn registry_data_lists_match_java_order_codecs_and_validators() {
        assert_java_contains(
            REGISTRY_DATA_LOADER_JAVA,
            &[
                "public static final List<RegistryDataLoader.RegistryData<?>> WORLDGEN_REGISTRIES = List.of(",
                "public static final List<RegistryDataLoader.RegistryData<?>> DIMENSION_REGISTRIES = List.of(",
                "public static final List<RegistryDataLoader.RegistryData<?>> SYNCHRONIZED_REGISTRIES = List.of(",
                "new RegistryDataLoader.RegistryData<>(Registries.TIMELINE, Timeline.DIRECT_CODEC, Timeline::validateRegistry)",
                "new RegistryDataLoader.RegistryData<>(Registries.TIMELINE, Timeline.NETWORK_CODEC)",
                "new RegistryDataLoader.RegistryData<>(Registries.VILLAGER_TRADE, VillagerTrade.CODEC)",
                "new RegistryDataLoader.RegistryData<>(Registries.TRADE_SET, TradeSet.CODEC)",
            ],
        );

        assert_eq!(WORLDGEN_REGISTRIES.len(), 45);
        assert_eq!(DIMENSION_REGISTRIES.len(), 1);
        assert_eq!(SYNCHRONIZED_REGISTRIES.len(), 28);
        assert_eq!(
            WORLDGEN_REGISTRIES
                .iter()
                .filter(|entry| entry.validator == RegistryDataLoaderValidatorKind::NonEmpty)
                .count(),
            13
        );
        assert_eq!(
            WORLDGEN_REGISTRIES.last(),
            Some(&RegistryDataLoaderRegistryData::new(
                "minecraft:trade_set",
                "TradeSet.CODEC",
            ))
        );
        assert_eq!(
            WORLDGEN_REGISTRIES[42],
            RegistryDataLoaderRegistryData::with_validator(
                "minecraft:timeline",
                "Timeline.DIRECT_CODEC",
                RegistryDataLoaderValidatorKind::TimelineValidateRegistry,
            )
        );
        assert_eq!(
            SYNCHRONIZED_REGISTRIES
                .iter()
                .map(|entry| entry.key)
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
    }

    #[test]
    fn public_load_entrypoints_choose_java_loader_factory_kind() {
        assert_java_contains(
            REGISTRY_DATA_LOADER_JAVA,
            &[
                "return new ResourceManagerRegistryLoadTask<>(data, Lifecycle.stable(), loadingErrors, resourceManager);",
                "return new NetworkRegistryLoadTask<>(data, Lifecycle.stable(), loadingErrors, entries, knownDataSource);",
                "return load(loaderFactory, contextRegistries, registriesToLoad, executor);",
            ],
        );

        let tasks = [RegistryLoadTaskStub {
            registry: "minecraft:chat_type",
            load_error: None,
            freeze_error: None,
            validate_error: None,
        }];
        assert_eq!(
            load_from_resource_manager(&[], &tasks).factory_kind,
            RegistryDataLoaderFactoryKind::ResourceManager
        );
        assert_eq!(
            load_from_network(&[], &tasks).factory_kind,
            RegistryDataLoaderFactoryKind::Network
        );
    }

    #[test]
    fn shared_load_pipeline_matches_java_context_freeze_error_and_validation_order() {
        assert_java_contains(
            REGISTRY_DATA_LOADER_JAVA,
            &[
                "Map<ResourceKey<?>, Exception> loadingErrors = new ConcurrentHashMap<>();",
                "loaderFactory.create((RegistryDataLoader.RegistryData<?>)r, loadingErrors)",
                "RegistryOps.RegistryInfoLookup contextAndNewRegistries = createContext(contextRegistries, loadTasks);",
                "loadCompletions[i] = loadTasks.get(i).load(contextAndNewRegistries, executor);",
                "CompletableFuture.allOf(loadCompletions).thenApplyAsync",
                "filter(task -> task.freezeRegistry(loadingErrors))",
                "if (!loadingErrors.isEmpty())",
                "flatMap(task -> task.validateRegistry(loadingErrors).stream()).toList()",
                "return new RegistryAccess.ImmutableRegistryAccess(registries).freeze();",
            ],
        );

        let tasks = [
            RegistryLoadTaskStub {
                registry: "minecraft:chat_type",
                load_error: None,
                freeze_error: None,
                validate_error: None,
            },
            RegistryLoadTaskStub {
                registry: "minecraft:wolf_variant",
                load_error: None,
                freeze_error: None,
                validate_error: None,
            },
        ];
        let outcome = load_from_resource_manager(&["minecraft:dimension_type"], &tasks);
        assert_eq!(
            create_context_lookup_order(&["minecraft:dimension_type"], &tasks),
            vec![
                "minecraft:dimension_type",
                "minecraft:chat_type",
                "minecraft:wolf_variant"
            ]
        );
        assert_eq!(
            outcome.final_registries,
            vec!["minecraft:chat_type", "minecraft:wolf_variant"]
        );
        assert_eq!(
            outcome.operations,
            vec![
                "ResourceManagerRegistryLoadTask(Lifecycle.stable())",
                "createContext(contextRegistries, loadTasks)",
                "load each task with contextAndNewRegistries",
                "CompletableFuture.allOf(loadCompletions)",
                "freezeRegistry",
                "validateRegistry",
                "new RegistryAccess.ImmutableRegistryAccess(registries).freeze()",
            ]
        );
        assert!(outcome.loading_errors.is_empty());

        let freeze_failure = load_from_network(
            &[],
            &[RegistryLoadTaskStub {
                registry: "minecraft:chat_type",
                load_error: None,
                freeze_error: Some("freeze failed"),
                validate_error: None,
            }],
        );
        assert_eq!(
            freeze_failure.operations,
            vec![
                "NetworkRegistryLoadTask(Lifecycle.stable())",
                "createContext(contextRegistries, loadTasks)",
                "load each task with contextAndNewRegistries",
                "CompletableFuture.allOf(loadCompletions)",
                "freezeRegistry",
                "logErrors",
            ]
        );
        assert_eq!(
            freeze_failure.report.as_deref(),
            Some("Failed to load registries due to errors\n\t\tminecraft:chat_type: freeze failed")
        );

        let validation_failure = load_from_resource_manager(
            &[],
            &[RegistryLoadTaskStub {
                registry: "minecraft:wolf_variant",
                load_error: None,
                freeze_error: None,
                validate_error: Some("Registry must be non-empty: minecraft:wolf_variant"),
            }],
        );
        assert_eq!(
            validation_failure.operations,
            vec![
                "ResourceManagerRegistryLoadTask(Lifecycle.stable())",
                "createContext(contextRegistries, loadTasks)",
                "load each task with contextAndNewRegistries",
                "CompletableFuture.allOf(loadCompletions)",
                "freezeRegistry",
                "validateRegistry",
                "logErrors",
            ]
        );
    }

    #[test]
    fn registry_data_run_with_arguments_forwards_key_and_codec() {
        assert_java_contains(
            REGISTRY_DATA_LOADER_JAVA,
            &[
                "public record RegistryData<T>(ResourceKey<? extends Registry<T>> key, Codec<T> elementCodec, RegistryValidator<T> validator)",
                "private RegistryData(final ResourceKey<? extends Registry<T>> key, final Codec<T> elementCodec)",
                "this(key, elementCodec, RegistryValidator.none());",
                "output.accept(this.key, this.elementCodec);",
            ],
        );

        assert_eq!(
            WORLDGEN_REGISTRIES[0].run_with_arguments(),
            ("minecraft:dimension_type", "DimensionType.DIRECT_CODEC")
        );
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
