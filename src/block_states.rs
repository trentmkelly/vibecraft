//! Full vanilla 26.1.2 block-state table: every block's state definition and the
//! global state-id space used by the protocol (chunk palettes, block updates).
//!
//! Java assigns block-state ids by walking `BuiltInRegistries.BLOCK` in registry
//! order and registering every state of each block's `StateDefinition` into
//! `Block.BLOCK_STATE_REGISTRY` (see `Block.java` static initializer). States of a
//! block are the cartesian product of its properties with the *last* property
//! varying fastest, so each block owns a contiguous id range starting at
//! `base_state_id`. The data tables here are generated from the official server's
//! data generator report (`reports/blocks.json`, vendored at
//! `vanilla-data/reports/blocks_26_1_2.json`) by `tools/generate_block_states.py`.
//!
//! Name parsing follows `NbtUtils.readBlockState` leniency: unknown properties and
//! invalid property values are ignored (Java logs and keeps the default), so chunk
//! palettes from foreign worlds degrade exactly like vanilla instead of failing.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::LazyLock;

mod state_data_a;
mod state_data_b;
mod state_data_c;
mod state_data_d;

/// Total number of block states registered by vanilla 26.1.2.
pub const VANILLA_BLOCK_STATE_COUNT_26_1_2: usize = 29873;

/// One property of a block's state definition, with values in vanilla order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateProperty {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

/// A block's complete state definition surface in the global state-id space.
///
/// `properties` is in cartesian order (the last property varies fastest), which
/// matches Java's `StateDefinition` state enumeration, so
/// `state_id = base_state_id + Σ value_index(i) * stride(i)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockStateEntryData {
    pub registry_id: &'static str,
    pub base_state_id: i32,
    pub default_state_id: i32,
    pub properties: &'static [StateProperty],
}

impl BlockStateEntryData {
    /// Number of states this block owns (product of property value counts).
    pub fn state_count(&self) -> i32 {
        self.properties
            .iter()
            .map(|property| property.values.len() as i32)
            .product::<i32>()
            .max(1)
    }

    /// The stride of `properties[index]` in the state-id encoding.
    fn stride(&self, index: usize) -> i32 {
        self.properties[index + 1..]
            .iter()
            .map(|property| property.values.len() as i32)
            .product()
    }

    /// Value indices of a state id owned by this block, in property order.
    fn decode(&self, state_id: i32) -> Vec<usize> {
        let mut offset = state_id - self.base_state_id;
        let mut indices = vec![0; self.properties.len()];
        for index in (0..self.properties.len()).rev() {
            let len = self.properties[index].values.len() as i32;
            indices[index] = (offset % len) as usize;
            offset /= len;
        }
        indices
    }
}

static ENTRIES: LazyLock<Vec<BlockStateEntryData>> = LazyLock::new(|| {
    state_data_b::ENTRIES
        .iter()
        .chain(state_data_c::ENTRIES.iter())
        .chain(state_data_d::ENTRIES.iter())
        .copied()
        .collect()
});

static BY_NAME: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    ENTRIES
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.registry_id, index))
        .collect()
});

/// All block-state entries in block-registry (protocol) order.
pub fn block_state_entries() -> &'static [BlockStateEntryData] {
    &ENTRIES
}

/// Looks up a block's state definition by registry id. A bare path without a
/// namespace resolves in `minecraft:` like Java `Identifier.parse`.
pub fn block_state_entry(registry_id: &str) -> Option<&'static BlockStateEntryData> {
    if registry_id.contains(':') {
        return BY_NAME.get(registry_id).map(|&index| &ENTRIES[index]);
    }
    let namespaced = format!("minecraft:{registry_id}");
    BY_NAME.get(namespaced.as_str()).map(|&index| &ENTRIES[index])
}

/// The default-state network id for a block, like Java `block.defaultBlockState()`.
pub fn default_state_network_id(registry_id: &str) -> Option<i32> {
    block_state_entry(registry_id).map(|entry| entry.default_state_id)
}

/// Resolves a `block[prop=value,...]` state string (or bare block name) to its
/// global network state id.
///
/// Mirrors `NbtUtils.readBlockState`: missing properties keep the default-state
/// value, and unknown properties or invalid values are ignored rather than
/// failing, so the result is always a valid state of the named block.
pub fn network_id_for_block_state(name: &str) -> Option<i32> {
    let (base, raw_properties) = match name.split_once('[') {
        Some((base, rest)) => (base, rest.trim_end_matches(']')),
        None => (name, ""),
    };
    let entry = block_state_entry(base)?;
    if raw_properties.is_empty() {
        return Some(entry.default_state_id);
    }

    let mut indices = entry.decode(entry.default_state_id);
    for pair in raw_properties.split(',') {
        let Some((property_name, value)) = pair.split_once('=') else {
            continue;
        };
        let property_name = property_name.trim();
        let value = value.trim();
        let Some(position) = entry
            .properties
            .iter()
            .position(|property| property.name == property_name)
        else {
            continue;
        };
        if let Some(value_index) = entry.properties[position]
            .values
            .iter()
            .position(|candidate| *candidate == value)
        {
            indices[position] = value_index;
        }
    }

    let mut state_id = entry.base_state_id;
    for (position, value_index) in indices.iter().enumerate() {
        state_id += *value_index as i32 * entry.stride(position);
    }
    Some(state_id)
}

/// The default-state property assignments of a block, in definition order.
/// Java equivalent: reading `block.defaultBlockState().getValues()`.
pub fn default_state_properties(
    registry_id: &str,
) -> Option<Vec<(&'static str, &'static str)>> {
    let entry = block_state_entry(registry_id)?;
    let indices = entry.decode(entry.default_state_id);
    Some(
        entry
            .properties
            .iter()
            .zip(indices)
            .map(|(property, value_index)| (property.name, property.values[value_index]))
            .collect(),
    )
}

/// The block-state entry owning a global network state id.
pub fn block_state_entry_for_network_id(state_id: i32) -> Option<&'static BlockStateEntryData> {
    if state_id < 0 {
        return None;
    }
    let index = ENTRIES
        .partition_point(|entry| entry.base_state_id <= state_id)
        .checked_sub(1)?;
    let entry = &ENTRIES[index];
    (state_id < entry.base_state_id + entry.state_count()).then_some(entry)
}

/// Canonical `block[prop=value,...]` string for a global network state id, with
/// properties in definition order; property-less blocks render as the bare id.
pub fn block_state_name_for_network_id(state_id: i32) -> Option<String> {
    let entry = block_state_entry_for_network_id(state_id)?;
    if entry.properties.is_empty() {
        return Some(entry.registry_id.to_string());
    }
    let indices = entry.decode(state_id);
    let rendered = entry
        .properties
        .iter()
        .zip(indices)
        .map(|(property, value_index)| format!("{}={}", property.name, property.values[value_index]))
        .collect::<Vec<_>>()
        .join(",");
    Some(format!("{}[{rendered}]", entry.registry_id))
}

#[cfg(test)]
mod tests;
