//! Clientbound packets for item entities changed by block-entity tickers.

use std::io::{self, Write};

use crate::item_catalog::item_protocol_id;
use crate::network::compression::CompressionState;
use crate::network::play::{
    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
};
use crate::network::status::write_framed_packet_with_compression;
use crate::network::varint::write_var_i32;
use crate::network::world_broadcast::WorldPacketBus;

use super::hopper::HopperEntityEffects;

/// `ItemEntity.DATA_ITEM` entity-data index.
const ITEM_DATA_INDEX: u8 = 8;
/// `EntityDataSerializers.ITEM_STACK` id.
const ITEM_STACK_SERIALIZER: i32 = 7;

/// Broadcasts item entities a hopper discarded (`Entity.discard` ->
/// `ClientboundRemoveEntitiesPacket`) or shrank (`ItemEntity.setItem` ->
/// `ClientboundSetEntityDataPacket`).
pub fn publish_item_effects(bus: &WorldPacketBus, effects: &HopperEntityEffects) -> io::Result<()> {
    let mut frames = Vec::new();
    write_item_effects(&mut frames, effects)?;
    bus.publish_frames(&frames)
}

fn write_item_effects(frames: &mut Vec<u8>, effects: &HopperEntityEffects) -> io::Result<()> {
    if !effects.removed.is_empty() {
        write_framed_packet_with_compression(
            frames,
            CompressionState::disabled(),
            CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
            |payload| {
                write_var_i32(payload, effects.removed.len() as i32)?;
                effects
                    .removed
                    .iter()
                    .try_for_each(|id| write_var_i32(payload, *id))
            },
        )?;
    }
    for (entity_id, item, count) in &effects.count_updates {
        let Some(item_id) = item_protocol_id(item) else {
            continue;
        };
        write_framed_packet_with_compression(
            frames,
            CompressionState::disabled(),
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            |payload| {
                write_var_i32(payload, *entity_id)?;
                payload.write_all(&[ITEM_DATA_INDEX])?;
                write_var_i32(payload, ITEM_STACK_SERIALIZER)?;
                write_var_i32(payload, *count)?;
                write_var_i32(payload, item_id)?;
                write_var_i32(payload, 0)?; // component add count
                write_var_i32(payload, 0)?; // component remove count
                payload.write_all(&[0xFF]) // end of entity data
            },
        )?;
    }
    Ok(())
}
