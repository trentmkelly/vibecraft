use super::*;
use crate::network::play::{ClientboundOpenScreenPacket, CLIENTBOUND_OPEN_SCREEN_PACKET_ID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::network::status) struct BlockMenuOpen {
    menu_type_id: i32,
    title_key: &'static str,
}

pub(in crate::network::status) fn block_menu_open_for_state(
    state: &crate::block_behavior::BlockStateModel,
) -> Option<BlockMenuOpen> {
    block_menu_open_for_block_id(&state.registry_id)
}

fn block_menu_open_for_block_id(block_id: &str) -> Option<BlockMenuOpen> {
    // Java source: net.minecraft.world.inventory.MenuType registers these in
    // declaration order; the registry VarInt in ClientboundOpenScreenPacket is
    // that zero-based registration ID.
    let (menu_type_id, title_key) = match block_id {
        "minecraft:chest" | "minecraft:trapped_chest" => (2, "container.chest"),
        "minecraft:barrel" => (2, "container.barrel"),
        id if id.starts_with("minecraft:") && id.ends_with("_shulker_box") => {
            (20, "container.shulkerBox")
        }
        "minecraft:dispenser" => (6, "container.dispenser"),
        "minecraft:dropper" => (6, "container.dropper"),
        "minecraft:crafter" => (7, "container.crafter"),
        "minecraft:anvil" | "minecraft:chipped_anvil" | "minecraft:damaged_anvil" => {
            (8, "container.repair")
        }
        "minecraft:beacon" => (9, "container.beacon"),
        "minecraft:blast_furnace" => (10, "container.blast_furnace"),
        "minecraft:brewing_stand" => (11, "container.brewing"),
        "minecraft:crafting_table" => (12, "container.crafting"),
        "minecraft:enchanting_table" => (13, "container.enchant"),
        "minecraft:furnace" => (14, "container.furnace"),
        "minecraft:grindstone" => (15, "container.grindstone"),
        "minecraft:hopper" => (16, "container.hopper"),
        "minecraft:loom" => (18, "container.loom"),
        "minecraft:smithing_table" => (21, "container.upgrade"),
        "minecraft:smoker" => (22, "container.smoker"),
        "minecraft:cartography_table" => (23, "container.cartography_table"),
        "minecraft:stonecutter" => (24, "container.stonecutter"),
        _ => return None,
    };
    Some(BlockMenuOpen {
        menu_type_id,
        title_key,
    })
}

fn next_open_container_id(state: &mut PlaySessionState) -> i32 {
    let id = state.next_container_id;
    state.next_container_id += 1;
    if state.next_container_id > 100 {
        state.next_container_id = 1;
    }
    id
}

fn menu_title_component(title_key: &str) -> crate::storage::nbt::Tag {
    crate::storage::nbt::Tag::Compound(vec![(
        "translate".to_string(),
        crate::storage::nbt::Tag::String(title_key.to_string()),
    )])
}

pub(in crate::network::status) fn write_open_block_menu<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    menu: BlockMenuOpen,
    sequence: i32,
) -> io::Result<()> {
    super::write_block_change_ack(writer, compression, sequence)?;
    let container_id = next_open_container_id(state);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_OPEN_SCREEN_PACKET_ID,
        |payload| {
            ClientboundOpenScreenPacket {
                container_id,
                menu_type_id: menu.menu_type_id,
                title: menu_title_component(menu.title_key),
            }
            .write(payload)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_menu_mapping_uses_java_menu_type_registry_ids() {
        let cases = [
            ("minecraft:chest", 2, "container.chest"),
            ("minecraft:trapped_chest", 2, "container.chest"),
            ("minecraft:white_shulker_box", 20, "container.shulkerBox"),
            ("minecraft:furnace", 14, "container.furnace"),
            ("minecraft:blast_furnace", 10, "container.blast_furnace"),
            ("minecraft:smoker", 22, "container.smoker"),
            ("minecraft:crafting_table", 12, "container.crafting"),
            ("minecraft:stonecutter", 24, "container.stonecutter"),
            ("minecraft:cartography_table", 23, "container.cartography_table"),
        ];

        for (block_id, menu_type_id, title_key) in cases {
            assert_eq!(
                block_menu_open_for_block_id(block_id),
                Some(BlockMenuOpen {
                    menu_type_id,
                    title_key
                })
            );
        }
        assert_eq!(block_menu_open_for_block_id("minecraft:stone"), None);
    }

    #[test]
    fn open_container_ids_match_java_counter_range() {
        let mut state = PlaySessionState::default();
        assert_eq!(next_open_container_id(&mut state), 1);
        assert_eq!(next_open_container_id(&mut state), 2);

        state.next_container_id = 100;
        assert_eq!(next_open_container_id(&mut state), 100);
        assert_eq!(next_open_container_id(&mut state), 1);
    }
}
