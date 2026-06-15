//! Mirrors net.minecraft.network.chat.ComponentSerialization.java.
//!
//! This module models the recursive component codec surface used by commands,
//! saved data, and network stream-codec wrappers. It intentionally keeps the
//! actual component graph as `chat_component::Component`; the Java-specific
//! behavior here is the serialization contract around that graph.

use crate::chat_component::{
    Component, ComponentArgument, NbtSource, ObjectContent, TextColor,
};
use serde_json::{Map, Value};

pub const STREAM_CODEC: ComponentStreamCodec = ComponentStreamCodec {
    name: "STREAM_CODEC",
    trusted: false,
    optional: false,
    registry_friendly: true,
};
pub const OPTIONAL_STREAM_CODEC: ComponentStreamCodec = ComponentStreamCodec {
    name: "OPTIONAL_STREAM_CODEC",
    trusted: false,
    optional: true,
    registry_friendly: true,
};
pub const TRUSTED_STREAM_CODEC: ComponentStreamCodec = ComponentStreamCodec {
    name: "TRUSTED_STREAM_CODEC",
    trusted: true,
    optional: false,
    registry_friendly: true,
};
pub const TRUSTED_OPTIONAL_STREAM_CODEC: ComponentStreamCodec = ComponentStreamCodec {
    name: "TRUSTED_OPTIONAL_STREAM_CODEC",
    trusted: true,
    optional: true,
    registry_friendly: true,
};
pub const TRUSTED_CONTEXT_FREE_STREAM_CODEC: ComponentStreamCodec = ComponentStreamCodec {
    name: "TRUSTED_CONTEXT_FREE_STREAM_CODEC",
    trusted: true,
    optional: false,
    registry_friendly: false,
};

pub const BOOTSTRAP_CONTENT_TYPES: [&str; 7] = [
    "text",
    "translatable",
    "keybind",
    "score",
    "selector",
    "nbt",
    "object",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentStreamCodec {
    pub name: &'static str,
    pub trusted: bool,
    pub optional: bool,
    pub registry_friendly: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyMatcherPath {
    TypedDiscriminator,
    FuzzyCompact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentSerializationError {
    Json(String),
    EmptyList,
    NoMatchingCodec,
    MissingField(&'static str),
    InvalidField(&'static str),
    TooLarge { max_flat_size: usize },
}

pub fn decode_json_str(raw: &str) -> Result<Component, ComponentSerializationError> {
    let value = serde_json::from_str(raw)
        .map_err(|err| ComponentSerializationError::Json(err.to_string()))?;
    decode_json_value(&value)
}

pub fn decode_json_value(value: &Value) -> Result<Component, ComponentSerializationError> {
    match value {
        Value::String(text) => Ok(Component::literal(text.clone())),
        Value::Array(values) => create_from_list(
            values
                .iter()
                .map(decode_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(fields) => decode_json_object(fields),
        _ => Err(ComponentSerializationError::NoMatchingCodec),
    }
}

pub fn encode_json_value(component: &Component) -> Result<Value, ComponentSerializationError> {
    if let Some(text) = component.try_collapse_to_string() {
        return Ok(Value::String(text.to_string()));
    }
    serde_json::from_str(&component.to_json())
        .map_err(|err| ComponentSerializationError::Json(err.to_string()))
}

pub fn encode_json_string(component: &Component) -> Result<String, ComponentSerializationError> {
    Ok(encode_json_value(component)?.to_string())
}

pub fn flat_restricted_decode_json_str(
    raw: &str,
    max_flat_size: usize,
) -> Result<Component, ComponentSerializationError> {
    let component = decode_json_str(raw)?;
    if encode_json_string(&component)?.len() > max_flat_size {
        Err(ComponentSerializationError::TooLarge { max_flat_size })
    } else {
        Ok(component)
    }
}

pub fn create_from_list(
    mut list: Vec<Component>,
) -> Result<Component, ComponentSerializationError> {
    if list.is_empty() {
        return Err(ComponentSerializationError::EmptyList);
    }
    let mut result = list.remove(0).copy_component();
    for component in list {
        result = result.append(component);
    }
    Ok(result)
}

pub fn legacy_matcher_path(fields: &Map<String, Value>, type_field_name: &str) -> LegacyMatcherPath {
    if fields.contains_key(type_field_name) {
        LegacyMatcherPath::TypedDiscriminator
    } else {
        LegacyMatcherPath::FuzzyCompact
    }
}

fn decode_json_object(
    fields: &Map<String, Value>,
) -> Result<Component, ComponentSerializationError> {
    let mut component = match legacy_matcher_path(fields, "type") {
        LegacyMatcherPath::TypedDiscriminator => decode_typed_content(fields)?,
        LegacyMatcherPath::FuzzyCompact => decode_fuzzy_content(fields)?,
    };
    apply_style(fields, &mut component)?;
    if let Some(extra) = fields.get("extra") {
        let siblings = extra
            .as_array()
            .ok_or(ComponentSerializationError::InvalidField("extra"))?;
        if siblings.is_empty() {
            return Err(ComponentSerializationError::EmptyList);
        }
        for sibling in siblings {
            component = component.append(decode_json_value(sibling)?);
        }
    }
    Ok(component)
}

fn decode_typed_content(
    fields: &Map<String, Value>,
) -> Result<Component, ComponentSerializationError> {
    match string_field(fields, "type")?.as_str() {
        "text" => decode_text(fields),
        "translatable" => decode_translatable(fields),
        "keybind" => Ok(Component::keybind_component(string_field(fields, "keybind")?)),
        "score" => decode_score(fields),
        "selector" => decode_selector(fields),
        "nbt" => decode_nbt(fields),
        "object" => decode_object(fields),
        _ => Err(ComponentSerializationError::NoMatchingCodec),
    }
}

fn decode_fuzzy_content(
    fields: &Map<String, Value>,
) -> Result<Component, ComponentSerializationError> {
    if fields.contains_key("text") {
        return decode_text(fields);
    }
    if fields.contains_key("translate") {
        return decode_translatable(fields);
    }
    if fields.contains_key("keybind") {
        return Ok(Component::keybind_component(string_field(fields, "keybind")?));
    }
    if fields.contains_key("score") {
        return decode_score(fields);
    }
    if fields.contains_key("selector") {
        return decode_selector(fields);
    }
    if fields.contains_key("nbt") {
        return decode_nbt(fields);
    }
    if fields.contains_key("object") {
        return decode_object(fields);
    }
    Err(ComponentSerializationError::NoMatchingCodec)
}

fn decode_text(fields: &Map<String, Value>) -> Result<Component, ComponentSerializationError> {
    Ok(Component::literal(string_field(fields, "text")?))
}

fn decode_translatable(
    fields: &Map<String, Value>,
) -> Result<Component, ComponentSerializationError> {
    let args = match fields.get("with") {
        Some(Value::Array(values)) => values
            .iter()
            .map(component_arg_from_json)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(ComponentSerializationError::InvalidField("with")),
        None => Vec::new(),
    };
    let mut component = Component::translatable(string_field(fields, "translate")?, args);
    if let Some(fallback) = optional_string_field(fields, "fallback")? {
        component = component.with_fallback(fallback);
    }
    Ok(component)
}

fn decode_score(fields: &Map<String, Value>) -> Result<Component, ComponentSerializationError> {
    let score = fields
        .get("score")
        .and_then(Value::as_object)
        .ok_or(ComponentSerializationError::MissingField("score"))?;
    Ok(Component::score_component(
        string_field(score, "name")?,
        string_field(score, "objective")?,
    ))
}

fn decode_selector(fields: &Map<String, Value>) -> Result<Component, ComponentSerializationError> {
    let separator = fields
        .get("separator")
        .map(decode_json_value)
        .transpose()?;
    Ok(Component::selector_component(
        string_field(fields, "selector")?,
        separator,
    ))
}

fn decode_nbt(fields: &Map<String, Value>) -> Result<Component, ComponentSerializationError> {
    let source = if let Some(block) = optional_string_field(fields, "block")? {
        NbtSource::Block(block)
    } else if let Some(entity) = optional_string_field(fields, "entity")? {
        NbtSource::Entity(entity)
    } else if let Some(storage) = optional_string_field(fields, "storage")? {
        NbtSource::Storage(storage)
    } else {
        return Err(ComponentSerializationError::MissingField("source"));
    };
    let separator = fields
        .get("separator")
        .map(decode_json_value)
        .transpose()?;
    Ok(Component::nbt_component(
        string_field(fields, "nbt")?,
        source,
        bool_field(fields, "interpret")?.unwrap_or(false),
        bool_field(fields, "plain")?.unwrap_or(false),
        separator,
    ))
}

fn decode_object(fields: &Map<String, Value>) -> Result<Component, ComponentSerializationError> {
    let object = fields
        .get("object")
        .and_then(Value::as_object)
        .ok_or(ComponentSerializationError::MissingField("object"))?;
    let info = match string_field(object, "type")?.as_str() {
        "atlas" | "atlas_sprite" => ObjectContent::AtlasSprite {
            atlas: optional_string_field(object, "atlas")?
                .unwrap_or_else(|| "minecraft:blocks".to_string()),
            sprite: string_field(object, "sprite")?,
        },
        "player" => ObjectContent::PlayerSprite {
            profile: string_field(object, "profile")?,
            hat: bool_field(object, "hat")?.unwrap_or(true),
        },
        _ => return Err(ComponentSerializationError::NoMatchingCodec),
    };
    let fallback = fields
        .get("fallback")
        .map(decode_json_value)
        .transpose()?;
    Ok(Component::object_component(info, fallback))
}

fn component_arg_from_json(value: &Value) -> Result<ComponentArgument, ComponentSerializationError> {
    Ok(match value {
        Value::String(text) => ComponentArgument::String(text.clone()),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                if let Ok(value) = i32::try_from(value) {
                    ComponentArgument::Number(value)
                } else {
                    ComponentArgument::Long(value)
                }
            } else {
                return Err(ComponentSerializationError::InvalidField("with"));
            }
        }
        Value::Bool(value) => ComponentArgument::Boolean(*value),
        Value::Null => ComponentArgument::Null,
        Value::Array(_) | Value::Object(_) => {
            ComponentArgument::Component(Box::new(decode_json_value(value)?))
        }
    })
}

fn apply_style(
    fields: &Map<String, Value>,
    component: &mut Component,
) -> Result<(), ComponentSerializationError> {
    if let Some(color) = optional_string_field(fields, "color")? {
        component.style.color =
            Some(TextColor::parse(&color).ok_or(ComponentSerializationError::InvalidField(
                "color",
            ))?);
    }
    component.style.bold = bool_field(fields, "bold")?;
    component.style.italic = bool_field(fields, "italic")?;
    component.style.underlined = bool_field(fields, "underlined")?;
    component.style.strikethrough = bool_field(fields, "strikethrough")?;
    component.style.obfuscated = bool_field(fields, "obfuscated")?;
    if let Some(font) = optional_string_field(fields, "font")? {
        component.style.font = Some(crate::chat_component::FontDescription::Resource(font));
    }
    Ok(())
}

fn string_field(
    fields: &Map<String, Value>,
    name: &'static str,
) -> Result<String, ComponentSerializationError> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(ComponentSerializationError::MissingField(name))
}

fn optional_string_field(
    fields: &Map<String, Value>,
    name: &'static str,
) -> Result<Option<String>, ComponentSerializationError> {
    match fields.get(name) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ComponentSerializationError::InvalidField(name)),
        None => Ok(None),
    }
}

fn bool_field(
    fields: &Map<String, Value>,
    name: &'static str,
) -> Result<Option<bool>, ComponentSerializationError> {
    match fields.get(name) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ComponentSerializationError::InvalidField(name)),
        None => Ok(None),
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::{ResolutionContext, TranslationTable};

    const COMPONENT_SERIALIZATION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/ComponentSerialization.java");

    #[test]
    fn component_serialization_java_source_contract_is_tracked() {
        for sentinel in [
            "public static final Codec<Component> CODEC = Codec.recursive(\"Component\", ComponentSerialization::createCodec);",
            "public static final StreamCodec<RegistryFriendlyByteBuf, Component> STREAM_CODEC",
            "public static final StreamCodec<RegistryFriendlyByteBuf, Optional<Component>> OPTIONAL_STREAM_CODEC",
            "public static final StreamCodec<RegistryFriendlyByteBuf, Component> TRUSTED_STREAM_CODEC",
            "public static final StreamCodec<RegistryFriendlyByteBuf, Optional<Component>> TRUSTED_OPTIONAL_STREAM_CODEC",
            "public static final StreamCodec<ByteBuf, Component> TRUSTED_CONTEXT_FREE_STREAM_CODEC",
            "public static Codec<Component> flatRestrictedCodec(final int maxFlatSize)",
            "ComponentSerialization.CODEC.encodeStart(asJsonOps(ops), input)",
            "return json.isSuccess() && GsonHelper.encodesLongerThan",
            "private static MutableComponent createFromList(final List<Component> list)",
            "MutableComponent result = list.get(0).copy();",
            "public static <T> MapCodec<T> createLegacyComponentMatcher",
            "MapCodec<T> compactCodec = new ComponentSerialization.FuzzyCodec<>",
            "MapCodec<T> discriminatorCodec = types.codec(Codec.STRING).dispatchMap(typeFieldName",
            "MapCodec<T> contentsCodec = new ComponentSerialization.StrictEither<>",
            "ExtraCodecs.orCompressed(contentsCodec, discriminatorCodec)",
            "Codec.either(Codec.either(Codec.STRING, ExtraCodecs.nonEmptyList(topSerializer.listOf())), fullCodec)",
            "special.map(Component::literal, ComponentSerialization::createFromList)",
            "component.tryCollapseToString()",
            "contentTypes.put(\"text\", PlainTextContents.MAP_CODEC);",
            "contentTypes.put(\"object\", ObjectContents.MAP_CODEC);",
            "return input.get(this.typeFieldName) != null ? this.typed.decode(ops, input) : this.fuzzy.decode(ops, input);",
            "return \"FuzzyCodec[\" + this.codecs + \"]\";",
        ] {
            assert!(
                COMPONENT_SERIALIZATION_JAVA.contains(sentinel),
                "missing ComponentSerialization Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn stream_codec_aliases_and_bootstrap_order_match_java() {
        assert_eq!(
            BOOTSTRAP_CONTENT_TYPES,
            ["text", "translatable", "keybind", "score", "selector", "nbt", "object"]
        );
        assert_eq!(
            [
                STREAM_CODEC,
                OPTIONAL_STREAM_CODEC,
                TRUSTED_STREAM_CODEC,
                TRUSTED_OPTIONAL_STREAM_CODEC,
                TRUSTED_CONTEXT_FREE_STREAM_CODEC,
            ],
            [
                ComponentStreamCodec {
                    name: "STREAM_CODEC",
                    trusted: false,
                    optional: false,
                    registry_friendly: true,
                },
                ComponentStreamCodec {
                    name: "OPTIONAL_STREAM_CODEC",
                    trusted: false,
                    optional: true,
                    registry_friendly: true,
                },
                ComponentStreamCodec {
                    name: "TRUSTED_STREAM_CODEC",
                    trusted: true,
                    optional: false,
                    registry_friendly: true,
                },
                ComponentStreamCodec {
                    name: "TRUSTED_OPTIONAL_STREAM_CODEC",
                    trusted: true,
                    optional: true,
                    registry_friendly: true,
                },
                ComponentStreamCodec {
                    name: "TRUSTED_CONTEXT_FREE_STREAM_CODEC",
                    trusted: true,
                    optional: false,
                    registry_friendly: false,
                },
            ]
        );
    }

    #[test]
    fn recursive_codec_decodes_strings_lists_and_full_objects_like_java() {
        let literal = decode_json_str("\"hello\"").unwrap();
        assert_eq!(literal, Component::literal("hello"));
        assert_eq!(
            encode_json_value(&literal).unwrap(),
            Value::String("hello".to_string())
        );

        let list = decode_json_str(r#"["hello",{"text":" world","color":"gold"}]"#).unwrap();
        assert_eq!(list.get_string(), "hello world");
        assert_eq!(list.get_siblings().len(), 1);
        assert_eq!(list.get_siblings()[0].style.color, TextColor::parse("gold"));
        assert_eq!(
            encode_json_string(&list).unwrap(),
            r#"{"extra":[{"color":"gold","text":" world"}],"text":"hello"}"#
        );

        assert_eq!(
            decode_json_str("[]"),
            Err(ComponentSerializationError::EmptyList)
        );
    }

    #[test]
    fn legacy_matcher_prefers_type_discriminator_and_falls_back_to_fuzzy_fields() {
        let typed: Value = serde_json::from_str(r#"{"type":"text","text":"typed"}"#).unwrap();
        let fuzzy: Value = serde_json::from_str(r#"{"text":"fuzzy"}"#).unwrap();
        let typed_fields = typed.as_object().unwrap();
        let fuzzy_fields = fuzzy.as_object().unwrap();

        assert_eq!(
            legacy_matcher_path(typed_fields, "type"),
            LegacyMatcherPath::TypedDiscriminator
        );
        assert_eq!(
            legacy_matcher_path(fuzzy_fields, "type"),
            LegacyMatcherPath::FuzzyCompact
        );
        assert_eq!(decode_json_value(&typed).unwrap().get_string(), "typed");
        assert_eq!(decode_json_value(&fuzzy).unwrap().get_string(), "fuzzy");
    }

    #[test]
    fn codec_decodes_all_bootstrapped_content_shapes_and_object_fallback() {
        let translations = TranslationTable::default().with("chat.type.text", "<%s> %s");
        let context = ResolutionContext::default()
            .with_selector("@a", vec!["Steve", "Alex"])
            .with_keybind("key.jump", "Space")
            .with_score("Steve", "kills", 5)
            .with_nbt(
                NbtSource::Block("0 64 0".to_string()),
                "Items[0].id",
                vec!["minecraft:stone"],
            );
        let component = decode_json_str(
            r#"{"translate":"chat.type.text","with":[{"text":"Steve"},{"selector":"@a","separator":{"text":" | "}}],"extra":[{"keybind":"key.jump"},{"score":{"name":"Steve","objective":"kills"}},{"nbt":"Items[0].id","block":"0 64 0","separator":{"text":"; "}},{"object":{"type":"player","profile":"Alex","hat":false},"fallback":{"text":"Alex icon"}}]}"#,
        )
        .unwrap();

        assert_eq!(
            component.render_plain(&translations, &context),
            "<Steve> Steve | AlexSpace5minecraft:stoneAlex icon"
        );
    }

    #[test]
    fn flat_restricted_codec_checks_encoded_json_length_like_java() {
        assert_eq!(
            flat_restricted_decode_json_str(r#"{"text":"abcdef"}"#, 4),
            Err(ComponentSerializationError::TooLarge { max_flat_size: 4 })
        );
        assert_eq!(
            flat_restricted_decode_json_str(r#""abc""#, 5)
                .unwrap()
                .get_string(),
            "abc"
        );
    }
}
