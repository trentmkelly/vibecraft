use super::*;

    #[test]
    fn structure_processor_surfaces_and_core_decisions_match_vanilla_templatesystem() {
        assert_eq!(
            super::super::StructureProcessorTypeModel::REGISTRY_ORDER.map(|processor| processor.id()),
            [
                "minecraft:block_ignore",
                "minecraft:block_rot",
                "minecraft:gravity",
                "minecraft:jigsaw_replacement",
                "minecraft:rule",
                "minecraft:nop",
                "minecraft:block_age",
                "minecraft:blackstone_replace",
                "minecraft:lava_submerged_block",
                "minecraft:protected_blocks",
                "minecraft:capped",
            ]
        );
        assert_eq!(
            super::super::StructureRuleTestTypeModel::REGISTRY_ORDER.map(|rule| rule.id()),
            [
                "minecraft:always_true",
                "minecraft:block_match",
                "minecraft:blockstate_match",
                "minecraft:tag_match",
                "minecraft:random_block_match",
                "minecraft:random_blockstate_match",
            ]
        );
        assert_eq!(
            super::super::StructurePosRuleTestTypeModel::REGISTRY_ORDER.map(|rule| rule.id()),
            [
                "minecraft:always_true",
                "minecraft:linear_pos",
                "minecraft:axis_aligned_linear_pos",
            ]
        );

        let ignore = super::super::StructureProcessorModel::BlockIgnore {
            blocks: vec!["minecraft:air", "minecraft:structure_block"],
        };
        assert_eq!(ignore.codec_id(), "minecraft:block_ignore");
        assert!(ignore.block_ignore_should_drop("minecraft:air"));
        assert!(!ignore.block_ignore_should_drop("minecraft:stone"));

        let rot_all = super::super::StructureProcessorModel::BlockRot {
            rottable_blocks: None,
            integrity: 0.35,
        };
        assert!(rot_all.block_rot_keeps(false, 0.35));
        assert!(!rot_all.block_rot_keeps(false, 0.35001));
        let rot_tagged = super::super::StructureProcessorModel::BlockRot {
            rottable_blocks: Some("minecraft:replaceable"),
            integrity: 0.0,
        };
        assert!(rot_tagged.block_rot_keeps(false, 0.99));
        assert!(!rot_tagged.block_rot_keeps(true, 0.01));

        let jigsaw = super::super::StructureProcessorModel::JigsawReplacement;
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:oak_planks"),
                false
            ),
            Some("minecraft:oak_planks")
        );
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:structure_void"),
                false
            ),
            None
        );
        assert_eq!(
            jigsaw.jigsaw_replacement_output(
                "minecraft:jigsaw",
                Some("minecraft:oak_planks"),
                true
            ),
            Some("minecraft:jigsaw")
        );

        let gravity = super::super::StructureProcessorModel::Gravity {
            heightmap: "WORLD_SURFACE_WG",
            offset: -1,
        };
        assert_eq!(
            gravity.gravity_adjusted_y(80, 3, true),
            Some(("WORLD_SURFACE", 82))
        );
        assert_eq!(
            gravity.gravity_adjusted_y(80, 3, false),
            Some(("WORLD_SURFACE_WG", 82))
        );

        let lava = super::super::StructureProcessorModel::LavaSubmergedBlock;
        assert_eq!(
            lava.lava_submerged_output("minecraft:lava", false, "minecraft:chain"),
            "minecraft:lava"
        );
        assert_eq!(
            lava.lava_submerged_output("minecraft:lava", true, "minecraft:stone"),
            "minecraft:stone"
        );

        let capped = super::super::StructureProcessorModel::Capped {
            delegate: Box::new(super::super::StructureProcessorModel::Nop),
            limit: 2,
        };
        assert!(capped.capped_can_run(3, 3, 2));
        assert!(!capped.capped_can_run(3, 2, 2));
        assert!(!capped.capped_can_run(3, 3, 0));

        assert!(super::super::structure_random_rule_test_matches(true, 0.25, 0.249));
        assert!(!super::super::structure_random_rule_test_matches(true, 0.25, 0.25));
        assert_eq!(
            super::super::structure_linear_pos_chance(5, 0, 10, 0.2, 0.8),
            Ok(0.5)
        );
        assert_eq!(
            super::super::structure_linear_pos_chance(0, 4, 4, 0.0, 1.0),
            Err("Invalid range: [4,4]".to_string())
        );
        assert_eq!(
            super::super::structure_axis_aligned_distance((10, 65, -4), (3, 60, 1), 'x'),
            7
        );
        assert_eq!(
            super::super::structure_axis_aligned_distance((10, 65, -4), (3, 60, 1), 'z'),
            5
        );
    }

    #[test]
    fn rule_block_entity_modifiers_match_vanilla_template_rule_behavior() {
        assert_eq!(
            super::super::RuleBlockEntityModifierTypeModel::REGISTRY_ORDER.map(|modifier| modifier.id()),
            [
                "minecraft:clear",
                "minecraft:passthrough",
                "minecraft:append_static",
                "minecraft:append_loot",
            ]
        );

        let existing = super::super::TemplateCompoundTagModel::default()
            .with_string("id", "minecraft:chest")
            .with_string("CustomName", "{\"text\":\"Old\"}");
        let static_data = super::super::TemplateCompoundTagModel::default()
            .with_string("CustomName", "{\"text\":\"New\"}")
            .with_string("Lock", "key");

        let mut random =
            super::super::RandomSourceKind::new(12345, crate::random_source::RandomAlgorithm::Legacy);
        assert_eq!(
            super::super::RuleBlockEntityModifierModel::Passthrough
                .apply(&mut random, Some(existing.clone())),
            Some(existing.clone())
        );
        assert_eq!(
            super::super::RuleBlockEntityModifierModel::Passthrough.apply(&mut random, None),
            None
        );
        assert_eq!(
            super::super::RuleBlockEntityModifierModel::Clear.apply(&mut random, Some(existing.clone())),
            Some(super::super::TemplateCompoundTagModel::default())
        );

        let appended = super::super::RuleBlockEntityModifierModel::AppendStatic { data: static_data }
            .apply(&mut random, Some(existing.clone()))
            .expect("append static returns tag");
        assert_eq!(
            appended.values.get("id"),
            Some(&super::super::TemplateNbtValueModel::String("minecraft:chest"))
        );
        assert_eq!(
            appended.values.get("CustomName"),
            Some(&super::super::TemplateNbtValueModel::String("{\"text\":\"New\"}"))
        );
        assert_eq!(
            appended.values.get("Lock"),
            Some(&super::super::TemplateNbtValueModel::String("key"))
        );

        let mut loot_random =
            super::super::RandomSourceKind::new(12345, crate::random_source::RandomAlgorithm::Legacy);
        let loot = super::super::RuleBlockEntityModifierModel::AppendLoot {
            loot_table: "minecraft:chests/simple_dungeon",
        };
        assert_eq!(loot.codec_id(), "minecraft:append_loot");
        let loot_tag = loot
            .apply(&mut loot_random, Some(existing))
            .expect("append loot returns tag");
        assert_eq!(
            loot_tag.values.get("LootTable"),
            Some(&super::super::TemplateNbtValueModel::String(
                "minecraft:chests/simple_dungeon"
            ))
        );
        assert_eq!(
            loot_tag.values.get("LootTableSeed"),
            Some(&super::super::TemplateNbtValueModel::Long(6674089274190705457))
        );
    }

    #[test]
    fn structure_template_manager_paths_and_cache_follow_vanilla_loader_rules() {
        let factory = super::super::TemplatePathFactoryModel::new("/world/generated");
        let village = crate::registry::Identifier::parse(
            "minecraft:village/plains/houses/plains_small_house_1",
        )
        .expect("valid template id");
        assert_eq!(
            factory.create_and_validate_path_to_structure(
                &village,
                super::super::StructureTemplateFileKind::Nbt,
            ),
            Ok("/world/generated/minecraft/structure/village/plains/houses/plains_small_house_1.nbt"
                .to_string())
        );
        assert_eq!(
            factory.create_and_validate_path_to_structure(
                &village,
                super::super::StructureTemplateFileKind::Snbt,
            ),
            Ok("/world/generated/minecraft/structure/village/plains/houses/plains_small_house_1.snbt"
                .to_string())
        );

        let data_factory = super::super::TemplatePathFactoryModel::for_pack_type("/tmp/tests", "data");
        let resource =
            crate::registry::Identifier::parse("minecraft:structure/trial_chambers/start.nbt")
                .expect("valid resource path");
        assert_eq!(
            data_factory.create_and_validate_path_to_resource(&resource),
            Ok("/tmp/tests/data/minecraft/structure/trial_chambers/start.nbt".to_string())
        );

        let traversal = crate::registry::Identifier::parse("minecraft:structure/../bad.nbt")
            .expect("registry identifier allows dotted path segments");
        assert_eq!(
            factory.create_and_validate_path_to_resource(&traversal),
            Err(
                "Invalid file path 'minecraft:structure/../bad.nbt': invalid path segment '..'"
                    .to_string()
            )
        );
        let uppercase = crate::registry::Identifier::new("minecraft", "structure/Bad.nbt");
        assert!(uppercase.is_err());
        let dot_segment =
            crate::registry::Identifier::parse("minecraft:structure/./bad.nbt").unwrap();
        assert_eq!(
            factory.create_and_validate_path_to_resource(&dot_segment),
            Err(
                "Invalid file path 'minecraft:structure/./bad.nbt': invalid path segment '.'"
                    .to_string()
            )
        );

        assert_eq!(
            super::super::StructureTemplateManagerModel::save_kind(false),
            super::super::StructureTemplateFileKind::Nbt
        );
        assert_eq!(
            super::super::StructureTemplateManagerModel::save_kind(true),
            super::super::StructureTemplateFileKind::Snbt
        );

        let mut manager = super::super::StructureTemplateManagerModel::default();
        let missing = crate::registry::Identifier::parse("minecraft:missing").unwrap();
        assert_eq!(manager.get_or_try_load(missing.clone(), |_| None), None);
        assert_eq!(
            manager.get_or_try_load(missing.clone(), |_| Some("should_not_reload")),
            None
        );
        manager.remove(&missing);
        assert_eq!(
            manager.get_or_try_load(missing.clone(), |_| Some("loaded_after_remove")),
            Some("loaded_after_remove")
        );
        manager.on_resource_manager_reload();
        assert!(manager.cache.is_empty());
        assert_eq!(manager.get_or_create(missing), "runtime_template");
    }

    #[test]
    fn template_sources_load_in_vanilla_order_and_list_distinct_templates() {
        let id = crate::registry::Identifier::parse(
            "minecraft:village/plains/town_centers/plains_fountain_01",
        )
        .expect("valid template id");
        let generated = super::super::TemplateSourceModel::directory(
            Some("/world/generated"),
            false,
            &[(
                "minecraft:village/plains/town_centers/plains_fountain_01",
                "generated_nbt",
            )],
            &[],
        );
        let tests = super::super::TemplateSourceModel::directory(
            Some("/tmp/tests/data"),
            true,
            &[("minecraft:test/marker", "test_snbt")],
            &[],
        );
        let resources = super::super::TemplateSourceModel::resource_manager(
            &[
                (
                    "minecraft:village/plains/town_centers/plains_fountain_01",
                    "resource_pack_nbt",
                ),
                ("minecraft:bastion/starts/start", "bastion_nbt"),
            ],
            &[],
        );

        let mut manager = super::super::StructureTemplateManagerModel::default();
        let (loaded, attempts) = manager.try_load_from_sources(
            id.clone(),
            &[generated.clone(), tests.clone(), resources.clone()],
        );
        assert_eq!(loaded, Some("generated_nbt"));
        assert_eq!(
            attempts,
            vec![super::super::TemplateLoadAttemptModel {
                source_kind: super::super::TemplateSourceKindModel::Directory,
                id: id.clone(),
                result: super::super::TemplateLoadAttemptResultModel::Loaded,
            }]
        );

        let (cached, cached_attempts) =
            manager.try_load_from_sources(id.clone(), &[resources.clone()]);
        assert_eq!(cached, Some("generated_nbt"));
        assert!(cached_attempts.is_empty());

        let missing = crate::registry::Identifier::parse("minecraft:missing/template").unwrap();
        let unavailable = super::super::TemplateSourceModel::directory(None, false, &[], &[]);
        let failing_resource = super::super::TemplateSourceModel::resource_manager(
            &[("minecraft:missing/template", "would_have_loaded")],
            &["minecraft:missing/template"],
        );
        let (missing_result, missing_attempts) =
            manager.try_load_from_sources(missing.clone(), &[unavailable, failing_resource]);
        assert_eq!(missing_result, None);
        assert_eq!(
            missing_attempts
                .iter()
                .map(|attempt| attempt.result)
                .collect::<Vec<_>>(),
            vec![
                super::super::TemplateLoadAttemptResultModel::SourceUnavailable,
                super::super::TemplateLoadAttemptResultModel::ErrorSuppressed,
            ]
        );
        let (cached_missing, cached_missing_attempts) =
            manager.try_load_from_sources(missing, &[resources.clone()]);
        assert_eq!(cached_missing, None);
        assert!(cached_missing_attempts.is_empty());

        let listed = super::super::StructureTemplateManagerModel::list_templates_from_sources(&[
            generated,
            tests.clone(),
            resources,
        ]);
        assert_eq!(
            listed,
            vec![
                crate::registry::Identifier::parse(
                    "minecraft:village/plains/town_centers/plains_fountain_01"
                )
                .unwrap(),
                crate::registry::Identifier::parse("minecraft:test/marker").unwrap(),
                crate::registry::Identifier::parse("minecraft:bastion/starts/start").unwrap(),
            ]
        );
        assert!(tests.load_as_text);
    }

    #[test]
    fn jigsaw_structure_config_validation_and_generation_point_match_vanilla() {
        let start_height = super::super::HeightProvider::Constant {
            value: super::super::VerticalAnchor::Absolute(72),
        };
        let structure = super::super::JigsawStructureModel::new_full(
            "minecraft:village/plains/town_centers",
            Some("minecraft:town_centers"),
            7,
            start_height,
            true,
            Some("WORLD_SURFACE_WG"),
            super::super::JigsawMaxDistanceModel::new(80, 128).expect("valid max distance"),
            vec![super::super::JigsawPoolAliasBindingModel::Direct {
                alias: "minecraft:village/common/houses",
                target: "minecraft:village/plains/houses",
            }],
            super::super::DimensionPaddingModel { bottom: 4, top: 8 },
            super::super::LiquidSettingsModel::IgnoreWaterlogging,
            super::super::TerrainAdjustmentModel::BeardThin,
        )
        .expect("valid jigsaw structure config");
        assert_eq!(structure.verify_range(), Ok(()));

        let point = structure.find_generation_point(
            ChunkPos { x: -2, z: 3 },
            super::super::WorldGenerationHeightContext {
                min_y: -64,
                height: 384,
            },
            12345,
            0,
            0,
            0,
        );
        assert_eq!(point.start_pos, (-32, 72, 48));
        assert_eq!(point.max_depth, 7);
        assert!(point.use_expansion_hack);
        assert_eq!(point.project_start_to_heightmap, Some("WORLD_SURFACE_WG"));
        assert_eq!(
            point
                .pool_alias_lookup
                .lookup("minecraft:village/common/houses"),
            "minecraft:village/plains/houses"
        );
        assert_eq!(
            point.max_distance_from_center,
            super::super::JigsawMaxDistanceModel {
                horizontal: 80,
                vertical: 128,
            }
        );
        assert_eq!(
            point.dimension_padding,
            super::super::DimensionPaddingModel { bottom: 4, top: 8 }
        );
        assert_eq!(
            point.liquid_settings,
            super::super::LiquidSettingsModel::IgnoreWaterlogging
        );

        assert_eq!(
            super::super::JigsawStructureModel::new(
                "minecraft:empty",
                -1,
                start_height,
                false,
                super::super::TerrainAdjustmentModel::None,
            ),
            Err("jigsaw structure size must be in 0..=20".to_string())
        );
        assert_eq!(
            super::super::JigsawStructureModel::new_full(
                "minecraft:empty",
                None,
                0,
                start_height,
                false,
                None,
                super::super::JigsawMaxDistanceModel::new(128, 80).expect("valid max distance"),
                Vec::new(),
                super::super::DimensionPaddingModel::ZERO,
                super::super::LiquidSettingsModel::ApplyWaterlogging,
                super::super::TerrainAdjustmentModel::Bury,
            ),
            Err(
                "Horizontal structure size including terrain adaptation must not exceed 128"
                    .to_string()
            )
        );

        let compact = super::super::JigsawStructureModel::new(
            "minecraft:bastion/starts",
            3,
            start_height,
            false,
            super::super::TerrainAdjustmentModel::None,
        )
        .expect("compact constructor fills vanilla defaults");
        assert_eq!(compact.start_jigsaw_name, None);
        assert_eq!(
            compact.max_distance_from_center,
            super::super::JigsawMaxDistanceModel::DEFAULT
        );
        assert_eq!(
            compact.dimension_padding,
            super::super::DimensionPaddingModel::ZERO
        );
        assert_eq!(
            compact.liquid_settings,
            super::super::LiquidSettingsModel::ApplyWaterlogging
        );
    }

    #[test]
    fn structure_check_presence_and_lookup_branches_match_vanilla() {
        assert_eq!(
            super::super::structure_check_result_from_cached_references(None, false),
            super::super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::super::structure_check_result_from_cached_references(Some(0), true),
            super::super::StructureCheckResultModel::StartPresent
        );
        assert_eq!(
            super::super::structure_check_result_from_cached_references(Some(1), true),
            super::super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::super::structure_check_result_from_cached_references(Some(2), false),
            super::super::StructureCheckResultModel::StartPresent
        );

        assert!(!super::super::structure_fast_check_allows_lookup(
            super::super::StructureCheckResultModel::StartNotPresent
        ));
        assert!(super::super::structure_fast_check_allows_lookup(
            super::super::StructureCheckResultModel::ChunkLoadNeeded
        ));
        assert!(super::super::structure_locate_can_return_fast(
            super::super::StructureCheckResultModel::StartPresent,
            false
        ));
        assert!(!super::super::structure_locate_can_return_fast(
            super::super::StructureCheckResultModel::StartPresent,
            true
        ));
        assert!(!super::super::structure_locate_can_return_fast(
            super::super::StructureCheckResultModel::ChunkLoadNeeded,
            false
        ));

        let piece = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 20,
                min_z: 0,
                max_x: 15,
                max_y: 30,
                max_z: 15,
            },
        };
        let mut start = super::super::StructureStartModel {
            structure: Some("minecraft:desert_pyramid"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![piece],
        };
        assert!(super::super::structure_start_can_satisfy_lookup(&start, false));
        assert!(super::super::structure_start_can_satisfy_lookup(&start, true));
        assert!(super::super::structure_try_add_reference(&mut start));
        assert_eq!(start.references, 1);
        assert!(super::super::structure_start_can_satisfy_lookup(&start, false));
        assert!(!super::super::structure_start_can_satisfy_lookup(&start, true));
        assert!(!super::super::structure_try_add_reference(&mut start));

        let invalid = super::super::StructureStartModel::invalid();
        assert!(!super::super::structure_start_can_satisfy_lookup(&invalid, false));
    }

    #[test]
    fn terrain_adjustment_ids_and_bounding_boxes_match_vanilla() {
        let ids = [
            (super::super::TerrainAdjustmentModel::None, "none"),
            (super::super::TerrainAdjustmentModel::Bury, "bury"),
            (super::super::TerrainAdjustmentModel::BeardThin, "beard_thin"),
            (super::super::TerrainAdjustmentModel::BeardBox, "beard_box"),
            (super::super::TerrainAdjustmentModel::Encapsulate, "encapsulate"),
        ];
        for (adjustment, id) in ids {
            assert_eq!(adjustment.id(), id);
            assert_eq!(super::super::TerrainAdjustmentModel::from_id(id), Some(adjustment));
        }
        assert_eq!(super::super::TerrainAdjustmentModel::from_id("beard"), None);

        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 40,
            max_y: 50,
            max_z: 60,
        };
        assert_eq!(
            super::super::structure_adjust_bounding_box(super::super::TerrainAdjustmentModel::None, bounding_box),
            bounding_box
        );
        assert_eq!(
            super::super::structure_adjust_bounding_box(super::super::TerrainAdjustmentModel::Bury, bounding_box),
            super::super::StructureBoundingBoxModel {
                min_x: -2,
                min_y: 8,
                min_z: 18,
                max_x: 52,
                max_y: 62,
                max_z: 72,
            }
        );

        assert!(super::super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::super::TerrainAdjustmentModel::None
        ));
        assert!(!super::super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::super::TerrainAdjustmentModel::Bury
        ));
        assert!(super::super::jigsaw_max_distance_with_terrain_is_valid(
            116,
            super::super::TerrainAdjustmentModel::Encapsulate
        ));
        assert!(!super::super::jigsaw_max_distance_with_terrain_is_valid(
            117,
            super::super::TerrainAdjustmentModel::BeardBox
        ));
    }
