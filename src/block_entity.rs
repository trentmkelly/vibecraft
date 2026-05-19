#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::block_update::BlockPos;
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockEntityTypeId {
    Furnace,
    Chest,
    TrappedChest,
    EnderChest,
    Jukebox,
    Dispenser,
    Dropper,
    Sign,
    HangingSign,
    MobSpawner,
    CreakingHeart,
    Piston,
    BrewingStand,
    EnchantingTable,
    EndPortal,
    Beacon,
    Skull,
    DaylightDetector,
    Hopper,
    Comparator,
    Banner,
    StructureBlock,
    EndGateway,
    CommandBlock,
    ShulkerBox,
    Bed,
    Conduit,
    Barrel,
    Smoker,
    BlastFurnace,
    Lectern,
    Bell,
    Jigsaw,
    Campfire,
    Beehive,
    SculkSensor,
    CalibratedSculkSensor,
    SculkCatalyst,
    SculkShrieker,
    ChiseledBookshelf,
    Shelf,
    BrushableBlock,
    DecoratedPot,
    Crafter,
    TrialSpawner,
    Vault,
    TestBlock,
    TestInstanceBlock,
    CopperGolemStatue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityTickKind {
    None,
    Server,
    Client,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityTypeInfo {
    pub id: BlockEntityTypeId,
    pub key: &'static str,
    pub valid_blocks: &'static [&'static str],
    pub tick_kind: BlockEntityTickKind,
    pub op_only_custom_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntity {
    pub ty: BlockEntityTypeId,
    pub pos: BlockPos,
    pub block_state: String,
    pub custom_data: BTreeMap<String, Tag>,
    pub components: BTreeMap<String, Tag>,
    pub has_level: bool,
    pub removed: bool,
    pub changed: bool,
    pub tick_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBlockEntityDataPacket {
    pub pos: BlockPos,
    pub ty: BlockEntityTypeId,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEntityError {
    UnknownType(String),
    MissingId,
    InvalidBlockState {
        ty: BlockEntityTypeId,
        block_state: String,
    },
}

// Source: decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java
pub const BLOCK_ENTITY_TYPES: &[BlockEntityTypeInfo] = &[
    info(
        BlockEntityTypeId::Furnace,
        "furnace",
        &["minecraft:furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Chest,
        "chest",
        &[
            "minecraft:chest",
            "minecraft:copper_chest",
            "minecraft:exposed_copper_chest",
            "minecraft:weathered_copper_chest",
            "minecraft:oxidized_copper_chest",
            "minecraft:waxed_copper_chest",
            "minecraft:waxed_exposed_copper_chest",
            "minecraft:waxed_weathered_copper_chest",
            "minecraft:waxed_oxidized_copper_chest",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TrappedChest,
        "trapped_chest",
        &["minecraft:trapped_chest"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EnderChest,
        "ender_chest",
        &["minecraft:ender_chest"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Jukebox,
        "jukebox",
        &["minecraft:jukebox"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Dispenser,
        "dispenser",
        &["minecraft:dispenser"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Dropper,
        "dropper",
        &["minecraft:dropper"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Sign,
        "sign",
        &[
            "minecraft:oak_sign",
            "minecraft:spruce_sign",
            "minecraft:birch_sign",
            "minecraft:acacia_sign",
            "minecraft:cherry_sign",
            "minecraft:jungle_sign",
            "minecraft:dark_oak_sign",
            "minecraft:pale_oak_sign",
            "minecraft:mangrove_sign",
            "minecraft:crimson_sign",
            "minecraft:warped_sign",
            "minecraft:bamboo_sign",
            "minecraft:oak_wall_sign",
            "minecraft:spruce_wall_sign",
            "minecraft:birch_wall_sign",
            "minecraft:acacia_wall_sign",
            "minecraft:cherry_wall_sign",
            "minecraft:jungle_wall_sign",
            "minecraft:dark_oak_wall_sign",
            "minecraft:pale_oak_wall_sign",
            "minecraft:mangrove_wall_sign",
            "minecraft:crimson_wall_sign",
            "minecraft:warped_wall_sign",
            "minecraft:bamboo_wall_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::HangingSign,
        "hanging_sign",
        &[
            "minecraft:oak_hanging_sign",
            "minecraft:spruce_hanging_sign",
            "minecraft:birch_hanging_sign",
            "minecraft:acacia_hanging_sign",
            "minecraft:cherry_hanging_sign",
            "minecraft:jungle_hanging_sign",
            "minecraft:dark_oak_hanging_sign",
            "minecraft:pale_oak_hanging_sign",
            "minecraft:crimson_hanging_sign",
            "minecraft:warped_hanging_sign",
            "minecraft:mangrove_hanging_sign",
            "minecraft:bamboo_hanging_sign",
            "minecraft:oak_wall_hanging_sign",
            "minecraft:spruce_wall_hanging_sign",
            "minecraft:birch_wall_hanging_sign",
            "minecraft:acacia_wall_hanging_sign",
            "minecraft:cherry_wall_hanging_sign",
            "minecraft:jungle_wall_hanging_sign",
            "minecraft:dark_oak_wall_hanging_sign",
            "minecraft:pale_oak_wall_hanging_sign",
            "minecraft:crimson_wall_hanging_sign",
            "minecraft:warped_wall_hanging_sign",
            "minecraft:mangrove_wall_hanging_sign",
            "minecraft:bamboo_wall_hanging_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::MobSpawner,
        "mob_spawner",
        &["minecraft:spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::CreakingHeart,
        "creaking_heart",
        &["minecraft:creaking_heart"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Piston,
        "piston",
        &["minecraft:moving_piston"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BrewingStand,
        "brewing_stand",
        &["minecraft:brewing_stand"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::EnchantingTable,
        "enchanting_table",
        &["minecraft:enchanting_table"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::EndPortal,
        "end_portal",
        &["minecraft:end_portal"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Beacon,
        "beacon",
        &["minecraft:beacon"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Skull,
        "skull",
        &[
            "minecraft:skeleton_wall_skull",
            "minecraft:creeper_head",
            "minecraft:creeper_wall_head",
            "minecraft:dragon_wall_head",
            "minecraft:skeleton_skull",
            "minecraft:player_head",
            "minecraft:player_wall_head",
            "minecraft:dragon_head",
            "minecraft:zombie_head",
            "minecraft:zombie_wall_head",
            "minecraft:wither_skeleton_skull",
            "minecraft:wither_skeleton_wall_skull",
            "minecraft:piglin_head",
            "minecraft:piglin_wall_head",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::DaylightDetector,
        "daylight_detector",
        &["minecraft:daylight_detector"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Hopper,
        "hopper",
        &["minecraft:hopper"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Comparator,
        "comparator",
        &["minecraft:comparator"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Banner,
        "banner",
        &[
            "minecraft:white_banner",
            "minecraft:orange_banner",
            "minecraft:magenta_banner",
            "minecraft:light_blue_banner",
            "minecraft:yellow_banner",
            "minecraft:lime_banner",
            "minecraft:pink_banner",
            "minecraft:gray_banner",
            "minecraft:light_gray_banner",
            "minecraft:cyan_banner",
            "minecraft:purple_banner",
            "minecraft:blue_banner",
            "minecraft:brown_banner",
            "minecraft:green_banner",
            "minecraft:red_banner",
            "minecraft:black_banner",
            "minecraft:white_wall_banner",
            "minecraft:orange_wall_banner",
            "minecraft:magenta_wall_banner",
            "minecraft:light_blue_wall_banner",
            "minecraft:yellow_wall_banner",
            "minecraft:lime_wall_banner",
            "minecraft:pink_wall_banner",
            "minecraft:gray_wall_banner",
            "minecraft:light_gray_wall_banner",
            "minecraft:cyan_wall_banner",
            "minecraft:purple_wall_banner",
            "minecraft:blue_wall_banner",
            "minecraft:brown_wall_banner",
            "minecraft:green_wall_banner",
            "minecraft:red_wall_banner",
            "minecraft:black_wall_banner",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::StructureBlock,
        "structure_block",
        &["minecraft:structure_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EndGateway,
        "end_gateway",
        &["minecraft:end_gateway"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CommandBlock,
        "command_block",
        &[
            "minecraft:command_block",
            "minecraft:chain_command_block",
            "minecraft:repeating_command_block",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::ShulkerBox,
        "shulker_box",
        &[
            "minecraft:shulker_box",
            "minecraft:black_shulker_box",
            "minecraft:blue_shulker_box",
            "minecraft:brown_shulker_box",
            "minecraft:cyan_shulker_box",
            "minecraft:gray_shulker_box",
            "minecraft:green_shulker_box",
            "minecraft:light_blue_shulker_box",
            "minecraft:light_gray_shulker_box",
            "minecraft:lime_shulker_box",
            "minecraft:magenta_shulker_box",
            "minecraft:orange_shulker_box",
            "minecraft:pink_shulker_box",
            "minecraft:purple_shulker_box",
            "minecraft:red_shulker_box",
            "minecraft:white_shulker_box",
            "minecraft:yellow_shulker_box",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Bed,
        "bed",
        &[
            "minecraft:black_bed",
            "minecraft:blue_bed",
            "minecraft:brown_bed",
            "minecraft:cyan_bed",
            "minecraft:gray_bed",
            "minecraft:green_bed",
            "minecraft:light_blue_bed",
            "minecraft:light_gray_bed",
            "minecraft:lime_bed",
            "minecraft:magenta_bed",
            "minecraft:orange_bed",
            "minecraft:pink_bed",
            "minecraft:purple_bed",
            "minecraft:red_bed",
            "minecraft:white_bed",
            "minecraft:yellow_bed",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Conduit,
        "conduit",
        &["minecraft:conduit"],
        BlockEntityTickKind::Both,
        false,
    ),
    info(
        BlockEntityTypeId::Barrel,
        "barrel",
        &["minecraft:barrel"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Smoker,
        "smoker",
        &["minecraft:smoker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BlastFurnace,
        "blast_furnace",
        &["minecraft:blast_furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Lectern,
        "lectern",
        &["minecraft:lectern"],
        BlockEntityTickKind::None,
        true,
    ),
    info(
        BlockEntityTypeId::Bell,
        "bell",
        &["minecraft:bell"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Jigsaw,
        "jigsaw",
        &["minecraft:jigsaw"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Campfire,
        "campfire",
        &["minecraft:campfire", "minecraft:soul_campfire"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Beehive,
        "beehive",
        &["minecraft:bee_nest", "minecraft:beehive"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkSensor,
        "sculk_sensor",
        &["minecraft:sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CalibratedSculkSensor,
        "calibrated_sculk_sensor",
        &["minecraft:calibrated_sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkCatalyst,
        "sculk_catalyst",
        &["minecraft:sculk_catalyst"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkShrieker,
        "sculk_shrieker",
        &["minecraft:sculk_shrieker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::ChiseledBookshelf,
        "chiseled_bookshelf",
        &["minecraft:chiseled_bookshelf"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Shelf,
        "shelf",
        &[
            "minecraft:acacia_shelf",
            "minecraft:bamboo_shelf",
            "minecraft:birch_shelf",
            "minecraft:cherry_shelf",
            "minecraft:crimson_shelf",
            "minecraft:dark_oak_shelf",
            "minecraft:jungle_shelf",
            "minecraft:mangrove_shelf",
            "minecraft:oak_shelf",
            "minecraft:pale_oak_shelf",
            "minecraft:spruce_shelf",
            "minecraft:warped_shelf",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::BrushableBlock,
        "brushable_block",
        &["minecraft:suspicious_sand", "minecraft:suspicious_gravel"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::DecoratedPot,
        "decorated_pot",
        &["minecraft:decorated_pot"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Crafter,
        "crafter",
        &["minecraft:crafter"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TrialSpawner,
        "trial_spawner",
        &["minecraft:trial_spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::Vault,
        "vault",
        &["minecraft:vault"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TestBlock,
        "test_block",
        &["minecraft:test_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TestInstanceBlock,
        "test_instance_block",
        &["minecraft:test_instance_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::CopperGolemStatue,
        "copper_golem_statue",
        &[
            "minecraft:copper_golem_statue",
            "minecraft:waxed_copper_golem_statue",
        ],
        BlockEntityTickKind::Server,
        false,
    ),
];

const fn info(
    id: BlockEntityTypeId,
    key: &'static str,
    valid_blocks: &'static [&'static str],
    tick_kind: BlockEntityTickKind,
    op_only_custom_data: bool,
) -> BlockEntityTypeInfo {
    BlockEntityTypeInfo {
        id,
        key,
        valid_blocks,
        tick_kind,
        op_only_custom_data,
    }
}

pub fn type_info(ty: BlockEntityTypeId) -> &'static BlockEntityTypeInfo {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.id == ty)
        .expect("block entity type table covers every id")
}

pub fn type_by_key(key: &str) -> Option<BlockEntityTypeId> {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.key == key || format!("minecraft:{}", info.key) == key)
        .map(|info| info.id)
}

pub fn is_valid_block_state(ty: BlockEntityTypeId, block_state: &str) -> bool {
    type_info(ty).valid_blocks.contains(&block_state)
}

pub fn only_op_can_set_nbt(ty: BlockEntityTypeId) -> bool {
    type_info(ty).op_only_custom_data
}

pub fn has_block_entity_for_block(registry_id: &str) -> bool {
    BLOCK_ENTITY_TYPES
        .iter()
        .any(|entry| entry.valid_blocks.contains(&registry_id))
}

impl BlockEntity {
    pub fn new(
        ty: BlockEntityTypeId,
        pos: BlockPos,
        block_state: &str,
    ) -> Result<Self, BlockEntityError> {
        if !is_valid_block_state(ty, block_state) {
            return Err(BlockEntityError::InvalidBlockState {
                ty,
                block_state: block_state.to_string(),
            });
        }

        Ok(Self {
            ty,
            pos,
            block_state: block_state.to_string(),
            custom_data: BTreeMap::new(),
            components: BTreeMap::new(),
            has_level: false,
            removed: false,
            changed: false,
            tick_count: 0,
        })
    }

    pub fn set_level(&mut self) {
        self.has_level = true;
    }

    pub fn set_removed(&mut self) {
        self.removed = true;
    }

    pub fn clear_removed(&mut self) {
        self.removed = false;
    }

    pub fn set_changed(&mut self) {
        if self.has_level {
            self.changed = true;
        }
    }

    pub fn save_custom_only(&self) -> Tag {
        compound_from_map(&self.custom_data)
    }

    pub fn save_without_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn save_with_id(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        compound_from_map(&values)
    }

    pub fn save_with_full_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        values.insert("x".to_string(), Tag::Int(self.pos.x));
        values.insert("y".to_string(), Tag::Int(self.pos.y));
        values.insert("z".to_string(), Tag::Int(self.pos.z));
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn get_update_tag(&self) -> Tag {
        match self.ty {
            BlockEntityTypeId::Chest
            | BlockEntityTypeId::TrappedChest
            | BlockEntityTypeId::Barrel
            | BlockEntityTypeId::Hopper
            | BlockEntityTypeId::Dispenser
            | BlockEntityTypeId::Dropper => Tag::Compound(Vec::new()),
            _ => self.save_without_metadata(),
        }
    }

    pub fn get_update_packet(&self) -> ClientboundBlockEntityDataPacket {
        ClientboundBlockEntityDataPacket {
            pos: self.pos,
            ty: self.ty,
            tag: self.get_update_tag(),
        }
    }

    pub fn tick(&mut self, client_side: bool) -> bool {
        let tick_kind = type_info(self.ty).tick_kind;
        let should_tick = matches!(
            (tick_kind, client_side),
            (BlockEntityTickKind::Both, _)
                | (BlockEntityTickKind::Server, false)
                | (BlockEntityTickKind::Client, true)
        );

        if should_tick && !self.removed && self.has_level {
            self.tick_count += 1;
            true
        } else {
            false
        }
    }
}

pub fn load_static(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
) -> Result<BlockEntity, BlockEntityError> {
    let values = compound_entries(tag);
    let id = values
        .and_then(|entries| get_string(entries, "id"))
        .ok_or(BlockEntityError::MissingId)?;
    let ty = type_by_key(id).ok_or_else(|| BlockEntityError::UnknownType(id.to_string()))?;
    let mut entity = BlockEntity::new(ty, pos, block_state)?;

    if let Some(entries) = values {
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    entity.components = map_from_compound(value);
                }
                _ => {
                    entity.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    Ok(entity)
}

pub fn corrected_pos_from_chunk(base_chunk_x: i32, base_chunk_z: i32, tag: &Tag) -> BlockPos {
    let entries = compound_entries(tag);
    let x = entries
        .and_then(|entries| get_int(entries, "x"))
        .unwrap_or(0);
    let y = entries
        .and_then(|entries| get_int(entries, "y"))
        .unwrap_or(0);
    let z = entries
        .and_then(|entries| get_int(entries, "z"))
        .unwrap_or(0);
    let section_x = x.div_euclid(16);
    let section_z = z.div_euclid(16);

    if section_x == base_chunk_x && section_z == base_chunk_z {
        BlockPos { x, y, z }
    } else {
        BlockPos {
            x: base_chunk_x * 16 + x.rem_euclid(16),
            y,
            z: base_chunk_z * 16 + z.rem_euclid(16),
        }
    }
}

pub fn block_entity_packet_from_chunk(
    entity: &BlockEntity,
    chunk_min_y: i32,
) -> (u8, i16, BlockEntityTypeId, Tag) {
    let packed_xz = ((entity.pos.x & 15) << 4) | (entity.pos.z & 15);
    let section_y = (entity.pos.y - chunk_min_y) as i16;
    (
        packed_xz as u8,
        section_y,
        entity.ty,
        entity.get_update_tag(),
    )
}

fn compound_from_map(values: &BTreeMap<String, Tag>) -> Tag {
    Tag::Compound(
        values
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

fn map_from_compound(tag: &Tag) -> BTreeMap<String, Tag> {
    compound_entries(tag)
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

fn compound_entries(tag: &Tag) -> Option<&Vec<(String, Tag)>> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn get_string<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::String(value) if name == key => Some(value.as_str()),
        _ => None,
    })
}

fn get_int(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Int(value) if name == key => Some(*value),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> BlockPos {
        BlockPos {
            x: 18,
            y: 64,
            z: 35,
        }
    }

    #[test]
    fn block_entity_registry_matches_26_1_2_type_surface() {
        assert_eq!(BLOCK_ENTITY_TYPES.len(), 49);
        assert_eq!(type_info(BlockEntityTypeId::Furnace).key, "furnace");
        assert_eq!(
            type_info(BlockEntityTypeId::CopperGolemStatue).key,
            "copper_golem_statue"
        );
        assert!(is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:oak_wall_sign"
        ));
        assert!(!is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:stone"
        ));
    }

    #[test]
    fn op_only_custom_data_matches_vanilla_guarded_types() {
        assert!(only_op_can_set_nbt(BlockEntityTypeId::CommandBlock));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Lectern));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Sign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::HangingSign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::MobSpawner));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::TrialSpawner));
        assert!(!only_op_can_set_nbt(BlockEntityTypeId::Vault));
    }

    #[test]
    fn validates_block_state_on_creation_and_load() {
        assert!(BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").is_ok());
        assert_eq!(
            BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:furnace"),
            Err(BlockEntityError::InvalidBlockState {
                ty: BlockEntityTypeId::Chest,
                block_state: "minecraft:furnace".to_string()
            })
        );
    }

    #[test]
    fn detects_block_entity_support_for_block_states() {
        assert!(has_block_entity_for_block("minecraft:chest"));
        assert!(has_block_entity_for_block("minecraft:oak_sign"));
        assert!(has_block_entity_for_block("minecraft:oak_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:lectern"));
        assert!(has_block_entity_for_block("minecraft:command_block"));
        assert!(has_block_entity_for_block("minecraft:wither_skeleton_skull"));
        assert!(has_block_entity_for_block("minecraft:red_banner"));
        assert!(has_block_entity_for_block("minecraft:conduit"));
        assert!(has_block_entity_for_block("minecraft:bell"));
        assert!(has_block_entity_for_block("minecraft:crimson_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:brown_banner"));
        assert!(has_block_entity_for_block("minecraft:waxed_copper_chest"));
        assert!(has_block_entity_for_block("minecraft:dark_oak_wall_sign"));
        assert!(has_block_entity_for_block("minecraft:spawner"));
        assert!(has_block_entity_for_block("minecraft:vault"));
        assert!(has_block_entity_for_block("minecraft:trial_spawner"));
        assert!(has_block_entity_for_block("minecraft:calibrated_sculk_sensor"));
        assert!(has_block_entity_for_block("minecraft:chiseled_bookshelf"));
        assert!(has_block_entity_for_block("minecraft:suspicious_sand"));
        assert!(has_block_entity_for_block("minecraft:green_bed"));
        assert!(has_block_entity_for_block("minecraft:black_shulker_box"));
        assert!(has_block_entity_for_block("minecraft:crimson_shelf"));
        assert!(has_block_entity_for_block("minecraft:warped_wall_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:campfire"));
        assert!(!has_block_entity_for_block("minecraft:candle"));
        assert!(!has_block_entity_for_block("minecraft:cauldron"));
        assert!(!has_block_entity_for_block("minecraft:stone"));
        assert!(!has_block_entity_for_block("minecraft:dirt"));
    }

    #[test]
    fn save_modes_match_metadata_and_custom_data_boundaries() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        entity
            .custom_data
            .insert("front_text".to_string(), Tag::String("hello".to_string()));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Name\"".to_string()),
        );

        assert_eq!(
            entity.save_custom_only(),
            Tag::Compound(vec![(
                "front_text".to_string(),
                Tag::String("hello".to_string())
            )])
        );
        assert!(
            matches!(entity.save_without_metadata(), Tag::Compound(values) if values.iter().any(|(k, _)| k == "components") && values.iter().all(|(k, _)| k != "id"))
        );
        assert!(
            matches!(entity.save_with_id(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "id" && *v == Tag::String("sign".to_string())) && values.iter().all(|(k, _)| k != "x"))
        );
        assert!(
            matches!(entity.save_with_full_metadata(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "x" && *v == Tag::Int(18)))
        );
    }

    #[test]
    fn load_static_reads_id_components_and_custom_payload() {
        let tag = Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:campfire".to_string()),
            ),
            ("x".to_string(), Tag::Int(18)),
            ("y".to_string(), Tag::Int(64)),
            ("z".to_string(), Tag::Int(35)),
            ("CookingTimes".to_string(), Tag::List(vec![Tag::Int(10)])),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:lore".to_string(),
                    Tag::String("[]".to_string()),
                )]),
            ),
        ]);
        let entity = load_static(pos(), "minecraft:campfire", &tag).unwrap();
        assert_eq!(entity.ty, BlockEntityTypeId::Campfire);
        assert!(entity.custom_data.contains_key("CookingTimes"));
        assert!(entity.components.contains_key("minecraft:lore"));
    }

    #[test]
    fn wrong_chunk_positions_are_corrected_like_vanilla() {
        let tag = Tag::Compound(vec![
            ("x".to_string(), Tag::Int(34)),
            ("y".to_string(), Tag::Int(-20)),
            ("z".to_string(), Tag::Int(-17)),
        ]);
        assert_eq!(
            corrected_pos_from_chunk(0, 0, &tag),
            BlockPos {
                x: 2,
                y: -20,
                z: 15
            }
        );
    }

    #[test]
    fn ticking_requires_level_side_match_and_not_removed() {
        let mut furnace =
            BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
        assert!(!furnace.tick(false));
        furnace.set_level();
        assert!(furnace.tick(false));
        assert!(!furnace.tick(true));
        furnace.set_removed();
        assert!(!furnace.tick(false));

        let mut conduit =
            BlockEntity::new(BlockEntityTypeId::Conduit, pos(), "minecraft:conduit").unwrap();
        conduit.set_level();
        assert!(conduit.tick(false));
        assert!(conduit.tick(true));
    }

    #[test]
    fn changed_flag_only_sets_when_attached_to_level() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Bell, pos(), "minecraft:bell").unwrap();
        entity.set_changed();
        assert!(!entity.changed);
        entity.set_level();
        entity.set_changed();
        assert!(entity.changed);
    }

    #[test]
    fn update_packets_use_position_type_and_update_tag() {
        let mut sign =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        sign.custom_data
            .insert("front_text".to_string(), Tag::String("hi".to_string()));
        let packet = sign.get_update_packet();
        assert_eq!(packet.pos, pos());
        assert_eq!(packet.ty, BlockEntityTypeId::Sign);
        assert!(
            matches!(packet.tag, Tag::Compound(values) if values.iter().any(|(k, _)| k == "front_text"))
        );

        let chest = BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").unwrap();
        assert_eq!(chest.get_update_tag(), Tag::Compound(Vec::new()));
    }

    #[test]
    fn chunk_packet_data_packs_local_xz_y_type_and_tag() {
        let entity = BlockEntity::new(BlockEntityTypeId::Vault, pos(), "minecraft:vault").unwrap();
        let (packed_xz, y, ty, tag) = block_entity_packet_from_chunk(&entity, -64);
        assert_eq!(packed_xz, 0x23);
        assert_eq!(y, 128);
        assert_eq!(ty, BlockEntityTypeId::Vault);
        assert_eq!(
            tag,
            Tag::Compound(vec![("components".to_string(), Tag::Compound(Vec::new()))])
        );
    }
}
