use super::*;

pub(super) fn block_state_tag_fast(block: &str) -> Tag {
    Tag::Compound(vec![("Name".to_string(), Tag::String(block.to_string()))])
}

pub(super) struct GeneratedBlockTags {
    default_block_name: &'static str,
    default_fluid_name: &'static str,
    default_block: Tag,
    default_fluid: Tag,
    air: Tag,
    water: Tag,
    lava: Tag,
    copper_ore: Tag,
    raw_copper_block: Tag,
    granite: Tag,
    deepslate_iron_ore: Tag,
    raw_iron_block: Tag,
    tuff: Tag,
}

impl GeneratedBlockTags {
    pub(super) fn new(settings: &NoiseGeneratorSettings) -> Self {
        Self {
            default_block_name: settings.default_block,
            default_fluid_name: settings.default_fluid,
            default_block: block_state_tag_fast(settings.default_block),
            default_fluid: block_state_tag_fast(settings.default_fluid),
            air: block_state_tag_fast("minecraft:air"),
            water: block_state_tag_fast("minecraft:water"),
            lava: block_state_tag_fast("minecraft:lava"),
            copper_ore: block_state_tag_fast("minecraft:copper_ore"),
            raw_copper_block: block_state_tag_fast("minecraft:raw_copper_block"),
            granite: block_state_tag_fast("minecraft:granite"),
            deepslate_iron_ore: block_state_tag_fast("minecraft:deepslate_iron_ore"),
            raw_iron_block: block_state_tag_fast("minecraft:raw_iron_block"),
            tuff: block_state_tag_fast("minecraft:tuff"),
        }
    }

    pub(super) fn tag_for(&self, block: &str) -> &Tag {
        match block {
            "minecraft:air" => &self.air,
            block if block == self.default_block_name => &self.default_block,
            block if block == self.default_fluid_name => &self.default_fluid,
            "minecraft:water" => &self.water,
            "minecraft:lava" => &self.lava,
            "minecraft:copper_ore" => &self.copper_ore,
            "minecraft:raw_copper_block" => &self.raw_copper_block,
            "minecraft:granite" => &self.granite,
            "minecraft:deepslate_iron_ore" => &self.deepslate_iron_ore,
            "minecraft:raw_iron_block" => &self.raw_iron_block,
            "minecraft:tuff" => &self.tuff,
            _ => &self.default_block,
        }
    }
}

pub(super) fn block_name_from_tag_fast(entry: &Tag) -> Option<&str> {
    let Tag::Compound(fields) = entry else {
        return None;
    };
    fields
        .iter()
        .find_map(|(key, value)| match (key.as_str(), value) {
            ("Name", Tag::String(name)) => Some(name.as_str()),
            _ => None,
        })
}

fn section_index_for_y(min_section: i32, section_count: usize, world_y: i32) -> Option<usize> {
    let index = world_y.div_euclid(16) - min_section;
    (index >= 0 && (index as usize) < section_count).then_some(index as usize)
}

fn section_block_index(world_x: i32, world_y: i32, world_z: i32) -> usize {
    let local_x = world_x.rem_euclid(16) as usize;
    let local_y = world_y.rem_euclid(16) as usize;
    let local_z = world_z.rem_euclid(16) as usize;
    local_y * 256 + local_z * 16 + local_x
}

pub(super) struct GeneratedSectionBlocks {
    pub(super) min_section_y: i32,
    pub(super) sections: Vec<GeneratedSection>,
    pub(super) palette_names: Vec<String>,
    palette_lookup: HashMap<String, u16>,
}

pub(super) struct GeneratedSection {
    pub(super) ids: Vec<u16>,
    pub(super) non_air_blocks: usize,
}

impl GeneratedSectionBlocks {
    pub(super) fn new(min_section_y: i32, section_count: i32) -> Self {
        let mut palette_lookup = HashMap::new();
        palette_lookup.insert("minecraft:air".to_string(), 0);
        Self {
            min_section_y,
            sections: (0..section_count)
                .map(|_| GeneratedSection {
                    ids: vec![0; SECTION_VOLUME],
                    non_air_blocks: 0,
                })
                .collect(),
            palette_names: vec!["minecraft:air".to_string()],
            palette_lookup,
        }
    }

    pub(super) fn id_for(&mut self, block: &str) -> u16 {
        if let Some(id) = self.palette_lookup.get(block).copied() {
            return id;
        }
        let id = u16::try_from(self.palette_names.len())
            .expect("generated chunk block palette exceeded u16 ids");
        self.palette_names.push(block.to_string());
        self.palette_lookup.insert(block.to_string(), id);
        id
    }

    pub(super) fn get_name(&self, world_x: i32, world_y: i32, world_z: i32) -> &str {
        let id = self.get_id(world_x, world_y, world_z) as usize;
        self.palette_names
            .get(id)
            .map(String::as_str)
            .unwrap_or("minecraft:air")
    }

    pub(super) fn get_id(&self, world_x: i32, world_y: i32, world_z: i32) -> u16 {
        let Some(section_index) =
            section_index_for_y(self.min_section_y, self.sections.len(), world_y)
        else {
            return 0;
        };
        let index = section_block_index(world_x, world_y, world_z);
        self.sections[section_index].ids[index]
    }

    fn set_id(&mut self, world_x: i32, world_y: i32, world_z: i32, id: u16) {
        let Some(section_index) =
            section_index_for_y(self.min_section_y, self.sections.len(), world_y)
        else {
            return;
        };
        let index = section_block_index(world_x, world_y, world_z);
        let section = &mut self.sections[section_index];
        let old_id = section.ids[index];
        if old_id == id {
            return;
        }
        if old_id == 0 && id != 0 {
            section.non_air_blocks += 1;
        } else if old_id != 0 && id == 0 {
            section.non_air_blocks = section.non_air_blocks.saturating_sub(1);
        }
        section.ids[index] = id;
    }

    pub(super) fn set_name(&mut self, world_x: i32, world_y: i32, world_z: i32, block: &str) {
        let id = self.id_for(block);
        self.set_id(world_x, world_y, world_z, id);
    }

    pub(super) fn to_paletted_containers(&self) -> Vec<PalettedContainer> {
        self.sections
            .iter()
            .map(|section| {
                if section.non_air_blocks == 0 {
                    return PalettedContainer::single(
                        block_state_tag_fast("minecraft:air"),
                        SECTION_VOLUME,
                    );
                }

                let mut local_lookup = HashMap::<u16, u64>::new();
                let mut palette = Vec::<Tag>::new();
                let mut indices = Vec::<u64>::with_capacity(SECTION_VOLUME);
                for id in section.ids.iter().copied() {
                    let local_index = if let Some(index) = local_lookup.get(&id).copied() {
                        index
                    } else {
                        let index = palette.len() as u64;
                        let name = self
                            .palette_names
                            .get(id as usize)
                            .map(String::as_str)
                            .unwrap_or("minecraft:air");
                        palette.push(block_state_tag_fast(name));
                        local_lookup.insert(id, index);
                        index
                    };
                    indices.push(local_index);
                }

                if palette.len() == 1 {
                    PalettedContainer::single(palette.remove(0), SECTION_VOLUME)
                } else {
                    PalettedContainer {
                        data: Some(pack_palette_indices(
                            &indices,
                            palette_bits_for_size(palette.len()),
                        )),
                        palette,
                        expected_entries: SECTION_VOLUME,
                    }
                }
            })
            .collect()
    }
}

pub(super) fn get_generated_block<'a>(
    sections: &'a [PalettedContainer],
    min_section: i32,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> &'a str {
    let Some(section_index) = section_index_for_y(min_section, sections.len(), world_y) else {
        return "minecraft:air";
    };
    let index = section_block_index(world_x, world_y, world_z);
    sections[section_index]
        .get_entry(index)
        .and_then(block_name_from_tag_fast)
        .unwrap_or("minecraft:air")
}

pub(super) fn set_generated_block(
    sections: &mut [PalettedContainer],
    min_section: i32,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    block: &str,
) {
    let Some(section_index) = section_index_for_y(min_section, sections.len(), world_y) else {
        return;
    };
    let index = section_block_index(world_x, world_y, world_z);
    sections[section_index].set_entry(index, block_state_tag_fast(block));
}

pub(super) fn set_generated_block_tag(
    sections: &mut [PalettedContainer],
    min_section: i32,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    tag: &Tag,
) {
    let Some(section_index) = section_index_for_y(min_section, sections.len(), world_y) else {
        return;
    };
    let index = section_block_index(world_x, world_y, world_z);
    sections[section_index].set_entry_ref(index, tag);
}
