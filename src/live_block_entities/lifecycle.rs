//! Block-entity creation and removal on block changes, mirroring the block-entity
//! half of Java `LevelChunk.setBlockState`.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::block_entity::{
    type_by_key, type_info, AbstractFurnaceBlockEntity, BlockEntityTypeId, FurnaceBlockEntityKind,
    BLOCK_ENTITY_TYPES,
};
use crate::block_update::BlockPos;
use crate::storage::chunk::LevelChunk;
use crate::storage::nbt::Tag;

/// The furnace-family kind a block entity type ticks as, if any.
pub fn furnace_kind(ty: BlockEntityTypeId) -> Option<FurnaceBlockEntityKind> {
    match ty {
        BlockEntityTypeId::Furnace => Some(FurnaceBlockEntityKind::Furnace),
        BlockEntityTypeId::BlastFurnace => Some(FurnaceBlockEntityKind::BlastFurnace),
        BlockEntityTypeId::Smoker => Some(FurnaceBlockEntityKind::Smoker),
        _ => None,
    }
}

/// `BlockState.hasBlockEntity` + `EntityBlock.newBlockEntity`: the block entity
/// type a block creates, from the `BlockEntityType.validBlocks` sets.
pub fn block_entity_type_for_block(block: &str) -> Option<BlockEntityTypeId> {
    static BY_BLOCK: OnceLock<HashMap<&'static str, BlockEntityTypeId>> = OnceLock::new();
    BY_BLOCK
        .get_or_init(|| {
            let mut map = HashMap::new();
            for info in BLOCK_ENTITY_TYPES {
                for block in info.valid_blocks {
                    map.entry(*block).or_insert(info.id);
                }
            }
            map
        })
        .get(block)
        .copied()
}

/// `BlockEntityType.isValid(state)` for a block name.
pub fn type_accepts_block(ty: BlockEntityTypeId, block: &str) -> bool {
    type_info(ty).valid_blocks.contains(&block)
}

/// The position and type stored in a block entity NBT compound
/// (`BlockEntity.loadStatic`'s `x`/`y`/`z`/`id`).
pub fn block_entity_identity(tag: &Tag) -> Option<(BlockPos, BlockEntityTypeId)> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    let int = |key: &str| {
        fields.iter().find_map(|(name, value)| match value {
            Tag::Int(value) if name == key => Some(*value),
            _ => None,
        })
    };
    let id = fields.iter().find_map(|(name, value)| match value {
        Tag::String(value) if name == "id" => Some(value.as_str()),
        _ => None,
    })?;
    Some((
        BlockPos {
            x: int("x")?,
            y: int("y")?,
            z: int("z")?,
        },
        type_by_key(id)?,
    ))
}

/// A freshly constructed block entity as saved by `BlockEntity.saveWithId`:
/// the id and position, plus the type's own default fields (Java furnaces always
/// write their counters).
pub fn new_block_entity_tag(ty: BlockEntityTypeId, pos: BlockPos) -> Tag {
    let key = type_info(ty).key;
    let id = if key.contains(':') {
        key.to_string()
    } else {
        format!("minecraft:{key}")
    };
    let mut fields = vec![
        ("id".to_string(), Tag::String(id)),
        ("x".to_string(), Tag::Int(pos.x)),
        ("y".to_string(), Tag::Int(pos.y)),
        ("z".to_string(), Tag::Int(pos.z)),
    ];
    if let Some(kind) = furnace_kind(ty) {
        if let Tag::Compound(own) = AbstractFurnaceBlockEntity::new(kind).save_additional() {
            fields.extend(own);
        }
    }
    Tag::Compound(fields)
}

/// The block-entity part of `LevelChunk.setBlockState` after the section write:
/// a changed block drops the old block entity, a block entity that no longer
/// accepts the new state is removed, and a block that has a block entity gets a
/// new one when none exists.
///
/// `old_block` is the previous block name (`None` for air / unknown) and
/// `new_state` the written state string (`name` or `name[props]`).
pub fn sync_block_entity_after_set_block(
    chunk: &mut LevelChunk,
    pos: BlockPos,
    old_block: Option<&str>,
    new_state: &str,
) {
    let new_block = new_state.split('[').next().unwrap_or(new_state);
    let block_changed = old_block != Some(new_block);
    let at_pos = |tag: &Tag| block_entity_identity(tag).map(|(p, _)| p) == Some(pos);

    if block_changed && old_block.and_then(block_entity_type_for_block).is_some() {
        chunk.block_entities.retain(|tag| !at_pos(tag));
    }

    let Some(new_type) = block_entity_type_for_block(new_block) else {
        return;
    };
    let existing = chunk
        .block_entities
        .iter()
        .find_map(|tag| block_entity_identity(tag).filter(|(p, _)| *p == pos));
    match existing {
        Some((_, ty)) if type_accepts_block(ty, new_block) => {}
        Some(_) => {
            chunk.block_entities.retain(|tag| !at_pos(tag));
            chunk
                .block_entities
                .push(new_block_entity_tag(new_type, pos));
        }
        None => chunk
            .block_entities
            .push(new_block_entity_tag(new_type, pos)),
    }
}
