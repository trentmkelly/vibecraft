use super::{
    builtin_registry_manifest_26_1_2, feature_flags, registries, FeatureFlagRegistry, Holder,
    HolderSet, Identifier, Lifecycle, Registry, TagKey,
};

const TAG_KEY_JAVA: &str = vibecraft_java_source!("/net/minecraft/tags/TagKey.java");

const INTENTIONALLY_OMITTED_REGISTRIES: &[(&str, &str)] = &[
    (
        "minecraft:advancement",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:banner_pattern",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:cat_sound_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:cat_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:chat_type",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:chicken_sound_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:chicken_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:cow_sound_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:cow_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:damage_type",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:dialog",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:dimension",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:dimension_type",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:enchantment",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:enchantment_provider",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:frog_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:instrument",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:item_modifier",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:jukebox_song",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:loot_table",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:painting_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:pig_sound_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:pig_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:predicate",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:recipe",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:test_environment",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:test_instance",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:timeline",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:trade_set",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:trial_spawner",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:trim_material",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:trim_pattern",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:villager_trade",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:wolf_sound_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:wolf_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:world_clock",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/biome",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/configured_carver",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/configured_feature",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/density_function",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/flat_level_generator_preset",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/noise",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/noise_settings",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/placed_feature",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/processor_list",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/structure",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/structure_set",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/template_pool",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:worldgen/world_preset",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
    (
        "minecraft:zombie_nautilus_variant",
        "not referenced by BuiltInRegistries in 26.1.2",
    ),
];

fn parse_java_constant_to_path_mapping(source: &str) -> std::collections::BTreeMap<String, String> {
    let mut mapping = std::collections::BTreeMap::new();
    let create_call = "createRegistryKey(";
    let mut cursor = 0usize;

    while let Some(create_offset) = source[cursor..].find(create_call) {
        let create_pos = cursor + create_offset;
        let before_call = &source[..create_pos];
        let line_start = before_call.rfind('\n').map_or(0, |idx| idx + 1);
        let before_call_line = &before_call[line_start..];
        if let Some(eq_pos) = before_call_line.rfind('=') {
            let lhs = before_call_line[..eq_pos].trim();
            if let Some(constant) = lhs.split_whitespace().last() {
                let after_call = &source[create_pos + create_call.len()..];
                if let Some(start_quote) = after_call.find('\"') {
                    let quoted = &after_call[start_quote + 1..];
                    if let Some(end_quote) = quoted.find('\"') {
                        mapping.insert(constant.to_string(), quoted[..end_quote].to_string());
                    }
                }
            }
        }
        cursor = create_pos + create_call.len();
    }

    assert!(
        !mapping.is_empty(),
        "could not parse java registry constant mapping"
    );
    mapping
}

fn is_java_all_caps_registry_identifier(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn parse_java_registry_refs_from_source(source: &str, prefix: &str) -> Vec<String> {
    let mut constants = Vec::new();
    let mut cursor = source;

    while let Some(index) = cursor.find(prefix) {
        let after = &cursor[index + prefix.len()..];
        let mut len = 0usize;
        for ch in after.bytes() {
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                len += 1;
            } else {
                break;
            }
        }
        if len > 0 {
            let candidate = &after[..len];
            if is_java_all_caps_registry_identifier(candidate) {
                constants.push(candidate.to_string());
            }
        }
        cursor = &after[len..];
    }

    constants
}

fn parse_java_registry_ids() -> std::collections::BTreeSet<String> {
    let source = vibecraft_java_source!("/net/minecraft/core/registries/Registries.java");
    let mut registry_ids = std::collections::BTreeSet::new();
    for id in parse_java_constant_to_path_mapping(source).values() {
        registry_ids.insert(format!("minecraft:{id}", id = id));
    }
    if registry_ids.is_empty() {
        panic!("could not parse java registry ids");
    }
    registry_ids
}

fn parse_builtin_registry_ids() -> std::collections::BTreeSet<String> {
    let registries_java = vibecraft_java_source!("/net/minecraft/core/registries/Registries.java");
    let mapping = parse_java_constant_to_path_mapping(registries_java);
    let source = vibecraft_java_source!("/net/minecraft/core/registries/BuiltInRegistries.java");
    let mut registry_ids = std::collections::BTreeSet::new();
    for constant in parse_java_registry_refs_from_source(source, "Registries.") {
        if constant == "ROOT_REGISTRY_NAME" || constant == "REGISTRY" {
            continue;
        }
        if let Some(id) = mapping.get(&constant) {
            registry_ids.insert(format!("minecraft:{id}", id = id));
        } else {
            panic!("unmapped registry constant {constant} in BuiltInRegistries.java");
        }
    }
    if registry_ids.is_empty() {
        panic!("could not parse built-in registry ids");
    }
    registry_ids
}

#[test]
fn parses_default_namespace_identifiers() {
    let id = Identifier::parse("stone").unwrap();
    assert_eq!(id.namespace(), "minecraft");
    assert_eq!(id.path(), "stone");
    assert_eq!(id.to_string(), "minecraft:stone");
}

#[test]
fn rejects_invalid_identifier_case() {
    assert!(Identifier::parse("Minecraft:Stone").is_err());
}

#[test]
fn identifier_factory_and_validation_helpers_match_java_resource_location() {
    let source = vibecraft_java_source!("/net/minecraft/resources/Identifier.java");
    for sentinel in [
        "public static final Codec<Identifier> CODEC = Codec.STRING.comapFlatMap(Identifier::read, Identifier::toString).stable();",
        "public static final char NAMESPACE_SEPARATOR = ':';",
        "public static final String DEFAULT_NAMESPACE = \"minecraft\";",
        "public static Identifier withDefaultNamespace(final String path)",
        "public static @Nullable Identifier tryBuild(final String namespace, final String path)",
        "if (namespace.equals(\"..\"))",
        "return c == '_' || c == '-' || c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '.';",
        "return c == '_' || c == '-' || c >= 'a' && c <= 'z' || c >= '0' && c <= '9' || c == '/' || c == '.';",
    ] {
        if !source.is_empty() {
            assert!(
                source.contains(sentinel),
                "Identifier.java sentinel missing: {sentinel}"
            );
        }
    }

    assert_eq!(Identifier::DEFAULT_NAMESPACE, "minecraft");
    assert_eq!(Identifier::REALMS_NAMESPACE, "realms");
    assert_eq!(Identifier::ALLOWED_NAMESPACE_CHARACTERS, "[a-z0-9_.-]");

    let dotted_namespace = Identifier::from_namespace_and_path("my.pack", "path/to.file").unwrap();
    assert_eq!(dotted_namespace.to_string(), "my.pack:path/to.file");
    assert_eq!(Identifier::with_default_namespace("").unwrap().to_string(), "minecraft:");
    assert_eq!(Identifier::parse(":stone").unwrap().to_string(), "minecraft:stone");
    assert_eq!(Identifier::parse("mod:").unwrap().to_string(), "mod:");
    assert_eq!(
        Identifier::by_separator("mod|thing", '|').unwrap().to_string(),
        "mod:thing"
    );

    assert!(Identifier::is_valid_namespace(""));
    assert!(!Identifier::is_valid_namespace(".."));
    assert!(Identifier::is_valid_namespace("a.b_c-1"));
    assert!(Identifier::is_valid_path(""));
    assert!(Identifier::is_valid_path("foo/bar.baz-1"));
    assert!(!Identifier::is_valid_path("foo:bar"));

    assert_eq!(
        Identifier::try_parse("Minecraft:stone"),
        None,
        "tryParse returns null instead of throwing on invalid namespace"
    );
    assert_eq!(
        Identifier::try_build("..", "stone"),
        None,
        "tryBuild rejects the special parent-directory namespace"
    );
    assert_eq!(
        Identifier::read("Upper:stone").unwrap_err(),
        "Not a valid resource location: Upper:stone Non [a-z0-9_.-] character in namespace of identifier: Upper:stone"
    );
    assert_eq!(
        Identifier::read("minecraft:Bad").unwrap_err(),
        "Not a valid resource location: minecraft:Bad Non [a-z0-9/._-] character in path of location: minecraft:Bad"
    );
}

#[test]
fn identifier_derived_strings_and_order_match_java() {
    let source = vibecraft_java_source!("/net/minecraft/resources/Identifier.java");
    for sentinel in [
        "public int compareTo(final Identifier o) {\n      int result = this.path.compareTo(o.path);",
        "return this.toString().replace('/', '_').replace(':', '_');",
        "return this.namespace.equals(\"minecraft\") ? this.path : this.toLanguageKey();",
        "return this.namespace.equals(\"minecraft\") ? this.path : this.toString();",
        "return prefix + \".\" + this.toLanguageKey() + \".\" + suffix;",
    ] {
        if !source.is_empty() {
            assert!(
                source.contains(sentinel),
                "Identifier.java sentinel missing: {sentinel}"
            );
        }
    }

    let id = Identifier::parse("custom:block/stone").unwrap();
    assert_eq!(id.with_path("other/path").unwrap().to_string(), "custom:other/path");
    assert_eq!(
        id.with_modified_path(|path| format!("prefix/{path}"))
            .unwrap()
            .to_string(),
        "custom:prefix/block/stone"
    );
    assert_eq!(id.with_prefix("pre_").unwrap().to_string(), "custom:pre_block/stone");
    assert_eq!(
        id.with_suffix("_post").unwrap().to_string(),
        "custom:block/stone_post"
    );
    assert_eq!(id.to_debug_file_name(), "custom_block_stone");
    assert_eq!(id.to_language_key(), "custom.block/stone");
    assert_eq!(id.to_short_language_key(), "custom.block/stone");
    assert_eq!(id.to_short_string(), "custom:block/stone");
    assert_eq!(id.to_language_key_with_prefix("block"), "block.custom.block/stone");
    assert_eq!(
        id.to_language_key_with_prefix_and_suffix("block", "name"),
        "block.custom.block/stone.name"
    );
    assert_eq!(
        id.resolve_against(std::path::Path::new("assets")),
        std::path::PathBuf::from("assets/custom/block/stone")
    );

    let vanilla = Identifier::parse("minecraft:stone").unwrap();
    assert_eq!(vanilla.to_short_language_key(), "stone");
    assert_eq!(vanilla.to_short_string(), "stone");

    let mut sorted = vec![
        Identifier::parse("minecraft:z").unwrap(),
        Identifier::parse("a:same").unwrap(),
        Identifier::parse("minecraft:a").unwrap(),
        Identifier::parse("b:same").unwrap(),
    ];
    sorted.sort();
    assert_eq!(
        sorted.into_iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        vec!["minecraft:a", "a:same", "b:same", "minecraft:z"]
    );
}

#[test]
fn registry_assigns_stable_ids_and_freezes() {
    let mut registry = Registry::new(Identifier::parse(registries::BLOCK).unwrap());
    let stone = Identifier::parse("stone").unwrap();
    let dirt = Identifier::parse("dirt").unwrap();

    registry
        .register(stone.clone(), "stone block", Lifecycle::Stable)
        .unwrap();
    registry
        .register(dirt.clone(), "dirt block", Lifecycle::Stable)
        .unwrap();
    registry.freeze();

    assert_eq!(registry.get(&stone).unwrap().id(), 0);
    assert_eq!(registry.get_by_id(1).unwrap().value(), &"dirt block");
    assert!(registry
        .register(
            Identifier::parse("grass_block").unwrap(),
            "grass",
            Lifecycle::Stable
        )
        .is_err());
}

#[test]
fn registry_snapshots_round_trip_in_numeric_id_order() {
    let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
    registry
        .register(
            Identifier::parse("stick").unwrap(),
            "stick item".to_string(),
            Lifecycle::Stable,
        )
        .unwrap();
    registry
        .register(
            Identifier::parse("trial_key").unwrap(),
            "trial key item".to_string(),
            Lifecycle::Experimental,
        )
        .unwrap();

    let snapshot = registry.serialize_with(Clone::clone);
    assert_eq!(
        snapshot.registry,
        Identifier::parse(registries::ITEM).unwrap()
    );
    assert_eq!(snapshot.entries[0].id, 0);
    assert_eq!(
        snapshot.entries[1].location,
        Identifier::parse("trial_key").unwrap()
    );

    let decoded = Registry::deserialize_with(snapshot, |value| Ok(value.to_string())).unwrap();
    assert_eq!(
        decoded
            .get(&Identifier::parse("stick").unwrap())
            .unwrap()
            .value(),
        "stick item"
    );
    assert_eq!(
        decoded.get_by_id(1).unwrap().lifecycle(),
        Lifecycle::Experimental
    );
}

#[test]
fn registry_backed_keys_round_trip_json_nbt_and_network_forms() {
    let key = super::ResourceKey::<String>::new(
        Identifier::parse(registries::ITEM).unwrap(),
        Identifier::parse("minecraft:stick").unwrap(),
    );

    let decoded_json = super::ResourceKey::<String>::from_json_object(&key.to_json_object())
        .expect("json object should decode");
    assert_eq!(decoded_json, key);

    let decoded_nbt =
        super::ResourceKey::<String>::from_nbt(&key.to_nbt()).expect("nbt should decode");
    assert_eq!(decoded_nbt, key);

    let mut bytes = Vec::new();
    key.write_network(&mut bytes).unwrap();
    let mut input = crate::network::codec::cursor(bytes);
    assert_eq!(
        super::ResourceKey::<String>::read_network(&mut input).unwrap(),
        key
    );
}

#[test]
fn resource_key_helpers_match_java_registry_key_contracts() {
    let source = vibecraft_java_source!("/net/minecraft/resources/ResourceKey.java");
    for sentinel in [
        "private static final ConcurrentMap<ResourceKey.InternKey, ResourceKey<?>> VALUES",
        "public static <T> ResourceKey<T> create(final ResourceKey<? extends Registry<T>> registryName, final Identifier location)",
        "return create(registryName.identifier, location);",
        "public static <T> ResourceKey<Registry<T>> createRegistryKey(final Identifier identifier)",
        "return create(Registries.ROOT_REGISTRY_NAME, identifier);",
        "return \"ResourceKey[\" + this.registryName + \" / \" + this.identifier + \"]\";",
        "return this.registryName.equals(registry.identifier());",
        "return this.isFor(registry) ? Optional.of((ResourceKey<E>)this) : Optional.empty();",
        "return createRegistryKey(this.registryName);",
    ] {
        assert!(
            source.contains(sentinel),
            "ResourceKey.java sentinel missing: {sentinel}"
        );
    }

    let item_registry = super::ResourceKey::<String>::create_registry_key(
        Identifier::parse(registries::ITEM).unwrap(),
    );
    let block_registry = super::ResourceKey::<String>::create_registry_key(
        Identifier::parse(registries::BLOCK).unwrap(),
    );
    assert_eq!(item_registry.registry(), &Identifier::parse(registries::ROOT).unwrap());
    assert_eq!(item_registry.location(), &Identifier::parse(registries::ITEM).unwrap());
    assert_eq!(
        item_registry.to_string(),
        "ResourceKey[minecraft:root / minecraft:item]"
    );

    let stick = super::ResourceKey::<String>::create(
        &item_registry,
        Identifier::parse("minecraft:stick").unwrap(),
    );
    assert_eq!(stick.registry(), &Identifier::parse(registries::ITEM).unwrap());
    assert_eq!(stick.location(), &Identifier::parse("minecraft:stick").unwrap());
    let stick_registry_key = stick.registry_key();
    assert_eq!(stick_registry_key.registry(), item_registry.registry());
    assert_eq!(stick_registry_key.location(), item_registry.location());
    assert_eq!(stick.to_string(), "ResourceKey[minecraft:item / minecraft:stick]");
    assert!(stick.is_for(&item_registry));
    assert!(!stick.is_for(&block_registry));
    assert_eq!(stick.cast(&item_registry), Some(stick.clone()));
    assert_eq!(stick.cast::<String>(&block_registry), None);
}

#[test]
fn resource_key_bound_codecs_encode_only_location_like_java() {
    let source = vibecraft_java_source!("/net/minecraft/resources/ResourceKey.java");
    for sentinel in [
        "return Identifier.CODEC.xmap(name -> create(registryName, name), ResourceKey::identifier);",
        "return Identifier.STREAM_CODEC.map(name -> create(registryName, name), ResourceKey::identifier);",
    ] {
        assert!(
            source.contains(sentinel),
            "ResourceKey.java codec sentinel missing: {sentinel}"
        );
    }

    let item_registry = super::ResourceKey::<String>::create_registry_key(
        Identifier::parse(registries::ITEM).unwrap(),
    );
    let stick = super::ResourceKey::<String>::create(
        &item_registry,
        Identifier::parse("minecraft:stick").unwrap(),
    );

    assert_eq!(stick.to_bound_json(), "\"minecraft:stick\"");
    assert_eq!(
        super::ResourceKey::<String>::from_bound_json(&item_registry, "\"minecraft:stick\"")
            .unwrap(),
        stick
    );

    let mut bytes = Vec::new();
    stick.write_bound_network(&mut bytes).unwrap();
    let mut input = crate::network::codec::cursor(bytes);
    assert_eq!(
        super::ResourceKey::<String>::read_bound_network(&item_registry, &mut input).unwrap(),
        stick
    );
    assert_eq!(input.position(), u64::try_from(input.get_ref().len()).unwrap());
}

#[test]
fn registry_network_ids_resolve_to_stable_resource_keys() {
    let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
    let stick = Identifier::parse("minecraft:stick").unwrap();
    let apple = Identifier::parse("minecraft:apple").unwrap();
    registry
        .register(stick.clone(), "stick".to_string(), Lifecycle::Stable)
        .unwrap();
    registry
        .register(apple.clone(), "apple".to_string(), Lifecycle::Stable)
        .unwrap();

    let mut bytes = Vec::new();
    registry.write_network_id(&mut bytes, &apple).unwrap();
    let mut input = crate::network::codec::cursor(bytes);
    let key = registry.read_network_key(&mut input).unwrap();

    assert_eq!(key.registry(), registry.registry_id());
    assert_eq!(key.location(), &apple);
    assert_eq!(registry.id_for_location(&stick), Some(0));
    assert_eq!(registry.id_for_location(&apple), Some(1));

    let mut invalid = Vec::new();
    crate::network::codec::write_registry_value_id(
        &mut invalid,
        crate::network::codec::RegistryValueId(99),
    )
    .unwrap();
    let err = registry
        .read_network_key(&mut crate::network::codec::cursor(invalid))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn registry_data_pack_overrides_preserve_existing_ids_and_append_new_entries() {
    let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
    registry
        .register(
            Identifier::parse("stick").unwrap(),
            "vanilla stick".to_string(),
            Lifecycle::Stable,
        )
        .unwrap();
    registry
        .register(
            Identifier::parse("apple").unwrap(),
            "vanilla apple".to_string(),
            Lifecycle::Stable,
        )
        .unwrap();

    registry
        .apply_data_pack_overrides(vec![
            (
                Identifier::parse("stick").unwrap(),
                "pack stick".to_string(),
                Lifecycle::Experimental,
            ),
            (
                Identifier::parse("custom").unwrap(),
                "pack custom".to_string(),
                Lifecycle::Stable,
            ),
        ])
        .unwrap();

    let stick = registry.get(&Identifier::parse("stick").unwrap()).unwrap();
    assert_eq!(stick.id(), 0);
    assert_eq!(stick.value(), "pack stick");
    assert_eq!(stick.lifecycle(), Lifecycle::Experimental);
    assert_eq!(
        registry
            .get(&Identifier::parse("custom").unwrap())
            .unwrap()
            .id(),
        2
    );
}

#[test]
fn builtin_registries_bootstrap_before_datapack_overrides() -> Result<(), String> {
    let builtins = super::BuiltInRegistries::bootstrap_26_1_2()?;
    assert_eq!(
        builtins.registry_ids(),
        vec![
            Identifier::parse(registries::BLOCK).unwrap(),
            Identifier::parse(registries::ITEM).unwrap(),
            Identifier::parse(registries::ENTITY_TYPE).unwrap(),
            Identifier::parse(registries::DIMENSION_TYPE).unwrap(),
            Identifier::parse(registries::BIOME).unwrap(),
        ]
    );
    assert_eq!(
        builtins
            .blocks
            .get(&Identifier::parse("minecraft:air").unwrap())
            .unwrap()
            .id(),
        0
    );
    assert_eq!(
        builtins
            .dimension_types
            .get(&Identifier::parse("minecraft:the_nether").unwrap())
            .unwrap()
            .value(),
        "minecraft:the_nether"
    );
    assert!(builtins.blocks.is_frozen());
    assert!(builtins.items.is_frozen());
    assert!(builtins.entity_types.is_frozen());
    assert!(builtins.dimension_types.is_frozen());
    assert!(builtins.biomes.is_frozen());
    Ok(())
}

#[test]
fn builtin_registry_descriptors_cover_java_builtins_or_are_documented() {
    let java_registry_ids = parse_java_registry_ids();
    let built_in_registry_ids = parse_builtin_registry_ids();
    let manifest_registry_ids: std::collections::BTreeSet<String> =
        builtin_registry_manifest_26_1_2()
            .into_iter()
            .map(|entry| entry.registry.to_string())
            .collect();
    let intentional: std::collections::BTreeMap<&str, &str> =
        INTENTIONALLY_OMITTED_REGISTRIES.iter().copied().collect();

    assert!(
        built_in_registry_ids.is_subset(&java_registry_ids),
        "BuiltInRegistries.java references are not in sync with Registries.java",
    );
    assert!(
        built_in_registry_ids == manifest_registry_ids,
        "BuiltInRegistries.java references are not aligned with Rust manifest",
    );

    for registry_id in java_registry_ids.iter() {
        if built_in_registry_ids.contains(registry_id)
            || manifest_registry_ids.contains(registry_id)
        {
            continue;
        }
        if let Some(reason) = intentional.get(registry_id.as_str()) {
            assert!(
                !reason.trim().is_empty(),
                "omission reason missing for intentionally omitted {registry_id}"
            );
        } else {
            panic!(
                "missing built-in registry descriptor for {registry_id} with no omission reason"
            );
        }
    }

    for registry_id in manifest_registry_ids.iter() {
        assert!(
            java_registry_ids.contains(registry_id),
            "manifest includes unknown registry {registry_id} not present in Registries.java"
        );
    }
}

#[test]
fn builtin_registry_manifest_matches_26_1_2_builtin_registration_order() {
    let manifest = builtin_registry_manifest_26_1_2();
    assert_eq!(manifest.len(), 95);
    assert_eq!(manifest[0].java_field, "GAME_EVENT");
    assert_eq!(manifest[0].registry, "minecraft:game_event");
    assert_eq!(manifest[0].default_key, Some("step"));
    assert_eq!(manifest[4].java_field, "BLOCK");
    assert_eq!(manifest[4].registry, "minecraft:block");
    assert_eq!(manifest[4].default_key, Some("air"));
    assert!(manifest[4].intrusive_holders);
    assert_eq!(manifest[6].java_field, "ENTITY_TYPE");
    assert_eq!(manifest[7].java_field, "ITEM");
    assert_eq!(manifest[56].java_field, "BLOCK_TYPE");
    assert_eq!(manifest[81].registry, "minecraft:outgoing_rpc_methods");
    assert_eq!(manifest.last().unwrap().java_field, "TEST_FUNCTION");

    let mut seen = std::collections::BTreeSet::new();
    for descriptor in &manifest {
        assert!(
            seen.insert(descriptor.registry),
            "duplicate registry {}",
            descriptor.registry
        );
        Identifier::parse(descriptor.registry).unwrap();
    }
}

#[test]
fn dynamic_registry_access_applies_datapack_entries_over_builtins() -> Result<(), String> {
    let builtins = super::BuiltInRegistries::bootstrap_26_1_2()?;
    let mut dynamic = super::DynamicRegistryAccess::from_builtins(&builtins);
    dynamic
        .apply_data_pack_entries(vec![
            super::DataPackRegistryEntry {
                registry: Identifier::parse(registries::BIOME).unwrap(),
                location: Identifier::parse("minecraft:plains").unwrap(),
                value: "pack plains".to_string(),
                lifecycle: Lifecycle::Experimental,
            },
            super::DataPackRegistryEntry {
                registry: Identifier::parse("minecraft:chat_type").unwrap(),
                location: Identifier::parse("minecraft:chat").unwrap(),
                value: "chat codec".to_string(),
                lifecycle: Lifecycle::Stable,
            },
        ])
        .unwrap();

    let biomes = dynamic
        .registry(&Identifier::parse(registries::BIOME).unwrap())
        .unwrap();
    assert_eq!(
        biomes
            .get(&Identifier::parse("minecraft:plains").unwrap())
            .unwrap()
            .value(),
        "pack plains"
    );
    assert_eq!(
        dynamic
            .registry(&Identifier::parse("minecraft:chat_type").unwrap())
            .unwrap()
            .get(&Identifier::parse("minecraft:chat").unwrap())
            .unwrap()
            .value(),
        "chat codec"
    );
    dynamic.freeze_all();
    assert!(dynamic
        .registry(&Identifier::parse(registries::BIOME).unwrap())
        .unwrap()
        .is_frozen());
    Ok(())
}

#[test]
fn dynamic_registry_access_preserves_builtin_then_datapack_registry_order() -> Result<(), String> {
    let builtins = super::BuiltInRegistries::bootstrap_26_1_2()?;
    let mut dynamic = super::DynamicRegistryAccess::from_builtins(&builtins);
    dynamic
        .apply_data_pack_entries(vec![
            super::DataPackRegistryEntry {
                registry: Identifier::parse("minecraft:chat_type").unwrap(),
                location: Identifier::parse("minecraft:chat").unwrap(),
                value: "chat codec".to_string(),
                lifecycle: Lifecycle::Stable,
            },
            super::DataPackRegistryEntry {
                registry: Identifier::parse(registries::BIOME).unwrap(),
                location: Identifier::parse("minecraft:forest").unwrap(),
                value: "pack forest".to_string(),
                lifecycle: Lifecycle::Stable,
            },
            super::DataPackRegistryEntry {
                registry: Identifier::parse("minecraft:damage_type").unwrap(),
                location: Identifier::parse("minecraft:generic").unwrap(),
                value: "damage codec".to_string(),
                lifecycle: Lifecycle::Stable,
            },
        ])
        .unwrap();

    assert_eq!(
        dynamic.registry_ids(),
        vec![
            Identifier::parse(registries::BLOCK).unwrap(),
            Identifier::parse(registries::ITEM).unwrap(),
            Identifier::parse(registries::ENTITY_TYPE).unwrap(),
            Identifier::parse(registries::DIMENSION_TYPE).unwrap(),
            Identifier::parse(registries::BIOME).unwrap(),
            Identifier::parse("minecraft:chat_type").unwrap(),
            Identifier::parse("minecraft:damage_type").unwrap(),
        ]
    );
    Ok(())
}

#[test]
fn tag_loading_handles_replace_optional_entries_and_errors() {
    let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
    registry
        .register(
            Identifier::parse("stick").unwrap(),
            "stick".to_string(),
            Lifecycle::Stable,
        )
        .unwrap();
    registry
        .register(
            Identifier::parse("apple").unwrap(),
            "apple".to_string(),
            Lifecycle::Stable,
        )
        .unwrap();
    let tag = Identifier::parse("test/items").unwrap();

    let loaded = super::LoadedTags::load(
        &registry,
        vec![
            super::TagFile {
                registry: Identifier::parse(registries::ITEM).unwrap(),
                tag: tag.clone(),
                replace: false,
                entries: vec![super::TagEntry {
                    id: Identifier::parse("stick").unwrap(),
                    required: true,
                }],
            },
            super::TagFile {
                registry: Identifier::parse(registries::ITEM).unwrap(),
                tag: tag.clone(),
                replace: true,
                entries: vec![
                    super::TagEntry {
                        id: Identifier::parse("apple").unwrap(),
                        required: true,
                    },
                    super::TagEntry {
                        id: Identifier::parse("missing_optional").unwrap(),
                        required: false,
                    },
                ],
            },
        ],
    )
    .unwrap();

    assert_eq!(
        loaded
            .values(&Identifier::parse(registries::ITEM).unwrap(), &tag)
            .unwrap(),
        &[Identifier::parse("apple").unwrap()]
    );

    let errors = super::LoadedTags::load(
        &registry,
        vec![super::TagFile {
            registry: Identifier::parse(registries::ITEM).unwrap(),
            tag,
            replace: false,
            entries: vec![super::TagEntry {
                id: Identifier::parse("missing_required").unwrap(),
                required: true,
            }],
        }],
    )
    .unwrap_err();
    assert_eq!(
        errors,
        vec!["missing required tag entry minecraft:missing_required"]
    );
}

#[test]
fn reloadable_server_registries_order_datapack_registries_before_tags_and_freeze(
) -> Result<(), String> {
    let builtins = super::BuiltInRegistries::bootstrap_26_1_2()?;
    let mut reloadable = super::ReloadableServerRegistries::new(builtins);
    let biome_registry = Identifier::parse(registries::BIOME).unwrap();
    let custom_biome = Identifier::parse("example:glade").unwrap();
    let tag = Identifier::parse("minecraft:is_overworld").unwrap();

    let reload = reloadable
        .reload(super::ServerResourceReloadRequest {
            registry_entries: vec![super::DataPackRegistryEntry {
                registry: biome_registry.clone(),
                location: custom_biome.clone(),
                value: "glade codec".to_string(),
                lifecycle: Lifecycle::Experimental,
            }],
            tag_files: vec![super::TagFile {
                registry: biome_registry.clone(),
                tag: tag.clone(),
                replace: true,
                entries: vec![super::TagEntry {
                    id: custom_biome.clone(),
                    required: true,
                }],
            }],
        })
        .unwrap();

    assert_eq!(
        reload.stages(),
        &[
            super::ServerReloadStage::BuiltInRegistries,
            super::ServerReloadStage::DataPackRegistries,
            super::ServerReloadStage::Tags,
            super::ServerReloadStage::FrozenRegistries,
            super::ServerReloadStage::Recipes,
            super::ServerReloadStage::LootTables,
            super::ServerReloadStage::Advancements,
            super::ServerReloadStage::Functions,
        ]
    );
    assert!(reload
        .registries()
        .registry(&biome_registry)
        .unwrap()
        .is_frozen());
    assert_eq!(
        reload
            .registries()
            .registry(&biome_registry)
            .unwrap()
            .get(&custom_biome)
            .unwrap()
            .value(),
        "glade codec"
    );
    assert_eq!(
        reload
            .tags(&biome_registry)
            .unwrap()
            .values(&biome_registry, &tag),
        Some([custom_biome].as_slice())
    );
    Ok(())
}

#[test]
fn reloadable_server_registries_keep_last_successful_state_on_tag_failure() -> Result<(), String> {
    let builtins = super::BuiltInRegistries::bootstrap_26_1_2()?;
    let mut reloadable = super::ReloadableServerRegistries::new(builtins);
    let item_registry = Identifier::parse(registries::ITEM).unwrap();
    let stick = Identifier::parse("minecraft:stick").unwrap();
    let tag = Identifier::parse("minecraft:test_items").unwrap();

    reloadable
        .reload(super::ServerResourceReloadRequest {
            registry_entries: Vec::new(),
            tag_files: vec![super::TagFile {
                registry: item_registry.clone(),
                tag: tag.clone(),
                replace: true,
                entries: vec![super::TagEntry {
                    id: stick.clone(),
                    required: true,
                }],
            }],
        })
        .unwrap();

    let before = reloadable
        .last_successful()
        .registry(&item_registry)
        .unwrap()
        .get(&stick)
        .unwrap()
        .id();
    let err = reloadable
        .reload(super::ServerResourceReloadRequest {
            registry_entries: Vec::new(),
            tag_files: vec![super::TagFile {
                registry: item_registry.clone(),
                tag,
                replace: true,
                entries: vec![super::TagEntry {
                    id: Identifier::parse("minecraft:missing_required").unwrap(),
                    required: true,
                }],
            }],
        })
        .unwrap_err();

    assert_eq!(err, "missing required tag entry minecraft:missing_required");
    assert_eq!(
        reloadable
            .last_successful()
            .registry(&item_registry)
            .unwrap()
            .get(&stick)
            .unwrap()
            .id(),
        before
    );
    Ok(())
}

#[test]
fn represents_holders_and_named_tag_sets() {
    let registry = Identifier::parse(registries::ITEM).unwrap();
    let tag = TagKey::<String>::new(registry.clone(), Identifier::parse("logs").unwrap());
    let set = HolderSet::<String>::Named(tag);
    assert!(matches!(set, HolderSet::Named(_)));

    let key = super::ResourceKey::new(registry, Identifier::parse("stick").unwrap());
    let holder = Holder::<String>::Reference(key);
    assert!(matches!(holder, Holder::Reference(_)));
}

#[test]
fn tag_key_matches_java_codec_cast_and_display_contract() -> Result<(), String> {
    assert_eq!(TAG_KEY_JAVA.lines().count(), 53);
    for fragment in [
        "public record TagKey<T>(ResourceKey<? extends Registry<T>> registry, Identifier location)",
        "Identifier.CODEC.xmap(name -> create(registryName, name), TagKey::location)",
        "name.startsWith(\"#\")",
        "e -> \"#\" + e.location",
        "public boolean isFor(final ResourceKey<? extends Registry<?>> registry)",
        "public <E> Optional<TagKey<E>> cast",
        "VALUES.intern(new TagKey<>(registry, location))",
    ] {
        assert!(TAG_KEY_JAVA.contains(fragment), "missing TagKey source fragment: {fragment}");
    }

    let registry = Identifier::parse("minecraft:item")?;
    let tag = TagKey::<String>::codec(registry.clone(), "minecraft:logs")?;
    assert_eq!(tag.registry(), &registry);
    assert_eq!(tag.location().to_string(), "minecraft:logs");
    assert_eq!(tag.hashed_string(), "#minecraft:logs");
    assert_eq!(tag.to_string(), "TagKey[minecraft:item / minecraft:logs]");
    assert!(tag.is_for(&registry));
    assert!(tag.cast::<String>(&registry).is_some());
    assert!(!tag.is_for(&Identifier::parse("minecraft:block")?));
    assert!(tag.cast::<u8>(&Identifier::parse("minecraft:block")?).is_none());

    let hashed = TagKey::<String>::hashed_codec(registry.clone(), "#minecraft:logs")?;
    assert_eq!(hashed, tag);
    assert_eq!(
        TagKey::<String>::hashed_codec(registry, "minecraft:logs").map(|_| ()),
        Err("Not a tag id".to_string())
    );
    Ok(())
}

#[test]
fn feature_flags_match_26_1_2_defaults() -> Result<(), String> {
    let registry = FeatureFlagRegistry::main_26_1_2()?;
    let defaults = feature_flags::default_flags_26_1_2();
    assert!(defaults.contains(feature_flags::VANILLA));
    assert!(!defaults.contains(feature_flags::TRADE_REBALANCE));
    assert_eq!(
        registry.to_names(defaults),
        vec![Identifier::parse("minecraft:vanilla").unwrap()]
    );

    let experimental = defaults.join(super::FeatureFlagSet::of(&[
        feature_flags::MINECART_IMPROVEMENTS,
    ]));
    assert!(defaults.is_subset_of(experimental));
    assert!(!experimental.is_subset_of(defaults));
    Ok(())
}
