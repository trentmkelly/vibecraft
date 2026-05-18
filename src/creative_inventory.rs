#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeTabRow {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeTabType {
    Category,
    Inventory,
    Hotbar,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabVisibility {
    ParentAndSearchTabs,
    ParentTabOnly,
    SearchTabOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreativeModeTabDef {
    pub key: &'static str,
    pub row: CreativeTabRow,
    pub column: u8,
    pub tab_type: CreativeTabType,
    pub background: &'static str,
    pub can_scroll: bool,
    pub show_title: bool,
    pub aligned_right: bool,
}

pub const DEFAULT_BACKGROUND: &str =
    "minecraft:textures/gui/container/creative_inventory/tab_items.png";
pub const SEARCH_BACKGROUND: &str =
    "minecraft:textures/gui/container/creative_inventory/tab_item_search.png";
pub const INVENTORY_BACKGROUND: &str =
    "minecraft:textures/gui/container/creative_inventory/tab_inventory.png";

pub const CREATIVE_MODE_TABS_COUNT_26_1_2: usize = 14;

pub const CREATIVE_MODE_TABS: &[CreativeModeTabDef] = &[
    category("building_blocks", CreativeTabRow::Top, 0),
    category("colored_blocks", CreativeTabRow::Top, 1),
    category("natural_blocks", CreativeTabRow::Top, 2),
    category("functional_blocks", CreativeTabRow::Top, 3),
    category("redstone_blocks", CreativeTabRow::Top, 4),
    CreativeModeTabDef {
        key: "hotbar",
        row: CreativeTabRow::Top,
        column: 5,
        tab_type: CreativeTabType::Hotbar,
        background: DEFAULT_BACKGROUND,
        can_scroll: true,
        show_title: true,
        aligned_right: false,
    },
    CreativeModeTabDef {
        key: "search",
        row: CreativeTabRow::Top,
        column: 6,
        tab_type: CreativeTabType::Search,
        background: SEARCH_BACKGROUND,
        can_scroll: true,
        show_title: true,
        aligned_right: false,
    },
    category("tools_and_utilities", CreativeTabRow::Bottom, 0),
    category("combat", CreativeTabRow::Bottom, 1),
    category("food_and_drinks", CreativeTabRow::Bottom, 2),
    category("ingredients", CreativeTabRow::Bottom, 3),
    category("spawn_eggs", CreativeTabRow::Bottom, 4),
    category("op_blocks", CreativeTabRow::Bottom, 5),
    CreativeModeTabDef {
        key: "inventory",
        row: CreativeTabRow::Bottom,
        column: 6,
        tab_type: CreativeTabType::Inventory,
        background: INVENTORY_BACKGROUND,
        can_scroll: true,
        show_title: true,
        aligned_right: false,
    },
];

const fn category(key: &'static str, row: CreativeTabRow, column: u8) -> CreativeModeTabDef {
    CreativeModeTabDef {
        key,
        row,
        column,
        tab_type: CreativeTabType::Category,
        background: DEFAULT_BACKGROUND,
        can_scroll: true,
        show_title: true,
        aligned_right: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreativeStack {
    pub item: &'static str,
    pub count: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreativeAcceptError {
    StackCountNotOne,
    DuplicateParentStack,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreativeTabContents {
    pub parent: Vec<&'static str>,
    pub search: Vec<&'static str>,
}

impl CreativeTabContents {
    pub fn accept(
        &mut self,
        stack: CreativeStack,
        visibility: TabVisibility,
    ) -> Result<(), CreativeAcceptError> {
        if stack.count != 1 {
            return Err(CreativeAcceptError::StackCountNotOne);
        }
        if !stack.enabled {
            return Ok(());
        }
        if visibility != TabVisibility::SearchTabOnly && self.parent.contains(&stack.item) {
            return Err(CreativeAcceptError::DuplicateParentStack);
        }

        match visibility {
            TabVisibility::ParentAndSearchTabs => {
                self.parent.push(stack.item);
                if !self.search.contains(&stack.item) {
                    self.search.push(stack.item);
                }
            }
            TabVisibility::ParentTabOnly => self.parent.push(stack.item),
            TabVisibility::SearchTabOnly => {
                if !self.search.contains(&stack.item) {
                    self.search.push(stack.item);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreativeDisplayParameters {
    pub enabled_features_hash: u64,
    pub has_permissions: bool,
    pub holders_generation: u64,
}

impl CreativeDisplayParameters {
    pub fn needs_update(self, other: Self) -> bool {
        self.enabled_features_hash != other.enabled_features_hash
            || self.has_permissions != other.has_permissions
            || self.holders_generation != other.holders_generation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCreativeModeSlotPacket {
    pub slot_num: i16,
    pub item: Option<CreativeStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreativeSlotAction {
    SetInventorySlot { slot: i16, item: &'static str },
    DropItem { item: &'static str },
    ClearInventorySlot { slot: i16 },
    Reject,
}

pub fn handle_creative_slot_packet(
    packet: ServerboundSetCreativeModeSlotPacket,
    player_is_creative: bool,
) -> CreativeSlotAction {
    if !player_is_creative {
        return CreativeSlotAction::Reject;
    }

    match (packet.slot_num, packet.item) {
        (slot, Some(stack)) if (0..=45).contains(&slot) && stack.count > 0 => {
            CreativeSlotAction::SetInventorySlot {
                slot,
                item: stack.item,
            }
        }
        (slot, None) if (0..=45).contains(&slot) => CreativeSlotAction::ClearInventorySlot { slot },
        (-1, Some(stack)) if stack.count > 0 => CreativeSlotAction::DropItem { item: stack.item },
        _ => CreativeSlotAction::Reject,
    }
}

pub fn creative_tab(key: &str) -> Option<&'static CreativeModeTabDef> {
    CREATIVE_MODE_TABS.iter().find(|tab| tab.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creative_mode_tab_manifest_matches_vanilla_tab_keys_and_special_types() {
        let keys: Vec<_> = CREATIVE_MODE_TABS.iter().map(|tab| tab.key).collect();
        assert_eq!(CREATIVE_MODE_TABS.len(), CREATIVE_MODE_TABS_COUNT_26_1_2);
        assert_eq!(
            keys,
            vec![
                "building_blocks",
                "colored_blocks",
                "natural_blocks",
                "functional_blocks",
                "redstone_blocks",
                "hotbar",
                "search",
                "tools_and_utilities",
                "combat",
                "food_and_drinks",
                "ingredients",
                "spawn_eggs",
                "op_blocks",
                "inventory"
            ]
        );
        assert_eq!(
            creative_tab("hotbar").unwrap().tab_type,
            CreativeTabType::Hotbar
        );
        assert_eq!(
            creative_tab("search").unwrap().background,
            SEARCH_BACKGROUND
        );
        assert_eq!(
            creative_tab("inventory").unwrap().tab_type,
            CreativeTabType::Inventory
        );
        assert_eq!(
            creative_tab("inventory").unwrap().background,
            INVENTORY_BACKGROUND
        );
    }

    #[test]
    fn creative_tab_contents_enforce_count_enabled_features_visibility_and_duplicates() {
        let mut contents = CreativeTabContents::default();
        contents
            .accept(
                CreativeStack {
                    item: "minecraft:stone",
                    count: 1,
                    enabled: true,
                },
                TabVisibility::ParentAndSearchTabs,
            )
            .unwrap();
        contents
            .accept(
                CreativeStack {
                    item: "minecraft:command_block",
                    count: 1,
                    enabled: true,
                },
                TabVisibility::SearchTabOnly,
            )
            .unwrap();
        contents
            .accept(
                CreativeStack {
                    item: "minecraft:disabled_future_item",
                    count: 1,
                    enabled: false,
                },
                TabVisibility::ParentAndSearchTabs,
            )
            .unwrap();

        assert_eq!(contents.parent, vec!["minecraft:stone"]);
        assert_eq!(
            contents.search,
            vec!["minecraft:stone", "minecraft:command_block"]
        );
        assert_eq!(
            contents.accept(
                CreativeStack {
                    item: "minecraft:stone",
                    count: 1,
                    enabled: true
                },
                TabVisibility::ParentTabOnly
            ),
            Err(CreativeAcceptError::DuplicateParentStack)
        );
        assert_eq!(
            contents.accept(
                CreativeStack {
                    item: "minecraft:dirt",
                    count: 2,
                    enabled: true
                },
                TabVisibility::ParentAndSearchTabs
            ),
            Err(CreativeAcceptError::StackCountNotOne)
        );
    }

    #[test]
    fn creative_display_parameters_require_rebuild_on_feature_permission_or_holder_change() {
        let base = CreativeDisplayParameters {
            enabled_features_hash: 1,
            has_permissions: false,
            holders_generation: 7,
        };
        assert!(!base.needs_update(base));
        assert!(base.needs_update(CreativeDisplayParameters {
            enabled_features_hash: 2,
            ..base
        }));
        assert!(base.needs_update(CreativeDisplayParameters {
            has_permissions: true,
            ..base
        }));
        assert!(base.needs_update(CreativeDisplayParameters {
            holders_generation: 8,
            ..base
        }));
    }

    #[test]
    fn set_creative_mode_slot_packet_validates_creative_mode_slots_and_drops() {
        let stack = CreativeStack {
            item: "minecraft:diamond",
            count: 64,
            enabled: true,
        };
        assert_eq!(
            handle_creative_slot_packet(
                ServerboundSetCreativeModeSlotPacket {
                    slot_num: 36,
                    item: Some(stack.clone())
                },
                true
            ),
            CreativeSlotAction::SetInventorySlot {
                slot: 36,
                item: "minecraft:diamond"
            }
        );
        assert_eq!(
            handle_creative_slot_packet(
                ServerboundSetCreativeModeSlotPacket {
                    slot_num: 36,
                    item: None
                },
                true
            ),
            CreativeSlotAction::ClearInventorySlot { slot: 36 }
        );
        assert_eq!(
            handle_creative_slot_packet(
                ServerboundSetCreativeModeSlotPacket {
                    slot_num: -1,
                    item: Some(stack)
                },
                true
            ),
            CreativeSlotAction::DropItem {
                item: "minecraft:diamond"
            }
        );
        assert_eq!(
            handle_creative_slot_packet(
                ServerboundSetCreativeModeSlotPacket {
                    slot_num: 46,
                    item: Some(CreativeStack {
                        item: "minecraft:stone",
                        count: 1,
                        enabled: true
                    })
                },
                true
            ),
            CreativeSlotAction::Reject
        );
        assert_eq!(
            handle_creative_slot_packet(
                ServerboundSetCreativeModeSlotPacket {
                    slot_num: 0,
                    item: Some(CreativeStack {
                        item: "minecraft:stone",
                        count: 1,
                        enabled: true
                    })
                },
                false
            ),
            CreativeSlotAction::Reject
        );
    }
}
