use std::collections::BTreeMap;

use super::*;

const DEPRECATED_TRANSLATIONS_INFO_JAVA: &str = vibecraft_java_source!("/net/minecraft/locale/DeprecatedTranslationsInfo.java");
const LANGUAGE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/locale/Language.java");
const PACKAGE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/locale/package-info.java");

#[test]
fn deprecated_translations_info_matches_java_codec_and_application_shape() {
    for sentinel in [
        "public record DeprecatedTranslationsInfo(List<String> removed, Map<String, String> renamed)",
        "fieldOf(\"removed\")",
        "fieldOf(\"renamed\")",
        "translations.remove(key);",
        "String value = translations.remove(fromKey);",
        "translations.remove(toKey);",
        "translations.put(toKey, value);",
    ] {
        assert!(
            DEPRECATED_TRANSLATIONS_INFO_JAVA.contains(sentinel),
            "missing DeprecatedTranslationsInfo sentinel {sentinel}"
        );
    }

    let info = DeprecatedTranslationsInfoModel::load_from_json(
        r#"{
            "removed": ["legacy.remove"],
            "renamed": {
                "legacy.rename": "modern.rename",
                "missing.rename": "modern.missing"
            }
        }"#,
    )
    .unwrap();
    let mut translations = BTreeMap::from([
        ("legacy.remove".to_string(), "gone".to_string()),
        ("legacy.rename".to_string(), "kept".to_string()),
        ("modern.missing".to_string(), "deleted".to_string()),
    ]);
    info.apply_to_map(&mut translations);

    assert!(!translations.contains_key("legacy.remove"));
    assert!(!translations.contains_key("legacy.rename"));
    assert_eq!(
        translations.get("modern.rename").map(String::as_str),
        Some("kept")
    );
    assert!(!translations.contains_key("modern.missing"));
    assert!(DeprecatedTranslationsInfoModel::empty().removed.is_empty());
}

#[test]
fn language_load_from_json_rewrites_java_unsupported_format_specifiers() {
    assert!(LANGUAGE_JAVA.contains("Pattern.compile(\"%(\\\\d+\\\\$)?[\\\\d.]*[df]\")"));
    assert!(LANGUAGE_JAVA.contains("replaceAll(\"%$1s\")"));

    let mut translations = BTreeMap::new();
    load_translations_from_json(
        r#"{
            "plain": "hello",
            "number": "Score: %d",
            "ordered": "%2$f then %1$d",
            "string": "%s survives",
            "percent": "%% %04d"
        }"#,
        &mut translations,
    )
    .unwrap();

    assert_eq!(translations["plain"], "hello");
    assert_eq!(translations["number"], "Score: %s");
    assert_eq!(translations["ordered"], "%2$s then %1$s");
    assert_eq!(translations["string"], "%s survives");
    assert_eq!(translations["percent"], "%% %s");
}

#[test]
fn default_language_applies_deprecated_resource_and_lookup_contracts() {
    for sentinel in [
        "public static final String DEFAULT = \"en_us\";",
        "private static volatile Language instance = loadDefault();",
        "DeprecatedTranslationsInfo.loadFromDefaultResource()",
        "deprecatedInfo.applyToMap(loadedData);",
        "return storage.getOrDefault(elementId, defaultValue);",
        "return storage.containsKey(elementId);",
        "return false;",
    ] {
        assert!(
            LANGUAGE_JAVA.contains(sentinel),
            "missing Language sentinel {sentinel}"
        );
    }

    let language = LanguageModel::load_default().unwrap();
    assert_eq!(DEFAULT_LANGUAGE, "en_us");
    assert!(language.has("block.minecraft.stone"));
    assert_eq!(
        language.get_or_default("block.minecraft.stone"),
        "Stone".to_string()
    );
    assert_eq!(
        language.get_or_default_with("missing.translation", "fallback"),
        "fallback".to_string()
    );
    assert_eq!(
        language.get_or_default("missing.translation"),
        "missing.translation".to_string()
    );
    assert!(!language.is_default_right_to_left());
    assert!(!language.has("block.minecraft.grass"));
    assert_eq!(
        language.get_or_default("gamerule.minecraft.keep_inventory"),
        "Keep inventory after death".to_string()
    );
    assert!(!language.has("gamerule.keepInventory"));
}

#[test]
fn injected_language_and_visual_order_model_java_contracts() {
    assert!(LANGUAGE_JAVA.contains("public static void inject(final Language language)"));
    assert!(LANGUAGE_JAVA.contains("getVisualOrder(final List<FormattedText> lines)"));

    let language = LanguageModel::injected(
        BTreeMap::from([("custom.key".to_string(), "Custom".to_string())]),
        true,
    );
    assert_eq!(language.get_or_default("custom.key"), "Custom");
    assert!(language.is_default_right_to_left());
    assert_eq!(language.visual_order("abc"), "abc");
    assert_eq!(
        language.visual_order_lines(["one", "two"]),
        vec!["one".to_string(), "two".to_string()]
    );
}

#[test]
fn locale_package_info_is_null_marked_like_java() {
    assert!(PACKAGE_INFO_JAVA.contains("@NullMarked"));
    assert!(PACKAGE_INFO_JAVA.contains("package net.minecraft.locale;"));
    assert!(PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
}
