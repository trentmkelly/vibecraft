//! Predicates and world queries behind the `execute if|unless` conditions: block/biome/item
//! predicates, heightmaps (`positioned over`), `checkRegions` and loot predicates.

use super::execute_args::*;
use super::*;
use crate::block_states::{block_state_entry, default_state_properties};
use crate::command_slot_arguments::{SlotRangeModel, SlotRangesModel};
use crate::item_catalog::primary_item_static_name;
use crate::storage::nbt::nbt_utils::compare_nbt;
use std::collections::BTreeMap;

/// Maximum `execute if blocks` volume (`ExecuteCommand.MAX_TEST_AREA`).
pub(super) const MAX_TEST_AREA: i64 = 32768;

/// Splits `namespace:id` input and applies the default `minecraft` namespace.
fn normalized_id(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("minecraft:{input}")
    }
}

/// The block identity of a `BlockPredicateArgument` / `ResourceOrTagArgument` / item argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum IdOrTag {
    Id(String),
    Tag(String),
}

impl IdOrTag {
    fn parse(input: &str) -> Self {
        match input.strip_prefix('#') {
            Some(tag) => Self::Tag(normalized_id(tag)),
            None => Self::Id(normalized_id(input)),
        }
    }
}

/// `BlockPredicateArgument.blockPredicate(...)`: `id|#tag[properties]{nbt}`.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct BlockPredicate {
    target: IdOrTag,
    properties: Vec<(String, String)>,
    nbt: Option<Tag>,
}

impl BlockPredicate {
    pub(super) fn parse(input: &str) -> Result<Self, CommandError> {
        let head_end = input.find(['[', '{']).unwrap_or(input.len());
        let (head, mut rest) = input.split_at(head_end);
        if head.is_empty() {
            return Err(CommandError::InvalidSyntax);
        }
        let target = IdOrTag::parse(head);
        let mut properties = Vec::new();
        if let Some(after_bracket) = rest.strip_prefix('[') {
            let close = after_bracket.find(']').ok_or(CommandError::InvalidSyntax)?;
            for entry in after_bracket[..close].split(',').filter(|e| !e.is_empty()) {
                let (name, value) = entry.split_once('=').ok_or(CommandError::InvalidSyntax)?;
                properties.push((name.to_string(), value.to_string()));
            }
            rest = &after_bracket[close + 1..];
        }
        let nbt = if rest.is_empty() {
            None
        } else if rest.starts_with('{') {
            Some(parse_snbt(rest).map_err(|_| CommandError::InvalidSyntax)?)
        } else {
            return Err(CommandError::InvalidSyntax);
        };
        match &target {
            IdOrTag::Id(id) => {
                let entry = block_state_entry(id).ok_or(CommandError::InvalidSyntax)?;
                for (name, value) in &properties {
                    let known = entry
                        .properties
                        .iter()
                        .find(|property| property.name == name)
                        .ok_or(CommandError::InvalidSyntax)?;
                    if !known.values.contains(&value.as_str()) {
                        return Err(CommandError::InvalidSyntax);
                    }
                }
            }
            IdOrTag::Tag(tag) => {
                if crate::block_tags::block_tag_members(tag.strip_prefix("minecraft:").unwrap_or(tag))
                    .is_none()
                    && crate::block_tags::block_tag_members(tag).is_none()
                {
                    return Err(CommandError::InvalidSyntax);
                }
            }
        }
        Ok(Self {
            target,
            properties,
            nbt,
        })
    }

    /// `BlockPredicateArgument.Result.test(BlockInWorld)` for the block stored at `pos`.
    pub(super) fn test(&self, state: &ServerCommandState, dimension: &str, pos: BlockPos) -> bool {
        let block = block_at(state, dimension, pos);
        let (id, actual) = block_state_values(&block);
        let matches_target = match &self.target {
            IdOrTag::Id(expected) => *expected == id,
            IdOrTag::Tag(tag) => {
                crate::block_tags::block_tag_contains(tag, &id)
                    || tag
                        .strip_prefix("minecraft:")
                        .is_some_and(|short| crate::block_tags::block_tag_contains(short, &id))
            }
        };
        if !matches_target {
            return false;
        }
        if !self
            .properties
            .iter()
            .all(|(name, value)| actual.get(name.as_str()) == Some(value))
        {
            return false;
        }
        match &self.nbt {
            None => true,
            Some(expected) => state
                .macro_block_nbt_sources
                .iter()
                .find(|source| source.pos == pos)
                .is_some_and(|source| compare_nbt(Some(expected), Some(&source.nbt), true)),
        }
    }
}

/// A block-state string (`id` or `id[k=v,...]`) as its namespaced id and full property values
/// (explicit values over the block's default state).
pub(super) fn block_state_values(block: &str) -> (String, BTreeMap<String, String>) {
    let (id, explicit) = match block.split_once('[') {
        Some((id, rest)) => (id, rest.trim_end_matches(']')),
        None => (block, ""),
    };
    let id = normalized_id(id);
    let mut values: BTreeMap<String, String> = default_state_properties(&id)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    for entry in explicit.split(',').filter(|entry| !entry.is_empty()) {
        if let Some((name, value)) = entry.split_once('=') {
            values.insert(name.to_string(), value.to_string());
        }
    }
    (id, values)
}

/// `ResourceOrTagArgument.resourceOrTag(context, Registries.BIOME)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BiomePredicate(IdOrTag);

impl BiomePredicate {
    pub(super) fn parse(input: &str) -> Result<Self, CommandError> {
        let target = IdOrTag::parse(input);
        if let IdOrTag::Tag(tag) = &target {
            if !crate::biome_tags::is_known_biome_tag(tag)
                && !crate::biome_tags::is_known_biome_tag(
                    tag.strip_prefix("minecraft:").unwrap_or(tag),
                )
            {
                return Err(CommandError::InvalidSyntax);
            }
        }
        Ok(Self(target))
    }

    pub(super) fn test(&self, biome: &str) -> bool {
        match &self.0 {
            IdOrTag::Id(id) => id == biome,
            IdOrTag::Tag(tag) => {
                crate::biome_tags::biome_in_tag(biome, tag)
                    || tag
                        .strip_prefix("minecraft:")
                        .is_some_and(|short| crate::biome_tags::biome_in_tag(biome, short))
            }
        }
    }
}

/// `ItemPredicateArgument.itemPredicate(context)`: `*`, `id` or `#tag`.
///
/// TODO(execute-item-components): the `id[component=...]` / `~component` / `|` grammar needs
/// item stacks with data components; command-model stacks are only `(item, count)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ItemPredicate {
    Any,
    Item(IdOrTag),
}

impl ItemPredicate {
    pub(super) fn parse(input: &str) -> Result<Self, CommandError> {
        if input == "*" {
            return Ok(Self::Any);
        }
        if input.contains(['[', '~', '|']) {
            return Err(CommandError::InvalidSyntax);
        }
        let target = IdOrTag::parse(input);
        if let IdOrTag::Id(id) = &target {
            if primary_item_static_name(id).is_none() {
                return Err(CommandError::InvalidSyntax);
            }
        }
        Ok(Self::Item(target))
    }

    fn test(&self, item: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Item(IdOrTag::Id(id)) => id == item,
            Self::Item(IdOrTag::Tag(tag)) => crate::item_tags::item_in_tag(item, tag),
        }
    }
}

/// `SlotsArgument.slots()`.
pub(super) fn parse_slot_range(input: &str) -> Result<SlotRangeModel, CommandError> {
    SlotRangesModel::name_to_ids(input).ok_or(CommandError::InvalidSyntax)
}

/// Whether a stored slot name addresses slot id `slot_id`.
fn slot_name_has_id(name: &str, slot_id: i32) -> bool {
    SlotRangesModel::name_to_ids(name)
        .is_some_and(|range| range.size() == 1 && range.slots()[0] == slot_id)
}

/// `ExecuteCommand.countItems(Iterable<SlotProvider>, SlotRange, Predicate)`.
pub(super) fn count_entity_items(
    state: &ServerCommandState,
    entities: &[EntityRef],
    range: &SlotRangeModel,
    predicate: &ItemPredicate,
) -> i32 {
    let mut count = 0;
    for entity in entities {
        for slot_id in range.slots() {
            let stack = state
                .entity_item_slots
                .iter()
                .find(|entry| entry.entity.id == entity.id && slot_name_has_id(&entry.slot, *slot_id))
                .and_then(|entry| entry.item.as_ref());
            if let Some(stack) = stack {
                if predicate.test(&stack.item) {
                    count += stack.count;
                }
            }
        }
    }
    count
}

/// `ExecuteCommand.countItems(CommandSourceStack, BlockPos, SlotRange, Predicate)`.
pub(super) fn count_block_items(
    state: &ServerCommandState,
    pos: BlockPos,
    range: &SlotRangeModel,
    predicate: &ItemPredicate,
) -> Result<i32, CommandError> {
    // `ItemCommands.getContainer(..., ERROR_SOURCE_NOT_A_CONTAINER)`.
    if !state.block_item_slots.iter().any(|entry| entry.pos == pos) {
        return Err(translatable("commands.item.source.not_a_container", [
            pos.x.to_string(),
            pos.y.to_string(),
            pos.z.to_string(),
        ]));
    }
    let mut count = 0;
    for slot_id in range.slots() {
        let stack = state
            .block_item_slots
            .iter()
            .find(|entry| entry.pos == pos && slot_name_has_id(&entry.slot, *slot_id))
            .and_then(|entry| entry.item.as_ref());
        if let Some(stack) = stack {
            if predicate.test(&stack.item) {
                count += stack.count;
            }
        }
    }
    Ok(count)
}

/// The `Heightmap.Types` selectable by `HeightmapTypeArgument`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CommandHeightmap {
    WorldSurface,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl CommandHeightmap {
    pub(super) fn parse(input: &str) -> Result<Self, CommandError> {
        Ok(match input {
            "world_surface" => Self::WorldSurface,
            "ocean_floor" => Self::OceanFloor,
            "motion_blocking" => Self::MotionBlocking,
            "motion_blocking_no_leaves" => Self::MotionBlockingNoLeaves,
            _ => return Err(CommandError::InvalidSyntax),
        })
    }

    /// `Heightmap.Types.isOpaque()` for a stored block state.
    fn is_opaque(self, block: &str) -> bool {
        let (id, _) = block_state_values(block);
        let physics = crate::block_properties::state_physics_by_name(block).or_else(|| {
            crate::block_states::default_state_network_id(&id)
                .and_then(crate::block_properties::state_physics)
        });
        let Some(physics) = physics else {
            return false;
        };
        match self {
            Self::WorldSurface => !physics.is_air,
            Self::OceanFloor => physics.blocks_motion,
            Self::MotionBlocking => physics.blocks_motion || physics.liquid,
            Self::MotionBlockingNoLeaves => {
                (physics.blocks_motion || physics.liquid) && !id.ends_with("_leaves")
            }
        }
    }
}

/// `(min_y, max_y)` build limits of a vanilla dimension (`DimensionType.minY/height`).
fn dimension_height_limits(dimension: &str) -> (i32, i32) {
    if dimension == "minecraft:overworld" {
        (-64, 320)
    } else {
        (0, 256)
    }
}

/// `ServerLevel.getHeight(Heightmap.Types, x, z)`: one above the highest matching block, or the
/// dimension's minimum build height when the column has none.
pub(super) fn level_height(
    state: &ServerCommandState,
    dimension: &str,
    heightmap: CommandHeightmap,
    x: i32,
    z: i32,
) -> i32 {
    let (min_y, max_y) = dimension_height_limits(dimension);
    let highest = state
        .blocks
        .iter()
        .filter(|entry| {
            entry.dimension == dimension
                && entry.position.x == x
                && entry.position.z == z
                && (min_y..max_y).contains(&entry.position.y)
                && heightmap.is_opaque(&entry.block)
        })
        .map(|entry| entry.position.y)
        .max();
    highest.map_or(min_y, |y| y + 1)
}

/// `ExecuteCommand.checkRegions(level, start, end, destination, skipAir)`: the number of
/// compared blocks, or `None` when the regions differ.
pub(super) fn check_regions(
    state: &ServerCommandState,
    dimension: &str,
    start: BlockPos,
    end: BlockPos,
    destination: BlockPos,
    skip_air: bool,
) -> Result<Option<i32>, CommandError> {
    let (min, max) = (
        BlockPos {
            x: start.x.min(end.x),
            y: start.y.min(end.y),
            z: start.z.min(end.z),
        },
        BlockPos {
            x: start.x.max(end.x),
            y: start.y.max(end.y),
            z: start.z.max(end.z),
        },
    );
    // `BoundingBox.fromCorners(dest, dest.offset(from.getLength()))` only ever shifts by the
    // destination's own minimum, so the block offset is `destination - from.min`.
    let offset = BlockPos {
        x: destination.x - min.x,
        y: destination.y - min.y,
        z: destination.z - min.z,
    };
    let area = i64::from(max.x - min.x + 1) * i64::from(max.y - min.y + 1)
        * i64::from(max.z - min.z + 1);
    if area > MAX_TEST_AREA {
        return Err(translatable(
            "commands.execute.blocks.toobig",
            [MAX_TEST_AREA.to_string(), area.to_string()],
        ));
    }
    let mut count = 0;
    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let source_pos = BlockPos { x, y, z };
                let dest_pos = BlockPos {
                    x: x + offset.x,
                    y: y + offset.y,
                    z: z + offset.z,
                };
                let source_block = block_at(state, dimension, source_pos);
                if skip_air && source_block == "minecraft:air" {
                    continue;
                }
                if source_block != block_at(state, dimension, dest_pos) {
                    return Ok(None);
                }
                let source_entity = state
                    .macro_block_nbt_sources
                    .iter()
                    .find(|source| source.pos == source_pos);
                if let Some(source_entity) = source_entity {
                    let dest_entity = state
                        .macro_block_nbt_sources
                        .iter()
                        .find(|source| source.pos == dest_pos);
                    if dest_entity.is_none_or(|dest| dest.nbt != source_entity.nbt) {
                        return Ok(None);
                    }
                }
                count += 1;
            }
        }
    }
    Ok(Some(count))
}

/// `ExecuteCommand.checkCustomPredicate`: evaluates a loot predicate with `origin` = the source
/// position and `this` = the source entity.
///
/// TODO(loot-conditions): only the structural conditions (`inverted`, `all_of`, `any_of`) and
/// the ones the command model can answer (`weather_check`, `entity_scores`) are evaluated; the
/// remaining `LootItemCondition` types need the loot-context runtime and are treated as
/// failing.
pub(super) fn evaluate_loot_predicate(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    definition: &Tag,
) -> bool {
    let Tag::Compound(fields) = definition else {
        // A predicate file may also be a list of conditions (`all_of` shorthand).
        return match definition {
            Tag::List(terms) => terms
                .iter()
                .all(|term| evaluate_loot_predicate(state, source, term)),
            _ => false,
        };
    };
    let field = |name: &str| fields.iter().find(|(key, _)| key == name).map(|(_, tag)| tag);
    let Some(Tag::String(condition)) = field("condition") else {
        return false;
    };
    match condition.strip_prefix("minecraft:").unwrap_or(condition) {
        "inverted" => field("term").is_some_and(|term| !evaluate_loot_predicate(state, source, term)),
        "all_of" | "any_of" => {
            let terms: &[Tag] = match field("terms") {
                Some(Tag::List(terms)) => terms,
                _ => &[],
            };
            let mut results = terms
                .iter()
                .map(|term| evaluate_loot_predicate(state, source, term));
            if condition.ends_with("all_of") {
                results.all(|result| result)
            } else {
                results.any(|result| result)
            }
        }
        "weather_check" => {
            let raining = matches!(state.weather.mode, WeatherMode::Rain | WeatherMode::Thunder);
            let thundering = state.weather.mode == WeatherMode::Thunder;
            let wanted = |name: &str, actual: bool| match field(name) {
                Some(Tag::Byte(value)) => (*value != 0) == actual,
                _ => true,
            };
            wanted("raining", raining) && wanted("thundering", thundering)
        }
        "entity_scores" => loot_entity_scores(state, source, field("scores")),
        _ => false,
    }
}

/// `EntityHasScoreCondition` for `entity: this`.
fn loot_entity_scores(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    scores: Option<&Tag>,
) -> bool {
    let (Some(entity), Some(Tag::Compound(scores))) = (&source.entity, scores) else {
        return false;
    };
    scores.iter().all(|(objective, bounds)| {
        let Some(score) = scoreboard_score(state, &entity.id, objective) else {
            return false;
        };
        match bounds {
            Tag::Int(value) => score.value == *value,
            Tag::Compound(range) => {
                let bound = |name: &str| {
                    range.iter().find_map(|(key, tag)| match (key.as_str(), tag) {
                        (key, Tag::Int(value)) if key == name => Some(*value),
                        _ => None,
                    })
                };
                bound("min").is_none_or(|min| score.value >= min)
                    && bound("max").is_none_or(|max| score.value <= max)
            }
            _ => false,
        }
    })
}
