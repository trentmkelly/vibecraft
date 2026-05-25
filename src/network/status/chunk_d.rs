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
pub fn write_superflat_spawn_chunk_packet<W: Write>(
    writer: &mut W,
    x: i32,
    z: i32,
) -> io::Result<()> {
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

pub fn visible_spawn_terrain_height(
    chunk_x: i32,
    chunk_z: i32,
    local_x: usize,
    local_z: usize,
) -> i32 {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let broad = (world_x.div_euclid(12) + world_z.div_euclid(14)).rem_euclid(8);
    let terrace = (world_x.div_euclid(5) - world_z.div_euclid(7)).rem_euclid(6);
    let wrinkle = (world_x.wrapping_mul(31) ^ world_z.wrapping_mul(17)) & 3;
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
    let hash = world_x.wrapping_mul(19_349_663) ^ world_z.wrapping_mul(83_492_791);
    match hash.rem_euclid(43) {
        0 => STONE_BLOCK_STATE_ID,
        9 => GRANITE_BLOCK_STATE_ID,
        18 => DIORITE_BLOCK_STATE_ID,
        27 => ANDESITE_BLOCK_STATE_ID,
        34 | 41 => DIRT_BLOCK_STATE_ID,
        _ => GRASS_BLOCK_STATE_ID,
    }
}
