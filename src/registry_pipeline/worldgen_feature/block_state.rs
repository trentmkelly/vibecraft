//! `BlockState.CODEC`, `FluidState.CODEC` and block-registry reference codecs.

use serde_json::Value as Json;

use super::BLOCK_REGISTRY;
use crate::block_states::{block_state_entry, default_state_properties};
use crate::registry_pipeline::codec::{
    describe, holder_set, parse_identifier, tag_key_hashed, Codec, CodecResult,
};
use crate::storage::nbt::Tag;

/// A property of a state definition: its name, allowed values and default value.
struct PropertyDef<'a> {
    name: &'a str,
    values: Vec<&'a str>,
    default: &'a str,
}

/// `StateHolder.codec(ownerCodec, defaultState, stateDefinition)`.
///
/// Dispatches on `Name`; a stateful owner then reads `Properties` leniently
/// (`lenientOptionalFieldOf`), every property with `orElse(default)`, and writes back
/// *all* property values sorted by name (`StateDefinition.propertiesByName` is an
/// `ImmutableSortedMap`). Unknown properties are ignored.
fn state_holder(
    json: &Json,
    owner_kind: &str,
    definition_of: impl Fn(&str) -> Option<Vec<PropertyDef<'static>>>,
) -> CodecResult<Tag> {
    let Json::Object(object) = json else {
        return Err(format!("Not a map: {}", describe(json)));
    };
    let name = object
        .get("Name")
        .ok_or_else(|| format!("No key Name in MapLike[{}]", describe(json)))?;
    let id = parse_identifier(name)?;
    let properties = definition_of(&id.to_string()).ok_or_else(|| {
        format!("Unknown registry key in ResourceKey[minecraft:root / minecraft:{owner_kind}]: {id}")
    })?;
    let mut entries = vec![("Name".to_string(), Tag::String(id.to_string()))];
    if properties.is_empty() {
        return Ok(Tag::Compound(entries));
    }
    let supplied = match object.get("Properties") {
        Some(Json::Object(map)) => Some(map),
        _ => None,
    };
    let mut written: Vec<(String, Tag)> = properties
        .iter()
        .map(|property| {
            let value = supplied
                .and_then(|map| map.get(property.name))
                .and_then(Json::as_str)
                .filter(|candidate| property.values.contains(candidate))
                .unwrap_or(property.default);
            (property.name.to_string(), Tag::String(value.to_string()))
        })
        .collect();
    written.sort_by(|a, b| a.0.cmp(&b.0));
    entries.push(("Properties".to_string(), Tag::Compound(written)));
    Ok(Tag::Compound(entries))
}

/// The state definition of a block from the authoritative block-state table.
fn block_definition(id: &str) -> Option<Vec<PropertyDef<'static>>> {
    let entry = block_state_entry(id)?;
    let defaults = default_state_properties(id)?;
    Some(
        entry
            .properties
            .iter()
            .zip(defaults)
            .map(|(property, (_, default))| PropertyDef {
                name: property.name,
                values: property.values.to_vec(),
                default,
            })
            .collect(),
    )
}

/// `BlockState.CODEC`.
pub fn block_state() -> Codec {
    Codec::new(|json, _| state_holder(json, "block", block_definition))
}

const FALLING: PropertyDef<'static> = PropertyDef {
    name: "falling",
    values: Vec::new(),
    default: "false",
};

/// The state definition of a fluid (`Fluid.createFluidStateDefinition`): `empty` has no
/// properties, source fluids have `falling`, flowing fluids additionally `level`
/// (`FlowingFluid`, `WaterFluid.Flowing`, `LavaFluid.Flowing`; default level 7).
fn fluid_definition(id: &str) -> Option<Vec<PropertyDef<'static>>> {
    let falling = PropertyDef {
        values: vec!["true", "false"],
        ..FALLING
    };
    let level = PropertyDef {
        name: "level",
        values: vec!["1", "2", "3", "4", "5", "6", "7", "8"],
        default: "7",
    };
    match id {
        "minecraft:empty" => Some(Vec::new()),
        "minecraft:water" | "minecraft:lava" => Some(vec![falling]),
        "minecraft:flowing_water" | "minecraft:flowing_lava" => Some(vec![falling, level]),
        _ => None,
    }
}

/// `FluidState.CODEC`.
pub fn fluid_state() -> Codec {
    Codec::new(|json, _| state_holder(json, "fluid", fluid_definition))
}

/// `BuiltInRegistries.BLOCK.byNameCodec()`.
pub fn block_by_name() -> Codec {
    crate::registry_pipeline::codec::holder_fixed(BLOCK_REGISTRY)
}

/// `BuiltInRegistries.BLOCK.byNameCodec()` restricted to `MultifaceSpreadeableBlock`s
/// (`MultifaceGrowthConfiguration.apply`): glow lichen and sculk vein.
pub fn multiface_block() -> Codec {
    block_by_name().validate(|tag| match tag {
        Tag::String(name)
            if block_state_entry(name)
                .is_some_and(|entry| matches!(entry.block_type, "glow_lichen" | "sculk_vein")) =>
        {
            Ok(())
        }
        _ => Err("Growth block should be a multiface spreadeable block".to_string()),
    })
}

/// `RegistryCodecs.homogeneousList(Registries.BLOCK)`.
pub fn block_set() -> Codec {
    holder_set(BLOCK_REGISTRY, false)
}

/// `TagKey.hashedCodec(Registries.BLOCK)`.
pub fn block_tag_hashed() -> Codec {
    tag_key_hashed(BLOCK_REGISTRY)
}

/// `TagKey.codec(Registries.BLOCK)`: the bare (unhashed) tag identifier.
pub fn block_tag() -> Codec {
    crate::registry_pipeline::codec::identifier_codec()
}
