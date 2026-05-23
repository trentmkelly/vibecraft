use super::*;

pub fn jigsaw_expansion_hack_target_size(
    do_expansion_hack: bool,
    hack_box: StructureBoundingBoxModel,
    target_jigsaws: &[JigsawLocalConnectorModel],
    pools: &[JigsawPoolSizeModel],
    pool_alias_lookup: &JigsawPoolAliasLookupModel,
) -> i32 {
    if !do_expansion_hack || hack_box.max_y - hack_box.min_y + 1 > 16 {
        return 0;
    }

    target_jigsaws
        .iter()
        .filter_map(|target_jigsaw| {
            let facing_step = target_jigsaw.connector.front.step();
            let facing_pos = BlockPos {
                x: target_jigsaw.local_pos.x + facing_step.x,
                y: target_jigsaw.local_pos.y + facing_step.y,
                z: target_jigsaw.local_pos.z + facing_step.z,
            };
            if !hack_box.is_inside(facing_pos) {
                return None;
            }

            let child_pool_name = pool_alias_lookup.lookup(target_jigsaw.connector.pool);
            let child_pool = pools.iter().find(|pool| pool.name == child_pool_name);
            let child_pool_size = child_pool.map_or(0, |pool| pool.max_size);
            let child_fallback_size = child_pool
                .and_then(|pool| pool.fallback)
                .and_then(|fallback_name| pools.iter().find(|pool| pool.name == fallback_name))
                .map_or(0, |pool| pool.max_size);
            Some(child_pool_size.max(child_fallback_size))
        })
        .max()
        .unwrap_or(0)
}

pub fn jigsaw_pool_availability_decision(
    target_pool_exists: bool,
    target_pool_size: usize,
    fallback_pool_name: &'static str,
    fallback_pool_size: usize,
) -> JigsawPoolAvailabilityDecisionModel {
    if !target_pool_exists || target_pool_size == 0 {
        return JigsawPoolAvailabilityDecisionModel {
            can_place_children: false,
            warning: Some(JigsawPoolAvailabilityWarning::EmptyOrNonExistentTarget),
        };
    }
    if fallback_pool_size == 0 && fallback_pool_name != "minecraft:empty" {
        return JigsawPoolAvailabilityDecisionModel {
            can_place_children: false,
            warning: Some(JigsawPoolAvailabilityWarning::EmptyOrNonExistentFallback),
        };
    }
    JigsawPoolAvailabilityDecisionModel {
        can_place_children: true,
        warning: None,
    }
}

pub fn jigsaw_candidate_elements_in_iteration_order(
    target_pool: &JigsawTemplatePoolModel,
    fallback_pool: &JigsawTemplatePoolModel,
    depth: i32,
    max_depth: i32,
    target_expanded_indices: &[usize],
    fallback_expanded_indices: &[usize],
) -> Vec<JigsawCandidateElementModel> {
    let mut candidates = Vec::new();
    if depth != max_depth {
        append_jigsaw_candidate_elements(
            &mut candidates,
            target_pool,
            JigsawCandidatePoolSource::Target,
            target_expanded_indices,
        );
    }
    append_jigsaw_candidate_elements(
        &mut candidates,
        fallback_pool,
        JigsawCandidatePoolSource::Fallback,
        fallback_expanded_indices,
    );
    candidates
}

fn append_jigsaw_candidate_elements(
    candidates: &mut Vec<JigsawCandidateElementModel>,
    pool: &JigsawTemplatePoolModel,
    source: JigsawCandidatePoolSource,
    expanded_indices: &[usize],
) {
    for expanded_index in expanded_indices {
        let Some((raw_template_index, element)) = pool.element_at_expanded_index(*expanded_index)
        else {
            continue;
        };
        if element.element_type == JigsawPoolElementTypeModel::Empty {
            break;
        }
        candidates.push(JigsawCandidateElementModel {
            source,
            raw_template_index,
            element,
        });
    }
}

impl PoolElementStructurePieceModel {
    pub const DEFAULT_LIQUID_SETTINGS: LiquidSettingsModel = LiquidSettingsModel::ApplyWaterlogging;

    pub fn new(
        element: JigsawPoolElementModel,
        position: (i32, i32, i32),
        ground_level_delta: i32,
        rotation: StructurePieceRotation,
        bounding_box: StructureBoundingBoxModel,
        liquid_settings: LiquidSettingsModel,
    ) -> Self {
        Self {
            element,
            position,
            ground_level_delta,
            rotation,
            bounding_box,
            liquid_settings,
            junctions: Vec::new(),
        }
    }

    pub fn add_junction(&mut self, junction: JigsawJunctionModel) {
        self.junctions.push(junction);
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) {
        self.bounding_box = self.bounding_box.moved(dx, dy, dz);
        self.position = (
            self.position.0 + dx,
            self.position.1 + dy,
            self.position.2 + dz,
        );
    }

    pub fn save_tag(&self) -> PoolElementStructurePieceTagModel {
        PoolElementStructurePieceTagModel {
            pos_x: self.position.0,
            pos_y: self.position.1,
            pos_z: self.position.2,
            ground_level_delta: self.ground_level_delta,
            rotation: self.rotation,
            junctions: self
                .junctions
                .iter()
                .map(JigsawJunctionModel::serialize)
                .collect(),
            liquid_settings: (self.liquid_settings != Self::DEFAULT_LIQUID_SETTINGS)
                .then_some(self.liquid_settings),
        }
    }
}

pub fn jigsaw_child_placement_y(
    source_projection: JigsawProjectionModel,
    target_projection: JigsawProjectionModel,
    source_box_y: i32,
    source_jigsaw_local_y: i32,
    target_jigsaw_local_y: i32,
    source_direction_step_y: i32,
    source_ground_level_delta: i32,
    target_ground_level_delta: i32,
    source_jigsaw_base_height: i32,
) -> JigsawChildPlacementYModel {
    let source_rigid = source_projection == JigsawProjectionModel::Rigid;
    let target_rigid = target_projection == JigsawProjectionModel::Rigid;
    let delta_y = source_jigsaw_local_y - target_jigsaw_local_y + source_direction_step_y;
    let target_box_y = if source_rigid && target_rigid {
        source_box_y + delta_y
    } else {
        source_jigsaw_base_height - target_jigsaw_local_y
    };
    let target_ground_level_delta = if target_rigid {
        source_ground_level_delta - delta_y
    } else {
        target_ground_level_delta
    };
    let (junction_y, case) = if source_rigid {
        (
            source_box_y + source_jigsaw_local_y,
            if target_rigid {
                JigsawJunctionYOffsetCase::BothRigid
            } else {
                JigsawJunctionYOffsetCase::SourceRigid
            },
        )
    } else if target_rigid {
        (
            target_box_y + target_jigsaw_local_y,
            JigsawJunctionYOffsetCase::TargetRigid,
        )
    } else {
        (
            source_jigsaw_base_height + delta_y / 2,
            JigsawJunctionYOffsetCase::BothTerrainMatching,
        )
    };

    JigsawChildPlacementYModel {
        target_box_y,
        target_ground_level_delta,
        junction_y,
        case,
    }
}

pub fn jigsaw_child_box_placement(
    target_jigsaw_world_pos: BlockPos,
    target_jigsaw_local_pos: BlockPos,
    raw_target_bounding_box_at_origin: StructureBoundingBoxModel,
    placement: JigsawChildPlacementYModel,
) -> JigsawChildBoxPlacementModel {
    let raw_target_box_pos = BlockPos {
        x: target_jigsaw_world_pos.x - target_jigsaw_local_pos.x,
        y: target_jigsaw_world_pos.y - target_jigsaw_local_pos.y,
        z: target_jigsaw_world_pos.z - target_jigsaw_local_pos.z,
    };
    let raw_target_bounding_box = raw_target_bounding_box_at_origin.moved(
        raw_target_box_pos.x,
        raw_target_box_pos.y,
        raw_target_box_pos.z,
    );
    let y_offset = placement.target_box_y - raw_target_bounding_box.min_y;
    let target_box_position = BlockPos {
        x: raw_target_box_pos.x,
        y: raw_target_box_pos.y + y_offset,
        z: raw_target_box_pos.z,
    };
    let target_bounding_box = raw_target_bounding_box.moved(0, y_offset, 0);

    JigsawChildBoxPlacementModel {
        raw_target_box_pos,
        raw_target_bounding_box,
        y_offset,
        target_box_position,
        target_bounding_box,
    }
}

pub fn jigsaw_apply_expansion_hack_to_target_box(
    target_bounding_box: StructureBoundingBoxModel,
    expand_to: i32,
) -> StructureBoundingBoxModel {
    if expand_to <= 0 {
        return target_bounding_box;
    }

    let new_size = (expand_to + 1).max(target_bounding_box.max_y - target_bounding_box.min_y);
    target_bounding_box.encapsulate_pos(BlockPos {
        x: target_bounding_box.min_x,
        y: target_bounding_box.min_y + new_size,
        z: target_bounding_box.min_z,
    })
}

pub fn jigsaw_child_free_shape_selection(
    source_bounding_box: StructureBoundingBoxModel,
    target_jigsaw_pos: BlockPos,
    source_shape_already_initialized: bool,
) -> JigsawChildFreeShapeSelectionModel {
    if source_bounding_box.is_inside(target_jigsaw_pos) {
        JigsawChildFreeShapeSelectionModel {
            scope: JigsawChildFreeShapeScope::SourcePiece,
            initialized_source_shape: (!source_shape_already_initialized)
                .then_some(source_bounding_box),
        }
    } else {
        JigsawChildFreeShapeSelectionModel {
            scope: JigsawChildFreeShapeScope::Context,
            initialized_source_shape: None,
        }
    }
}

pub fn jigsaw_accepted_child_scheduling(
    depth: i32,
    max_depth: i32,
    placement_priority: i32,
) -> JigsawAcceptedChildSchedulingModel {
    let child_depth = depth + 1;
    JigsawAcceptedChildSchedulingModel {
        child_depth,
        queue_for_expansion: child_depth <= max_depth,
        placement_priority,
    }
}

pub fn jigsaw_source_junction(
    target_jigsaw_pos: (i32, i32, i32),
    placement: JigsawChildPlacementYModel,
    source_jigsaw_local_y: i32,
    source_ground_level_delta: i32,
    delta_y: i32,
    target_projection: JigsawProjectionModel,
) -> JigsawJunctionModel {
    JigsawJunctionModel {
        source_x: target_jigsaw_pos.0,
        source_ground_y: placement.junction_y - source_jigsaw_local_y + source_ground_level_delta,
        source_z: target_jigsaw_pos.2,
        delta_y,
        dest_projection: target_projection,
    }
}

pub fn jigsaw_target_junction(
    source_jigsaw_pos: (i32, i32, i32),
    placement: JigsawChildPlacementYModel,
    target_jigsaw_local_y: i32,
    target_ground_level_delta: i32,
    delta_y: i32,
    source_projection: JigsawProjectionModel,
) -> JigsawJunctionModel {
    JigsawJunctionModel {
        source_x: source_jigsaw_pos.0,
        source_ground_y: placement.junction_y - target_jigsaw_local_y + target_ground_level_delta,
        source_z: source_jigsaw_pos.2,
        delta_y: -delta_y,
        dest_projection: source_projection,
    }
}

pub fn jigsaw_start_too_close_to_world_height_limits(
    min_y: i32,
    height: i32,
    dimension_padding: DimensionPaddingModel,
    center_piece_box: StructureBoundingBoxModel,
) -> bool {
    if dimension_padding == DimensionPaddingModel::ZERO {
        return false;
    }
    let max_y = min_y + height - 1;
    let min_y_with_padding = min_y + dimension_padding.bottom;
    let max_y_with_padding = max_y - dimension_padding.top;
    center_piece_box.min_y < min_y_with_padding || center_piece_box.max_y > max_y_with_padding
}

pub fn jigsaw_initial_expansion_bounds(
    center_x: i32,
    center_y: i32,
    center_z: i32,
    max_distance_from_center: JigsawMaxDistanceModel,
    min_y: i32,
    height: i32,
    dimension_padding: DimensionPaddingModel,
) -> JigsawExpansionBoundsModel {
    let max_y = min_y + height - 1;
    JigsawExpansionBoundsModel {
        min_x: center_x - max_distance_from_center.horizontal,
        min_y: (center_y - max_distance_from_center.vertical).max(min_y + dimension_padding.bottom),
        min_z: center_z - max_distance_from_center.horizontal,
        max_x_exclusive: center_x + max_distance_from_center.horizontal + 1,
        max_y_exclusive: (center_y + max_distance_from_center.vertical + 1)
            .min(max_y + 1 - dimension_padding.top),
        max_z_exclusive: center_z + max_distance_from_center.horizontal + 1,
    }
}

pub fn jigsaw_start_anchor_adjustment(
    requested_position: BlockPos,
    anchored_position: BlockPos,
) -> JigsawStartAnchorAdjustmentModel {
    let local_anchor = BlockPos {
        x: anchored_position.x - requested_position.x,
        y: anchored_position.y - requested_position.y,
        z: anchored_position.z - requested_position.z,
    };
    let adjusted_position = BlockPos {
        x: requested_position.x - local_anchor.x,
        y: requested_position.y - local_anchor.y,
        z: requested_position.z - local_anchor.z,
    };
    JigsawStartAnchorAdjustmentModel {
        local_anchor,
        adjusted_position,
    }
}

impl JigsawStructureModel {
    pub const MAX_TOTAL_STRUCTURE_RANGE: i32 = 128;
    pub const MIN_DEPTH: i32 = 0;
    pub const MAX_DEPTH: i32 = 20;

    pub fn new(
        start_pool: &'static str,
        max_depth: i32,
        start_height: HeightProvider,
        use_expansion_hack: bool,
        terrain_adjustment: TerrainAdjustmentModel,
    ) -> Result<Self, String> {
        Self::new_full(
            start_pool,
            None,
            max_depth,
            start_height,
            use_expansion_hack,
            None,
            JigsawMaxDistanceModel::DEFAULT,
            Vec::new(),
            DimensionPaddingModel::ZERO,
            LiquidSettingsModel::ApplyWaterlogging,
            terrain_adjustment,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        start_pool: &'static str,
        start_jigsaw_name: Option<&'static str>,
        max_depth: i32,
        start_height: HeightProvider,
        use_expansion_hack: bool,
        project_start_to_heightmap: Option<&'static str>,
        max_distance_from_center: JigsawMaxDistanceModel,
        pool_aliases: Vec<JigsawPoolAliasBindingModel>,
        dimension_padding: DimensionPaddingModel,
        liquid_settings: LiquidSettingsModel,
        terrain_adjustment: TerrainAdjustmentModel,
    ) -> Result<Self, String> {
        if !(Self::MIN_DEPTH..=Self::MAX_DEPTH).contains(&max_depth) {
            return Err("jigsaw structure size must be in 0..=20".to_string());
        }
        let structure = Self {
            start_pool,
            start_jigsaw_name,
            max_depth,
            start_height,
            use_expansion_hack,
            project_start_to_heightmap,
            max_distance_from_center,
            pool_aliases,
            dimension_padding,
            liquid_settings,
            terrain_adjustment,
        };
        structure.verify_range()?;
        Ok(structure)
    }

    pub fn verify_range(&self) -> Result<(), String> {
        if self.max_distance_from_center.horizontal + self.terrain_adjustment.jigsaw_edge_needed()
            > Self::MAX_TOTAL_STRUCTURE_RANGE
        {
            Err(
                "Horizontal structure size including terrain adaptation must not exceed 128"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }

    pub fn find_generation_point(
        &self,
        chunk_pos: ChunkPos,
        height_context: WorldGenerationHeightContext,
        world_seed: i64,
        first_height_roll: i32,
        second_height_roll: i32,
        third_height_roll: i32,
    ) -> JigsawGenerationPointModel {
        let height = height_provider_sample_with_rolls(
            self.start_height,
            height_context,
            first_height_roll,
            second_height_roll,
            third_height_roll,
        );
        let start_pos = (chunk_pos.x * 16, height, chunk_pos.z * 16);
        JigsawGenerationPointModel {
            start_pos,
            pool_alias_lookup: JigsawPoolAliasLookupModel::create(
                &self.pool_aliases,
                start_pos,
                world_seed,
            ),
            max_depth: self.max_depth,
            use_expansion_hack: self.use_expansion_hack,
            project_start_to_heightmap: self.project_start_to_heightmap,
            max_distance_from_center: self.max_distance_from_center,
            dimension_padding: self.dimension_padding,
            liquid_settings: self.liquid_settings,
        }
    }
}

