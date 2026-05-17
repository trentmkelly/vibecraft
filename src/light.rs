#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

use crate::storage::chunk::SECTION_VOLUME;

pub const MAX_LIGHT_LEVEL: u8 = 15;
pub const DATA_LAYER_SIZE: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayer {
    data: Option<Vec<i8>>,
    default_value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightBlock {
    pub opacity: u8,
    pub emission: u8,
    pub occludes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightSection {
    pub blocks: Vec<LightBlock>,
    pub block_light: DataLayer,
    pub sky_light: DataLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightLayerKind {
    Block,
    Sky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightUpdate {
    pub layer: LightLayerKind,
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightEngine {
    pub section: LightSection,
    pub has_sky_light: bool,
}

impl DataLayer {
    pub fn new(default_value: u8) -> Self {
        Self {
            data: None,
            default_value: default_value & MAX_LIGHT_LEVEL,
        }
    }

    pub fn from_bytes(data: Vec<i8>) -> Result<Self, String> {
        if data.len() != DATA_LAYER_SIZE {
            return Err(format!(
                "DataLayer should be 2048 bytes not: {}",
                data.len()
            ));
        }
        Ok(Self {
            data: Some(data),
            default_value: 0,
        })
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> u8 {
        self.get_index(index(x, y, z))
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, value: u8) {
        self.set_index(index(x, y, z), value);
    }

    pub fn fill(&mut self, value: u8) {
        self.default_value = value & MAX_LIGHT_LEVEL;
        self.data = None;
    }

    pub fn bytes(&mut self) -> &mut Vec<i8> {
        let default_value = self.default_value;
        self.data.get_or_insert_with(|| {
            let packed = pack_filled(default_value);
            vec![packed as i8; DATA_LAYER_SIZE]
        })
    }

    pub fn into_bytes(mut self) -> Vec<i8> {
        self.bytes().clone()
    }

    pub fn is_definitely_filled_with(&self, value: u8) -> bool {
        self.data.is_none() && self.default_value == (value & MAX_LIGHT_LEVEL)
    }

    fn get_index(&self, index: usize) -> u8 {
        match &self.data {
            Some(data) => {
                let byte = data[index >> 1] as u8;
                (byte >> (4 * (index & 1))) & MAX_LIGHT_LEVEL
            }
            None => self.default_value,
        }
    }

    fn set_index(&mut self, index: usize, value: u8) {
        let value = value & MAX_LIGHT_LEVEL;
        let byte_index = index >> 1;
        let shift = 4 * (index & 1);
        let mask = !(MAX_LIGHT_LEVEL << shift);
        let data = self.bytes();
        let old = data[byte_index] as u8;
        data[byte_index] = ((old & mask) | (value << shift)) as i8;
    }
}

impl LightBlock {
    pub const AIR: Self = Self {
        opacity: 0,
        emission: 0,
        occludes: false,
    };
    pub const STONE: Self = Self {
        opacity: 15,
        emission: 0,
        occludes: true,
    };

    pub fn light(emission: u8) -> Self {
        Self {
            opacity: 0,
            emission: emission.min(MAX_LIGHT_LEVEL),
            occludes: false,
        }
    }

    fn propagation_opacity(self) -> u8 {
        self.opacity.max(1)
    }

    fn blocks_sky_source_edge(self) -> bool {
        self.opacity != 0 || self.occludes
    }
}

impl LightSection {
    pub fn air() -> Self {
        Self {
            blocks: vec![LightBlock::AIR; SECTION_VOLUME],
            block_light: DataLayer::new(0),
            sky_light: DataLayer::new(15),
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: LightBlock) {
        self.blocks[index(x, y, z)] = block;
    }

    pub fn block(&self, x: u8, y: u8, z: u8) -> LightBlock {
        self.blocks[index(x, y, z)]
    }
}

impl LightEngine {
    pub fn new(section: LightSection, has_sky_light: bool) -> Self {
        Self {
            section,
            has_sky_light,
        }
    }

    pub fn initialize_light(&mut self) -> Vec<LightUpdate> {
        let mut updates = self.propagate_block_light_sources();
        updates.extend(self.initialize_sky_light_sources());
        updates
    }

    pub fn propagate_block_light_sources(&mut self) -> Vec<LightUpdate> {
        self.section.block_light.fill(0);
        let mut queue = VecDeque::new();
        let mut updates = Vec::new();

        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let block = self.section.block(x, y, z);
                    if block.emission > 0 {
                        self.section.block_light.set(x, y, z, block.emission);
                        queue.push_back((x, y, z, block.emission));
                        updates.push(LightUpdate {
                            layer: LightLayerKind::Block,
                            x,
                            y,
                            z,
                            level: block.emission,
                        });
                    }
                }
            }
        }

        while let Some((x, y, z, level)) = queue.pop_front() {
            for (nx, ny, nz) in neighbors(x, y, z) {
                let target = self.section.block(nx, ny, nz);
                let next_level = level.saturating_sub(target.propagation_opacity());
                if next_level > self.section.block_light.get(nx, ny, nz) {
                    self.section.block_light.set(nx, ny, nz, next_level);
                    updates.push(LightUpdate {
                        layer: LightLayerKind::Block,
                        x: nx,
                        y: ny,
                        z: nz,
                        level: next_level,
                    });
                    if next_level > 1 {
                        queue.push_back((nx, ny, nz, next_level));
                    }
                }
            }
        }

        updates
    }

    pub fn initialize_sky_light_sources(&mut self) -> Vec<LightUpdate> {
        if !self.has_sky_light {
            self.section.sky_light.fill(0);
            return Vec::new();
        }

        self.section.sky_light.fill(0);
        let mut queue = VecDeque::new();
        let mut updates = Vec::new();

        for z in 0..16 {
            for x in 0..16 {
                let lowest = lowest_sky_source_y(&self.section, x, z);
                for y in lowest..16 {
                    self.section.sky_light.set(x, y, z, MAX_LIGHT_LEVEL);
                    queue.push_back((x, y, z, MAX_LIGHT_LEVEL));
                    updates.push(LightUpdate {
                        layer: LightLayerKind::Sky,
                        x,
                        y,
                        z,
                        level: MAX_LIGHT_LEVEL,
                    });
                }
            }
        }

        while let Some((x, y, z, level)) = queue.pop_front() {
            for (nx, ny, nz) in neighbors(x, y, z) {
                let target = self.section.block(nx, ny, nz);
                let next_level = level.saturating_sub(target.propagation_opacity());
                if next_level > self.section.sky_light.get(nx, ny, nz) {
                    self.section.sky_light.set(nx, ny, nz, next_level);
                    updates.push(LightUpdate {
                        layer: LightLayerKind::Sky,
                        x: nx,
                        y: ny,
                        z: nz,
                        level: next_level,
                    });
                    if next_level > 1 {
                        queue.push_back((nx, ny, nz, next_level));
                    }
                }
            }
        }

        updates
    }

    pub fn changed_lights_for_block(
        &mut self,
        x: u8,
        y: u8,
        z: u8,
        new_block: LightBlock,
    ) -> Vec<LightUpdate> {
        self.section.set_block(x, y, z, new_block);
        self.initialize_light()
    }
}

pub fn has_different_light_properties(old: LightBlock, new: LightBlock) -> bool {
    old.opacity != new.opacity || old.emission != new.emission || old.occludes || new.occludes
}

fn lowest_sky_source_y(section: &LightSection, x: u8, z: u8) -> u8 {
    let mut top = LightBlock::AIR;
    for y in (0..16).rev() {
        let bottom = section.block(x, y, z);
        if bottom.blocks_sky_source_edge() || (top.occludes && bottom.occludes) {
            return (y + 1).min(15);
        }
        top = bottom;
    }
    0
}

fn neighbors(x: u8, y: u8, z: u8) -> impl Iterator<Item = (u8, u8, u8)> {
    let mut values = Vec::with_capacity(6);
    if x > 0 {
        values.push((x - 1, y, z));
    }
    if x < 15 {
        values.push((x + 1, y, z));
    }
    if y > 0 {
        values.push((x, y - 1, z));
    }
    if y < 15 {
        values.push((x, y + 1, z));
    }
    if z > 0 {
        values.push((x, y, z - 1));
    }
    if z < 15 {
        values.push((x, y, z + 1));
    }
    values.into_iter()
}

fn index(x: u8, y: u8, z: u8) -> usize {
    y as usize * 16 * 16 + z as usize * 16 + x as usize
}

fn pack_filled(value: u8) -> u8 {
    let value = value & MAX_LIGHT_LEVEL;
    value | (value << 4)
}

pub fn section_light_arrays(section: &LightSection) -> BTreeMap<&'static str, Vec<i8>> {
    let mut block_light = section.block_light.clone();
    let mut sky_light = section.sky_light.clone();
    BTreeMap::from([
        ("BlockLight", block_light.bytes().clone()),
        ("SkyLight", sky_light.bytes().clone()),
    ])
}

#[cfg(test)]
mod tests {
    use super::{
        has_different_light_properties, section_light_arrays, DataLayer, LightBlock, LightEngine,
        LightLayerKind, LightSection, DATA_LAYER_SIZE,
    };

    #[test]
    fn data_layer_uses_vanilla_nibble_order_and_size() {
        let mut layer = DataLayer::new(0);
        layer.set(0, 0, 0, 3);
        layer.set(1, 0, 0, 12);
        layer.set(0, 1, 0, 5);
        layer.set(15, 15, 15, 14);

        let bytes = layer.into_bytes();
        assert_eq!(bytes.len(), DATA_LAYER_SIZE);
        assert_eq!(bytes[0] as u8, 0xC3);
        assert_eq!(bytes[128] as u8 & 0x0F, 5);
        assert_eq!(bytes[2047] as u8 >> 4, 14);
    }

    #[test]
    fn data_layer_homogenous_fill_materializes_like_vanilla() {
        let mut layer = DataLayer::new(15);
        assert!(layer.is_definitely_filled_with(15));
        assert_eq!(layer.get(7, 8, 9), 15);
        assert!(layer.bytes().iter().all(|byte| *byte as u8 == 0xFF));

        layer.fill(0);
        assert!(layer.is_definitely_filled_with(0));
    }

    #[test]
    fn block_light_propagates_from_emission_with_minimum_opacity() {
        let mut section = LightSection::air();
        section.set_block(8, 8, 8, LightBlock::light(14));
        section.set_block(10, 8, 8, LightBlock::STONE);
        let mut engine = LightEngine::new(section, true);

        let updates = engine.propagate_block_light_sources();
        assert!(updates
            .iter()
            .any(|update| update.layer == LightLayerKind::Block));
        assert_eq!(engine.section.block_light.get(8, 8, 8), 14);
        assert_eq!(engine.section.block_light.get(9, 8, 8), 13);
        assert_eq!(engine.section.block_light.get(10, 8, 8), 0);
        assert_eq!(engine.section.block_light.get(8, 8, 10), 12);
    }

    #[test]
    fn sky_light_columns_start_below_first_occluding_edge() {
        let mut section = LightSection::air();
        section.set_block(4, 10, 4, LightBlock::STONE);
        let mut engine = LightEngine::new(section, true);

        engine.initialize_sky_light_sources();
        assert_eq!(engine.section.sky_light.get(4, 15, 4), 15);
        assert_eq!(engine.section.sky_light.get(4, 11, 4), 15);
        assert_eq!(engine.section.sky_light.get(4, 10, 4), 0);
        assert_eq!(engine.section.sky_light.get(4, 9, 4), 14);
    }

    #[test]
    fn dimensions_without_sky_light_keep_sky_layer_dark() {
        let mut engine = LightEngine::new(LightSection::air(), false);
        let updates = engine.initialize_light();
        assert!(updates
            .iter()
            .all(|update| update.layer != LightLayerKind::Sky));
        assert!(engine.section.sky_light.is_definitely_filled_with(0));
    }

    #[test]
    fn block_changes_recompute_light_and_report_light_property_differences() {
        assert!(has_different_light_properties(
            LightBlock::AIR,
            LightBlock::light(15)
        ));
        assert!(has_different_light_properties(
            LightBlock::AIR,
            LightBlock::STONE
        ));
        assert!(!has_different_light_properties(
            LightBlock::AIR,
            LightBlock::AIR
        ));

        let mut engine = LightEngine::new(LightSection::air(), true);
        engine.changed_lights_for_block(2, 2, 2, LightBlock::light(12));
        assert_eq!(engine.section.block_light.get(2, 2, 2), 12);
        engine.changed_lights_for_block(2, 2, 2, LightBlock::AIR);
        assert_eq!(engine.section.block_light.get(2, 2, 2), 0);
    }

    #[test]
    fn section_light_arrays_are_storage_ready_nibble_arrays() {
        let mut section = LightSection::air();
        section.block_light.set(0, 0, 0, 1);
        section.sky_light.set(15, 15, 15, 15);

        let arrays = section_light_arrays(&section);
        assert_eq!(arrays["BlockLight"].len(), DATA_LAYER_SIZE);
        assert_eq!(arrays["SkyLight"].len(), DATA_LAYER_SIZE);
        assert_eq!(arrays["BlockLight"][0] as u8 & 0x0F, 1);
        assert_eq!(arrays["SkyLight"][2047] as u8 >> 4, 15);
    }
}
