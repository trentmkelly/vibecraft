use super::*;

pub fn frog_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

pub fn pig_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

pub fn wolf_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "ashen" => Some(0),
        "black" => Some(1),
        "chestnut" => Some(2),
        "pale" => Some(3),
        "rusty" => Some(4),
        "snowy" => Some(5),
        "spotted" => Some(6),
        "striped" => Some(7),
        "woods" => Some(8),
        _ => None,
    }
}

pub fn wolf_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "angry" => Some(0),
        "big" => Some(1),
        "classic" => Some(2),
        "cute" => Some(3),
        "grumpy" => Some(4),
        "puglin" => Some(5),
        "sad" => Some(6),
        _ => None,
    }
}

pub fn pig_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "big" => Some(0),
        "classic" => Some(1),
        "mini" => Some(2),
        _ => None,
    }
}

pub fn zombie_nautilus_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "temperate" => Some(0),
        "warm" => Some(1),
        _ => None,
    }
}

pub fn tag_double_triplet_field(fields: &[(String, Tag)], name: &str) -> Option<[f64; 3]> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name != name {
            return None;
        }
        let Tag::List(values) = value else {
            return None;
        };
        let [Tag::Double(x), Tag::Double(y), Tag::Double(z)] = values.as_slice() else {
            return None;
        };
        Some([*x, *y, *z])
    })
}

pub fn tag_float_pair_field(fields: &[(String, Tag)], name: &str) -> Option<[f32; 2]> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name != name {
            return None;
        }
        let Tag::List(values) = value else {
            return None;
        };
        let [Tag::Float(first), Tag::Float(second)] = values.as_slice() else {
            return None;
        };
        Some([*first, *second])
    })
}

pub fn generated_chunk_entity_runtime_id(chunk_pos: ChunkPos, index: usize) -> i32 {
    let x = chunk_pos.x.rem_euclid(1024);
    let z = chunk_pos.z.rem_euclid(1024);
    1_000_000 + x * 1_048_576 + z * 256 + (index as i32 & 0xff)
}

pub fn generated_mob_entity_type_network_id(entity_type: &str) -> Option<i32> {
    match entity_type {
        "minecraft:armadillo" => Some(4),
        "minecraft:axolotl" => Some(7),
        "minecraft:bat" => Some(10),
        "minecraft:bogged" => Some(16),
        "minecraft:camel" => Some(19),
        "minecraft:chicken" => Some(26),
        "minecraft:cod" => Some(27),
        "minecraft:cow" => Some(30),
        "minecraft:creeper" => Some(32),
        "minecraft:dolphin" => Some(35),
        "minecraft:donkey" => Some(36),
        "minecraft:drowned" => Some(38),
        "minecraft:enderman" => Some(41),
        "minecraft:fox" => Some(54),
        "minecraft:frog" => Some(55),
        "minecraft:ghast" => Some(57),
        "minecraft:glow_squid" => Some(61),
        "minecraft:goat" => Some(62),
        "minecraft:hoglin" => Some(64),
        "minecraft:horse" => Some(66),
        "minecraft:husk" => Some(67),
        "minecraft:llama" => Some(78),
        "minecraft:magma_cube" => Some(80),
        "minecraft:mooshroom" => Some(86),
        "minecraft:mule" => Some(87),
        "minecraft:ocelot" => Some(91),
        "minecraft:panda" => Some(96),
        "minecraft:parched" => Some(97),
        "minecraft:parrot" => Some(98),
        "minecraft:pig" => Some(100),
        "minecraft:piglin" => Some(101),
        "minecraft:polar_bear" => Some(104),
        "minecraft:pufferfish" => Some(107),
        "minecraft:rabbit" => Some(108),
        "minecraft:salmon" => Some(110),
        "minecraft:sheep" => Some(111),
        "minecraft:skeleton" => Some(115),
        "minecraft:slime" => Some(117),
        "minecraft:spider" => Some(124),
        "minecraft:squid" => Some(127),
        "minecraft:stray" => Some(128),
        "minecraft:strider" => Some(129),
        "minecraft:trader_llama" => Some(134),
        "minecraft:tropical_fish" => Some(136),
        "minecraft:turtle" => Some(137),
        "minecraft:witch" => Some(144),
        "minecraft:wolf" => Some(148),
        "minecraft:zombie" => Some(150),
        "minecraft:zombie_horse" => Some(151),
        "minecraft:zombie_nautilus" => Some(152),
        "minecraft:zombie_villager" => Some(153),
        "minecraft:zombified_piglin" => Some(154),
        _ => None,
    }
}

pub fn try_load_chunk_from_region(
    region_dir: &Path,
    pos: ChunkPos,
) -> Option<crate::storage::chunk::LevelChunk> {
    let region = RegionFile::open(region_dir, pos.region()).ok()?;
    let (_name, tag) = region.read_chunk_nbt(pos).ok()??;
    crate::storage::chunk::LevelChunk::from_nbt(pos, &tag)
        .ok()
        .filter(chunk_has_non_air_blocks)
}

pub fn chunk_has_non_air_blocks(chunk: &LevelChunk) -> bool {
    chunk.sections.iter().any(|section| {
        PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
            .ok()
            .is_some_and(|container| container.palette.iter().any(palette_entry_is_non_air))
    })
}

pub fn palette_entry_is_non_air(entry: &Tag) -> bool {
    match entry {
        Tag::String(name) => name != "minecraft:air" && name != "air",
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(name, value)| {
                (name == "Name").then_some(value).and_then(|value| {
                    if let Tag::String(block_name) = value {
                        Some(block_name != "minecraft:air" && block_name != "air")
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(false),
        Tag::Int(id) => *id != 0,
        _ => false,
    }
}

pub fn write_level_chunk_with_light_payload<W: Write>(
    writer: &mut W,
    packet: &ClientboundLevelChunkWithLightPacket,
) -> io::Result<()> {
    writer.write_all(&packet.pos.x.to_be_bytes())?;
    writer.write_all(&packet.pos.z.to_be_bytes())?;
    let chunk_data = packet.chunk_data.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "level chunk with light packet requires chunk data",
        )
    })?;
    write_level_chunk_packet_data(writer, chunk_data)?;
    let light_data = packet.light_data.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "level chunk with light packet requires light data",
        )
    })?;
    light_data.write(writer)
}

pub fn write_level_chunk_packet_data<W: Write>(
    writer: &mut W,
    data: &ClientboundLevelChunkPacketData,
) -> io::Result<()> {
    data.write(writer)
}

#[allow(dead_code)]
pub fn write_superflat_spawn_chunk_packet<W: Write>(writer: &mut W, x: i32, z: i32) -> io::Result<()> {
    writer.write_all(&x.to_be_bytes())?;
    writer.write_all(&z.to_be_bytes())?;
    write_var_i32(writer, 0)?;

    let mut section_buffer = Vec::with_capacity(SPAWN_CHUNK_SECTION_COUNT * 10);
    for section_index in 0..SPAWN_CHUNK_SECTION_COUNT {
        let non_empty_block_count = visible_spawn_terrain_block_count(x, z, section_index);
        section_buffer.write_all(&non_empty_block_count.to_be_bytes())?;
        section_buffer.write_all(&0_i16.to_be_bytes())?;
        if non_empty_block_count > 0 {
            write_visible_spawn_terrain_block_state_container(
                &mut section_buffer,
                x,
                z,
                section_index,
            )?;
        } else {
            write_single_value_paletted_container(&mut section_buffer, AIR_BLOCK_STATE_ID)?;
        }
        write_single_value_paletted_container(&mut section_buffer, PLAINS_BIOME_ID)?;
    }
    write_var_i32(writer, section_buffer.len() as i32)?;
    writer.write_all(&section_buffer)?;
    write_var_i32(writer, 0)?;

    write_bitset(writer, &[(1_u64 << SPAWN_CHUNK_SECTION_COUNT) - 1])?;
    write_empty_bitset(writer)?;
    write_empty_bitset(writer)?;
    write_bitset(writer, &[(1_u64 << SPAWN_CHUNK_SECTION_COUNT) - 1])?;
    write_var_i32(writer, SPAWN_CHUNK_SECTION_COUNT as i32)?;
    for _ in 0..SPAWN_CHUNK_SECTION_COUNT {
        write_var_i32(writer, 2048)?;
        writer.write_all(&[0xff; 2048])?;
    }
    write_var_i32(writer, 0)
}

#[allow(dead_code)]
pub fn write_single_value_paletted_container<W: Write>(writer: &mut W, id: i32) -> io::Result<()> {
    writer.write_all(&[0])?;
    write_var_i32(writer, id)
}

pub fn visible_spawn_terrain_height(chunk_x: i32, chunk_z: i32, local_x: usize, local_z: usize) -> i32 {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let broad = (world_x.div_euclid(12) + world_z.div_euclid(14)).rem_euclid(8);
    let terrace = (world_x.div_euclid(5) - world_z.div_euclid(7)).rem_euclid(6);
    let wrinkle = ((world_x.wrapping_mul(31) ^ world_z.wrapping_mul(17)) & 3) as i32;
    let ridge = if (world_x.wrapping_mul(11) + world_z.wrapping_mul(13)).rem_euclid(29) <= 2 {
        14
    } else {
        0
    };
    let plateau = if (world_x.div_euclid(24) - world_z.div_euclid(19)).rem_euclid(5) == 0 {
        14
    } else {
        0
    };
    let valley = if (world_x.wrapping_mul(5) - world_z.wrapping_mul(7)).rem_euclid(37) <= 3 {
        7
    } else {
        0
    };
    (TERRAIN_MIN_SURFACE_Y + broad + terrace + wrinkle + ridge + plateau - valley).clamp(68, 104)
}

pub fn visible_spawn_surface_feature_id(
    chunk_x: i32,
    chunk_z: i32,
    local_x: usize,
    local_z: usize,
) -> Option<i32> {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let hash = world_x.wrapping_mul(734_287) ^ world_z.wrapping_mul(912_931);
    match hash.rem_euclid(23) {
        0 => Some(DANDELION_BLOCK_STATE_ID),
        7 | 17 => Some(POPPY_BLOCK_STATE_ID),
        5 | 13 | 19 => Some(SHORT_GRASS_BLOCK_STATE_ID),
        _ => None,
    }
}

pub fn visible_spawn_surface_top_block_id(
    chunk_x: i32,
    chunk_z: i32,
    local_x: usize,
    local_z: usize,
) -> i32 {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let hash = world_x.wrapping_mul(193_496_63) ^ world_z.wrapping_mul(83_492_791);
    match hash.rem_euclid(43) {
        0 => STONE_BLOCK_STATE_ID,
        9 => GRANITE_BLOCK_STATE_ID,
        18 => DIORITE_BLOCK_STATE_ID,
        27 => ANDESITE_BLOCK_STATE_ID,
        34 | 41 => DIRT_BLOCK_STATE_ID,
        _ => GRASS_BLOCK_STATE_ID,
    }
}

/// Builds the block loot table for `block_name`, matching the JSON loot tables
/// from data/minecraft/loot_table/blocks/ in the Java source.
pub fn block_loot_table(block_name: &str) -> Option<LootTable> {
    let key = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let random_sequence = format!("minecraft:blocks/{key}");

    // A pool that drops one stack of `item` unconditionally.
    pub fn self_drop_table(item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::item(
                format!("minecraft:{item}"),
                1,
            ))],
            functions: Vec::new(),
        }
    }

    // A pool that drops one stack of `item` only if the block survives explosion.
    // For normal block breaking (no explosion), this always drops.
    pub fn self_drop_survives_explosion(item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool {
                entries: vec![LootEntry::item(format!("minecraft:{item}"), 1)],
                conditions: vec![LootCondition::SurvivesExplosion],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            }],
            functions: Vec::new(),
        }
    }

    // Ore that always drops a single item (coal, iron, gold, diamond, emerald, quartz).
    // Silk touch (ore block self-drop) not yet implemented; always uses the raw-product path.
    pub fn ore_drop_1(drop_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: format!("minecraft:{drop_item}"),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::ApplyExplosionDecay],
            })],
            functions: Vec::new(),
        }
    }

    // Ore that drops a uniform-count range (copper 2-5, redstone 4-5, lapis 4-9, etc.).
    // Fortune bonuses not yet implemented; `min`/`max` are base counts.
    pub fn ore_drop_count(drop_item: &str, min: f32, max: f32, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: format!("minecraft:{drop_item}"),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min, max }),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        }
    }

    // Grass-type plants: shears → self, else 12.5% chance of wheat_seeds.
    pub fn grass_table(self_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: format!("minecraft:{self_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.125)],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        }
    }

    // Standard leaf block: shears → leaves, else sapling at `sapling_chance` (fortune-0 base),
    // plus a 2% stick pool when not using shears. Oak additionally has a 0.5% apple pool.
    // Silk touch and fortune bonuses not yet implemented.
    pub fn leaves_table(
        leaves_item: &str,
        sapling_item: &str,
        sapling_chance: f32,
        has_apple_pool: bool,
        sequence: String,
    ) -> LootTable {
        let not_shears = LootCondition::Inverted(Box::new(LootCondition::MatchTool {
            item: "minecraft:shears".to_string(),
        }));
        let mut pools = vec![
            // Pool 0: shears → leaves block, else sapling with survival + chance
            LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: format!("minecraft:{leaves_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: format!("minecraft:{sapling_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(sapling_chance),
                    ],
                    functions: Vec::new(),
                },
            ])),
            // Pool 1: 2% chance of 1-2 sticks when not using shears
            LootPool {
                entries: vec![LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                }],
                conditions: vec![not_shears.clone()],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            },
        ];
        if has_apple_pool {
            // Pool 2 (oak only): 0.5% apple when not using shears
            pools.push(LootPool {
                entries: vec![LootEntry::Item {
                    item: "minecraft:apple".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.005),
                    ],
                    functions: Vec::new(),
                }],
                conditions: vec![not_shears],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            });
        }
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools,
            functions: Vec::new(),
        }
    }

    // Fully-grown crop: 1 food item + Binomial(3, 0.5714) bonus `bonus_item`.
    // Block-state age checks not yet implemented; always applies mature-crop drops.
    // Fortune bonuses not yet implemented; Binomial(3, 0.5714) is the fortune-0 base count.
    pub fn mature_crop_table(food_item: &str, bonus_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![
                LootPool::single(LootEntry::item(format!("minecraft:{food_item}"), 1)),
                LootPool::single(LootEntry::Item {
                    item: format!("minecraft:{bonus_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                        n: 3,
                        p: 0.5714286,
                    })],
                }),
            ],
            functions: vec![LootFunction::ApplyExplosionDecay],
        }
    }

    Some(match key {
        // ── TERRAIN ────────────────────────────────────────────────────────────────────────

        // These drop cobblestone/dirt instead of themselves (silk touch not implemented)
        "stone" => self_drop_table("cobblestone", random_sequence),
        "grass_block" | "mycelium" | "podzol" | "dirt_path" | "farmland" => {
            self_drop_table("dirt", random_sequence)
        }

        // Clay → 4 clay_balls (silk touch not implemented)
        "clay" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:clay_ball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(4.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Gravel: 10% flint (fortune-0 base), else gravel. Both require survives_explosion.
        "gravel" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:flint".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.1),
                    ],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:gravel".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::SurvivesExplosion],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },

        // Terrain self-drops
        "granite"
        | "polished_granite"
        | "diorite"
        | "polished_diorite"
        | "andesite"
        | "polished_andesite"
        | "deepslate"
        | "cobbled_deepslate"
        | "cobblestone"
        | "dirt"
        | "coarse_dirt"
        | "rooted_dirt"
        | "mud"
        | "sand"
        | "red_sand"
        | "sandstone"
        | "chiseled_sandstone"
        | "cut_sandstone"
        | "smooth_sandstone" => self_drop_table(key, random_sequence),

        // ── ORES ───────────────────────────────────────────────────────────────────────────
        // Silk touch (ore block self-drop) not yet implemented; always drops raw product.
        // Fortune bonuses not yet implemented; counts are base values.

        "coal_ore" | "deepslate_coal_ore" => ore_drop_1("coal", random_sequence),
        "iron_ore" | "deepslate_iron_ore" => ore_drop_1("raw_iron", random_sequence),
        "gold_ore" | "deepslate_gold_ore" => ore_drop_1("raw_gold", random_sequence),
        "copper_ore" | "deepslate_copper_ore" => {
            ore_drop_count("raw_copper", 2.0, 5.0, random_sequence)
        }
        "redstone_ore"
        | "lit_redstone_ore"
        | "deepslate_redstone_ore"
        | "lit_deepslate_redstone_ore" => ore_drop_count("redstone", 4.0, 5.0, random_sequence),
        "emerald_ore" | "deepslate_emerald_ore" => ore_drop_1("emerald", random_sequence),
        "lapis_ore" | "deepslate_lapis_ore" => {
            ore_drop_count("lapis_lazuli", 4.0, 9.0, random_sequence)
        }
        "diamond_ore" | "deepslate_diamond_ore" => ore_drop_1("diamond", random_sequence),
        "nether_quartz_ore" => ore_drop_1("quartz", random_sequence),
        "nether_gold_ore" => ore_drop_count("gold_nugget", 2.0, 6.0, random_sequence),

        // ── WOOD ───────────────────────────────────────────────────────────────────────────
        "oak_log"
        | "spruce_log"
        | "birch_log"
        | "jungle_log"
        | "acacia_log"
        | "dark_oak_log"
        | "stripped_oak_log"
        | "stripped_spruce_log"
        | "stripped_birch_log"
        | "stripped_jungle_log"
        | "stripped_acacia_log"
        | "stripped_dark_oak_log"
        | "oak_wood"
        | "spruce_wood"
        | "birch_wood"
        | "jungle_wood"
        | "acacia_wood"
        | "dark_oak_wood"
        | "stripped_oak_wood"
        | "stripped_spruce_wood"
        | "stripped_birch_wood"
        | "stripped_jungle_wood"
        | "stripped_acacia_wood"
        | "stripped_dark_oak_wood"
        | "oak_planks"
        | "spruce_planks"
        | "birch_planks"
        | "jungle_planks"
        | "acacia_planks"
        | "dark_oak_planks" => self_drop_table(key, random_sequence),

        // ── LEAVES ─────────────────────────────────────────────────────────────────────────
        // Silk touch and fortune bonuses not yet implemented. Sapling chance is fortune-0 base.
        // Stick drop: 2% when not using shears. Oak additionally has a 0.5% apple drop.

        "oak_leaves" => leaves_table("oak_leaves", "oak_sapling", 0.05, true, random_sequence),
        "spruce_leaves" => {
            leaves_table("spruce_leaves", "spruce_sapling", 0.05, false, random_sequence)
        }
        "birch_leaves" => {
            leaves_table("birch_leaves", "birch_sapling", 0.05, false, random_sequence)
        }
        // Jungle sapling has a lower base drop chance (2.5% vs 5%)
        "jungle_leaves" => {
            leaves_table("jungle_leaves", "jungle_sapling", 0.025, false, random_sequence)
        }
        "acacia_leaves" => {
            leaves_table("acacia_leaves", "acacia_sapling", 0.05, false, random_sequence)
        }
        "dark_oak_leaves" => {
            leaves_table("dark_oak_leaves", "dark_oak_sapling", 0.05, false, random_sequence)
        }
        "cherry_leaves" => {
            leaves_table("cherry_leaves", "cherry_sapling", 0.05, false, random_sequence)
        }
        "pale_oak_leaves" => {
            leaves_table("pale_oak_leaves", "pale_oak_sapling", 0.05, false, random_sequence)
        }
        // Azalea leaves drop an azalea bush (not a sapling variant)
        "azalea_leaves" => {
            leaves_table("azalea_leaves", "azalea", 0.05, false, random_sequence)
        }
        "flowering_azalea_leaves" => leaves_table(
            "flowering_azalea_leaves",
            "flowering_azalea",
            0.05,
            false,
            random_sequence,
        ),
        // Mangrove leaves: no propagule from breaking; only sticks via the shears-alternative
        "mangrove_leaves" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:mangrove_leaves".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                },
            ]))],
            functions: Vec::new(),
        },

        // ── PLANTS ─────────────────────────────────────────────────────────────────────────

        // Grass-type: shears → self, else 12.5% wheat_seeds
        "short_grass" => grass_table("short_grass", random_sequence),
        "fern" => grass_table("fern", random_sequence),

        // Double-tall grass: same logic, but shears yield 2 items
        "tall_grass" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:short_grass".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.125),
                    ],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },
        "large_fern" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:fern".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.125),
                    ],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },

        // Dead bush: shears → dead_bush, else 0-2 sticks
        "dead_bush" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:dead_bush".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 0.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                },
            ]))],
            functions: Vec::new(),
        },

        // Single-block flowers: always self-drop
        "dandelion"
        | "golden_dandelion"
        | "torchflower"
        | "poppy"
        | "blue_orchid"
        | "allium"
        | "azure_bluet"
        | "red_tulip"
        | "orange_tulip"
        | "white_tulip"
        | "pink_tulip"
        | "oxeye_daisy"
        | "cornflower"
        | "wither_rose"
        | "lily_of_the_valley"
        | "brown_mushroom"
        | "red_mushroom"
        | "wildflowers"
        | "firefly_bush" => self_drop_table(key, random_sequence),

        // Double-tall flowers: drop self (survives_explosion).
        // Java checks block_state_property half=lower (only lower half drops), which we can't
        // evaluate yet. Simplification: always drop on any half break.
        "sunflower" | "lilac" | "rose_bush" | "peony" => {
            self_drop_survives_explosion(key, random_sequence)
        }

        // ── CROPS ──────────────────────────────────────────────────────────────────────────
        // Block-state age checks not yet implemented; always treats crop as fully grown.
        // Fortune bonuses not yet implemented; Binomial(3, 0.5714) is the fortune-0 base count.

        // Wheat at age 7: 1 wheat + Binomial(3, 0.57) bonus wheat_seeds
        "wheat" => mature_crop_table("wheat", "wheat_seeds", random_sequence),
        // Carrots at age 7: 1 carrot + Binomial(3, 0.57) bonus carrots
        "carrots" => mature_crop_table("carrot", "carrot", random_sequence),
        // Beetroots at age 3: 1 beetroot + Binomial(3, 0.57) bonus beetroot_seeds
        "beetroots" => mature_crop_table("beetroot", "beetroot_seeds", random_sequence),
        // Potatoes at age 7: 1 potato + bonus potatoes + 2% poisonous_potato
        "potatoes" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![
                LootPool::single(LootEntry::item("minecraft:potato".to_string(), 1)),
                LootPool::single(LootEntry::Item {
                    item: "minecraft:potato".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                        n: 3,
                        p: 0.5714286,
                    })],
                }),
                LootPool::single(LootEntry::Item {
                    item: "minecraft:poisonous_potato".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: Vec::new(),
                }),
            ],
            functions: vec![LootFunction::ApplyExplosionDecay],
        },

        // ── SPECIAL BLOCKS ─────────────────────────────────────────────────────────────────

        // Glowstone: 2-4 glowstone_dust, limited to 1-4 (silk touch not implemented)
        "glowstone" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:glowstone_dust".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 4.0 }),
                    LootFunction::LimitCount { min: 1, max: 4 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Sea lantern: 2-3 prismarine_crystals, limited to 1-5 (silk touch not implemented)
        "sea_lantern" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:prismarine_crystals".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 3.0 }),
                    LootFunction::LimitCount { min: 1, max: 5 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Bookshelf: 3 books (silk touch not implemented)
        "bookshelf" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:book".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(3.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Snow block: 4 snowballs (silk touch not implemented)
        "snow_block" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:snowball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(4.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Snow layers: 1 snowball (layer count from block state not yet tracked)
        "snow" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:snowball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::SetCount(NumberProvider::Constant(1.0))],
            })],
            functions: Vec::new(),
        },

        // Melon: 3-7 melon_slices, capped at 9 (silk touch not implemented)
        "melon" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:melon_slice".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 3.0, max: 7.0 }),
                    LootFunction::LimitCount { min: 0, max: 9 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Pumpkin / carved pumpkin: self-drop (survives explosion)
        "pumpkin" | "carved_pumpkin" => self_drop_survives_explosion("pumpkin", random_sequence),

        // Simple self-drops conditional on surviving explosion
        "sugar_cane" => self_drop_survives_explosion("sugar_cane", random_sequence),
        "cactus" => self_drop_survives_explosion("cactus", random_sequence),
        "bamboo" => self_drop_survives_explosion("bamboo", random_sequence),

        // ── NO DROP ────────────────────────────────────────────────────────────────────────
        "air"
        | "cave_air"
        | "void_air"
        | "bedrock"
        | "water"
        | "flowing_water"
        | "lava"
        | "flowing_lava"
        | "fire"
        | "soul_fire"
        | "ice"        // silk touch only
        | "packed_ice" // silk touch only
        | "blue_ice"   // silk touch only
        | "glass"      // silk touch only
        | "glass_pane" // silk touch only
        | "nether_portal" => return None,

        _ => return None,
    })
}
