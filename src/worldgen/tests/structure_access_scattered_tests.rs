use super::*;

    #[test]
    fn structure_access_stores_starts_and_reference_sets_like_chunks() {
        let piece = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 64,
                min_z: 48,
                max_x: 47,
                max_y: 80,
                max_z: 63,
            },
        };
        let start = super::super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: 3 },
            references: 0,
            pieces: vec![piece],
        };
        let invalid = super::super::StructureStartModel::invalid();
        let mut access = super::super::StructureAccessModel::default();

        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[] as &[i64]
        );
        assert!(!access.has_any_structure_references());
        assert!(!access.unsaved);

        access.set_start_for_structure("minecraft:shipwreck", start.clone());
        access.set_start_for_structure("minecraft:mineshaft", invalid.clone());
        assert_eq!(
            access.get_start_for_structure("minecraft:shipwreck"),
            Some(&start)
        );
        assert!(access.unsaved);

        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[0x0000_0002_0000_0003]
        );
        assert!(access.has_any_structure_references());

        let mut replacement_starts = BTreeMap::new();
        replacement_starts.insert("minecraft:mineshaft", invalid.clone());
        access.set_all_starts(replacement_starts);
        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_start_for_structure("minecraft:mineshaft"),
            Some(&invalid)
        );

        let mut replacement_references = BTreeMap::new();
        replacement_references.insert("minecraft:mineshaft", vec![7]);
        access.set_all_references(replacement_references);
        assert_eq!(
            access.get_references_for_structure("minecraft:mineshaft"),
            &[7]
        );
    }

    #[test]
    fn structure_access_resolves_only_valid_referenced_starts() {
        let valid_start = super::super::StructureStartModel {
            structure: Some("minecraft:buried_treasure"),
            chunk_pos: ChunkPos { x: 1, z: 1 },
            references: 0,
            pieces: vec![super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: 16,
                    min_y: 45,
                    min_z: 16,
                    max_x: 31,
                    max_y: 55,
                    max_z: 31,
                },
            }],
        };
        let mut valid_access = super::super::StructureAccessModel::default();
        valid_access.set_start_for_structure("minecraft:buried_treasure", valid_start.clone());

        let mut invalid_access = super::super::StructureAccessModel::default();
        invalid_access.set_start_for_structure(
            "minecraft:buried_treasure",
            super::super::StructureStartModel::invalid(),
        );

        let mut chunks = BTreeMap::new();
        chunks.insert(11, valid_access);
        chunks.insert(12, invalid_access);
        let starts = super::super::structure_access_valid_starts_for_references(
            &chunks,
            "minecraft:buried_treasure",
            &[11, 12, 13],
        );
        assert_eq!(starts, vec![valid_start]);
    }

    #[test]
    fn chunk_generator_create_references_scans_nearby_valid_intersections() {
        let shipwreck_start = super::super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 0,
            pieces: vec![super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: 31,
                    min_y: 50,
                    min_z: -16,
                    max_x: 48,
                    max_y: 70,
                    max_z: -1,
                },
            }],
        };
        let out_of_range_start = super::super::StructureStartModel {
            structure: Some("minecraft:mineshaft"),
            chunk_pos: ChunkPos { x: 9, z: 0 },
            references: 0,
            pieces: vec![super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: 0,
                    min_y: 0,
                    min_z: 0,
                    max_x: 15,
                    max_y: 10,
                    max_z: 15,
                },
            }],
        };
        let non_intersecting_start = super::super::StructureStartModel {
            structure: Some("minecraft:village_plains"),
            chunk_pos: ChunkPos { x: -1, z: 0 },
            references: 0,
            pieces: vec![super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: -32,
                    min_y: 60,
                    min_z: 0,
                    max_x: -17,
                    max_y: 80,
                    max_z: 15,
                },
            }],
        };

        assert_eq!(
            super::super::chunk_pos_key(ChunkPos { x: -1, z: 2 }),
            0x0000_0002_ffff_ffff
        );
        assert_eq!(
            super::super::structure_reference_writable_area(ChunkPos { x: 2, z: -1 }),
            super::super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: i32::MIN,
                min_z: -16,
                max_x: 47,
                max_y: i32::MAX,
                max_z: -1,
            }
        );

        let mut source_access = super::super::StructureAccessModel::default();
        source_access.set_start_for_structure("minecraft:shipwreck", shipwreck_start);
        source_access.set_start_for_structure(
            "minecraft:stronghold",
            super::super::StructureStartModel::invalid(),
        );

        let mut out_of_range_access = super::super::StructureAccessModel::default();
        out_of_range_access.set_start_for_structure("minecraft:mineshaft", out_of_range_start);

        let mut non_intersecting_access = super::super::StructureAccessModel::default();
        non_intersecting_access
            .set_start_for_structure("minecraft:village_plains", non_intersecting_start);

        let mut chunks = BTreeMap::new();
        chunks.insert(
            super::super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
            source_access,
        );
        chunks.insert(
            super::super::chunk_pos_key(ChunkPos { x: 9, z: 0 }),
            out_of_range_access,
        );
        chunks.insert(
            super::super::chunk_pos_key(ChunkPos { x: -1, z: 0 }),
            non_intersecting_access,
        );

        let references = super::super::structure_references_for_chunk(&chunks, ChunkPos { x: 2, z: -1 });
        assert_eq!(
            references,
            vec![super::super::StructureReferenceModel {
                structure: "minecraft:shipwreck",
                source_chunk_key: super::super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
            }]
        );

        let mut target_access = super::super::StructureAccessModel::default();
        let applied = super::super::create_structure_references_for_chunk(
            &chunks,
            ChunkPos { x: 2, z: -1 },
            &mut target_access,
        );
        assert_eq!(applied, references);
        assert_eq!(
            target_access.get_references_for_structure("minecraft:shipwreck"),
            &[super::super::chunk_pos_key(ChunkPos { x: 2, z: -1 })]
        );
        assert!(target_access.unsaved);
    }

    #[test]
    fn structure_access_serializes_chunk_structures_payload() {
        let start = super::super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 1,
            pieces: vec![super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: 32,
                    min_y: 50,
                    min_z: -16,
                    max_x: 47,
                    max_y: 70,
                    max_z: -1,
                },
            }],
        };
        let mut access = super::super::StructureAccessModel::default();
        access.set_start_for_structure("minecraft:shipwreck", start);
        access.set_start_for_structure(
            "minecraft:stronghold",
            super::super::StructureStartModel::invalid(),
        );
        access.add_reference_for_structure(
            "minecraft:shipwreck",
            super::super::chunk_pos_key(ChunkPos { x: 2, z: -1 }),
        );

        let Tag::Compound(root) =
            super::super::structure_access_to_chunk_structures_tag(&access, ChunkPos { x: 2, z: -1 })
        else {
            panic!("structures payload should be a compound");
        };
        let Some((_, Tag::Compound(starts))) = root.iter().find(|(name, _)| name == "starts")
        else {
            panic!("starts should be a compound");
        };
        let Some((_, Tag::Compound(shipwreck))) = starts
            .iter()
            .find(|(name, _)| name == "minecraft:shipwreck")
        else {
            panic!("shipwreck start should be serialized");
        };

        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "id"),
            Some((_, Tag::String(id))) if id == "minecraft:shipwreck"
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "ChunkX"),
            Some((_, Tag::Int(2)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "ChunkZ"),
            Some((_, Tag::Int(-1)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "references"),
            Some((_, Tag::Int(1)))
        ));
        assert!(matches!(
            shipwreck.iter().find(|(name, _)| name == "Children"),
            Some((_, Tag::List(children))) if children.len() == 1
        ));
        assert!(matches!(
            starts.iter().find(|(name, _)| name == "minecraft:stronghold"),
            Some((_, Tag::Compound(invalid))) if matches!(
                invalid.iter().find(|(name, _)| name == "id"),
                Some((_, Tag::String(id))) if id == "INVALID"
            )
        ));

        let Some((_, Tag::Compound(references))) =
            root.iter().find(|(name, _)| name == "References")
        else {
            panic!("References should be a compound");
        };
        assert!(matches!(
            references.iter().find(|(name, _)| name == "minecraft:shipwreck"),
            Some((_, Tag::LongArray(values)))
                if values == &[super::super::chunk_pos_key(ChunkPos { x: 2, z: -1 })]
        ));
    }

    #[test]
    fn structure_family_coverage_matches_builtin_structure_keys() {
        assert_eq!(STRUCTURE_FAMILIES.len(), 21);
        assert_eq!(
            STRUCTURE_FAMILIES
                .iter()
                .map(|entry| entry.structures.len())
                .sum::<usize>(),
            BUILTIN_STRUCTURES.len()
        );

        let villages = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::Village)
            .unwrap();
        assert_eq!(villages.structures.len(), 5);

        let ruined_portals = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::RuinedPortal)
            .unwrap();
        assert_eq!(ruined_portals.structures.len(), 7);
        assert!(ruined_portals
            .structures
            .contains(&"minecraft:ruined_portal_nether"));

        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::OceanRuins)
            .unwrap()
            .structures
            .contains(&"minecraft:ocean_ruin_warm"));
        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::TrialChambers)
            .unwrap()
            .structures
            .contains(&"minecraft:trial_chambers"));
    }

    #[test]
    fn buried_treasure_piece_generation_and_support_scan_match_vanilla() {
        let piece = super::super::buried_treasure_generation_piece(ChunkPos { x: -2, z: 3 });
        assert_eq!(
            piece.bounding_box,
            super::super::StructureBoundingBoxModel {
                min_x: -23,
                min_y: 90,
                min_z: 57,
                max_x: -23,
                max_y: 90,
                max_z: 57,
            }
        );
        assert!(super::super::buried_treasure_is_support("minecraft:sandstone"));
        assert!(super::super::buried_treasure_is_support("minecraft:diorite"));
        assert!(!super::super::buried_treasure_is_support("minecraft:sand"));
        assert_eq!(
            super::super::buried_treasure_soft_state("minecraft:water"),
            "minecraft:sand"
        );
        assert_eq!(
            super::super::buried_treasure_soft_state("minecraft:gravel"),
            "minecraft:gravel"
        );

        let placement = super::super::buried_treasure_place(piece, 75, 60, |pos| {
            if pos.y == 69 && pos.x == -23 && pos.z == 57 {
                "minecraft:stone"
            } else if pos.y == 70 && pos.x == -23 && pos.z == 57 {
                "minecraft:water"
            } else if pos.y == 69 {
                "minecraft:water"
            } else {
                "minecraft:air"
            }
        })
        .unwrap();
        assert_eq!(
            placement.chest_pos,
            BlockPos {
                x: -23,
                y: 70,
                z: 57,
            }
        );
        assert_eq!(placement.bounding_box.min_y, 70);
        assert_eq!(placement.side_fill[0].0, "minecraft:stone");
        assert_eq!(placement.side_fill[1].0, "minecraft:sand");
        assert_eq!(placement.side_fill[2].0, "minecraft:stone");
        assert_eq!(placement.side_fill[3].0, "minecraft:stone");
        assert_eq!(placement.side_fill[4].0, "minecraft:stone");
        assert_eq!(placement.side_fill[5].0, "minecraft:stone");

        assert!(super::super::buried_treasure_place(piece, 64, 60, |_| "minecraft:sand").is_none());
    }

    #[test]
    fn swamp_hut_piece_layout_height_and_entity_flags_match_vanilla() {
        let piece = super::super::swamp_hut_generation_piece(
            ChunkPos { x: 0, z: 0 },
            super::super::HorizontalDirection::South,
        );
        assert_eq!(piece.scattered.width, 7);
        assert_eq!(piece.scattered.height, 7);
        assert_eq!(piece.scattered.depth, 9);
        assert_eq!(piece.scattered.height_position, -1);
        assert_eq!(
            piece.scattered.bounding_box,
            super::super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 64,
                min_z: 0,
                max_x: 6,
                max_y: 70,
                max_z: 8,
            }
        );
        assert_eq!(
            super::super::swamp_hut_save_tag(&piece),
            super::super::SwampHutSaveTagModel {
                width: 7,
                height: 7,
                depth: 9,
                height_position: -1,
                witch: false,
                cat: false,
            }
        );

        let chunk_bb = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: i32::MIN,
            min_z: 0,
            max_x: 15,
            max_y: i32::MAX,
            max_z: 15,
        };
        let processed = super::super::swamp_hut_post_process(piece.clone(), chunk_bb, |x, z| {
            70 + (x == 6 && z == 8) as i32
        })
        .unwrap();
        assert_eq!(processed.piece.scattered.height_position, 70);
        assert_eq!(processed.piece.scattered.bounding_box.min_y, 70);
        assert_eq!(processed.fill_columns.len(), 4);
        assert_eq!(
            processed.entity_spawns,
            vec![
                super::super::SwampHutEntitySpawnModel {
                    entity: "minecraft:witch",
                    pos: BlockPos { x: 2, y: 72, z: 5 },
                },
                super::super::SwampHutEntitySpawnModel {
                    entity: "minecraft:cat",
                    pos: BlockPos { x: 2, y: 72, z: 5 },
                },
            ]
        );
        assert!(processed.piece.spawned_witch);
        assert!(processed.piece.spawned_cat);
        assert!(processed.blocks.iter().any(|block| {
            block.world_pos == BlockPos { x: 4, y: 72, z: 6 } && block.state == "minecraft:cauldron"
        }));
        assert!(processed.blocks.iter().any(|block| {
            block.world_pos == BlockPos { x: 0, y: 74, z: 1 }
                && block.state == "minecraft:spruce_stairs[facing=north,shape=outer_right]"
        }));

        let mut already_spawned = piece.clone();
        already_spawned.spawned_witch = true;
        already_spawned.spawned_cat = true;
        let no_spawns =
            super::super::swamp_hut_post_process(already_spawned, chunk_bb, |_, _| 70).unwrap();
        assert!(no_spawns.entity_spawns.is_empty());

        let outside_chunk = super::super::StructureBoundingBoxModel {
            min_x: 100,
            min_y: i32::MIN,
            min_z: 100,
            max_x: 115,
            max_y: i32::MAX,
            max_z: 115,
        };
        assert!(super::super::swamp_hut_post_process(piece, outside_chunk, |_, _| 70).is_none());
    }

