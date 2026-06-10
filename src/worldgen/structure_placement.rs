use super::*;

pub fn validate_random_spread_placement(spacing: i32, separation: i32) -> Result<(), String> {
    if !(0..=4096).contains(&spacing) || !(0..=4096).contains(&separation) {
        return Err("Random spread spacing and separation must be in 0..=4096".to_string());
    }
    if spacing <= separation {
        return Err("Spacing has to be larger than separation".to_string());
    }
    Ok(())
}

pub fn random_spread_potential_structure_chunk(
    seed: i64,
    source_x: i32,
    source_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
    spread_type: RandomSpreadType,
) -> Result<ChunkPos, String> {
    validate_random_spread_placement(spacing, separation)?;
    let spaced_grid_x = source_x.div_euclid(spacing);
    let spaced_grid_z = source_z.div_euclid(spacing);
    let mut random = LegacyRandom::new(large_feature_seed_with_salt(
        seed,
        spaced_grid_x,
        spaced_grid_z,
        salt,
    ));
    let limit = spacing - separation;
    let spread_x = spread_type.evaluate(&mut random, limit);
    let spread_z = spread_type.evaluate(&mut random, limit);
    Ok(ChunkPos {
        x: spaced_grid_x * spacing + spread_x,
        z: spaced_grid_z * spacing + spread_z,
    })
}

pub fn random_spread_is_placement_chunk(
    seed: i64,
    source_x: i32,
    source_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
    spread_type: RandomSpreadType,
) -> Result<bool, String> {
    let chunk = random_spread_potential_structure_chunk(
        seed,
        source_x,
        source_z,
        spacing,
        separation,
        salt,
        spread_type,
    )?;
    Ok(chunk.x == source_x && chunk.z == source_z)
}

pub fn validate_structure_frequency(frequency: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&frequency) {
        return Err("Structure placement frequency must be in 0.0..=1.0".to_string());
    }
    Ok(())
}

pub fn structure_frequency_reducer_should_generate(
    method: FrequencyReductionMethod,
    seed: i64,
    salt: i32,
    source_x: i32,
    source_z: i32,
    frequency: f32,
) -> Result<bool, String> {
    validate_structure_frequency(frequency)?;
    if frequency >= 1.0 {
        return Ok(true);
    }
    if frequency <= 0.0 {
        return Ok(false);
    }
    Ok(match method {
        FrequencyReductionMethod::Default => {
            let mut random =
                LegacyRandom::new(large_feature_seed_with_salt(seed, salt, source_x, source_z));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType1 => {
            let cx = source_x >> 4;
            let cz = source_z >> 4;
            let mut random = LegacyRandom::new((cx ^ (cz << 4)) as i64 ^ seed);
            random.next_i32();
            random.next_i32_bound((1.0 / frequency) as i32) == 0
        }
        FrequencyReductionMethod::LegacyType2 => {
            let mut random = LegacyRandom::new(large_feature_seed_with_salt(
                seed, source_x, source_z, 10_387_320,
            ));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType3 => {
            let mut random = LegacyRandom::new(crate::random_source::large_feature_seed(
                seed, source_x, source_z,
            ));
            random.next_f64() < frequency as f64
        }
    })
}

pub fn validate_structure_locate_offset(offset: BlockPos) -> Result<(), String> {
    if (-16..=16).contains(&offset.x)
        && (-16..=16).contains(&offset.y)
        && (-16..=16).contains(&offset.z)
    {
        Ok(())
    } else {
        Err("Structure locate offset components must be in -16..=16".to_string())
    }
}

pub fn structure_locate_pos(
    chunk_pos: ChunkPos,
    locate_offset: BlockPos,
) -> Result<BlockPos, String> {
    validate_structure_locate_offset(locate_offset)?;
    Ok(BlockPos {
        x: chunk_pos.x * 16 + locate_offset.x,
        y: locate_offset.y,
        z: chunk_pos.z * 16 + locate_offset.z,
    })
}

pub fn validate_concentric_rings_placement(
    distance: i32,
    spread: i32,
    count: i32,
) -> Result<(), String> {
    if !(0..=1023).contains(&distance) || !(0..=1023).contains(&spread) {
        return Err("Concentric rings distance and spread must be in 0..=1023".to_string());
    }
    if !(1..=4095).contains(&count) {
        return Err("Concentric rings count must be in 1..=4095".to_string());
    }
    Ok(())
}

pub fn concentric_ring_initial_candidates(
    seed: i64,
    distance: i32,
    spread: i32,
    count: i32,
) -> Result<Vec<ConcentricRingPlacementCandidate>, String> {
    validate_concentric_rings_placement(distance, spread, count)?;
    let mut random = LegacyRandom::new(seed);
    let mut angle = random.next_f64() * std::f64::consts::PI * 2.0;
    let mut position_in_circle = 0;
    let mut circle = 0;
    let mut current_spread = spread;
    let mut candidates = Vec::with_capacity(count as usize);
    for index in 0..count {
        let dist = 4.0 * distance as f64
            + (distance * circle * 6) as f64
            + (random.next_f64() - 0.5) * (distance as f64 * 2.5);
        let initial_x = (angle.cos() * dist).round() as i32;
        let initial_z = (angle.sin() * dist).round() as i32;
        let _biome_search_generator = random.fork();
        candidates.push(ConcentricRingPlacementCandidate {
            index,
            circle,
            chunk_pos: ChunkPos {
                x: initial_x,
                z: initial_z,
            },
        });
        angle += (std::f64::consts::PI * 2.0) / current_spread as f64;
        position_in_circle += 1;
        if position_in_circle == current_spread {
            circle += 1;
            position_in_circle = 0;
            current_spread += 2 * current_spread / (circle + 1);
            current_spread = current_spread.min(count - index);
            angle += random.next_f64() * std::f64::consts::PI * 2.0;
        }
    }
    Ok(candidates)
}

pub fn concentric_rings_is_placement_chunk(
    ring_positions: &[ChunkPos],
    source_x: i32,
    source_z: i32,
) -> bool {
    ring_positions
        .iter()
        .any(|position| position.x == source_x && position.z == source_z)
}

pub fn concentric_ring_candidate_search_center(
    candidate: ConcentricRingPlacementCandidate,
) -> BlockPos {
    BlockPos {
        x: candidate.chunk_pos.x * 16 + 8,
        y: 0,
        z: candidate.chunk_pos.z * 16 + 8,
    }
}

pub fn concentric_ring_adjusted_position(
    candidate: ConcentricRingPlacementCandidate,
    closest_preferred_biome: Option<ConcentricRingBiomeSearchResult>,
) -> ChunkPos {
    closest_preferred_biome.map_or(candidate.chunk_pos, |position| ChunkPos {
        x: position.block_x.div_euclid(16),
        z: position.block_z.div_euclid(16),
    })
}

pub fn concentric_ring_adjusted_positions(
    candidates: &[ConcentricRingPlacementCandidate],
    preferred_biome_results: &[Option<ConcentricRingBiomeSearchResult>],
) -> Vec<ChunkPos> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            concentric_ring_adjusted_position(
                *candidate,
                preferred_biome_results.get(index).copied().flatten(),
            )
        })
        .collect()
}

impl ChunkGeneratorStructureStateModel {
    pub fn create_for_normal(
        level_seed: i64,
        structure_sets: &[StructureSetEntry],
        placeable_structures: &[&'static str],
    ) -> Self {
        Self::new(level_seed, level_seed, structure_sets, placeable_structures)
    }

    pub fn create_for_flat(
        level_seed: i64,
        structure_sets: &[StructureSetEntry],
        placeable_structures: &[&'static str],
    ) -> Self {
        Self::new(level_seed, 0, structure_sets, placeable_structures)
    }

    fn new(
        level_seed: i64,
        concentric_rings_seed: i64,
        structure_sets: &[StructureSetEntry],
        placeable_structures: &[&'static str],
    ) -> Self {
        let possible_structure_sets = structure_sets
            .iter()
            .copied()
            .filter(|set| Self::has_placeable_structure(set, placeable_structures))
            .collect();
        Self {
            level_seed,
            concentric_rings_seed,
            possible_structure_sets,
            placements_for_structure: BTreeMap::new(),
            ring_positions: BTreeMap::new(),
            has_generated_positions: false,
        }
    }

    pub fn ensure_structures_generated(&mut self, placeable_structures: &[&'static str]) {
        if self.has_generated_positions {
            return;
        }
        self.generate_positions(placeable_structures);
        self.has_generated_positions = true;
    }

    pub fn get_placements_for_structure(
        &mut self,
        structure: &'static str,
        placeable_structures: &[&'static str],
    ) -> Vec<StructurePlacementKind> {
        self.ensure_structures_generated(placeable_structures);
        self.placements_for_structure
            .get(structure)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_ring_positions_for(
        &mut self,
        structure_set_id: &'static str,
        placeable_structures: &[&'static str],
    ) -> Option<Vec<ChunkPos>> {
        self.ensure_structures_generated(placeable_structures);
        self.ring_positions.get(structure_set_id).cloned()
    }

    pub fn has_structure_chunk_in_range(
        &mut self,
        structure_set_id: &'static str,
        source_x: i32,
        source_z: i32,
        range: i32,
        placeable_structures: &[&'static str],
    ) -> Result<bool, String> {
        self.ensure_structures_generated(placeable_structures);
        let Some(set) = self
            .possible_structure_sets
            .iter()
            .find(|set| set.id == structure_set_id)
        else {
            return Ok(false);
        };
        for test_x in source_x - range..=source_x + range {
            for test_z in source_z - range..=source_z + range {
                if self.is_structure_chunk(set, test_x, test_z)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn generate_positions(&mut self, placeable_structures: &[&'static str]) {
        for set in &self.possible_structure_sets {
            let mut has_any_placeable_structures = false;
            for structure in set.structures {
                if placeable_structures.contains(structure) {
                    self.placements_for_structure
                        .entry(*structure)
                        .or_default()
                        .push(set.placement);
                    has_any_placeable_structures = true;
                }
            }
            if has_any_placeable_structures {
                if let StructurePlacementKind::ConcentricRings {
                    distance,
                    spread,
                    count,
                } = set.placement
                {
                    let candidates = concentric_ring_initial_candidates(
                        self.concentric_rings_seed,
                        distance,
                        spread,
                        count,
                    )
                    .unwrap_or_default();
                    self.ring_positions.insert(
                        set.id,
                        candidates
                            .iter()
                            .map(|candidate| candidate.chunk_pos)
                            .collect(),
                    );
                }
            }
        }
    }

    fn is_structure_chunk(
        &self,
        set: &StructureSetEntry,
        source_x: i32,
        source_z: i32,
    ) -> Result<bool, String> {
        match set.placement {
            StructurePlacementKind::RandomSpread {
                spacing,
                separation,
                salt,
                spread_type,
            } => random_spread_is_placement_chunk(
                self.level_seed,
                source_x,
                source_z,
                spacing,
                separation,
                salt,
                spread_type,
            ),
            StructurePlacementKind::ConcentricRings { .. } => {
                Ok(self.ring_positions.get(set.id).is_some_and(|positions| {
                    concentric_rings_is_placement_chunk(positions, source_x, source_z)
                }))
            }
        }
    }

    fn has_placeable_structure(
        set: &StructureSetEntry,
        placeable_structures: &[&'static str],
    ) -> bool {
        set.structures
            .iter()
            .any(|structure| placeable_structures.contains(structure))
    }
}

pub fn validate_structure_exclusion_zone(
    zone: StructureExclusionZoneModel,
) -> Result<StructureExclusionZoneModel, String> {
    if (1..=16).contains(&zone.chunk_count) {
        Ok(zone)
    } else {
        Err("Structure exclusion zone chunk_count must be in 1..=16".to_string())
    }
}

pub fn structure_has_chunk_in_range(
    structure_chunks: &[ChunkPos],
    source_x: i32,
    source_z: i32,
    range: i32,
) -> bool {
    (source_x - range..=source_x + range).any(|test_x| {
        (source_z - range..=source_z + range).any(|test_z| {
            structure_chunks
                .iter()
                .any(|chunk| chunk.x == test_x && chunk.z == test_z)
        })
    })
}

pub fn structure_exclusion_zone_forbids(
    zone: StructureExclusionZoneModel,
    other_structure_chunks: &[ChunkPos],
    source_x: i32,
    source_z: i32,
) -> Result<bool, String> {
    validate_structure_exclusion_zone(zone)?;
    Ok(structure_has_chunk_in_range(
        other_structure_chunks,
        source_x,
        source_z,
        zone.chunk_count,
    ))
}

impl StructureBoundingBoxModel {
    pub fn union(self, other: StructureBoundingBoxModel) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            min_z: self.min_z.min(other.min_z),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
            max_z: self.max_z.max(other.max_z),
        }
    }

    pub fn intersects(self, other: StructureBoundingBoxModel) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }

    pub fn is_inside(self, pos: BlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.z >= self.min_z
            && pos.z <= self.max_z
            && pos.y >= self.min_y
            && pos.y <= self.max_y
    }

    pub fn center(self) -> BlockPos {
        BlockPos {
            x: self.min_x + (self.max_x - self.min_x + 1) / 2,
            y: self.min_y + (self.max_y - self.min_y + 1) / 2,
            z: self.min_z + (self.max_z - self.min_z + 1) / 2,
        }
    }

    pub fn inflated_by(self, amount: i32) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x - amount,
            min_y: self.min_y - amount,
            min_z: self.min_z - amount,
            max_x: self.max_x + amount,
            max_y: self.max_y + amount,
            max_z: self.max_z + amount,
        }
    }

    pub fn moved(self, dx: i32, dy: i32, dz: i32) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x + dx,
            min_y: self.min_y + dy,
            min_z: self.min_z + dz,
            max_x: self.max_x + dx,
            max_y: self.max_y + dy,
            max_z: self.max_z + dz,
        }
    }

    pub fn encapsulate_pos(self, pos: BlockPos) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x.min(pos.x),
            min_y: self.min_y.min(pos.y),
            min_z: self.min_z.min(pos.z),
            max_x: self.max_x.max(pos.x),
            max_y: self.max_y.max(pos.y),
            max_z: self.max_z.max(pos.z),
        }
    }
}
