//! Tests for the live hopper ticker.
//! Java references: `HopperBlockEntity.pushItemsTick` / `tryMoveItems` /
//! `ejectItems` / `suckInItems` / `addItem` / `tryMoveInItem`,
//! `WorldlyContainer` sided rules of furnaces and shulker boxes,
//! `ChestBlock.getContainer` (double chests), `BlockTags.DOES_NOT_BLOCK_HOPPERS`.

use std::collections::HashMap;
use std::sync::Arc;

use super::container::ChunkWorld;
use super::hopper::{HopperEntityEffects, HopperTicker};
use super::lifecycle::sync_block_entity_after_set_block;
use super::stack::Stack;
use super::*;
use crate::item_entity::DroppedItem;
use crate::recipe_system::{load_recipe_directory, FuelValues};
use crate::storage::chunk::LevelChunk;

const HOPPER: BlockPos = BlockPos { x: 3, y: 65, z: 5 };
const BELOW: BlockPos = BlockPos { x: 3, y: 64, z: 5 };
const ABOVE: BlockPos = BlockPos { x: 3, y: 66, z: 5 };

/// A world of one chunk plus everything a hopper pass needs.
struct Fixture {
    chunk: LevelChunk,
    items: WorldItemEntities,
    ticker: HopperTicker,
    recipes: crate::recipe_system::RecipeManagerModel,
    game_time: i64,
    dirtied: Vec<ChunkPos>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            chunk: LevelChunk::empty(ChunkPos { x: 0, z: 0 }),
            items: WorldItemEntities::new(),
            ticker: HopperTicker::default(),
            recipes: load_recipe_directory(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("vanilla-data/data/minecraft/recipe"),
            )
            .expect("bundled recipes load"),
            game_time: 0,
            dirtied: Vec::new(),
        }
    }

    fn place(&mut self, pos: BlockPos, state: &str) -> &mut Self {
        self.chunk.set_block_state(pos.x, pos.y, pos.z, state);
        sync_block_entity_after_set_block(&mut self.chunk, pos, None, state);
        self
    }

    fn hopper(&mut self, pos: BlockPos, facing: &str, enabled: bool) -> &mut Self {
        self.place(
            pos,
            &format!("minecraft:hopper[enabled={enabled},facing={facing}]"),
        )
    }

    fn chest(&mut self, pos: BlockPos, chest_type: &str, facing: &str) -> &mut Self {
        self.place(
            pos,
            &format!("minecraft:chest[facing={facing},type={chest_type},waterlogged=false]"),
        )
    }

    /// Replaces a block entity's `Items` with `(slot, stack)` entries.
    fn fill(&mut self, pos: BlockPos, stacks: &[(usize, Stack)]) -> &mut Self {
        let tag = self.tag(pos);
        let Tag::Compound(mut fields) = tag else {
            panic!("compound");
        };
        fields.retain(|(key, _)| key != "Items");
        fields.push((
            "Items".to_string(),
            Tag::List(stacks.iter().map(|(slot, s)| s.to_tag(*slot)).collect()),
        ));
        self.chunk.set_block_entity_nbt(Tag::Compound(fields));
        self
    }

    fn tag(&self, pos: BlockPos) -> Tag {
        self.chunk
            .block_entities
            .iter()
            .find(|tag| lifecycle::block_entity_identity(tag).map(|(at, _)| at) == Some(pos))
            .cloned()
            .expect("block entity")
    }

    /// The stacks of a container block entity by slot.
    fn slots(&self, pos: BlockPos, size: usize) -> Vec<Option<Stack>> {
        let mut slots = vec![None; size];
        if let Tag::Compound(fields) = self.tag(pos) {
            if let Some((_, Tag::List(items))) = fields.iter().find(|(k, _)| k == "Items") {
                for (slot, stack) in items.iter().filter_map(Stack::from_tag) {
                    slots[slot] = Some(stack);
                }
            }
        }
        slots
    }

    fn count(&self, pos: BlockPos, size: usize) -> i32 {
        self.slots(pos, size).iter().flatten().map(|s| s.count).sum()
    }

    fn cooldown(&self, pos: BlockPos) -> i32 {
        let Tag::Compound(fields) = self.tag(pos) else {
            panic!("compound");
        };
        fields
            .iter()
            .find_map(|(k, v)| match v {
                Tag::Int(v) if k == "TransferCooldown" => Some(*v),
                _ => None,
            })
            .unwrap_or(-1)
    }

    /// One `Level.tickBlockEntities` hopper pass.
    fn tick(&mut self) -> HopperEntityEffects {
        self.game_time += 1;
        let mut chunks = HashMap::new();
        chunks.insert(self.chunk.pos, Arc::new(std::mem::replace(
            &mut self.chunk,
            LevelChunk::empty(ChunkPos { x: 0, z: 0 }),
        )));
        let env = BlockEntityTickEnvironment {
            recipes: self.recipes.recipe_map(),
            fuel_values: FuelValues::shared_vanilla(),
        };
        let mut world = ChunkWorld::new(&mut chunks);
        let effects = self
            .ticker
            .tick(&mut world, &mut self.items, &env, self.game_time);
        self.dirtied.extend(world.into_dirtied());
        let arc = chunks.remove(&ChunkPos { x: 0, z: 0 }).expect("chunk");
        self.chunk = Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone());
        effects
    }

    fn ticks(&mut self, count: usize) {
        for _ in 0..count {
            self.tick();
        }
    }
}

fn coal(count: i32) -> Stack {
    Stack::new("minecraft:coal", count)
}

#[test]
fn a_hopper_pushes_one_item_per_eight_ticks_into_the_container_it_faces() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .fill(HOPPER, &[(0, coal(3))]);
    world.tick();
    assert_eq!(world.count(BELOW, 27), 1, "first tick: cooldown -1 -> 0, moves one");
    assert_eq!(world.cooldown(HOPPER), 8);
    world.ticks(7);
    assert_eq!(world.count(BELOW, 27), 1, "ticks 2..8 are on cooldown");
    world.tick();
    assert_eq!(world.count(BELOW, 27), 2, "the ninth tick moves the next item");
    world.ticks(8);
    assert_eq!(world.count(BELOW, 27), 3);
    assert_eq!(world.count(HOPPER, 5), 0);
}

#[test]
fn a_hopper_ejects_and_sucks_in_the_same_tick() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .chest(ABOVE, "single", "north")
        .fill(HOPPER, &[(0, coal(1))])
        .fill(ABOVE, &[(4, Stack::new("minecraft:stick", 5))]);
    world.tick();
    assert_eq!(world.slots(BELOW, 27)[0], Some(coal(1)), "pushed down");
    assert_eq!(world.slots(ABOVE, 27)[4], Some(Stack::new("minecraft:stick", 4)));
    assert_eq!(
        world.slots(HOPPER, 5)[0],
        Some(Stack::new("minecraft:stick", 1)),
        "pulled from above into the slot the pushed item left"
    );
}

#[test]
fn a_disabled_hopper_moves_nothing_but_still_counts_its_cooldown_down() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", false)
        .chest(BELOW, "single", "north")
        .fill(HOPPER, &[(0, coal(2))]);
    world.ticks(3);
    assert_eq!(world.count(BELOW, 27), 0);
    assert_eq!(world.count(HOPPER, 5), 2);
}

#[test]
fn an_idle_hopper_is_not_rewritten() {
    let mut world = Fixture::new();
    world.hopper(HOPPER, "down", true);
    world.tick();
    world.dirtied.clear();
    world.ticks(5);
    assert!(world.dirtied.is_empty(), "settled hopper must not dirty the chunk");
}

#[test]
fn a_full_target_container_rejects_the_transfer_without_changes() {
    let mut world = Fixture::new();
    let full: Vec<(usize, Stack)> = (0..27).map(|slot| (slot, Stack::new("minecraft:dirt", 64))).collect();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .fill(BELOW, &full)
        .fill(HOPPER, &[(0, Stack::new("minecraft:dirt", 1))]);
    world.tick();
    world.dirtied.clear();
    world.ticks(3);
    assert_eq!(world.count(HOPPER, 5), 1);
    assert!(world.dirtied.is_empty());
}

#[test]
fn item_components_survive_a_transfer() {
    let mut named = Stack::new("minecraft:diamond_sword", 1);
    named.extra.push((
        "components".to_string(),
        Tag::Compound(vec![(
            "minecraft:custom_name".to_string(),
            Tag::String("Excalibur".to_string()),
        )]),
    ));
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .fill(HOPPER, &[(2, named.clone())]);
    world.tick();
    assert_eq!(world.slots(BELOW, 27)[0], Some(named));
}

#[test]
fn stacks_with_different_components_do_not_merge() {
    let plain = Stack::new("minecraft:stick", 1);
    let mut named = Stack::new("minecraft:stick", 1);
    named.extra.push((
        "components".to_string(),
        Tag::Compound(vec![("minecraft:custom_name".to_string(), Tag::String("x".into()))]),
    ));
    assert!(!plain.same_item_same_components(&named));
    let mut reordered = named.clone();
    assert!(named.same_item_same_components(&reordered));
    reordered.count = 7;
    assert!(named.same_item_same_components(&reordered), "count is ignored");
}

#[test]
fn hoppers_walk_the_target_slots_in_order_merging_or_filling_the_first_fit() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .fill(BELOW, &[(0, coal(10)), (1, Stack::new("minecraft:stick", 1))])
        .fill(HOPPER, &[(0, coal(2))]);
    world.tick();
    let slots = world.slots(BELOW, 27);
    assert_eq!(slots[0], Some(coal(11)), "merged into the matching stack");
    assert_eq!(slots[1], Some(Stack::new("minecraft:stick", 1)));
    world.ticks(8);
    assert_eq!(world.slots(BELOW, 27)[0], Some(coal(12)));
}

#[test]
fn a_hopper_sucks_from_the_container_above_through_its_down_face() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .place(ABOVE, "minecraft:barrel[facing=north,open=false]")
        .fill(ABOVE, &[(0, Stack::new("minecraft:stick", 2))]);
    world.tick();
    assert_eq!(world.count(HOPPER, 5), 1);
    assert_eq!(world.count(ABOVE, 27), 1);
}

#[test]
fn furnace_faces_route_items_to_the_ingredient_fuel_and_result_slots() {
    // Hopper above a furnace pushes into the ingredient slot (UP face).
    let mut above = Fixture::new();
    above
        .hopper(HOPPER, "down", true)
        .place(BELOW, "minecraft:furnace[facing=north,lit=false]")
        .fill(HOPPER, &[(0, Stack::new("minecraft:iron_ore", 2))]);
    above.tick();
    assert_eq!(above.slots(BELOW, 3)[0], Some(Stack::new("minecraft:iron_ore", 1)));

    // A hopper at the side pushes fuel only into the fuel slot and refuses
    // non-fuel (`canPlaceItem(1)`).
    let side = BlockPos { x: 4, y: 64, z: 5 };
    let mut fuel = Fixture::new();
    fuel.hopper(side, "west", true)
        .place(BELOW, "minecraft:furnace[facing=north,lit=false]")
        .fill(side, &[(0, Stack::new("minecraft:stone", 1)), (1, coal(2))]);
    fuel.tick();
    assert_eq!(fuel.slots(BELOW, 3)[1], Some(coal(1)), "coal is fuel");
    assert_eq!(fuel.slots(BELOW, 3)[0], None, "sides never reach the ingredient");
    assert_eq!(fuel.count(side, 5), 2, "stone stayed behind, one coal left");
}

#[test]
fn a_hopper_below_a_furnace_takes_the_result_and_only_empty_fuel_buckets() {
    let mut world = Fixture::new();
    world
        .hopper(BELOW, "down", true)
        .place(HOPPER, "minecraft:furnace[facing=north,lit=false]")
        .fill(
            HOPPER,
            &[
                (1, coal(1)),
                (2, Stack::new("minecraft:iron_ingot", 3)),
            ],
        );
    world.tick();
    assert_eq!(
        world.slots(BELOW, 5)[0],
        Some(Stack::new("minecraft:iron_ingot", 1)),
        "result slot is offered first (SLOTS_FOR_DOWN = [2, 1])"
    );
    // With the result slot empty, coal in the fuel slot must stay.
    world.fill(HOPPER, &[(1, coal(1))]);
    world.ticks(9);
    assert_eq!(world.slots(HOPPER, 3)[1], Some(coal(1)));
    // But a spent (empty) bucket can be drained.
    world.fill(HOPPER, &[(1, Stack::new("minecraft:bucket", 1))]);
    world.ticks(9);
    assert_eq!(world.slots(HOPPER, 3)[1], None);
}

#[test]
fn shulker_boxes_refuse_shulker_boxes_from_a_hopper() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .place(BELOW, "minecraft:shulker_box[facing=up]")
        .fill(HOPPER, &[(0, Stack::new("minecraft:red_shulker_box", 1))]);
    world.ticks(3);
    assert_eq!(world.count(HOPPER, 5), 1);
    assert_eq!(world.count(BELOW, 27), 0);
    world.fill(HOPPER, &[(0, coal(1))]);
    world.ticks(9);
    assert_eq!(world.count(BELOW, 27), 1);
}

#[test]
fn a_double_chest_is_one_54_slot_container_with_the_right_half_first() {
    // `getConnectedDirection`: a RIGHT chest facing north looks
    // counter-clockwise (west) for its partner, which is the LEFT half.
    let right = BELOW;
    let left = BlockPos { x: 2, y: 64, z: 5 };
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(right, "right", "north")
        .chest(left, "left", "north");
    let full: Vec<(usize, Stack)> = (0..27).map(|slot| (slot, Stack::new("minecraft:dirt", 64))).collect();
    world.fill(right, &full).fill(HOPPER, &[(0, coal(1))]);
    world.tick();
    assert_eq!(world.slots(left, 27)[0], Some(coal(1)), "overflows into the second half");
}

#[test]
fn a_single_chest_next_to_another_is_not_merged() {
    let neighbour = BlockPos { x: 4, y: 64, z: 5 };
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .chest(neighbour, "single", "north")
        .fill(HOPPER, &[(0, coal(1))]);
    world.tick();
    assert_eq!(world.count(BELOW, 27), 1);
    assert_eq!(world.count(neighbour, 27), 0);
}

#[test]
fn a_hopper_chain_restarts_the_receiving_hoppers_cooldown() {
    let second = BlockPos { x: 3, y: 64, z: 5 };
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .hopper(second, "down", true)
        .fill(HOPPER, &[(0, coal(2))]);
    world.tick();
    // The lower hopper (later chunk order? same chunk: list order) received its
    // first item: `wasEmpty` -> `setCooldown(8 - skip)`.
    assert_eq!(world.count(second, 5), 1);
    assert!([7, 8].contains(&world.cooldown(second)), "got {}", world.cooldown(second));
}

#[test]
fn hoppers_absorb_item_entities_above_them() {
    let mut world = Fixture::new();
    world.hopper(HOPPER, "down", true);
    let eid = world.items.alloc_entity_id();
    world.items.entities.push(DroppedItem {
        entity_id: eid,
        item: "minecraft:coal",
        count: 3,
        x: 3.5,
        y: 66.0,
        z: 5.5,
        vel_x: 0.0,
        vel_y: 0.0,
        vel_z: 0.0,
        pickup_delay: 0,
        age: 0,
        target_uuid: None,
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    });
    let effects = world.tick();
    assert_eq!(world.slots(HOPPER, 5)[0], Some(coal(3)));
    assert_eq!(effects.removed, vec![eid]);
    assert!(world.items.entities.is_empty());
}

#[test]
fn a_partly_absorbed_item_entity_keeps_the_rest_and_reports_its_count() {
    let mut world = Fixture::new();
    world
        .hopper(HOPPER, "down", true)
        .fill(
            HOPPER,
            &(0..5)
                .map(|slot| (slot, Stack::new("minecraft:coal", 64)))
                .take(4)
                .chain([(4, coal(62))])
                .collect::<Vec<_>>(),
        );
    let eid = world.items.alloc_entity_id();
    world.items.entities.push(DroppedItem {
        entity_id: eid,
        item: "minecraft:coal",
        count: 5,
        x: 3.5,
        y: 66.0,
        z: 5.5,
        vel_x: 0.0,
        vel_y: 0.0,
        vel_z: 0.0,
        pickup_delay: 0,
        age: 0,
        target_uuid: None,
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    });
    let effects = world.tick();
    assert_eq!(world.count(HOPPER, 5), 5 * 64, "hopper filled up");
    assert_eq!(world.items.entities[0].count, 3);
    assert_eq!(effects.count_updates, vec![(eid, "minecraft:coal", 3)]);
    assert!(effects.removed.is_empty());
}

#[test]
fn a_full_block_above_stops_the_hopper_sucking_items_but_a_hopper_blocking_tag_does_not() {
    let mut blocked = Fixture::new();
    blocked.hopper(HOPPER, "down", true).place(ABOVE, "minecraft:stone");
    let eid = blocked.items.alloc_entity_id();
    let item = |y: f64| DroppedItem {
        entity_id: eid,
        item: "minecraft:coal",
        count: 1,
        x: 3.5,
        y,
        z: 5.5,
        vel_x: 0.0,
        vel_y: 0.0,
        vel_z: 0.0,
        pickup_delay: 0,
        age: 0,
        target_uuid: None,
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    };
    // Item inside the hopper's own block space (still in the suck box).
    blocked.items.entities.push(item(65.75));
    blocked.tick();
    assert_eq!(blocked.count(HOPPER, 5), 0, "stone above blocks the hopper");

    let mut open = Fixture::new();
    open.hopper(HOPPER, "down", true)
        .place(ABOVE, "minecraft:oak_slab[type=bottom,waterlogged=false]");
    open.items.entities.push(item(65.75));
    open.tick();
    assert_eq!(open.count(HOPPER, 5), 1, "a bottom slab is not a full collision block");
}

#[test]
fn item_entities_outside_the_suck_box_are_ignored() {
    let mut world = Fixture::new();
    world.hopper(HOPPER, "down", true);
    for (x, y) in [(3.5, 68.5), (5.5, 66.0), (3.5, 64.0)] {
        let eid = world.items.alloc_entity_id();
        world.items.entities.push(DroppedItem {
            entity_id: eid,
            item: "minecraft:coal",
            count: 1,
            x,
            y,
            z: 5.5,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            pickup_delay: 0,
            age: 0,
            target_uuid: None,
            health: crate::item_entity::ITEM_DEFAULT_HEALTH,
        });
    }
    world.tick();
    assert_eq!(world.count(HOPPER, 5), 0);
}

#[test]
fn containers_with_unresolved_loot_are_left_alone() {
    let mut world = Fixture::new();
    world.hopper(HOPPER, "down", true).chest(BELOW, "single", "north");
    let Tag::Compound(mut fields) = world.tag(BELOW) else {
        panic!("compound");
    };
    fields.push((
        "LootTable".to_string(),
        Tag::String("minecraft:chests/simple_dungeon".to_string()),
    ));
    world.chunk.set_block_entity_nbt(Tag::Compound(fields));
    world.fill(HOPPER, &[(0, coal(1))]);
    world.ticks(3);
    assert_eq!(world.count(HOPPER, 5), 1);
}

/// Drives the production ticker (`LiveBlockEntityTicker::tick`) instead of the
/// pass in isolation: the hopper runs off the shared cache and world items, and
/// the item entity it absorbs is announced on the world bus.
#[test]
fn the_live_ticker_runs_hoppers_and_broadcasts_absorbed_item_entities() {
    let mut fixture = Fixture::new();
    fixture
        .hopper(HOPPER, "down", true)
        .chest(BELOW, "single", "north")
        .fill(HOPPER, &[(0, coal(1))]);
    let cache = GeneratedChunkCache::default();
    cache
        .chunks
        .lock()
        .unwrap()
        .insert(fixture.chunk.pos, Arc::new(fixture.chunk.clone()));
    let world_items = Arc::new(Mutex::new(WorldItemEntities::new()));
    let eid = {
        let mut items = world_items.lock().unwrap();
        let eid = items.alloc_entity_id();
        items.entities.push(DroppedItem {
            entity_id: eid,
            item: "minecraft:stick",
            count: 1,
            x: 3.5,
            y: 66.0,
            z: 5.5,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            pickup_delay: 0,
            age: 0,
            target_uuid: None,
            health: crate::item_entity::ITEM_DEFAULT_HEALTH,
        });
        eid
    };
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(1);
    let mut ticker = LiveBlockEntityTicker::new(
        cache.clone(),
        Arc::new(load_recipe_directory(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("vanilla-data/data/minecraft/recipe"),
        )
        .expect("recipes")),
        Arc::new(std::env::temp_dir().join("vibecraft-live-hopper")),
        0,
        bus,
        Arc::clone(&world_items),
    );

    ticker.tick();

    let chunks = cache.chunks.lock().unwrap();
    let chunk = chunks.values().next().unwrap();
    let hopper_items = chunk
        .block_entities
        .iter()
        .filter_map(|tag| {
            let (pos, _) = lifecycle::block_entity_identity(tag)?;
            (pos == HOPPER).then_some(tag.clone())
        })
        .next()
        .unwrap();
    let Tag::Compound(fields) = hopper_items else {
        panic!("compound");
    };
    let Some((_, Tag::List(items))) = fields.iter().find(|(key, _)| key == "Items") else {
        panic!("Items");
    };
    // The coal was pushed down and the stick pulled in from above in one tick.
    assert_eq!(items.len(), 1);
    assert_eq!(Stack::from_tag(&items[0]).unwrap().1.id, "minecraft:stick");
    assert!(world_items.lock().unwrap().entities.is_empty());
    assert!(cache.dirty.lock().unwrap().contains(&ChunkPos { x: 0, z: 0 }));

    let mut out = Vec::new();
    subscription
        .drain_into(&mut out, crate::network::compression::CompressionState::disabled())
        .unwrap();
    let mut cursor = &out[..];
    let _len = crate::network::varint::read_var_i32(&mut cursor).unwrap();
    let id = crate::network::varint::read_var_i32(&mut cursor).unwrap();
    assert_eq!(id, crate::network::play::CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID);
    assert_eq!(crate::network::varint::read_var_i32(&mut cursor).unwrap(), 1);
    assert_eq!(crate::network::varint::read_var_i32(&mut cursor).unwrap(), eid);
}
