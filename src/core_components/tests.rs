use super::*;

fn requires_rarity(map: &ComponentMapModel) -> Result<(), String> {
    map.has("rarity")
        .then_some(())
        .ok_or_else(|| "rarity required".to_string())
}

#[test]
fn data_components_registry_matches_java_order_and_flags() {
    assert_eq!(COMPONENTS.len(), 110);
    assert_eq!(
        COMPONENTS.first().unwrap().id,
        DATA_COMPONENTS_BOOTSTRAP_RETURN
    );
    assert_eq!(COMPONENTS.last().unwrap().id, "shulker/color");
    assert_eq!(ENCODER_CACHE_SIZE, 512);
    assert!(core_component_package_is_null_marked());

    let transient: Vec<_> = COMPONENTS
        .iter()
        .filter(|component| component.is_transient())
        .map(|component| component.id)
        .collect();
    assert_eq!(
        transient,
        vec![
            "creative_slot_lock",
            "additional_trade_cost",
            "map_post_processing"
        ]
    );
    assert!(component_type("damage").unwrap().ignore_swap_animation);
    assert!(!component_type("custom_data").unwrap().network);
    assert!(component_type("custom_name").unwrap().cache_encoding);
    assert!(component_type("map_decorations").unwrap().persistent);
    assert!(!component_type("map_decorations").unwrap().network);
    assert!(component_type("villager/variant").unwrap().network);
}

#[test]
fn component_type_builder_and_typed_component_errors_match_java() {
    assert_eq!(
        component_type("custom_name").unwrap().codec_or_throw(),
        Ok("codec")
    );
    assert_eq!(
        component_type("creative_slot_lock")
            .unwrap()
            .codec_or_throw()
            .unwrap_err(),
        "creative_slot_lock is not a persistent component"
    );
    assert_eq!(
        TypedComponent {
            ty: "custom_name",
            value: "Name"
        }
        .to_java_string(),
        "custom_name=>Name"
    );
    assert_eq!(
        TypedComponent {
            ty: "creative_slot_lock",
            value: "unit"
        }
        .encode_value()
        .unwrap_err(),
        "Component of type creative_slot_lock is not encodable"
    );
}

#[test]
fn component_getter_holder_map_builder_filter_and_composite_match_java() {
    let base = ComponentMapModel::builder()
        .set("rarity", Some("common"))
        .set("repair_cost", Some("2"))
        .set("repair_cost", None)
        .add_validator(requires_rarity)
        .build()
        .unwrap();
    assert_eq!(base.get("rarity"), Some("common"));
    assert_eq!(base.get_or_default("repair_cost", "0"), "0");
    assert_eq!(
        base.get_typed("rarity"),
        Some(TypedComponent {
            ty: "rarity",
            value: "common"
        })
    );
    assert_eq!(base.key_set(), BTreeSet::from(["rarity"]));

    let overrides = ComponentMapModel::builder()
        .set("custom_name", Some("Sword"))
        .build()
        .unwrap();
    let composite = ComponentMapModel::composite(&base, &overrides);
    assert_eq!(composite.get("rarity"), Some("common"));
    assert_eq!(composite.get("custom_name"), Some("Sword"));
    assert_eq!(
        composite.filter(|ty| ty.contains("name")).key_set(),
        BTreeSet::from(["custom_name"])
    );
    assert_eq!(
        ComponentMapModel::builder()
            .add_all(&base)
            .add_validator(requires_rarity)
            .build()
            .unwrap(),
        base
    );
    assert_eq!(
        ComponentMapModel::builder()
            .set("custom_name", Some("Sword"))
            .add_validator(requires_rarity)
            .build()
            .unwrap_err(),
        "rarity required"
    );
}

#[test]
fn component_map_codecs_drop_transient_components() {
    let map = ComponentMapModel::builder()
        .set("custom_name", Some("Name"))
        .set("creative_slot_lock", Some("unit"))
        .set("additional_trade_cost", Some("4"))
        .build()
        .unwrap();
    assert_eq!(
        map.persistent_value_map(),
        BTreeMap::from([("custom_name", "Name")])
    );
}

#[test]
fn data_component_patch_get_split_forget_codec_keys_and_stream_order_match_java() {
    let prototype = ComponentMapModel::builder()
        .set("rarity", Some("common"))
        .set("repair_cost", Some("0"))
        .build()
        .unwrap();
    let patch = ComponentPatchModel::builder()
        .set("custom_name", "Sword")
        .remove("rarity")
        .set("repair_cost", "3")
        .build();

    assert_eq!(patch.get(&prototype, "custom_name"), Some("Sword"));
    assert_eq!(patch.get(&prototype, "rarity"), None);
    assert_eq!(patch.get(&prototype, "repair_cost"), Some("3"));
    assert_eq!(
        patch.forget(|ty| ty == "repair_cost").java_string(),
        "{custom_name=>Sword, !rarity}"
    );
    let (added, removed) = patch.split();
    assert_eq!(
        added.values,
        BTreeMap::from([("custom_name", "Sword"), ("repair_cost", "3")])
    );
    assert_eq!(removed, BTreeSet::from(["rarity"]));
    assert_eq!(
        patch.encode_order(),
        (
            vec![
                TypedComponent {
                    ty: "custom_name",
                    value: "Sword"
                },
                TypedComponent {
                    ty: "repair_cost",
                    value: "3"
                }
            ],
            vec!["rarity"]
        )
    );
    assert_eq!(
        ComponentPatchModel::codec_key("custom_name", true).unwrap(),
        "!custom_name"
    );
    assert_eq!(
        ComponentPatchModel::codec_key("creative_slot_lock", false).unwrap_err(),
        "'creative_slot_lock' is not a persistent component"
    );
    assert_eq!(
        ComponentPatchModel::codec_key("missing", false).unwrap_err(),
        "No component with type: 'missing'"
    );
}

#[test]
fn patched_component_map_sanitizes_defaults_and_iterates_patch_before_prototype() {
    let prototype = common_item_components();
    let patch = ComponentPatchModel::builder()
        .set("repair_cost", "0")
        .set("custom_name", "Sword")
        .remove("lore")
        .remove("missing_default")
        .build();
    let mut patched = PatchedComponentMapModel::from_patch(prototype.clone(), patch);

    assert_eq!(patched.get("repair_cost"), Some("0"));
    assert!(!patched.has_non_default("repair_cost"));
    assert_eq!(patched.get("custom_name"), Some("Sword"));
    assert!(patched.has_non_default("custom_name"));
    assert_eq!(patched.get("lore"), None);
    assert!(!patched.has_non_default("missing_default"));
    assert_eq!(
        patched.key_set(),
        prototype
            .key_set()
            .into_iter()
            .filter(|key| *key != "lore")
            .chain(["custom_name"])
            .collect()
    );

    assert_eq!(
        patched.iter()[0],
        TypedComponent {
            ty: "custom_name",
            value: "Sword"
        }
    );
    assert_eq!(patched.set("rarity", Some("rare")), Some("Rarity.COMMON"));
    assert_eq!(patched.remove("rarity"), Some("rare"));
    assert_eq!(patched.get("rarity"), None);
    let saved_patch = patched.as_patch();
    patched.clear_patch();
    assert_eq!(patched.get("rarity"), Some("Rarity.COMMON"));
    patched.restore_patch(saved_patch);
    assert_eq!(patched.get("rarity"), None);
}

#[test]
fn exact_predicate_factories_builder_matching_and_patch_match_java() {
    let components = ComponentMapModel::builder()
        .set("rarity", Some("common"))
        .set("custom_name", Some("Sword"))
        .set("repair_cost", Some("3"))
        .build()
        .unwrap();
    assert!(ExactPredicateModel::empty().always_matches());
    assert!(ExactPredicateModel::expect("rarity", "common").test(&components));
    assert!(!ExactPredicateModel::expect("rarity", "rare").test(&components));
    assert_eq!(ExactPredicateModel::all_of(&components).expected.len(), 3);
    assert_eq!(
        ExactPredicateModel::some_of(&components, &["custom_name", "missing"]).expected,
        vec![TypedComponent {
            ty: "custom_name",
            value: "Sword"
        }]
    );
    assert_eq!(
        ExactPredicateBuilder::default()
            .expect("rarity", "common")
            .unwrap()
            .expect("rarity", "rare")
            .unwrap_err(),
        "Predicate already has component of type: 'rarity'"
    );
    assert_eq!(
        ExactPredicateBuilder::default()
            .expect("rarity", "common")
            .unwrap()
            .build()
            .as_patch()
            .java_string(),
        "{rarity=>common}"
    );
}

#[test]
fn component_lookup_scans_holder_components_by_value_or_predicate() {
    let mut lookup = ComponentLookupModel::new(vec![
        HolderModel {
            key: "minecraft:stone",
            components: ComponentMapModel::builder()
                .set("rarity", Some("common"))
                .build()
                .unwrap(),
        },
        HolderModel {
            key: "minecraft:dragon_egg",
            components: ComponentMapModel::builder()
                .set("rarity", Some("rare"))
                .build()
                .unwrap(),
        },
    ]);
    assert_eq!(
        lookup.find_all("rarity", Some("rare")),
        vec!["minecraft:dragon_egg"]
    );
    assert_eq!(
        lookup.find_matching("rarity", |value| value.starts_with('c')),
        vec!["minecraft:stone"]
    );
    assert_eq!(lookup.scanned, BTreeSet::from(["rarity"]));
}

#[test]
fn data_component_initializers_group_by_registry_include_empty_entries_and_apply_order() {
    let mut initializers = ComponentInitializersModel::default();
    initializers.add("item", "minecraft:stick", "rarity", "common");
    initializers.add("item", "minecraft:stick", "custom_name", "Stick");
    initializers.add("block", "minecraft:stone", "custom_name", "Stone");

    let pending = initializers.build(&[
        ("item", vec!["minecraft:stick", "minecraft:air"]),
        ("block", vec!["minecraft:stone"]),
    ]);
    assert_eq!(pending[0].registry, "item");
    assert_eq!(pending[0].entries[0].0, "minecraft:stick");
    assert_eq!(
        pending[0].entries[0].1.values,
        BTreeMap::from([("custom_name", "Stick"), ("rarity", "common")])
    );
    assert!(pending[0].entries[1].1.values.is_empty());
    assert_eq!(pending[1].registry, "block");
    assert_eq!(
        pending[1].entries[0].1.values,
        BTreeMap::from([("custom_name", "Stone")])
    );
}

#[test]
fn common_item_components_match_java_defaults() {
    assert_eq!(common_item_components().values.len(), 10);
    assert_eq!(
        common_item_components().values,
        COMMON_ITEM_COMPONENTS.iter().copied().collect()
    );
}
