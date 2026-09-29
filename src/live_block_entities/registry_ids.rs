//! Static-registry protocol ids the live block-entity effects put on the wire.

use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;

/// `Registry.getId(name)` in the static registry `registry`
/// (`minecraft:block`, `minecraft:sound_event`, ...).
fn protocol_id(registry: &str, name: &str) -> Option<i32> {
    let registries = BuiltinRegistries::vanilla().ok()?;
    let registry = registries.get(&Identifier::parse(registry).ok()?)?;
    let id = registry.id_of(&Identifier::parse(name).ok()?)?;
    i32::try_from(id).ok()
}

/// `BuiltInRegistries.BLOCK.getId(block)`, as written by
/// `ClientboundBlockEventPacket`.
pub fn block_protocol_id(block: &str) -> Option<i32> {
    protocol_id("minecraft:block", block)
}

/// `BuiltInRegistries.SOUND_EVENT.getId(event)`; the packet writes `id + 1`
/// (see `SoundEventHolder::Registered`).
pub fn sound_event_protocol_id(event: &str) -> Option<i32> {
    protocol_id("minecraft:sound_event", event)
}

/// `BuiltInRegistries.ENTITY_TYPE.getId(type)`, as written by
/// `ClientboundAddEntityPacket`.
pub fn entity_type_protocol_id(entity: &str) -> Option<i32> {
    protocol_id("minecraft:entity_type", entity)
}

/// `BuiltInRegistries.PARTICLE_TYPE.getId(type)`, as written by
/// `ParticleTypes.STREAM_CODEC`.
pub fn particle_type_protocol_id(particle: &str) -> Option<i32> {
    protocol_id("minecraft:particle_type", particle)
}
