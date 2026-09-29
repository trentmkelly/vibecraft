//! Tests for the live block-entity ticker, lifecycle hook and furnace bridge.
//! Java references: `LevelChunk.setBlockState`, `Level.tickBlockEntities`,
//! `AbstractFurnaceBlockEntity.serverTick` / `setItem`, `FuelValues`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::furnace::apply_menu_slots;
use super::lifecycle::{block_entity_identity, sync_block_entity_after_set_block};
use super::*;
use crate::block_entity::{FurnaceBlockEntityKind, PotItemStack};
use crate::network::compression::CompressionState;
use crate::recipe_system::{load_recipe_directory, FuelValues};
use crate::storage::region::ChunkPos;

fn vanilla_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft")
        .join(relative)
}

fn fuel() -> FuelValues {
    FuelValues::from_vanilla_data(&vanilla_path("tags/item"))
}

fn recipes() -> crate::recipe_system::RecipeManagerModel {
    load_recipe_directory(&vanilla_path("recipe")).expect("bundled recipes load")
}

fn stack(id: &str, count: i32) -> Option<PotItemStack> {
    Some(PotItemStack {
        item_id: id.to_string(),
        count,
    })
}

const POS: BlockPos = BlockPos { x: 3, y: 64, z: 5 };

/// A chunk with a furnace at [`POS`] created through the lifecycle hook.
fn chunk_with_furnace(state: &str) -> LevelChunk {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(POS.x, POS.y, POS.z, state);
    sync_block_entity_after_set_block(&mut chunk, POS, None, state);
    chunk
}

fn cache_with(chunk: LevelChunk) -> GeneratedChunkCache {
    let cache = GeneratedChunkCache::default();
    cache
        .chunks
        .lock()
        .unwrap()
        .insert(chunk.pos, Arc::new(chunk));
    cache
}

fn furnace_tag_with(cache: &GeneratedChunkCache, slots: [Option<PotItemStack>; 3]) {
    furnace_kind_tag_with(cache, FurnaceBlockEntityKind::Furnace, slots);
}

fn furnace_kind_tag_with(
    cache: &GeneratedChunkCache,
    kind: FurnaceBlockEntityKind,
    slots: [Option<PotItemStack>; 3],
) {
    let tag = {
        let chunks = cache.chunks.lock().unwrap();
        let chunk = chunks.values().next().unwrap();
        chunk.block_entities[0].clone()
    };
    let recipes = recipes();
    let updated = apply_menu_slots(kind, &tag, &slots, recipes.recipe_map());
    cache.chunks.lock().unwrap().values_mut().for_each(|chunk| {
        Arc::make_mut(chunk).set_block_entity_nbt(updated.clone());
    });
}

fn stored_furnace(cache: &GeneratedChunkCache) -> AbstractFurnaceBlockEntity {
    let chunks = cache.chunks.lock().unwrap();
    let chunk = chunks.values().next().unwrap();
    AbstractFurnaceBlockEntity::load_additional(
        FurnaceBlockEntityKind::Furnace,
        &chunk.block_entities[0],
    )
}

use crate::block_entity::AbstractFurnaceBlockEntity;

#[test]
fn fuel_values_are_built_from_the_bundled_item_tags() {
    let fuel = fuel();
    assert_eq!(fuel.burn_duration(Some("minecraft:coal")), 1600);
    assert_eq!(fuel.burn_duration(Some("minecraft:oak_planks")), 300);
    assert_eq!(fuel.burn_duration(Some("minecraft:lava_bucket")), 20000);
    // `.remove(ItemTags.NON_FLAMMABLE_WOOD)`.
    assert_eq!(fuel.burn_duration(Some("minecraft:crimson_planks")), 0);
    assert_eq!(fuel.burn_duration(Some("minecraft:stone")), 0);
}

#[test]
fn placing_a_furnace_creates_its_block_entity_and_replacing_it_removes_it() {
    let mut chunk = chunk_with_furnace("minecraft:furnace[facing=north,lit=false]");
    let (pos, ty) = block_entity_identity(&chunk.block_entities[0]).expect("identity");
    assert_eq!((pos, ty), (POS, BlockEntityTypeId::Furnace));
    // A block-state change of the same block keeps the block entity.
    sync_block_entity_after_set_block(
        &mut chunk,
        POS,
        Some("minecraft:furnace"),
        "minecraft:furnace[facing=north,lit=true]",
    );
    assert_eq!(chunk.block_entities.len(), 1);
    // Replacing the block removes it (`LevelChunk.removeBlockEntity`).
    sync_block_entity_after_set_block(
        &mut chunk,
        POS,
        Some("minecraft:furnace"),
        "minecraft:stone",
    );
    assert!(chunk.block_entities.is_empty());
}

#[test]
fn switching_between_block_entity_blocks_replaces_the_block_entity() {
    let mut chunk = chunk_with_furnace("minecraft:furnace");
    sync_block_entity_after_set_block(
        &mut chunk,
        POS,
        Some("minecraft:furnace"),
        "minecraft:smoker",
    );
    let (_, ty) = block_entity_identity(&chunk.block_entities[0]).unwrap();
    assert_eq!(ty, BlockEntityTypeId::Smoker);
    assert_eq!(chunk.block_entities.len(), 1);
}

#[test]
fn cache_set_block_runs_the_lifecycle_hook() {
    let cache = cache_with(LevelChunk::empty(ChunkPos { x: 0, z: 0 }));
    let root = std::env::temp_dir().join("vibecraft-live-be-set-block");
    cache.set_block(&root, 0, POS, "minecraft:chest[facing=north]");
    let tag = cache
        .block_entity_nbt_at(&root, 0, POS)
        .expect("chest block entity");
    assert_eq!(
        block_entity_identity(&tag).map(|(_, ty)| ty),
        Some(BlockEntityTypeId::Chest)
    );
    cache.set_block(&root, 0, POS, "minecraft:air");
    assert!(cache.block_entity_nbt_at(&root, 0, POS).is_none());
}

#[test]
fn ticking_a_furnace_lights_it_cooks_and_produces_output() {
    let cache = cache_with(chunk_with_furnace(
        "minecraft:furnace[facing=north,lit=false]",
    ));
    furnace_tag_with(
        &cache,
        [
            stack("minecraft:raw_iron", 1),
            stack("minecraft:coal", 1),
            None,
        ],
    );
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };

    // First tick: the fuel is consumed and the block flips to `lit=true`.
    let changes = tick_cached_block_entities(&cache, &env);
    assert_eq!(
        changes,
        vec![BlockStateChange {
            pos: POS,
            state: "minecraft:furnace[facing=north,lit=true]".to_string(),
        }]
    );
    let furnace = stored_furnace(&cache);
    assert!(furnace.is_lit());
    assert!(furnace.items[AbstractFurnaceBlockEntity::FUEL_SLOT].is_none());
    assert!(cache
        .dirty
        .lock()
        .unwrap()
        .contains(&ChunkPos { x: 0, z: 0 }));

    // The block state is applied by the caller; the remaining 199 ticks finish the cook.
    let root = std::env::temp_dir().join("vibecraft-live-be-cook");
    apply_block_state_changes(&cache, &root, 0, &WorldPacketBus::default(), &changes);
    for _ in 0..199 {
        assert!(tick_cached_block_entities(&cache, &env).is_empty());
    }
    let furnace = stored_furnace(&cache);
    assert_eq!(
        furnace.items[AbstractFurnaceBlockEntity::RESULT_SLOT],
        stack("minecraft:iron_ingot", 1)
    );
    assert!(furnace.items[AbstractFurnaceBlockEntity::INGREDIENT_SLOT].is_none());
    assert_eq!(furnace.recipes_used.values().sum::<i32>(), 1);
    // The block entity survives the lit-state rewrite done by `set_block`.
    assert!(cache.block_entity_nbt_at(&root, 0, POS).is_some());
}

#[test]
fn an_idle_furnace_is_not_rewritten_or_marked_dirty() {
    let cache = cache_with(chunk_with_furnace(
        "minecraft:furnace[facing=north,lit=false]",
    ));
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };
    assert!(tick_cached_block_entities(&cache, &env).is_empty());
    assert!(cache.dirty.lock().unwrap().is_empty());
}

#[test]
fn a_furnace_with_an_invalid_block_state_is_skipped() {
    let mut chunk = chunk_with_furnace("minecraft:furnace[facing=north,lit=false]");
    chunk.set_block_state(POS.x, POS.y, POS.z, "minecraft:stone");
    let cache = cache_with(chunk);
    furnace_tag_with(
        &cache,
        [
            stack("minecraft:raw_iron", 1),
            stack("minecraft:coal", 1),
            None,
        ],
    );
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };
    assert!(tick_cached_block_entities(&cache, &env).is_empty());
    assert!(!stored_furnace(&cache).is_lit());
}

#[test]
fn ticking_preserves_foreign_fields_and_item_components() {
    let mut chunk = chunk_with_furnace("minecraft:furnace[facing=north,lit=false]");
    let Tag::Compound(fields) = &mut chunk.block_entities[0] else {
        panic!("compound");
    };
    fields.retain(|(key, _)| key != "Items" && key != "cooking_total_time");
    fields.push(("CustomName".to_string(), Tag::String("Smelty".to_string())));
    fields.push((
        "Items".to_string(),
        Tag::List(vec![
            Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(0)),
                (
                    "id".to_string(),
                    Tag::String("minecraft:raw_iron".to_string()),
                ),
                ("count".to_string(), Tag::Int(2)),
            ]),
            Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(1)),
                ("id".to_string(), Tag::String("minecraft:coal".to_string())),
                ("count".to_string(), Tag::Int(3)),
                ("components".to_string(), Tag::Compound(vec![])),
            ]),
        ]),
    ));
    fields.push(("cooking_total_time".to_string(), Tag::Short(200)));
    let cache = cache_with(chunk);
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };
    tick_cached_block_entities(&cache, &env);
    let chunks = cache.chunks.lock().unwrap();
    let Tag::Compound(fields) = &chunks.values().next().unwrap().block_entities[0] else {
        panic!("compound");
    };
    assert!(fields
        .iter()
        .any(|(k, v)| k == "CustomName" && *v == Tag::String("Smelty".into())));
    let items = fields
        .iter()
        .find(|(k, _)| k == "Items")
        .map(|(_, v)| v)
        .unwrap();
    let Tag::List(items) = items else {
        panic!("list")
    };
    assert_eq!(
        items.len(),
        2,
        "coal stack shrank to 2 but both stacks remain"
    );
    let Tag::Compound(coal) = &items[1] else {
        panic!("compound")
    };
    assert!(
        coal.iter().any(|(k, _)| k == "components"),
        "item components survive the write-back"
    );
    assert!(coal.iter().any(|(k, v)| k == "count" && *v == Tag::Int(2)));
}

#[test]
fn menu_writes_reset_cook_progress_only_when_the_ingredient_changes() {
    let recipes = recipes();
    let mut entity = AbstractFurnaceBlockEntity::furnace();
    entity.cooking_time_spent = 50;
    entity.cooking_total_time = 200;
    entity.items[0] = stack("minecraft:raw_iron", 1);
    let tag = {
        let mut tag = lifecycle::new_block_entity_tag(BlockEntityTypeId::Furnace, POS);
        let (Tag::Compound(fields), Tag::Compound(own)) = (&mut tag, entity.save_additional())
        else {
            panic!("compound")
        };
        fields.retain(|(k, _)| {
            ![
                "cooking_time_spent",
                "cooking_total_time",
                "Items",
                "RecipesUsed",
                "lit_time_remaining",
                "lit_total_time",
            ]
            .contains(&k.as_str())
        });
        fields.extend(own);
        tag
    };
    let load = |tag: &Tag| {
        AbstractFurnaceBlockEntity::load_additional(FurnaceBlockEntityKind::Furnace, tag)
    };

    // Topping up the same ingredient keeps progress (`isSameItemSameComponents`).
    let topped = apply_menu_slots(
        FurnaceBlockEntityKind::Furnace,
        &tag,
        &[stack("minecraft:raw_iron", 5), None, None],
        recipes.recipe_map(),
    );
    assert_eq!(load(&topped).cooking_time_spent, 50);

    // A different ingredient resets progress and picks up the recipe cook time.
    let swapped = apply_menu_slots(
        FurnaceBlockEntityKind::Furnace,
        &tag,
        &[stack("minecraft:beef", 1), None, None],
        recipes.recipe_map(),
    );
    let swapped = load(&swapped);
    assert_eq!(swapped.cooking_time_spent, 0);
    assert_eq!(swapped.cooking_total_time, 200);
}

#[test]
fn lit_state_changes_are_broadcast_on_the_world_bus() {
    let cache = cache_with(chunk_with_furnace(
        "minecraft:furnace[facing=north,lit=false]",
    ));
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(1);
    let root = std::env::temp_dir().join("vibecraft-live-be-bus");
    apply_block_state_changes(
        &cache,
        &root,
        0,
        &bus,
        &[BlockStateChange {
            pos: POS,
            state: "minecraft:furnace[facing=north,lit=true]".to_string(),
        }],
    );
    let mut out = Vec::new();
    let sent = subscription
        .drain_into(&mut out, CompressionState::disabled())
        .unwrap();
    assert_eq!(sent, 1);
    let mut cursor = &out[..];
    let _len = crate::network::varint::read_var_i32(&mut cursor).unwrap();
    let id = crate::network::varint::read_var_i32(&mut cursor).unwrap();
    assert_eq!(id, crate::network::play::CLIENTBOUND_BLOCK_UPDATE_PACKET_ID);
    assert_eq!(
        cache
            .chunks
            .lock()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .get_block_state_model(POS.x, POS.y, POS.z)
            .unwrap()
            .properties
            .get("lit")
            .map(String::as_str),
        Some("true")
    );
}

/// Ticks a cache holding one furnace-family block until the result slot is
/// filled, returning how many ticks that took (`None` after `limit` ticks).
fn ticks_until_output(
    state: &str,
    kind: FurnaceBlockEntityKind,
    slots: [Option<PotItemStack>; 3],
    limit: usize,
) -> Option<usize> {
    let cache = cache_with(chunk_with_furnace(state));
    furnace_kind_tag_with(&cache, kind, slots);
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };
    let root = std::env::temp_dir().join("vibecraft-live-be-speed");
    for tick in 1..=limit {
        let changes = tick_cached_block_entities(&cache, &env);
        apply_block_state_changes(&cache, &root, 0, &WorldPacketBus::default(), &changes);
        let chunks = cache.chunks.lock().unwrap();
        let tag = &chunks.values().next().unwrap().block_entities[0];
        let furnace = AbstractFurnaceBlockEntity::load_additional(kind, tag);
        if furnace.items[AbstractFurnaceBlockEntity::RESULT_SLOT].is_some() {
            return Some(tick);
        }
    }
    None
}

#[test]
fn blast_furnace_and_smoker_cook_twice_as_fast_as_the_furnace() {
    let furnace = ticks_until_output(
        "minecraft:furnace[facing=north,lit=false]",
        FurnaceBlockEntityKind::Furnace,
        [
            stack("minecraft:raw_iron", 1),
            stack("minecraft:coal", 1),
            None,
        ],
        400,
    );
    let blast = ticks_until_output(
        "minecraft:blast_furnace[facing=north,lit=false]",
        FurnaceBlockEntityKind::BlastFurnace,
        [
            stack("minecraft:raw_iron", 1),
            stack("minecraft:coal", 1),
            None,
        ],
        400,
    );
    let smoker = ticks_until_output(
        "minecraft:smoker[facing=north,lit=false]",
        FurnaceBlockEntityKind::Smoker,
        [stack("minecraft:beef", 1), stack("minecraft:coal", 1), None],
        400,
    );
    assert_eq!(furnace, Some(200));
    assert_eq!(blast, Some(100));
    assert_eq!(smoker, Some(100));
}

#[test]
fn blast_furnaces_and_smokers_only_smelt_their_own_recipe_types() {
    // Raw iron has a smelting and a blasting recipe but no smoking recipe.
    assert_eq!(
        ticks_until_output(
            "minecraft:smoker[facing=north,lit=false]",
            FurnaceBlockEntityKind::Smoker,
            [
                stack("minecraft:raw_iron", 1),
                stack("minecraft:coal", 1),
                None
            ],
            300,
        ),
        None
    );
    // Beef has no blasting recipe.
    assert_eq!(
        ticks_until_output(
            "minecraft:blast_furnace[facing=north,lit=false]",
            FurnaceBlockEntityKind::BlastFurnace,
            [stack("minecraft:beef", 1), stack("minecraft:coal", 1), None],
            300,
        ),
        None
    );
}

#[test]
fn invalid_fuel_items_never_light_the_furnace() {
    assert_eq!(
        ticks_until_output(
            "minecraft:furnace[facing=north,lit=false]",
            FurnaceBlockEntityKind::Furnace,
            [
                stack("minecraft:raw_iron", 1),
                stack("minecraft:stone", 8),
                None
            ],
            300,
        ),
        None
    );
}

#[test]
fn blast_furnace_fuel_lasts_half_as_long() {
    // Coal burns 1600 ticks in a furnace; blast furnaces and smokers halve it.
    let cache = cache_with(chunk_with_furnace(
        "minecraft:blast_furnace[facing=north,lit=false]",
    ));
    furnace_kind_tag_with(
        &cache,
        FurnaceBlockEntityKind::BlastFurnace,
        [
            stack("minecraft:raw_iron", 64),
            stack("minecraft:coal", 1),
            None,
        ],
    );
    let recipes = recipes();
    let fuel = fuel();
    let env = BlockEntityTickEnvironment {
        recipes: recipes.recipe_map(),
        fuel_values: &fuel,
    };
    tick_cached_block_entities(&cache, &env);
    let chunks = cache.chunks.lock().unwrap();
    let furnace = AbstractFurnaceBlockEntity::load_additional(
        FurnaceBlockEntityKind::BlastFurnace,
        &chunks.values().next().unwrap().block_entities[0],
    );
    assert_eq!(furnace.lit_total_time, 800);
    assert_eq!(furnace.lit_time_remaining, 800);
}
