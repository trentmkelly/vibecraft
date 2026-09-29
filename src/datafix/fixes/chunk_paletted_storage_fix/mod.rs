//! Port of `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix`: legacy
//! chunk sections (`Blocks` / `Data` / `Add`) become palettised `BlockStates`
//! with `UpgradeData` for the blocks that need neighbour-dependent fix-ups.
//!
//! The Java palette is *identity* based (`CrudeIncrementalIntIdentityHashBiMap`,
//! `Sets.newIdentityHashSet`), so blocks are handled here as identity numbers
//! ([`StateId`]): entries of `BlockStateData.MAP` share an identity when the Java
//! code shares the instance, and every `MappingConstants` entry has its own.
//! This is what makes equal-looking palette entries appear more than once in
//! the result, exactly as in Java.

mod mapping_constants;
mod section;

use crate::datafix::dynamic::{get, get_bool_or, get_i32_or, get_str_or, list_items, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::write_and_read_fix::write_fix_and_read_fix;
use mapping_constants::{MappingConstants, StateId};
use section::Section;

const SECTION_COUNT: usize = 16;

// `ChunkPalettedStorageFix` side masks.
const NORTH_WEST_MASK: i32 = 128;
const WEST_MASK: i32 = 64;
const SOUTH_WEST_MASK: i32 = 32;
const SOUTH_MASK: i32 = 16;
const SOUTH_EAST_MASK: i32 = 8;
const EAST_MASK: i32 = 4;
const NORTH_EAST_MASK: i32 = 2;
const NORTH_MASK: i32 = 1;

/// `ChunkPalettedStorageFix.getSideMask`.
pub fn get_side_mask(west: bool, east: bool, north: bool, south: bool) -> i32 {
    if north {
        if east {
            NORTH_EAST_MASK
        } else if west {
            NORTH_WEST_MASK
        } else {
            NORTH_MASK
        }
    } else if south {
        if west {
            SOUTH_WEST_MASK
        } else if east {
            SOUTH_EAST_MASK
        } else {
            SOUTH_MASK
        }
    } else if east {
        EAST_MASK
    } else if west {
        WEST_MASK
    } else {
        0
    }
}

/// `new ChunkPalettedStorageFix(schema, changesType)`.
pub fn fix() -> Fix {
    write_fix_and_read_fix("ChunkPalettedStorageFix", r::CHUNK, |chunk| {
        let Some(level) = get(chunk, "Level") else {
            return;
        };
        if get(level, "Sections").and_then(list_items).is_none() {
            return;
        }
        if let Some(upgraded) = UpgradeChunk::new(level).and_then(UpgradeChunk::write) {
            set(chunk, "Level", upgraded);
        }
    })
}

/// A direction of `ChunkPalettedStorageFix.Direction`, reduced to what
/// [`relative`] needs.
#[derive(Clone, Copy)]
enum Direction {
    Down,
    Up,
}

/// `UpgradeChunk.relative`: the block position next to `pos`, or -1 outside the chunk.
fn relative(pos: i32, direction: Direction) -> i32 {
    let step = match direction {
        Direction::Up => 1,
        Direction::Down => -1,
    };
    let y = (pos >> 8) + step;
    if (0..=255).contains(&y) {
        (pos & 0xFF) | (y << 8)
    } else {
        -1
    }
}

/// `UpgradeChunk`: one chunk being converted.
struct UpgradeChunk {
    sides: i32,
    sections: Vec<Option<Section>>,
    level: Tag,
    /// Block entities by packed position, in insertion order (`Int2ObjectLinkedOpenHashMap`).
    block_entities: Vec<(i32, Tag)>,
    convert_from_alpha: bool,
    constants: &'static MappingConstants,
}

impl UpgradeChunk {
    /// The `UpgradeChunk` constructor. `None` where the Java code would throw.
    fn new(level: &Tag) -> Option<Self> {
        let constants = MappingConstants::get();
        let x = get_i32_or(level, "xPos", 0) << 4;
        let z = get_i32_or(level, "zPos", 0) << 4;
        let mut block_entities: Vec<(i32, Tag)> = Vec::new();
        if let Some(entities) = get(level, "TileEntities").and_then(list_items) {
            for entity in entities {
                let ex = (get_i32_or(&entity, "x", 0).wrapping_sub(x)) & 15;
                let ey = get_i32_or(&entity, "y", 0);
                let ez = (get_i32_or(&entity, "z", 0).wrapping_sub(z)) & 15;
                let key = (ey << 8) | (ez << 4) | ex;
                match block_entities
                    .iter_mut()
                    .find(|(existing, _)| *existing == key)
                {
                    Some((_, slot)) => *slot = entity,
                    None => block_entities.push((key, entity)),
                }
            }
        }
        let convert_from_alpha = get_bool_or(level, "convertedFromAlphaFormat", false);
        let mut chunk = Self {
            sides: 0,
            sections: (0..SECTION_COUNT).map(|_| None).collect(),
            level: level.clone(),
            block_entities,
            convert_from_alpha,
            constants,
        };
        for section_tag in get(level, "Sections").and_then(list_items)? {
            let mut section = Section::new(&section_tag);
            chunk.sides = section.upgrade(chunk.sides)?;
            let y = usize::try_from(section.y)
                .ok()
                .filter(|y| *y < SECTION_COUNT)?;
            chunk.sections[y] = Some(section);
        }
        chunk.apply_fixes();
        Some(chunk)
    }

    fn get_section(&self, pos: i32) -> Option<&Section> {
        let section_y = usize::try_from(pos >> 12).ok()?;
        self.sections.get(section_y)?.as_ref()
    }

    /// `UpgradeChunk.getBlock`.
    fn get_block(&self, pos: i32) -> StateId {
        if (0..=65535).contains(&pos) {
            match self.get_section(pos) {
                Some(section) => section.get_block(pos & 4095, self.constants),
                None => self.constants.air,
            }
        } else {
            self.constants.air
        }
    }

    /// `UpgradeChunk.setBlock`.
    fn set_block(&mut self, pos: i32, block: StateId) {
        if !(0..=65535).contains(&pos) {
            return;
        }
        let constants = self.constants;
        if let Some(Some(section)) = usize::try_from(pos >> 12)
            .ok()
            .and_then(|section_y| self.sections.get_mut(section_y))
        {
            section.set_block(pos & 4095, block, constants);
        }
    }

    fn name_of(&self, state: StateId) -> String {
        section::state_name(state, self.constants)
    }

    fn property_of(&self, state: StateId, property: &str) -> String {
        section::state_property(state, property, self.constants)
    }

    fn get_block_entity(&self, pos: i32) -> Option<&Tag> {
        self.block_entities
            .iter()
            .find(|(key, _)| *key == pos)
            .map(|(_, entity)| entity)
    }

    fn remove_block_entity(&mut self, pos: i32) -> Option<Tag> {
        let index = self
            .block_entities
            .iter()
            .position(|(key, _)| *key == pos)?;
        Some(self.block_entities.remove(index).1)
    }

    /// The fix-up pass of the constructor over every section's `toFix` lists.
    fn apply_fixes(&mut self) {
        for section_index in 0..SECTION_COUNT {
            let Some(section) = &self.sections[section_index] else {
                continue;
            };
            let dy = section.y << 12;
            let to_fix = section.to_fix().to_vec();
            for (block_id, positions) in to_fix {
                for position in positions {
                    self.fix_block(block_id, position | dy);
                }
            }
        }
    }

    fn fix_block(&mut self, block_id: usize, pos: i32) {
        match block_id {
            2 => self.fix_snowy(pos, "minecraft:grass_block", self.constants.snowy_grass),
            3 => self.fix_snowy(pos, "minecraft:podzol", self.constants.snowy_podzol),
            110 => self.fix_snowy(pos, "minecraft:mycelium", self.constants.snowy_mycelium),
            25 => self.fix_note_block(pos),
            26 => self.fix_bed(pos),
            64 | 71 | 193..=197 => self.fix_door(pos),
            86 => self.fix_pumpkin(pos),
            140 => self.fix_flower_pot(pos),
            144 => self.fix_skull(pos),
            175 => self.fix_double_plant(pos),
            176 | 177 => self.fix_banner(pos, block_id),
            _ => {}
        }
    }

    fn fix_snowy(&mut self, pos: i32, block_name: &str, snowy: StateId) {
        let state = self.get_block(pos);
        if self.name_of(state) == block_name {
            let above = self.get_block(relative(pos, Direction::Up));
            let name = self.name_of(above);
            if name == "minecraft:snow" || name == "minecraft:snow_layer" {
                self.set_block(pos, snowy);
            }
        }
    }

    fn fix_note_block(&mut self, pos: i32) {
        let Some(entity) = self.remove_block_entity(pos) else {
            return;
        };
        let note = get_i32_or(&entity, "note", 0).clamp(0, 24);
        let key = format!("{}{}", get_bool_or(&entity, "powered", false), note as i8);
        let constants = self.constants;
        let state = constants
            .note_block
            .get(&key)
            .or_else(|| constants.note_block.get("false0"))
            .copied();
        if let Some(state) = state {
            self.set_block(pos, state);
        }
    }

    fn fix_bed(&mut self, pos: i32) {
        let entity_color = self
            .get_block_entity(pos)
            .map(|entity| get_i32_or(entity, "color", 0));
        let state = self.get_block(pos);
        let Some(color) = entity_color else {
            return;
        };
        if color != 14 && (0..16).contains(&color) {
            let key = format!(
                "{}{}{}{}",
                self.property_of(state, "facing"),
                self.property_of(state, "occupied"),
                self.property_of(state, "part"),
                color
            );
            if let Some(bed) = self.constants.bed.get(&key).copied() {
                self.set_block(pos, bed);
            }
        }
    }

    fn fix_door(&mut self, pos: i32) {
        let state = self.get_block(pos);
        if !self.name_of(state).ends_with("_door") {
            return;
        }
        let lower = self.get_block(pos);
        if self.property_of(lower, "half") != "lower" {
            return;
        }
        let above_pos = relative(pos, Direction::Up);
        let upper = self.get_block(above_pos);
        let name = self.name_of(lower);
        if name != self.name_of(upper) {
            return;
        }
        let facing = self.property_of(lower, "facing");
        let open = self.property_of(lower, "open");
        let (hinge, powered) = if self.convert_from_alpha {
            ("left".to_string(), "false".to_string())
        } else {
            (
                self.property_of(upper, "hinge"),
                self.property_of(upper, "powered"),
            )
        };
        let constants = self.constants;
        let lower_key = format!("{name}{facing}lower{hinge}{open}{powered}");
        let upper_key = format!("{name}{facing}upper{hinge}{open}{powered}");
        // Java would throw on a missing key; such doors are left alone.
        if let (Some(lower), Some(upper)) = (
            constants.door.get(&lower_key).copied(),
            constants.door.get(&upper_key).copied(),
        ) {
            self.set_block(pos, lower);
            self.set_block(above_pos, upper);
        }
    }

    fn fix_pumpkin(&mut self, pos: i32) {
        let state = self.get_block(pos);
        if self.name_of(state) == "minecraft:carved_pumpkin" {
            let below = self.get_block(relative(pos, Direction::Down));
            let name = self.name_of(below);
            if name == "minecraft:grass_block" || name == "minecraft:dirt" {
                self.set_block(pos, self.constants.pumpkin);
            }
        }
    }

    fn fix_flower_pot(&mut self, pos: i32) {
        let Some(entity) = self.remove_block_entity(pos) else {
            return;
        };
        let key = format!(
            "{}{}",
            get_str_or(&entity, "Item", ""),
            get_i32_or(&entity, "Data", 0)
        );
        let constants = self.constants;
        let state = constants
            .flower_pot
            .get(&key)
            .or_else(|| constants.flower_pot.get("minecraft:air0"))
            .copied();
        if let Some(state) = state {
            self.set_block(pos, state);
        }
    }

    fn fix_skull(&mut self, pos: i32) {
        let Some(entity) = self.get_block_entity(pos) else {
            return;
        };
        let skull_type = get_i32_or(entity, "SkullType", 0).to_string();
        let rotation = get_i32_or(entity, "Rot", 0);
        let state = self.get_block(pos);
        let facing = self.property_of(state, "facing");
        let key = if facing != "up" && facing != "down" {
            format!("{skull_type}{facing}")
        } else {
            format!("{skull_type}{rotation}")
        };
        // `entity.remove(..)` results are discarded in Java: the block entity keeps its keys.
        let constants = self.constants;
        let state = constants
            .skull
            .get(&key)
            .or_else(|| constants.skull.get("0north"))
            .copied();
        if let Some(state) = state {
            self.set_block(pos, state);
        }
    }

    fn fix_double_plant(&mut self, pos: i32) {
        let block = self.get_block(pos);
        if self.property_of(block, "half") != "upper" {
            return;
        }
        let below = self.get_block(relative(pos, Direction::Down));
        let constants = self.constants;
        let replacement = match self.name_of(below).as_str() {
            "minecraft:sunflower" => constants.upper_sunflower,
            "minecraft:lilac" => constants.upper_lilac,
            "minecraft:tall_grass" => constants.upper_tall_grass,
            "minecraft:large_fern" => constants.upper_large_fern,
            "minecraft:rose_bush" => constants.upper_rose_bush,
            "minecraft:peony" => constants.upper_peony,
            _ => return,
        };
        self.set_block(pos, replacement);
    }

    fn fix_banner(&mut self, pos: i32, block_id: usize) {
        let Some(color) = self
            .get_block_entity(pos)
            .map(|entity| get_i32_or(entity, "Base", 0))
        else {
            return;
        };
        let state = self.get_block(pos);
        if color != 15 && (0..16).contains(&color) {
            let property = if block_id == 176 {
                "rotation"
            } else {
                "facing"
            };
            let key = format!("{}_{color}", self.property_of(state, property));
            if let Some(banner) = self.constants.banner.get(&key).copied() {
                self.set_block(pos, banner);
            }
        }
    }

    /// `UpgradeChunk.write`. `None` where the Java code would throw.
    fn write(mut self) -> Option<Tag> {
        let mut level = std::mem::replace(&mut self.level, Tag::End);
        if self.block_entities.is_empty() {
            remove(&mut level, "TileEntities");
        } else {
            let entities = self.block_entities.iter().map(|(_, e)| e.clone()).collect();
            set(&mut level, "TileEntities", Tag::List(entities));
        }
        let mut indices = Vec::new();
        let mut sections = Vec::new();
        for section in self.sections.iter().flatten() {
            sections.push(section.write(self.constants)?);
            indices.push((
                section.y.to_string(),
                Tag::IntArray(section.update().to_vec()),
            ));
        }
        let upgrade_data = Tag::Compound(vec![
            ("Sides".to_string(), Tag::Byte(self.sides as i8)),
            ("Indices".to_string(), Tag::Compound(indices)),
        ]);
        set(&mut level, "UpgradeData", upgrade_data);
        set(&mut level, "Sections", Tag::List(sections));
        Some(level)
    }
}
