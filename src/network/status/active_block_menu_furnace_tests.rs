//! Live-wiring tests for the persistent block menus: furnace result XP,
//! `FurnaceFuelSlot` placement rules and `Container.startOpen` registration.
//! Java references: `FurnaceResultSlot.onTake`,
//! `AbstractFurnaceBlockEntity.awardUsedRecipesAndPopExperience`,
//! `FurnaceFuelSlot.mayPlace` / `getMaxStackSize`,
//! `ChestMenu` / `ShulkerBoxMenu` `startOpen` / `removed`.

use super::*;
use crate::live_block_entities::lifecycle::sync_block_entity_after_set_block;
use crate::recipe_system::load_recipe_directory;
use crate::storage::chunk::LevelChunk;
use crate::storage::region::ChunkPos;

const POS: crate::block_update::BlockPos = crate::block_update::BlockPos { x: 3, y: 64, z: 5 };
const IRON_RECIPE: &str = "minecraft:iron_ingot_from_smelting_iron_ore";

fn click(
    container_id: i32,
    state_id: i32,
    slot_num: i16,
    button_num: i8,
) -> ServerboundContainerClickPacket {
    ServerboundContainerClickPacket {
        container_id,
        state_id,
        slot_num,
        button_num,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: crate::network::play::HashedStack::empty(),
    }
}

fn recipes() -> crate::recipe_system::RecipeManagerModel {
    load_recipe_directory(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla-data/data/minecraft/recipe"),
    )
    .expect("bundled recipes load")
}

/// A cache holding one block of `state` at [`POS`] with its block entity.
fn cache_with_block(state: &str) -> GeneratedChunkCache {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(POS.x, POS.y, POS.z, state);
    sync_block_entity_after_set_block(&mut chunk, POS, None, state);
    let cache = GeneratedChunkCache::default();
    cache
        .chunks
        .lock()
        .unwrap()
        .insert(chunk.pos, std::sync::Arc::new(chunk));
    cache
}

fn stored_tag(cache: &GeneratedChunkCache, layout: &WorldLayout) -> Tag {
    cache
        .block_entity_nbt_at(layout.root(), 0, POS)
        .expect("block entity")
}

fn furnace_with_result_and_recipes_used(cache: &GeneratedChunkCache, layout: &WorldLayout) {
    let tag = stored_tag(cache, layout);
    let Tag::Compound(mut fields) = tag else {
        panic!("compound");
    };
    fields.retain(|(key, _)| key != "Items" && key != "RecipesUsed");
    fields.push((
        "Items".to_string(),
        Tag::List(vec![Tag::Compound(vec![
            ("Slot".to_string(), Tag::Byte(2)),
            ("id".to_string(), Tag::String("minecraft:iron_ingot".to_string())),
            ("count".to_string(), Tag::Int(3)),
        ])]),
    ));
    fields.push((
        "RecipesUsed".to_string(),
        Tag::Compound(vec![(IRON_RECIPE.to_string(), Tag::Int(3))]),
    ));
    cache.set_block_entity_nbt(layout.root(), 0, POS, Tag::Compound(fields));
}

fn open_furnace(
    cache: &GeneratedChunkCache,
    layout: &WorldLayout,
    recipes: &crate::recipe_system::RecipeManagerModel,
) -> ActiveBlockMenu {
    ActiveBlockMenu::open(
        4,
        POS,
        LiveBlockMenuKind::Furnace {
            block_entity_id: "minecraft:furnace",
        },
        layout,
        0,
        cache,
        recipes.recipe_map(),
    )
}

fn recipes_used(tag: &Tag) -> Option<Tag> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    fields
        .iter()
        .find(|(key, _)| key == "RecipesUsed")
        .map(|(_, value)| value.clone())
}

#[test]
fn taking_the_furnace_result_queues_experience_and_unlocks_the_used_recipes() {
    let recipes = recipes();
    assert!(recipes.recipe_map().by_key(IRON_RECIPE).is_some());
    let layout = WorldLayout::new(std::env::temp_dir().join("vibecraft-furnace-xp"));
    let cache = cache_with_block("minecraft:furnace[facing=north,lit=false]");
    furnace_with_result_and_recipes_used(&cache, &layout);
    let mut menu = open_furnace(&cache, &layout, &recipes);
    let mut state = PlaySessionState {
        x: 10.0,
        y: 70.0,
        z: -4.0,
        ..Default::default()
    };
    state.inventory_menu = crate::player_inventory::InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.recipe_map().clone(),
    );

    let instructions = menu.handle_click(&click(4, 0, 2, 0), &mut state, &layout, 0, &cache);

    assert_eq!(state.carried_item, ItemStack::new("minecraft:iron_ingot", 3));
    let awards = cache.experience_awards.drain();
    // 3 uses x 0.7 xp = 2.1: floor 2, plus one with probability 0.1.
    assert_eq!(awards.len(), 1);
    assert!((2..=3).contains(&awards[0].amount), "got {}", awards[0].amount);
    assert_eq!(awards[0].pos, (10.0, 70.0, -4.0), "orbs pop at the player");
    assert_eq!(
        recipes_used(&stored_tag(&cache, &layout)),
        Some(Tag::Compound(Vec::new())),
        "RecipesUsed is cleared"
    );
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::RecipesUnlocked(ids) if ids == &vec![IRON_RECIPE]
    )));
    assert!(cache.experience_awards.drain().is_empty());
}

#[test]
fn clicking_other_furnace_slots_keeps_the_recipes_used() {
    let recipes = recipes();
    let layout = WorldLayout::new(std::env::temp_dir().join("vibecraft-furnace-xp-keep"));
    let cache = cache_with_block("minecraft:furnace[facing=north,lit=false]");
    furnace_with_result_and_recipes_used(&cache, &layout);
    let mut menu = open_furnace(&cache, &layout, &recipes);
    let mut state = PlaySessionState::default();
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:iron_ore", 4));

    // Pick the ore up from the hotbar (menu slot 30 + ...) and drop it into
    // the ingredient slot: neither touches the result slot.
    menu.handle_click(&click(4, 0, 3 + 27, 0), &mut state, &layout, 0, &cache);
    menu.handle_click(&click(4, 1, 0, 0), &mut state, &layout, 0, &cache);

    assert!(cache.experience_awards.drain().is_empty());
    assert_ne!(
        recipes_used(&stored_tag(&cache, &layout)),
        Some(Tag::Compound(Vec::new()))
    );
}

#[test]
fn the_fuel_slot_only_accepts_fuel_and_single_buckets() {
    let recipes = recipes();
    let layout = WorldLayout::new(std::env::temp_dir().join("vibecraft-furnace-fuel"));
    let cache = cache_with_block("minecraft:furnace[facing=north,lit=false]");
    let mut menu = open_furnace(&cache, &layout, &recipes);
    let mut state = PlaySessionState {
        carried_item: ItemStack::new("minecraft:stone", 8),
        ..Default::default()
    };

    menu.handle_click(&click(4, 0, 1, 0), &mut state, &layout, 0, &cache);
    assert!(menu.slots[1].is_empty(), "stone is not fuel");
    assert_eq!(state.carried_item, ItemStack::new("minecraft:stone", 8));

    state.carried_item = ItemStack::new("minecraft:coal", 8);
    menu.handle_click(&click(4, menu.state_id, 1, 0), &mut state, &layout, 0, &cache);
    assert_eq!(menu.slots[1], ItemStack::new("minecraft:coal", 8));

    let mut buckets = open_furnace(&cache_with_block("minecraft:furnace[facing=north,lit=false]"), &layout, &recipes);
    state.carried_item = ItemStack::new("minecraft:bucket", 2);
    buckets.handle_click(&click(4, 0, 1, 0), &mut state, &layout, 0, &cache);
    assert_eq!(buckets.slots[1], ItemStack::new("minecraft:bucket", 1));
    assert_eq!(state.carried_item, ItemStack::new("minecraft:bucket", 1));
}

#[test]
fn the_result_slot_never_accepts_items() {
    let recipes = recipes();
    let layout = WorldLayout::new(std::env::temp_dir().join("vibecraft-furnace-result"));
    let cache = cache_with_block("minecraft:furnace[facing=north,lit=false]");
    let mut menu = open_furnace(&cache, &layout, &recipes);
    let mut state = PlaySessionState {
        carried_item: ItemStack::new("minecraft:coal", 8),
        ..Default::default()
    };
    menu.handle_click(&click(4, 0, 2, 0), &mut state, &layout, 0, &cache);
    assert!(menu.slots[2].is_empty());
}

fn open_persistent(block: &str, kind: LiveBlockMenuKind) -> (GeneratedChunkCache, ActiveBlockMenu) {
    let layout = WorldLayout::new(std::env::temp_dir().join("vibecraft-opener-menu"));
    let cache = cache_with_block(block);
    let menu = ActiveBlockMenu::open(1, POS, kind, &layout, 0, &cache, &RecipeMap::default());
    (cache, menu)
}

#[test]
fn opening_a_chest_registers_an_opener_that_the_menu_drop_releases() {
    let (cache, mut menu) = open_persistent(
        "minecraft:chest[facing=north,type=single,waterlogged=false]",
        LiveBlockMenuKind::Container {
            block_entity_id: "minecraft:chest",
            slot_count: 27,
        },
    );
    let state = PlaySessionState::default();
    menu.start_open(&cache.container_openers, &state);
    assert_eq!(cache.container_openers.users_at(POS).len(), 1);
    assert_eq!(cache.container_openers.users_at(POS)[0].range, 4.5);

    let menu_clone = menu.clone();
    menu.close(&mut PlaySessionState::default());
    assert_eq!(
        cache.container_openers.users_at(POS).len(),
        1,
        "a clone of the menu still holds the container open"
    );
    drop(menu_clone);
    assert!(cache.container_openers.users_at(POS).is_empty());
    let events = cache.container_openers.drain_events();
    assert_eq!(events.len(), 2, "one start, one stop: {events:?}");
}

#[test]
fn spectators_and_non_counting_containers_do_not_register() {
    let (cache, mut menu) = open_persistent(
        "minecraft:barrel[facing=north,open=false]",
        LiveBlockMenuKind::Container {
            block_entity_id: "minecraft:barrel",
            slot_count: 27,
        },
    );
    let spectator = PlaySessionState {
        game_mode: GameMode::Spectator,
        ..Default::default()
    };
    menu.start_open(&cache.container_openers, &spectator);
    assert!(cache.container_openers.users_at(POS).is_empty());

    let creative = PlaySessionState {
        game_mode: GameMode::Creative,
        ..Default::default()
    };
    menu.start_open(&cache.container_openers, &creative);
    assert_eq!(cache.container_openers.users_at(POS)[0].range, 5.0);

    let (cache, mut dispenser) = open_persistent(
        "minecraft:dispenser[facing=north,triggered=false]",
        LiveBlockMenuKind::Container {
            block_entity_id: "minecraft:dispenser",
            slot_count: 9,
        },
    );
    dispenser.start_open(&cache.container_openers, &PlaySessionState::default());
    assert!(cache.container_openers.users_at(POS).is_empty());
}

#[test]
fn hopper_and_dispenser_menus_have_java_slot_counts() {
    for (block, slots) in [
        ("minecraft:hopper", 5),
        ("minecraft:dispenser", 9),
        ("minecraft:dropper", 9),
        ("minecraft:chest", 27),
    ] {
        let open = block_menu_open_for_block_id_for_tests(block);
        assert!(matches!(
            open.live_kind,
            LiveBlockMenuKind::Container { slot_count, .. } if slot_count == slots
        ));
    }
}

fn block_menu_open_for_block_id_for_tests(block: &str) -> super::super::block_menu_open::BlockMenuOpen {
    let state = crate::block_behavior::BlockStateModel {
        registry_id: block.to_string(),
        properties: BTreeMap::new(),
    };
    crate::network::status::block_menu_open::block_menu_open_for_state(&state).expect("menu")
}
