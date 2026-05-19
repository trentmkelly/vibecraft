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
pub struct RecipeHolder {
    pub id: &'static str,
    pub recipe: RecipeKind,
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeManagerModel {
    recipes: RecipeMap,
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

pub fn load_recipe_json(id: &'static str, raw: &str) -> Result<RecipeHolder, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|err| format!("failed to parse recipe {id} as JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("recipe {id} must be a JSON object"))?;
    let recipe_type = json_str(object, "type")?
        .strip_prefix("minecraft:")
        .unwrap_or(json_str(object, "type")?);
    let recipe = match recipe_type {
        "crafting_shaped" => {
            let key = object
                .get("key")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| format!("shaped recipe {id} is missing key object"))?;
            let pattern_rows = object
                .get("pattern")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("shaped recipe {id} is missing pattern array"))?;
            let height = pattern_rows.len();
            let width = pattern_rows
                .first()
                .and_then(serde_json::Value::as_str)
                .map(str::len)
                .ok_or_else(|| format!("shaped recipe {id} has empty pattern"))?;
            let mut pattern = Vec::with_capacity(width * height);
            for row in pattern_rows {
                let row = row
                    .as_str()
                    .ok_or_else(|| format!("shaped recipe {id} has non-string pattern row"))?;
                if row.len() != width {
                    return Err(format!("shaped recipe {id} has ragged pattern rows"));
                }
                for key_char in row.chars() {
                    if key_char == ' ' {
                        pattern.push(None);
                    } else {
                        let key_name = key_char.to_string();
                        let ingredient = key
                            .get(&key_name)
                            .ok_or_else(|| {
                                format!("shaped recipe {id} has unmapped key '{key_char}'")
                            })
                            .and_then(parse_ingredient)?;
                        pattern.push(Some(ingredient));
                    }
                }
            }
            RecipeKind::Shaped {
                width,
                height,
                pattern,
                result: parse_result(object, id)?,
            }
        }
        "crafting_shapeless" => RecipeKind::Shapeless {
            ingredients: parse_ingredient_array(object, "ingredients", id)?,
            result: parse_result(object, id)?,
        },
        "smelting" | "blasting" | "smoking" | "campfire_cooking" => RecipeKind::Cooking {
            kind: match recipe_type {
                "smelting" => CookingKind::Smelting,
                "blasting" => CookingKind::Blasting,
                "smoking" => CookingKind::Smoking,
                "campfire_cooking" => CookingKind::CampfireCooking,
                _ => unreachable!(),
            },
            ingredient: parse_field_ingredient(object, "ingredient", id)?,
            result: parse_result(object, id)?,
            experience_millis: object
                .get("experience")
                .and_then(serde_json::Value::as_f64)
                .map(|value| (value * 1000.0).round() as i32)
                .unwrap_or(0),
            cooking_time: object
                .get("cookingtime")
                .and_then(serde_json::Value::as_i64)
                .map(|value| value as i32),
        },
        "stonecutting" => RecipeKind::Stonecutting {
            ingredient: parse_field_ingredient(object, "ingredient", id)?,
            result: parse_result(object, id)?,
        },
        "smithing_transform" => RecipeKind::SmithingTransform {
            template: parse_optional_field_ingredient(object, "template")?,
            base: parse_field_ingredient(object, "base", id)?,
            addition: parse_optional_field_ingredient(object, "addition")?,
            result: parse_result(object, id)?,
        },
        "smithing_trim" => RecipeKind::SmithingTrim {
            template: parse_field_ingredient(object, "template", id)?,
            base: parse_field_ingredient(object, "base", id)?,
            addition: parse_field_ingredient(object, "addition", id)?,
        },
        "crafting_transmute" => RecipeKind::Special {
            kind: SpecialRecipeKind::Transmute,
            result_hint: Some(parse_result(object, id)?),
        },
        "crafting_imbue" => RecipeKind::Special {
            kind: SpecialRecipeKind::Imbue,
            result_hint: Some(parse_result(object, id)?),
        },
        "crafting_dye" => RecipeKind::Special {
            kind: SpecialRecipeKind::DyedItem,
            result_hint: Some(parse_result(object, id)?),
        },
        "crafting_decorated_pot" => RecipeKind::Special {
            kind: SpecialRecipeKind::DecoratedPot,
            result_hint: Some(parse_result(object, id)?),
        },
        "crafting_special_bannerduplicate" => special_recipe(SpecialRecipeKind::BannerDuplicate),
        "crafting_special_bookcloning" => special_recipe(SpecialRecipeKind::BookCloning),
        "crafting_special_firework_rocket" => special_recipe(SpecialRecipeKind::FireworkRocket),
        "crafting_special_firework_star" => special_recipe(SpecialRecipeKind::FireworkStar),
        "crafting_special_firework_star_fade" => {
            special_recipe(SpecialRecipeKind::FireworkStarFade)
        }
        "crafting_special_mapcloning" => special_recipe(SpecialRecipeKind::MapCloning),
        "crafting_special_mapextending" => special_recipe(SpecialRecipeKind::MapExtending),
        "crafting_special_repairitem" => special_recipe(SpecialRecipeKind::RepairItem),
        "crafting_special_shielddecoration" => special_recipe(SpecialRecipeKind::ShieldDecoration),
        other => {
            return Err(format!(
                "recipe {id} has unsupported type minecraft:{other}"
            ))
        }
    };

    Ok(RecipeHolder { id, recipe })
}

pub fn load_recipe_directory(recipe_dir: &std::path::Path) -> Result<RecipeManagerModel, String> {
    let mut paths = std::fs::read_dir(recipe_dir)
        .map_err(|err| {
            format!(
                "failed to read recipe directory {}: {err}",
                recipe_dir.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            format!(
                "failed to enumerate recipe directory {}: {err}",
                recipe_dir.display()
            )
        })?;
    paths.sort_by_key(|entry| entry.path());

    let mut recipes = Vec::new();
    for entry in paths {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("recipe path {} has no UTF-8 file stem", path.display()))?;
        let recipe_id = Box::leak(format!("minecraft:{stem}").into_boxed_str());
        let raw = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read recipe file {}: {err}", path.display()))?;
        recipes.push(load_recipe_json(recipe_id, &raw)?);
    }

    Ok(RecipeManagerModel::new(recipes))
}

fn special_recipe(kind: SpecialRecipeKind) -> RecipeKind {
    RecipeKind::Special {
        kind,
        result_hint: None,
    }
}

fn json_str<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {field}"))
}

fn parse_field_ingredient(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    id: &str,
) -> Result<IngredientSpec, String> {
    object
        .get(field)
        .ok_or_else(|| format!("recipe {id} is missing ingredient field {field}"))
        .and_then(parse_ingredient)
}

fn parse_optional_field_ingredient(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<IngredientSpec, String> {
    object
        .get(field)
        .map(parse_ingredient)
        .transpose()
        .map(|ingredient| ingredient.unwrap_or(IngredientSpec::Empty))
}

fn parse_ingredient(value: &serde_json::Value) -> Result<IngredientSpec, String> {
    if let Some(item) = value.as_str() {
        return Ok(IngredientSpec::Item(Box::leak(
            item.to_string().into_boxed_str(),
        )));
    }

    if let Some(items) = value.as_array() {
        let mut parsed = Vec::with_capacity(items.len());
        for item in items {
            let item = item
                .as_str()
                .ok_or_else(|| "ingredient array contains non-string entry".to_string())?;
            parsed.push(Box::leak(item.to_string().into_boxed_str()) as &'static str);
        }
        return Ok(IngredientSpec::AnyOf(parsed));
    }

    Err("ingredient must be a string or string array".to_string())
}

fn parse_ingredient_array(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    id: &str,
) -> Result<Vec<IngredientSpec>, String> {
    let values = object
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("recipe {id} is missing ingredient array {field}"))?;
    values.iter().map(parse_ingredient).collect()
}

fn parse_result(
    object: &serde_json::Map<String, serde_json::Value>,
    id: &str,
) -> Result<ItemAmount, String> {
    let result = object
        .get("result")
        .ok_or_else(|| format!("recipe {id} is missing result"))?;
    let result_object = result
        .as_object()
        .ok_or_else(|| format!("recipe {id} result must be an object"))?;
    let item = json_str(result_object, "id")?;
    let count = result_object
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1) as u32;
    Ok(ItemAmount {
        item: Box::leak(item.to_string().into_boxed_str()),
        count,
    })
}

fn collect_recipe_property_sets(recipes: &[RecipeHolder]) -> Vec<RecipePropertySet> {
    let mut furnace = Vec::new();
    let mut blast_furnace = Vec::new();
    let mut smoker = Vec::new();
    let mut campfire = Vec::new();
    let mut smithing_template = Vec::new();
    let mut smithing_base = Vec::new();
    let mut smithing_addition = Vec::new();

    for holder in recipes {
        match &holder.recipe {
            RecipeKind::Cooking {
                kind, ingredient, ..
            } => push_ingredient_items(
                match kind {
                    CookingKind::Smelting => &mut furnace,
                    CookingKind::Blasting => &mut blast_furnace,
                    CookingKind::Smoking => &mut smoker,
                    CookingKind::CampfireCooking => &mut campfire,
                },
                ingredient,
            ),
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
                push_ingredient_items(&mut smithing_template, template);
                push_ingredient_items(&mut smithing_base, base);
                push_ingredient_items(&mut smithing_addition, addition);
            }
            _ => {}
        }
    }

    vec![
        RecipePropertySet {
            key: "minecraft:furnace_input",
            accepted_items: furnace,
        },
        RecipePropertySet {
            key: "minecraft:blast_furnace_input",
            accepted_items: blast_furnace,
        },
        RecipePropertySet {
            key: "minecraft:smoker_input",
            accepted_items: smoker,
        },
        RecipePropertySet {
            key: "minecraft:campfire_input",
            accepted_items: campfire,
        },
        RecipePropertySet {
            key: "minecraft:smithing_template",
            accepted_items: smithing_template,
        },
        RecipePropertySet {
            key: "minecraft:smithing_base",
            accepted_items: smithing_base,
        },
        RecipePropertySet {
            key: "minecraft:smithing_addition",
            accepted_items: smithing_addition,
        },
    ]
}

fn push_ingredient_items(target: &mut Vec<&'static str>, ingredient: &IngredientSpec) {
    match ingredient {
        IngredientSpec::Empty => {}
        IngredientSpec::Item(item) => push_unique(target, item),
        IngredientSpec::AnyOf(items) => {
            for item in items {
                push_unique(target, item);
            }
        }
    }
}

fn push_unique(target: &mut Vec<&'static str>, item: &'static str) {
    if !target.contains(&item) {
        target.push(item);
    }
}

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
    pub const fn one(item: &'static str) -> Self {
        Self { item, count: 1 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentComponent {
    pub id: &'static str,
    pub level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorTrimComponent {
    pub material: &'static str,
    pub pattern: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithingComponentStack {
    pub item: &'static str,
    pub count: u32,
    pub custom_name: Option<&'static str>,
    pub enchantments: Vec<EnchantmentComponent>,
    pub trim: Option<ArmorTrimComponent>,
}

impl SmithingComponentStack {
    pub fn one(item: &'static str) -> Self {
        Self {
            item,
            count: 1,
            custom_name: None,
            enchantments: Vec::new(),
            trim: None,
        }
    }
}

pub fn smithing_transform_result(
    result_item: &'static str,
    base: &SmithingComponentStack,
) -> SmithingComponentStack {
    let mut result = base.clone();
    result.item = result_item;
    result.count = 1;
    result
}

pub fn smithing_trim_result(
    base: &SmithingComponentStack,
    material_from_addition: Option<&'static str>,
    pattern: &'static str,
) -> Option<SmithingComponentStack> {
    let material = material_from_addition?;
    let trim = ArmorTrimComponent { material, pattern };
    if base.trim.as_ref() == Some(&trim) {
        return None;
    }

    let mut result = base.clone();
    result.count = 1;
    result.trim = Some(trim);
    Some(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientSpec {
    Empty,
    Item(&'static str),
    AnyOf(Vec<&'static str>),
}

impl IngredientSpec {
    pub fn matches(&self, item: &'static str) -> bool {
        match self {
            IngredientSpec::Empty => false,
            IngredientSpec::Item(expected) => *expected == item,
            IngredientSpec::AnyOf(items) => items.contains(&item),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, IngredientSpec::Empty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementInfo {
    pub ingredients: Vec<IngredientSpec>,
    pub slots_to_ingredient_index: Vec<i32>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSmithingRecipeModel {
    pub show_notification: bool,
    pub placement_info: PlacementInfo,
}

impl SimpleSmithingRecipeModel {
    pub fn new(show_notification: bool, placement_info: PlacementInfo) -> Self {
        Self {
            show_notification,
            placement_info,
        }
    }

    pub fn group(&self) -> &'static str {
        ""
    }

    pub fn placement_info(&self) -> &PlacementInfo {
        &self.placement_info
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
            CookingKind::CampfireCooking => 100,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelValues {
    entries: Vec<(&'static str, i32)>,
}

impl FuelValues {
    pub fn vanilla() -> Self {
        Self::vanilla_with_base_unit(200)
    }

    pub fn vanilla_with_base_unit(base_unit: i32) -> Self {
        let mut entries = Vec::new();
        let mut add = |item, ticks| push_fuel(&mut entries, item, ticks);

        add("minecraft:lava_bucket", base_unit * 100);
        add("minecraft:coal_block", base_unit * 8 * 10);
        add("minecraft:blaze_rod", base_unit * 12);
        add("minecraft:coal", base_unit * 8);
        add("minecraft:charcoal", base_unit * 8);
        add("minecraft:oak_log", base_unit * 3 / 2);
        add("minecraft:bamboo_block", base_unit * 3 / 2);
        add("minecraft:oak_planks", base_unit * 3 / 2);
        add("minecraft:bamboo_mosaic", base_unit * 3 / 2);
        add("minecraft:oak_stairs", base_unit * 3 / 2);
        add("minecraft:bamboo_mosaic_stairs", base_unit * 3 / 2);
        add("minecraft:oak_slab", base_unit * 3 / 4);
        add("minecraft:bamboo_mosaic_slab", base_unit * 3 / 4);
        add("minecraft:oak_trapdoor", base_unit * 3 / 2);
        add("minecraft:oak_pressure_plate", base_unit * 3 / 2);
        add("minecraft:oak_shelf", base_unit * 3 / 2);
        add("minecraft:oak_fence", base_unit * 3 / 2);
        add("minecraft:oak_fence_gate", base_unit * 3 / 2);
        add("minecraft:note_block", base_unit * 3 / 2);
        add("minecraft:bookshelf", base_unit * 3 / 2);
        add("minecraft:chiseled_bookshelf", base_unit * 3 / 2);
        add("minecraft:lectern", base_unit * 3 / 2);
        add("minecraft:jukebox", base_unit * 3 / 2);
        add("minecraft:chest", base_unit * 3 / 2);
        add("minecraft:trapped_chest", base_unit * 3 / 2);
        add("minecraft:crafting_table", base_unit * 3 / 2);
        add("minecraft:daylight_detector", base_unit * 3 / 2);
        add("minecraft:white_banner", base_unit * 3 / 2);
        add("minecraft:bow", base_unit * 3 / 2);
        add("minecraft:fishing_rod", base_unit * 3 / 2);
        add("minecraft:ladder", base_unit * 3 / 2);
        add("minecraft:oak_sign", base_unit);
        add("minecraft:oak_hanging_sign", base_unit * 4);
        add("minecraft:wooden_shovel", base_unit);
        add("minecraft:wooden_sword", base_unit);
        add("minecraft:wooden_spear", base_unit);
        add("minecraft:wooden_hoe", base_unit);
        add("minecraft:wooden_axe", base_unit);
        add("minecraft:wooden_pickaxe", base_unit);
        add("minecraft:oak_door", base_unit);
        add("minecraft:oak_boat", base_unit * 6);
        add("minecraft:white_wool", base_unit / 2);
        add("minecraft:oak_button", base_unit / 2);
        add("minecraft:stick", base_unit / 2);
        add("minecraft:oak_sapling", base_unit / 2);
        add("minecraft:bowl", base_unit / 2);
        add("minecraft:white_carpet", 1 + base_unit / 3);
        add("minecraft:dried_kelp_block", 1 + base_unit * 20);
        add("minecraft:crossbow", base_unit * 3 / 2);
        add("minecraft:bamboo", base_unit / 4);
        add("minecraft:dead_bush", base_unit / 2);
        add("minecraft:short_dry_grass", base_unit / 2);
        add("minecraft:tall_dry_grass", base_unit / 2);
        add("minecraft:scaffolding", base_unit / 4);
        add("minecraft:loom", base_unit * 3 / 2);
        add("minecraft:barrel", base_unit * 3 / 2);
        add("minecraft:cartography_table", base_unit * 3 / 2);
        add("minecraft:fletching_table", base_unit * 3 / 2);
        add("minecraft:smithing_table", base_unit * 3 / 2);
        add("minecraft:composter", base_unit * 3 / 2);
        add("minecraft:azalea", base_unit / 2);
        add("minecraft:flowering_azalea", base_unit / 2);
        add("minecraft:mangrove_roots", base_unit * 3 / 2);
        add("minecraft:leaf_litter", base_unit / 2);

        Self { entries }
    }

    pub fn burn_duration(&self, item: Option<&str>) -> i32 {
        let Some(item) = item else {
            return 0;
        };
        self.entries
            .iter()
            .find_map(|(candidate, ticks)| (*candidate == item).then_some(*ticks))
            .unwrap_or(0)
    }

    pub fn is_fuel(&self, item: &str) -> bool {
        self.burn_duration(Some(item)) > 0
    }

    pub fn fuel_items(&self) -> Vec<&'static str> {
        self.entries.iter().map(|(item, _)| *item).collect()
    }
}

fn push_fuel(entries: &mut Vec<(&'static str, i32)>, item: &'static str, ticks: i32) {
    if let Some((_, existing)) = entries.iter_mut().find(|(candidate, _)| *candidate == item) {
        *existing = ticks;
    } else {
        entries.push((item, ticks));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnaceRecipeUsage {
    pub recipe_id: &'static str,
    pub times_used: i32,
    pub experience_millis: i32,
}

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
    pub fn recipe_type(&self) -> &'static str {
        match self {
            RecipeKind::Shaped { .. }
            | RecipeKind::Shapeless { .. }
            | RecipeKind::Special { .. } => "crafting",
            RecipeKind::Cooking { kind, .. } => match kind {
                CookingKind::Smelting => "smelting",
                CookingKind::Blasting => "blasting",
                CookingKind::Smoking => "smoking",
                CookingKind::CampfireCooking => "campfire_cooking",
            },
            RecipeKind::Stonecutting { .. } => "stonecutting",
            RecipeKind::SmithingTransform { .. } | RecipeKind::SmithingTrim { .. } => "smithing",
        }
    }

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

        let asymmetric = RecipeKind::Shaped {
            width: 2,
            height: 1,
            pattern: vec![
                Some(IngredientSpec::Item("minecraft:stick")),
                Some(IngredientSpec::Item("minecraft:coal")),
            ],
            result: ItemAmount::one("minecraft:torch"),
        };
        assert!(asymmetric.matches(2, 1, &[Some("minecraft:stick"), Some("minecraft:coal")]));
        assert!(!asymmetric.matches(2, 1, &[Some("minecraft:coal"), Some("minecraft:stick")]));

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
            (CookingKind::CampfireCooking, 100),
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
    fn fuel_values_match_vanilla_burn_time_defaults() {
        let fuels = FuelValues::vanilla();
        assert_eq!(fuels.burn_duration(Some("minecraft:lava_bucket")), 20_000);
        assert_eq!(fuels.burn_duration(Some("minecraft:coal_block")), 16_000);
        assert_eq!(fuels.burn_duration(Some("minecraft:blaze_rod")), 2_400);
        assert_eq!(fuels.burn_duration(Some("minecraft:coal")), 1_600);
        assert_eq!(fuels.burn_duration(Some("minecraft:charcoal")), 1_600);
        assert_eq!(fuels.burn_duration(Some("minecraft:oak_log")), 300);
        assert_eq!(fuels.burn_duration(Some("minecraft:oak_planks")), 300);
        assert_eq!(fuels.burn_duration(Some("minecraft:oak_slab")), 150);
        assert_eq!(fuels.burn_duration(Some("minecraft:oak_hanging_sign")), 800);
        assert_eq!(fuels.burn_duration(Some("minecraft:oak_boat")), 1_200);
        assert_eq!(fuels.burn_duration(Some("minecraft:white_wool")), 100);
        assert_eq!(fuels.burn_duration(Some("minecraft:white_carpet")), 67);
        assert_eq!(
            fuels.burn_duration(Some("minecraft:dried_kelp_block")),
            4_001
        );
        assert_eq!(fuels.burn_duration(Some("minecraft:bamboo")), 50);
        assert_eq!(fuels.burn_duration(None), 0);
        assert_eq!(fuels.burn_duration(Some("minecraft:diamond")), 0);
        assert!(fuels.is_fuel("minecraft:stick"));
        assert!(!fuels.is_fuel("minecraft:bucket"));

        let faster = FuelValues::vanilla_with_base_unit(100);
        assert_eq!(faster.burn_duration(Some("minecraft:coal")), 800);
        assert_eq!(faster.burn_duration(Some("minecraft:white_carpet")), 34);
    }

    #[test]
    fn cooking_recipe_experience_and_fuel_interaction_follow_furnace_rules() {
        let recipe = RecipeKind::Cooking {
            kind: CookingKind::Smelting,
            ingredient: IngredientSpec::Item("minecraft:raw_iron"),
            result: ItemAmount::one("minecraft:iron_ingot"),
            experience_millis: 700,
            cooking_time: None,
        };
        let fuels = FuelValues::vanilla();

        assert_eq!(recipe.cooking_time(), Some(200));
        assert!(recipe.matches(1, 1, &[Some("minecraft:raw_iron")]));
        assert_eq!(fuels.burn_duration(Some("minecraft:coal")), 1_600);
        assert_eq!(fuels.burn_duration(Some("minecraft:stick")), 100);

        let usage = FurnaceRecipeUsage {
            recipe_id: "minecraft:iron_ingot_from_smelting_raw_iron",
            times_used: 3,
            experience_millis: 700,
        };
        assert_eq!(furnace_experience_to_award(&usage, 0.05), 3);
        assert_eq!(furnace_experience_to_award(&usage, 0.95), 2);
        assert_eq!(
            furnace_experience_to_award(
                &FurnaceRecipeUsage {
                    times_used: 0,
                    ..usage
                },
                0.0
            ),
            0
        );
    }

    #[test]
    fn stonecutter_selectable_recipes_filter_all_outputs_for_input() {
        let recipes = vec![
            StonecutterSelection {
                recipe_id: "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
                input: IngredientSpec::Item("minecraft:smooth_stone"),
                result: ItemAmount {
                    item: "minecraft:smooth_stone_slab",
                    count: 2,
                },
            },
            StonecutterSelection {
                recipe_id: "minecraft:stone_slab_from_stone_stonecutting",
                input: IngredientSpec::Item("minecraft:stone"),
                result: ItemAmount {
                    item: "minecraft:stone_slab",
                    count: 2,
                },
            },
            StonecutterSelection {
                recipe_id: "minecraft:stone_bricks_from_stone_stonecutting",
                input: IngredientSpec::AnyOf(vec!["minecraft:stone"]),
                result: ItemAmount::one("minecraft:stone_bricks"),
            },
        ];

        let selected = stonecutter_recipes_for_input(&recipes, "minecraft:smooth_stone");
        assert_eq!(selected.len(), 1);
        assert_eq!(
            selected[0].recipe_id,
            "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting"
        );
        assert_eq!(
            selected[0].result,
            ItemAmount {
                item: "minecraft:smooth_stone_slab",
                count: 2,
            }
        );

        let stone_outputs = stonecutter_recipes_for_input(&recipes, "minecraft:stone");
        assert_eq!(stone_outputs.len(), 2);
        assert!(stone_outputs
            .iter()
            .any(|recipe| recipe.recipe_id == "minecraft:stone_slab_from_stone_stonecutting"));
        assert!(stone_outputs
            .iter()
            .any(|recipe| recipe.recipe_id == "minecraft:stone_bricks_from_stone_stonecutting"));
    }

    #[test]
    fn smithing_transform_preserves_original_components() {
        let base = SmithingComponentStack {
            item: "minecraft:diamond_sword",
            count: 1,
            custom_name: Some("Silk Edge"),
            enchantments: vec![
                EnchantmentComponent {
                    id: "minecraft:sharpness",
                    level: 5,
                },
                EnchantmentComponent {
                    id: "minecraft:unbreaking",
                    level: 3,
                },
            ],
            trim: None,
        };

        let result = smithing_transform_result("minecraft:netherite_sword", &base);
        assert_eq!(result.item, "minecraft:netherite_sword");
        assert_eq!(result.count, 1);
        assert_eq!(result.custom_name, Some("Silk Edge"));
        assert_eq!(result.enchantments, base.enchantments);
    }

    #[test]
    fn smithing_trim_applies_material_and_pattern_components() {
        let base = SmithingComponentStack::one("minecraft:iron_chestplate");
        let trimmed = smithing_trim_result(&base, Some("minecraft:amethyst"), "minecraft:spire")
            .expect("valid trim material should produce a trimmed copy");

        assert_eq!(trimmed.item, "minecraft:iron_chestplate");
        assert_eq!(trimmed.count, 1);
        assert_eq!(
            trimmed.trim,
            Some(ArmorTrimComponent {
                material: "minecraft:amethyst",
                pattern: "minecraft:spire",
            })
        );
        assert_eq!(
            smithing_trim_result(&trimmed, Some("minecraft:amethyst"), "minecraft:spire"),
            None
        );
        assert_eq!(smithing_trim_result(&base, None, "minecraft:spire"), None);
    }

    #[test]
    fn placement_info_matches_vanilla_slot_index_contracts() {
        let single = PlacementInfo::create(IngredientSpec::Item("minecraft:stone"));
        assert_eq!(
            single.ingredients,
            vec![IngredientSpec::Item("minecraft:stone")]
        );
        assert_eq!(single.slots_to_ingredient_index, vec![0]);
        assert!(!single.is_impossible_to_place());

        let smithing_transform = PlacementInfo::create_from_optionals(vec![
            None,
            Some(IngredientSpec::Item("minecraft:diamond_sword")),
            Some(IngredientSpec::Item("minecraft:netherite_ingot")),
        ]);
        assert_eq!(
            smithing_transform.ingredients,
            vec![
                IngredientSpec::Item("minecraft:diamond_sword"),
                IngredientSpec::Item("minecraft:netherite_ingot"),
            ]
        );
        assert_eq!(
            smithing_transform.slots_to_ingredient_index,
            vec![PlacementInfo::EMPTY_SLOT, 0, 1]
        );

        let impossible = PlacementInfo::create_list(vec![
            IngredientSpec::Item("minecraft:stick"),
            IngredientSpec::Empty,
        ]);
        assert!(impossible.is_impossible_to_place());

        let simple = SimpleSmithingRecipeModel::new(true, smithing_transform.clone());
        assert_eq!(simple.group(), "");
        assert!(simple.show_notification);
        assert_eq!(simple.placement_info(), &smithing_transform);
    }

    #[test]
    fn recipe_manager_indexes_by_type_key_and_matching_input() {
        let manager = RecipeManagerModel::new(vec![
            RecipeHolder {
                id: "minecraft:crafting_table",
                recipe: RecipeKind::Shaped {
                    width: 2,
                    height: 2,
                    pattern: vec![
                        Some(IngredientSpec::Item("minecraft:oak_planks")),
                        Some(IngredientSpec::Item("minecraft:oak_planks")),
                        Some(IngredientSpec::Item("minecraft:oak_planks")),
                        Some(IngredientSpec::Item("minecraft:oak_planks")),
                    ],
                    result: ItemAmount::one("minecraft:crafting_table"),
                },
            },
            RecipeHolder {
                id: "minecraft:firework_star",
                recipe: RecipeKind::Shapeless {
                    ingredients: vec![
                        IngredientSpec::Item("minecraft:gunpowder"),
                        IngredientSpec::AnyOf(vec!["minecraft:red_dye", "minecraft:blue_dye"]),
                    ],
                    result: ItemAmount::one("minecraft:firework_star"),
                },
            },
            RecipeHolder {
                id: "minecraft:iron_ingot_from_smelting_raw_iron",
                recipe: RecipeKind::Cooking {
                    kind: CookingKind::Smelting,
                    ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                    result: ItemAmount::one("minecraft:iron_ingot"),
                    experience_millis: 700,
                    cooking_time: None,
                },
            },
            RecipeHolder {
                id: "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
                recipe: RecipeKind::Stonecutting {
                    ingredient: IngredientSpec::Item("minecraft:smooth_stone"),
                    result: ItemAmount {
                        item: "minecraft:smooth_stone_slab",
                        count: 2,
                    },
                },
            },
        ]);

        assert_eq!(manager.recipe_map().values().len(), 4);
        assert_eq!(
            manager
                .recipe_map()
                .by_key("minecraft:crafting_table")
                .unwrap()
                .recipe
                .serializer(),
            "crafting_shaped"
        );
        assert_eq!(manager.recipe_map().by_type("crafting").len(), 2);
        assert_eq!(manager.recipe_map().by_type("smelting").len(), 1);

        let shaped = manager.recipe_map().get_recipe_for(
            "crafting",
            2,
            2,
            &[
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
            ],
        );
        assert_eq!(shaped.unwrap().id, "minecraft:crafting_table");
        assert!(manager
            .recipe_map()
            .get_recipe_for("crafting", 2, 2, &[None, None, None, None])
            .is_none());

        let cooking =
            manager
                .recipe_map()
                .get_recipe_for("smelting", 1, 1, &[Some("minecraft:raw_iron")]);
        assert_eq!(
            cooking.unwrap().id,
            "minecraft:iron_ingot_from_smelting_raw_iron"
        );
        assert_eq!(
            manager
                .property_set("minecraft:furnace_input")
                .accepted_items,
            vec!["minecraft:raw_iron"]
        );
        assert_eq!(manager.stonecutter_recipes().len(), 1);
    }

    #[test]
    fn recipe_manager_reload_replaces_indexes_and_recipe_access_sets() {
        let mut manager = RecipeManagerModel::new(vec![RecipeHolder {
            id: "minecraft:iron_ingot_from_smelting_raw_iron",
            recipe: RecipeKind::Cooking {
                kind: CookingKind::Smelting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: None,
            },
        }]);

        assert!(manager
            .recipe_map()
            .by_key("minecraft:iron_ingot_from_smelting_raw_iron")
            .is_some());
        manager.reload(vec![RecipeHolder {
            id: "minecraft:netherite_sword_smithing",
            recipe: RecipeKind::SmithingTransform {
                template: IngredientSpec::Item("minecraft:netherite_upgrade_smithing_template"),
                base: IngredientSpec::Item("minecraft:diamond_sword"),
                addition: IngredientSpec::Item("minecraft:netherite_ingot"),
                result: ItemAmount::one("minecraft:netherite_sword"),
            },
        }]);

        assert!(manager
            .recipe_map()
            .by_key("minecraft:iron_ingot_from_smelting_raw_iron")
            .is_none());
        assert_eq!(manager.recipe_map().by_type("smithing").len(), 1);
        assert_eq!(
            manager
                .property_set("minecraft:smithing_template")
                .accepted_items,
            vec!["minecraft:netherite_upgrade_smithing_template"]
        );
        assert_eq!(
            manager
                .property_set("minecraft:smithing_base")
                .accepted_items,
            vec!["minecraft:diamond_sword"]
        );
        assert!(manager.stonecutter_recipes().is_empty());
    }

    #[test]
    fn recipe_json_loader_decodes_representative_vanilla_files() {
        let shaped = load_recipe_json(
            "minecraft:crafting_table",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/recipe/crafting_table.json"
            ),
        )
        .expect("crafting table recipe should decode");
        assert_eq!(shaped.recipe.serializer(), "crafting_shaped");
        assert_eq!(shaped.recipe.recipe_type(), "crafting");

        let smelting = load_recipe_json(
            "minecraft:iron_ingot_from_smelting_raw_iron",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/recipe/iron_ingot_from_smelting_raw_iron.json"
            ),
        )
        .expect("smelting recipe should decode");
        assert_eq!(smelting.recipe.serializer(), "smelting");
        assert_eq!(smelting.recipe.cooking_time(), Some(200));

        let stonecutting = load_recipe_json(
            "minecraft:smooth_stone_slab_from_smooth_stone_stonecutting",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/recipe/smooth_stone_slab_from_smooth_stone_stonecutting.json"
            ),
        )
        .expect("stonecutting recipe should decode");
        assert_eq!(stonecutting.recipe.serializer(), "stonecutting");
        assert_eq!(
            stonecutting.recipe.assemble(),
            Some(ItemAmount {
                item: "minecraft:smooth_stone_slab",
                count: 2,
            })
        );
    }

    #[test]
    fn recipe_manager_loads_all_vanilla_recipe_json_files() {
        let manager = load_recipe_directory(std::path::Path::new(
            "../decompiled-server-26.1.2/data/minecraft/recipe",
        ))
        .expect("vanilla recipe directory should load");
        assert_eq!(manager.recipe_map().values().len(), 1515);
        assert_eq!(manager.recipe_map().by_type("crafting").len(), 1094);
        assert_eq!(manager.recipe_map().by_type("smelting").len(), 73);
        assert_eq!(manager.recipe_map().by_type("stonecutting").len(), 275);
        assert_eq!(manager.stonecutter_recipes().len(), 275);
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
