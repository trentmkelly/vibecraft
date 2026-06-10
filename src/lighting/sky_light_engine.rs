//! Mirrors `net.minecraft.world.level.lighting.SkyLightEngine`.
//!
//! Sky-light propagation. Sources are columns of voxels at or above the
//! lowest-sky-source Y (`ChunkSkyLightSources`). The engine seeds those source
//! voxels at level 15, then propagates downward, with cardinal-direction
//! "bleed" through neighbouring open columns (so sky-light flows sideways
//! into the shadow of an overhang).
//!
//! Java has a `propagateFromEmptySections` helper that handles the case where
//! propagation crosses a section boundary into a sequence of empty sections
//! below; we replicate that behaviour so light correctly bleeds through air
//! columns spanning multiple unfilled sections.

use crate::lighting::chunk_sky_light_sources::{ChunkSkyLightSources, NEGATIVE_INFINITY};
use crate::lighting::data_layer::MAX_LIGHT_LEVEL;
use crate::lighting::direction::{Direction, PROPAGATION_DIRECTIONS};
use crate::lighting::light_chunk::{shape_occludes, LightBlockProperties, LightChunkGetter};
use crate::lighting::light_engine::{
    light_engine_get_opacity, neighbour_block_node, run_light_updates, LightEngineBase,
};
use crate::lighting::positions::{
    block_pos_as_long, block_pos_x, block_pos_y, block_pos_z, block_to_section,
    block_to_section_coord, section_pos_as_long, section_pos_offset, section_pos_y,
    section_pos_zero_node, section_relative, section_to_block_coord,
};
use crate::lighting::queue_entry::{
    decrease_all_directions, decrease_skip_one_direction, get_from_level,
    increase_only_one_direction, increase_skip_one_direction, increase_sky_source_in_directions,
    is_from_empty_shape, pull_light_in_entry, should_propagate_in_direction,
};
use crate::lighting::storage::LayerLightSectionStorage;

const SOURCE_LEVEL: i32 = MAX_LIGHT_LEVEL as i32;

fn remove_top_sky_source_entry() -> i64 {
    decrease_all_directions(SOURCE_LEVEL)
}
fn remove_sky_source_entry() -> i64 {
    decrease_skip_one_direction(SOURCE_LEVEL, Direction::Up)
}
fn add_sky_source_entry() -> i64 {
    increase_skip_one_direction(SOURCE_LEVEL, false, Direction::Up)
}

/// Java: `SkyLightEngine`.
#[derive(Debug)]
pub struct SkyLightEngine {
    pub base: LightEngineBase,
    empty_chunk_sources: ChunkSkyLightSources,
}

impl SkyLightEngine {
    pub fn new(empty_chunk_sources: ChunkSkyLightSources) -> Self {
        Self {
            base: LightEngineBase::new(LayerLightSectionStorage::new_sky()),
            empty_chunk_sources,
        }
    }

    pub fn check_block(&mut self, block_node: i64) {
        self.base.check_block(block_node);
    }

    pub fn has_light_work(&self) -> bool {
        self.base.has_light_work()
    }

    pub fn last_run_node_count(&self) -> u64 {
        self.base.last_run_node_count()
    }

    pub fn run_light_updates(&mut self, chunk_source: &dyn LightChunkGetter) -> u64 {
        let empty = self.empty_chunk_sources.clone();
        run_light_updates(
            &mut self.base,
            chunk_source,
            move |base, src, node| Self::check_node(base, src, &empty, node),
            move |base, src, from_node, increase_data, from_level| {
                Self::propagate_increase(base, src, from_node, increase_data, from_level);
            },
            move |base, src, from_node, decrease_data| {
                Self::propagate_decrease(base, src, from_node, decrease_data);
            },
        )
    }

    /// Java: `setLightEnabled(ChunkPos, boolean)`.
    pub fn set_light_enabled(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
        enable: bool,
    ) {
        let zero = section_pos_zero_node(chunk_x, chunk_z);
        self.base.storage.set_light_enabled(zero, enable);
        if !enable {
            return;
        }
        let sources = chunk_source
            .sky_light_sources(chunk_x, chunk_z)
            .unwrap_or_else(|| self.empty_chunk_sources.clone());
        let highest_lowest_source_y = sources.get_highest_lowest_source_y();
        if highest_lowest_source_y == NEGATIVE_INFINITY {
            return;
        }
        let highest_non_source_y = highest_lowest_source_y - 1;
        let lowest_fully_source_section_y = block_to_section_coord(highest_non_source_y) + 1;
        let top_section_y = self.base.storage.get_top_section_y(zero);
        let bottom_section_y = self
            .base
            .storage
            .get_bottom_section_y()
            .max(lowest_fully_source_section_y);
        for section_y in (bottom_section_y..top_section_y).rev() {
            let section_node = section_pos_as_long(chunk_x, section_y, chunk_z);
            if let Some(layer) = self.base.storage.get_data_layer_to_write(section_node) {
                if layer.is_empty() {
                    layer.fill(SOURCE_LEVEL as u8);
                }
            }
        }
    }

    /// Java: `propagateLightSources(ChunkPos)`.
    pub fn propagate_light_sources(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
    ) {
        let zero = section_pos_zero_node(chunk_x, chunk_z);
        self.base.storage.set_light_enabled(zero, true);
        let neighbours = NeighbourSkySources {
            sources: chunk_source
                .sky_light_sources(chunk_x, chunk_z)
                .unwrap_or_else(|| self.empty_chunk_sources.clone()),
            north: chunk_source
                .sky_light_sources(chunk_x, chunk_z - 1)
                .unwrap_or_else(|| self.empty_chunk_sources.clone()),
            south: chunk_source
                .sky_light_sources(chunk_x, chunk_z + 1)
                .unwrap_or_else(|| self.empty_chunk_sources.clone()),
            west: chunk_source
                .sky_light_sources(chunk_x - 1, chunk_z)
                .unwrap_or_else(|| self.empty_chunk_sources.clone()),
            east: chunk_source
                .sky_light_sources(chunk_x + 1, chunk_z)
                .unwrap_or_else(|| self.empty_chunk_sources.clone()),
        };
        let top_section_y = self.base.storage.get_top_section_y(zero);
        let bottom_section_y = self.base.storage.get_bottom_section_y();
        let section_min_x = section_to_block_coord(chunk_x);
        let section_min_z = section_to_block_coord(chunk_z);

        for section_y in (bottom_section_y..top_section_y).rev() {
            let section_node = section_pos_as_long(chunk_x, section_y, chunk_z);
            if self
                .base
                .storage
                .get_data_layer_to_write(section_node)
                .is_none()
            {
                continue;
            }
            let plan = seed_section_columns(&neighbours, section_y, section_min_x, section_min_z);
            if let Some(layer) = self.base.storage.get_data_layer_to_write(section_node) {
                for (x, ly, z, v) in plan.writes {
                    layer.set(x, ly, z, v as u8);
                }
            }
            for (node, entry) in plan.enqueues {
                self.base.enqueue_increase(node, entry);
            }
            if !plan.sources_below {
                break;
            }
        }
    }

    fn count_empty_sections_below_if_at_border(
        storage: &LayerLightSectionStorage,
        block_node: i64,
    ) -> i32 {
        let y = block_pos_y(block_node);
        let local_y = section_relative(y);
        if local_y != 0 {
            return 0;
        }
        let x = block_pos_x(block_node);
        let z = block_pos_z(block_node);
        let local_x = section_relative(x);
        let local_z = section_relative(z);
        if local_x != 0 && local_x != 15 && local_z != 0 && local_z != 15 {
            return 0;
        }
        let section_x = block_to_section_coord(x);
        let section_y = block_to_section_coord(y);
        let section_z = block_to_section_coord(z);
        let mut empty_sections_below = 0;
        while !storage.storing_light_for_section(section_pos_as_long(
            section_x,
            section_y - empty_sections_below - 1,
            section_z,
        )) && storage.has_light_data_at_or_below(section_y - empty_sections_below - 1)
        {
            empty_sections_below += 1;
        }
        empty_sections_below
    }

    fn check_node(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        empty_sources: &ChunkSkyLightSources,
        block_node: i64,
    ) {
        let x = block_pos_x(block_node);
        let y = block_pos_y(block_node);
        let z = block_pos_z(block_node);
        let section_node = block_to_section(block_node);
        let lowest_source_y = if base.storage.light_on_in_section(section_node) {
            get_lowest_source_y(chunk_source, empty_sources, x, z, i32::MAX)
        } else {
            i32::MAX
        };
        if lowest_source_y != i32::MAX {
            Self::update_sources_in_column(
                base,
                chunk_source,
                empty_sources,
                x,
                z,
                lowest_source_y,
            );
        }
        if base.storage.storing_light_for_section(section_node) {
            let is_source = y >= lowest_source_y;
            if is_source {
                base.enqueue_decrease(block_node, remove_sky_source_entry());
                base.enqueue_increase(block_node, add_sky_source_entry());
            } else {
                let old_level = base.storage.get_stored_level(block_node);
                if old_level > 0 {
                    base.storage.set_stored_level(block_node, 0);
                    base.enqueue_decrease(block_node, decrease_all_directions(old_level));
                } else {
                    base.enqueue_decrease(block_node, pull_light_in_entry());
                }
            }
        }
    }

    fn update_sources_in_column(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        empty_sources: &ChunkSkyLightSources,
        x: i32,
        z: i32,
        lowest_source_y: i32,
    ) {
        let world_bottom_y = section_to_block_coord(base.storage.get_bottom_section_y());
        Self::remove_sources_below(base, x, z, lowest_source_y, world_bottom_y);
        Self::add_sources_above(
            base,
            chunk_source,
            empty_sources,
            x,
            z,
            lowest_source_y,
            world_bottom_y,
        );
    }

    fn remove_sources_below(
        base: &mut LightEngineBase,
        x: i32,
        z: i32,
        lowest_source_y: i32,
        world_bottom_y: i32,
    ) {
        if lowest_source_y <= world_bottom_y {
            return;
        }
        let section_x = block_to_section_coord(x);
        let section_z = block_to_section_coord(z);
        let start_y = lowest_source_y - 1;
        let mut section_y = block_to_section_coord(start_y);
        while base.storage.has_light_data_at_or_below(section_y) {
            if base
                .storage
                .storing_light_for_section(section_pos_as_long(section_x, section_y, section_z))
            {
                let section_bottom_y = section_to_block_coord(section_y);
                let section_top_y = section_bottom_y + 15;
                let mut y = section_top_y.min(start_y);
                while y >= section_bottom_y {
                    let block_node = block_pos_as_long(x, y, z);
                    if base.storage.get_stored_level(block_node) != SOURCE_LEVEL {
                        return;
                    }
                    base.storage.set_stored_level(block_node, 0);
                    let entry = if y == lowest_source_y - 1 {
                        remove_top_sky_source_entry()
                    } else {
                        remove_sky_source_entry()
                    };
                    base.enqueue_decrease(block_node, entry);
                    if y == section_bottom_y {
                        break;
                    }
                    y -= 1;
                }
            }
            section_y -= 1;
        }
    }

    fn add_sources_above(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        empty_sources: &ChunkSkyLightSources,
        x: i32,
        z: i32,
        lowest_source_y: i32,
        world_bottom_y: i32,
    ) {
        let section_x = block_to_section_coord(x);
        let section_z = block_to_section_coord(z);
        let neighbor_lowest_source_y =
            get_lowest_source_y(chunk_source, empty_sources, x - 1, z, i32::MIN)
                .max(get_lowest_source_y(
                    chunk_source,
                    empty_sources,
                    x + 1,
                    z,
                    i32::MIN,
                ))
                .max(get_lowest_source_y(
                    chunk_source,
                    empty_sources,
                    x,
                    z - 1,
                    i32::MIN,
                ))
                .max(get_lowest_source_y(
                    chunk_source,
                    empty_sources,
                    x,
                    z + 1,
                    i32::MIN,
                ));
        let start_y = lowest_source_y.max(world_bottom_y);
        let mut section_node =
            section_pos_as_long(section_x, block_to_section_coord(start_y), section_z);
        while !base.storage.is_above_data(section_node) {
            if base.storage.storing_light_for_section(section_node) {
                let section_bottom_y = section_to_block_coord(section_pos_y(section_node));
                let section_top_y = section_bottom_y + 15;
                let mut y = section_bottom_y.max(start_y);
                while y <= section_top_y {
                    let block_node = block_pos_as_long(x, y, z);
                    if base.storage.get_stored_level(block_node) == SOURCE_LEVEL {
                        return;
                    }
                    base.storage.set_stored_level(block_node, SOURCE_LEVEL);
                    if y < neighbor_lowest_source_y || y == lowest_source_y {
                        base.enqueue_increase(block_node, add_sky_source_entry());
                    }
                    y += 1;
                }
            }
            section_node = section_pos_offset(section_node, 0, 1, 0);
        }
    }

    fn propagate_increase(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        increase_data: i64,
        from_level: i32,
    ) {
        let mut from_state: Option<LightBlockProperties> = None;
        let empty_sections_below =
            Self::count_empty_sections_below_if_at_border(&base.storage, from_node);
        for direction in PROPAGATION_DIRECTIONS {
            if !should_propagate_in_direction(increase_data, direction) {
                continue;
            }
            let to_node = neighbour_block_node(from_node, direction);
            let to_section = block_to_section(to_node);
            if !base.storage.storing_light_for_section(to_section) {
                continue;
            }
            let to_level = base.storage.get_stored_level(to_node);
            let max_possible_new_to_level = from_level - 1;
            if max_possible_new_to_level <= to_level {
                continue;
            }
            let to_x = block_pos_x(to_node);
            let to_y = block_pos_y(to_node);
            let to_z = block_pos_z(to_node);
            let to_state = chunk_source.light_properties_at(to_x, to_y, to_z);
            let to_opacity = light_engine_get_opacity(to_state);
            let new_to_level = from_level - to_opacity;
            if new_to_level <= to_level {
                continue;
            }
            let from_state = *from_state.get_or_insert_with(|| {
                if is_from_empty_shape(increase_data) {
                    LightBlockProperties::AIR
                } else {
                    let fx = block_pos_x(from_node);
                    let fy = block_pos_y(from_node);
                    let fz = block_pos_z(from_node);
                    chunk_source.light_properties_at(fx, fy, fz)
                }
            });
            if shape_occludes(from_state, to_state, direction) {
                continue;
            }
            base.storage.set_stored_level(to_node, new_to_level);
            if new_to_level > 1 {
                base.enqueue_increase(
                    to_node,
                    increase_skip_one_direction(
                        new_to_level,
                        to_state.has_empty_occlusion_shape(),
                        direction.opposite(),
                    ),
                );
            }
            Self::propagate_from_empty_sections(
                base,
                to_node,
                direction,
                new_to_level,
                true,
                empty_sections_below,
            );
        }
    }

    fn propagate_decrease(
        base: &mut LightEngineBase,
        _chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        decrease_data: i64,
    ) {
        let empty_sections_below =
            Self::count_empty_sections_below_if_at_border(&base.storage, from_node);
        let old_from_level = get_from_level(decrease_data);
        for direction in PROPAGATION_DIRECTIONS {
            if !should_propagate_in_direction(decrease_data, direction) {
                continue;
            }
            let to_node = neighbour_block_node(from_node, direction);
            let to_section = block_to_section(to_node);
            if !base.storage.storing_light_for_section(to_section) {
                continue;
            }
            let to_level = base.storage.get_stored_level(to_node);
            if to_level == 0 {
                continue;
            }
            if to_level < old_from_level {
                base.storage.set_stored_level(to_node, 0);
                base.enqueue_decrease(
                    to_node,
                    decrease_skip_one_direction(to_level, direction.opposite()),
                );
                Self::propagate_from_empty_sections(
                    base,
                    to_node,
                    direction,
                    to_level,
                    false,
                    empty_sections_below,
                );
            } else {
                base.enqueue_increase(
                    to_node,
                    increase_only_one_direction(to_level, false, direction.opposite()),
                );
            }
        }
    }

    fn propagate_from_empty_sections(
        base: &mut LightEngineBase,
        to_node: i64,
        direction: Direction,
        to_level: i32,
        increase: bool,
        empty_sections_below: i32,
    ) {
        if empty_sections_below == 0 {
            return;
        }
        let x = block_pos_x(to_node);
        let z = block_pos_z(to_node);
        if !crossed_section_edge(direction, section_relative(x), section_relative(z)) {
            return;
        }
        let y = block_pos_y(to_node);
        let section_x = block_to_section_coord(x);
        let section_z = block_to_section_coord(z);
        let mut section_y = block_to_section_coord(y) - 1;
        let bottom_section_y = section_y - empty_sections_below + 1;
        while section_y >= bottom_section_y {
            if !base
                .storage
                .storing_light_for_section(section_pos_as_long(section_x, section_y, section_z))
            {
                section_y -= 1;
                continue;
            }
            let section_min_y = section_to_block_coord(section_y);
            for local_y in (0..16).rev() {
                let block_node = block_pos_as_long(x, section_min_y + local_y, z);
                if increase {
                    base.storage.set_stored_level(block_node, to_level);
                    if to_level > 1 {
                        base.enqueue_increase(
                            block_node,
                            increase_skip_one_direction(to_level, true, direction.opposite()),
                        );
                    }
                } else {
                    base.storage.set_stored_level(block_node, 0);
                    base.enqueue_decrease(
                        block_node,
                        decrease_skip_one_direction(to_level, direction.opposite()),
                    );
                }
            }
            section_y -= 1;
        }
    }

    pub fn get_light_value(&self, block_node: i64) -> i32 {
        self.base.storage.get_light_value(block_node)
    }

    pub fn update_section_status(&mut self, section_node: i64, section_empty: bool) {
        self.base
            .storage
            .update_section_status(section_node, section_empty);
    }
}

fn crossed_section_edge(direction: Direction, x: i32, z: i32) -> bool {
    match direction {
        Direction::North => z == 15,
        Direction::South => z == 0,
        Direction::West => x == 15,
        Direction::East => x == 0,
        _ => false,
    }
}

fn get_lowest_source_y(
    chunk_source: &dyn LightChunkGetter,
    empty_sources: &ChunkSkyLightSources,
    x: i32,
    z: i32,
    default_value: i32,
) -> i32 {
    let chunk_x = block_to_section_coord(x);
    let chunk_z = block_to_section_coord(z);
    let sources = chunk_source
        .sky_light_sources(chunk_x, chunk_z)
        .unwrap_or_else(|| empty_sources.clone());
    let value = sources.get_lowest_source_y(section_relative(x), section_relative(z));
    if value == NEGATIVE_INFINITY {
        default_value
    } else {
        value
    }
}

// Re-export of MAX_LIGHT_LEVEL for callers; kept here so the engine's public
// surface mirrors the Java constant the storage and queue layers also expose.
pub const SKY_MAX_LIGHT_LEVEL: u8 = MAX_LIGHT_LEVEL;

/// Bundle of the five `ChunkSkyLightSources` heightmaps consulted by
/// `propagate_light_sources` — the centre chunk plus its four cardinal
/// neighbours. Extracted into a struct so the per-section seeding helper
/// stays narrow.
struct NeighbourSkySources {
    sources: ChunkSkyLightSources,
    north: ChunkSkyLightSources,
    south: ChunkSkyLightSources,
    west: ChunkSkyLightSources,
    east: ChunkSkyLightSources,
}

struct SectionSeedPlan {
    writes: Vec<(usize, usize, usize, i32)>,
    enqueues: Vec<(i64, i64)>,
    sources_below: bool,
}

fn seed_section_columns(
    neighbours: &NeighbourSkySources,
    section_y: i32,
    section_min_x: i32,
    section_min_z: i32,
) -> SectionSeedPlan {
    let section_min_y = section_to_block_coord(section_y);
    let section_max_y = section_min_y + 15;
    let mut writes = Vec::new();
    let mut enqueues = Vec::new();
    let mut sources_below = false;
    for z in 0..16_i32 {
        for x in 0..16_i32 {
            let lowest_source_y = neighbours.sources.get_lowest_source_y(x, z);
            if lowest_source_y > section_max_y {
                continue;
            }
            let north_lowest_source_y = if z == 0 {
                neighbours.north.get_lowest_source_y(x, 15)
            } else {
                neighbours.sources.get_lowest_source_y(x, z - 1)
            };
            let south_lowest_source_y = if z == 15 {
                neighbours.south.get_lowest_source_y(x, 0)
            } else {
                neighbours.sources.get_lowest_source_y(x, z + 1)
            };
            let west_lowest_source_y = if x == 0 {
                neighbours.west.get_lowest_source_y(15, z)
            } else {
                neighbours.sources.get_lowest_source_y(x - 1, z)
            };
            let east_lowest_source_y = if x == 15 {
                neighbours.east.get_lowest_source_y(0, z)
            } else {
                neighbours.sources.get_lowest_source_y(x + 1, z)
            };
            let neighbor_lowest_source_y = north_lowest_source_y
                .max(south_lowest_source_y)
                .max(west_lowest_source_y)
                .max(east_lowest_source_y);
            let mut y = section_max_y;
            while y >= section_min_y.max(lowest_source_y) {
                let local_y = section_relative(y) as usize;
                writes.push((x as usize, local_y, z as usize, SOURCE_LEVEL));
                if y == lowest_source_y || y < neighbor_lowest_source_y {
                    let block_node = block_pos_as_long(section_min_x + x, y, section_min_z + z);
                    enqueues.push((
                        block_node,
                        increase_sky_source_in_directions(
                            y == lowest_source_y,
                            y < north_lowest_source_y,
                            y < south_lowest_source_y,
                            y < west_lowest_source_y,
                            y < east_lowest_source_y,
                        ),
                    ));
                }
                if y == section_min_y.max(lowest_source_y) {
                    break;
                }
                y -= 1;
            }
            if lowest_source_y < section_min_y {
                sources_below = true;
            }
        }
    }
    SectionSeedPlan {
        writes,
        enqueues,
        sources_below,
    }
}
