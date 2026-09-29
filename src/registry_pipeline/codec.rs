//! Transcoding codec engine for registry element data.
//!
//! Java decodes an element from a data pack with `JsonOps` and, when the element
//! is synchronised to a client, re-encodes it with `NbtOps` (`RegistrySynchronization
//! .packRegistry`). Because every registry codec is a lossless (modulo defaults and
//! stripped fields) mapping between those two representations, this port models a
//! codec as a single function `JSON -> NBT` that validates the input exactly like
//! the Java decoder and emits exactly what the Java encoder would have produced:
//! defaulted optional fields are omitted, `orElse` fields are always written, numeric
//! leaves carry the NBT type of the Java field (`Codec.FLOAT` -> `FloatTag`, ...), and
//! registry holders collapse to their identifier string.
//!
//! Codecs are built from the combinators in this module, mirroring the DFU building
//! blocks the Java classes use (`RecordCodecBuilder`, `fieldOf`, `optionalFieldOf`,
//! `listOf`, `Codec.either`, `dispatch`, `RegistryFixedCodec`, ...). Decoding a
//! registry reference is context dependent (`RegistryOps`), which [`CodecContext`]
//! models: built-in registries are resolved immediately, registries that are being
//! loaded record the reference so [`crate::registry_pipeline::loader`] can report
//! `Unbound values in registry ...` when the registry freezes.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use serde_json::{Map, Value as Json};

use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;
use crate::storage::nbt::Tag;

/// Result of a codec run; the error is the DFU `DataResult` error message.
pub type CodecResult<T> = Result<T, String>;

/// A `(registry, element id)` reference recorded while decoding.
pub type ElementReference = (Identifier, Identifier);

/// References collected by a decode run, resolved when the target registry freezes.
#[derive(Debug, Default, Clone)]
pub struct References {
    /// Element references (`RegistryFixedCodec`/`RegistryFileCodec` lookups).
    pub elements: BTreeSet<ElementReference>,
    /// Tag references (`HolderSetCodec` named sets).
    pub tags: BTreeSet<ElementReference>,
}

/// Equivalent of the `RegistryOps` a Java decoder runs with.
pub struct CodecContext<'a> {
    builtin: &'a BuiltinRegistries,
    loading: &'a BTreeSet<Identifier>,
    references: RefCell<References>,
}

/// How a registry can be resolved from inside a codec.
enum RegistryKind {
    /// A static registry whose keys are all known up front.
    Builtin,
    /// A data-pack registry that is part of the current load.
    Loading,
    /// A registry this port does not load yet; references cannot be verified.
    Unloaded,
}

impl<'a> CodecContext<'a> {
    /// Creates a context over the built-in registries and the set of registries
    /// that the current load is populating.
    pub fn new(builtin: &'a BuiltinRegistries, loading: &'a BTreeSet<Identifier>) -> Self {
        Self {
            builtin,
            loading,
            references: RefCell::new(References::default()),
        }
    }

    /// The built-in registries used to resolve static holders.
    pub fn builtin(&self) -> &BuiltinRegistries {
        self.builtin
    }

    /// Drains the references recorded so far.
    pub fn take_references(&self) -> References {
        std::mem::take(&mut *self.references.borrow_mut())
    }

    fn kind(&self, registry: &Identifier) -> RegistryKind {
        if self.builtin.contains_registry(registry) {
            RegistryKind::Builtin
        } else if self.loading.contains(registry) {
            RegistryKind::Loading
        } else {
            RegistryKind::Unloaded
        }
    }

    /// `HolderGetter.get(ResourceKey)`: resolves or records an element reference.
    fn resolve_element(&self, registry: &Identifier, id: &Identifier) -> CodecResult<()> {
        match self.kind(registry) {
            RegistryKind::Builtin => {
                if self.builtin.contains_element(registry, id) {
                    Ok(())
                } else {
                    Err(format!(
                        "Failed to get element ResourceKey[minecraft:root / {registry}]: {id}"
                    ))
                }
            }
            RegistryKind::Loading => {
                self.references
                    .borrow_mut()
                    .elements
                    .insert((registry.clone(), id.clone()));
                Ok(())
            }
            RegistryKind::Unloaded => Ok(()),
        }
    }

    /// `HolderGetter.get(TagKey)`: static registries accept any tag (the
    /// registration lookup creates it on demand); loading registries must bind it.
    fn resolve_tag(&self, registry: &Identifier, tag: &Identifier) {
        if matches!(self.kind(registry), RegistryKind::Loading) {
            self.references
                .borrow_mut()
                .tags
                .insert((registry.clone(), tag.clone()));
        }
    }
}

/// The transcoding function of a [`Codec`].
type CodecFn = dyn Fn(&Json, &CodecContext<'_>) -> CodecResult<Tag>;

/// A `JSON -> NBT` codec.
#[derive(Clone)]
pub struct Codec(Rc<CodecFn>);

impl Codec {
    /// Wraps a transcoding function.
    pub fn new(f: impl Fn(&Json, &CodecContext<'_>) -> CodecResult<Tag> + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// Decodes `json` and returns its network/NBT encoding.
    pub fn parse(&self, json: &Json, ctx: &CodecContext<'_>) -> CodecResult<Tag> {
        (self.0)(json, ctx)
    }

    /// `Codec.validate`: runs `check` on the encoded value after a successful parse.
    pub fn validate(self, check: impl Fn(&Tag) -> CodecResult<()> + 'static) -> Self {
        Self::new(move |json, ctx| {
            let tag = self.parse(json, ctx)?;
            check(&tag)?;
            Ok(tag)
        })
    }

    /// `Codec.xmap`/`flatXmap` where only the produced value changes.
    pub fn map_tag(self, map: impl Fn(Tag) -> CodecResult<Tag> + 'static) -> Self {
        Self::new(move |json, ctx| map(self.parse(json, ctx)?))
    }
}

/// Builds a codec lazily so recursive definitions (`Codec.recursive`) can refer to
/// themselves.
pub fn lazy(build: fn() -> Codec) -> Codec {
    Codec::new(move |json, ctx| build().parse(json, ctx))
}

/// Renders JSON for error messages the way `JsonElement.toString` does.
pub fn describe(json: &Json) -> String {
    json.to_string()
}

// ---------------------------------------------------------------------------
// Primitive codecs
// ---------------------------------------------------------------------------

/// `JsonOps.getNumberValue`: numbers, plus booleans as 1/0.
fn number_of(json: &Json) -> CodecResult<f64> {
    match json {
        Json::Number(n) => n
            .as_f64()
            .ok_or_else(|| format!("Not a number: {}", describe(json))),
        Json::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(format!("Not a number: {}", describe(json))),
    }
}

/// `Number.intValue()` of a JSON number: integers wrap, decimals truncate/saturate.
pub fn json_int_value(json: &Json) -> CodecResult<i32> {
    match json {
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i as i32)
            } else if let Some(u) = n.as_u64() {
                Ok(u as i32)
            } else {
                Ok(number_of(json)? as i32)
            }
        }
        _ => Ok(number_of(json)? as i32),
    }
}

/// `Codec.BOOL`.
pub fn bool_codec() -> Codec {
    Codec::new(|json, _| match json {
        Json::Bool(b) => Ok(Tag::Byte(i8::from(*b))),
        Json::Number(_) => Ok(Tag::Byte(i8::from((number_of(json)? as i64 as i8) != 0))),
        _ => Err(format!("Not a boolean: {}", describe(json))),
    })
}

/// `Codec.INT`.
pub fn int_codec() -> Codec {
    Codec::new(|json, _| Ok(Tag::Int(json_int_value(json)?)))
}

/// `Codec.intRange(min, max)`.
pub fn int_range(min: i32, max: i32) -> Codec {
    Codec::new(move |json, _| {
        let value = json_int_value(json)?;
        if value >= min && value <= max {
            Ok(Tag::Int(value))
        } else {
            Err(format!("Value {value} outside of range [{min}:{max}]"))
        }
    })
}

/// `ExtraCodecs.NON_NEGATIVE_INT`.
pub fn non_negative_int() -> Codec {
    Codec::new(|json, _| {
        let value = json_int_value(json)?;
        if value >= 0 {
            Ok(Tag::Int(value))
        } else {
            Err(format!("Value must be non-negative: {value}"))
        }
    })
}

/// `ExtraCodecs.POSITIVE_INT`.
pub fn positive_int() -> Codec {
    Codec::new(|json, _| {
        let value = json_int_value(json)?;
        if value >= 1 {
            Ok(Tag::Int(value))
        } else {
            Err(format!("Value must be positive: {value}"))
        }
    })
}

/// `Codec.FLOAT`.
pub fn float_codec() -> Codec {
    Codec::new(|json, _| Ok(Tag::Float(number_of(json)? as f32)))
}

/// `Codec.floatRange(min, max)`.
pub fn float_range(min: f32, max: f32) -> Codec {
    Codec::new(move |json, _| {
        let value = number_of(json)? as f32;
        if value >= min && value <= max {
            Ok(Tag::Float(value))
        } else {
            Err(format!("Value {value} outside of range [{min}:{max}]"))
        }
    })
}

/// `ExtraCodecs.POSITIVE_FLOAT` (exclusive lower bound of zero).
pub fn positive_float() -> Codec {
    Codec::new(|json, _| {
        let value = number_of(json)? as f32;
        if value > 0.0 && value <= f32::MAX {
            Ok(Tag::Float(value))
        } else {
            Err(format!("Value must be positive: {value}"))
        }
    })
}

/// `Codec.LONG` (`Number.longValue()`: decimals truncate).
pub fn long_codec() -> Codec {
    Codec::new(|json, _| match json {
        Json::Number(n) => Ok(Tag::Long(
            n.as_i64()
                .or_else(|| n.as_u64().map(|u| u as i64))
                .unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        )),
        _ => Ok(Tag::Long(number_of(json)? as i64)),
    })
}

/// `Codec.DOUBLE`.
pub fn double_codec() -> Codec {
    Codec::new(|json, _| Ok(Tag::Double(number_of(json)?)))
}

/// `Codec.doubleRange(min, max)`.
pub fn double_range(min: f64, max: f64) -> Codec {
    Codec::new(move |json, _| {
        let value = number_of(json)?;
        if value >= min && value <= max {
            Ok(Tag::Double(value))
        } else {
            Err(format!("Value {value} outside of range [{min}:{max}]"))
        }
    })
}

fn string_of(json: &Json) -> CodecResult<&str> {
    json.as_str()
        .ok_or_else(|| format!("Not a string: {}", describe(json)))
}

/// `Codec.STRING`.
pub fn string_codec() -> Codec {
    Codec::new(|json, _| Ok(Tag::String(string_of(json)?.to_string())))
}

/// Parses `Identifier.CODEC` input.
pub fn parse_identifier(json: &Json) -> CodecResult<Identifier> {
    let text = string_of(json)?;
    Identifier::parse(text).map_err(|err| format!("Not a valid resource location: {text} {err}"))
}

/// `Identifier.CODEC`.
pub fn identifier_codec() -> Codec {
    Codec::new(|json, _| Ok(Tag::String(parse_identifier(json)?.to_string())))
}

/// `ExtraCodecs.RESOURCE_PATH_CODEC`.
pub fn resource_path_codec() -> Codec {
    Codec::new(|json, _| {
        let text = string_of(json)?;
        if Identifier::is_valid_path(text) {
            Ok(Tag::String(text.to_string()))
        } else {
            Err(format!(
                "Invalid string to use as a resource path element: {text}"
            ))
        }
    })
}

/// `StringRepresentable.fromEnum`.
pub fn enum_codec(names: &'static [&'static str]) -> Codec {
    Codec::new(move |json, _| {
        let text = string_of(json)?;
        if names.contains(&text) {
            Ok(Tag::String(text.to_string()))
        } else {
            Err(format!("Unknown element name:{text}"))
        }
    })
}

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

/// `Codec.listOf`.
pub fn list(element: Codec) -> Codec {
    Codec::new(move |json, ctx| match json {
        Json::Array(items) => items
            .iter()
            .map(|item| element.parse(item, ctx))
            .collect::<CodecResult<Vec<_>>>()
            .map(Tag::List),
        _ => Err(format!("Not a list: {}", describe(json))),
    })
}

/// `ExtraCodecs.compactListCodec`: a lone element is accepted and a one-element
/// list is written back as the bare element.
pub fn compact_list(element: Codec) -> Codec {
    Codec::new(move |json, ctx| {
        let tag = match json {
            Json::Array(_) => list(element.clone()).parse(json, ctx)?,
            _ => Tag::List(vec![element.parse(json, ctx)?]),
        };
        Ok(match tag {
            Tag::List(mut items) if items.len() == 1 => items.remove(0),
            other => other,
        })
    })
}

/// `Codec.either(first, second)` where the value keeps the branch that parsed.
pub fn either(first: Codec, second: Codec) -> Codec {
    Codec::new(move |json, ctx| match first.parse(json, ctx) {
        Ok(tag) => Ok(tag),
        Err(first_error) => second.parse(json, ctx).map_err(|second_error| {
            format!("Failed to parse either. First: {first_error}; Second: {second_error}")
        }),
    })
}

/// `Codec.unboundedMap(keyCodec, value)` where the value codec depends on the key
/// (`Codec.dispatchedMap`). `key` yields the canonical key string.
pub fn dispatched_map(
    key: Codec,
    value_for_key: impl Fn(&str) -> CodecResult<Codec> + 'static,
) -> Codec {
    Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let mut entries = Vec::with_capacity(object.len());
        for (raw_key, value) in object {
            let key_tag = key.parse(&Json::String(raw_key.clone()), ctx)?;
            let Tag::String(canonical) = key_tag else {
                return Err(format!("Map key {raw_key} did not encode to a string"));
            };
            let value_tag = value_for_key(&canonical)?.parse(value, ctx)?;
            entries.push((canonical, value_tag));
        }
        Ok(Tag::Compound(entries))
    })
}

/// `Codec.unboundedMap(keyCodec, valueCodec)`.
pub fn unbounded_map(key: Codec, value: Codec) -> Codec {
    dispatched_map(key, move |_| Ok(value.clone()))
}

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

/// One `RecordCodecBuilder` group entry.
pub struct Field(Rc<FieldFn>);

type FieldFn =
    dyn Fn(&Map<String, Json>, &CodecContext<'_>, &mut Vec<(String, Tag)>) -> CodecResult<()>;

impl Field {
    fn new(
        f: impl Fn(&Map<String, Json>, &CodecContext<'_>, &mut Vec<(String, Tag)>) -> CodecResult<()>
            + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }
}

fn missing_key(name: &str, object: &Map<String, Json>) -> String {
    format!(
        "No key {name} in MapLike[{}]",
        describe(&Json::Object(object.clone()))
    )
}

/// `codec.fieldOf(name)`.
pub fn req(name: &'static str, codec: Codec) -> Field {
    Field::new(move |object, ctx, out| {
        let value = object.get(name).ok_or_else(|| missing_key(name, object))?;
        out.push((name.to_string(), codec.parse(value, ctx)?));
        Ok(())
    })
}

/// `codec.optionalFieldOf(name)` (an `Optional` component; omitted when empty).
pub fn opt(name: &'static str, codec: Codec) -> Field {
    Field::new(move |object, ctx, out| {
        if let Some(value) = object.get(name) {
            out.push((name.to_string(), codec.parse(value, ctx)?));
        }
        Ok(())
    })
}

/// `codec.lenientOptionalFieldOf(name)`: a value that fails to parse is treated
/// as absent.
pub fn lenient_opt(name: &'static str, codec: Codec) -> Field {
    Field::new(move |object, ctx, out| {
        if let Some(value) = object.get(name) {
            if let Ok(tag) = codec.parse(value, ctx) {
                out.push((name.to_string(), tag));
            }
        }
        Ok(())
    })
}

/// `codec.optionalFieldOf(name, default)`: the encoder omits the field when the
/// value equals the default.
pub fn opt_default(name: &'static str, codec: Codec, default: Json) -> Field {
    Field::new(move |object, ctx, out| {
        if let Some(value) = object.get(name) {
            let tag = codec.parse(value, ctx)?;
            if codec.parse(&default, ctx).ok().as_ref() != Some(&tag) {
                out.push((name.to_string(), tag));
            }
        }
        Ok(())
    })
}

/// `codec.fieldOf(name).orElse(default)`: failures and absence fall back to the
/// default, and the encoder always writes the field.
pub fn or_else(name: &'static str, codec: Codec, default: Json) -> Field {
    Field::new(move |object, ctx, out| {
        let tag = match object.get(name) {
            Some(value) => codec
                .parse(value, ctx)
                .or_else(|_| codec.parse(&default, ctx))?,
            None => codec.parse(&default, ctx)?,
        };
        out.push((name.to_string(), tag));
        Ok(())
    })
}

/// `RecordCodecBuilder.create`.
pub fn record(fields: Vec<Field>) -> Codec {
    Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let mut out = Vec::with_capacity(fields.len());
        for field in &fields {
            (field.0)(object, ctx, &mut out)?;
        }
        Ok(Tag::Compound(out))
    })
}

/// `Codec.dispatch(typeKey, ...)` over a `type` identifier: the variant codec is
/// looked up by `variants(type_id)`; its fields follow the written `type`.
pub fn dispatch(
    type_key: &'static str,
    variants: impl Fn(&Identifier) -> CodecResult<Codec> + 'static,
) -> Codec {
    Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let raw = object
            .get(type_key)
            .ok_or_else(|| missing_key(type_key, object))?;
        let type_id = parse_identifier(raw)?;
        let variant = variants(&type_id)?;
        let mut entries = vec![(type_key.to_string(), Tag::String(type_id.to_string()))];
        match variant.parse(json, ctx)? {
            Tag::Compound(fields) => entries.extend(fields),
            other => return Err(format!("Dispatch variant produced a non-map: {other:?}")),
        }
        Ok(Tag::Compound(entries))
    })
}

// ---------------------------------------------------------------------------
// Registry holders
// ---------------------------------------------------------------------------

fn registry_identifier(registry: &str) -> CodecResult<Identifier> {
    Identifier::parse(registry).map_err(|err| format!("Invalid registry name {registry}: {err}"))
}

/// `RegistryFixedCodec.create(registry)`: a reference by identifier.
pub fn holder_fixed(registry: &'static str) -> Codec {
    Codec::new(move |json, ctx| {
        let registry = registry_identifier(registry)?;
        let id = parse_identifier(json)?;
        ctx.resolve_element(&registry, &id)?;
        Ok(Tag::String(id.to_string()))
    })
}

/// `RegistryFileCodec.create(registry, direct)`: a reference by identifier, or an
/// inline definition when the input is not an identifier.
pub fn holder_file(registry: &'static str, direct: Codec) -> Codec {
    Codec::new(move |json, ctx| {
        let registry = registry_identifier(registry)?;
        match parse_identifier(json) {
            Ok(id) => {
                ctx.resolve_element(&registry, &id)?;
                Ok(Tag::String(id.to_string()))
            }
            Err(_) => direct.parse(json, ctx),
        }
    })
}

/// Parses a `#namespace:path` tag reference.
fn parse_tag_reference(text: &str) -> CodecResult<Identifier> {
    let rest = text
        .strip_prefix('#')
        .ok_or_else(|| format!("Not a tag id: {text}"))?;
    Identifier::parse(rest).map_err(|err| format!("Not a valid resource location: {rest} {err}"))
}

/// `TagKey.hashedCodec(registry)`.
pub fn tag_key_hashed(_registry: &'static str) -> Codec {
    Codec::new(move |json, _| {
        let text = string_of(json)?;
        let id = parse_tag_reference(text)?;
        Ok(Tag::String(format!("#{id}")))
    })
}

/// `RegistryCodecs.homogeneousList(registry)` (`alwaysUseList == false`) or the
/// `alwaysUseList == true` variant: a `#tag`, a list of ids, or a lone id.
pub fn holder_set(registry: &'static str, always_use_list: bool) -> Codec {
    Codec::new(move |json, ctx| {
        let registry_id = registry_identifier(registry)?;
        if let Json::String(text) = json {
            if text.starts_with('#') {
                let tag = parse_tag_reference(text)?;
                ctx.resolve_tag(&registry_id, &tag);
                return Ok(Tag::String(format!("#{tag}")));
            }
        }
        let element = holder_fixed(registry);
        let items = match json {
            Json::Array(_) => list(element).parse(json, ctx)?,
            _ if always_use_list => {
                return Err(format!("Not a list: {}", describe(json)));
            }
            _ => Tag::List(vec![element.parse(json, ctx)?]),
        };
        Ok(match items {
            Tag::List(mut values) if values.len() == 1 && !always_use_list => values.remove(0),
            other => other,
        })
    })
}

// ---------------------------------------------------------------------------
// Generic conversion
// ---------------------------------------------------------------------------

/// Converts arbitrary JSON to NBT without a schema (`JsonOps` -> `NbtOps`
/// `convertTo`). Integers become `IntTag` (or `LongTag` when they do not fit),
/// decimals become `FloatTag` when the literal is the shortest representation of an
/// `f32` and `DoubleTag` otherwise, booleans become `ByteTag`.
///
/// This is the fallback for codecs whose typed model is not ported yet; the
/// client's `NbtOps` decoders accept any numeric tag for any numeric codec, so the
/// result is functionally equivalent but not byte-identical to Java's typed
/// encoding. Callers mark such uses with a `TODO(registry-pipeline-...)`.
pub fn json_to_tag(json: &Json) -> Tag {
    match json {
        Json::Null => Tag::End,
        Json::Bool(b) => Tag::Byte(i8::from(*b)),
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                match i32::try_from(i) {
                    Ok(v) => Tag::Int(v),
                    Err(_) => Tag::Long(i),
                }
            } else {
                let d = n.as_f64().unwrap_or(0.0);
                let f = d as f32;
                // A decimal literal that is the shortest representation of an `f32`
                // (`0.15`) is a float field; anything with more precision (the widened
                // `f32` constant `9.999999747378752E-6`) is a double field.
                if f.to_string() == d.to_string() {
                    Tag::Float(f)
                } else {
                    Tag::Double(d)
                }
            }
        }
        Json::String(s) => Tag::String(s.clone()),
        Json::Array(items) => Tag::List(items.iter().map(json_to_tag).collect()),
        Json::Object(map) => Tag::Compound(
            map.iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), json_to_tag(v)))
                .collect(),
        ),
    }
}

/// Schema-less passthrough codec (see [`json_to_tag`]).
pub fn generic() -> Codec {
    Codec::new(|json, _| Ok(json_to_tag(json)))
}
