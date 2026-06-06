use super::*;

/// Recursively resolves a single tag into a flat, deduplicated list of item IDs.
///
/// `visiting` tracks the tags currently on the call stack so that circular
/// references are detected and skipped rather than causing infinite recursion.
pub(super) fn resolve_tag(
    tag_id: &str,
    raw: &std::collections::HashMap<String, Vec<String>>,
    resolved: &mut std::collections::HashMap<String, Vec<&'static str>>,
    visiting: &mut std::collections::HashSet<String>,
) -> Vec<&'static str> {
    // If already resolved, return the cached result.
    if let Some(items) = resolved.get(tag_id) {
        return items.clone();
    }

    // Cycle detected — return an empty list and let the caller skip this entry.
    if !visiting.insert(tag_id.to_string()) {
        return Vec::new();
    }

    let mut items: Vec<&'static str> = Vec::new();

    if let Some(raw_values) = raw.get(tag_id) {
        for value in raw_values.clone() {
            if let Some(ref_tag_id) = value.strip_prefix('#') {
                // This value is a reference to another tag; resolve it recursively.
                let inner = resolve_tag(ref_tag_id, raw, resolved, visiting);
                for item in inner {
                    push_unique(&mut items, item);
                }
            } else {
                // Plain item ID — intern the string and store it.
                let interned: &'static str = Box::leak(value.into_boxed_str());
                push_unique(&mut items, interned);
            }
        }
    }

    visiting.remove(tag_id);
    resolved.insert(tag_id.to_string(), items.clone());
    items
}

pub fn load_recipe_json(
    id: &'static str,
    raw: &str,
    tags: &ItemTagMap,
) -> Result<RecipeHolder, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|err| format!("failed to parse recipe {id} as JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("recipe {id} must be a JSON object"))?;
    let raw_recipe_type = json_str(object, "type")?;
    let recipe_type = raw_recipe_type
        .strip_prefix("minecraft:")
        .unwrap_or(raw_recipe_type);
    let recipe = parse_recipe_kind(id, recipe_type, object, tags)?;

    Ok(RecipeHolder { id, recipe })
}

fn parse_recipe_kind(
    id: &str,
    recipe_type: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
    let recipe = match recipe_type {
        "crafting_shaped" => parse_shaped_recipe(id, object, tags),
        "crafting_shapeless" => Ok(RecipeKind::Shapeless {
            ingredients: parse_ingredient_array(object, "ingredients", id, tags)?,
            result: parse_result(object, id)?,
        }),
        "smelting" | "blasting" | "smoking" | "campfire_cooking" => {
            parse_cooking_recipe(recipe_type, id, object, tags)
        }
        "stonecutting" => Ok(RecipeKind::Stonecutting {
            ingredient: parse_field_ingredient(object, "ingredient", id, tags)?,
            result: parse_result(object, id)?,
        }),
        "smithing_transform" => Ok(RecipeKind::SmithingTransform {
            template: parse_optional_field_ingredient(object, "template", tags)?,
            base: parse_field_ingredient(object, "base", id, tags)?,
            addition: parse_optional_field_ingredient(object, "addition", tags)?,
            result: parse_result(object, id)?,
        }),
        "smithing_trim" => Ok(RecipeKind::SmithingTrim {
            template: parse_field_ingredient(object, "template", id, tags)?,
            base: parse_field_ingredient(object, "base", id, tags)?,
            addition: parse_field_ingredient(object, "addition", id, tags)?,
            pattern: Box::leak(json_str(object, "pattern")?.to_string().into_boxed_str()),
        }),
        "crafting_transmute" => Ok(RecipeKind::Transmute {
            input: parse_field_ingredient(object, "input", id, tags)?,
            material: parse_field_ingredient(object, "material", id, tags)?,
            min_material_count: parse_material_count_bound(object, "min").unwrap_or(1),
            max_material_count: parse_material_count_bound(object, "max").unwrap_or(1),
            result: parse_result(object, id)?,
            add_material_count_to_result: object
                .get("add_material_count_to_result")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }),
        "crafting_imbue" => Ok(RecipeKind::Imbue {
            source: parse_field_ingredient(object, "source", id, tags)?,
            material: parse_field_ingredient(object, "material", id, tags)?,
            result: parse_result(object, id)?,
        }),
        "crafting_dye" => parse_dyed_item_recipe(id, object, tags),
        "crafting_decorated_pot" => parse_decorated_pot_recipe(id, object, tags),
        recipe_type if recipe_type.starts_with("crafting_special_") => {
            parse_crafting_special_recipe(recipe_type, id, object, tags)
        }
        other => {
            return Err(format!(
                "recipe {id} has unsupported type minecraft:{other}"
            ))
        }
    }?;
    Ok(recipe)
}

fn parse_shaped_recipe(
    id: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
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
        parse_shaped_recipe_row(id, row, width, key, tags, &mut pattern)?;
    }
    Ok(RecipeKind::Shaped {
        width,
        height,
        pattern,
        result: parse_result(object, id)?,
    })
}

fn parse_shaped_recipe_row(
    id: &str,
    row: &serde_json::Value,
    width: usize,
    key: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
    pattern: &mut Vec<Option<IngredientSpec>>,
) -> Result<(), String> {
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
                .ok_or_else(|| format!("shaped recipe {id} has unmapped key '{key_char}'"))
                .and_then(|v| parse_ingredient(v, tags))?;
            pattern.push(Some(ingredient));
        }
    }
    Ok(())
}

fn parse_cooking_recipe(
    recipe_type: &str,
    id: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
    let kind = match recipe_type {
        "smelting" => CookingKind::Smelting,
        "blasting" => CookingKind::Blasting,
        "smoking" => CookingKind::Smoking,
        "campfire_cooking" => CookingKind::CampfireCooking,
        other => return Err(format!("recipe {id} has unsupported cooking type {other}")),
    };
    Ok(RecipeKind::Cooking {
        kind,
        ingredient: parse_field_ingredient(object, "ingredient", id, tags)?,
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
        category: CookingBookCategory::from_id(
            object.get("category").and_then(serde_json::Value::as_str),
        ),
    })
}

fn parse_dyed_item_recipe(
    id: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
    parse_field_ingredient(object, "target", id, tags)?;
    parse_field_ingredient(object, "dye", id, tags)?;
    Ok(RecipeKind::Special {
        kind: SpecialRecipeKind::DyedItem,
        result_hint: Some(parse_result(object, id)?),
    })
}

fn parse_decorated_pot_recipe(
    id: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
    parse_field_ingredient(object, "back", id, tags)?;
    parse_field_ingredient(object, "left", id, tags)?;
    parse_field_ingredient(object, "right", id, tags)?;
    parse_field_ingredient(object, "front", id, tags)?;
    Ok(RecipeKind::Special {
        kind: SpecialRecipeKind::DecoratedPot,
        result_hint: Some(parse_result(object, id)?),
    })
}

fn parse_crafting_special_recipe(
    recipe_type: &str,
    id: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    tags: &ItemTagMap,
) -> Result<RecipeKind, String> {
    let recipe = match recipe_type {
        "crafting_special_bannerduplicate" => {
            parse_field_ingredient(object, "banner", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::BannerDuplicate, Some(result))
        }
        "crafting_special_bookcloning" => {
            parse_field_ingredient(object, "source", id, tags)?;
            parse_field_ingredient(object, "material", id, tags)?;
            let result = parse_result(object, id)?;
            parse_allowed_generations(object, id)?;
            special_recipe(SpecialRecipeKind::BookCloning, Some(result))
        }
        "crafting_special_firework_rocket" => {
            parse_field_ingredient(object, "shell", id, tags)?;
            parse_field_ingredient(object, "fuel", id, tags)?;
            parse_field_ingredient(object, "star", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::FireworkRocket, Some(result))
        }
        "crafting_special_firework_star" => {
            parse_shape_ingredients(object, id, tags)?;
            parse_field_ingredient(object, "trail", id, tags)?;
            parse_field_ingredient(object, "twinkle", id, tags)?;
            parse_field_ingredient(object, "fuel", id, tags)?;
            parse_field_ingredient(object, "dye", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::FireworkStar, Some(result))
        }
        "crafting_special_firework_star_fade" => {
            parse_field_ingredient(object, "target", id, tags)?;
            parse_field_ingredient(object, "dye", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::FireworkStarFade, Some(result))
        }
        "crafting_special_mapcloning" => special_recipe(SpecialRecipeKind::MapCloning, None),
        "crafting_special_mapextending" => {
            parse_field_ingredient(object, "map", id, tags)?;
            parse_field_ingredient(object, "material", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::MapExtending, Some(result))
        }
        "crafting_special_repairitem" => special_recipe(SpecialRecipeKind::RepairItem, None),
        "crafting_special_shielddecoration" => {
            parse_field_ingredient(object, "banner", id, tags)?;
            parse_field_ingredient(object, "target", id, tags)?;
            let result = parse_result(object, id)?;
            special_recipe(SpecialRecipeKind::ShieldDecoration, Some(result))
        }
        other => return Err(format!("recipe {id} has unsupported type minecraft:{other}")),
    };
    Ok(recipe)
}

pub fn load_recipe_directory(recipe_dir: &std::path::Path) -> Result<RecipeManagerModel, String> {
    // Item tags live at `<namespace>/tags/item/` relative to the recipe directory's
    // parent namespace directory.  Given `data/minecraft/recipe/` the tags are at
    // `data/minecraft/tags/item/`.
    let tag_dir = recipe_dir
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("tags")
        .join("item");
    let tags = load_item_tag_directory(&tag_dir);

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
        recipes.push(load_recipe_json(recipe_id, &raw, &tags)?);
    }

    Ok(RecipeManagerModel::new(recipes))
}

fn special_recipe(kind: SpecialRecipeKind, result_hint: Option<ItemAmount>) -> RecipeKind {
    RecipeKind::Special { kind, result_hint }
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
    tags: &ItemTagMap,
) -> Result<IngredientSpec, String> {
    object
        .get(field)
        .ok_or_else(|| format!("recipe {id} is missing ingredient field {field}"))
        .and_then(|v| parse_ingredient(v, tags))
}

fn parse_optional_field_ingredient(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    tags: &ItemTagMap,
) -> Result<IngredientSpec, String> {
    object
        .get(field)
        .map(|v| parse_ingredient(v, tags))
        .transpose()
        .map(|ingredient| ingredient.unwrap_or(IngredientSpec::Empty))
}

/// Parses a single ingredient value, resolving tag references (strings starting
/// with `#`) through the provided `ItemTagMap`.
///
/// A tag reference like `"#minecraft:oak_logs"` becomes
/// `IngredientSpec::AnyOf([oak_log, oak_wood, ...])`.  An unknown tag resolves
/// to an empty `AnyOf`, which will never match any item and causes the recipe to
/// be un-craftable — this is intentional: we'd rather have a recipe that never
/// fires than silently accept the wrong items.
///
/// A plain string like `"minecraft:stick"` becomes `IngredientSpec::Item(...)`.
/// An array of strings or tag references produces a merged `AnyOf` list.
fn parse_ingredient(
    value: &serde_json::Value,
    tags: &ItemTagMap,
) -> Result<IngredientSpec, String> {
    if let Some(item) = value.as_str() {
        return resolve_ingredient_string(item, tags);
    }

    if let Some(items) = value.as_array() {
        // An array may contain plain item IDs or tag references; collect all
        // resolved items into a single flat AnyOf list.
        let mut parsed: Vec<&'static str> = Vec::new();
        for item in items {
            let s = item
                .as_str()
                .ok_or_else(|| "ingredient array contains non-string entry".to_string())?;
            match resolve_ingredient_string(s, tags)? {
                IngredientSpec::Item(id) => push_unique(&mut parsed, id),
                IngredientSpec::AnyOf(ids) => {
                    for id in ids {
                        push_unique(&mut parsed, id);
                    }
                }
                IngredientSpec::Empty => {}
            }
        }
        return Ok(IngredientSpec::AnyOf(parsed));
    }

    Err("ingredient must be a string or string array".to_string())
}

/// Resolves a single ingredient string: either a tag reference (`#minecraft:...`)
/// or a plain item ID (`minecraft:stick`).
fn resolve_ingredient_string(s: &str, tags: &ItemTagMap) -> Result<IngredientSpec, String> {
    if let Some(tag_id) = s.strip_prefix('#') {
        // Tag reference — resolve through the tag map.
        let resolved = tags.resolve(tag_id);
        Ok(IngredientSpec::AnyOf(resolved.to_vec()))
    } else {
        // Plain item ID — intern and return.
        Ok(IngredientSpec::Item(Box::leak(
            s.to_string().into_boxed_str(),
        )))
    }
}

fn parse_ingredient_array(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    id: &str,
    tags: &ItemTagMap,
) -> Result<Vec<IngredientSpec>, String> {
    let values = object
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("recipe {id} is missing ingredient array {field}"))?;
    values.iter().map(|v| parse_ingredient(v, tags)).collect()
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

fn parse_material_count_bound(
    object: &serde_json::Map<String, serde_json::Value>,
    bound: &str,
) -> Option<u32> {
    object
        .get("material_count")
        .and_then(serde_json::Value::as_object)
        .and_then(|bounds| bounds.get(bound))
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as u32)
}

fn parse_allowed_generations(
    object: &serde_json::Map<String, serde_json::Value>,
    id: &str,
) -> Result<(), String> {
    let Some(value) = object.get("allowed_generations") else {
        return Ok(());
    };
    let bounds = value
        .as_object()
        .ok_or_else(|| format!("recipe {id} allowed_generations must be an object"))?;
    for field in ["min", "max"] {
        if let Some(value) = bounds.get(field) {
            let Some(bound) = value.as_u64() else {
                return Err(format!(
                    "recipe {id} allowed_generations.{field} must be an integer"
                ));
            };
            if bound > 2 {
                return Err(format!(
                    "recipe {id} allowed_generations.{field} is outside vanilla 0..=2"
                ));
            }
        }
    }
    Ok(())
}

fn parse_shape_ingredients(
    object: &serde_json::Map<String, serde_json::Value>,
    id: &str,
    tags: &ItemTagMap,
) -> Result<Vec<(&'static str, IngredientSpec)>, String> {
    let shapes = object
        .get("shapes")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("recipe {id} is missing shapes object"))?;
    let mut parsed = Vec::with_capacity(shapes.len());
    for (shape, ingredient) in shapes {
        match shape.as_str() {
            "small_ball" | "large_ball" | "star" | "creeper" | "burst" => {
                parsed.push((
                    Box::leak(shape.clone().into_boxed_str()) as &'static str,
                    parse_ingredient(ingredient, tags)?,
                ));
            }
            other => return Err(format!("recipe {id} has unknown firework shape {other}")),
        }
    }
    Ok(parsed)
}

#[cfg(test)]
pub(super) fn collect_recipe_property_sets(recipes: &[RecipeHolder]) -> Vec<RecipePropertySet> {
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
                ..
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

#[cfg(test)]
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
