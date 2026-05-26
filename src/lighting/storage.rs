//! Per-layer section storage shared by the block- and sky-light engines.
//!
//! Mirrors three Java classes:
//!
//! - `net.minecraft.world.level.lighting.DataLayerStorageMap` — packed map from
//!   section node → `DataLayer`. The Java implementation keeps a two-entry
//!   read cache for hot loops; in Rust the `HashMap` lookups are already cheap
//!   so we omit that microoptimisation. The visible/updating "swap on commit"
//!   pattern is preserved exactly because correctness depends on it.
//! - `net.minecraft.world.level.lighting.LayerLightSectionStorage` — owns the
//!   queued/changed/affected sets, `columnsWithSources`, `toRemove`, and the
//!   section-state byte map. All public hooks (`storingLightForSection`,
//!   `getStoredLevel`, `setStoredLevel`, `getDataLayerToWrite`, etc.) and the
//!   `SectionState` byte layout match Java bit-for-bit.
//! - `net.minecraft.world.level.lighting.BlockLightSectionStorage` and
//!   `SkyLightSectionStorage` — the subclasses, here folded into a single
//!   `LayerLightSectionStorage` driven by [`LightLayer`]. Sky-specific state
//!   (`top_sections`, `current_lowest_y`, `repeat_first_layer` materialisation)
//!   lives in dedicated fields that the block variant never touches.

use std::collections::{HashMap, HashSet};

use crate::lighting::data_layer::DataLayer;
use crate::lighting::light_layer::LightLayer;
use crate::lighting::positions::{
    block_pos_x, block_pos_y, block_pos_z, block_to_section, section_pos_as_long,
    section_pos_get_zero_node, section_pos_offset, section_pos_x, section_pos_y, section_pos_z,
    section_relative,
};

const SKY_MAX_LIGHT: u8 = 15;

/// Java: `LayerLightSectionStorage.SectionType`. The string in `display()` is
/// what Java prints in F3 debug screens; we keep it for parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionType {
    /// Java: `EMPTY ("2")` — no data, no neighbours storing light.
    Empty,
    /// Java: `LIGHT_ONLY ("1")` — the section is itself empty (or unloaded)
    /// but a neighbour stores light, so it is consulted for propagation.
    LightOnly,
    /// Java: `LIGHT_AND_DATA ("0")` — this section stores its own `DataLayer`.
    LightAndData,
}

impl SectionType {
    pub const fn display(self) -> &'static str {
        match self {
            SectionType::LightAndData => "0",
            SectionType::LightOnly => "1",
            SectionType::Empty => "2",
        }
    }
}

/// Java: `LayerLightSectionStorage.SectionState`.
///
/// Byte layout:
/// - bit 5 (`HAS_DATA_BIT = 32`): `true` iff this section stores its own data
///   layer (i.e. is not empty).
/// - bits 0..4 (`NEIGHBOR_COUNT_BITS = 31`): number of non-empty neighbouring
///   sections in the 3x3x3 cube centred on the section, range 0..=26.
mod section_state {
    pub const EMPTY: u8 = 0;
    const HAS_DATA_BIT: u8 = 32;
    const NEIGHBOR_COUNT_BITS: u8 = 31;

    #[inline]
    pub fn with_has_data(state: u8, has_data: bool) -> u8 {
        if has_data {
            state | HAS_DATA_BIT
        } else {
            state & !HAS_DATA_BIT
        }
    }

    #[inline]
    pub fn with_neighbor_count(state: u8, count: i32) -> u8 {
        assert!(
            (0..=26).contains(&count),
            "neighbour count must be in 0..=26"
        );
        (state & !NEIGHBOR_COUNT_BITS) | ((count as u8) & NEIGHBOR_COUNT_BITS)
    }

    #[inline]
    pub fn has_data(state: u8) -> bool {
        state & HAS_DATA_BIT != 0
    }

    #[inline]
    pub fn neighbor_count(state: u8) -> i32 {
        (state & NEIGHBOR_COUNT_BITS) as i32
    }

    #[inline]
    pub fn section_type(state: u8) -> super::SectionType {
        if state == EMPTY {
            super::SectionType::Empty
        } else if has_data(state) {
            super::SectionType::LightAndData
        } else {
            super::SectionType::LightOnly
        }
    }
}

/// Java: `DataLayerStorageMap` — the visible-vs-updating split lives here.
#[derive(Debug, Clone, Default)]
pub struct DataLayerStorageMap {
    sections: HashMap<i64, DataLayer>,
}

impl DataLayerStorageMap {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    pub fn has_layer(&self, section_node: i64) -> bool {
        self.sections.contains_key(&section_node)
    }

    pub fn get_layer(&self, section_node: i64) -> Option<&DataLayer> {
        self.sections.get(&section_node)
    }

    pub fn get_layer_mut(&mut self, section_node: i64) -> Option<&mut DataLayer> {
        self.sections.get_mut(&section_node)
    }

    pub fn set_layer(&mut self, section_node: i64, layer: DataLayer) {
        self.sections.insert(section_node, layer);
    }

    pub fn remove_layer(&mut self, section_node: i64) -> Option<DataLayer> {
        self.sections.remove(&section_node)
    }

    /// Java: `copyDataLayer(long)`. Returns `None` only if the caller violated
    /// the invariant that the section must already be present; callers in this
    /// module always check `has_layer` immediately before invoking this.
    pub fn copy_data_layer(&mut self, section_node: i64) -> Option<&mut DataLayer> {
        let cloned = self.sections.get(&section_node).map(DataLayer::copy)?;
        self.sections.insert(section_node, cloned);
        self.sections.get_mut(&section_node)
    }

    pub fn keys(&self) -> impl Iterator<Item = i64> + '_ {
        self.sections.keys().copied()
    }
}

/// Java: union of `LayerLightSectionStorage` + the block/sky subclass state.
#[derive(Debug)]
pub struct LayerLightSectionStorage {
    layer: LightLayer,
    /// Java: `sectionStates: Long2ByteMap` (defaults to 0 / EMPTY when absent).
    section_states: HashMap<i64, u8>,
    /// Java: `columnsWithSources` — set of zero-node columns that have light
    /// propagation enabled.
    columns_with_sources: HashSet<i64>,
    /// Java: `visibleSectionData` — read-only snapshot exposed to outside code.
    visible: DataLayerStorageMap,
    /// Java: `updatingSectionData` — what the engine mutates in place.
    updating: DataLayerStorageMap,
    /// Java: `changedSections` — accumulated section diffs awaiting `swapSectionMap`.
    changed_sections: HashSet<i64>,
    /// Java: `sectionsAffectedByLightUpdates` — flushed on `swapSectionMap`.
    sections_affected_by_light_updates: HashSet<i64>,
    /// Java: `queuedSections` — `Long2ObjectMap<DataLayer>`.
    queued_sections: HashMap<i64, DataLayer>,
    /// Java: `columnsToRetainQueuedDataFor`.
    columns_to_retain_queued: HashSet<i64>,
    /// Java: `toRemove`.
    to_remove: HashSet<i64>,
    /// Java: `hasInconsistencies`.
    has_inconsistencies: bool,

    // ---- sky-specific state ----
    /// Java (SkyLightSectionStorage): `topSections` — per zero-node, the
    /// `(top section Y) + 1`. Block-light storage ignores this map.
    top_sections: HashMap<i64, i32>,
    /// Java: `currentLowestY` — sentinel "below any stored section". Used as
    /// the default for `top_sections` and for `hasLightDataAtOrBelow`.
    current_lowest_y: i32,
}

impl LayerLightSectionStorage {
    pub fn new_block() -> Self {
        Self::new(LightLayer::Block)
    }

    pub fn new_sky() -> Self {
        Self::new(LightLayer::Sky)
    }

    fn new(layer: LightLayer) -> Self {
        Self {
            layer,
            section_states: HashMap::new(),
            columns_with_sources: HashSet::new(),
            visible: DataLayerStorageMap::new(),
            updating: DataLayerStorageMap::new(),
            changed_sections: HashSet::new(),
            sections_affected_by_light_updates: HashSet::new(),
            queued_sections: HashMap::new(),
            columns_to_retain_queued: HashSet::new(),
            to_remove: HashSet::new(),
            has_inconsistencies: false,
            top_sections: HashMap::new(),
            current_lowest_y: i32::MAX,
        }
    }

    pub fn layer(&self) -> LightLayer {
        self.layer
    }

    /// Java: `storingLightForSection(long)`.
    pub fn storing_light_for_section(&self, section_node: i64) -> bool {
        self.updating.has_layer(section_node)
    }

    /// Java: `getDataLayer(long, boolean)`.
    pub fn get_data_layer(&self, section_node: i64, updating: bool) -> Option<&DataLayer> {
        if updating {
            self.updating.get_layer(section_node)
        } else {
            self.visible.get_layer(section_node)
        }
    }

    pub fn get_data_layer_mut(&mut self, section_node: i64) -> Option<&mut DataLayer> {
        self.updating.get_layer_mut(section_node)
    }

    /// Java: `getDataLayerData(long)`.
    pub fn get_data_layer_data(&self, section_node: i64) -> Option<&DataLayer> {
        if let Some(queued) = self.queued_sections.get(&section_node) {
            Some(queued)
        } else {
            self.visible.get_layer(section_node)
        }
    }

    /// Java: `getDataLayerToWrite(long)`. Idempotent within a single
    /// `runLightUpdates` cycle thanks to `changed_sections`.
    pub fn get_data_layer_to_write(&mut self, section_node: i64) -> Option<&mut DataLayer> {
        if !self.updating.has_layer(section_node) {
            return None;
        }
        if self.changed_sections.insert(section_node) {
            // `has_layer` returned true above, so `copy_data_layer` is safe.
            let _ = self.updating.copy_data_layer(section_node);
        }
        self.updating.get_layer_mut(section_node)
    }

    /// Java: `getStoredLevel(long)`. Returns 0 if the section is not stored;
    /// callers always gate this on `storing_light_for_section(...)` first.
    pub fn get_stored_level(&self, block_node: i64) -> i32 {
        let section_node = block_to_section(block_node);
        let Some(layer) = self.updating.get_layer(section_node) else {
            return 0;
        };
        layer.get(
            section_relative(block_pos_x(block_node)) as usize,
            section_relative(block_pos_y(block_node)) as usize,
            section_relative(block_pos_z(block_node)) as usize,
        ) as i32
    }

    /// Java: `setStoredLevel(long, int)`. Silently no-ops if the section is
    /// not stored; callers gate this on `storing_light_for_section`.
    pub fn set_stored_level(&mut self, block_node: i64, level: i32) {
        let section_node = block_to_section(block_node);
        if self.changed_sections.insert(section_node) {
            let _ = self.updating.copy_data_layer(section_node);
        }
        let Some(layer) = self.updating.get_layer_mut(section_node) else {
            return;
        };
        layer.set(
            section_relative(block_pos_x(block_node)) as usize,
            section_relative(block_pos_y(block_node)) as usize,
            section_relative(block_pos_z(block_node)) as usize,
            level as u8,
        );
        Self::around_and_at_block_pos(block_node, &mut self.sections_affected_by_light_updates);
    }

    fn around_and_at_block_pos(block_node: i64, sink: &mut HashSet<i64>) {
        let bx = block_pos_x(block_node);
        let by = block_pos_y(block_node);
        let bz = block_pos_z(block_node);
        let min_section_x = (bx - 1) >> 4;
        let max_section_x = (bx + 1) >> 4;
        let min_section_y = (by - 1) >> 4;
        let max_section_y = (by + 1) >> 4;
        let min_section_z = (bz - 1) >> 4;
        let max_section_z = (bz + 1) >> 4;
        for sx in min_section_x..=max_section_x {
            for sy in min_section_y..=max_section_y {
                for sz in min_section_z..=max_section_z {
                    sink.insert(section_pos_as_long(sx, sy, sz));
                }
            }
        }
    }

    /// Java: `hasInconsistencies()`.
    pub fn has_inconsistencies(&self) -> bool {
        self.has_inconsistencies
    }

    /// Java: `setLightEnabled(zeroNode, enable)`.
    pub fn set_light_enabled(&mut self, zero_node: i64, enable: bool) {
        if enable {
            self.columns_with_sources.insert(zero_node);
        } else {
            self.columns_with_sources.remove(&zero_node);
        }
    }

    /// Java: `lightOnInSection(long)`.
    pub fn light_on_in_section(&self, section_node: i64) -> bool {
        self.columns_with_sources
            .contains(&section_pos_get_zero_node(section_node))
    }

    /// Java: `lightOnInColumn(zeroNode)`.
    pub fn light_on_in_column(&self, zero_node: i64) -> bool {
        self.columns_with_sources.contains(&zero_node)
    }

    /// Java: `retainData(zeroNode, retain)`.
    pub fn retain_data(&mut self, zero_node: i64, retain: bool) {
        if retain {
            self.columns_to_retain_queued.insert(zero_node);
        } else {
            self.columns_to_retain_queued.remove(&zero_node);
        }
    }

    /// Java: `queueSectionData(long, DataLayer?)`.
    pub fn queue_section_data(&mut self, section_node: i64, data: Option<DataLayer>) {
        match data {
            Some(layer) => {
                self.queued_sections.insert(section_node, layer);
                self.has_inconsistencies = true;
            }
            None => {
                self.queued_sections.remove(&section_node);
            }
        }
    }

    /// Java: `updateSectionStatus(sectionNode, sectionEmpty)`.
    pub fn update_section_status(&mut self, section_node: i64, section_empty: bool) {
        let state = self
            .section_states
            .get(&section_node)
            .copied()
            .unwrap_or(section_state::EMPTY);
        let new_state = section_state::with_has_data(state, !section_empty);
        if state == new_state {
            return;
        }
        self.put_section_state(section_node, new_state);
        let neighbour_increment = if section_empty { -1 } else { 1 };
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                for offset_z in -1..=1 {
                    if offset_x == 0 && offset_y == 0 && offset_z == 0 {
                        continue;
                    }
                    let neighbour = section_pos_offset(section_node, offset_x, offset_y, offset_z);
                    let neighbour_state = self
                        .section_states
                        .get(&neighbour)
                        .copied()
                        .unwrap_or(section_state::EMPTY);
                    let new_count = section_state::neighbor_count(neighbour_state) + neighbour_increment;
                    let new_neighbour_state =
                        section_state::with_neighbor_count(neighbour_state, new_count);
                    self.put_section_state(neighbour, new_neighbour_state);
                }
            }
        }
    }

    /// Java: `putSectionState`.
    fn put_section_state(&mut self, section_node: i64, state: u8) {
        if state != section_state::EMPTY {
            let previous = self
                .section_states
                .insert(section_node, state)
                .unwrap_or(section_state::EMPTY);
            if previous == section_state::EMPTY {
                self.initialize_section(section_node);
            }
        } else if self.section_states.remove(&section_node).is_some() {
            self.remove_section(section_node);
        }
    }

    fn initialize_section(&mut self, section_node: i64) {
        if !self.to_remove.remove(&section_node) {
            let layer = self.create_data_layer(section_node);
            self.updating.set_layer(section_node, layer);
            self.changed_sections.insert(section_node);
            self.on_node_added(section_node);
            // sectionsAffectedByLightUpdates += 3x3x3 around the section
            let sx = section_pos_x(section_node);
            let sy = section_pos_y(section_node);
            let sz = section_pos_z(section_node);
            for offset_z in -1..=1 {
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        self.sections_affected_by_light_updates.insert(
                            section_pos_as_long(sx + offset_x, sy + offset_y, sz + offset_z),
                        );
                    }
                }
            }
            self.has_inconsistencies = true;
        }
    }

    fn remove_section(&mut self, section_node: i64) {
        self.to_remove.insert(section_node);
        self.has_inconsistencies = true;
    }

    /// Java: `createDataLayer(long)` — block/sky differ here.
    fn create_data_layer(&mut self, section_node: i64) -> DataLayer {
        if let Some(queued) = self.queued_sections.get(&section_node) {
            return queued.copy();
        }
        match self.layer {
            LightLayer::Block => DataLayer::empty(),
            LightLayer::Sky => {
                let zero = section_pos_get_zero_node(section_node);
                let top_section = self
                    .top_sections
                    .get(&zero)
                    .copied()
                    .unwrap_or(self.current_lowest_y);
                if top_section != self.current_lowest_y && section_pos_y(section_node) < top_section
                {
                    // Java: walk up until we find a stored layer and copy its
                    // bottom slab into a fresh 16-deep layer.
                    let mut walk = section_pos_offset(section_node, 0, 1, 0);
                    loop {
                        match self.updating.get_layer(walk) {
                            Some(layer) => break repeat_first_layer(&layer.clone()),
                            None => walk = section_pos_offset(walk, 0, 1, 0),
                        }
                    }
                } else if self.light_on_in_section(section_node) {
                    DataLayer::with_default(SKY_MAX_LIGHT)
                } else {
                    DataLayer::empty()
                }
            }
        }
    }

    /// Java: `onNodeAdded` (sky variant).
    fn on_node_added(&mut self, section_node: i64) {
        if !matches!(self.layer, LightLayer::Sky) {
            return;
        }
        let y = section_pos_y(section_node);
        if self.current_lowest_y > y {
            self.current_lowest_y = y;
        }
        let zero = section_pos_get_zero_node(section_node);
        let entry = self.top_sections.entry(zero).or_insert(self.current_lowest_y);
        if *entry < y + 1 {
            *entry = y + 1;
        }
    }

    /// Java: `onNodeRemoved` (sky variant).
    fn on_node_removed(&mut self, section_node: i64) {
        if !matches!(self.layer, LightLayer::Sky) {
            return;
        }
        let zero = section_pos_get_zero_node(section_node);
        let mut y = section_pos_y(section_node);
        let current_top = self
            .top_sections
            .get(&zero)
            .copied()
            .unwrap_or(self.current_lowest_y);
        if current_top != y + 1 {
            return;
        }
        let mut walk = section_node;
        while !self.storing_light_for_section(walk) && self.has_light_data_at_or_below(y) {
            walk = section_pos_offset(walk, 0, -1, 0);
            y -= 1;
        }
        if self.storing_light_for_section(walk) {
            self.top_sections.insert(zero, y + 1);
        } else {
            self.top_sections.remove(&zero);
        }
    }

    pub fn has_light_data_at_or_below(&self, section_y: i32) -> bool {
        section_y >= self.current_lowest_y
    }

    pub fn is_above_data(&self, section_node: i64) -> bool {
        let zero = section_pos_get_zero_node(section_node);
        let top = self
            .top_sections
            .get(&zero)
            .copied()
            .unwrap_or(self.current_lowest_y);
        top == self.current_lowest_y || section_pos_y(section_node) >= top
    }

    pub fn get_top_section_y(&self, zero_node: i64) -> i32 {
        self.top_sections
            .get(&zero_node)
            .copied()
            .unwrap_or(self.current_lowest_y)
    }

    pub fn get_bottom_section_y(&self) -> i32 {
        self.current_lowest_y
    }

    /// Java: `markNewInconsistencies(LightEngine)`. The Java method takes an
    /// `engine` reference because it dispatches to override hooks via the
    /// engine reference; in our unified storage we don't need that
    /// indirection. Returns the list of affected sections so callers can run
    /// the engine's `enqueueIncrease`/`enqueueDecrease` work for pending block
    /// nodes.
    pub fn mark_new_inconsistencies(&mut self) {
        if !self.has_inconsistencies {
            return;
        }
        self.has_inconsistencies = false;
        let removals: Vec<i64> = self.to_remove.iter().copied().collect();
        for node in &removals {
            let queued = self.queued_sections.remove(node);
            let stored = self.updating.remove_layer(*node);
            if self
                .columns_to_retain_queued
                .contains(&section_pos_get_zero_node(*node))
            {
                if let Some(layer) = queued.or(stored) {
                    self.queued_sections.insert(*node, layer);
                }
            }
        }
        for node in &removals {
            self.on_node_removed(*node);
            self.changed_sections.insert(*node);
        }
        self.to_remove.clear();

        let queued_nodes: Vec<i64> = self.queued_sections.keys().copied().collect();
        for section_node in queued_nodes {
            if self.storing_light_for_section(section_node) {
                if let Some(data) = self.queued_sections.remove(&section_node) {
                    self.updating.set_layer(section_node, data);
                    self.changed_sections.insert(section_node);
                }
            }
        }
    }

    /// Java: `swapSectionMap`.
    pub fn swap_section_map(&mut self) {
        if !self.changed_sections.is_empty() {
            self.visible = self.updating.clone();
            self.changed_sections.clear();
        }
        self.sections_affected_by_light_updates.clear();
    }

    /// Java: `getDebugSectionType(long)`.
    pub fn get_debug_section_type(&self, section_node: i64) -> SectionType {
        let state = self
            .section_states
            .get(&section_node)
            .copied()
            .unwrap_or(section_state::EMPTY);
        section_state::section_type(state)
    }

    /// Iterate every section node currently stored in the *visible* map.
    pub fn visible_section_nodes(&self) -> impl Iterator<Item = i64> + '_ {
        self.visible.keys()
    }

    /// Snapshot the visible data layer for a section, materialising the
    /// lazy default if necessary.
    pub fn snapshot_visible_layer(&self, section_node: i64) -> Option<DataLayer> {
        self.visible.get_layer(section_node).map(|l| l.copy())
    }

    /// Block-light specific Java path: `getLightValue` reads the visible map
    /// directly with a 0 default.
    fn get_block_light_value(&self, block_node: i64) -> i32 {
        let section_node = block_to_section(block_node);
        match self.visible.get_layer(section_node) {
            None => 0,
            Some(layer) => layer.get(
                section_relative(block_pos_x(block_node)) as usize,
                section_relative(block_pos_y(block_node)) as usize,
                section_relative(block_pos_z(block_node)) as usize,
            ) as i32,
        }
    }

    /// Sky-light specific Java path: walks up to the topmost stored layer.
    fn get_sky_light_value(&self, block_node: i64, updating: bool) -> i32 {
        let mut section_node = block_to_section(block_node);
        let mut section_y = section_pos_y(section_node);
        let map = if updating { &self.updating } else { &self.visible };
        let top_section = self
            .top_sections
            .get(&section_pos_get_zero_node(section_node))
            .copied()
            .unwrap_or(self.current_lowest_y);
        if top_section != self.current_lowest_y && section_y < top_section {
            // Java's `BlockPos.getFlatIndex` clears the Y bits *only* when
            // walking up the column; an in-place hit preserves the real Y.
            let mut effective_local_y = section_relative(block_pos_y(block_node)) as usize;
            let mut lookup_layer = map.get_layer(section_node);
            if lookup_layer.is_none() {
                effective_local_y = 0;
                loop {
                    section_y += 1;
                    if section_y >= top_section {
                        return SKY_MAX_LIGHT as i32;
                    }
                    section_node = section_pos_offset(section_node, 0, 1, 0);
                    if let Some(layer) = map.get_layer(section_node) {
                        lookup_layer = Some(layer);
                        break;
                    }
                }
            }
            let Some(lookup) = lookup_layer else {
                return SKY_MAX_LIGHT as i32;
            };
            return lookup.get(
                section_relative(block_pos_x(block_node)) as usize,
                effective_local_y,
                section_relative(block_pos_z(block_node)) as usize,
            ) as i32;
        }
        if updating && !self.light_on_in_section(section_node) {
            0
        } else {
            SKY_MAX_LIGHT as i32
        }
    }

    /// Java (subclass-dispatched): `getLightValue(long)`.
    pub fn get_light_value(&self, block_node: i64) -> i32 {
        match self.layer {
            LightLayer::Block => self.get_block_light_value(block_node),
            LightLayer::Sky => self.get_sky_light_value(block_node, false),
        }
    }

}

/// Java: `SkyLightSectionStorage.repeatFirstLayer(DataLayer)`.
fn repeat_first_layer(template: &DataLayer) -> DataLayer {
    if template.is_definitely_homogenous() {
        return template.copy();
    }
    let input = template.to_bytes();
    // The first 128 bytes are the y=0 slab; copy it into all 16 y-layers.
    let mut output = Vec::with_capacity(input.len());
    for _ in 0..16 {
        output.extend_from_slice(&input[..128]);
    }
    DataLayer::from_bytes(output)
}

