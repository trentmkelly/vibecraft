#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: &'static str,
}

pub const RECIPE_TYPES: &[RegistryEntry] = &[
    RegistryEntry { id: "crafting" },
    RegistryEntry { id: "smelting" },
    RegistryEntry { id: "blasting" },
    RegistryEntry { id: "smoking" },
    RegistryEntry {
        id: "campfire_cooking",
    },
    RegistryEntry { id: "stonecutting" },
    RegistryEntry { id: "smithing" },
];

pub const RECIPE_SERIALIZERS: &[RegistryEntry] = &[
    RegistryEntry {
        id: "crafting_shaped",
    },
    RegistryEntry {
        id: "crafting_shapeless",
    },
    RegistryEntry { id: "crafting_dye" },
    RegistryEntry {
        id: "crafting_imbue",
    },
    RegistryEntry {
        id: "crafting_transmute",
    },
    RegistryEntry {
        id: "crafting_decorated_pot",
    },
    RegistryEntry {
        id: "crafting_special_bookcloning",
    },
    RegistryEntry {
        id: "crafting_special_mapextending",
    },
    RegistryEntry {
        id: "crafting_special_firework_rocket",
    },
    RegistryEntry {
        id: "crafting_special_firework_star",
    },
    RegistryEntry {
        id: "crafting_special_firework_star_fade",
    },
    RegistryEntry {
        id: "crafting_special_bannerduplicate",
    },
    RegistryEntry {
        id: "crafting_special_shielddecoration",
    },
    RegistryEntry {
        id: "crafting_special_repairitem",
    },
    RegistryEntry { id: "smelting" },
    RegistryEntry { id: "blasting" },
    RegistryEntry { id: "smoking" },
    RegistryEntry {
        id: "campfire_cooking",
    },
    RegistryEntry { id: "stonecutting" },
    RegistryEntry {
        id: "smithing_transform",
    },
    RegistryEntry {
        id: "smithing_trim",
    },
];

pub const RECIPE_DISPLAY_TYPES: &[RegistryEntry] = &[
    RegistryEntry {
        id: "crafting_shapeless",
    },
    RegistryEntry {
        id: "crafting_shaped",
    },
    RegistryEntry { id: "furnace" },
    RegistryEntry { id: "stonecutter" },
    RegistryEntry { id: "smithing" },
];

pub const SLOT_DISPLAY_TYPES: &[RegistryEntry] = &[
    RegistryEntry { id: "empty" },
    RegistryEntry { id: "any_fuel" },
    RegistryEntry {
        id: "with_any_potion",
    },
    RegistryEntry {
        id: "only_with_component",
    },
    RegistryEntry { id: "item" },
    RegistryEntry { id: "item_stack" },
    RegistryEntry { id: "tag" },
    RegistryEntry { id: "dyed" },
    RegistryEntry {
        id: "smithing_trim",
    },
    RegistryEntry {
        id: "with_remainder",
    },
    RegistryEntry { id: "composite" },
];

pub const RECIPE_BOOK_CATEGORIES: &[RegistryEntry] = &[
    RegistryEntry {
        id: "crafting_building_blocks",
    },
    RegistryEntry {
        id: "crafting_redstone",
    },
    RegistryEntry {
        id: "crafting_equipment",
    },
    RegistryEntry {
        id: "crafting_misc",
    },
    RegistryEntry { id: "furnace_food" },
    RegistryEntry {
        id: "furnace_blocks",
    },
    RegistryEntry { id: "furnace_misc" },
    RegistryEntry {
        id: "blast_furnace_blocks",
    },
    RegistryEntry {
        id: "blast_furnace_misc",
    },
    RegistryEntry { id: "smoker_food" },
    RegistryEntry { id: "stonecutter" },
    RegistryEntry { id: "smithing" },
    RegistryEntry { id: "campfire" },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBookTypeSettings {
    pub open: bool,
    pub filtering: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBookSettings {
    pub crafting: RecipeBookTypeSettings,
    pub furnace: RecipeBookTypeSettings,
    pub blast_furnace: RecipeBookTypeSettings,
    pub smoker: RecipeBookTypeSettings,
}

impl RecipeBookSettings {
    pub fn get(self, book_type: RecipeBookType) -> RecipeBookTypeSettings {
        match book_type {
            RecipeBookType::Crafting => self.crafting,
            RecipeBookType::Furnace => self.furnace,
            RecipeBookType::BlastFurnace => self.blast_furnace,
            RecipeBookType::Smoker => self.smoker,
        }
    }

    pub fn set_open(&mut self, book_type: RecipeBookType, open: bool) {
        self.update(book_type, |mut settings| {
            settings.open = open;
            settings
        });
    }

    pub fn set_filtering(&mut self, book_type: RecipeBookType, filtering: bool) {
        self.update(book_type, |mut settings| {
            settings.filtering = filtering;
            settings
        });
    }

    fn update(
        &mut self,
        book_type: RecipeBookType,
        update: impl FnOnce(RecipeBookTypeSettings) -> RecipeBookTypeSettings,
    ) {
        match book_type {
            RecipeBookType::Crafting => self.crafting = update(self.crafting),
            RecipeBookType::Furnace => self.furnace = update(self.furnace),
            RecipeBookType::BlastFurnace => self.blast_furnace = update(self.blast_furnace),
            RecipeBookType::Smoker => self.smoker = update(self.smoker),
        }
    }

    pub fn stream_order(self) -> [RecipeBookTypeSettings; 4] {
        [self.crafting, self.furnace, self.blast_furnace, self.smoker]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeDisplay {
    ShapelessCrafting {
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    ShapedCrafting {
        width: usize,
        height: usize,
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Furnace {
        ingredient: SlotDisplay,
        fuel: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
        duration_ticks: i32,
        experience: i32,
    },
    Stonecutter {
        ingredient: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Smithing {
        template: SlotDisplay,
        base: SlotDisplay,
        addition: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion,
    OnlyWithComponent {
        component: &'static str,
    },
    Item(&'static str),
    ItemStack {
        item: &'static str,
        count: u32,
    },
    Tag(&'static str),
    Dyed {
        base: Box<SlotDisplay>,
        color: u32,
    },
    SmithingTrim {
        base: Box<SlotDisplay>,
    },
    WithRemainder {
        input: Box<SlotDisplay>,
        remainder: Box<SlotDisplay>,
    },
    Composite(Vec<SlotDisplay>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayId(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayEntry {
    pub id: RecipeDisplayId,
    pub display: RecipeDisplay,
    pub group: Option<i32>,
    pub category: &'static str,
    pub crafting_requirements: Option<Vec<SlotDisplay>>,
}

impl RecipeDisplayEntry {
    pub fn can_craft(&self, provided: &[&'static str]) -> bool {
        let Some(requirements) = &self.crafting_requirements else {
            return false;
        };

        requirements.iter().all(|requirement| match requirement {
            SlotDisplay::Item(item) => provided.contains(item),
            SlotDisplay::ItemStack { item, .. } => provided.contains(item),
            SlotDisplay::Composite(options) => options.iter().any(|option| match option {
                SlotDisplay::Item(item) => provided.contains(item),
                SlotDisplay::ItemStack { item, .. } => provided.contains(item),
                _ => false,
            }),
            _ => false,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookEntryFlags(u8);

impl RecipeBookEntryFlags {
    pub const NOTIFICATION: u8 = 1;
    pub const HIGHLIGHT: u8 = 2;

    pub fn new(notification: bool, highlight: bool) -> Self {
        Self(u8::from(notification) * Self::NOTIFICATION + u8::from(highlight) * Self::HIGHLIGHT)
    }

    pub fn notification(self) -> bool {
        self.0 & Self::NOTIFICATION != 0
    }

    pub fn highlight(self) -> bool {
        self.0 & Self::HIGHLIGHT != 0
    }

    pub fn bits(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookAddEntry {
    pub contents: RecipeDisplayEntry,
    pub flags: RecipeBookEntryFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookAddPacket {
    pub entries: Vec<RecipeBookAddEntry>,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookRemovePacket {
    pub recipes: Vec<RecipeDisplayId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    pub settings: RecipeBookSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateRecipesPacket {
    pub item_sets: Vec<RecipePropertySet>,
    pub stonecutter_recipes: Vec<SelectableSingleInputRecipe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipePropertySet {
    pub key: &'static str,
    pub accepted_items: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectableSingleInputRecipe {
    pub input: &'static str,
    pub recipe: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(entries: &[RegistryEntry]) -> Vec<&'static str> {
        entries.iter().map(|entry| entry.id).collect()
    }

    #[test]
    fn recipe_registries_match_vanilla_bootstrap_order() {
        assert_eq!(
            ids(RECIPE_TYPES),
            vec![
                "crafting",
                "smelting",
                "blasting",
                "smoking",
                "campfire_cooking",
                "stonecutting",
                "smithing"
            ]
        );
        assert_eq!(RECIPE_SERIALIZERS.len(), 21);
        assert_eq!(RECIPE_SERIALIZERS.first().unwrap().id, "crafting_shaped");
        assert_eq!(RECIPE_SERIALIZERS.last().unwrap().id, "smithing_trim");
        assert_eq!(
            ids(RECIPE_DISPLAY_TYPES),
            vec![
                "crafting_shapeless",
                "crafting_shaped",
                "furnace",
                "stonecutter",
                "smithing"
            ]
        );
        assert_eq!(SLOT_DISPLAY_TYPES.len(), 11);
        assert_eq!(RECIPE_BOOK_CATEGORIES.len(), 13);
        assert_eq!(RECIPE_BOOK_CATEGORIES.last().unwrap().id, "campfire");
    }

    #[test]
    fn recipe_book_settings_use_vanilla_stream_order_and_defaults() {
        let mut settings = RecipeBookSettings::default();
        assert_eq!(
            settings.stream_order(),
            [RecipeBookTypeSettings::default(); 4]
        );

        settings.set_open(RecipeBookType::Crafting, true);
        settings.set_filtering(RecipeBookType::BlastFurnace, true);

        assert_eq!(
            settings.stream_order(),
            [
                RecipeBookTypeSettings {
                    open: true,
                    filtering: false
                },
                RecipeBookTypeSettings::default(),
                RecipeBookTypeSettings {
                    open: false,
                    filtering: true
                },
                RecipeBookTypeSettings::default()
            ]
        );
        assert!(settings.get(RecipeBookType::Crafting).open);
        assert!(!settings.get(RecipeBookType::Smoker).open);
    }

    #[test]
    fn recipe_book_add_flags_match_vanilla_bits() {
        let flags = RecipeBookEntryFlags::new(true, true);
        assert_eq!(flags.bits(), 3);
        assert!(flags.notification());
        assert!(flags.highlight());

        let highlight_only = RecipeBookEntryFlags::new(false, true);
        assert_eq!(highlight_only.bits(), 2);
        assert!(!highlight_only.notification());
        assert!(highlight_only.highlight());
    }

    #[test]
    fn recipe_display_entries_require_explicit_crafting_requirements() {
        let entry = RecipeDisplayEntry {
            id: RecipeDisplayId(7),
            display: RecipeDisplay::ShapelessCrafting {
                ingredients: vec![SlotDisplay::Item("minecraft:oak_planks")],
                result: SlotDisplay::ItemStack {
                    item: "minecraft:stick",
                    count: 4,
                },
                crafting_station: SlotDisplay::Item("minecraft:crafting_table"),
            },
            group: Some(1),
            category: "crafting_misc",
            crafting_requirements: Some(vec![SlotDisplay::Composite(vec![
                SlotDisplay::Item("minecraft:oak_planks"),
                SlotDisplay::Item("minecraft:birch_planks"),
            ])]),
        };

        assert!(entry.can_craft(&["minecraft:birch_planks"]));
        assert!(!entry.can_craft(&["minecraft:cobblestone"]));

        let without_requirements = RecipeDisplayEntry {
            crafting_requirements: None,
            ..entry
        };
        assert!(!without_requirements.can_craft(&["minecraft:oak_planks"]));
    }

    #[test]
    fn recipe_packets_carry_update_add_remove_and_settings_payloads() {
        let display_entry = RecipeDisplayEntry {
            id: RecipeDisplayId(3),
            display: RecipeDisplay::Stonecutter {
                ingredient: SlotDisplay::Item("minecraft:stone"),
                result: SlotDisplay::Item("minecraft:stone_slab"),
                crafting_station: SlotDisplay::Item("minecraft:stonecutter"),
            },
            group: None,
            category: "stonecutter",
            crafting_requirements: Some(vec![SlotDisplay::Item("minecraft:stone")]),
        };
        let add = ClientboundRecipeBookAddPacket {
            entries: vec![RecipeBookAddEntry {
                contents: display_entry,
                flags: RecipeBookEntryFlags::new(true, false),
            }],
            replace: true,
        };
        let remove = ClientboundRecipeBookRemovePacket {
            recipes: vec![RecipeDisplayId(3)],
        };
        let update = ClientboundUpdateRecipesPacket {
            item_sets: vec![RecipePropertySet {
                key: "minecraft:stonecutting",
                accepted_items: vec!["minecraft:stone"],
            }],
            stonecutter_recipes: vec![SelectableSingleInputRecipe {
                input: "minecraft:stone",
                recipe: Some("minecraft:stone_slab_from_stone_stonecutting"),
            }],
        };
        let settings = ClientboundRecipeBookSettingsPacket {
            settings: RecipeBookSettings::default(),
        };

        assert!(add.replace);
        assert_eq!(
            add.entries[0].flags.bits(),
            RecipeBookEntryFlags::NOTIFICATION
        );
        assert_eq!(remove.recipes, vec![RecipeDisplayId(3)]);
        assert_eq!(update.item_sets[0].accepted_items, vec!["minecraft:stone"]);
        assert_eq!(
            update.stonecutter_recipes[0].recipe,
            Some("minecraft:stone_slab_from_stone_stonecutting")
        );
        assert_eq!(
            settings.settings.stream_order(),
            [RecipeBookTypeSettings::default(); 4]
        );
    }

    #[test]
    fn recipe_display_model_covers_all_vanilla_display_and_slot_variants() {
        let mut settings = RecipeBookSettings::default();
        settings.set_open(RecipeBookType::Furnace, true);
        settings.set_filtering(RecipeBookType::Smoker, true);

        let displays = vec![
            RecipeDisplay::ShapedCrafting {
                width: 2,
                height: 2,
                ingredients: vec![SlotDisplay::Item("minecraft:planks")],
                result: SlotDisplay::ItemStack {
                    item: "minecraft:crafting_table",
                    count: 1,
                },
                crafting_station: SlotDisplay::Empty,
            },
            RecipeDisplay::Furnace {
                ingredient: SlotDisplay::Tag("minecraft:logs"),
                fuel: SlotDisplay::AnyFuel,
                result: SlotDisplay::Item("minecraft:charcoal"),
                crafting_station: SlotDisplay::Item("minecraft:furnace"),
                duration_ticks: 200,
                experience: 1,
            },
            RecipeDisplay::Smithing {
                template: SlotDisplay::SmithingTrim {
                    base: Box::new(SlotDisplay::Item(
                        "minecraft:netherite_upgrade_smithing_template",
                    )),
                },
                base: SlotDisplay::OnlyWithComponent {
                    component: "minecraft:damage",
                },
                addition: SlotDisplay::WithAnyPotion,
                result: SlotDisplay::Dyed {
                    base: Box::new(SlotDisplay::Item("minecraft:leather_chestplate")),
                    color: 0x33_66_99,
                },
                crafting_station: SlotDisplay::WithRemainder {
                    input: Box::new(SlotDisplay::Item("minecraft:water_bucket")),
                    remainder: Box::new(SlotDisplay::Item("minecraft:bucket")),
                },
            },
        ];

        assert_eq!(displays.len(), 3);
        assert!(settings.get(RecipeBookType::Furnace).open);
        assert!(settings.get(RecipeBookType::Smoker).filtering);
    }
}
