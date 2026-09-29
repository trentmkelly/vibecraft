//! `broadcastChanges` for block-entity backed menus (Java
//! `AbstractContainerMenu.broadcastChanges`, `FurnaceMenu` container data).

use super::*;
use crate::network::varint::read_var_i32;
use crate::storage::chunk::LevelChunk;
use crate::storage::region::ChunkPos;

/// Splits a buffer of uncompressed frames into their packet ids.
fn packet_ids(mut frames: &[u8]) -> Vec<i32> {
    let mut ids = Vec::new();
    while !frames.is_empty() {
        let length = read_var_i32(&mut frames).unwrap() as usize;
        let (payload, rest) = frames.split_at(length);
        ids.push(read_var_i32(&mut &payload[..]).unwrap());
        frames = rest;
    }
    ids
}

#[test]
fn furnace_menu_broadcasts_data_once_then_only_slot_and_data_changes() {
    let pos = crate::block_update::BlockPos { x: 1, y: 64, z: 2 };
    let root = std::env::temp_dir().join("vibecraft-furnace-menu-sync");
    let layout = WorldLayout::new(&root);
    let cache = GeneratedChunkCache::default();
    cache.chunks.lock().unwrap().insert(
        ChunkPos { x: 0, z: 0 },
        Arc::new(LevelChunk::empty(ChunkPos { x: 0, z: 0 })),
    );
    cache.set_block(&root, 0, pos, "minecraft:furnace[facing=north,lit=false]");

    let mut menu = ActiveBlockMenu::open(
        9,
        pos,
        LiveBlockMenuKind::Furnace {
            block_entity_id: "minecraft:furnace",
        },
        &layout,
        0,
        &cache,
        &RecipeMap::default(),
    );
    let mut out = Vec::new();
    menu.broadcast_changes(&mut out, CompressionState::disabled(), &layout, 0, &cache)
        .unwrap();
    // First call: the four `ContainerData` slots, no item changes.
    assert_eq!(
        packet_ids(&out),
        vec![CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID; 4]
    );

    // Nothing changed: nothing is sent.
    out.clear();
    menu.broadcast_changes(&mut out, CompressionState::disabled(), &layout, 0, &cache)
        .unwrap();
    assert!(out.is_empty());

    // The ticker consumes fuel: the fuel slot and the lit data slots change.
    let mut furnace = AbstractFurnaceBlockEntity::furnace();
    furnace.items[1] = Some(PotItemStack {
        item_id: "minecraft:coal".to_string(),
        count: 4,
    });
    furnace.lit_time_remaining = 1600;
    furnace.lit_total_time = 1600;
    let existing = cache.block_entity_nbt_at(&root, 0, pos).unwrap();
    let Tag::Compound(mut fields) = existing else {
        panic!("compound");
    };
    fields.retain(|(key, _)| {
        !["Items", "lit_time_remaining", "lit_total_time"].contains(&key.as_str())
    });
    let Tag::Compound(own) = furnace.save_additional() else {
        panic!("compound");
    };
    fields.extend(own.into_iter().filter(|(key, _)| {
        ["Items", "lit_time_remaining", "lit_total_time"].contains(&key.as_str())
    }));
    cache.set_block_entity_nbt(&root, 0, pos, Tag::Compound(fields));

    let state_before = menu.state_id();
    out.clear();
    menu.broadcast_changes(&mut out, CompressionState::disabled(), &layout, 0, &cache)
        .unwrap();
    assert_eq!(
        packet_ids(&out),
        vec![
            CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
            CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID,
            CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID,
        ]
    );
    assert_eq!(menu.state_id(), state_before + 1);
    assert_eq!(menu.slots[1], ItemStack::new("minecraft:coal", 4));
}
