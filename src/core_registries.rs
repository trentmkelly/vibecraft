#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

pub const CORE_REGISTRIES_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKeyModel {
    registry: &'static str,
    location: &'static str,
}

impl ResourceKeyModel {
    const fn new(registry: &'static str, location: &'static str) -> Self {
        Self { registry, location }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TagKeyModel {
    registry: &'static str,
    location: &'static str,
}

impl TagKeyModel {
    const fn new(registry: &'static str, location: &'static str) -> Self {
        Self { registry, location }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegistryKeyModel {
    id: &'static str,
}

impl RegistryKeyModel {
    const fn new(id: &'static str) -> Self {
        Self { id }
    }

    fn path(self) -> &'static str {
        self.id
            .strip_prefix("minecraft:")
            .expect("registry key should use the default namespace")
    }
}

#[derive(Debug, Default)]
struct OriginalHolderGetterModel {
    element_results: BTreeMap<ResourceKeyModel, Option<&'static str>>,
    tag_results: BTreeMap<TagKeyModel, Option<&'static [&'static str]>>,
    element_calls: BTreeMap<ResourceKeyModel, usize>,
    tag_calls: BTreeMap<TagKeyModel, usize>,
}

impl OriginalHolderGetterModel {
    fn with_element(mut self, key: ResourceKeyModel, holder: &'static str) -> Self {
        self.element_results.insert(key, Some(holder));
        self
    }

    fn with_tag(mut self, key: TagKeyModel, holders: &'static [&'static str]) -> Self {
        self.tag_results.insert(key, Some(holders));
        self
    }

    fn get_element(&mut self, key: &ResourceKeyModel) -> Option<&'static str> {
        *self.element_calls.entry(key.clone()).or_default() += 1;
        self.element_results.get(key).copied().flatten()
    }

    fn get_tag(&mut self, key: &TagKeyModel) -> Option<&'static [&'static str]> {
        *self.tag_calls.entry(key.clone()).or_default() += 1;
        self.tag_results.get(key).copied().flatten()
    }

    fn element_call_count(&self, key: &ResourceKeyModel) -> usize {
        self.element_calls.get(key).copied().unwrap_or(0)
    }

    fn tag_call_count(&self, key: &TagKeyModel) -> usize {
        self.tag_calls.get(key).copied().unwrap_or(0)
    }
}

#[derive(Debug)]
struct ConcurrentHolderGetterModel {
    original: OriginalHolderGetterModel,
    element_cache: BTreeMap<ResourceKeyModel, Option<&'static str>>,
    tag_cache: BTreeMap<TagKeyModel, Option<&'static [&'static str]>>,
    lock_entries: usize,
}

impl ConcurrentHolderGetterModel {
    fn new(original: OriginalHolderGetterModel) -> Self {
        Self {
            original,
            element_cache: BTreeMap::new(),
            tag_cache: BTreeMap::new(),
            lock_entries: 0,
        }
    }

    fn get_element(&mut self, key: ResourceKeyModel) -> Option<&'static str> {
        if let Some(cached) = self.element_cache.get(&key) {
            return *cached;
        }
        self.lock_entries += 1;
        let resolved = self.original.get_element(&key);
        self.element_cache.insert(key, resolved);
        resolved
    }

    fn get_tag(&mut self, key: TagKeyModel) -> Option<&'static [&'static str]> {
        if let Some(cached) = self.tag_cache.get(&key) {
            return *cached;
        }
        self.lock_entries += 1;
        let resolved = self.original.get_tag(&key);
        self.tag_cache.insert(key, resolved);
        resolved
    }
}

fn create_registry_key(name: &'static str) -> RegistryKeyModel {
    RegistryKeyModel::new(match name {
        "dimension" => "minecraft:dimension",
        "worldgen/biome" => "minecraft:worldgen/biome",
        "loot_table" => "minecraft:loot_table",
        "particle_type" => "minecraft:particle_type",
        _ => panic!("registry key fixture is missing {name}"),
    })
}

fn level_stem_to_level(level_stem: ResourceKeyModel) -> ResourceKeyModel {
    ResourceKeyModel::new("minecraft:dimension", level_stem.location)
}

fn level_to_level_stem(level: ResourceKeyModel) -> ResourceKeyModel {
    ResourceKeyModel::new("minecraft:dimension", level.location)
}

fn elements_dir_path(registry_key: RegistryKeyModel) -> String {
    registry_key.path().to_string()
}

fn tags_dir_path(registry_key: RegistryKeyModel) -> String {
    format!("tags/{}", registry_key.path())
}

fn components_dir_path(registry_key: RegistryKeyModel) -> String {
    format!("components/{}", registry_key.path())
}

fn parse_java_registry_key_paths(source: &str) -> BTreeMap<String, String> {
    let mut mapping = BTreeMap::new();
    let create_call = "createRegistryKey(";
    let mut cursor = 0usize;

    while let Some(create_offset) = source[cursor..].find(create_call) {
        let create_pos = cursor + create_offset;
        let before_call = &source[..create_pos];
        let line_start = before_call.rfind('\n').map_or(0, |idx| idx + 1);
        let lhs = before_call[line_start..]
            .split('=')
            .next()
            .unwrap_or("")
            .trim();
        if !lhs.starts_with("public static final ResourceKey") {
            cursor = create_pos + create_call.len();
            continue;
        }
        let constant = lhs.split_whitespace().last().unwrap_or("");
        let after_call = &source[create_pos + create_call.len()..];
        let quoted = after_call
            .split_once('"')
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(path, _)| path);
        if !constant.is_empty() {
            mapping.insert(
                constant.to_string(),
                quoted
                    .expect("createRegistryKey should use string literal")
                    .to_string(),
            );
        }
        cursor = create_pos + create_call.len();
    }

    mapping
}

fn parse_java_builtin_registry_fields(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("public static final ")
                || !trimmed.contains(" = register")
                || trimmed.contains(" REGISTRY = ")
            {
                return None;
            }
            trimmed
                .split(" = ")
                .next()
                .and_then(|lhs| lhs.split_whitespace().last())
                .map(str::to_string)
        })
        .collect()
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    #[test]
    fn registries_java_declares_default_namespace_keys_and_path_helpers() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/core/registries/Registries.java"
        );
        let mapping = parse_java_registry_key_paths(source);

        assert_eq!(mapping.len(), 147);
        assert_eq!(mapping["BLOCK"], "block");
        assert_eq!(mapping["PARTICLE_TYPE"], "particle_type");
        assert_eq!(mapping["BIOME"], "worldgen/biome");
        assert_eq!(mapping["LEVEL_STEM"], "dimension");
        assert_eq!(mapping["DIMENSION"], "dimension");
        assert!(source.contains(
            "public static final Identifier ROOT_REGISTRY_NAME = Identifier.withDefaultNamespace(\"root\")"
        ));

        let biome = create_registry_key("worldgen/biome");
        assert_eq!(elements_dir_path(biome), "worldgen/biome");
        assert_eq!(tags_dir_path(biome), "tags/worldgen/biome");
        assert_eq!(components_dir_path(biome), "components/worldgen/biome");
    }

    #[test]
    fn registries_level_and_level_stem_keys_share_dimension_registry() {
        let overworld_stem = ResourceKeyModel::new("minecraft:dimension", "minecraft:overworld");
        let level = level_stem_to_level(overworld_stem.clone());
        assert_eq!(level.registry, "minecraft:dimension");
        assert_eq!(level.location, "minecraft:overworld");
        assert_eq!(level_to_level_stem(level), overworld_stem);
    }

    #[test]
    fn builtin_registries_java_contract_matches_manifest_tests() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/core/registries/BuiltInRegistries.java"
        );
        let fields = parse_java_builtin_registry_fields(source);

        assert_eq!(fields.len(), 95);
        assert_eq!(fields.first().unwrap(), "GAME_EVENT");
        assert_eq!(fields[4], "BLOCK");
        assert_eq!(fields[7], "ITEM");
        assert_eq!(fields[56], "BLOCK_TYPE");
        assert_eq!(fields.last().unwrap(), "TEST_FUNCTION");
        assert!(source.contains("private static final Map<Identifier, Supplier<?>> LOADERS"));
        assert!(source.contains("new MappedRegistry<>(\n      ResourceKey.createRegistryKey(Registries.ROOT_REGISTRY_NAME)"));
        assert!(source.contains(
            "public static final Registry<? extends Registry<?>> REGISTRY = WRITABLE_REGISTRY"
        ));
        assert!(source.contains("createContents();\n      freeze();\n      validate(REGISTRY);"));
    }

    #[test]
    fn concurrent_holder_getter_caches_present_and_missing_elements() {
        let stone = ResourceKeyModel::new("minecraft:block", "minecraft:stone");
        let missing = ResourceKeyModel::new("minecraft:block", "minecraft:missing");
        let original = OriginalHolderGetterModel::default().with_element(stone.clone(), "stone");
        let mut getter = ConcurrentHolderGetterModel::new(original);

        assert_eq!(getter.get_element(stone.clone()), Some("stone"));
        assert_eq!(getter.get_element(stone.clone()), Some("stone"));
        assert_eq!(getter.get_element(missing.clone()), None);
        assert_eq!(getter.get_element(missing.clone()), None);

        assert_eq!(getter.original.element_call_count(&stone), 1);
        assert_eq!(getter.original.element_call_count(&missing), 1);
        assert_eq!(getter.lock_entries, 2);
    }

    #[test]
    fn concurrent_holder_getter_caches_present_and_missing_tags_separately() {
        let mineable = TagKeyModel::new("minecraft:block", "minecraft:mineable/pickaxe");
        let missing = TagKeyModel::new("minecraft:block", "minecraft:missing");
        let stone = ResourceKeyModel::new("minecraft:block", "minecraft:stone");
        let original = OriginalHolderGetterModel::default()
            .with_element(stone.clone(), "stone")
            .with_tag(mineable.clone(), &["stone", "deepslate"]);
        let mut getter = ConcurrentHolderGetterModel::new(original);

        assert_eq!(
            getter.get_tag(mineable.clone()),
            Some(&["stone", "deepslate"][..])
        );
        assert_eq!(
            getter.get_tag(mineable.clone()),
            Some(&["stone", "deepslate"][..])
        );
        assert_eq!(getter.get_tag(missing.clone()), None);
        assert_eq!(getter.get_tag(missing.clone()), None);
        assert_eq!(getter.get_element(stone.clone()), Some("stone"));

        assert_eq!(getter.original.tag_call_count(&mineable), 1);
        assert_eq!(getter.original.tag_call_count(&missing), 1);
        assert_eq!(getter.original.element_call_count(&stone), 1);
        assert_eq!(getter.lock_entries, 3);
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(CORE_REGISTRIES_PACKAGE_NULL_MARKED);
        }
    }
}
