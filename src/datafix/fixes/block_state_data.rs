//! Port of `net.minecraft.util.datafix.fixes.BlockStateData`: the table that maps
//! pre-flattening block ids and states (`id << 4 | data`) to 1.13 block states.
//!
//! The ~1700 `register(..)` calls of the Java class are too large to transcribe
//! by hand, so the resulting tables (`MAP`, `ID_BY_OLD`, `ID_BY_OLD_NAME` after
//! `finalizeMaps`) were dumped from the real class and are stored in
//! `data/block_state_data.tsv`:
//!
//! ```text
//! M <id> <group> <name|-> <k=v,k=v>  MAP[id] (`-` = null); `group` numbers the
//!                                    distinct `Dynamic` instances of MAP
//! O <name> <k=v,k=v> <id>          ID_BY_OLD (legacy state -> id)
//! N <name> <id>                    ID_BY_OLD_NAME (legacy name -> first id)
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::storage::nbt::Tag;

const DATA: &str = include_str!("../data/block_state_data.tsv");

/// A block state as `(Name, sorted properties)`.
type State = (String, Vec<(String, String)>);

struct Tables {
    /// `MAP` with the identity group of each entry.
    map: Vec<Option<(u32, State)>>,
    /// The state of each identity group.
    groups: Vec<State>,
    /// `MAP[0]`, the fallback of `getTag`.
    air: (u32, State),
    by_old: HashMap<State, i32>,
    by_old_name: HashMap<String, i32>,
}

fn parse_properties(text: &str) -> Vec<(String, String)> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = Tables {
            map: vec![None; 4096],
            groups: Vec::new(),
            air: (0, ("minecraft:air".to_string(), Vec::new())),
            by_old: HashMap::new(),
            by_old_name: HashMap::new(),
        };
        for line in DATA.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            match parts.as_slice() {
                ["M", id, group, name, props] if *name != "-" => {
                    let (Ok(id), Ok(group)) = (id.parse::<usize>(), group.parse::<u32>()) else {
                        continue;
                    };
                    let state = (name.to_string(), parse_properties(props));
                    if tables.groups.len() <= group as usize {
                        tables
                            .groups
                            .resize(group as usize + 1, (String::new(), Vec::new()));
                        tables.groups[group as usize] = state.clone();
                    }
                    tables.map[id] = Some((group, state));
                }
                ["O", name, props, id] => {
                    if let Ok(id) = id.parse::<i32>() {
                        tables
                            .by_old
                            .insert((name.to_string(), parse_properties(props)), id);
                    }
                }
                ["N", name, id] => {
                    if let Ok(id) = id.parse::<i32>() {
                        tables.by_old_name.insert(name.to_string(), id);
                    }
                }
                _ => {}
            }
        }
        if let Some(Some(first)) = tables.map.first() {
            tables.air = first.clone();
        }
        tables
    })
}

/// Builds the `{Name, Properties}` compound `BlockStateData.create` produces.
fn state_tag(state: &State) -> Tag {
    let mut entries = vec![("Name".to_string(), Tag::String(state.0.clone()))];
    if !state.1.is_empty() {
        entries.push((
            "Properties".to_string(),
            Tag::Compound(
                state
                    .1
                    .iter()
                    .map(|(key, value)| (key.clone(), Tag::String(value.clone())))
                    .collect(),
            ),
        ));
    }
    Tag::Compound(entries)
}

/// Reads a tag as a table key; anything but exactly `Name` (+ string
/// `Properties`) can never equal a table entry.
fn state_of(tag: &Tag) -> Option<State> {
    let Tag::Compound(entries) = tag else {
        return None;
    };
    let mut name = None;
    let mut properties = Vec::new();
    for (key, value) in entries {
        match (key.as_str(), value) {
            ("Name", Tag::String(text)) => name = Some(text.clone()),
            ("Properties", Tag::Compound(props)) => {
                for (prop, prop_value) in props {
                    let Tag::String(text) = prop_value else {
                        return None;
                    };
                    properties.push((prop.clone(), text.clone()));
                }
            }
            _ => return None,
        }
    }
    properties.sort();
    Some((name?, properties))
}

/// `BlockStateData.upgradeBlockStateTag`.
pub fn upgrade_block_state_tag(old_tag: &Tag) -> Tag {
    let tables = tables();
    let Some(id) = state_of(old_tag).and_then(|state| tables.by_old.get(&state).copied()) else {
        return old_tag.clone();
    };
    match usize::try_from(id).ok().and_then(|id| tables.map.get(id)) {
        Some(Some((_, state))) => state_tag(state),
        _ => old_tag.clone(),
    }
}

/// `BlockStateData.upgradeBlock(String)`.
pub fn upgrade_block_name(old_name: &str) -> String {
    let tables = tables();
    let Some(id) = tables.by_old_name.get(old_name) else {
        return old_name.to_string();
    };
    match usize::try_from(*id).ok().and_then(|id| tables.map.get(id)) {
        Some(Some((_, state))) => state.0.clone(),
        _ => old_name.to_string(),
    }
}

/// `BlockStateData.upgradeBlock(int)`.
pub fn upgrade_block_id(id: i32) -> String {
    match usize::try_from(id).ok().and_then(|id| tables().map.get(id)) {
        Some(Some((_, state))) => state.0.clone(),
        _ => "minecraft:air".to_string(),
    }
}

/// `BlockStateData.getTag(int)` as an identity-carrying state: out-of-range and
/// unmapped ids give `MAP[0]`. The returned number identifies the Java `Dynamic`
/// instance (ids sharing the block default share it), which
/// `ChunkPalettedStorageFix` relies on because its palette is identity based.
pub fn get_state(id: i32) -> (u32, &'static State) {
    let tables = tables();
    let entry = usize::try_from(id)
        .ok()
        .and_then(|id| tables.map.get(id))
        .and_then(Option::as_ref)
        .unwrap_or(&tables.air);
    (entry.0, &entry.1)
}

/// The number of distinct `MAP` instances (identity groups).
pub fn map_group_count() -> u32 {
    tables().groups.len() as u32
}

/// The state of an identity group returned by [`get_state`].
pub fn group_state(group: u32) -> &'static State {
    &tables().groups[group as usize]
}

/// `BlockStateData.getTag(int)`.
pub fn get_tag(id: i32) -> Tag {
    state_tag(get_state(id).1)
}

/// A block state as `(Name, sorted properties)`.
pub type BlockState = State;

/// The `{Name, Properties}` compound of a block state.
pub fn block_state_to_tag(state: &State) -> Tag {
    state_tag(state)
}
