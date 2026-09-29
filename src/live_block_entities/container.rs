//! `Container` / `WorldlyContainer` views over the NBT block entities in the
//! chunk cache, for code that moves items between containers (the hopper).
//!
//! Java resolves a container with `HopperBlockEntity.getContainerAt`: a
//! `WorldlyContainerHolder` block, else the block entity when it is a
//! `Container` (a chest merged with its neighbour through
//! `ChestBlock.getContainer(..., ignoreBeingBlocked = true)`), else a container
//! entity. [`container_at`] does the same against the cached chunks and loads
//! the result into an in-memory [`LoadedContainer`]; mutations are written back
//! with [`LoadedContainer::store`].
//!
//! TODO(hopper-container-sources): `WorldlyContainerHolder` (composter), the
//! remaining `Container` block entities (brewing stand, crafter, jukebox,
//! chiseled bookshelf) and container entities (chest/hopper minecarts) are not
//! resolved yet; a hopper treats them as "no container".
//! TODO(hopper-loot-unpack): a container whose loot table is still unresolved is
//! skipped instead of being unpacked with `unpackLootTable(null)`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::block_entity::{BlockEntityTypeId, FurnaceBlockEntityKind};
use crate::block_update::{BlockPos, Direction};
use crate::storage::chunk::{BlockStateEntry, LevelChunk};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

use super::lifecycle::{block_entity_identity, furnace_kind, type_accepts_block};
use super::stack::Stack;
use super::{furnace, BlockEntityTickEnvironment};

/// Mutable access to the loaded chunks while a world-level ticker runs.
pub struct ChunkWorld<'a> {
    chunks: &'a mut HashMap<ChunkPos, Arc<LevelChunk>>,
    dirtied: Vec<ChunkPos>,
}

impl<'a> ChunkWorld<'a> {
    pub fn new(chunks: &'a mut HashMap<ChunkPos, Arc<LevelChunk>>) -> Self {
        Self {
            chunks,
            dirtied: Vec::new(),
        }
    }

    fn chunk(&self, pos: BlockPos) -> Option<&Arc<LevelChunk>> {
        self.chunks.get(&ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        })
    }

    /// `Level.getBlockState` (`None` for unloaded chunks).
    pub fn block_state(&self, pos: BlockPos) -> Option<BlockStateEntry> {
        self.chunk(pos)?.get_block_state_model(pos.x, pos.y, pos.z)
    }

    /// `Level.getBlockEntity`, as its NBT.
    pub fn block_entity(&self, pos: BlockPos) -> Option<Tag> {
        self.chunk(pos)?
            .block_entities
            .iter()
            .find(|tag| block_entity_identity(tag).map(|(at, _)| at) == Some(pos))
            .cloned()
    }

    /// Replaces a block entity's NBT and marks its chunk for saving.
    pub fn put_block_entity(&mut self, tag: Tag) {
        let Some((pos, _)) = block_entity_identity(&tag) else {
            return;
        };
        let chunk_pos = ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        };
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            Arc::make_mut(chunk).set_block_entity_nbt(tag);
            self.dirtied.push(chunk_pos);
        }
    }

    /// Positions of every loaded block entity of `ty`, ordered by chunk (then
    /// by their order in the chunk) so ticking order is deterministic.
    pub fn block_entities_of(&self, ty: BlockEntityTypeId) -> Vec<BlockPos> {
        let mut keys: Vec<&ChunkPos> = self.chunks.keys().collect();
        keys.sort_by_key(|key| (key.x, key.z));
        keys.into_iter()
            .flat_map(|key| self.chunks[key].block_entities.iter())
            .filter_map(block_entity_identity)
            .filter(|(_, found)| *found == ty)
            .map(|(pos, _)| pos)
            .collect()
    }

    /// The chunks written since [`Self::new`].
    pub fn into_dirtied(self) -> Vec<ChunkPos> {
        self.dirtied
    }
}

/// Which `Container` interface a block entity type implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rules {
    /// A plain `Container` (chest, barrel, dispenser, dropper).
    Plain,
    /// `HopperBlockEntity`: plain container that also carries the transfer
    /// cooldown.
    Hopper,
    /// `ShulkerBoxBlockEntity implements WorldlyContainer`.
    Shulker,
    /// `AbstractFurnaceBlockEntity implements WorldlyContainer`.
    Furnace(FurnaceBlockEntityKind),
}

/// One block entity's worth of slots.
#[derive(Debug, Clone)]
struct Part {
    tag: Tag,
    rules: Rules,
    slots: Vec<Option<Stack>>,
    /// Hopper `cooldownTime`.
    cooldown: i32,
    /// The cooldown as loaded, to tell whether it needs writing back.
    loaded_cooldown: i32,
    changed: bool,
}

/// A loaded container: one block entity, or a double chest's two halves in
/// `CompoundContainer(first, second)` order.
#[derive(Debug, Clone)]
pub struct LoadedContainer {
    parts: Vec<Part>,
}

impl Part {
    fn position(&self) -> BlockPos {
        block_entity_identity(&self.tag).map_or(BlockPos { x: 0, y: 0, z: 0 }, |(pos, _)| pos)
    }
}

/// `HopperBlockEntity.getContainerAt(level, pos)` for the block entity types
/// this ticker can resolve.
pub fn container_at(world: &ChunkWorld<'_>, pos: BlockPos) -> Option<LoadedContainer> {
    let state = world.block_state(pos)?;
    let tag = world.block_entity(pos)?;
    let (_, ty) = block_entity_identity(&tag)?;
    if !type_accepts_block(ty, &state.name) {
        return None;
    }
    match ty {
        BlockEntityTypeId::Chest | BlockEntityTypeId::TrappedChest => {
            chest_container(world, pos, &state, tag)
        }
        _ => single_part(ty, tag).map(|part| LoadedContainer { parts: vec![part] }),
    }
}

/// `ChestBlock.getContainer(block, state, level, pos, true)`.
fn chest_container(
    world: &ChunkWorld<'_>,
    pos: BlockPos,
    state: &BlockStateEntry,
    tag: Tag,
) -> Option<LoadedContainer> {
    let this = single_part(BlockEntityTypeId::Chest, tag)?;
    let chest_type = state.properties.get("type").map(String::as_str);
    let Some(connected) = connected_direction(state).filter(|_| chest_type != Some("single"))
    else {
        return Some(LoadedContainer { parts: vec![this] });
    };
    let neighbour_pos = pos.relative(connected);
    let neighbour_state = world.block_state(neighbour_pos);
    let neighbour = neighbour_state.as_ref().filter(|neighbour| {
        let neighbour_type = neighbour.properties.get("type").map(String::as_str);
        neighbour.name == state.name
            && neighbour_type != Some("single")
            && neighbour_type != chest_type
            && neighbour.properties.get("facing") == state.properties.get("facing")
    });
    let neighbour_part = neighbour
        .and_then(|_| world.block_entity(neighbour_pos))
        .and_then(|tag| single_part(BlockEntityTypeId::Chest, tag));
    let Some(neighbour_part) = neighbour_part else {
        return Some(LoadedContainer { parts: vec![this] });
    };
    // `ChestBlock.getBlockType`: RIGHT is FIRST, LEFT is SECOND.
    let parts = if chest_type == Some("right") {
        vec![this, neighbour_part]
    } else {
        vec![neighbour_part, this]
    };
    Some(LoadedContainer { parts })
}

/// `ChestBlock.getConnectedDirection`.
fn connected_direction(state: &BlockStateEntry) -> Option<Direction> {
    let facing = match state.properties.get("facing")?.as_str() {
        "north" => Direction::North,
        "east" => Direction::East,
        "south" => Direction::South,
        "west" => Direction::West,
        _ => return None,
    };
    let clockwise = state.properties.get("type").map(String::as_str) == Some("left");
    Some(if clockwise {
        rotate_y_clockwise(facing)
    } else {
        rotate_y_counter_clockwise(facing)
    })
}

fn rotate_y_clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        _ => Direction::North,
    }
}

fn rotate_y_counter_clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        _ => Direction::North,
    }
}

fn container_size(ty: BlockEntityTypeId) -> Option<usize> {
    match ty {
        BlockEntityTypeId::Chest
        | BlockEntityTypeId::TrappedChest
        | BlockEntityTypeId::Barrel
        | BlockEntityTypeId::ShulkerBox => Some(27),
        BlockEntityTypeId::Dispenser | BlockEntityTypeId::Dropper => Some(9),
        BlockEntityTypeId::Hopper => Some(5),
        _ => furnace_kind(ty).map(|_| 3),
    }
}

fn single_part(ty: BlockEntityTypeId, tag: Tag) -> Option<Part> {
    let size = container_size(ty)?;
    let Tag::Compound(entries) = &tag else {
        return None;
    };
    // Unresolved loot: see TODO(hopper-loot-unpack).
    if entries.iter().any(|(key, _)| key == "LootTable") {
        return None;
    }
    let mut slots = vec![None; size];
    if let Some((_, Tag::List(items))) = entries.iter().find(|(key, _)| key == "Items") {
        for (slot, stack) in items.iter().filter_map(Stack::from_tag) {
            if slot < size {
                slots[slot] = Some(stack);
            }
        }
    }
    let rules = match ty {
        BlockEntityTypeId::Hopper => Rules::Hopper,
        BlockEntityTypeId::ShulkerBox => Rules::Shulker,
        other => furnace_kind(other).map_or(Rules::Plain, Rules::Furnace),
    };
    let cooldown = entries
        .iter()
        .find_map(|(key, value)| match value {
            Tag::Int(value) if key == "TransferCooldown" => Some(*value),
            _ => None,
        })
        .unwrap_or(-1);
    Some(Part {
        tag,
        rules,
        slots,
        cooldown,
        loaded_cooldown: cooldown,
        changed: false,
    })
}

impl LoadedContainer {
    /// `Container.getContainerSize`.
    pub fn size(&self) -> usize {
        self.parts.iter().map(|part| part.slots.len()).sum()
    }

    fn locate(&self, slot: usize) -> Option<(usize, usize)> {
        let mut base = 0;
        for (index, part) in self.parts.iter().enumerate() {
            if slot < base + part.slots.len() {
                return Some((index, slot - base));
            }
            base += part.slots.len();
        }
        None
    }

    /// `Container.getItem`.
    pub fn get(&self, slot: usize) -> Option<&Stack> {
        let (part, local) = self.locate(slot)?;
        self.parts[part].slots[local].as_ref()
    }

    /// `Container.isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.parts
            .iter()
            .all(|part| part.slots.iter().all(Option::is_none))
    }

    /// `Container.removeItem(slot, count)`: splits off up to `count` items.
    pub fn remove_item(&mut self, slot: usize, count: i32) -> Option<Stack> {
        let (part, local) = self.locate(slot)?;
        let part = &mut self.parts[part];
        let stack = part.slots[local].as_mut().filter(|_| count > 0)?;
        let split = stack.split(count);
        if stack.count <= 0 {
            part.slots[local] = None;
        }
        part.changed = true;
        Some(split)
    }

    /// `Container.setItem`, including `limitSize(getMaxStackSize(stack))`.
    pub fn set_item(&mut self, slot: usize, stack: Option<Stack>) {
        let Some((part, local)) = self.locate(slot) else {
            return;
        };
        let part = &mut self.parts[part];
        part.slots[local] = stack
            .filter(|stack| stack.count > 0)
            .map(|mut stack| {
                stack.count = stack.count.min(64).min(stack.max_stack_size());
                stack
            });
        part.changed = true;
    }

    /// `Container.setChanged`.
    pub fn set_changed(&mut self) {
        for part in &mut self.parts {
            part.changed = true;
        }
    }

    fn worldly_rules(&self) -> Option<Rules> {
        match (self.parts.as_slice(), self.parts.first()) {
            ([_], Some(part)) if matches!(part.rules, Rules::Shulker | Rules::Furnace(_)) => {
                Some(part.rules)
            }
            _ => None,
        }
    }

    /// Whether this is a `WorldlyContainer` (single shulker box or furnace).
    pub fn is_worldly(&self) -> bool {
        self.worldly_rules().is_some()
    }

    /// `WorldlyContainer.getSlotsForFace`.
    pub fn slots_for_face(&self, direction: Direction) -> Vec<usize> {
        match self.worldly_rules() {
            Some(Rules::Shulker) => (0..27).collect(),
            Some(Rules::Furnace(_)) => match direction {
                Direction::Down => vec![2, 1],
                Direction::Up => vec![0],
                _ => vec![1],
            },
            _ => (0..self.size()).collect(),
        }
    }

    /// `Container.canPlaceItem`.
    pub fn can_place_item(
        &self,
        slot: usize,
        stack: &Stack,
        env: &BlockEntityTickEnvironment<'_>,
    ) -> bool {
        match self.parts.first().map(|part| part.rules) {
            Some(Rules::Furnace(_)) => match slot {
                2 => false,
                1 => {
                    env.fuel_values.is_fuel(&stack.id)
                        || stack.id == "minecraft:bucket"
                            && self.get(1).is_none_or(|fuel| fuel.id != "minecraft:bucket")
                }
                _ => true,
            },
            _ => true,
        }
    }

    /// `WorldlyContainer.canPlaceItemThroughFace` (true for non-worldly).
    pub fn can_place_item_through_face(
        &self,
        slot: usize,
        stack: &Stack,
        env: &BlockEntityTickEnvironment<'_>,
    ) -> bool {
        match self.worldly_rules() {
            Some(Rules::Shulker) => !is_shulker_box_item(&stack.id),
            Some(Rules::Furnace(_)) => self.can_place_item(slot, stack, env),
            _ => true,
        }
    }

    /// `WorldlyContainer.canTakeItemThroughFace` (true for non-worldly).
    pub fn can_take_item_through_face(
        &self,
        slot: usize,
        stack: &Stack,
        direction: Direction,
    ) -> bool {
        match self.worldly_rules() {
            Some(Rules::Furnace(_)) if direction == Direction::Down && slot == 1 => {
                stack.id == "minecraft:water_bucket" || stack.id == "minecraft:bucket"
            }
            _ => true,
        }
    }

    /// Whether this container is a single `HopperBlockEntity`.
    pub fn is_hopper(&self) -> bool {
        matches!(self.parts.as_slice(), [part] if part.rules == Rules::Hopper)
    }

    /// The position of a single-block-entity container.
    pub fn hopper_pos(&self) -> Option<BlockPos> {
        self.is_hopper().then(|| self.parts[0].position())
    }

    /// `HopperBlockEntity.cooldownTime` (hopper containers only).
    pub fn hopper_cooldown(&self) -> i32 {
        self.parts.first().map_or(-1, |part| part.cooldown)
    }

    /// `HopperBlockEntity.setCooldown`. The block entity is only written back
    /// when the final value differs from the loaded one, so an idle hopper
    /// (cooldown settled at 0) is never rewritten.
    pub fn set_hopper_cooldown(&mut self, time: i32) {
        if let Some(part) = self.parts.first_mut() {
            part.cooldown = time;
        }
    }

    /// The modified flag of every part, for [`Self::restore_slot`].
    pub fn changed_flags(&self) -> Vec<bool> {
        self.parts.iter().map(|part| part.changed).collect()
    }

    /// Puts a slot back exactly as it was before a failed transfer (Java resets
    /// the split stack's count and `setItem`s it back), leaving the modified
    /// flags as they were so a rejected transfer never dirties the chunk.
    pub fn restore_slot(&mut self, slot: usize, original: Option<Stack>, flags: &[bool]) {
        if let Some((part, local)) = self.locate(slot) {
            self.parts[part].slots[local] = original;
        }
        for (part, flag) in self.parts.iter_mut().zip(flags) {
            part.changed = *flag;
        }
    }

    /// Writes every modified block entity back to the world.
    pub fn store(self, world: &mut ChunkWorld<'_>, env: &BlockEntityTickEnvironment<'_>) {
        for part in self
            .parts
            .into_iter()
            .filter(|part| part.changed || part.cooldown != part.loaded_cooldown)
        {
            let tag = part.to_tag(env);
            world.put_block_entity(tag);
        }
    }
}

impl Part {
    /// The block entity NBT with this part's slots (and hopper cooldown).
    fn to_tag(&self, env: &BlockEntityTickEnvironment<'_>) -> Tag {
        let items: Vec<Tag> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, stack)| stack.as_ref().map(|stack| stack.to_tag(slot)))
            .collect();
        let Tag::Compound(entries) = &self.tag else {
            return self.tag.clone();
        };
        let mut fields: Vec<(String, Tag)> = entries
            .iter()
            .filter(|(key, _)| !matches!(key.as_str(), "Items" | "TransferCooldown"))
            .cloned()
            .collect();
        // `ContainerHelper.saveAllItems(..., alsoWhenEmpty)`: only shulker boxes
        // omit an empty list.
        if !(items.is_empty() && self.rules == Rules::Shulker) {
            fields.push(("Items".to_string(), Tag::List(items)));
        }
        if self.rules == Rules::Hopper {
            fields.push(("TransferCooldown".to_string(), Tag::Int(self.cooldown)));
        }
        let updated = Tag::Compound(fields);
        match self.rules {
            Rules::Furnace(kind) => furnace::apply_container_slots(kind, &self.tag, &updated, &self.slots, env.recipes),
            _ => updated,
        }
    }
}

/// `Block.byItem(item) instanceof ShulkerBoxBlock`.
fn is_shulker_box_item(id: &str) -> bool {
    id == "minecraft:shulker_box" || id.ends_with("_shulker_box")
}
