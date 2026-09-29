//! `RegistryDataLoader`/`TagLoader` behaviour on synthetic data packs.

use super::{builtin, vanilla_with, TestPackDir};
use crate::network::configuration::KnownPack;
use crate::registry::Identifier;
use crate::registry_pipeline::load_registries;
use crate::registry_pipeline::loader::RegistryLoadError;
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};
use crate::registry_pipeline::store::Registries;
use crate::registry_pipeline::tags::load_tags_for_registry;

fn id(text: &str) -> Identifier {
    Identifier::parse(text).unwrap_or_else(|err| panic!("{text}: {err}"))
}

fn load_with(pack: &TestPackDir) -> Result<Registries, String> {
    load_registries(&vanilla_with(pack.pack("test", None)), builtin())
}

fn expect_error(pack: &TestPackDir) -> String {
    match load_with(pack) {
        Ok(_) => panic!("loading unexpectedly succeeded"),
        Err(message) => message,
    }
}

/// Loads only tags, with an element lookup that knows `known` and (like the
/// registration getter) accepts every required element when `register_required`.
fn tags_of(
    manager: &ResourceManager,
    path: &str,
    known: &[&str],
    register_required: bool,
) -> (
    std::collections::BTreeMap<Identifier, Vec<Identifier>>,
    Vec<String>,
) {
    let known: Vec<Identifier> = known.iter().map(|text| id(text)).collect();
    let mut errors = Vec::new();
    let mut lookup = |element: &Identifier, required: bool| {
        (register_required && required) || known.contains(element)
    };
    let tags = load_tags_for_registry(manager, path, &mut lookup, &mut errors);
    (tags, errors)
}

#[test]
fn vanilla_registries_load_and_freeze() {
    let registries = super::registries();
    // The 28 synchronised registries plus the `configured_carver`, `placed_feature`
    // and `structure` registries loaded for reference resolution, and the thirteen
    // server-side worldgen/data registries (`villager_trade`, `trade_set`,
    // `structure_set`, `processor_list`,
    // `template_pool`, `noise_settings`, `noise`, `density_function`,
    // `multi_noise_biome_source_parameter_list`, `world_preset`,
    // `flat_level_generator_preset`, `enchantment_provider`, `trial_spawner`).
    assert_eq!(registries.worldgen_layer().len(), 44);
    assert!(registries.static_layer().len() > 50);
    // Every synchronised element records the vanilla known pack it came from.
    for registry in registries.worldgen_layer() {
        for element in registry.elements() {
            assert_eq!(
                element.info.known_pack,
                Some(KnownPack::vanilla("core")),
                "{}/{}",
                registry.key(),
                element.key
            );
        }
    }
}

#[test]
fn elements_are_registered_in_sorted_resource_file_order() {
    // Java sorts by resource file id (path then namespace), so `a-b.json` sorts
    // before `a.json` even though the element id `a` sorts before `a-b`.
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/banner_pattern/zzz_first.json",
        r##"{"asset_id":"minecraft:x","translation_key":"k"}"##,
    );
    pack.write(
        "minecraft/banner_pattern/zzz.json",
        r##"{"asset_id":"minecraft:x","translation_key":"k"}"##,
    );
    let registries = load_with(&pack).expect("load");
    let banner = registries
        .lookup(&id("minecraft:banner_pattern"))
        .expect("banner_pattern");
    let zzz = banner.id_of(&id("minecraft:zzz")).expect("zzz");
    let zzz_first = banner.id_of(&id("minecraft:zzz_first")).expect("zzz_first");
    // '.' (0x2e) sorts before '_' (0x5f): zzz.json < zzz_first.json.
    assert_eq!(zzz + 1, zzz_first);
}

#[test]
fn higher_priority_packs_override_elements_and_lose_known_pack_status() {
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/banner_pattern/base.json",
        r##"{"asset_id":"minecraft:base","translation_key":"custom.base"}"##,
    );
    let registries = load_with(&pack).expect("load");
    let banner = registries
        .lookup(&id("minecraft:banner_pattern"))
        .expect("banner_pattern");
    let element = &banner.elements()[banner.id_of(&id("minecraft:base")).expect("base")];
    assert_eq!(element.json["translation_key"], "custom.base");
    // A pack without a known-pack identity is registered as experimental and must be
    // sent to clients in full.
    assert_eq!(element.info.known_pack, None);
    assert_eq!(
        element.info.lifecycle,
        crate::registry::Lifecycle::Experimental
    );
}

#[test]
fn parse_errors_are_reported_per_element_like_java() {
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/banner_pattern/broken.json",
        r##"{"asset_id":"minecraft:x"}"##,
    );
    pack.write(
        "minecraft/banner_pattern/bad_id.json",
        r##"{"asset_id":"Not Valid","translation_key":"k"}"##,
    );
    let message = expect_error(&pack);
    assert!(
        message.contains("Failed to load registries due to errors"),
        "{message}"
    );
    assert!(
        message.contains("minecraft:banner_pattern/minecraft:broken: Failed to parse minecraft:broken from pack test"),
        "{message}"
    );
    assert!(
        message.contains("minecraft:banner_pattern/minecraft:bad_id"),
        "{message}"
    );
    // The full log carries the cause of each failure.
    assert!(
        message.contains("> Errors in registry minecraft:banner_pattern:"),
        "{message}"
    );
    assert!(
        message.contains("No key translation_key in MapLike["),
        "{message}"
    );
}

#[test]
fn unbound_element_references_fail_when_the_registry_freezes() {
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/timeline/broken.json",
        r##"{"clock":"minecraft:not_a_clock","tracks":{}}"##,
    );
    let message = expect_error(&pack);
    assert!(
        message.contains(
            "Unbound values in registry ResourceKey[minecraft:root / minecraft:world_clock]: [minecraft:not_a_clock]"
        ),
        "{message}"
    );
}

#[test]
fn unbound_tag_references_fail_when_the_registry_freezes() {
    let pack = TestPackDir::new();
    let overworld = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla-data/data/minecraft/dimension_type/overworld.json"),
    )
    .expect("overworld.json");
    pack.write(
        "minecraft/dimension_type/overworld.json",
        &overworld.replace(
            "#minecraft:in_overworld",
            "#minecraft:undefined_timeline_tag",
        ),
    );
    let message = expect_error(&pack);
    assert!(
        message.contains(
            "Unbound tags in registry ResourceKey[minecraft:root / minecraft:timeline]: [minecraft:undefined_timeline_tag]"
        ),
        "{message}"
    );
}

#[test]
fn timeline_time_markers_may_only_be_defined_once_per_clock() {
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/timeline/duplicate.json",
        r##"{"clock":"minecraft:overworld","time_markers":{"minecraft:noon":6000}}"##,
    );
    let message = expect_error(&pack);
    assert!(
        message.contains("was defined multiple times in minecraft:overworld"),
        "{message}"
    );
}

#[test]
fn non_empty_registries_must_keep_at_least_one_element() {
    // RegistryValidator.nonEmpty on a registry that is empty: exercised through the
    // validator directly since vanilla always provides elements.
    let manager = ResourceManager::new(Vec::new());
    let tasks: Vec<_> = crate::registry_pipeline::registry_data::worldgen_load_tasks()
        .into_iter()
        .filter(|task| task.data.key == "minecraft:frog_variant")
        .collect();
    let error: RegistryLoadError =
        crate::registry_pipeline::loader::load(&manager, builtin(), &tasks).unwrap_err();
    let (key, failure) = error.errors.iter().next().expect("error");
    assert_eq!(key.1, id("minecraft:frog_variant"));
    assert_eq!(
        failure.message,
        "Registry must be non-empty: minecraft:frog_variant"
    );
}

#[test]
fn tags_merge_across_packs_and_replace_clears_earlier_entries() {
    let low = TestPackDir::new();
    low.write(
        "minecraft/tags/thing/a.json",
        r##"{"values":["minecraft:one","minecraft:two"]}"##,
    );
    low.write(
        "minecraft/tags/thing/b.json",
        r##"{"values":["minecraft:one"]}"##,
    );
    let mid = TestPackDir::new();
    mid.write(
        "minecraft/tags/thing/a.json",
        r##"{"values":["minecraft:three"]}"##,
    );
    mid.write(
        "minecraft/tags/thing/b.json",
        r##"{"replace":true,"values":["minecraft:two"]}"##,
    );
    let manager = ResourceManager::new(vec![
        Box::new(low.pack("low", None)),
        Box::new(mid.pack("mid", None)),
    ]);
    let (tags, errors) = tags_of(
        &manager,
        "thing",
        &["minecraft:one", "minecraft:two", "minecraft:three"],
        false,
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        tags[&id("minecraft:a")],
        vec![
            id("minecraft:one"),
            id("minecraft:two"),
            id("minecraft:three")
        ]
    );
    assert_eq!(tags[&id("minecraft:b")], vec![id("minecraft:two")]);
}

#[test]
fn nested_tags_optional_entries_and_missing_references_follow_tag_loader() {
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/tags/thing/inner.json",
        r##"{"values":["minecraft:one"]}"##,
    );
    pack.write(
        "minecraft/tags/thing/outer.json",
        r##"{"values":["#minecraft:inner","minecraft:two",{"id":"minecraft:gone","required":false},{"id":"#minecraft:absent","required":false}]}"##,
    );
    pack.write(
        "minecraft/tags/thing/broken.json",
        r##"{"values":["minecraft:gone"]}"##,
    );
    pack.write(
        "minecraft/tags/thing/needs_broken.json",
        r##"{"values":["#minecraft:broken"]}"##,
    );
    let manager = ResourceManager::new(vec![Box::new(pack.pack("p", None))]);
    let (tags, errors) = tags_of(
        &manager,
        "thing",
        &["minecraft:one", "minecraft:two"],
        false,
    );

    assert_eq!(
        tags[&id("minecraft:outer")],
        vec![id("minecraft:one"), id("minecraft:two")]
    );
    // Tags with unresolved required references are omitted and logged, and so are
    // the tags that depend on them.
    assert!(!tags.contains_key(&id("minecraft:broken")));
    assert!(!tags.contains_key(&id("minecraft:needs_broken")));
    assert!(errors.iter().any(|line| line
        .contains("Couldn't load tag minecraft:broken as it is missing following references: minecraft:gone (from p)")));
    assert!(errors.iter().any(|line| line
        .contains("Couldn't load tag minecraft:needs_broken as it is missing following references: #minecraft:broken (from p)")));
}

#[test]
fn required_elements_resolve_through_the_registration_getter() {
    // While a registry is being loaded, required tag entries always resolve (the
    // registration getter creates a holder); freezing reports the unbound ones.
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/tags/thing/t.json",
        r##"{"values":["minecraft:unknown_yet"]}"##,
    );
    let manager = ResourceManager::new(vec![Box::new(pack.pack("p", None))]);
    let (tags, errors) = tags_of(&manager, "thing", &[], true);
    assert!(errors.is_empty());
    assert_eq!(tags[&id("minecraft:t")], vec![id("minecraft:unknown_yet")]);
}

#[test]
fn malformed_tag_files_are_logged_and_skipped() {
    let pack = TestPackDir::new();
    pack.write("minecraft/tags/thing/bad.json", r##"{"values": 5}"##);
    pack.write("minecraft/tags/thing/worse.json", "not json");
    pack.write("minecraft/tags/thing/good.json", r##"{"values":[]}"##);
    let manager = ResourceManager::new(vec![Box::new(pack.pack("p", None))]);
    let (tags, errors) = tags_of(&manager, "thing", &[], false);
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors
        .iter()
        .all(|line| line.starts_with("Couldn't read tag list")));
    assert_eq!(tags.len(), 1);
    assert!(tags[&id("minecraft:good")].is_empty());
}

#[test]
fn cyclic_tag_references_fail_both_tags_without_hanging() {
    // DependencySorter drops the edge that would close the cycle, so the tag that
    // is visited first cannot see its partner and fails; the other then depends on
    // a tag that does not exist. Java logs both as missing references.
    let pack = TestPackDir::new();
    pack.write(
        "minecraft/tags/thing/a.json",
        r##"{"values":["minecraft:one","#minecraft:b"]}"##,
    );
    pack.write(
        "minecraft/tags/thing/b.json",
        r##"{"values":["minecraft:two","#minecraft:a"]}"##,
    );
    let manager = ResourceManager::new(vec![Box::new(pack.pack("p", None))]);
    let (tags, errors) = tags_of(
        &manager,
        "thing",
        &["minecraft:one", "minecraft:two"],
        false,
    );
    assert!(tags.is_empty(), "{tags:?}");
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors
        .iter()
        .all(|line| line.contains("missing following references")));
}

#[test]
fn file_to_id_converter_round_trips_registry_paths() {
    let converter = FileToIdConverter::json("worldgen/biome");
    let file = id("minecraft:worldgen/biome/deep/ocean.json");
    assert_eq!(converter.file_to_id(&file), Ok(id("minecraft:deep/ocean")));
    assert!(converter.file_to_id(&id("minecraft:other/x.json")).is_err());
}

#[test]
fn registries_reject_duplicates_and_writes_after_freezing() {
    use crate::registry_pipeline::store::{MappedRegistry, RegistrationInfo};
    let mut registry = MappedRegistry::new(id("minecraft:test"));
    let info = RegistrationInfo::built_in();
    assert_eq!(
        registry.register(id("minecraft:a"), serde_json::Value::Null, info.clone()),
        Ok(0)
    );
    assert_eq!(
        registry.register(id("minecraft:b"), serde_json::Value::Null, info.clone()),
        Ok(1)
    );
    assert_eq!(
        registry.register(id("minecraft:a"), serde_json::Value::Null, info.clone()),
        Err("Adding duplicate key 'minecraft:a' to registry".to_string())
    );
    registry.freeze();
    assert_eq!(
        registry.register(id("minecraft:c"), serde_json::Value::Null, info),
        Err("Registry is already frozen (trying to add key minecraft:c)".to_string())
    );
    assert_eq!(registry.id_of(&id("minecraft:b")), Some(1));
    assert_eq!(registry.len(), 2);
}

#[test]
fn resource_manager_offers_the_vanilla_core_known_pack() {
    assert_eq!(
        ResourceManager::vanilla().known_packs(),
        vec![KnownPack::new("minecraft", "core", "26.1.2")]
    );
    // Packs without a known-pack identity are not offered to clients.
    let pack = TestPackDir::new();
    let manager = vanilla_with(pack.pack("custom", None));
    assert_eq!(manager.known_packs().len(), 1);
}
