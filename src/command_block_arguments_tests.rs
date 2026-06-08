use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn registry() -> BlockRegistryModel {
    BlockRegistryModel::default()
        .with_block(BlockDefinitionModel::new("minecraft:stone"))
        .with_block(
            BlockDefinitionModel::new("minecraft:oak_log").with_property(
                "axis",
                "y",
                &["x", "y", "z"],
            ),
        )
        .with_block(
            BlockDefinitionModel::new("minecraft:chest")
                .with_property("facing", "north", &["north", "south"])
                .with_block_entity(),
        )
        .with_block(BlockDefinitionModel::new("minecraft:fragile_air"))
        .with_tag("minecraft:stone", &["minecraft:stone", "minecraft:oak_log"])
        .with_tag("minecraft:containers", &["minecraft:chest"])
}

#[test]
fn java_block_state_argument_factory_examples_context_and_parse_match_source() {
    const { assert!(BLOCK_ARGUMENTS_PACKAGE_NULL_MARKED) };
    let registry = registry();
    let argument = BlockStateArgumentModel::block(&CommandBuildContextModel);
    assert_eq!(
        argument.examples(),
        ["stone", "minecraft:stone", "stone[foo=bar]", "foo{bar=baz}"]
    );

    let mut reader = StringReaderModel::new("oak_log[axis=x]{name=beam} rest");
    let input = argument
        .parse(&registry, &mut reader)
        .unwrap_or_else(|error| panic!("block state should parse: {error:?}"));
    assert_eq!(reader.cursor(), 26);
    assert_eq!(input.get_state().block_id, "minecraft:oak_log");
    assert_eq!(
        input.get_defined_properties(),
        &BTreeSet::from(["axis".to_string()])
    );
    assert_eq!(
        input.nbt,
        Some(BTreeMap::from([("name".to_string(), "beam".to_string())]))
    );

    let context = CommandContextModel::default().with_block("block", input.clone());
    assert_eq!(get_block(&context, "block"), Some(input));
    assert_eq!(
        argument.list_suggestions(&registry, &mut SuggestionsBuilderModel::new("")),
        vec![
            "minecraft:chest".to_string(),
            "minecraft:fragile_air".to_string(),
            "minecraft:oak_log".to_string(),
            "minecraft:stone".to_string(),
        ]
    );
}

#[test]
fn java_block_state_parser_resets_cursor_and_rejects_tags_when_not_testing() {
    let registry = registry();
    let argument = BlockStateArgumentModel::block(&CommandBuildContextModel);
    let mut unknown = StringReaderModel::new("missing[axis=x]");
    assert_eq!(
        argument.parse(&registry, &mut unknown),
        Err(BlockArgumentError::UnknownBlock {
            cursor: 0,
            id: "minecraft:missing".to_string()
        })
    );
    assert_eq!(unknown.cursor(), 0);

    let mut tag = StringReaderModel::new("#minecraft:stone");
    assert_eq!(
        argument.parse(&registry, &mut tag),
        Err(BlockArgumentError::NoTagsAllowed { cursor: 0 })
    );
    assert_eq!(tag.cursor(), 0);
}

#[test]
fn java_block_properties_validate_unknown_duplicate_missing_invalid_and_unclosed() {
    let registry = registry();
    let argument = BlockStateArgumentModel::block(&CommandBuildContextModel);
    assert!(matches!(
        argument.parse(&registry, &mut StringReaderModel::new("oak_log[missing=x]")),
        Err(BlockArgumentError::UnknownProperty { cursor: 8, .. })
    ));
    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("oak_log[axis=x,axis=y]")
        ),
        Err(BlockArgumentError::DuplicateProperty { cursor: 15, .. })
    ));
    assert!(matches!(
        argument.parse(&registry, &mut StringReaderModel::new("oak_log[axis]")),
        Err(BlockArgumentError::ExpectedValue { cursor: 12, .. })
    ));
    assert!(matches!(
        argument.parse(&registry, &mut StringReaderModel::new("oak_log[axis=q]")),
        Err(BlockArgumentError::InvalidValue { cursor: 13, .. })
    ));
    assert_eq!(
        argument.parse(&registry, &mut StringReaderModel::new("oak_log[axis=x")),
        Err(BlockArgumentError::ExpectedEndOfProperties { cursor: 14 })
    );
}

#[test]
fn java_block_input_test_and_place_match_defined_properties_and_nbt() {
    let registry = registry();
    let argument = BlockStateArgumentModel::block(&CommandBuildContextModel);
    let input = argument
        .parse(
            &registry,
            &mut StringReaderModel::new("chest[facing=south]{loot=gold}"),
        )
        .unwrap_or_else(|error| panic!("block input should parse: {error:?}"));

    let matching = BlockInWorldModel::new(
        BlockStateModel::new("minecraft:chest").with_property("facing", "south"),
        Some(BlockEntityModel::new(&[
            ("loot", "gold"),
            ("extra", "kept"),
        ])),
    );
    let wrong_property = BlockInWorldModel::new(
        BlockStateModel::new("minecraft:chest").with_property("facing", "north"),
        Some(BlockEntityModel::new(&[("loot", "gold")])),
    );
    assert!(input.test(&matching));
    assert!(!input.test(&wrong_property));

    let mut level = ServerLevelBlockModel::default().with_block(
        (1, 2, 3),
        BlockInWorldModel::new(
            BlockStateModel::new("minecraft:chest").with_property("facing", "north"),
            Some(BlockEntityModel::new(&[("loot", "old")])),
        ),
    );
    assert!(input.place(&mut level, (1, 2, 3), 0));
    let placed = level.blocks.get(&(1, 2, 3)).expect("placed block exists");
    assert_eq!(
        placed.state.properties.get("facing"),
        Some(&"south".to_string())
    );
    assert_eq!(
        placed
            .entity
            .as_ref()
            .and_then(|entity| entity.nbt.get("loot")),
        Some(&"gold".to_string())
    );
    assert!(level.changed_blocks.contains(&(1, 2, 3)));

    let fragile = BlockInputModel::new(
        BlockStateModel::new("minecraft:fragile_air"),
        BTreeSet::new(),
        None,
    );
    assert!(fragile.place(&mut level, (2, 2, 2), 0));
    assert_eq!(
        level
            .blocks
            .get(&(2, 2, 2))
            .map(|block| &block.state.block_id),
        Some(&"minecraft:fragile_air".to_string())
    );
}

#[test]
fn java_block_predicate_argument_supports_blocks_tags_vague_properties_and_nbt() {
    let registry = registry();
    let argument = BlockPredicateArgumentModel::block_predicate(&CommandBuildContextModel);
    assert_eq!(
        argument.examples(),
        [
            "stone",
            "minecraft:stone",
            "stone[foo=bar]",
            "#stone",
            "#stone[foo=bar]{baz=nbt}"
        ]
    );

    let block = argument
        .parse(&registry, &mut StringReaderModel::new("oak_log[axis=z]"))
        .unwrap_or_else(|error| panic!("block predicate should parse: {error:?}"));
    assert!(!block.requires_nbt());
    assert!(block.test(
        &registry,
        &BlockInWorldModel::new(
            BlockStateModel::new("minecraft:oak_log").with_property("axis", "z"),
            None,
        )
    ));

    let tag = argument
        .parse(&registry, &mut StringReaderModel::new("#stone[axis=x]"))
        .unwrap_or_else(|error| panic!("tag predicate should parse: {error:?}"));
    assert!(!tag.requires_nbt());
    assert!(tag.test(
        &registry,
        &BlockInWorldModel::new(
            BlockStateModel::new("minecraft:oak_log").with_property("axis", "x"),
            None,
        )
    ));
    assert!(!tag.test(
        &registry,
        &BlockInWorldModel::new(BlockStateModel::new("minecraft:stone"), None)
    ));

    let nbt = argument
        .parse(
            &registry,
            &mut StringReaderModel::new("#containers{loot=gold}"),
        )
        .unwrap_or_else(|error| panic!("tag nbt predicate should parse: {error:?}"));
    assert!(nbt.requires_nbt());
    assert!(nbt.test(
        &registry,
        &BlockInWorldModel::new(
            BlockStateModel::new("minecraft:chest").with_property("facing", "north"),
            Some(BlockEntityModel::new(&[("loot", "gold")]))
        )
    ));

    let context = CommandContextModel::default().with_predicate("p", nbt.clone());
    assert_eq!(get_block_predicate(&context, "p"), Some(nbt));
}

#[test]
fn java_tag_errors_and_suggestions_match_parser_entry_points() {
    let registry = registry();
    let argument = BlockPredicateArgumentModel::block_predicate(&CommandBuildContextModel);
    let mut unknown = StringReaderModel::new("#missing");
    assert_eq!(
        argument.parse(&registry, &mut unknown),
        Err(BlockArgumentError::UnknownTag {
            cursor: 0,
            tag: "minecraft:missing".to_string()
        })
    );
    assert_eq!(unknown.cursor(), 0);

    let mut duplicate = StringReaderModel::new("#stone[axis=x,axis=y]");
    assert!(matches!(
        argument.parse(&registry, &mut duplicate),
        Err(BlockArgumentError::DuplicateProperty { cursor: 14, .. })
    ));

    assert_eq!(
        argument.list_suggestions(&registry, &mut SuggestionsBuilderModel::new("")),
        vec![
            "minecraft:chest".to_string(),
            "minecraft:fragile_air".to_string(),
            "minecraft:oak_log".to_string(),
            "minecraft:stone".to_string(),
            "#minecraft:containers".to_string(),
            "#minecraft:stone".to_string(),
        ]
    );
    assert_eq!(
        argument.list_suggestions(
            &registry,
            &mut SuggestionsBuilderModel::new("#minecraft:containers")
        ),
        vec!["[".to_string(), "{".to_string()]
    );
    assert_eq!(
        serialize(&BlockStateModel::new("minecraft:oak_log").with_property("axis", "x")),
        "minecraft:oak_log[axis=x]"
    );
}
