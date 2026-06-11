use super::*;
use crate::network::play::{ClientboundOpenScreenPacket, CLIENTBOUND_OPEN_SCREEN_PACKET_ID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::network::status) struct BlockMenuOpen {
    pub menu_type_id: i32,
    pub title_key: &'static str,
    pub live_kind: LiveBlockMenuKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::network::status) enum LiveBlockMenuKind {
    Generic9x3 { block_entity_id: &'static str },
    Furnace { block_entity_id: &'static str },
    Crafting,
    Ephemeral {
        slot_count: usize,
        result_slot: Option<usize>,
    },
}

pub(in crate::network::status) fn block_menu_open_for_state(
    state: &crate::block_behavior::BlockStateModel,
) -> Option<BlockMenuOpen> {
    block_menu_open_for_block_id(&state.registry_id)
}

#[expect(
    clippy::too_many_lines,
    reason = "menu registry mapping stays grouped for Java registration-order review"
)]
fn block_menu_open_for_block_id(block_id: &str) -> Option<BlockMenuOpen> {
    // Java source: net.minecraft.world.inventory.MenuType registers these in
    // declaration order; the registry VarInt in ClientboundOpenScreenPacket is
    // that zero-based registration ID.
    let (menu_type_id, title_key, live_kind) = match block_id {
        "minecraft:chest" => (
            2,
            "container.chest",
            LiveBlockMenuKind::Generic9x3 {
                block_entity_id: "minecraft:chest",
            },
        ),
        "minecraft:trapped_chest" => (
            2,
            "container.chest",
            LiveBlockMenuKind::Generic9x3 {
                block_entity_id: "minecraft:trapped_chest",
            },
        ),
        "minecraft:barrel" => (
            2,
            "container.barrel",
            LiveBlockMenuKind::Generic9x3 {
                block_entity_id: "minecraft:barrel",
            },
        ),
        id if id.starts_with("minecraft:") && id.ends_with("_shulker_box") => {
            (
                20,
                "container.shulkerBox",
                LiveBlockMenuKind::Generic9x3 {
                    block_entity_id: "minecraft:shulker_box",
                },
            )
        }
        "minecraft:dispenser" => (
            6,
            "container.dispenser",
            LiveBlockMenuKind::Generic9x3 {
                block_entity_id: "minecraft:dispenser",
            },
        ),
        "minecraft:dropper" => (
            6,
            "container.dropper",
            LiveBlockMenuKind::Generic9x3 {
                block_entity_id: "minecraft:dropper",
            },
        ),
        "minecraft:crafter" => (
            7,
            "container.crafter",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 9,
                result_slot: None,
            },
        ),
        "minecraft:anvil" | "minecraft:chipped_anvil" | "minecraft:damaged_anvil" => {
            (
                8,
                "container.repair",
                LiveBlockMenuKind::Ephemeral {
                    slot_count: 3,
                    result_slot: Some(2),
                },
            )
        }
        "minecraft:beacon" => (
            9,
            "container.beacon",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 1,
                result_slot: None,
            },
        ),
        "minecraft:blast_furnace" => (
            10,
            "container.blast_furnace",
            LiveBlockMenuKind::Furnace {
                block_entity_id: "minecraft:blast_furnace",
            },
        ),
        "minecraft:brewing_stand" => (
            11,
            "container.brewing",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 5,
                result_slot: None,
            },
        ),
        "minecraft:crafting_table" => (
            12,
            "container.crafting",
            LiveBlockMenuKind::Crafting,
        ),
        "minecraft:enchanting_table" => (
            13,
            "container.enchant",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 2,
                result_slot: None,
            },
        ),
        "minecraft:furnace" => (
            14,
            "container.furnace",
            LiveBlockMenuKind::Furnace {
                block_entity_id: "minecraft:furnace",
            },
        ),
        "minecraft:grindstone" => (
            15,
            "container.grindstone",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 3,
                result_slot: Some(2),
            },
        ),
        "minecraft:hopper" => (
            16,
            "container.hopper",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 5,
                result_slot: None,
            },
        ),
        "minecraft:loom" => (
            18,
            "container.loom",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 4,
                result_slot: Some(3),
            },
        ),
        "minecraft:smithing_table" => (
            21,
            "container.upgrade",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 4,
                result_slot: Some(3),
            },
        ),
        "minecraft:smoker" => (
            22,
            "container.smoker",
            LiveBlockMenuKind::Furnace {
                block_entity_id: "minecraft:smoker",
            },
        ),
        "minecraft:cartography_table" => (
            23,
            "container.cartography_table",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 3,
                result_slot: Some(2),
            },
        ),
        "minecraft:stonecutter" => (
            24,
            "container.stonecutter",
            LiveBlockMenuKind::Ephemeral {
                slot_count: 2,
                result_slot: Some(1),
            },
        ),
        _ => return None,
    };
    Some(BlockMenuOpen {
        menu_type_id,
        title_key,
        live_kind,
    })
}

fn menu_title_component(title_key: &str) -> crate::storage::nbt::Tag {
    crate::storage::nbt::Tag::Compound(vec![(
        "translate".to_string(),
        crate::storage::nbt::Tag::String(title_key.to_string()),
    )])
}

pub(in crate::network::status) fn next_open_container_id(state: &mut PlaySessionState) -> i32 {
    let id = state.next_container_id;
    state.next_container_id += 1;
    if state.next_container_id > 100 {
        state.next_container_id = 1;
    }
    id
}

pub(in crate::network::status) fn write_open_block_menu<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    container_id: i32,
    menu: BlockMenuOpen,
    sequence: i32,
) -> io::Result<()> {
    super::write_block_change_ack(writer, compression, sequence)?;
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
            let menu = block_menu_open_for_block_id(block_id).unwrap();
            assert_eq!(menu.menu_type_id, menu_type_id);
            assert_eq!(menu.title_key, title_key);
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
