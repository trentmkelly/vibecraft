use super::super::*;

    #[test]
    fn feature_sorter_builds_step_order_and_index_mapping_like_vanilla() {
        let plains = super::super::biome_generation_settings("plains").unwrap();
        let forest = super::super::biome_generation_settings("forest").unwrap();
        let sorted =
            super::super::build_features_per_step(&[plains.feature_steps, forest.feature_steps], true)
                .unwrap();

        assert_eq!(sorted.len(), 11);
        assert!(sorted[0].features.is_empty());
        assert_eq!(
            sorted[1].feature_names(),
            vec![
                "minecraft:lake_lava_underground",
                "minecraft:lake_lava_surface"
            ]
        );
        assert!(sorted[6]
            .feature_names()
            .contains(&"minecraft:ore_diamond_buried"));
        assert!(sorted[9]
            .feature_names()
            .contains(&"minecraft:trees_plains"));
        assert!(sorted[9]
            .feature_names()
            .contains(&"minecraft:trees_birch_and_oak_leaf_litter"));
        assert!(
            super::super::FeatureSorterData {
                feature_index: 99,
                step: 1,
                feature: "minecraft:earlier_step",
            } < super::super::FeatureSorterData {
                feature_index: 0,
                step: 9,
                feature: "minecraft:later_step",
            },
            "FeatureSorter comparator must match Java: step first, then feature index"
        );
        let trees_plains_index = sorted[9]
            .index_mapping("minecraft:trees_plains")
            .expect("trees_plains should have an index in the vegetal decoration step");
        assert_eq!(
            sorted[9].features[trees_plains_index].feature,
            "minecraft:trees_plains"
        );
        assert_eq!(sorted[9].index_mapping("minecraft:missing"), None);
    }

    #[test]
    fn biome_decoration_feature_plan_uses_possible_biomes_sorted_indices_and_feature_seeds() {
        let plains = super::super::biome_generation_settings("plains").unwrap();
        let forest = super::super::biome_generation_settings("forest").unwrap();
        let sorted =
            super::super::build_features_per_step(&[plains.feature_steps, forest.feature_steps], true)
                .unwrap();
        let plan = super::super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps],
        );

        assert_eq!(
            plan.origin,
            BlockPos {
                x: 64,
                y: -64,
                z: -112
            }
        );
        assert_eq!(
            plan.decoration_seed,
            crate::random_source::decoration_seed(
                12_345,
                64,
                -112,
                crate::random_source::RandomAlgorithm::Xoroshiro
            )
        );
        assert!(plan.feature_calls.windows(2).all(|calls| (
            calls[0].step_index,
            calls[0].global_feature_index
        ) <= (
            calls[1].step_index,
            calls[1].global_feature_index
        )));

        let plains_tree_call = plan
            .feature_calls
            .iter()
            .find(|call| call.feature == "minecraft:trees_plains")
            .expect("plains trees should be planned for plains biome decoration");
        assert_eq!(
            plains_tree_call.global_feature_index,
            sorted[plains_tree_call.step_index]
                .index_mapping("minecraft:trees_plains")
                .unwrap()
        );
        assert_eq!(
            plains_tree_call.seed,
            crate::random_source::feature_seed(
                plan.decoration_seed,
                plains_tree_call.global_feature_index as i32,
                plains_tree_call.step_index as i32
            )
        );
        assert!(!plan
            .feature_calls
            .iter()
            .any(|call| call.feature == "minecraft:trees_birch_and_oak_leaf_litter"));

        let mixed_plan = super::super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps, forest.feature_steps],
        );
        assert!(mixed_plan
            .feature_calls
            .iter()
            .any(|call| call.feature == "minecraft:trees_birch_and_oak_leaf_litter"));
    }

    #[test]
    fn decoration_seed_for_fixed_biome_chunk_and_step_matches_vanilla() {
        let plains = super::super::biome_generation_settings("plains").unwrap();
        let sorted = super::super::build_features_per_step(&[plains.feature_steps], true).unwrap();
        let plan = super::super::biome_decoration_feature_plan(
            12_345,
            4,
            -7,
            -4,
            &sorted,
            &[plains.feature_steps],
        );
        let trees = plan
            .feature_calls
            .iter()
            .find(|call| call.feature == "minecraft:trees_plains")
            .expect("plains decoration should schedule trees_plains");

        assert_eq!(plan.decoration_seed, -6_006_185_048_957_774_615);
        assert_eq!(
            trees.step_index,
            GenerationDecorationStep::VegetalDecoration as usize
        );
        assert_eq!(trees.global_feature_index, 3);
        assert_eq!(trees.seed, -6_006_185_048_957_684_612);
    }

    #[test]
    fn biome_decoration_structure_calls_use_per_step_indices_before_features() {
        let decoration_seed = crate::random_source::decoration_seed(
            12_345,
            64,
            -112,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        let calls = super::super::biome_decoration_structure_calls(
            decoration_seed,
            5,
            &[
                &[],
                &["minecraft:mineshaft", "minecraft:village"],
                &[],
                &["minecraft:stronghold"],
            ],
        );

        assert_eq!(
            calls,
            vec![
                super::super::BiomeDecorationStructureCall {
                    step_index: 1,
                    step_structure_index: 0,
                    structure: "minecraft:mineshaft",
                    seed: crate::random_source::feature_seed(decoration_seed, 0, 1),
                },
                super::super::BiomeDecorationStructureCall {
                    step_index: 1,
                    step_structure_index: 1,
                    structure: "minecraft:village",
                    seed: crate::random_source::feature_seed(decoration_seed, 1, 1),
                },
                super::super::BiomeDecorationStructureCall {
                    step_index: 3,
                    step_structure_index: 0,
                    structure: "minecraft:stronghold",
                    seed: crate::random_source::feature_seed(decoration_seed, 0, 3),
                },
            ]
        );
    }

    #[test]
    fn feature_sorter_reports_order_cycles() {
        static SOURCE_A: &[&[&str]] = &[&["minecraft:a", "minecraft:b"]];
        static SOURCE_B: &[&[&str]] = &[&["minecraft:b", "minecraft:a"]];
        static SOURCE_C: &[&[&str]] = &[&["minecraft:c", "minecraft:d"]];

        assert_eq!(
            super::super::build_features_per_step(&[SOURCE_A, SOURCE_B], false).unwrap_err(),
            "Feature order cycle found".to_string()
        );
        assert_eq!(
            super::super::build_features_per_step(&[SOURCE_A, SOURCE_B], true).unwrap_err(),
            "Feature order cycle found, involved sources: 2".to_string()
        );
        assert_eq!(
            super::super::build_features_per_step_with_source_ids(
                &[
                    super::super::FeatureSorterSourceModel {
                        id: "source_a",
                        feature_steps: SOURCE_A,
                    },
                    super::super::FeatureSorterSourceModel {
                        id: "source_b",
                        feature_steps: SOURCE_B,
                    },
                    super::super::FeatureSorterSourceModel {
                        id: "irrelevant_source_c",
                        feature_steps: SOURCE_C,
                    },
                ],
                true,
            )
            .unwrap_err(),
            "Feature order cycle found, involved sources: [source_a, source_b]".to_string()
        );
    }
