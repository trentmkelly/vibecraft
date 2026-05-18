#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: &'static str,
}

// Source: decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeType.java
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

// Source: decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeSerializers.java
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAmount {
    pub item: &'static str,
    pub count: u32,
}

impl ItemAmount {
    pub const fn one(item: &'static str) -> Self {
        Self { item, count: 1 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientSpec {
    Item(&'static str),
    AnyOf(Vec<&'static str>),
}

impl IngredientSpec {
    pub fn matches(&self, item: &'static str) -> bool {
        match self {
            IngredientSpec::Item(expected) => *expected == item,
            IngredientSpec::AnyOf(items) => items.contains(&item),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookingKind {
    Smelting,
    Blasting,
    Smoking,
    CampfireCooking,
}

impl CookingKind {
    pub fn default_cooking_time(self) -> i32 {
        match self {
            CookingKind::Smelting => 200,
            CookingKind::Blasting | CookingKind::Smoking => 100,
            CookingKind::CampfireCooking => 600,
        }
    }

    pub fn serializer(self) -> &'static str {
        match self {
            CookingKind::Smelting => "smelting",
            CookingKind::Blasting => "blasting",
            CookingKind::Smoking => "smoking",
            CookingKind::CampfireCooking => "campfire_cooking",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRecipeKind {
    Transmute,
    MapCloning,
    MapExtending,
    BannerDuplicate,
    ShieldDecoration,
    FireworkRocket,
    FireworkStar,
    FireworkStarFade,
    SuspiciousStew,
    BookCloning,
    RepairItem,
    DyedItem,
    DecoratedPot,
    Imbue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeKind {
    Shaped {
        width: usize,
        height: usize,
        pattern: Vec<Option<IngredientSpec>>,
        result: ItemAmount,
    },
    Shapeless {
        ingredients: Vec<IngredientSpec>,
        result: ItemAmount,
    },
    Cooking {
        kind: CookingKind,
        ingredient: IngredientSpec,
        result: ItemAmount,
        experience_millis: i32,
        cooking_time: Option<i32>,
    },
    Stonecutting {
        ingredient: IngredientSpec,
        result: ItemAmount,
    },
    SmithingTransform {
        template: IngredientSpec,
        base: IngredientSpec,
        addition: IngredientSpec,
        result: ItemAmount,
    },
    SmithingTrim {
        template: IngredientSpec,
        base: IngredientSpec,
        addition: IngredientSpec,
    },
    Special {
        kind: SpecialRecipeKind,
        result_hint: Option<ItemAmount>,
    },
}

impl RecipeKind {
    pub fn serializer(&self) -> &'static str {
        match self {
            RecipeKind::Shaped { .. } => "crafting_shaped",
            RecipeKind::Shapeless { .. } => "crafting_shapeless",
            RecipeKind::Cooking { kind, .. } => kind.serializer(),
            RecipeKind::Stonecutting { .. } => "stonecutting",
            RecipeKind::SmithingTransform { .. } => "smithing_transform",
            RecipeKind::SmithingTrim { .. } => "smithing_trim",
            RecipeKind::Special { kind, .. } => match kind {
                SpecialRecipeKind::Transmute => "crafting_transmute",
                SpecialRecipeKind::MapCloning => "crafting_special_mapcloning",
                SpecialRecipeKind::MapExtending => "crafting_special_mapextending",
                SpecialRecipeKind::BannerDuplicate => "crafting_special_bannerduplicate",
                SpecialRecipeKind::ShieldDecoration => "crafting_special_shielddecoration",
                SpecialRecipeKind::FireworkRocket => "crafting_special_firework_rocket",
                SpecialRecipeKind::FireworkStar => "crafting_special_firework_star",
                SpecialRecipeKind::FireworkStarFade => "crafting_special_firework_star_fade",
                SpecialRecipeKind::SuspiciousStew => "crafting_special_suspiciousstew",
                SpecialRecipeKind::BookCloning => "crafting_special_bookcloning",
                SpecialRecipeKind::RepairItem => "crafting_special_repairitem",
                SpecialRecipeKind::DyedItem => "crafting_dye",
                SpecialRecipeKind::DecoratedPot => "crafting_decorated_pot",
                SpecialRecipeKind::Imbue => "crafting_imbue",
            },
        }
    }

    pub fn matches(
        &self,
        grid_width: usize,
        grid_height: usize,
        items: &[Option<&'static str>],
    ) -> bool {
        match self {
            RecipeKind::Shaped {
                width,
                height,
                pattern,
                ..
            } => shaped_matches(*width, *height, pattern, grid_width, grid_height, items),
            RecipeKind::Shapeless { ingredients, .. } => shapeless_matches(ingredients, items),
            RecipeKind::Cooking { ingredient, .. }
            | RecipeKind::Stonecutting { ingredient, .. } => {
                let mut present = items.iter().flatten();
                let Some(item) = present.next() else {
                    return false;
                };
                present.next().is_none() && ingredient.matches(item)
            }
            RecipeKind::SmithingTransform {
                template,
                base,
                addition,
                ..
            }
            | RecipeKind::SmithingTrim {
                template,
                base,
                addition,
            } => {
                let [Some(template_item), Some(base_item), Some(addition_item)] = items else {
                    return false;
                };
                template.matches(template_item)
                    && base.matches(base_item)
                    && addition.matches(addition_item)
            }
            RecipeKind::Special { .. } => false,
        }
    }

    pub fn assemble(&self) -> Option<ItemAmount> {
        match self {
            RecipeKind::Shaped { result, .. }
            | RecipeKind::Shapeless { result, .. }
            | RecipeKind::Cooking { result, .. }
            | RecipeKind::Stonecutting { result, .. }
            | RecipeKind::SmithingTransform { result, .. } => Some(result.clone()),
            RecipeKind::SmithingTrim { .. } => None,
            RecipeKind::Special { result_hint, .. } => result_hint.clone(),
        }
    }

    pub fn cooking_time(&self) -> Option<i32> {
        match self {
            RecipeKind::Cooking {
                kind, cooking_time, ..
            } => Some(cooking_time.unwrap_or_else(|| kind.default_cooking_time())),
            _ => None,
        }
    }
}

fn shaped_matches(
    recipe_width: usize,
    recipe_height: usize,
    pattern: &[Option<IngredientSpec>],
    grid_width: usize,
    grid_height: usize,
    items: &[Option<&'static str>],
) -> bool {
    if recipe_width == 0
        || recipe_height == 0
        || recipe_width > grid_width
        || recipe_height > grid_height
        || items.len() != grid_width * grid_height
        || pattern.len() != recipe_width * recipe_height
    {
        return false;
    }

    for y_offset in 0..=(grid_height - recipe_height) {
        for x_offset in 0..=(grid_width - recipe_width) {
            if shaped_matches_at(
                recipe_width,
                recipe_height,
                pattern,
                grid_width,
                grid_height,
                items,
                x_offset,
                y_offset,
            ) {
                return true;
            }
        }
    }

    false
}

fn shaped_matches_at(
    recipe_width: usize,
    recipe_height: usize,
    pattern: &[Option<IngredientSpec>],
    grid_width: usize,
    grid_height: usize,
    items: &[Option<&'static str>],
    x_offset: usize,
    y_offset: usize,
) -> bool {
    for y in 0..grid_height {
        for x in 0..grid_width {
            let grid_item = items[y * grid_width + x];
            let pattern_item = if x >= x_offset
                && x < x_offset + recipe_width
                && y >= y_offset
                && y < y_offset + recipe_height
            {
                pattern[(y - y_offset) * recipe_width + (x - x_offset)].as_ref()
            } else {
                None
            };

            match (pattern_item, grid_item) {
                (None, None) => {}
                (Some(ingredient), Some(item)) if ingredient.matches(item) => {}
                _ => return false,
            }
        }
    }

    true
}

fn shapeless_matches(ingredients: &[IngredientSpec], items: &[Option<&'static str>]) -> bool {
    let provided: Vec<&'static str> = items.iter().flatten().copied().collect();
    if provided.len() != ingredients.len() {
        return false;
    }

    let mut used = vec![false; provided.len()];
    ingredients.iter().all(|ingredient| {
        let Some(index) = provided
            .iter()
            .enumerate()
            .position(|(index, item)| !used[index] && ingredient.matches(item))
        else {
            return false;
        };
        used[index] = true;
        true
    })
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

    #[test]
    fn shaped_and_shapeless_recipes_match_vanilla_grid_rules() {
        let shaped = RecipeKind::Shaped {
            width: 2,
            height: 2,
            pattern: vec![
                Some(IngredientSpec::Item("minecraft:oak_planks")),
                Some(IngredientSpec::Item("minecraft:oak_planks")),
                Some(IngredientSpec::Item("minecraft:oak_planks")),
                Some(IngredientSpec::Item("minecraft:oak_planks")),
            ],
            result: ItemAmount::one("minecraft:crafting_table"),
        };
        let grid = vec![
            None,
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            None,
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            None,
            None,
            None,
        ];

        assert!(shaped.matches(3, 3, &grid));
        assert_eq!(shaped.serializer(), "crafting_shaped");
        assert_eq!(
            shaped.assemble(),
            Some(ItemAmount::one("minecraft:crafting_table"))
        );

        let shapeless = RecipeKind::Shapeless {
            ingredients: vec![
                IngredientSpec::Item("minecraft:gunpowder"),
                IngredientSpec::AnyOf(vec!["minecraft:red_dye", "minecraft:blue_dye"]),
            ],
            result: ItemAmount::one("minecraft:firework_star"),
        };
        assert!(shapeless.matches(
            2,
            2,
            &[
                Some("minecraft:blue_dye"),
                None,
                Some("minecraft:gunpowder"),
                None
            ]
        ));
        assert!(!shapeless.matches(
            2,
            2,
            &[
                Some("minecraft:blue_dye"),
                Some("minecraft:gunpowder"),
                Some("minecraft:paper"),
                None
            ]
        ));
    }

    #[test]
    fn cooking_stonecutting_and_smithing_recipes_match_single_input_contracts() {
        for (kind, expected_time) in [
            (CookingKind::Smelting, 200),
            (CookingKind::Blasting, 100),
            (CookingKind::Smoking, 100),
            (CookingKind::CampfireCooking, 600),
        ] {
            let recipe = RecipeKind::Cooking {
                kind,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None,
            };
            assert!(recipe.matches(1, 1, &[Some("minecraft:raw_iron")]));
            assert_eq!(recipe.cooking_time(), Some(expected_time));
            assert_eq!(recipe.serializer(), kind.serializer());
        }

        let stonecutting = RecipeKind::Stonecutting {
            ingredient: IngredientSpec::Item("minecraft:stone"),
            result: ItemAmount {
                item: "minecraft:stone_slab",
                count: 2,
            },
        };
        assert!(stonecutting.matches(1, 1, &[Some("minecraft:stone")]));
        assert!(!stonecutting.matches(1, 2, &[Some("minecraft:stone"), Some("minecraft:stone")]));

        let transform = RecipeKind::SmithingTransform {
            template: IngredientSpec::Item("minecraft:netherite_upgrade_smithing_template"),
            base: IngredientSpec::Item("minecraft:diamond_sword"),
            addition: IngredientSpec::Item("minecraft:netherite_ingot"),
            result: ItemAmount::one("minecraft:netherite_sword"),
        };
        let trim = RecipeKind::SmithingTrim {
            template: IngredientSpec::Item("minecraft:spire_armor_trim_smithing_template"),
            base: IngredientSpec::Item("minecraft:iron_chestplate"),
            addition: IngredientSpec::Item("minecraft:amethyst_shard"),
        };

        assert!(transform.matches(
            3,
            1,
            &[
                Some("minecraft:netherite_upgrade_smithing_template"),
                Some("minecraft:diamond_sword"),
                Some("minecraft:netherite_ingot")
            ]
        ));
        assert_eq!(transform.serializer(), "smithing_transform");
        assert!(trim.matches(
            3,
            1,
            &[
                Some("minecraft:spire_armor_trim_smithing_template"),
                Some("minecraft:iron_chestplate"),
                Some("minecraft:amethyst_shard")
            ]
        ));
        assert_eq!(trim.serializer(), "smithing_trim");
        assert_eq!(trim.assemble(), None);
    }

    #[test]
    fn special_recipe_kinds_cover_checklist_families_and_serializer_names() {
        let special = [
            (SpecialRecipeKind::Transmute, "crafting_transmute"),
            (SpecialRecipeKind::MapCloning, "crafting_special_mapcloning"),
            (
                SpecialRecipeKind::MapExtending,
                "crafting_special_mapextending",
            ),
            (
                SpecialRecipeKind::BannerDuplicate,
                "crafting_special_bannerduplicate",
            ),
            (
                SpecialRecipeKind::ShieldDecoration,
                "crafting_special_shielddecoration",
            ),
            (
                SpecialRecipeKind::FireworkRocket,
                "crafting_special_firework_rocket",
            ),
            (
                SpecialRecipeKind::FireworkStar,
                "crafting_special_firework_star",
            ),
            (
                SpecialRecipeKind::FireworkStarFade,
                "crafting_special_firework_star_fade",
            ),
            (
                SpecialRecipeKind::SuspiciousStew,
                "crafting_special_suspiciousstew",
            ),
            (
                SpecialRecipeKind::BookCloning,
                "crafting_special_bookcloning",
            ),
            (SpecialRecipeKind::RepairItem, "crafting_special_repairitem"),
            (SpecialRecipeKind::DyedItem, "crafting_dye"),
            (SpecialRecipeKind::DecoratedPot, "crafting_decorated_pot"),
            (SpecialRecipeKind::Imbue, "crafting_imbue"),
        ];

        for (kind, serializer) in special {
            let recipe = RecipeKind::Special {
                kind,
                result_hint: None,
            };
            assert_eq!(recipe.serializer(), serializer);
            assert!(!recipe.matches(3, 3, &[None; 9]));
            assert_eq!(recipe.assemble(), None);
        }
    }
}
