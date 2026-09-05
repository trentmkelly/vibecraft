#![cfg_attr(
    all(test, not(vibecraft_has_decompiled_sources)),
    allow(dead_code)
)]

use crate::item_stack::ItemStack;

#[cfg(test)]
use crate::advancement_system::RecipeDefinition;
#[cfg(test)]
use crate::registry::Identifier;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: &'static str,
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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
    #[cfg(test)]
    Furnace,
    #[cfg(test)]
    BlastFurnace,
    #[cfg(test)]
    Smoker,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBookTypeSettings {
    pub open: bool,
    pub filtering: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBookSettings {
    pub crafting: RecipeBookTypeSettings,
    pub furnace: RecipeBookTypeSettings,
    pub blast_furnace: RecipeBookTypeSettings,
    pub smoker: RecipeBookTypeSettings,
}

#[cfg(test)]
impl RecipeBookSettings {
    pub const CRAFTING_CODEC_FIELDS: (&'static str, &'static str) =
        ("isGuiOpen", "isFilteringCraftable");
    pub const FURNACE_CODEC_FIELDS: (&'static str, &'static str) =
        ("isFurnaceGuiOpen", "isFurnaceFilteringCraftable");
    pub const BLAST_FURNACE_CODEC_FIELDS: (&'static str, &'static str) = (
        "isBlastingFurnaceGuiOpen",
        "isBlastingFurnaceFilteringCraftable",
    );
    pub const SMOKER_CODEC_FIELDS: (&'static str, &'static str) =
        ("isSmokerGuiOpen", "isSmokerFilteringCraftable");

    pub fn get(self, book_type: RecipeBookType) -> RecipeBookTypeSettings {
        match book_type {
            RecipeBookType::Crafting => self.crafting,
            RecipeBookType::Furnace => self.furnace,
            RecipeBookType::BlastFurnace => self.blast_furnace,
            RecipeBookType::Smoker => self.smoker,
        }
    }

    pub fn set_open(&mut self, book_type: RecipeBookType, open: bool) {
        self.update(book_type, |settings| settings.set_open(open));
    }

    pub fn set_filtering(&mut self, book_type: RecipeBookType, filtering: bool) {
        self.update(book_type, |settings| settings.set_filtering(filtering));
    }

    pub fn copy(self) -> Self {
        self
    }

    pub fn replace_from(&mut self, other: Self) {
        self.crafting = other.crafting;
        self.furnace = other.furnace;
        self.blast_furnace = other.blast_furnace;
        self.smoker = other.smoker;
    }

    pub fn codec_fields(book_type: RecipeBookType) -> (&'static str, &'static str) {
        match book_type {
            RecipeBookType::Crafting => Self::CRAFTING_CODEC_FIELDS,
            RecipeBookType::Furnace => Self::FURNACE_CODEC_FIELDS,
            RecipeBookType::BlastFurnace => Self::BLAST_FURNACE_CODEC_FIELDS,
            RecipeBookType::Smoker => Self::SMOKER_CODEC_FIELDS,
        }
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

#[cfg(test)]
impl RecipeBookTypeSettings {
    pub fn set_open(self, open: bool) -> Self {
        Self { open, ..self }
    }

    pub fn set_filtering(self, filtering: bool) -> Self {
        Self { filtering, ..self }
    }

    pub fn java_display(self) -> String {
        format!("[open={}, filtering={}]", self.open, self.filtering)
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecipeBook {
    book_settings: RecipeBookSettings,
}

#[cfg(test)]
impl RecipeBook {
    pub fn is_open(self, book_type: RecipeBookType) -> bool {
        self.book_settings.get(book_type).open
    }

    pub fn set_open(&mut self, book_type: RecipeBookType, open: bool) {
        self.book_settings.set_open(book_type, open);
    }

    pub fn is_filtering(self, book_type: RecipeBookType) -> bool {
        self.book_settings.get(book_type).filtering
    }

    pub fn set_filtering(&mut self, book_type: RecipeBookType, filtering: bool) {
        self.book_settings.set_filtering(book_type, filtering);
    }

    pub fn set_book_settings(&mut self, settings: RecipeBookSettings) {
        self.book_settings.replace_from(settings);
    }

    pub fn get_book_settings(self) -> RecipeBookSettings {
        self.book_settings
    }

    pub fn set_book_setting(&mut self, book_type: RecipeBookType, open: bool, filtering: bool) {
        self.book_settings.set_open(book_type, open);
        self.book_settings.set_filtering(book_type, filtering);
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayId(pub i32);

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayEntry {
    pub id: RecipeDisplayId,
    pub display: RecipeDisplay,
    pub group: Option<i32>,
    pub category: &'static str,
    pub crafting_requirements: Option<Vec<SlotDisplay>>,
}

#[cfg(test)]
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

#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerRecipeBookDisplayResolver {
    displays: std::collections::BTreeMap<Identifier, Vec<RecipeDisplayEntry>>,
}

#[cfg(test)]
impl ServerRecipeBookDisplayResolver {
    pub fn new(displays: impl IntoIterator<Item = (Identifier, Vec<RecipeDisplayEntry>)>) -> Self {
        Self {
            displays: displays.into_iter().collect(),
        }
    }

    pub fn displays_for_recipe(&self, id: &Identifier) -> &[RecipeDisplayEntry] {
        self.displays.get(id).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerRecipeBookPacked {
    pub settings: RecipeBookSettings,
    pub known: Vec<Identifier>,
    pub highlight: Vec<Identifier>,
}

#[cfg(test)]
impl ServerRecipeBookPacked {
    pub const RECIPES_FIELD: &'static str = "recipes";
    pub const HIGHLIGHT_FIELD: &'static str = "toBeDisplayed";
}

#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerRecipeBook {
    recipe_book: RecipeBook,
    display_resolver: ServerRecipeBookDisplayResolver,
    known: std::collections::BTreeSet<Identifier>,
    highlight: std::collections::BTreeSet<Identifier>,
}

#[cfg(test)]
impl ServerRecipeBook {
    pub const RECIPE_BOOK_TAG: &'static str = "recipeBook";

    pub fn new(display_resolver: ServerRecipeBookDisplayResolver) -> Self {
        Self {
            display_resolver,
            ..Self::default()
        }
    }

    pub fn add(&mut self, id: Identifier) {
        self.known.insert(id);
    }

    pub fn contains(&self, id: &Identifier) -> bool {
        self.known.contains(id)
    }

    pub fn remove(&mut self, id: &Identifier) {
        self.known.remove(id);
        self.highlight.remove(id);
    }

    pub fn remove_highlight(&mut self, id: &Identifier) {
        self.highlight.remove(id);
    }

    fn add_highlight(&mut self, id: Identifier) {
        self.highlight.insert(id);
    }

    pub fn add_recipes(
        &mut self,
        recipes: &[RecipeDefinition],
    ) -> (usize, Option<ClientboundRecipeBookAddPacket>, Vec<Identifier>) {
        let mut entries = Vec::new();
        let mut triggered = Vec::new();
        for recipe in recipes {
            if self.known.contains(&recipe.id) || recipe.special {
                continue;
            }
            self.add(recipe.id.clone());
            self.add_highlight(recipe.id.clone());
            entries.extend(self.display_resolver.displays_for_recipe(&recipe.id).iter().map(
                |display| RecipeBookAddEntry {
                    contents: display.clone(),
                    flags: RecipeBookEntryFlags::new(recipe.show_notification, true),
                },
            ));
            triggered.push(recipe.id.clone());
        }

        let count = entries.len();
        let packet = (!entries.is_empty()).then_some(ClientboundRecipeBookAddPacket {
            entries,
            replace: false,
        });
        (count, packet, triggered)
    }

    pub fn remove_recipes(
        &mut self,
        recipes: &[Identifier],
    ) -> (usize, Option<ClientboundRecipeBookRemovePacket>) {
        let mut removed = Vec::new();
        for recipe in recipes {
            if self.known.contains(recipe) {
                self.remove(recipe);
                removed.extend(
                    self.display_resolver
                        .displays_for_recipe(recipe)
                        .iter()
                        .map(|display| display.id.clone()),
                );
            }
        }

        let count = removed.len();
        let packet = (!removed.is_empty()).then_some(ClientboundRecipeBookRemovePacket {
            recipes: removed,
        });
        (count, packet)
    }

    pub fn send_initial_recipe_book(
        &self,
    ) -> (
        ClientboundRecipeBookSettingsPacket,
        ClientboundRecipeBookAddPacket,
    ) {
        let entries = self
            .known
            .iter()
            .flat_map(|id| {
                self.display_resolver
                    .displays_for_recipe(id)
                    .iter()
                    .map(|display| RecipeBookAddEntry {
                        contents: display.clone(),
                        flags: RecipeBookEntryFlags::new(false, self.highlight.contains(id)),
                    })
            })
            .collect();

        (
            ClientboundRecipeBookSettingsPacket {
                settings: self.recipe_book.get_book_settings().copy(),
            },
            ClientboundRecipeBookAddPacket {
                entries,
                replace: true,
            },
        )
    }

    pub fn copy_over_data(&mut self, book_to_copy: &ServerRecipeBook) {
        self.apply(book_to_copy.pack());
    }

    pub fn pack(&self) -> ServerRecipeBookPacked {
        ServerRecipeBookPacked {
            settings: self.recipe_book.get_book_settings().copy(),
            known: self.known.iter().cloned().collect(),
            highlight: self.highlight.iter().cloned().collect(),
        }
    }

    fn apply(&mut self, packed: ServerRecipeBookPacked) {
        self.known.clear();
        self.highlight.clear();
        self.recipe_book.set_book_settings(packed.settings);
        self.known.extend(packed.known);
        self.highlight.extend(packed.highlight);
    }

    pub fn load_untrusted(
        &mut self,
        packed: ServerRecipeBookPacked,
        validator: impl Fn(&Identifier) -> bool,
    ) {
        self.recipe_book.set_book_settings(packed.settings);
        self.known
            .extend(packed.known.into_iter().filter(|recipe| validator(recipe)));
        self.highlight.extend(
            packed
                .highlight
                .into_iter()
                .filter(|recipe| validator(recipe)),
        );
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookEntryFlags(u8);

#[cfg(test)]
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookAddEntry {
    pub contents: RecipeDisplayEntry,
    pub flags: RecipeBookEntryFlags,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookAddPacket {
    pub entries: Vec<RecipeBookAddEntry>,
    pub replace: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookRemovePacket {
    pub recipes: Vec<RecipeDisplayId>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    pub settings: RecipeBookSettings,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateRecipesPacket {
    pub item_sets: Vec<RecipePropertySet>,
    pub stonecutter_recipes: Vec<SelectableSingleInputRecipe>,
}

/// `RecipePropertySet` — the set of items accepted by a `RecipeType`'s input slot,
/// used for client-side ingredient caching (e.g. furnace input, smithing slots).
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RecipePropertySet {
    pub key: &'static str,
    pub accepted_items: Vec<&'static str>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectableSingleInputRecipe {
    pub input: &'static str,
    pub recipe: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeMap {
    recipes: Vec<RecipeHolder>,
}

impl RecipeMap {
    pub fn create(recipes: Vec<RecipeHolder>) -> Self {
        Self { recipes }
    }

    pub fn by_key(&self, id: &str) -> Option<&RecipeHolder> {
        self.recipes.iter().find(|holder| holder.id == id)
    }

    #[cfg(test)]
    pub fn by_type(&self, recipe_type: &str) -> Vec<&RecipeHolder> {
        self.recipes
            .iter()
            .filter(|holder| holder.recipe.recipe_type() == recipe_type)
            .collect()
    }

    pub fn values(&self) -> &[RecipeHolder] {
        &self.recipes
    }

    pub fn get_recipe_for(
        &self,
        recipe_type: &str,
        grid_width: usize,
        grid_height: usize,
        items: &[Option<&'static str>],
    ) -> Option<&RecipeHolder> {
        if items.iter().all(Option::is_none) {
            return None;
        }

        self.recipes.iter().find(|holder| {
            holder.recipe.recipe_type() == recipe_type
                && holder.recipe.matches(grid_width, grid_height, items)
        })
    }

    /// Evaluate the registered `crafting` special recipes (`CustomRecipe`s) against
    /// the live grid of stacks, returning the id + outcome of the first match.
    ///
    /// The ordinary `RecipeKind::matches` path is id-only and cannot express the
    /// component-dependent rules of `CustomRecipe`s, so these are checked here with
    /// full `ItemStack`s. A special recipe only matches inputs that no ordinary
    /// recipe does, so callers consult this after `get_recipe_for` returns `None`.
    pub fn special_crafting_result(
        &self,
        grid: &[ItemStack],
    ) -> Option<(&'static str, SpecialCraftOutcome)> {
        if grid.iter().all(ItemStack::is_empty) {
            return None;
        }
        self.recipes.iter().find_map(|holder| {
            let RecipeKind::Special { kind, result_hint } = &holder.recipe else {
                return None;
            };
            special_crafting_result(*kind, result_hint.as_ref(), grid)
                .map(|outcome| (holder.id, outcome))
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeManagerModel {
    recipes: RecipeMap,
    acquisition_unlocks: Vec<RecipeAcquisitionUnlock>,
    initial_unlocks: Vec<&'static str>,
    property_sets: Vec<RecipePropertySet>,
    stonecutter_recipes: Vec<StonecutterSelection>,
}

impl RecipeManagerModel {
    pub fn new(recipes: Vec<RecipeHolder>) -> Self {
        let mut manager = Self::default();
        manager.reload(recipes);
        manager
    }

    pub fn reload(&mut self, recipes: Vec<RecipeHolder>) {
        self.recipes = RecipeMap::create(recipes);
        // `RecipeManager` recomputes the per-`RecipeType` property sets on reload.
        self.property_sets = collect_recipe_property_sets(self.recipes.values());
        self.stonecutter_recipes = self
            .recipes
            .values()
            .iter()
            .filter_map(|holder| match &holder.recipe {
                RecipeKind::Stonecutting { ingredient, result } => Some(StonecutterSelection {
                    recipe_id: holder.id,
                    input: ingredient.clone(),
                    result: result.clone(),
                }),
                _ => None,
            })
            .collect();
    }

    pub fn recipe_map(&self) -> &RecipeMap {
        &self.recipes
    }

    pub fn set_acquisition_unlocks(&mut self, unlocks: Vec<RecipeAcquisitionUnlock>) {
        self.acquisition_unlocks = unlocks;
    }

    pub fn set_initial_unlocks(&mut self, unlocks: Vec<&'static str>) {
        self.initial_unlocks = unlocks;
    }

    pub fn recipes_unlocked_by_item(&self, item: &str) -> Vec<&'static str> {
        self.acquisition_unlocks
            .iter()
            .filter(|unlock| unlock.matches(item))
            .map(|unlock| unlock.recipe_id)
            .collect()
    }

    pub fn initially_unlocked_recipes(&self) -> &[&'static str] {
        &self.initial_unlocks
    }

    pub fn property_sets(&self) -> &[RecipePropertySet] {
        &self.property_sets
    }

    #[allow(dead_code)]
    pub fn property_set(&self, key: &str) -> RecipePropertySet {
        self.property_sets
            .iter()
            .find(|set| set.key == key)
            .cloned()
            .unwrap_or_else(|| RecipePropertySet {
                key: "minecraft:empty",
                accepted_items: Vec::new(),
            })
    }

    pub fn stonecutter_recipes(&self) -> &[StonecutterSelection] {
        &self.stonecutter_recipes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeAcquisitionUnlock {
    pub recipe_id: &'static str,
    pub ingredients: Vec<IngredientSpec>,
}

impl RecipeAcquisitionUnlock {
    pub fn new(recipe_id: &'static str, ingredients: Vec<IngredientSpec>) -> Self {
        Self {
            recipe_id,
            ingredients,
        }
    }

    pub fn matches(&self, item: &str) -> bool {
        self.ingredients
            .iter()
            .any(|ingredient| ingredient.matches(item))
    }
}

/// Resolved item tag map: maps a tag ID (e.g. `minecraft:oak_logs`) to the flat,
/// deduplicated list of item IDs it contains.
///
/// Tags are loaded from `data/minecraft/tags/item/*.json` and resolved recursively
/// so that a tag referencing another tag (e.g. `#minecraft:logs_that_burn`) is
/// fully expanded to concrete item IDs before being stored.
///
/// All item ID strings are interned via `Box::leak` and stored as `&'static str` to
/// match the convention used throughout the rest of the recipe system.
#[derive(Debug, Default, Clone)]
pub struct ItemTagMap {
    /// Maps tag ID → flat, deduplicated list of resolved item IDs.
    tags: std::collections::HashMap<String, Vec<&'static str>>,
}

impl ItemTagMap {
    /// Returns the resolved item list for the given tag ID (e.g. `"minecraft:oak_logs"`).
    /// Returns an empty slice if the tag is unknown.
    pub fn resolve(&self, tag_id: &str) -> &[&'static str] {
        self.tags.get(tag_id).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Loads and recursively resolves all item tags from a directory of tag JSON files.
///
/// Each file is expected to be named `<tag_stem>.json` and contain a JSON object of
/// the form `{"values": [...]}` where each value is either a plain item ID like
/// `"minecraft:oak_log"` or a tag reference like `"#minecraft:another_tag"`.
///
/// The tag namespace is derived from the filename stem: `oak_logs.json` →
/// `minecraft:oak_logs`.  Circular tag references are silently broken (the cycle
/// is excluded from the resolved list rather than causing infinite recursion).
///
/// If the directory does not exist or cannot be read, an empty `ItemTagMap` is
/// returned — this is treated as a non-fatal situation so that the recipe loader
/// can still parse recipes that use only direct item IDs.
pub fn load_item_tag_directory(tag_dir: &std::path::Path) -> ItemTagMap {
    // Raw tag data: tag_id → list of values (each value is either a plain item ID
    // or a tag reference starting with '#').
    let mut raw: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    let entries = match std::fs::read_dir(tag_dir) {
        Ok(e) => e,
        Err(_) => {
            // Tag directory missing — degrade gracefully.
            return ItemTagMap::default();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let tag_id = format!("minecraft:{stem}");

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let value: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(values_array) = value
            .as_object()
            .and_then(|o| o.get("values"))
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };

        let mut values: Vec<String> = Vec::with_capacity(values_array.len());
        for v in values_array {
            if let Some(s) = v.as_str() {
                values.push(s.to_string());
            }
        }
        raw.insert(tag_id, values);
    }

    // Resolve all tags recursively.  We compute the final resolved list for each
    // tag by depth-first traversal, tracking the current path to break cycles.
    let tag_ids: Vec<String> = raw.keys().cloned().collect();
    let mut resolved: std::collections::HashMap<String, Vec<&'static str>> =
        std::collections::HashMap::new();

    for tag_id in &tag_ids {
        resolve_tag(
            tag_id,
            &raw,
            &mut resolved,
            &mut std::collections::HashSet::new(),
        );
    }

    ItemTagMap { tags: resolved }
}

mod recipe_loading;
pub use recipe_loading::*;
mod holder;
pub use holder::RecipeHolder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StonecutterSelection {
    pub recipe_id: &'static str,
    pub input: IngredientSpec,
    pub result: ItemAmount,
}

impl StonecutterSelection {
    pub fn matches_input(&self, input: &'static str) -> bool {
        self.input.matches(input)
    }
}

pub fn stonecutter_recipes_for_input(
    recipes: &[StonecutterSelection],
    input: &'static str,
) -> Vec<StonecutterSelection> {
    recipes
        .iter()
        .filter(|recipe| recipe.matches_input(input))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAmount {
    pub item: &'static str,
    pub count: u32,
}

impl ItemAmount {
    #[cfg(test)]
    pub const fn one(item: &'static str) -> Self {
        Self { item, count: 1 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingStack {
    pub item: &'static str,
    pub count: u32,
}

impl CraftingStack {
    pub fn one(item: &'static str) -> Self {
        Self { item, count: 1 }
    }
}

mod component_models;
pub use component_models::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientSpec {
    Empty,
    Item(&'static str),
    AnyOf(Vec<&'static str>),
}

impl IngredientSpec {
    pub fn matches(&self, item: &str) -> bool {
        match self {
            IngredientSpec::Empty => false,
            IngredientSpec::Item(expected) => *expected == item,
            // `contains(&item)` would require `item: &'static str`; the `&str`
            // parameter (so non-static furnace inputs can match) needs `any`.
            #[allow(clippy::manual_contains)]
            IngredientSpec::AnyOf(items) => items.iter().any(|candidate| *candidate == item),
        }
    }

    pub fn items(&self) -> Vec<&'static str> {
        match self {
            IngredientSpec::Empty => Vec::new(),
            IngredientSpec::Item(item) => vec![*item],
            IngredientSpec::AnyOf(items) => items.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        matches!(self, IngredientSpec::Empty)
    }
}

/// `PlacementInfo` — the client-side ghost-recipe placement hint: the recipe's
/// ingredients plus, for each grid slot, the index of the ingredient that goes
/// there (`EMPTY_SLOT` for a gap). An empty/impossible recipe has no slots.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlacementInfo {
    pub ingredients: Vec<IngredientSpec>,
    pub slots_to_ingredient_index: Vec<i32>,
}

#[allow(dead_code)]
impl PlacementInfo {
    pub const EMPTY_SLOT: i32 = -1;

    pub fn not_placeable() -> Self {
        Self {
            ingredients: Vec::new(),
            slots_to_ingredient_index: Vec::new(),
        }
    }

    pub fn create(ingredient: IngredientSpec) -> Self {
        if ingredient.is_empty() {
            Self::not_placeable()
        } else {
            Self {
                ingredients: vec![ingredient],
                slots_to_ingredient_index: vec![0],
            }
        }
    }

    pub fn create_list(ingredients: Vec<IngredientSpec>) -> Self {
        if ingredients.iter().any(IngredientSpec::is_empty) {
            return Self::not_placeable();
        }

        Self {
            slots_to_ingredient_index: (0..ingredients.len()).map(|index| index as i32).collect(),
            ingredients,
        }
    }

    pub fn create_from_optionals(ingredients: Vec<Option<IngredientSpec>>) -> Self {
        let mut present = Vec::with_capacity(ingredients.len());
        let mut slots = Vec::with_capacity(ingredients.len());
        for maybe_ingredient in ingredients {
            if let Some(ingredient) = maybe_ingredient {
                if ingredient.is_empty() {
                    return Self::not_placeable();
                }

                slots.push(present.len() as i32);
                present.push(ingredient);
            } else {
                slots.push(Self::EMPTY_SLOT);
            }
        }

        Self {
            ingredients: present,
            slots_to_ingredient_index: slots,
        }
    }

    pub fn is_impossible_to_place(&self) -> bool {
        self.slots_to_ingredient_index.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookingKind {
    Smelting,
    Blasting,
    Smoking,
    CampfireCooking,
}

/// `CookingBookCategory` — the recipe-book grouping a cooking recipe declares in its
/// JSON `category` field (defaults to `Misc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookingBookCategory {
    Food,
    Blocks,
    Misc,
}

impl CookingBookCategory {
    /// Parse the `category` string, defaulting to `Misc` for unknown/absent values
    /// (matching the `AbstractCookingRecipe` codec default).
    pub fn from_id(id: Option<&str>) -> Self {
        match id {
            Some("food") => Self::Food,
            Some("blocks") => Self::Blocks,
            _ => Self::Misc,
        }
    }
}

impl CookingKind {
    /// `AbstractCookingRecipe` default cook time per type (ticks): smelting 200,
    /// blasting/smoking/campfire 100. Used when a recipe omits `cookingtime`.
    pub fn default_cooking_time(self) -> i32 {
        match self {
            CookingKind::Smelting => 200,
            CookingKind::Blasting | CookingKind::Smoking => 100,
            CookingKind::CampfireCooking => 100,
        }
    }

    #[allow(dead_code)]
    pub fn serializer(self) -> &'static str {
        match self {
            CookingKind::Smelting => "smelting",
            CookingKind::Blasting => "blasting",
            CookingKind::Smoking => "smoking",
            CookingKind::CampfireCooking => "campfire_cooking",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelValues {
    entries: Vec<(&'static str, i32)>,
}

mod fuel_values;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnaceRecipeUsage {
    pub recipe_id: &'static str,
    pub times_used: i32,
    pub experience_millis: i32,
}

#[cfg(test)]
pub fn furnace_experience_to_award(usage: &FurnaceRecipeUsage, fraction_roll: f32) -> i32 {
    if usage.times_used <= 0 || usage.experience_millis <= 0 {
        return 0;
    }

    let total_millis = usage.times_used * usage.experience_millis;
    let whole = total_millis / 1000;
    let fraction = (total_millis % 1000) as f32 / 1000.0;
    if fraction != 0.0 && fraction_roll < fraction {
        whole + 1
    } else {
        whole
    }
}

mod recipe_results;
pub use recipe_results::*;
mod recipe_display_ids;
mod special_crafting;
pub use special_crafting::*;

#[cfg(test)]
mod tests;
