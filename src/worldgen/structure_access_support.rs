use super::*;

impl TerrainAdjustmentModel {
    pub fn id(self) -> &'static str {
        match self {
            TerrainAdjustmentModel::None => "none",
            TerrainAdjustmentModel::Bury => "bury",
            TerrainAdjustmentModel::BeardThin => "beard_thin",
            TerrainAdjustmentModel::BeardBox => "beard_box",
            TerrainAdjustmentModel::Encapsulate => "encapsulate",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "none" => Some(TerrainAdjustmentModel::None),
            "bury" => Some(TerrainAdjustmentModel::Bury),
            "beard_thin" => Some(TerrainAdjustmentModel::BeardThin),
            "beard_box" => Some(TerrainAdjustmentModel::BeardBox),
            "encapsulate" => Some(TerrainAdjustmentModel::Encapsulate),
            _ => None,
        }
    }

    pub fn jigsaw_edge_needed(self) -> i32 {
        match self {
            TerrainAdjustmentModel::None => 0,
            TerrainAdjustmentModel::Bury
            | TerrainAdjustmentModel::BeardThin
            | TerrainAdjustmentModel::BeardBox
            | TerrainAdjustmentModel::Encapsulate => 12,
        }
    }
}

pub fn structure_adjust_bounding_box(
    terrain_adjustment: TerrainAdjustmentModel,
    bounding_box: StructureBoundingBoxModel,
) -> StructureBoundingBoxModel {
    if terrain_adjustment == TerrainAdjustmentModel::None {
        bounding_box
    } else {
        bounding_box.inflated_by(12)
    }
}

pub fn jigsaw_max_distance_with_terrain_is_valid(
    horizontal_max_distance_from_center: i32,
    terrain_adjustment: TerrainAdjustmentModel,
) -> bool {
    horizontal_max_distance_from_center + terrain_adjustment.jigsaw_edge_needed() <= 128
}

impl StructureStartModel {
    pub fn invalid() -> Self {
        Self {
            structure: None,
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.pieces.is_empty()
    }

    pub fn max_references(&self) -> i32 {
        1
    }

    pub fn can_be_referenced(&self) -> bool {
        self.references < self.max_references()
    }

    pub fn add_reference(&mut self) {
        self.references += 1;
    }

    pub fn bounding_box(&self) -> Option<StructureBoundingBoxModel> {
        self.pieces
            .iter()
            .map(|piece| piece.bounding_box)
            .reduce(StructureBoundingBoxModel::union)
    }

    pub fn create_tag(&self, chunk_pos: ChunkPos) -> StructureStartTagModel {
        if self.is_valid() {
            StructureStartTagModel {
                id: self.structure.unwrap_or("minecraft:unknown"),
                chunk_x: Some(chunk_pos.x),
                chunk_z: Some(chunk_pos.z),
                references: Some(self.references),
                children: self.pieces.len(),
            }
        } else {
            StructureStartTagModel {
                id: "INVALID",
                chunk_x: None,
                chunk_z: None,
                references: None,
                children: 0,
            }
        }
    }
}

impl StructureAccessModel {
    pub fn get_start_for_structure(&self, structure: &'static str) -> Option<&StructureStartModel> {
        self.starts.get(structure)
    }

    pub fn set_start_for_structure(&mut self, structure: &'static str, start: StructureStartModel) {
        self.starts.insert(structure, start);
        self.unsaved = true;
    }

    pub fn set_all_starts(&mut self, starts: BTreeMap<&'static str, StructureStartModel>) {
        self.starts = starts;
        self.unsaved = true;
    }

    pub fn get_references_for_structure(&self, structure: &'static str) -> &[i64] {
        self.references
            .get(structure)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn add_reference_for_structure(&mut self, structure: &'static str, reference: i64) {
        let references = self.references.entry(structure).or_default();
        if !references.contains(&reference) {
            references.push(reference);
        }
        self.unsaved = true;
    }

    pub fn set_all_references(&mut self, references: BTreeMap<&'static str, Vec<i64>>) {
        self.references = references;
        self.unsaved = true;
    }

    pub fn has_any_structure_references(&self) -> bool {
        self.references
            .values()
            .any(|references| !references.is_empty())
    }
}

pub fn structure_access_to_chunk_structures_tag(
    access: &StructureAccessModel,
    chunk_pos: ChunkPos,
) -> Tag {
    let starts = access
        .starts
        .iter()
        .map(|(structure, start)| {
            (
                (*structure).to_string(),
                structure_start_to_chunk_tag(start, chunk_pos),
            )
        })
        .collect();
    let references = access
        .references
        .iter()
        .map(|(structure, reference_chunks)| {
            (
                (*structure).to_string(),
                Tag::LongArray(reference_chunks.clone()),
            )
        })
        .collect();
    Tag::Compound(vec![
        ("starts".to_string(), Tag::Compound(starts)),
        ("References".to_string(), Tag::Compound(references)),
    ])
}

fn structure_start_to_chunk_tag(start: &StructureStartModel, chunk_pos: ChunkPos) -> Tag {
    let tag = start.create_tag(chunk_pos);
    let mut fields = vec![("id".to_string(), Tag::String(tag.id.to_string()))];
    if let Some(chunk_x) = tag.chunk_x {
        fields.push(("ChunkX".to_string(), Tag::Int(chunk_x)));
    }
    if let Some(chunk_z) = tag.chunk_z {
        fields.push(("ChunkZ".to_string(), Tag::Int(chunk_z)));
    }
    if let Some(references) = tag.references {
        fields.push(("references".to_string(), Tag::Int(references)));
    }
    fields.push((
        "Children".to_string(),
        Tag::List(
            start
                .pieces
                .iter()
                .map(|piece| {
                    Tag::Compound(vec![(
                        "BB".to_string(),
                        Tag::IntArray(vec![
                            piece.bounding_box.min_x,
                            piece.bounding_box.min_y,
                            piece.bounding_box.min_z,
                            piece.bounding_box.max_x,
                            piece.bounding_box.max_y,
                            piece.bounding_box.max_z,
                        ]),
                    )])
                })
                .collect(),
        ),
    ));
    Tag::Compound(fields)
}

pub fn structure_start_reference_pos(first_piece: StructurePieceModel) -> BlockPos {
    let center = first_piece.bounding_box.center();
    BlockPos {
        x: center.x,
        y: first_piece.bounding_box.min_y,
        z: center.z,
    }
}

pub fn structure_pieces_intersecting_chunk(
    start: &StructureStartModel,
    chunk_bb: StructureBoundingBoxModel,
) -> Vec<StructurePieceModel> {
    start
        .pieces
        .iter()
        .copied()
        .filter(|piece| piece.bounding_box.intersects(chunk_bb))
        .collect()
}

pub fn structure_has_piece_at(pos: BlockPos, start: &StructureStartModel) -> bool {
    start
        .pieces
        .iter()
        .any(|piece| piece.bounding_box.is_inside(pos))
}

pub fn structure_start_contains_pos(pos: BlockPos, start: &StructureStartModel) -> bool {
    start
        .bounding_box()
        .is_some_and(|bounding_box| bounding_box.is_inside(pos))
}

pub fn structure_spawn_override_applies(
    pos: BlockPos,
    start: &StructureStartModel,
    override_model: StructureSpawnOverrideModel,
) -> bool {
    match override_model.bounding_box {
        StructureSpawnBoundingBoxTypeModel::Piece => structure_has_piece_at(pos, start),
        StructureSpawnBoundingBoxTypeModel::Full => structure_start_contains_pos(pos, start),
    }
}

pub fn chunk_generator_mobs_at<'a>(
    biome_spawns: &'a [&'a str],
    mob_category: &str,
    pos: BlockPos,
    structures_at_pos: &[StructureSpawnCandidateModel<'_>],
) -> Vec<&'a str> {
    for candidate in structures_at_pos {
        let Some(override_model) = candidate.override_model else {
            continue;
        };
        if override_model.category == mob_category
            && structure_spawn_override_applies(pos, candidate.start, override_model)
        {
            return override_model.spawns.to_vec();
        }
    }
    biome_spawns.to_vec()
}

pub fn first_structure_start_containing_pos(
    pos: BlockPos,
    starts: &[StructureStartModel],
) -> Option<StructureStartModel> {
    starts
        .iter()
        .find(|start| structure_start_contains_pos(pos, start))
        .cloned()
}

pub fn first_structure_start_with_piece_at(
    pos: BlockPos,
    starts: &[StructureStartModel],
) -> Option<StructureStartModel> {
    starts
        .iter()
        .find(|start| structure_has_piece_at(pos, start))
        .cloned()
}

pub fn structure_check_result_from_cached_references(
    references: Option<i32>,
    require_unreferenced: bool,
) -> StructureCheckResultModel {
    match references {
        None => StructureCheckResultModel::StartNotPresent,
        Some(reference_count) if require_unreferenced && reference_count != 0 => {
            StructureCheckResultModel::StartNotPresent
        }
        Some(_) => StructureCheckResultModel::StartPresent,
    }
}

pub fn structure_fast_check_allows_lookup(result: StructureCheckResultModel) -> bool {
    result != StructureCheckResultModel::StartNotPresent
}

pub fn structure_locate_can_return_fast(
    result: StructureCheckResultModel,
    create_reference: bool,
) -> bool {
    !create_reference && result == StructureCheckResultModel::StartPresent
}

pub fn structure_start_can_satisfy_lookup(
    start: &StructureStartModel,
    create_reference: bool,
) -> bool {
    start.is_valid() && (!create_reference || start.can_be_referenced())
}

pub fn structure_try_add_reference(start: &mut StructureStartModel) -> bool {
    if start.can_be_referenced() {
        start.add_reference();
        true
    } else {
        false
    }
}

pub fn structure_access_valid_starts_for_references(
    access_by_chunk: &BTreeMap<i64, StructureAccessModel>,
    structure: &'static str,
    references: &[i64],
) -> Vec<StructureStartModel> {
    references
        .iter()
        .filter_map(|reference| access_by_chunk.get(reference))
        .filter_map(|access| access.get_start_for_structure(structure))
        .filter(|start| start.is_valid())
        .cloned()
        .collect()
}

pub fn chunk_pos_key(chunk_pos: ChunkPos) -> i64 {
    (chunk_pos.x as u32 as i64) | ((chunk_pos.z as u32 as i64) << 32)
}

pub fn structure_reference_writable_area(chunk_pos: ChunkPos) -> StructureBoundingBoxModel {
    StructureBoundingBoxModel {
        min_x: chunk_pos.x * 16,
        min_y: i32::MIN,
        min_z: chunk_pos.z * 16,
        max_x: chunk_pos.x * 16 + 15,
        max_y: i32::MAX,
        max_z: chunk_pos.z * 16 + 15,
    }
}

pub fn structure_references_for_chunk(
    access_by_chunk: &BTreeMap<i64, StructureAccessModel>,
    target_chunk: ChunkPos,
) -> Vec<StructureReferenceModel> {
    let target_area = structure_reference_writable_area(target_chunk);
    let mut references = Vec::new();
    for source_x in target_chunk.x - 8..=target_chunk.x + 8 {
        for source_z in target_chunk.z - 8..=target_chunk.z + 8 {
            let source_chunk = ChunkPos {
                x: source_x,
                z: source_z,
            };
            let source_key = chunk_pos_key(source_chunk);
            let Some(access) = access_by_chunk.get(&source_key) else {
                continue;
            };
            for start in access.starts.values() {
                if start.is_valid()
                    && start
                        .bounding_box()
                        .is_some_and(|bbox| bbox.intersects(target_area))
                {
                    references.push(StructureReferenceModel {
                        structure: start.structure.unwrap_or("minecraft:unknown"),
                        source_chunk_key: source_key,
                    });
                }
            }
        }
    }
    references
}

pub fn create_structure_references_for_chunk(
    access_by_chunk: &BTreeMap<i64, StructureAccessModel>,
    target_chunk: ChunkPos,
    target_access: &mut StructureAccessModel,
) -> Vec<StructureReferenceModel> {
    let references = structure_references_for_chunk(access_by_chunk, target_chunk);
    for reference in &references {
        target_access.add_reference_for_structure(reference.structure, reference.source_chunk_key);
    }
    references
}

