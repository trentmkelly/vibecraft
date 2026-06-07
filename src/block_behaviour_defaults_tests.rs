use crate::block_behavior::InteractionResult;
use crate::block_behaviour_defaults::{
    cache_large_collision_shape, calculate_solid, can_be_replaced_by_fluid,
    can_be_replaced_by_item, face_sturdy_table, face_support_index, get_destroy_progress,
    get_light_dampening, get_seed, get_shade_brightness, is_pathfindable,
    occlusion_faces_for_shape, plan_on_explosion_hit, plan_update_neighbour_shapes,
    propagates_skylight_down, BlockBehaviourDefaults, BlockStateBaseModel, DelegatedBlockCallModel,
    ExplosionActionModel, ExplosionBlockInteractionModel, ExplosionHitPlanInput,
    JavaDirectionModel, OcclusionFacesModel, PathComputationTypeModel, RenderShapeModel,
    ShapeModel, SupportTypeModel, BLOCK_BEHAVIOUR_UPDATE_SHAPE_ORDER,
};
use crate::block_behaviour_properties::{
    BlockBehaviourPropertiesModel, BlockOffsetType, NoteBlockInstrumentModel, PushReactionModel,
    StatePredicateModel,
};
use crate::block_update::BlockPos;
use crate::fluid::FluidKind;

#[test]
fn block_behaviour_default_returns_match_java_noop_methods() {
    let behaviour = BlockBehaviourDefaults::new(true, true, "stone", 6.0, 1.5);
    assert_eq!(behaviour.update_shape("state"), "state");
    assert!(!behaviour.skip_rendering());
    assert_eq!(behaviour.use_without_item(), InteractionResult::Pass);
    assert_eq!(behaviour.use_item_on(), InteractionResult::TryWithEmptyHand);
    assert!(!behaviour.trigger_event());
    assert_eq!(behaviour.render_shape(), RenderShapeModel::Model);
    assert!(!behaviour.use_shape_for_light_occlusion());
    assert!(!behaviour.is_signal_source());
    assert!(!behaviour.has_analog_output_signal());
    assert!(!behaviour.should_changed_state_keep_block_entity());
    assert!(behaviour.can_survive());
    assert_eq!(behaviour.analog_output_signal(), 0);
    assert_eq!(behaviour.signal(), 0);
    assert_eq!(behaviour.direct_signal(), 0);
}

#[test]
fn block_behaviour_constructor_backed_defaults_match_java_fields() {
    let behaviour = BlockBehaviourDefaults::new(false, true, "wood", 2.5, 2.0);
    assert!(behaviour.is_randomly_ticking());
    assert_eq!(behaviour.get_sound_type(), "wood");
    assert_eq!(behaviour.default_destroy_time(), 2.0);
    assert_eq!(behaviour.explosion_resistance, 2.5);
    assert_eq!(behaviour.max_horizontal_offset(), 0.25);
    assert_eq!(behaviour.max_vertical_offset(), 0.2);
    assert_eq!(
        behaviour.get_collision_shape(ShapeModel::Block),
        ShapeModel::Empty
    );
}

#[test]
fn block_behaviour_pathfinding_replaceability_and_shapes_match_java_defaults() {
    assert!(!is_pathfindable(PathComputationTypeModel::Land, true, None));
    assert!(is_pathfindable(PathComputationTypeModel::Land, false, None));
    assert!(!is_pathfindable(PathComputationTypeModel::Air, true, None));
    assert!(is_pathfindable(PathComputationTypeModel::Air, false, None));
    assert!(is_pathfindable(
        PathComputationTypeModel::Water,
        true,
        Some(FluidKind::Water)
    ));
    assert!(!is_pathfindable(
        PathComputationTypeModel::Water,
        false,
        Some(FluidKind::Lava)
    ));

    assert!(can_be_replaced_by_item(true, true, true));
    assert!(can_be_replaced_by_item(true, false, false));
    assert!(!can_be_replaced_by_item(true, false, true));
    assert!(!can_be_replaced_by_item(false, true, false));
    assert!(can_be_replaced_by_fluid(false, false));
    assert!(can_be_replaced_by_fluid(true, true));
    assert!(!can_be_replaced_by_fluid(false, true));
}

#[test]
fn block_behaviour_light_shade_and_skylight_defaults_match_java() {
    assert_eq!(get_light_dampening(true, true), 15);
    assert_eq!(get_light_dampening(false, true), 0);
    assert_eq!(get_light_dampening(false, false), 1);
    assert_eq!(get_shade_brightness(true), 0.2);
    assert_eq!(get_shade_brightness(false), 1.0);
    assert!(!propagates_skylight_down(true, true));
    assert!(!propagates_skylight_down(false, false));
    assert!(propagates_skylight_down(false, true));
}

#[test]
fn block_behaviour_destroy_progress_and_seed_match_java_formulas() {
    assert_eq!(get_destroy_progress(-1.0, 8.0, true), 0.0);
    assert!((get_destroy_progress(1.5, 8.0, true) - 0.17777778).abs() < 1e-7);
    assert!((get_destroy_progress(1.5, 8.0, false) - 0.053333335).abs() < 1e-7);
    assert_eq!(
        get_seed(BlockPos { x: 1, y: 64, z: 2 }),
        -117_935_115_545_999
    );
}

#[test]
fn block_state_base_solid_and_cache_helpers_match_java_cache_rules() {
    assert!(calculate_solid(true, false, false, ShapeModel::Empty));
    assert!(!calculate_solid(false, true, true, ShapeModel::Block));
    assert!(!calculate_solid(false, false, false, ShapeModel::Block));
    assert!(!calculate_solid(false, false, true, ShapeModel::Empty));
    assert!(calculate_solid(false, false, true, ShapeModel::Block));
    assert!(calculate_solid(
        false,
        false,
        true,
        ShapeModel::Custom {
            full_block: false,
            bounds_size_large_enough_for_solid: true,
            y_size_full: false,
            extends_outside_block: false,
        }
    ));
    assert!(!cache_large_collision_shape(ShapeModel::Block));
    assert!(cache_large_collision_shape(ShapeModel::Custom {
        full_block: false,
        bounds_size_large_enough_for_solid: false,
        y_size_full: false,
        extends_outside_block: true,
    }));
}

#[test]
fn block_state_base_constructor_copies_java_property_fields() {
    let properties = BlockBehaviourPropertiesModel::of()
        .map_color("wood")
        .light_level(9)
        .strength(2.0, 3.0)
        .requires_correct_tool_for_drops()
        .push_reaction(PushReactionModel::Block)
        .air()
        .ignited_by_lava()
        .liquid()
        .no_occlusion()
        .redstone_conductor_predicate(StatePredicateModel::Custom)
        .suffocating_predicate(StatePredicateModel::False)
        .view_blocking_predicate(StatePredicateModel::False)
        .emissive_rendering(StatePredicateModel::Custom)
        .offset_type(BlockOffsetType::Xz)
        .no_terrain_particles()
        .instrument(NoteBlockInstrumentModel::Bell)
        .replaceable();
    let state = BlockStateBaseModel::from_properties("minecraft:test", &properties, true);

    assert_eq!(state.owner_id, "minecraft:test");
    assert_eq!(state.get_light_emission(), 9);
    assert!(state.use_shape_for_light_occlusion);
    assert!(state.is_air);
    assert!(state.ignited_by_lava);
    assert!(state.liquid);
    assert_eq!(state.get_piston_push_reaction(), PushReactionModel::Block);
    assert_eq!(state.map_color, "wood");
    assert_eq!(state.destroy_speed, 2.0);
    assert!(state.requires_correct_tool_for_drops);
    assert!(!state.can_occlude);
    assert_eq!(state.is_redstone_conductor, StatePredicateModel::Custom);
    assert_eq!(state.is_suffocating, StatePredicateModel::False);
    assert_eq!(state.is_view_blocking, StatePredicateModel::False);
    assert_eq!(state.emissive_rendering, StatePredicateModel::Custom);
    assert!(state.has_offset_function());
    assert!(!state.should_spawn_terrain_particles());
    assert_eq!(state.instrument, NoteBlockInstrumentModel::Bell);
    assert!(state.can_be_replaced());
}

#[test]
fn block_state_base_init_cache_matches_java_cached_state_fields() {
    let properties = BlockBehaviourPropertiesModel::of();
    let mut state = BlockStateBaseModel::from_properties("minecraft:stone", &properties, false);
    state.init_cache(
        &properties,
        false,
        ShapeModel::Block,
        None,
        true,
        ShapeModel::Block,
    );

    assert_eq!(state.fluid, None);
    assert!(state.is_randomly_ticking);
    assert_eq!(
        state.get_collision_shape(ShapeModel::Empty),
        ShapeModel::Block
    );
    assert!(state.is_solid());
    assert!(state.blocks_motion());
    assert_eq!(state.occlusion_shape, ShapeModel::Block);
    assert!(state.solid_render);
    assert_eq!(state.occlusion_faces, OcclusionFacesModel::FullBlock);
    assert!(!state.propagates_skylight_down);
    assert_eq!(state.light_dampening, 15);
    assert!(!state.has_large_collision_shape());
}

#[test]
fn block_state_base_dynamic_shape_cache_absence_matches_java() {
    let properties = BlockBehaviourPropertiesModel::of();
    let mut state =
        BlockStateBaseModel::from_properties("minecraft:moving_piston", &properties, false);
    state.init_cache(
        &properties,
        true,
        ShapeModel::Block,
        Some(FluidKind::Water),
        false,
        ShapeModel::Empty,
    );

    assert!(state.cache.is_none());
    assert!(!state.is_solid());
    assert!(state.has_large_collision_shape());
    assert_eq!(
        state.get_collision_shape(ShapeModel::Custom {
            full_block: false,
            bounds_size_large_enough_for_solid: false,
            y_size_full: false,
            extends_outside_block: true,
        }),
        ShapeModel::Custom {
            full_block: false,
            bounds_size_large_enough_for_solid: false,
            y_size_full: false,
            extends_outside_block: true,
        }
    );
}

#[test]
fn block_state_base_blocks_motion_and_occlusion_face_rules_match_java() {
    let properties = BlockBehaviourPropertiesModel::of();
    let mut cobweb = BlockStateBaseModel::from_properties("minecraft:cobweb", &properties, false);
    cobweb.init_cache(
        &properties,
        false,
        ShapeModel::Block,
        None,
        false,
        ShapeModel::Block,
    );
    assert!(cobweb.is_solid());
    assert!(!cobweb.blocks_motion());

    let mut bamboo =
        BlockStateBaseModel::from_properties("minecraft:bamboo_sapling", &properties, false);
    bamboo.init_cache(
        &properties,
        false,
        ShapeModel::Block,
        None,
        false,
        ShapeModel::Block,
    );
    assert!(bamboo.is_solid());
    assert!(!bamboo.blocks_motion());

    assert_eq!(
        occlusion_faces_for_shape(ShapeModel::Empty),
        OcclusionFacesModel::Empty
    );
    assert_eq!(
        occlusion_faces_for_shape(ShapeModel::Block),
        OcclusionFacesModel::FullBlock
    );
    assert_eq!(
        occlusion_faces_for_shape(ShapeModel::Custom {
            full_block: false,
            bounds_size_large_enough_for_solid: false,
            y_size_full: false,
            extends_outside_block: false,
        }),
        OcclusionFacesModel::PerFace
    );
}

#[test]
fn block_state_base_offset_accessor_matches_java_state_offset() {
    let properties = BlockBehaviourPropertiesModel::of().offset_type(BlockOffsetType::Xz);
    let state = BlockStateBaseModel::from_properties("minecraft:grass", &properties, false);
    assert_eq!(
        state.get_offset(BlockPos { x: 1, y: 99, z: 2 }),
        (-0.11666666666666667, 0.0, -0.18333333333333335)
    );
}

#[test]
fn block_behaviour_delegation_explosion_plan_matches_java_order_and_gates() {
    let pos = BlockPos { x: 3, y: 64, z: -2 };
    assert!(plan_on_explosion_hit(ExplosionHitPlanInput {
        state_is_air: true,
        block_interaction: ExplosionBlockInteractionModel::Destroy,
        drop_from_explosion: true,
        has_block_entity: true,
        has_direct_source_entity: true,
        indirect_source_is_player: true,
        drops: &["minecraft:stone"],
        pos,
    })
    .is_empty());
    assert!(plan_on_explosion_hit(ExplosionHitPlanInput {
        state_is_air: false,
        block_interaction: ExplosionBlockInteractionModel::TriggerBlock,
        drop_from_explosion: true,
        has_block_entity: true,
        has_direct_source_entity: true,
        indirect_source_is_player: true,
        drops: &["minecraft:stone"],
        pos,
    })
    .is_empty());

    assert_eq!(
        plan_on_explosion_hit(ExplosionHitPlanInput {
            state_is_air: false,
            block_interaction: ExplosionBlockInteractionModel::DestroyWithDecay,
            drop_from_explosion: true,
            has_block_entity: true,
            has_direct_source_entity: true,
            indirect_source_is_player: true,
            drops: &["minecraft:cobblestone", "minecraft:gravel"],
            pos,
        }),
        vec![
            ExplosionActionModel::SpawnAfterBreak {
                drop_experience_hack: true
            },
            ExplosionActionModel::GetDrops {
                include_block_entity: true,
                include_this_entity: true,
                include_explosion_radius: true,
            },
            ExplosionActionModel::EmitDrop {
                item: "minecraft:cobblestone",
                pos,
            },
            ExplosionActionModel::EmitDrop {
                item: "minecraft:gravel",
                pos,
            },
            ExplosionActionModel::SetAir { flags: 3 },
            ExplosionActionModel::WasExploded,
        ]
    );

    assert_eq!(
        plan_on_explosion_hit(ExplosionHitPlanInput {
            state_is_air: false,
            block_interaction: ExplosionBlockInteractionModel::Keep,
            drop_from_explosion: false,
            has_block_entity: false,
            has_direct_source_entity: false,
            indirect_source_is_player: false,
            drops: &["minecraft:ignored"],
            pos,
        }),
        vec![
            ExplosionActionModel::SetAir { flags: 3 },
            ExplosionActionModel::WasExploded,
        ]
    );
}

#[test]
fn block_behaviour_delegation_surface_matches_java_forwarders() {
    let state = BlockStateBaseModel::from_properties(
        "minecraft:stone",
        &BlockBehaviourPropertiesModel::of(),
        false,
    );
    assert_eq!(
        state.delegate_update_indirect_neighbour_shapes(512),
        DelegatedBlockCallModel::UpdateIndirectNeighbourShapes { update_limit: 512 }
    );
    assert_eq!(
        state.delegate_trigger_event(4, 9),
        DelegatedBlockCallModel::TriggerEvent { b0: 4, b1: 9 }
    );
    assert_eq!(
        state.delegate_neighbor_changed(true),
        DelegatedBlockCallModel::HandleNeighborChanged {
            moved_by_piston: true
        }
    );
    assert_eq!(state.delegate_tick(), DelegatedBlockCallModel::Tick);
    assert_eq!(
        state.delegate_random_tick(),
        DelegatedBlockCallModel::RandomTick
    );
    assert_eq!(
        state.delegate_entity_inside(false),
        DelegatedBlockCallModel::EntityInside { is_precise: false }
    );
    assert_eq!(
        state.delegate_spawn_after_break(true),
        DelegatedBlockCallModel::SpawnAfterBreak {
            drop_experience: true
        }
    );
    assert_eq!(
        state.delegate_get_drops(),
        DelegatedBlockCallModel::GetDrops
    );
    assert_eq!(
        state.delegate_use_item_on(),
        DelegatedBlockCallModel::UseItemOn
    );
    assert_eq!(
        state.delegate_use_without_item(),
        DelegatedBlockCallModel::UseWithoutItem
    );
    assert_eq!(state.delegate_attack(), DelegatedBlockCallModel::Attack);
    assert_eq!(
        state.delegate_update_shape(),
        DelegatedBlockCallModel::UpdateShape
    );
    assert_eq!(
        state.delegate_can_be_replaced_by_item(),
        DelegatedBlockCallModel::CanBeReplacedByItem
    );
    assert_eq!(
        state.delegate_can_be_replaced_by_fluid(),
        DelegatedBlockCallModel::CanBeReplacedByFluid
    );
    assert_eq!(
        state.delegate_can_survive(),
        DelegatedBlockCallModel::CanSurvive
    );
    assert_eq!(
        state.delegate_get_menu_provider(),
        DelegatedBlockCallModel::GetMenuProvider
    );
    assert_eq!(
        state.delegate_get_ticker(),
        DelegatedBlockCallModel::GetTicker
    );
    assert_eq!(
        state.delegate_get_clone_item_stack(true),
        DelegatedBlockCallModel::GetCloneItemStack { include_data: true }
    );
    assert_eq!(
        state.delegate_on_projectile_hit(),
        DelegatedBlockCallModel::OnProjectileHit
    );
}

#[test]
fn block_behaviour_delegation_neighbour_shape_update_order_matches_java() {
    assert_eq!(
        BLOCK_BEHAVIOUR_UPDATE_SHAPE_ORDER,
        [
            JavaDirectionModel::West,
            JavaDirectionModel::East,
            JavaDirectionModel::North,
            JavaDirectionModel::South,
            JavaDirectionModel::Down,
            JavaDirectionModel::Up,
        ]
    );
    let plan = plan_update_neighbour_shapes(512);
    assert_eq!(plan.len(), 6);
    assert_eq!(plan[0].direction_to_neighbor, JavaDirectionModel::West);
    assert_eq!(plan[0].direction_from_neighbor, JavaDirectionModel::East);
    assert_eq!(plan[4].direction_to_neighbor, JavaDirectionModel::Down);
    assert_eq!(plan[4].direction_from_neighbor, JavaDirectionModel::Up);
    assert!(plan.iter().all(|entry| entry.update_limit == 512));
}

#[test]
fn block_behaviour_delegation_face_support_cache_matches_java_indexing() {
    assert_eq!(
        face_support_index(JavaDirectionModel::Down, SupportTypeModel::Full),
        0
    );
    assert_eq!(
        face_support_index(JavaDirectionModel::East, SupportTypeModel::Rigid),
        17
    );

    let full_table = face_sturdy_table(ShapeModel::Block);
    assert!(full_table[face_support_index(JavaDirectionModel::North, SupportTypeModel::Center)]);
    let empty_table = face_sturdy_table(ShapeModel::Empty);
    assert!(!empty_table[face_support_index(JavaDirectionModel::South, SupportTypeModel::Rigid)]);

    let properties = BlockBehaviourPropertiesModel::of();
    let mut cached = BlockStateBaseModel::from_properties("minecraft:stone", &properties, false);
    cached.init_cache(
        &properties,
        false,
        ShapeModel::Block,
        None,
        false,
        ShapeModel::Block,
    );
    assert!(cached.is_face_sturdy_cached(JavaDirectionModel::Up, SupportTypeModel::Full, false));
    assert!(cached.is_collision_shape_full_block(false));

    let mut uncached =
        BlockStateBaseModel::from_properties("minecraft:dynamic", &properties, false);
    uncached.init_cache(
        &properties,
        true,
        ShapeModel::Empty,
        None,
        false,
        ShapeModel::Empty,
    );
    assert!(uncached.is_face_sturdy_cached(JavaDirectionModel::Up, SupportTypeModel::Full, true));
    assert!(uncached.is_collision_shape_full_block(true));
}
