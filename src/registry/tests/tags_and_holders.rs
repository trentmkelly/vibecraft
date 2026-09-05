use super::*;

#[test]
fn represents_holders_and_named_tag_sets() {
    let registry = Identifier::parse(registries::ITEM).unwrap();
    let tag = TagKey::<String>::new(registry.clone(), Identifier::parse("logs").unwrap());
    let set = HolderSet::<String>::Named(tag);
    assert!(matches!(set, HolderSet::Named(_)));

    let key = super::super::ResourceKey::new(registry, Identifier::parse("stick").unwrap());
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
fn tag_entry_and_file_match_java_codec_and_dependency_contracts() -> Result<(), String> {
    assert_eq!(TAG_ENTRY_JAVA.lines().count(), 115);
    for fragment in [
        "public static TagEntry element(final Identifier id)",
        "public static TagEntry optionalElement(final Identifier id)",
        "public static TagEntry tag(final Identifier id)",
        "public static TagEntry optionalTag(final Identifier id)",
        "public void visitRequiredDependencies(final Consumer<Identifier> output)",
        "public void visitOptionalDependencies(final Consumer<Identifier> output)",
        "public boolean verifyIfPresent",
        "result.append('?');",
    ] {
        assert!(TAG_ENTRY_JAVA.contains(fragment), "missing TagEntry source fragment: {fragment}");
    }
    assert_eq!(TAG_FILE_JAVA.lines().count(), 14);
    for fragment in [
        "public record TagFile(List<TagEntry> entries, boolean replace)",
        "TagEntry.CODEC.listOf().fieldOf(\"values\")",
        "Codec.BOOL.optionalFieldOf(\"replace\", false)",
    ] {
        assert!(TAG_FILE_JAVA.contains(fragment), "missing TagFile source fragment: {fragment}");
    }

    let element = super::super::TagEntry::element(Identifier::parse("minecraft:stick")?);
    let optional_tag = super::super::TagEntry::optional_tag(Identifier::parse("minecraft:logs")?);
    assert_eq!(element.to_string(), "minecraft:stick");
    assert_eq!(optional_tag.to_string(), "#minecraft:logs?");
    assert_eq!(element.to_json(), serde_json::json!("minecraft:stick"));
    assert_eq!(
        optional_tag.to_json(),
        serde_json::json!({"id": "#minecraft:logs", "required": false})
    );
    assert_eq!(
        super::super::TagEntry::from_json(&serde_json::json!("#minecraft:logs"))?,
        super::super::TagEntry::tag(Identifier::parse("minecraft:logs")?)
    );
    assert_eq!(
        super::super::TagEntry::from_json(&serde_json::json!({
            "id": "minecraft:missing",
            "required": false
        }))?,
        super::super::TagEntry::optional_element(Identifier::parse("minecraft:missing")?)
    );
    let mut required = Vec::new();
    optional_tag.visit_required_dependencies(&mut required);
    assert!(required.is_empty());
    let mut optional = Vec::new();
    optional_tag.visit_optional_dependencies(&mut optional);
    assert_eq!(optional, vec![Identifier::parse("minecraft:logs")?]);
    assert!(optional_tag.verify_if_present(|_| false, |_| false));

    let file = super::super::TagFile::new(
        Identifier::parse("minecraft:item")?,
        Identifier::parse("minecraft:logs")?,
        vec![element, optional_tag],
        false,
    );
    let encoded = file.to_json();
    assert_eq!(encoded["values"].as_array().map(Vec::len), Some(2));
    assert!(encoded.get("replace").is_none());
    let decoded = super::super::TagFile::from_json(
        Identifier::parse("minecraft:item")?,
        Identifier::parse("minecraft:logs")?,
        &encoded,
    )?;
    assert_eq!(decoded, file);
    Ok(())
}

#[test]
fn loaded_tags_resolve_nested_required_and_optional_tag_entries() -> Result<(), String> {
    let mut registry = Registry::new(Identifier::parse(registries::ITEM)?);
    let stick = Identifier::parse("minecraft:stick")?;
    let apple = Identifier::parse("minecraft:apple")?;
    registry.register(stick.clone(), "stick".to_string(), Lifecycle::Stable)?;
    registry.register(apple.clone(), "apple".to_string(), Lifecycle::Stable)?;
    let base = Identifier::parse("minecraft:base")?;
    let combined = Identifier::parse("minecraft:combined")?;
    let loaded = super::super::LoadedTags::load(
        &registry,
        [
            super::super::TagFile::new(
                Identifier::parse(registries::ITEM)?,
                base.clone(),
                vec![super::super::TagEntry::element(stick.clone())],
                true,
            ),
            super::super::TagFile::new(
                Identifier::parse(registries::ITEM)?,
                combined.clone(),
                vec![
                    super::super::TagEntry::tag(base.clone()),
                    super::super::TagEntry::optional_element(Identifier::parse("minecraft:missing")?),
                    super::super::TagEntry::element(apple.clone()),
                ],
                true,
            ),
        ],
    )
    .map_err(|errors| errors.join("; "))?;
    assert_eq!(
        loaded.values(&Identifier::parse(registries::ITEM)?, &combined),
        Some([stick, apple].as_slice())
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

    let experimental = defaults.join(super::super::FeatureFlagSet::of(&[
        feature_flags::MINECART_IMPROVEMENTS,
    ]));
    assert!(defaults.is_subset_of(experimental));
    assert!(!experimental.is_subset_of(defaults));
    Ok(())
}
