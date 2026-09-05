use super::Identifier;

#[test]
fn identifier_validation_and_factory_edges_match_java() {
    let source = vibecraft_java_source!("/net/minecraft/resources/Identifier.java");
    for sentinel in [
        "public static Identifier withDefaultNamespace(final String path)",
        "public static @Nullable Identifier tryBuild(final String namespace, final String path)",
        "public static boolean isValidNamespace(final String namespace)",
        "if (namespace.equals(\"..\"))",
        "throw new IdentifierException(\"Non [a-z0-9/._-] character in path of location: \" + namespace + \":\" + path);",
    ] {
        if !source.is_empty() {
            assert!(
                source.contains(sentinel),
                "Identifier.java sentinel missing: {sentinel}"
            );
        }
    }

    assert_eq!(Identifier::parse("stone").unwrap().to_string(), "minecraft:stone");
    assert_eq!(Identifier::parse(":stone").unwrap().to_string(), "minecraft:stone");
    assert_eq!(Identifier::parse("mod:").unwrap().to_string(), "mod:");
    assert_eq!(
        Identifier::from_namespace_and_path("my.pack", "path/to.file")
            .unwrap()
            .to_string(),
        "my.pack:path/to.file"
    );
    assert!(Identifier::is_valid_namespace(""));
    assert!(!Identifier::is_valid_namespace(".."));
    assert!(Identifier::is_valid_path(""));
    assert!(!Identifier::is_valid_path("bad:path"));
    assert_eq!(Identifier::try_parse("Upper:stone"), None);
    assert_eq!(Identifier::try_build("..", "stone"), None);
    assert_eq!(
        Identifier::read("minecraft:Bad").unwrap_err(),
        "Not a valid resource location: minecraft:Bad Non [a-z0-9/._-] character in path of location: minecraft:Bad"
    );
}

#[test]
fn identifier_helpers_and_order_match_java() {
    let source = vibecraft_java_source!("/net/minecraft/resources/Identifier.java");
    for sentinel in [
        "int result = this.path.compareTo(o.path);",
        "return this.toString().replace('/', '_').replace(':', '_');",
        "return this.namespace.equals(\"minecraft\") ? this.path : this.toString();",
    ] {
        if !source.is_empty() {
            assert!(
                source.contains(sentinel),
                "Identifier.java sentinel missing: {sentinel}"
            );
        }
    }

    let id = Identifier::parse("custom:block/stone").unwrap();
    assert_eq!(id.with_prefix("pre_").unwrap().to_string(), "custom:pre_block/stone");
    assert_eq!(
        id.with_suffix("_post").unwrap().to_string(),
        "custom:block/stone_post"
    );
    assert_eq!(id.to_debug_file_name(), "custom_block_stone");
    assert_eq!(id.to_language_key(), "custom.block/stone");
    assert_eq!(id.to_short_string(), "custom:block/stone");
    assert_eq!(Identifier::parse("minecraft:stone").unwrap().to_short_string(), "stone");

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
