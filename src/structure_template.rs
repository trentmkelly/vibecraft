//! Port of Java `StructureTemplate.load` / `StructureTemplate.save`
//! (`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate`).
//!
//! A template is a list of palettes (each a block list sharing positions but not
//! states), a list of entities and a size. Block states are global block-state
//! network ids from [`crate::block_states`], resolved like `NbtUtils.readBlockState`.
//! Block-entity NBT (jigsaw, structure blocks, chests, ...) and entity NBT are
//! preserved verbatim.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::block_properties::{collision_shape_is_full_cube, state_physics};
use crate::block_states::{
    block_state_entry, block_state_entry_for_network_id, block_state_name_for_network_id,
    network_id_for_block_state,
};
use crate::storage::nbt::Tag;
use crate::world_version::CURRENT_DATA_VERSION;

/// A compound tag body (Java `CompoundTag`, insertion ordered).
pub type Compound = Vec<(String, Tag)>;

/// Java `StructureBlockInfo`: a block position, its state and optional block entity NBT.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureBlockInfo {
    pub pos: [i32; 3],
    /// Global block-state network id.
    pub state: i32,
    pub nbt: Option<Compound>,
}

/// Java `StructureEntityInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureEntityInfo {
    pub pos: [f64; 3],
    pub block_pos: [i32; 3],
    pub nbt: Compound,
}

/// Java `StructureTemplate` (palettes, entities, size).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructureTemplate {
    pub size: [i32; 3],
    /// Java `palettes`: each entry is `Palette.blocks()` in load order.
    pub palettes: Vec<Vec<StructureBlockInfo>>,
    pub entities: Vec<StructureEntityInfo>,
    /// `palette` entries that `NbtUtils.readBlockState` could not resolve exactly
    /// (unknown block, unknown property or invalid value). Java silently substitutes
    /// air / the default value; the model records them so callers can reject the data.
    pub unresolved_states: Vec<String>,
}

/// Java `StructureTemplate.SimplePalette` over an `IdMapper<BlockState>`.
#[derive(Default)]
struct SimplePalette {
    state_to_id: HashMap<i32, usize>,
    id_to_state: Vec<Option<i32>>,
    last_id: usize,
}

impl SimplePalette {
    /// `SimplePalette.idFor`.
    fn id_for(&mut self, state: i32) -> usize {
        if let Some(id) = self.state_to_id.get(&state) {
            return *id;
        }
        let id = self.last_id;
        self.last_id += 1;
        self.add_mapping(state, id);
        id
    }

    /// `SimplePalette.stateFor`: unmapped ids read as air.
    fn state_for(&self, id: i32) -> i32 {
        usize::try_from(id)
            .ok()
            .and_then(|id| self.id_to_state.get(id).copied().flatten())
            .unwrap_or_else(air_state)
    }

    /// `IdMapper.addMapping`.
    fn add_mapping(&mut self, state: i32, id: usize) {
        self.state_to_id.insert(state, id);
        if self.id_to_state.len() <= id {
            self.id_to_state.resize(id + 1, None);
        }
        self.id_to_state[id] = Some(state);
    }

    /// Iteration in id order, skipping unmapped ids.
    fn states(&self) -> impl Iterator<Item = i32> + '_ {
        self.id_to_state.iter().flatten().copied()
    }
}

fn air_state() -> i32 {
    // Air is the first vanilla block state (id 0).
    network_id_for_block_state("minecraft:air").unwrap_or(0)
}

fn field<'a>(compound: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, tag)| tag)
}

/// `CompoundTag.getListOrEmpty`.
fn list<'a>(compound: &'a [(String, Tag)], name: &str) -> &'a [Tag] {
    match field(compound, name) {
        Some(Tag::List(values)) => values,
        _ => &[],
    }
}

/// `CompoundTag.getCompound`.
fn compound_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Option<&'a [(String, Tag)]> {
    match field(compound, name) {
        Some(Tag::Compound(values)) => Some(values),
        _ => None,
    }
}

/// `ListTag.getIntOr(index, 0)` (numeric tags coerce like `NumericTag.intValue`).
fn int_at(values: &[Tag], index: usize) -> i32 {
    match values.get(index) {
        Some(Tag::Byte(v)) => i32::from(*v),
        Some(Tag::Short(v)) => i32::from(*v),
        Some(Tag::Int(v)) => *v,
        Some(Tag::Long(v)) => *v as i32,
        Some(Tag::Float(v)) => *v as i32,
        Some(Tag::Double(v)) => *v as i32,
        _ => 0,
    }
}

/// `ListTag.getDoubleOr(index, 0.0)`.
fn double_at(values: &[Tag], index: usize) -> f64 {
    match values.get(index) {
        Some(Tag::Byte(v)) => f64::from(*v),
        Some(Tag::Short(v)) => f64::from(*v),
        Some(Tag::Int(v)) => f64::from(*v),
        Some(Tag::Long(v)) => *v as f64,
        Some(Tag::Float(v)) => f64::from(*v),
        Some(Tag::Double(v)) => *v,
        _ => 0.0,
    }
}

/// `NbtUtils.readBlockState`; the flag is false when anything failed to resolve.
fn read_block_state(tag: &[(String, Tag)]) -> (i32, bool) {
    let Some(Tag::String(name)) = field(tag, "Name") else {
        return (air_state(), false);
    };
    let Some(entry) = block_state_entry(name) else {
        return (air_state(), false);
    };
    let mut exact = true;
    let mut assignments = Vec::new();
    for (key, value) in compound_field(tag, "Properties").unwrap_or(&[]) {
        let known = entry.properties.iter().find(|p| p.name == key);
        match (known, value) {
            (Some(property), Tag::String(value)) if property.values.contains(&value.as_str()) => {
                assignments.push(format!("{key}={value}"));
            }
            _ => exact = false,
        }
    }
    let spec = if assignments.is_empty() {
        entry.registry_id.to_string()
    } else {
        format!("{}[{}]", entry.registry_id, assignments.join(","))
    };
    network_id_for_block_state(&spec).map_or((air_state(), false), |id| (id, exact))
}

/// `NbtUtils.writeBlockState`: `Name` plus every property (definition order) when present.
/// Ids outside the state table cannot come from a loaded palette and write as air.
fn write_block_state(state: i32) -> Tag {
    let spec = block_state_name_for_network_id(state).unwrap_or_else(|| "minecraft:air".into());
    let (name, rendered) = spec
        .split_once('[')
        .map_or((spec.as_str(), ""), |(name, rest)| {
            (name, rest.trim_end_matches(']'))
        });
    let mut compound = vec![("Name".to_string(), Tag::String(name.to_string()))];
    if !rendered.is_empty() {
        let properties = rendered
            .split(',')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.to_string(), Tag::String(v.to_string())))
            .collect();
        compound.push(("Properties".to_string(), Tag::Compound(properties)));
    }
    Tag::Compound(compound)
}

/// Java `Block.hasDynamicShape()`: `dynamicShape()` in `Blocks.java` is set on moving
/// piston, bamboo, scaffolding, powder snow, pointed dripstone and shulker boxes.
fn has_dynamic_shape(state: i32) -> bool {
    block_state_entry_for_network_id(state).is_some_and(|entry| {
        matches!(
            entry.registry_id,
            "minecraft:moving_piston"
                | "minecraft:bamboo"
                | "minecraft:scaffolding"
                | "minecraft:powder_snow"
                | "minecraft:pointed_dripstone"
        ) || entry.block_type == "shulker_box"
    })
}

/// `StructureTemplate.addToLists` + `buildInfoList`: full blocks, then other
/// blocks, then block entities, each sorted by y, x, z.
fn build_info_list(blocks: Vec<StructureBlockInfo>) -> Vec<StructureBlockInfo> {
    let (mut full, mut other, mut entities) = (Vec::new(), Vec::new(), Vec::new());
    for info in blocks {
        if info.nbt.is_some() {
            entities.push(info);
        } else if !has_dynamic_shape(info.state)
            && state_physics(info.state).is_some_and(collision_shape_is_full_cube)
        {
            full.push(info);
        } else {
            other.push(info);
        }
    }
    let key = |info: &StructureBlockInfo| (info.pos[1], info.pos[0], info.pos[2]);
    full.sort_by_key(key);
    other.sort_by_key(key);
    entities.sort_by_key(key);
    full.into_iter().chain(other).chain(entities).collect()
}

fn triple(values: &[Tag]) -> [i32; 3] {
    [int_at(values, 0), int_at(values, 1), int_at(values, 2)]
}

fn int_list(values: [i32; 3]) -> Tag {
    Tag::List(values.iter().map(|v| Tag::Int(*v)).collect())
}

fn as_compound(tag: &Tag) -> &[(String, Tag)] {
    match tag {
        Tag::Compound(values) => values,
        _ => &[],
    }
}

impl StructureTemplate {
    /// `StructureTemplate.load(HolderGetter<Block>, CompoundTag)`.
    pub fn load(tag: &[(String, Tag)]) -> Self {
        let mut template = Self {
            size: triple(list(tag, "size")),
            ..Self::default()
        };
        let blocks = list(tag, "blocks");
        match field(tag, "palettes") {
            Some(Tag::List(palettes)) => {
                for palette in palettes {
                    let states = if let Tag::List(values) = palette {
                        values
                    } else {
                        &[][..]
                    };
                    template.load_palette(states, blocks);
                }
            }
            _ => template.load_palette(list(tag, "palette"), blocks),
        }
        for entity in list(tag, "entities") {
            let Tag::Compound(entity) = entity else {
                continue;
            };
            let pos = list(entity, "pos");
            if let Some(nbt) = compound_field(entity, "nbt") {
                template.entities.push(StructureEntityInfo {
                    pos: [double_at(pos, 0), double_at(pos, 1), double_at(pos, 2)],
                    block_pos: triple(list(entity, "blockPos")),
                    nbt: nbt.to_vec(),
                });
            }
        }
        template
    }

    /// `StructureTemplate.loadPalette`.
    fn load_palette(&mut self, palette_list: &[Tag], block_list: &[Tag]) {
        let mut palette = SimplePalette::default();
        for (index, state) in palette_list.iter().enumerate() {
            let (id, exact) = read_block_state(as_compound(state));
            if !exact {
                self.unresolved_states.push(format!("{state:?}"));
            }
            palette.add_mapping(id, index);
        }
        let blocks = block_list
            .iter()
            .filter_map(|block| match block {
                Tag::Compound(block) => Some(block),
                _ => None,
            })
            .map(|block| StructureBlockInfo {
                pos: triple(list(block, "pos")),
                state: palette.state_for(match field(block, "state") {
                    Some(Tag::Int(v)) => *v,
                    _ => 0,
                }),
                nbt: compound_field(block, "nbt").map(<[_]>::to_vec),
            })
            .collect();
        self.palettes.push(build_info_list(blocks));
    }

    /// `StructureTemplate.save`: Java's `tag.put` order, then the current data version.
    pub fn save(&self) -> Compound {
        let mut tag = Vec::new();
        if self.palettes.is_empty() {
            tag.push(("blocks".to_string(), Tag::List(Vec::new())));
            tag.push(("palette".to_string(), Tag::List(Vec::new())));
        } else {
            self.save_blocks_and_palettes(&mut tag);
        }
        let entities = self.entities.iter().map(save_entity).collect();
        tag.push(("entities".to_string(), Tag::List(entities)));
        tag.push(("size".to_string(), int_list(self.size)));
        tag.push(("DataVersion".to_string(), Tag::Int(CURRENT_DATA_VERSION)));
        tag
    }

    /// The `blocks` / `palette` / `palettes` part of `StructureTemplate.save`.
    fn save_blocks_and_palettes(&self, tag: &mut Compound) {
        let mut palettes: Vec<SimplePalette> = self
            .palettes
            .iter()
            .map(|_| SimplePalette::default())
            .collect();
        let mut block_list = Vec::new();
        for (i, info) in self.palettes[0].iter().enumerate() {
            let id = palettes[0].id_for(info.state);
            let mut block = vec![
                ("pos".to_string(), int_list(info.pos)),
                ("state".to_string(), Tag::Int(id as i32)),
            ];
            if let Some(nbt) = &info.nbt {
                block.push(("nbt".to_string(), Tag::Compound(nbt.clone())));
            }
            block_list.push(Tag::Compound(block));
            for (p, palette) in palettes.iter_mut().enumerate().skip(1) {
                palette.add_mapping(self.palettes[p][i].state, id);
            }
        }
        tag.push(("blocks".to_string(), Tag::List(block_list)));
        let to_list =
            |palette: &SimplePalette| Tag::List(palette.states().map(write_block_state).collect());
        if palettes.len() == 1 {
            tag.push(("palette".to_string(), to_list(&palettes[0])));
        } else {
            let lists = palettes.iter().map(to_list).collect();
            tag.push(("palettes".to_string(), Tag::List(lists)));
        }
    }
}

fn save_entity(entity: &StructureEntityInfo) -> Tag {
    Tag::Compound(vec![
        (
            "pos".to_string(),
            Tag::List(entity.pos.iter().map(|v| Tag::Double(*v)).collect()),
        ),
        ("blockPos".to_string(), int_list(entity.block_pos)),
        ("nbt".to_string(), Tag::Compound(entity.nbt.clone())),
    ])
}

#[cfg(test)]
#[path = "structure_template_tests.rs"]
mod tests;
