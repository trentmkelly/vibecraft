use super::*;

impl NumberFormat {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Blank => write_var_i32(writer, 0),
            Self::Styled { style } => {
                write_var_i32(writer, 1)?;
                write_network_tag(writer, style)
            }
            Self::Fixed { value } => {
                write_var_i32(writer, 2)?;
                write_network_tag(writer, value)
            }
        }
    }
}

impl ClientboundSetPlayerTeamPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 32767)?;
        match &self.method {
            TeamPacketMethod::Create {
                parameters,
                players,
            } => {
                writer.write_all(&[0])?;
                parameters.write(writer)?;
                write_team_players(writer, players)
            }
            TeamPacketMethod::Remove => writer.write_all(&[1]),
            TeamPacketMethod::Update { parameters } => {
                writer.write_all(&[2])?;
                parameters.write(writer)
            }
            TeamPacketMethod::AddPlayers { players } => {
                writer.write_all(&[3])?;
                write_team_players(writer, players)
            }
            TeamPacketMethod::RemovePlayers { players } => {
                writer.write_all(&[4])?;
                write_team_players(writer, players)
            }
        }
    }
}

impl ClientboundOpenScreenPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.menu_type_id)?;
        write_network_tag(writer, &self.title)
    }
}

impl TeamPacketParameters {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.display_name)?;
        writer.write_all(&[self.options])?;
        write_var_i32(writer, self.nametag_visibility as i32)?;
        write_var_i32(writer, self.collision_rule as i32)?;
        write_var_i32(writer, self.color_id)?;
        write_network_tag(writer, &self.prefix)?;
        write_network_tag(writer, &self.suffix)
    }
}

pub(super) fn write_team_players<W: Write>(writer: &mut W, players: &[String]) -> io::Result<()> {
    write_var_i32(writer, players.len() as i32)?;
    for player in players {
        write_string(writer, player, 32767)?;
    }
    Ok(())
}

impl ClientboundResourcePackPopPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.id {
            Some(id) => {
                write_bool(writer, true)?;
                write_uuid(writer, id)
            }
            None => write_bool(writer, false),
        }
    }
}

impl EntitySpawnBundle {
    pub fn instructions(self) -> Vec<PlayInstruction> {
        let mut instructions = vec![PlayInstruction::AddEntity(self.spawn)];
        if let Some(metadata) = self.metadata {
            instructions.push(PlayInstruction::SetEntityData(metadata));
        }
        if let Some(velocity) = self.velocity {
            instructions.push(PlayInstruction::SetEntityMotion(velocity));
        }
        if let Some(equipment) = self.equipment {
            instructions.push(PlayInstruction::SetEquipment(equipment));
        }
        if let Some(attributes) = self.attributes {
            instructions.push(PlayInstruction::UpdateAttributes(attributes));
        }
        instructions.extend(
            self.effects
                .into_iter()
                .map(PlayInstruction::UpdateMobEffect),
        );
        instructions
    }
}

pub(super) fn pack_degrees(degrees: f32) -> u8 {
    ((degrees * 256.0 / 360.0).floor() as i32 & 255) as u8
}

impl ClientboundLevelChunkPacketData {
    pub const MAX_BUFFER_SIZE: usize = 2_097_152;

    pub fn from_chunk(chunk: &LevelChunk) -> Self {
        let mut buffer = Vec::new();
        // Java's LevelChunk owns a dense section array sized from the dimension
        // height accessor.  The anvil NBT section list is sparse, so packet
        // serialization must rebuild the dense overworld range or the client
        // renders stored sections at the wrong Y.
        for section_y in
            OVERWORLD_MIN_SECTION_Y..OVERWORLD_MIN_SECTION_Y + OVERWORLD_SECTION_COUNT as i32
        {
            if let Err(err) =
                NetworkChunkSection::from_chunk_section_y(chunk, section_y).write(&mut buffer)
            {
                panic!("writing chunk section to Vec failed: {err}");
            }
        }
        assert!(
            buffer.len() <= Self::MAX_BUFFER_SIZE,
            "chunk packet buffer exceeds vanilla two-megabyte guard"
        );

        Self {
            heightmaps: chunk
                .heightmaps
                .iter()
                .filter_map(|(name, tag)| match tag {
                    Tag::LongArray(values) if heightmap_sent_to_client(name) => {
                        Some((name.clone(), values.clone()))
                    }
                    _ => None,
                })
                .collect(),
            buffer,
            block_entity_count: chunk.block_entities.len(),
            block_entities: chunk
                .block_entities
                .iter()
                .filter_map(LevelChunkBlockEntityInfo::from_nbt)
                .collect(),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut heightmaps = self
            .heightmaps
            .iter()
            .map(|(name, values)| Ok((heightmap_type_id(name)?, values)))
            .collect::<io::Result<Vec<_>>>()?;
        heightmaps.sort_by_key(|(type_id, _)| *type_id);
        write_var_i32(writer, heightmaps.len() as i32)?;
        for (type_id, values) in heightmaps {
            write_var_i32(writer, type_id)?;
            write_var_i32(writer, values.len() as i32)?;
            for value in values {
                write_i64(writer, *value)?;
            }
        }
        if self.buffer.len() > Self::MAX_BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chunk packet buffer exceeds vanilla two-megabyte guard",
            ));
        }
        write_var_i32(writer, self.buffer.len() as i32)?;
        writer.write_all(&self.buffer)?;
        write_var_i32(writer, self.block_entities.len() as i32)?;
        for block_entity in &self.block_entities {
            block_entity.write(writer)?;
        }
        Ok(())
    }
}

impl LevelChunkBlockEntityInfo {
    pub(super) fn from_nbt(tag: &Tag) -> Option<Self> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        let x = compound_i32(fields, "x")?;
        let y = compound_i32(fields, "y")?;
        let z = compound_i32(fields, "z")?;
        let id = compound_string(fields, "id")?;
        Some(Self {
            packed_xz: (((x & 15) << 4) | (z & 15)) as u8,
            y: y as i16,
            block_entity_type_id: block_entity_type_network_id(id).unwrap_or(0),
            tag: tag.clone(),
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.packed_xz])?;
        write_i16(writer, self.y)?;
        write_var_i32(writer, self.block_entity_type_id)?;
        write_network_compound_tag(writer, &self.tag)
    }
}

impl ClientboundLightUpdatePacketData {
    pub const DATA_LAYER_SIZE: usize = 2048;

    pub fn from_chunk(chunk: &LevelChunk) -> Self {
        let mut data = Self {
            sky_y_mask: Vec::new(),
            block_y_mask: Vec::new(),
            empty_sky_y_mask: Vec::new(),
            empty_block_y_mask: Vec::new(),
            sky_updates: Vec::new(),
            block_updates: Vec::new(),
        };

        // Java: `for (int i = 0; i < lightEngine.getLightSectionCount(); i++)`.
        // `lightSectionCount = sectionsCount + 2` because the engine pads the
        // chunk with one section above and one below (LIGHT_SECTION_PADDING).
        // `sectionIndex = 0` therefore maps to `minLightSection = minSectionY
        // - 1 = -5` on the overworld, not the lowest world section -4. Without
        // this padding, every bit position is off by one and the vanilla
        // client interprets all our sky/block updates as belonging to the
        // section directly below the one we meant.
        let min_light_section = OVERWORLD_MIN_SECTION_Y - 1;
        let light_section_count = OVERWORLD_SECTION_COUNT + 2;
        for section_index in 0..light_section_count {
            let section_y = min_light_section + section_index as i32;
            let section = chunk
                .sections
                .iter()
                .find(|section| i32::from(section.y) == section_y);
            data.add_layer(
                section_index,
                section.and_then(|section| section.sky_light.as_deref()),
                true,
            );
            data.add_layer(
                section_index,
                section.and_then(|section| section.block_light.as_deref()),
                false,
            );
        }

        data
    }

    pub fn from_chunk_sections(sections: &[ChunkSection]) -> Self {
        let mut data = Self {
            sky_y_mask: Vec::new(),
            block_y_mask: Vec::new(),
            empty_sky_y_mask: Vec::new(),
            empty_block_y_mask: Vec::new(),
            sky_updates: Vec::new(),
            block_updates: Vec::new(),
        };

        // Section index 0 corresponds to the padding section *below* the
        // first stored section; the first real section therefore goes into
        // bit 1. This mirrors `from_chunk` so tests that hand-craft a
        // section list see the same bit layout.
        for (logical_index, section) in sections.iter().enumerate() {
            let section_index = logical_index + 1;
            data.add_layer(section_index, section.sky_light.as_deref(), true);
            data.add_layer(section_index, section.block_light.as_deref(), false);
        }

        data
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bitset(writer, &self.sky_y_mask)?;
        write_bitset(writer, &self.block_y_mask)?;
        write_bitset(writer, &self.empty_sky_y_mask)?;
        write_bitset(writer, &self.empty_block_y_mask)?;
        write_collection(writer, &self.sky_updates, |writer, layer| {
            write_data_layer(writer, layer)
        })?;
        write_collection(writer, &self.block_updates, |writer, layer| {
            write_data_layer(writer, layer)
        })
    }

    pub(super) fn add_layer(&mut self, section_index: usize, layer: Option<&[i8]>, sky: bool) {
        let Some(layer) = layer else {
            return;
        };
        assert_eq!(
            layer.len(),
            Self::DATA_LAYER_SIZE,
            "light update data layers are always 2048 bytes"
        );
        let empty = layer.iter().all(|byte| *byte == 0);
        let mask = if sky {
            if empty {
                &mut self.empty_sky_y_mask
            } else {
                self.sky_updates.push(layer.to_vec());
                &mut self.sky_y_mask
            }
        } else if empty {
            &mut self.empty_block_y_mask
        } else {
            self.block_updates.push(layer.to_vec());
            &mut self.block_y_mask
        };
        set_bit(mask, section_index);
    }
}

impl NetworkChunkSection {
    pub fn from_storage_section(section: &ChunkSection) -> Self {
        Self {
            non_empty_block_count: section_non_empty_block_count(&section.block_states),
            fluid_count: section_fluid_count(&section.block_states),
            block_states: NetworkPalettedContainer::from_storage_container(
                &section.block_states,
                PaletteKind::BlockState,
            ),
            biomes: NetworkPalettedContainer::from_storage_container(
                &section.biomes,
                PaletteKind::Biome,
            ),
        }
    }

    pub(super) fn from_chunk_section_y(chunk: &LevelChunk, section_y: i32) -> Self {
        chunk
            .sections
            .iter()
            .find(|section| i32::from(section.y) == section_y)
            .map(Self::from_storage_section)
            .unwrap_or_else(Self::empty)
    }

    pub(super) fn empty() -> Self {
        Self {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: NetworkPalettedContainer::single(0),
            biomes: NetworkPalettedContainer::single(40),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.non_empty_block_count.to_be_bytes())?;
        writer.write_all(&self.fluid_count.to_be_bytes())?;
        self.block_states.write(writer)?;
        self.biomes.write(writer)
    }
}

impl NetworkPalettedContainer {
    pub fn single(global_id: i32) -> Self {
        Self {
            bits_per_entry: 0,
            palette_ids: vec![global_id],
            data: Vec::new(),
            uses_global_palette: false,
        }
    }

    pub(super) fn from_storage_container(tag: &Tag, kind: PaletteKind) -> Self {
        let Ok(container) = PalettedContainer::from_nbt(tag, 0) else {
            return Self::single(0);
        };
        let palette_ids = container
            .palette
            .iter()
            .map(|entry| storage_palette_entry_network_id(entry, kind))
            .collect::<Vec<_>>();
        let data = container.data.unwrap_or_default();
        let palette_ids = if palette_ids.is_empty() {
            vec![0]
        } else {
            palette_ids
        };
        let storage_bits = if data.is_empty() {
            0
        } else {
            packed_storage_bits_per_entry(container.palette.len())
        };
        let uses_global_palette = storage_bits > kind.max_indirect_bits();
        if uses_global_palette {
            let bits_per_entry = direct_palette_bits(&palette_ids).max(kind.min_direct_bits());
            let indices = crate::storage::chunk::unpack_palette_indices(
                &data,
                storage_bits,
                kind.entry_count(),
            );
            let global_ids = indices
                .into_iter()
                .map(|index| palette_ids.get(index as usize).copied().unwrap_or(0) as u64)
                .collect::<Vec<_>>();
            return Self {
                bits_per_entry: bits_per_entry as u8,
                palette_ids: Vec::new(),
                data: crate::storage::chunk::pack_palette_indices(&global_ids, bits_per_entry),
                uses_global_palette: true,
            };
        }

        Self {
            bits_per_entry: storage_bits as u8,
            palette_ids,
            data,
            uses_global_palette: false,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.bits_per_entry])?;
        if self.bits_per_entry == 0 {
            write_var_i32(writer, self.palette_ids.first().copied().unwrap_or(0))?;
        } else if !self.uses_global_palette {
            write_var_i32(writer, self.palette_ids.len() as i32)?;
            for id in &self.palette_ids {
                write_var_i32(writer, *id)?;
            }
        }
        for word in &self.data {
            writer.write_all(&word.to_be_bytes())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaletteKind {
    BlockState,
    Biome,
}

impl PaletteKind {
    pub(super) fn entry_count(self) -> usize {
        match self {
            Self::BlockState => 4096,
            Self::Biome => 64,
        }
    }

    pub(super) fn max_indirect_bits(self) -> usize {
        match self {
            Self::BlockState => 8,
            Self::Biome => 3,
        }
    }

    pub(super) fn min_direct_bits(self) -> usize {
        match self {
            Self::BlockState => 15,
            Self::Biome => 7,
        }
    }
}

pub(super) fn set_bit(mask: &mut Vec<u64>, index: usize) {
    let word = index / 64;
    if mask.len() <= word {
        mask.resize(word + 1, 0);
    }
    mask[word] |= 1_u64 << (index % 64);
}

pub(super) fn write_data_layer<W: Write>(writer: &mut W, layer: &[i8]) -> io::Result<()> {
    if layer.len() != ClientboundLightUpdatePacketData::DATA_LAYER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "light update layer must be 2048 bytes",
        ));
    }
    write_var_i32(writer, layer.len() as i32)?;
    let bytes = layer.iter().map(|byte| *byte as u8).collect::<Vec<_>>();
    writer.write_all(&bytes)
}

pub(super) fn storage_palette_entry_network_id(tag: &Tag, kind: PaletteKind) -> i32 {
    match tag {
        Tag::Int(id) => *id,
        Tag::String(name) => match kind {
            PaletteKind::BlockState => storage_block_state_name_network_id(name).unwrap_or(0),
            PaletteKind::Biome => biome_name_network_id(name).unwrap_or(0),
        },
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(name, value)| {
                (name == "id" || name == "network_id")
                    .then_some(value)
                    .and_then(|value| match value {
                        Tag::Int(id) => Some(*id),
                        _ => None,
                    })
            })
            .or_else(|| {
                fields.iter().find_map(|(name, value)| {
                    (name == "Name")
                        .then_some(value)
                        .and_then(|value| match value {
                            Tag::String(name) => match kind {
                                PaletteKind::BlockState => {
                                    storage_block_state_name_network_id(name)
                                }
                                PaletteKind::Biome => biome_name_network_id(name),
                            },
                            _ => None,
                        })
                })
            })
            .unwrap_or(0),
        _ => 0,
    }
}

pub(super) fn heightmap_type_id(name: &str) -> io::Result<i32> {
    match name {
        "WORLD_SURFACE_WG" => Ok(0),
        "WORLD_SURFACE" => Ok(1),
        "OCEAN_FLOOR_WG" => Ok(2),
        "OCEAN_FLOOR" => Ok(3),
        "MOTION_BLOCKING" => Ok(4),
        "MOTION_BLOCKING_NO_LEAVES" => Ok(5),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown heightmap type {name}"),
        )),
    }
}

pub(super) fn heightmap_sent_to_client(name: &str) -> bool {
    matches!(
        name,
        "WORLD_SURFACE" | "MOTION_BLOCKING" | "MOTION_BLOCKING_NO_LEAVES"
    )
}

pub(super) fn block_entity_type_network_id(name: &str) -> Option<i32> {
    let normalized = name.strip_prefix("minecraft:").unwrap_or(name);
    BLOCK_ENTITY_TYPES
        .iter()
        .position(|entry| entry.key == normalized)
        .map(|index| index as i32)
}

pub(super) fn compound_i32(fields: &[(String, Tag)], name: &str) -> Option<i32> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .and_then(|(_, value)| match value {
            Tag::Int(value) => Some(*value),
            _ => None,
        })
}

pub(super) fn compound_string<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .and_then(|(_, value)| match value {
            Tag::String(value) => Some(value.as_str()),
            _ => None,
        })
}

pub(super) fn section_non_empty_block_count(tag: &Tag) -> i16 {
    section_palette_entry_count(tag, |entry| !storage_palette_entry_is_air(entry))
}

pub(super) fn section_fluid_count(tag: &Tag) -> i16 {
    section_palette_entry_count(tag, storage_palette_entry_is_fluid)
}

pub(super) fn section_palette_entry_count<F>(tag: &Tag, is_match: F) -> i16
where
    F: FnMut(&Tag) -> bool,
{
    let Ok(container) = PalettedContainer::from_nbt(tag, 4096) else {
        return 0;
    };
    let matching_entries = container.palette.iter().map(is_match).collect::<Vec<_>>();
    if matching_entries.is_empty() {
        return 0;
    }
    let Some(data) = &container.data else {
        return if matching_entries.first().copied().unwrap_or(false) {
            4096
        } else {
            0
        };
    };

    let bits_per_entry = packed_storage_bits_per_entry(container.palette.len());
    let values_per_long = 64 / bits_per_entry;
    let mut count = 0_i16;
    for index in 0..container.expected_entries {
        let word_index = index / values_per_long;
        let Some(word) = data.get(word_index) else {
            break;
        };
        let bit_index = (index - word_index * values_per_long) * bits_per_entry;
        let palette_index = ((*word as u64) >> bit_index) & ((1_u64 << bits_per_entry) - 1);
        if matching_entries
            .get(palette_index as usize)
            .copied()
            .unwrap_or(false)
        {
            count += 1;
        }
    }
    count
}

pub(super) fn packed_storage_bits_per_entry(palette_len: usize) -> usize {
    let palette_len = palette_len.max(1) as u64;
    let needed = 64 - palette_len.saturating_sub(1).leading_zeros() as usize;
    needed.max(4)
}

pub(super) fn direct_palette_bits(palette_ids: &[i32]) -> usize {
    let max_id = palette_ids.iter().copied().max().unwrap_or(0).max(0) as u64;
    (64 - max_id.leading_zeros() as usize).max(1)
}

pub(super) fn storage_palette_entry_is_air(tag: &Tag) -> bool {
    match tag {
        Tag::Int(id) => *id == 0,
        Tag::String(name) => block_state_name_is_air(name),
        Tag::Compound(fields) => fields.iter().any(|(name, value)| {
            (name == "Name" || name == "id")
                && matches!(value, Tag::String(block_name) if block_state_name_is_air(block_name))
        }),
        _ => false,
    }
}

pub(super) fn storage_palette_entry_is_fluid(tag: &Tag) -> bool {
    match tag {
        Tag::Int(id) => matches!(*id, 86 | 102),
        Tag::String(name) => block_state_name_has_fluid(name),
        Tag::Compound(fields) => {
            fields.iter().any(|(field_name, value)| {
                (field_name == "Name" || field_name == "id")
                    && matches!(value, Tag::String(block_name) if block_state_name_has_fluid(block_name))
            }) || compound_string_property_is_true(fields, "waterlogged")
        }
        _ => false,
    }
}

pub(super) fn storage_block_state_name_network_id(name: &str) -> Option<i32> {
    block_state_name_network_id(name)
        .or_else(|| block_state_name_network_id(block_state_base_name(name)))
}

pub(super) fn block_state_name_is_air(name: &str) -> bool {
    matches!(
        block_state_base_name(name),
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

pub(super) fn block_state_name_has_fluid(name: &str) -> bool {
    matches!(
        block_state_base_name(name),
        "minecraft:water" | "minecraft:flowing_water" | "minecraft:lava" | "minecraft:flowing_lava"
    ) || block_state_string_property_is_true(name, "waterlogged")
}

pub(super) fn block_state_base_name(name: &str) -> &str {
    name.split_once('[').map_or(name, |(base, _)| base)
}

pub(super) fn block_state_string_property_is_true(name: &str, property_name: &str) -> bool {
    let Some((_, properties)) = name.split_once('[') else {
        return false;
    };
    let expected = format!("{property_name}=true");
    properties
        .trim_end_matches(']')
        .split(',')
        .any(|property| property == expected)
}

pub(super) fn compound_string_property_is_true(
    fields: &[(String, Tag)],
    property_name: &str,
) -> bool {
    fields.iter().any(|(field_name, value)| {
        field_name == "Properties"
            && matches!(
                value,
                Tag::Compound(properties)
                    if properties.iter().any(|(name, value)| {
                        name == property_name
                            && matches!(value, Tag::String(value) if value == "true")
                    })
            )
    })
}
