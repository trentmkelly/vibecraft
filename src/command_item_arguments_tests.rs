use super::*;

fn registry() -> ItemRegistryModel {
    ItemRegistryModel::default()
        .with_item("minecraft:stick", 64)
        .with_item("minecraft:diamond_sword", 1)
        .with_item("minecraft:apple", 64)
        .with_tag("minecraft:tools", &["minecraft:diamond_sword"])
        .with_component("minecraft:custom_name")
        .with_component("minecraft:custom_data")
        .with_component("minecraft:damage")
        .with_marker_component("minecraft:glider")
        .with_transient_component("minecraft:debug_stick_state")
        .with_predicate("minecraft:custom_data")
}

#[test]
fn item_argument_examples_context_parse_suggestions_and_stack_creation_match_java() {
    const { assert!(ITEM_ARGUMENTS_PACKAGE_NULL_MARKED) };
    let registry = registry();
    let argument = ItemArgumentModel::item(&CommandBuildContextModel);
    assert_eq!(
        argument.examples(),
        ["stick", "minecraft:stick", "stick{foo=bar}"]
    );

    let mut reader =
        StringReaderModel::new("stick[minecraft:custom_name=Blade,!minecraft:damage] tail");
    let input = argument
        .parse(&registry, &mut reader)
        .unwrap_or_else(|error| panic!("item should parse: {error:?}"));
    assert_eq!(reader.cursor(), 52);
    assert_eq!(input.item_id(), "minecraft:stick");
    assert_eq!(
        input
            .components()
            .set_components()
            .get("minecraft:custom_name"),
        Some(&"Blade".to_string())
    );
    assert!(input
        .components()
        .removed_components()
        .contains("minecraft:damage"));

    let context = CommandContextModel::default().with_item("item", input.clone());
    assert_eq!(
        ItemArgumentModel::get_item(&context, "item"),
        Some(input.clone())
    );
    assert_eq!(
        input.create_item_stack(&registry, 5).unwrap(),
        ItemStackModel::new("stick", 5).with_component("custom_name", "Blade")
    );
    assert_eq!(
        ItemInputModel::new("minecraft:diamond_sword".to_string(), Default::default())
            .create_item_stack(&registry, 2),
        Err(ItemArgumentError::StackTooBig {
            item: "minecraft:diamond_sword".to_string(),
            max: 1
        })
    );

    assert_eq!(
        argument.list_suggestions(&registry, &mut SuggestionsBuilderModel::new("")),
        vec![
            "minecraft:apple".to_string(),
            "minecraft:diamond_sword".to_string(),
            "minecraft:stick".to_string()
        ]
    );
    assert_eq!(
        argument.list_suggestions(&registry, &mut SuggestionsBuilderModel::new("stick")),
        vec!["[".to_string()]
    );
    assert_eq!(
        argument.list_suggestions(&registry, &mut SuggestionsBuilderModel::new("stick[")),
        vec![
            "!".to_string(),
            "minecraft:custom_data=".to_string(),
            "minecraft:custom_name=".to_string(),
            "minecraft:damage=".to_string()
        ]
    );
}

#[test]
fn item_parser_resets_cursor_and_reports_java_component_errors() {
    let registry = registry();
    let argument = ItemArgumentModel::item(&CommandBuildContextModel);
    let mut unknown_item = StringReaderModel::new("missing[minecraft:custom_name=x]");
    assert_eq!(
        argument.parse(&registry, &mut unknown_item),
        Err(ItemArgumentError::UnknownItem {
            cursor: 0,
            id: "minecraft:missing".to_string()
        })
    );
    assert_eq!(unknown_item.cursor(), 0);

    let mut unknown_component = StringReaderModel::new("stick[minecraft:missing=x]");
    assert_eq!(
        argument.parse(&registry, &mut unknown_component),
        Err(ItemArgumentError::UnknownComponent {
            cursor: 6,
            id: "minecraft:missing".to_string()
        })
    );
    assert_eq!(unknown_component.cursor(), 0);

    assert_eq!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("stick[minecraft:custom_name=x,minecraft:custom_name=y]")
        ),
        Err(ItemArgumentError::RepeatedComponent {
            id: "minecraft:custom_name".to_string()
        })
    );
    assert!(matches!(
        argument.parse(&registry, &mut StringReaderModel::new("stick[")),
        Err(ItemArgumentError::ExpectedChar { value: ']', .. })
    ));
    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("stick[minecraft:custom_name=bad]")
        ),
        Err(ItemArgumentError::MalformedComponent { cursor: 28, .. })
    ));
}

#[test]
fn item_predicate_argument_matches_type_tag_components_count_predicates_and_context() {
    let registry = registry();
    let argument = ItemPredicateArgumentModel::item_predicate(&CommandBuildContextModel);
    assert_eq!(
        argument.examples(),
        ["stick", "minecraft:stick", "#stick", "#stick{foo:'bar'}"]
    );

    let mut reader = StringReaderModel::new(
        "#tools[minecraft:damage=3|minecraft:custom_name=Tool,!minecraft:glider,count=1..2]",
    );
    let predicate = argument
        .parse(&registry, &mut reader)
        .unwrap_or_else(|error| panic!("predicate should parse: {error:?}"));
    let matching = ItemStackModel::new("diamond_sword", 1).with_component("damage", "3");
    let blocked = matching.clone().with_component("glider", "unit");
    assert!(predicate.test(&registry, &matching));
    assert!(!predicate.test(&registry, &blocked));
    assert!(!predicate.test(&registry, &ItemStackModel::new("apple", 1)));

    let custom_data = argument
        .parse(
            &registry,
            &mut StringReaderModel::new("*[minecraft:custom_data~{foo:bar}]"),
        )
        .unwrap();
    assert!(custom_data.test(
        &registry,
        &ItemStackModel::new("stick", 8).with_component("custom_data", "{foo:bar}")
    ));

    let context = CommandContextModel::default().with_predicate("predicate", predicate.clone());
    assert_eq!(
        ItemPredicateArgumentModel::get_item_predicate(&context, "predicate"),
        Some(predicate)
    );
}

#[test]
fn item_predicate_parser_reports_lookup_and_codec_errors_with_reset() {
    let registry = registry();
    let argument = ItemPredicateArgumentModel::item_predicate(&CommandBuildContextModel);
    let mut unknown_tag = StringReaderModel::new("#missing");
    assert_eq!(
        argument.parse(&registry, &mut unknown_tag),
        Err(ItemArgumentError::UnknownTag {
            cursor: 1,
            id: "minecraft:missing".to_string()
        })
    );
    assert_eq!(unknown_tag.cursor(), 0);

    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("*[minecraft:debug_stick_state]")
        ),
        Err(ItemArgumentError::UnknownComponent { cursor: 2, .. })
    ));
    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("*[minecraft:missing~1]")
        ),
        Err(ItemArgumentError::UnknownPredicate { cursor: 2, .. })
    ));
    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("*[minecraft:count=bad]")
        ),
        Err(ItemArgumentError::InvalidValue { cursor: 18 })
    ));
    assert!(matches!(
        argument.parse(
            &registry,
            &mut StringReaderModel::new("*[minecraft:custom_data~bad]")
        ),
        Err(ItemArgumentError::MalformedPredicate { cursor: 24, .. })
    ));
}

#[test]
fn function_argument_parses_without_validation_and_resolves_lazily_like_java() {
    let foo = CommandFunctionModel::new("minecraft:foo", &["say foo"]);
    let bar = CommandFunctionModel::new("foo:bar", &["say bar"]);
    let manager = CommandFunctionManagerModel::default()
        .with_function(foo.clone())
        .with_function(bar.clone())
        .with_tag("minecraft:load", vec![foo.clone(), bar.clone()]);
    let argument = FunctionArgumentModel::functions();
    assert_eq!(argument.examples(), ["foo", "foo:bar", "#foo"]);

    let parsed_function = argument
        .parse(&mut StringReaderModel::new("foo"))
        .expect("function id parses before lookup");
    let parsed_tag = argument
        .parse(&mut StringReaderModel::new("#load"))
        .expect("tag id parses before lookup");
    let context = CommandFunctionContextModel::new(manager)
        .with_argument("single", parsed_function)
        .with_argument("tagged", parsed_tag);

    assert_eq!(
        FunctionArgumentModel::get_functions(&context, "single"),
        Ok(vec![foo.clone()])
    );
    assert_eq!(
        FunctionArgumentModel::get_function_or_tag(&context, "tagged"),
        Ok(FunctionUnwrapModel::Tag(
            "minecraft:load".to_string(),
            vec![foo.clone(), bar.clone()]
        ))
    );
    assert_eq!(
        FunctionArgumentModel::get_function_collection(&context, "single"),
        Ok(("minecraft:foo".to_string(), vec![foo]))
    );

    let missing = CommandFunctionContextModel::new(CommandFunctionManagerModel::default())
        .with_argument(
            "missing",
            FunctionResultModel::Function("minecraft:missing".to_string()),
        )
        .with_argument(
            "missing_tag",
            FunctionResultModel::Tag("minecraft:missing".to_string()),
        );
    assert_eq!(
        FunctionArgumentModel::get_functions(&missing, "missing"),
        Err(ItemArgumentError::UnknownFunction {
            id: "minecraft:missing".to_string()
        })
    );
    assert_eq!(
        FunctionArgumentModel::get_functions(&missing, "missing_tag"),
        Err(ItemArgumentError::UnknownFunctionTag {
            id: "minecraft:missing".to_string()
        })
    );
}
