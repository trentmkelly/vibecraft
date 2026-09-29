//! `HopperBlockEntity.pushItemsTick` over the NBT-backed hoppers and containers
//! of the chunk cache.
//!
//! Every server tick each hopper counts its transfer cooldown down and, once it
//! is off cooldown and its block is `enabled`, first pushes one item into the
//! container it faces (`ejectItems`) and then pulls one item from the container
//! above, or the item entities above it, (`suckInItems`). Any successful move
//! restarts the [`MOVE_ITEM_SPEED`]-tick cooldown.
//!
//! TODO(hopper-entity-inside): `HopperBlockEntity.entityInside` (an item entity
//! touching the hopper triggers an immediate suck attempt) needs the entity
//! movement/collision callback; only the per-tick `getItemsAtAndAbove` scan is
//! live.
//! TODO(hopper-comparator-updates): `setChanged` also updates comparators next
//! to the container; there is no live comparator/redstone system yet.
//! TODO(hopper-powered-state): `HopperBlock.checkPoweredState` (redstone
//! toggling `ENABLED`) needs live neighbour signals; the ticker only reads the
//! stored `enabled` property.

use std::collections::BTreeMap;

use crate::block_entity::BlockEntityTypeId;
use crate::block_update::{BlockPos, Direction};
use crate::item_entity::WorldItemEntities;

use super::container::{container_at, ChunkWorld, LoadedContainer};
use super::lifecycle::type_accepts_block;
use super::stack::Stack;
use super::{format_block_state, BlockEntityTickEnvironment};

/// `HopperBlockEntity.MOVE_ITEM_SPEED`.
pub const MOVE_ITEM_SPEED: i32 = 8;

/// Item entities are 0.25 x 0.25 (`EntityType.ITEM`).
const ITEM_ENTITY_SIZE: f64 = 0.25;

/// What a hopper pass did to item entities, for the caller to broadcast.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HopperEntityEffects {
    /// Entity ids discarded after being fully absorbed.
    pub removed: Vec<i32>,
    /// `(entity id, item, new count)` for partially absorbed stacks.
    pub count_updates: Vec<(i32, &'static str, i32)>,
}

/// Per-hopper state that Java keeps in memory only (`tickedGameTime`).
#[derive(Debug, Default)]
pub struct HopperTicker {
    ticked_game_time: BTreeMap<BlockPos, i64>,
}

/// Inputs shared by every hopper of one pass.
struct PassContext<'a, 'e> {
    env: &'a BlockEntityTickEnvironment<'e>,
    game_time: i64,
    ticked: &'a mut BTreeMap<BlockPos, i64>,
    items: &'a mut WorldItemEntities,
    effects: &'a mut HopperEntityEffects,
}

impl HopperTicker {
    /// Ticks every loaded hopper once (`Level.tickBlockEntities`).
    pub fn tick(
        &mut self,
        world: &mut ChunkWorld<'_>,
        items: &mut WorldItemEntities,
        env: &BlockEntityTickEnvironment<'_>,
        game_time: i64,
    ) -> HopperEntityEffects {
        let positions = world.block_entities_of(BlockEntityTypeId::Hopper);
        self.ticked_game_time
            .retain(|pos, _| positions.contains(pos));
        let mut effects = HopperEntityEffects::default();
        let mut context = PassContext {
            env,
            game_time,
            ticked: &mut self.ticked_game_time,
            items,
            effects: &mut effects,
        };
        for pos in positions {
            push_items_tick(world, &mut context, pos);
        }
        effects
    }
}

/// `HopperBlockEntity.pushItemsTick`.
fn push_items_tick(world: &mut ChunkWorld<'_>, context: &mut PassContext<'_, '_>, pos: BlockPos) {
    let Some(state) = world.block_state(pos) else {
        return;
    };
    if !type_accepts_block(BlockEntityTypeId::Hopper, &state.name) {
        return;
    }
    let Some(mut hopper) = container_at(world, pos) else {
        return;
    };
    let facing = facing_of(state.properties.get("facing"));
    let enabled = state.properties.get("enabled").map(String::as_str) != Some("false");

    hopper.set_hopper_cooldown(hopper.hopper_cooldown() - 1);
    context.ticked.insert(pos, context.game_time);
    if !is_on_cooldown(&hopper) {
        hopper.set_hopper_cooldown(0);
        try_move_items(world, context, pos, facing, enabled, &mut hopper, |world, context, hopper| {
            suck_in_items(world, context, pos, hopper)
        });
    }
    hopper.store(world, context.env);
}

fn facing_of(value: Option<&String>) -> Direction {
    match value.map(String::as_str) {
        Some("north") => Direction::North,
        Some("south") => Direction::South,
        Some("west") => Direction::West,
        Some("east") => Direction::East,
        Some("up") => Direction::Up,
        _ => Direction::Down,
    }
}

fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::West => Direction::East,
        Direction::East => Direction::West,
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::North => Direction::South,
        Direction::South => Direction::North,
    }
}

fn is_on_cooldown(hopper: &LoadedContainer) -> bool {
    hopper.hopper_cooldown() > 0
}

fn is_on_custom_cooldown(hopper: &LoadedContainer) -> bool {
    hopper.hopper_cooldown() > MOVE_ITEM_SPEED
}

/// `HopperBlockEntity.tryMoveItems`.
fn try_move_items(
    world: &mut ChunkWorld<'_>,
    context: &mut PassContext<'_, '_>,
    pos: BlockPos,
    facing: Direction,
    enabled: bool,
    hopper: &mut LoadedContainer,
    action: impl FnOnce(&mut ChunkWorld<'_>, &mut PassContext<'_, '_>, &mut LoadedContainer) -> bool,
) -> bool {
    if is_on_cooldown(hopper) || !enabled {
        return false;
    }
    let mut changed = false;
    if !hopper.is_empty() {
        changed = eject_items(world, context, pos, facing, hopper);
    }
    if !inventory_full(hopper) {
        changed |= action(world, context, hopper);
    }
    if changed {
        hopper.set_hopper_cooldown(MOVE_ITEM_SPEED);
        hopper.set_changed();
    }
    changed
}

/// `HopperBlockEntity.inventoryFull`.
fn inventory_full(hopper: &LoadedContainer) -> bool {
    (0..hopper.size()).all(|slot| {
        hopper
            .get(slot)
            .is_some_and(|stack| stack.count == stack.max_stack_size())
    })
}

/// `HopperBlockEntity.ejectItems`.
fn eject_items(
    world: &mut ChunkWorld<'_>,
    context: &mut PassContext<'_, '_>,
    pos: BlockPos,
    facing: Direction,
    hopper: &mut LoadedContainer,
) -> bool {
    let Some(mut container) = container_at(world, pos.relative(facing)) else {
        return false;
    };
    let direction = opposite(facing);
    if is_full_container(&container, direction) {
        return false;
    }
    for slot in 0..hopper.size() {
        let Some(original) = hopper.get(slot).cloned() else {
            continue;
        };
        let flags = hopper.changed_flags();
        let one = hopper.remove_item(slot, 1);
        let result = one.and_then(|one| {
            add_item(context, Some(&*hopper), &mut container, one, Some(direction))
        });
        if result.is_none() {
            container.set_changed();
            container.store(world, context.env);
            return true;
        }
        hopper.restore_slot(slot, Some(original), &flags);
    }
    false
}

/// `HopperBlockEntity.isFullContainer`.
fn is_full_container(container: &LoadedContainer, direction: Direction) -> bool {
    container
        .slots_for_face(direction)
        .into_iter()
        .all(|slot| {
            container
                .get(slot)
                .is_some_and(|stack| stack.count >= stack.max_stack_size())
        })
}

/// `HopperBlockEntity.suckInItems`.
fn suck_in_items(
    world: &mut ChunkWorld<'_>,
    context: &mut PassContext<'_, '_>,
    pos: BlockPos,
    hopper: &mut LoadedContainer,
) -> bool {
    let above = pos.relative(Direction::Up);
    if let Some(mut container) = container_at(world, above) {
        for slot in container.slots_for_face(Direction::Down) {
            if try_take_in_item_from_slot(context, hopper, &mut container, slot) {
                container.store(world, context.env);
                return true;
            }
        }
        return false;
    }
    // `isGridAligned() && isCollisionShapeFullBlock && !DOES_NOT_BLOCK_HOPPERS`.
    if blocks_hopper(world, above) {
        return false;
    }
    let candidates = items_at_and_above(context.items, pos);
    for entity_id in candidates {
        if add_item_entity(context, hopper, entity_id) {
            return true;
        }
    }
    false
}

fn blocks_hopper(world: &ChunkWorld<'_>, above: BlockPos) -> bool {
    let Some(state) = world.block_state(above) else {
        return false;
    };
    let full = crate::block_properties::state_physics_by_name(&format_block_state(&state))
        .is_some_and(crate::block_properties::collision_shape_is_full_cube);
    full && !crate::block_tags::block_tag_contains("does_not_block_hoppers", &state.name)
}

/// `HopperBlockEntity.tryTakeInItemFromSlot` (the hopper is the destination).
fn try_take_in_item_from_slot(
    context: &mut PassContext<'_, '_>,
    hopper: &mut LoadedContainer,
    container: &mut LoadedContainer,
    slot: usize,
) -> bool {
    let Some(original) = container.get(slot).cloned() else {
        return false;
    };
    // `canTakeItemFromContainer(hopper, container, stack, slot, DOWN)`.
    if !container.can_take_item_through_face(slot, &original, Direction::Down) {
        return false;
    }
    let flags = container.changed_flags();
    let one = container.remove_item(slot, 1);
    let result = one.and_then(|one| add_item(context, Some(&*container), hopper, one, None));
    if result.is_none() {
        container.set_changed();
        return true;
    }
    container.restore_slot(slot, Some(original), &flags);
    false
}

/// The ids of the live item entities inside the hopper's suck box
/// (`getItemsAtAndAbove` with `ENTITY_STILL_ALIVE`).
fn items_at_and_above(items: &WorldItemEntities, pos: BlockPos) -> Vec<i32> {
    // `SUCK_AABB` = column(16, 11, 32) moved to the hopper's block.
    let (min_x, min_y, min_z) = (f64::from(pos.x), f64::from(pos.y) + 11.0 / 16.0, f64::from(pos.z));
    let (max_x, max_y, max_z) = (min_x + 1.0, f64::from(pos.y) + 2.0, min_z + 1.0);
    let half = ITEM_ENTITY_SIZE / 2.0;
    items
        .entities
        .iter()
        .filter(|entity| entity.count > 0)
        .filter(|entity| {
            entity.x - half < max_x
                && entity.x + half > min_x
                && entity.y < max_y
                && entity.y + ITEM_ENTITY_SIZE > min_y
                && entity.z - half < max_z
                && entity.z + half > min_z
        })
        .map(|entity| entity.entity_id)
        .collect()
}

/// `HopperBlockEntity.addItem(Container, ItemEntity)`: returns whether the
/// whole stack was absorbed (the entity is then discarded); a partly absorbed
/// stack stays behind with its reduced count.
fn add_item_entity(
    context: &mut PassContext<'_, '_>,
    hopper: &mut LoadedContainer,
    entity_id: i32,
) -> bool {
    let Some(entity) = context
        .items
        .entities
        .iter()
        .find(|entity| entity.entity_id == entity_id)
    else {
        return false;
    };
    let (item, original_count) = (entity.item, entity.count);
    let remainder = add_item(context, None, hopper, Stack::new(item, original_count), None);
    match remainder {
        None => {
            context.effects.removed.push(entity_id);
            context
                .items
                .entities
                .retain(|entity| entity.entity_id != entity_id);
            true
        }
        Some(rest) => {
            if rest.count != original_count {
                if let Some(entity) = context
                    .items
                    .entities
                    .iter_mut()
                    .find(|entity| entity.entity_id == entity_id)
                {
                    entity.count = rest.count;
                }
                context
                    .effects
                    .count_updates
                    .push((entity_id, item, rest.count));
            }
            false
        }
    }
}

/// `HopperBlockEntity.addItem(from, container, stack, direction)`: returns what
/// did not fit (`None` when everything was inserted).
fn add_item(
    context: &mut PassContext<'_, '_>,
    from: Option<&LoadedContainer>,
    container: &mut LoadedContainer,
    stack: Stack,
    direction: Option<Direction>,
) -> Option<Stack> {
    let slots: Vec<usize> = match direction {
        Some(direction) if container.is_worldly() => container.slots_for_face(direction),
        _ => (0..container.size()).collect(),
    };
    let mut remaining = Some(stack);
    for slot in slots {
        let Some(current) = remaining.take() else {
            break;
        };
        remaining = try_move_in_item(context, from, container, current, slot);
    }
    remaining
}

/// `HopperBlockEntity.tryMoveInItem` (the face check lives in
/// [`can_place_item_in_container`]; non-worldly containers ignore the face).
fn try_move_in_item(
    context: &mut PassContext<'_, '_>,
    from: Option<&LoadedContainer>,
    container: &mut LoadedContainer,
    mut stack: Stack,
    slot: usize,
) -> Option<Stack> {
    if !can_place_item_in_container(context, container, &stack, slot) {
        return Some(stack);
    }
    let was_empty = container.is_empty();
    let leftover;
    let success;
    match container.get(slot).cloned() {
        None => {
            container.set_item(slot, Some(stack));
            leftover = None;
            success = true;
        }
        Some(mut current) if can_merge_items(&current, &stack) => {
            let space = stack.max_stack_size() - current.count;
            let count = stack.count.min(space);
            stack.count -= count;
            current.count += count;
            success = count > 0;
            if success {
                container.set_item(slot, Some(current));
            }
            leftover = (stack.count > 0).then_some(stack);
        }
        Some(_) => {
            leftover = Some(stack);
            success = false;
        }
    }
    if success {
        if was_empty && container.is_hopper() && !is_on_custom_cooldown(container) {
            set_fresh_hopper_cooldown(context, from, container);
        }
        container.set_changed();
    }
    leftover
}

/// The `wasEmpty && container instanceof HopperBlockEntity` branch of
/// `tryMoveInItem`: a hopper that just received its first item waits a full
/// cooldown, one tick less when it has already ticked at or after the source
/// hopper this game tick.
fn set_fresh_hopper_cooldown(
    context: &PassContext<'_, '_>,
    from: Option<&LoadedContainer>,
    target: &mut LoadedContainer,
) {
    let ticked = |container: &LoadedContainer| {
        container
            .hopper_pos()
            .and_then(|pos| context.ticked.get(&pos).copied())
            .unwrap_or(0)
    };
    let skip_tick_count = match from {
        Some(from) if from.is_hopper() && ticked(target) >= ticked(from) => 1,
        _ => 0,
    };
    target.set_hopper_cooldown(MOVE_ITEM_SPEED - skip_tick_count);
}

/// `HopperBlockEntity.canPlaceItemInContainer`.
fn can_place_item_in_container(
    context: &PassContext<'_, '_>,
    container: &LoadedContainer,
    stack: &Stack,
    slot: usize,
) -> bool {
    container.can_place_item(slot, stack, context.env)
        && container.can_place_item_through_face(slot, stack, context.env)
}

/// `HopperBlockEntity.canMergeItems`.
fn can_merge_items(a: &Stack, b: &Stack) -> bool {
    a.count <= a.max_stack_size() && a.same_item_same_components(b)
}
