//! Port of `net.minecraft.util.datafix.fixes.LeavesFix`: leaves get their
//! `distance` / `persistent` block state properties, computed by a breadth-first
//! search from the logs, and chunk edges with leaves are recorded in
//! `UpgradeData.Sides`.
//!
//! Two Java details are observable and therefore reproduced: the sections are
//! visited in the iteration order of an `Int2ObjectOpenHashMap` built from a
//! `HashMap`, and the search frontier is an `IntOpenHashSet`; together they decide
//! the order in which new palette entries are appended.

pub mod section;

use std::collections::{HashMap, HashSet};

use crate::datafix::dynamic::{as_i8, compound, get, get_i32_or, get_mut, get_str_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::int_open_hash_set::IntOpenHashSet;
use crate::datafix::packed_bit_storage::PackedBitStorage;
use crate::datafix::references as r;
use crate::datafix::typed::{typed, typed_mut};
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::chunk_paletted_storage_fix::get_side_mask;
use section::{Section, SIZE};

const DIRECTIONS: [[i32; 3]; 6] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
];
/// `DECAY_DISTANCE`.
const DECAY_DISTANCE: i32 = 7;
/// `SIZE_BITS`.
const SIZE_BITS: i32 = 12;

/// `LEAVES`: leaf block name to its wood type index.
const LEAVES: [(&str, i32); 6] = [
    ("minecraft:acacia_leaves", 0),
    ("minecraft:birch_leaves", 1),
    ("minecraft:dark_oak_leaves", 2),
    ("minecraft:jungle_leaves", 3),
    ("minecraft:oak_leaves", 4),
    ("minecraft:spruce_leaves", 5),
];

/// `LOGS`.
const LOGS: [&str; 18] = [
    "minecraft:acacia_bark",
    "minecraft:birch_bark",
    "minecraft:dark_oak_bark",
    "minecraft:jungle_bark",
    "minecraft:oak_bark",
    "minecraft:spruce_bark",
    "minecraft:acacia_log",
    "minecraft:birch_log",
    "minecraft:dark_oak_log",
    "minecraft:jungle_log",
    "minecraft:oak_log",
    "minecraft:spruce_log",
    "minecraft:stripped_acacia_log",
    "minecraft:stripped_birch_log",
    "minecraft:stripped_dark_oak_log",
    "minecraft:stripped_jungle_log",
    "minecraft:stripped_oak_log",
    "minecraft:stripped_spruce_log",
];

fn leaf_type(block_name: &str) -> Option<i32> {
    LEAVES
        .iter()
        .find(|(name, _)| *name == block_name)
        .map(|(_, wood)| *wood)
}

/// `LeavesFix.getIndex`.
pub fn get_index(x: i32, y: i32, z: i32) -> i32 {
    y << 8 | z << 4 | x
}

fn get_x(index: i32) -> i32 {
    index & 15
}

fn get_y(index: i32) -> i32 {
    index >> 8 & 0xFF
}

fn get_z(index: i32) -> i32 {
    index >> 4 & 15
}

/// `LeavesFix.LeavesSection`.
struct LeavesSection {
    section: Section,
    leaf_ids: HashSet<usize>,
    log_ids: HashSet<usize>,
    state_to_id: HashMap<i32, usize>,
}

/// `getStateId`.
fn state_id(block_name: &str, persistent: bool, distance: i32) -> i32 {
    leaf_type(block_name).unwrap_or(0) << 5 | if persistent { 16 } else { 0 } | distance
}

/// `makeLeafTag`.
fn make_leaf_tag(block_name: &str, persistent: bool, distance: i32) -> Tag {
    let properties = compound(vec![
        (
            "persistent",
            Tag::String(if persistent { "true" } else { "false" }.to_string()),
        ),
        ("distance", Tag::String(distance.to_string())),
    ]);
    compound(vec![
        ("Properties", properties),
        ("Name", Tag::String(block_name.to_string())),
    ])
}

/// `palette.get(block).get("Properties").get(name).asString("")`.
fn property<'a>(palette_tag: &'a Tag, name: &str) -> &'a str {
    get(palette_tag, "Properties").map_or("", |properties| get_str_or(properties, name, ""))
}

impl LeavesSection {
    /// `new LeavesSection(section, inputSchema)`; `None` where Java throws.
    fn read(tag: &Tag) -> Option<Self> {
        let mut leaf_ids = HashSet::new();
        let mut log_ids = HashSet::new();
        let mut state_to_id = HashMap::new();
        let section = Section::read(tag, |palette| {
            for (i, palette_tag) in palette.iter_mut().enumerate() {
                let block_name = get_str_or(palette_tag, "Name", "").to_string();
                if leaf_type(&block_name).is_some() {
                    let persistent = property(palette_tag, "decayable") == "false";
                    leaf_ids.insert(i);
                    state_to_id.insert(state_id(&block_name, persistent, DECAY_DISTANCE), i);
                    *palette_tag = make_leaf_tag(&block_name, persistent, DECAY_DISTANCE);
                }
                if LOGS.contains(&block_name.as_str()) {
                    log_ids.insert(i);
                }
            }
            leaf_ids.is_empty() && log_ids.is_empty()
        })?;
        Some(Self {
            section,
            leaf_ids,
            log_ids,
            state_to_id,
        })
    }

    fn is_log(&self, block: usize) -> bool {
        self.log_ids.contains(&block)
    }

    fn is_leaf(&self, block: usize) -> bool {
        self.leaf_ids.contains(&block)
    }

    /// `getDistance`.
    fn distance(&self, block: usize) -> i32 {
        if self.is_log(block) {
            return 0;
        }
        // Leaf palette entries always carry a numeric `distance`.
        property(&self.section.palette[block], "distance")
            .parse()
            .unwrap_or(0)
    }

    /// `setDistance`.
    fn set_distance(&mut self, pos: usize, block: usize, distance: i32) {
        let base = self.section.palette[block].clone();
        let block_name = get_str_or(&base, "Name", "").to_string();
        let persistent = property(&base, "persistent") == "true";
        let state = state_id(&block_name, persistent, distance);
        if !self.state_to_id.contains_key(&state) {
            let id = self.section.palette.len();
            self.leaf_ids.insert(id);
            self.state_to_id.insert(state, id);
            self.section
                .palette
                .push(make_leaf_tag(&block_name, persistent, distance));
        }
        let id = self.state_to_id[&state];
        let Some(storage) = self.section.storage.as_mut() else {
            return;
        };
        if 1_usize << storage.bits() <= id {
            let mut grown = PackedBitStorage::new(storage.bits() + 1, SIZE);
            for i in 0..SIZE {
                grown.set(i, storage.get(i));
            }
            *storage = grown;
        }
        storage.set(pos, id as i64);
    }
}

/// The iteration order of `new Int2ObjectOpenHashMap(hashMap)` for the section
/// `Y` values, where `hashMap` was filled in list order (`Collectors.toMap`).
fn section_iteration_order(ys: &[i32]) -> Vec<i32> {
    // `HashMap` iterates by bucket (stable within a bucket); the table doubles
    // whenever the size exceeds 0.75 of its capacity, starting at 16.
    let mut capacity = 16;
    while ys.len() * 4 > capacity * 3 {
        capacity *= 2;
    }
    let bucket = |y: i32| ((y ^ ((y as u32) >> 16) as i32) as u32 as usize) & (capacity - 1);
    let mut hash_order = ys.to_vec();
    hash_order.sort_by_key(|y| bucket(*y));
    let mut map = IntOpenHashSet::with_expected(ys.len());
    for y in hash_order {
        map.add(y);
    }
    map.iter().collect()
}

/// `new LeavesFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere("Leaves fix", Target::Type(r::CHUNK), |chunk| {
        let Some(level) = typed_mut(chunk, "Level") else {
            return;
        };
        let Some(sides) = fix_sections(level) else {
            return;
        };
        if sides != 0 {
            record_sides(level, sides);
        }
    })
}

/// `newLevel.update(remainderFinder, ...)`: ors `sides` into `UpgradeData.Sides`.
fn record_sides(level: &mut Tag, sides: i32) {
    if get(level, "UpgradeData").is_none() {
        set(level, "UpgradeData", Tag::Compound(Vec::new()));
    }
    if let Some(upgrade_data @ Tag::Compound(_)) = get_mut(level, "UpgradeData") {
        let existing = get(upgrade_data, "Sides").and_then(as_i8).unwrap_or(0);
        set(
            upgrade_data,
            "Sides",
            Tag::Byte((i32::from(existing) | sides) as i8),
        );
    }
}

/// The `updateTyped(sectionsFinder, ...)` step. Returns the accumulated side
/// mask, or `None` when nothing may be written (no sections, or the Java code
/// would throw).
fn fix_sections(level: &mut Tag) -> Option<i32> {
    let Some(Tag::List(section_tags)) = typed(level, "Sections") else {
        return Some(0);
    };
    let mut sections = HashMap::new();
    let mut ys = Vec::new();
    for tag in section_tags {
        let section = LeavesSection::read(tag)?;
        let y = section.section.index();
        // `Collectors.toMap` throws on a duplicate key.
        if sections.insert(y, section).is_some() {
            return None;
        }
        ys.push(y);
    }
    if sections
        .values()
        .all(|section| section.section.is_skippable())
    {
        return Some(0);
    }
    let order = section_iteration_order(&ys);
    let (mut frontier, sides) = seed_frontier(&sections, &order);
    for distance in 1..7 {
        let mut next = IntOpenHashSet::new();
        for pos in frontier.iter() {
            spread(&mut sections, pos, distance, &mut next);
        }
        frontier = next;
    }
    let Some(Tag::List(section_tags)) = typed_mut(level, "Sections") else {
        return None;
    };
    for tag in section_tags {
        if let Some(section) = sections.get(&get_i32_or(tag, "Y", 0)) {
            section.section.write(tag);
        }
    }
    Some(sides)
}

/// The first phase: logs seed the frontier, leaves on a chunk edge set side bits.
fn seed_frontier(sections: &HashMap<i32, LeavesSection>, order: &[i32]) -> (IntOpenHashSet, i32) {
    let mut logs = IntOpenHashSet::new();
    let mut sides = 0;
    for y in order {
        let section = &sections[y];
        if section.section.is_skippable() {
            continue;
        }
        for i in 0..SIZE {
            let block = section.section.block(i);
            if section.is_log(block) {
                logs.add(section.section.index() << SIZE_BITS | i as i32);
            } else if section.is_leaf(block) {
                let x = get_x(i as i32);
                let z = get_z(i as i32);
                sides |= get_side_mask(x == 0, x == 15, z == 0, z == 15);
            }
        }
    }
    (logs, sides)
}

/// One step of the search from the block at chunk position `pos`.
fn spread(
    sections: &mut HashMap<i32, LeavesSection>,
    pos: i32,
    distance: i32,
    next: &mut IntOpenHashSet,
) {
    let (x, y, z) = (get_x(pos), get_y(pos), get_z(pos));
    for direction in DIRECTIONS {
        let (nx, ny, nz) = (x + direction[0], y + direction[1], z + direction[2]);
        if !(0..=15).contains(&nx) || !(0..=15).contains(&nz) || !(0..=255).contains(&ny) {
            continue;
        }
        let Some(section) = sections.get_mut(&(ny >> 4)) else {
            continue;
        };
        if section.section.is_skippable() {
            continue;
        }
        let in_section = get_index(nx, ny & 15, nz) as usize;
        let block = section.section.block(in_section);
        if section.is_leaf(block) && section.distance(block) > distance {
            section.set_distance(in_section, block, distance);
            next.add(get_index(nx, ny, nz));
        }
    }
}
