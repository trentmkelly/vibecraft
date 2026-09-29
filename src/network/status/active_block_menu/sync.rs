//! Per-tick `AbstractContainerMenu.broadcastChanges` for menus backed by a
//! block entity, so a furnace open in a player's GUI shows the items and
//! progress the server-side ticker produces.

use super::*;
use crate::network::play::{
    ClientboundContainerSetDataPacket, CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID,
    CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
};

impl ActiveBlockMenu {
    /// Reloads the container slots from the block entity and sends every slot and
    /// `ContainerData` value that changed since the last call (Java
    /// `broadcastChanges` -> `ContainerSynchronizer.sendSlotChange` /
    /// `sendDataChange`). Slot changes advance the menu state id like
    /// `ServerPlayer`'s synchronizer does.
    pub(in crate::network::status) fn broadcast_changes<W: Write>(
        &mut self,
        writer: &mut W,
        compression: CompressionState,
        world_layout: &WorldLayout,
        world_seed: i64,
        chunk_cache: &GeneratedChunkCache,
    ) -> io::Result<()> {
        if !matches!(self.kind, ActiveBlockMenuKind::Persistent { .. }) {
            return Ok(());
        }
        let Some(tag) = chunk_cache.block_entity_nbt_at(world_layout.root(), world_seed, self.pos)
        else {
            return Ok(());
        };
        let mut fresh = vec![ItemStack::empty(); self.slots.len()];
        load_items_from_block_entity_tag(&tag, &mut fresh);
        for (slot, stack) in fresh.into_iter().enumerate() {
            if self.slots[slot] == stack {
                continue;
            }
            self.increment_state_id();
            let packet = ClientboundContainerSetSlotPacket {
                container_id: self.container_id,
                state_id: self.state_id,
                slot: slot as i16,
                item_stack: raw_item_stack_from_item_stack(&stack)?,
            };
            self.slots[slot] = stack;
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
                |payload| packet.write(payload),
            )?;
        }
        self.broadcast_furnace_data(writer, compression, &tag)
    }

    /// `FurnaceMenu`'s four `ContainerData` slots: lit time remaining, lit
    /// duration, cooking progress and cooking total time.
    fn broadcast_furnace_data<W: Write>(
        &mut self,
        writer: &mut W,
        compression: CompressionState,
        tag: &Tag,
    ) -> io::Result<()> {
        let container_id = self.container_id;
        let ActiveBlockMenuKind::Persistent {
            furnace: Some(furnace),
            ..
        } = &mut self.kind
        else {
            return Ok(());
        };
        let entity = AbstractFurnaceBlockEntity::load_additional(furnace.kind, tag);
        let data = [
            entity.lit_time_remaining,
            entity.lit_total_time,
            entity.cooking_time_spent,
            entity.cooking_total_time,
        ]
        .map(|value| value as i16);
        for (id, value) in data.into_iter().enumerate() {
            if furnace.synced_data.is_some_and(|sent| sent[id] == value) {
                continue;
            }
            let packet = ClientboundContainerSetDataPacket {
                container_id,
                id: id as i16,
                value,
            };
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID,
                |payload| packet.write(payload),
            )?;
        }
        furnace.synced_data = Some(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
