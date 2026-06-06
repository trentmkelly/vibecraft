use super::*;
use crate::recipe_system::{IngredientSpec, ItemAmount, RecipeHolder, RecipeKind, RecipeMap};

#[test]
fn crafting_menu_repairs_two_damaged_tools_via_special_recipe() {
    use crate::item_properties::ItemComponent;
    use crate::recipe_system::SpecialRecipeKind;

    // A recipe map carrying only the repair_item CustomRecipe.
    let recipes = RecipeMap::create(vec![RecipeHolder {
        id: "minecraft:repair_item",
        recipe: RecipeKind::Special {
            kind: SpecialRecipeKind::RepairItem,
            result_hint: None,
        },
    }]);
    let mut player = PlayerInventory::new();
    let mut menu = CraftingMenu::new(recipes);

    let mut first = ItemStack::new("minecraft:diamond_pickaxe", 1);
    first.set_component(ItemComponent::MaxDamage(100));
    first.set_damage_value(80);
    let mut second = ItemStack::new("minecraft:diamond_pickaxe", 1);
    second.set_component(ItemComponent::MaxDamage(100));
    second.set_damage_value(90);
    menu.set_slot(1, first, &mut player); // grid slot 0
    menu.set_slot(2, second, &mut player); // grid slot 1

    // The result slot shows the repaired tool (durability 100, damage 65).
    let result = menu.get_slot(0, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:diamond_pickaxe");
    assert_eq!(result.max_damage(), 100);
    assert_eq!(result.damage_value(), 65);

    // Taking it consumes both inputs.
    let taken = menu.take_result();
    assert_eq!(taken.damage_value(), 65);
    assert!(menu.get_slot(1, &player).unwrap().is_empty());
    assert!(menu.get_slot(2, &player).unwrap().is_empty());
    assert!(menu.get_slot(0, &player).unwrap().is_empty());
}

#[test]
fn crafting_menu_assembles_firework_rocket_via_special_recipe() {
    use crate::item_properties::ItemComponent;
    use crate::recipe_system::SpecialRecipeKind;

    // The firework_rocket recipe carries result = firework_rocket x3.
    let recipes = RecipeMap::create(vec![RecipeHolder {
        id: "minecraft:firework_rocket",
        recipe: RecipeKind::Special {
            kind: SpecialRecipeKind::FireworkRocket,
            result_hint: Some(ItemAmount {
                item: "minecraft:firework_rocket",
                count: 3,
            }),
        },
    }]);
    let mut player = PlayerInventory::new();
    let mut menu = CraftingMenu::new(recipes);
    menu.set_slot(1, ItemStack::new("minecraft:paper", 1), &mut player); // grid 0
    menu.set_slot(2, ItemStack::new("minecraft:gunpowder", 1), &mut player); // grid 1
    menu.set_slot(3, ItemStack::new("minecraft:gunpowder", 1), &mut player); // grid 2

    let result = menu.get_slot(0, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:firework_rocket");
    assert_eq!(result.count(), 3, "result count comes from the recipe's result");
    assert_eq!(
        result.component("minecraft:fireworks"),
        Some(&ItemComponent::Fireworks {
            flight_duration: 2,
            explosions: Vec::new(),
        })
    );

    // Taking it consumes the paper and both gunpowder.
    let taken = menu.take_result();
    assert_eq!(taken.count(), 3);
    for grid_slot in 1..=3 {
        assert!(menu.get_slot(grid_slot, &player).unwrap().is_empty());
    }
}

#[test]
fn crafting_menu_transmute_preserves_input_components() {
    use crate::item_properties::ItemComponent;

    // Recolouring a shulker box (crafting_transmute) must keep its components
    // (e.g. stored contents / repair cost) — createWithOriginalComponents.
    let recipes = RecipeMap::create(vec![RecipeHolder {
        id: "minecraft:orange_shulker_box",
        recipe: RecipeKind::Transmute {
            input: IngredientSpec::Item("minecraft:white_shulker_box"),
            material: IngredientSpec::Item("minecraft:orange_dye"),
            min_material_count: 1,
            max_material_count: 1,
            result: ItemAmount {
                item: "minecraft:orange_shulker_box",
                count: 1,
            },
            add_material_count_to_result: false,
        },
    }]);
    let mut player = PlayerInventory::new();
    let mut menu = CraftingMenu::new(recipes);
    let mut shulker = ItemStack::new("minecraft:white_shulker_box", 1);
    shulker.set_component(ItemComponent::RepairCost(7)); // stand-in for preserved data
    menu.set_slot(1, shulker, &mut player); // grid 0
    menu.set_slot(2, ItemStack::new("minecraft:orange_dye", 1), &mut player); // grid 1

    let result = menu.get_slot(0, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:orange_shulker_box");
    assert_eq!(
        result.component("minecraft:repair_cost"),
        Some(&ItemComponent::RepairCost(7)),
        "transmute must preserve the input's components"
    );
}

#[test]
fn crafting_menu_imbue_copies_potion_contents_to_result() {
    use crate::item_properties::ItemComponent;

    // arrow x8 around a lingering potion -> tipped_arrow x8 carrying the potion.
    let recipes = RecipeMap::create(vec![RecipeHolder {
        id: "minecraft:tipped_arrow",
        recipe: RecipeKind::Imbue {
            source: IngredientSpec::Item("minecraft:lingering_potion"),
            material: IngredientSpec::Item("minecraft:arrow"),
            result: ItemAmount {
                item: "minecraft:tipped_arrow",
                count: 8,
            },
        },
    }]);
    let mut player = PlayerInventory::new();
    let mut menu = CraftingMenu::new(recipes);
    let mut potion = ItemStack::new("minecraft:lingering_potion", 1);
    potion.set_component(ItemComponent::PotionContents("minecraft:strength"));
    // grid slots 1..=9; centre (grid index 4) is menu slot 5.
    for menu_slot in [1, 2, 3, 4, 6, 7, 8, 9] {
        menu.set_slot(menu_slot, ItemStack::new("minecraft:arrow", 1), &mut player);
    }
    menu.set_slot(5, potion, &mut player);

    let result = menu.get_slot(0, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:tipped_arrow");
    assert_eq!(result.count(), 8);
    assert_eq!(
        result.component("minecraft:potion_contents"),
        Some(&ItemComponent::PotionContents("minecraft:strength"))
    );
}
