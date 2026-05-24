use super::*;

    #[test]
    fn queue_chunk_generation_mob_entity_appends_proto_entity_nbt() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        let snap = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.9,
            y: 70.0,
            z: -33.0,
            yaw: 90.0,
            pitch: 0.0,
        };

        assert!(super::super::queue_chunk_generation_mob_entity(
            &mut chunk,
            snap,
            "00000000-0000-0000-0000-000000000123"
        ));

        assert_eq!(chunk.entities.len(), 1);
        let Tag::Compound(fields) = &chunk.entities[0] else {
            panic!("queued entity must be a compound");
        };
        assert!(fields.contains(&("id".to_string(), Tag::String("minecraft:pig".to_string()))));
        assert!(fields.contains(&(
            "UUID".to_string(),
            Tag::String("00000000-0000-0000-0000-000000000123".to_string())
        )));
        assert!(fields.contains(&(
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0)
            ])
        )));
        assert!(fields.contains(&(
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)])
        )));
        assert!(fields.contains(&(
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)])
        )));
        assert!(fields.contains(&("fall_distance".to_string(), Tag::Double(0.0))));
        assert!(fields.contains(&("Fire".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("Air".to_string(), Tag::Short(300))));
        assert!(fields.contains(&("OnGround".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("Invulnerable".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("PortalCooldown".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("Age".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("ForcedAge".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("AgeLocked".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("Health".to_string(), Tag::Float(10.0))));
        assert!(fields.contains(&("HurtTime".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("HurtByTimestamp".to_string(), Tag::Int(0))));
        assert!(fields.contains(&("DeathTime".to_string(), Tag::Short(0))));
        assert!(fields.contains(&("AbsorptionAmount".to_string(), Tag::Float(0.0))));
        assert!(fields.contains(&(
            "current_impulse_context_reset_grace_time".to_string(),
            Tag::Int(0)
        )));
        assert!(fields.contains(&("CanPickUpLoot".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("PersistenceRequired".to_string(), Tag::Byte(0))));
        assert!(fields.contains(&("LeftHanded".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_omits_ageable_fields_for_non_ageable_mobs() {
        let snap = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:creeper",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(fields) =
            super::super::chunk_generation_mob_entity_nbt(snap, "00000000-0000-0000-0000-000000000124")
        else {
            panic!("generated entity nbt must be a compound");
        };

        assert!(!fields.iter().any(|(name, _)| name == "Age"));
        assert!(!fields.iter().any(|(name, _)| name == "ForcedAge"));
        assert!(!fields.iter().any(|(name, _)| name == "AgeLocked"));
        assert!(!fields.iter().any(|(name, _)| name == "InLove"));
        assert!(fields.contains(&("Health".to_string(), Tag::Float(20.0))));
        assert!(fields.contains(&("CanPickUpLoot".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_animal_superclass_save_fields() {
        let animal = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ageable_non_animal = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:dolphin",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(animal_fields) =
            super::super::chunk_generation_mob_entity_nbt(animal, "00000000-0000-0000-0000-000000000130")
        else {
            panic!("animal entity nbt must be a compound");
        };
        let Tag::Compound(ageable_non_animal_fields) = super::super::chunk_generation_mob_entity_nbt(
            ageable_non_animal,
            "00000000-0000-0000-0000-000000000131",
        ) else {
            panic!("ageable non-animal entity nbt must be a compound");
        };

        assert!(animal_fields.contains(&("InLove".to_string(), Tag::Int(0))));
        assert!(!ageable_non_animal_fields
            .iter()
            .any(|(name, _)| name == "InLove"));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_neutral_mob_anger_save_fields() {
        let neutral_entities = [
            "minecraft:bee",
            "minecraft:enderman",
            "minecraft:iron_golem",
            "minecraft:polar_bear",
            "minecraft:wolf",
            "minecraft:zombified_piglin",
        ];
        let zombie = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        for (index, entity_type) in neutral_entities.iter().enumerate() {
            let snap = super::super::ChunkGenerationMobEntitySnapPlan {
                entity_type,
                width: 0.6,
                x: 32.5,
                y: 70.0,
                z: -33.5,
                yaw: 0.0,
                pitch: 0.0,
            };
            let uuid = format!("00000000-0000-0000-0000-{:012}", 172 + index);
            let Tag::Compound(fields) = super::super::chunk_generation_mob_entity_nbt(snap, &uuid) else {
                panic!("neutral entity nbt must be a compound");
            };

            assert!(
                fields.contains(&("anger_end_time".to_string(), Tag::Long(0))),
                "{entity_type} should persist default anger end time"
            );
            assert!(
                !fields.iter().any(|(name, _)| name == "angry_at"),
                "{entity_type} should omit nullable angry_at without a target"
            );
        }

        let Tag::Compound(zombie_fields) =
            super::super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000174")
        else {
            panic!("zombie entity nbt must be a compound");
        };

        assert!(!zombie_fields.contains(&("anger_end_time".to_string(), Tag::Long(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_raider_patrol_save_fields() {
        let witch = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:witch",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ravager = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ravager",
            width: 1.95,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(witch_fields) =
            super::super::chunk_generation_mob_entity_nbt(witch, "00000000-0000-0000-0000-000000000182")
        else {
            panic!("witch entity nbt must be a compound");
        };
        let Tag::Compound(ravager_fields) =
            super::super::chunk_generation_mob_entity_nbt(ravager, "00000000-0000-0000-0000-000000000183")
        else {
            panic!("ravager entity nbt must be a compound");
        };
        let Tag::Compound(zombie_fields) =
            super::super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000184")
        else {
            panic!("zombie entity nbt must be a compound");
        };

        for fields in [&witch_fields, &ravager_fields] {
            assert!(fields.contains(&("PatrolLeader".to_string(), Tag::Byte(0))));
            assert!(fields.contains(&("Patrolling".to_string(), Tag::Byte(0))));
            assert!(!fields.iter().any(|(name, _)| name == "patrol_target"));
            assert!(fields.contains(&("Wave".to_string(), Tag::Int(0))));
            assert!(fields.contains(&("CanJoinRaid".to_string(), Tag::Byte(0))));
            assert!(!fields.iter().any(|(name, _)| name == "RaidId"));
        }
        assert!(!zombie_fields.contains(&("PatrolLeader".to_string(), Tag::Byte(0))));
        assert!(!zombie_fields.contains(&("Patrolling".to_string(), Tag::Byte(0))));
        assert!(!zombie_fields.contains(&("Wave".to_string(), Tag::Int(0))));
        assert!(!zombie_fields.contains(&("CanJoinRaid".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_phantom_and_shulker_save_fields() {
        let phantom = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:phantom",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let shulker = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:shulker",
            width: 1.0,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(phantom_fields) =
            super::super::chunk_generation_mob_entity_nbt(phantom, "00000000-0000-0000-0000-000000000192")
        else {
            panic!("phantom entity nbt must be a compound");
        };
        let Tag::Compound(shulker_fields) =
            super::super::chunk_generation_mob_entity_nbt(shulker, "00000000-0000-0000-0000-000000000193")
        else {
            panic!("shulker entity nbt must be a compound");
        };

        assert!(phantom_fields.contains(&("size".to_string(), Tag::Int(0))));
        assert!(!phantom_fields.iter().any(|(name, _)| name == "anchor_pos"));
        assert!(shulker_fields.contains(&("AttachFace".to_string(), Tag::Byte(0))));
        assert!(shulker_fields.contains(&("Peek".to_string(), Tag::Byte(0))));
        assert!(shulker_fields.contains(&("Color".to_string(), Tag::Byte(16))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_additional_animal_save_fields() {
        let bat = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bat",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let mooshroom = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:mooshroom",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let panda = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:panda",
            width: 1.3,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let parrot = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:parrot",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let turtle = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:turtle",
            width: 1.2,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(bat_fields) =
            super::super::chunk_generation_mob_entity_nbt(bat, "00000000-0000-0000-0000-000000000202")
        else {
            panic!("bat entity nbt must be a compound");
        };
        let Tag::Compound(mooshroom_fields) = super::super::chunk_generation_mob_entity_nbt(
            mooshroom,
            "00000000-0000-0000-0000-000000000203",
        ) else {
            panic!("mooshroom entity nbt must be a compound");
        };
        let Tag::Compound(panda_fields) =
            super::super::chunk_generation_mob_entity_nbt(panda, "00000000-0000-0000-0000-000000000204")
        else {
            panic!("panda entity nbt must be a compound");
        };
        let Tag::Compound(parrot_fields) =
            super::super::chunk_generation_mob_entity_nbt(parrot, "00000000-0000-0000-0000-000000000205")
        else {
            panic!("parrot entity nbt must be a compound");
        };
        let Tag::Compound(turtle_fields) =
            super::super::chunk_generation_mob_entity_nbt(turtle, "00000000-0000-0000-0000-000000000206")
        else {
            panic!("turtle entity nbt must be a compound");
        };

        assert!(bat_fields.contains(&("BatFlags".to_string(), Tag::Byte(0))));
        assert!(mooshroom_fields.contains(&("Type".to_string(), Tag::String("red".to_string()))));
        assert!(!mooshroom_fields
            .iter()
            .any(|(name, _)| name == "stew_effects"));
        assert!(panda_fields.contains(&("MainGene".to_string(), Tag::String("normal".to_string()))));
        assert!(
            panda_fields.contains(&("HiddenGene".to_string(), Tag::String("normal".to_string())))
        );
        assert!(parrot_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(turtle_fields.contains(&(
            "home_pos".to_string(),
            Tag::List(vec![Tag::Int(0), Tag::Int(0), Tag::Int(0)])
        )));
        assert!(turtle_fields.contains(&("has_egg".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_horse_family_save_fields() {
        let horse = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let donkey = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:donkey",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let skeleton_horse = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:skeleton_horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(horse_fields) =
            super::super::chunk_generation_mob_entity_nbt(horse, "00000000-0000-0000-0000-000000000132")
        else {
            panic!("horse entity nbt must be a compound");
        };
        let Tag::Compound(donkey_fields) =
            super::super::chunk_generation_mob_entity_nbt(donkey, "00000000-0000-0000-0000-000000000133")
        else {
            panic!("donkey entity nbt must be a compound");
        };
        let Tag::Compound(skeleton_horse_fields) = super::super::chunk_generation_mob_entity_nbt(
            skeleton_horse,
            "00000000-0000-0000-0000-000000000134",
        ) else {
            panic!("skeleton horse entity nbt must be a compound");
        };

        for field_name in ["EatingHaystack", "Bred", "Tame"] {
            assert!(horse_fields.contains(&(field_name.to_string(), Tag::Byte(0))));
        }
        assert!(horse_fields.contains(&("Temper".to_string(), Tag::Int(0))));
        assert!(donkey_fields.contains(&("ChestedHorse".to_string(), Tag::Byte(0))));
        assert!(skeleton_horse_fields.contains(&("SkeletonTrap".to_string(), Tag::Byte(0))));
        assert!(skeleton_horse_fields.contains(&("SkeletonTrapTime".to_string(), Tag::Int(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_water_and_ambient_animal_save_fields() {
        let pufferfish = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pufferfish",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let dolphin = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:dolphin",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let fox = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:fox",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let camel = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:camel",
            width: 1.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(pufferfish_fields) = super::super::chunk_generation_mob_entity_nbt(
            pufferfish,
            "00000000-0000-0000-0000-000000000135",
        ) else {
            panic!("pufferfish entity nbt must be a compound");
        };
        let Tag::Compound(dolphin_fields) =
            super::super::chunk_generation_mob_entity_nbt(dolphin, "00000000-0000-0000-0000-000000000136")
        else {
            panic!("dolphin entity nbt must be a compound");
        };
        let Tag::Compound(fox_fields) =
            super::super::chunk_generation_mob_entity_nbt(fox, "00000000-0000-0000-0000-000000000137")
        else {
            panic!("fox entity nbt must be a compound");
        };
        let Tag::Compound(camel_fields) =
            super::super::chunk_generation_mob_entity_nbt(camel, "00000000-0000-0000-0000-000000000138")
        else {
            panic!("camel entity nbt must be a compound");
        };

        assert!(pufferfish_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(pufferfish_fields.contains(&("PuffState".to_string(), Tag::Int(0))));
        assert!(dolphin_fields.contains(&("GotFish".to_string(), Tag::Byte(0))));
        assert!(dolphin_fields.contains(&("Moistness".to_string(), Tag::Int(2400))));
        for field_name in ["Sleeping", "Sitting", "Crouching"] {
            assert!(fox_fields.contains(&(field_name.to_string(), Tag::Byte(0))));
        }
        assert!(camel_fields.contains(&("LastPoseTick".to_string(), Tag::Long(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_special_animal_save_fields() {
        let bee = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bee",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let llama = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:llama",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let trader_llama = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:trader_llama",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let armadillo = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:armadillo",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let horse = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:horse",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let iron_golem = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:iron_golem",
            width: 1.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let snow_golem = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:snow_golem",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(bee_fields) =
            super::super::chunk_generation_mob_entity_nbt(bee, "00000000-0000-0000-0000-000000000139")
        else {
            panic!("bee entity nbt must be a compound");
        };
        let Tag::Compound(llama_fields) =
            super::super::chunk_generation_mob_entity_nbt(llama, "00000000-0000-0000-0000-000000000140")
        else {
            panic!("llama entity nbt must be a compound");
        };
        let Tag::Compound(trader_llama_fields) = super::super::chunk_generation_mob_entity_nbt(
            trader_llama,
            "00000000-0000-0000-0000-000000000141",
        ) else {
            panic!("trader llama entity nbt must be a compound");
        };
        let Tag::Compound(armadillo_fields) = super::super::chunk_generation_mob_entity_nbt(
            armadillo,
            "00000000-0000-0000-0000-000000000142",
        ) else {
            panic!("armadillo entity nbt must be a compound");
        };
        let Tag::Compound(horse_fields) =
            super::super::chunk_generation_mob_entity_nbt(horse, "00000000-0000-0000-0000-000000000144")
        else {
            panic!("horse entity nbt must be a compound");
        };
        let Tag::Compound(iron_golem_fields) = super::super::chunk_generation_mob_entity_nbt(
            iron_golem,
            "00000000-0000-0000-0000-000000000145",
        ) else {
            panic!("iron golem entity nbt must be a compound");
        };
        let Tag::Compound(snow_golem_fields) = super::super::chunk_generation_mob_entity_nbt(
            snow_golem,
            "00000000-0000-0000-0000-000000000143",
        ) else {
            panic!("snow golem entity nbt must be a compound");
        };

        assert!(bee_fields.contains(&("HasNectar".to_string(), Tag::Byte(0))));
        assert!(bee_fields.contains(&("HasStung".to_string(), Tag::Byte(0))));
        assert!(bee_fields.contains(&("TicksSincePollination".to_string(), Tag::Int(0))));
        assert!(bee_fields.contains(&("CannotEnterHiveTicks".to_string(), Tag::Int(0))));
        assert!(bee_fields.contains(&("CropsGrownSincePollination".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("Strength".to_string(), Tag::Int(0))));
        assert!(llama_fields.contains(&("ChestedHorse".to_string(), Tag::Byte(0))));
        assert!(trader_llama_fields.contains(&("DespawnDelay".to_string(), Tag::Int(47999))));
        assert!(armadillo_fields.contains(&("state".to_string(), Tag::String("idle".to_string()))));
        assert!(horse_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(iron_golem_fields.contains(&("PlayerCreated".to_string(), Tag::Byte(0))));
        assert!(snow_golem_fields.contains(&("Pumpkin".to_string(), Tag::Byte(1))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_monster_save_fields() {
        let creeper = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:creeper",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let slime = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:slime",
            width: 0.52,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ravager = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ravager",
            width: 1.95,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let ghast = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:ghast",
            width: 4.0,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let endermite = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:endermite",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zoglin = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zoglin",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(creeper_fields) =
            super::super::chunk_generation_mob_entity_nbt(creeper, "00000000-0000-0000-0000-000000000146")
        else {
            panic!("creeper entity nbt must be a compound");
        };
        let Tag::Compound(slime_fields) =
            super::super::chunk_generation_mob_entity_nbt(slime, "00000000-0000-0000-0000-000000000147")
        else {
            panic!("slime entity nbt must be a compound");
        };
        let Tag::Compound(ravager_fields) =
            super::super::chunk_generation_mob_entity_nbt(ravager, "00000000-0000-0000-0000-000000000148")
        else {
            panic!("ravager entity nbt must be a compound");
        };
        let Tag::Compound(ghast_fields) =
            super::super::chunk_generation_mob_entity_nbt(ghast, "00000000-0000-0000-0000-000000000149")
        else {
            panic!("ghast entity nbt must be a compound");
        };
        let Tag::Compound(endermite_fields) = super::super::chunk_generation_mob_entity_nbt(
            endermite,
            "00000000-0000-0000-0000-000000000150",
        ) else {
            panic!("endermite entity nbt must be a compound");
        };
        let Tag::Compound(zoglin_fields) =
            super::super::chunk_generation_mob_entity_nbt(zoglin, "00000000-0000-0000-0000-000000000151")
        else {
            panic!("zoglin entity nbt must be a compound");
        };

        assert!(creeper_fields.contains(&("powered".to_string(), Tag::Byte(0))));
        assert!(creeper_fields.contains(&("Fuse".to_string(), Tag::Short(30))));
        assert!(creeper_fields.contains(&("ExplosionRadius".to_string(), Tag::Byte(3))));
        assert!(creeper_fields.contains(&("ignited".to_string(), Tag::Byte(0))));
        assert!(slime_fields.contains(&("Size".to_string(), Tag::Int(0))));
        assert!(slime_fields.contains(&("wasOnGround".to_string(), Tag::Byte(0))));
        assert!(ravager_fields.contains(&("AttackTick".to_string(), Tag::Int(0))));
        assert!(ravager_fields.contains(&("StunTick".to_string(), Tag::Int(0))));
        assert!(ravager_fields.contains(&("RoarTick".to_string(), Tag::Int(0))));
        assert!(ghast_fields.contains(&("ExplosionPower".to_string(), Tag::Byte(1))));
        assert!(endermite_fields.contains(&("Lifetime".to_string(), Tag::Int(0))));
        assert!(zoglin_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_zombie_piglin_and_skeleton_save_fields() {
        let zombie = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie_villager = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie_villager",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let skeleton = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:skeleton",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let bogged = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:bogged",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let piglin = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:piglin",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let hoglin = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:hoglin",
            width: 1.3965,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(zombie_fields) =
            super::super::chunk_generation_mob_entity_nbt(zombie, "00000000-0000-0000-0000-000000000152")
        else {
            panic!("zombie entity nbt must be a compound");
        };
        let Tag::Compound(zombie_villager_fields) = super::super::chunk_generation_mob_entity_nbt(
            zombie_villager,
            "00000000-0000-0000-0000-000000000153",
        ) else {
            panic!("zombie villager entity nbt must be a compound");
        };
        let Tag::Compound(skeleton_fields) = super::super::chunk_generation_mob_entity_nbt(
            skeleton,
            "00000000-0000-0000-0000-000000000154",
        ) else {
            panic!("skeleton entity nbt must be a compound");
        };
        let Tag::Compound(bogged_fields) =
            super::super::chunk_generation_mob_entity_nbt(bogged, "00000000-0000-0000-0000-000000000155")
        else {
            panic!("bogged entity nbt must be a compound");
        };
        let Tag::Compound(piglin_fields) =
            super::super::chunk_generation_mob_entity_nbt(piglin, "00000000-0000-0000-0000-000000000156")
        else {
            panic!("piglin entity nbt must be a compound");
        };
        let Tag::Compound(hoglin_fields) =
            super::super::chunk_generation_mob_entity_nbt(hoglin, "00000000-0000-0000-0000-000000000157")
        else {
            panic!("hoglin entity nbt must be a compound");
        };

        assert!(zombie_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
        assert!(zombie_fields.contains(&("CanBreakDoors".to_string(), Tag::Byte(0))));
        assert!(zombie_fields.contains(&("InWaterTime".to_string(), Tag::Int(-1))));
        assert!(zombie_fields.contains(&("DrownedConversionTime".to_string(), Tag::Int(-1))));
        assert!(
            zombie_villager_fields.contains(&("VillagerDataFinalized".to_string(), Tag::Byte(0)))
        );
        assert!(zombie_villager_fields.contains(&("ConversionTime".to_string(), Tag::Int(-1))));
        assert!(zombie_villager_fields.contains(&("Xp".to_string(), Tag::Int(0))));
        assert!(skeleton_fields.contains(&("StrayConversionTime".to_string(), Tag::Int(-1))));
        assert!(bogged_fields.contains(&("sheared".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("IsImmuneToZombification".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("TimeInOverworld".to_string(), Tag::Int(0))));
        assert!(piglin_fields.contains(&("IsBaby".to_string(), Tag::Byte(0))));
        assert!(piglin_fields.contains(&("CannotHunt".to_string(), Tag::Byte(0))));
        assert!(hoglin_fields.contains(&("IsImmuneToZombification".to_string(), Tag::Byte(0))));
        assert!(hoglin_fields.contains(&("TimeInOverworld".to_string(), Tag::Int(0))));
        assert!(hoglin_fields.contains(&("CannotBeHunted".to_string(), Tag::Byte(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_stable_animal_specific_save_fields() {
        let sheep = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:sheep",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let cat = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:cat",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let chicken = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:chicken",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let goat = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:goat",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let rabbit = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:rabbit",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(sheep_fields) =
            super::super::chunk_generation_mob_entity_nbt(sheep, "00000000-0000-0000-0000-000000000125")
        else {
            panic!("sheep entity nbt must be a compound");
        };
        let Tag::Compound(cat_fields) =
            super::super::chunk_generation_mob_entity_nbt(cat, "00000000-0000-0000-0000-000000000126")
        else {
            panic!("cat entity nbt must be a compound");
        };
        let Tag::Compound(chicken_fields) =
            super::super::chunk_generation_mob_entity_nbt(chicken, "00000000-0000-0000-0000-000000000127")
        else {
            panic!("chicken entity nbt must be a compound");
        };
        let Tag::Compound(goat_fields) =
            super::super::chunk_generation_mob_entity_nbt(goat, "00000000-0000-0000-0000-000000000128")
        else {
            panic!("goat entity nbt must be a compound");
        };
        let Tag::Compound(rabbit_fields) =
            super::super::chunk_generation_mob_entity_nbt(rabbit, "00000000-0000-0000-0000-000000000129")
        else {
            panic!("rabbit entity nbt must be a compound");
        };

        assert!(sheep_fields.contains(&("Sheared".to_string(), Tag::Byte(0))));
        assert!(sheep_fields.contains(&("Color".to_string(), Tag::Byte(0))));
        assert!(cat_fields.contains(&("CollarColor".to_string(), Tag::Byte(14))));
        assert!(cat_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:black".to_string())
        )));
        assert!(cat_fields.contains(&(
            "sound_variant".to_string(),
            Tag::String("minecraft:classic".to_string())
        )));
        assert!(chicken_fields.contains(&("IsChickenJockey".to_string(), Tag::Byte(0))));
        assert!(chicken_fields.contains(&("EggLayTime".to_string(), Tag::Int(6000))));
        assert!(goat_fields.contains(&("IsScreamingGoat".to_string(), Tag::Byte(0))));
        assert!(goat_fields.contains(&("HasLeftHorn".to_string(), Tag::Byte(1))));
        assert!(goat_fields.contains(&("HasRightHorn".to_string(), Tag::Byte(1))));
        assert!(rabbit_fields.contains(&("RabbitType".to_string(), Tag::Int(0))));
        assert!(rabbit_fields.contains(&("MoreCarrotTicks".to_string(), Tag::Int(0))));
    }

    #[test]
    fn chunk_generation_mob_entity_nbt_adds_variant_save_fields() {
        let cow = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:cow",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let pig = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:pig",
            width: 0.9,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let chicken = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:chicken",
            width: 0.4,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let frog = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:frog",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let axolotl = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:axolotl",
            width: 0.75,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let salmon = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:salmon",
            width: 0.7,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let tropical_fish = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:tropical_fish",
            width: 0.5,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let wolf = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:wolf",
            width: 0.6,
            x: 32.5,
            y: 70.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };
        let zombie_nautilus = super::super::ChunkGenerationMobEntitySnapPlan {
            entity_type: "minecraft:zombie_nautilus",
            width: 1.2,
            x: 32.5,
            y: 62.0,
            z: -33.5,
            yaw: 0.0,
            pitch: 0.0,
        };

        let Tag::Compound(cow_fields) =
            super::super::chunk_generation_mob_entity_nbt(cow, "00000000-0000-0000-0000-000000000158")
        else {
            panic!("cow entity nbt must be a compound");
        };
        let Tag::Compound(pig_fields) =
            super::super::chunk_generation_mob_entity_nbt(pig, "00000000-0000-0000-0000-000000000159")
        else {
            panic!("pig entity nbt must be a compound");
        };
        let Tag::Compound(chicken_fields) =
            super::super::chunk_generation_mob_entity_nbt(chicken, "00000000-0000-0000-0000-000000000160")
        else {
            panic!("chicken entity nbt must be a compound");
        };
        let Tag::Compound(frog_fields) =
            super::super::chunk_generation_mob_entity_nbt(frog, "00000000-0000-0000-0000-000000000161")
        else {
            panic!("frog entity nbt must be a compound");
        };
        let Tag::Compound(axolotl_fields) =
            super::super::chunk_generation_mob_entity_nbt(axolotl, "00000000-0000-0000-0000-000000000162")
        else {
            panic!("axolotl entity nbt must be a compound");
        };
        let Tag::Compound(salmon_fields) =
            super::super::chunk_generation_mob_entity_nbt(salmon, "00000000-0000-0000-0000-000000000163")
        else {
            panic!("salmon entity nbt must be a compound");
        };
        let Tag::Compound(tropical_fish_fields) = super::super::chunk_generation_mob_entity_nbt(
            tropical_fish,
            "00000000-0000-0000-0000-000000000164",
        ) else {
            panic!("tropical fish entity nbt must be a compound");
        };
        let Tag::Compound(wolf_fields) =
            super::super::chunk_generation_mob_entity_nbt(wolf, "00000000-0000-0000-0000-000000000165")
        else {
            panic!("wolf entity nbt must be a compound");
        };
        let Tag::Compound(zombie_nautilus_fields) = super::super::chunk_generation_mob_entity_nbt(
            zombie_nautilus,
            "00000000-0000-0000-0000-000000000166",
        ) else {
            panic!("zombie nautilus entity nbt must be a compound");
        };

        for fields in [&cow_fields, &pig_fields, &chicken_fields, &frog_fields] {
            assert!(fields.contains(&(
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string())
            )));
        }
        for fields in [&cow_fields, &pig_fields, &chicken_fields] {
            assert!(fields.contains(&(
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string())
            )));
        }
        assert!(axolotl_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(axolotl_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(salmon_fields.contains(&("type".to_string(), Tag::String("medium".to_string()))));
        assert!(salmon_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(tropical_fish_fields.contains(&("Variant".to_string(), Tag::Int(0))));
        assert!(tropical_fish_fields.contains(&("FromBucket".to_string(), Tag::Byte(0))));
        assert!(wolf_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:pale".to_string())
        )));
        assert!(wolf_fields.contains(&(
            "sound_variant".to_string(),
            Tag::String("minecraft:classic".to_string())
        )));
        assert!(zombie_nautilus_fields.contains(&(
            "variant".to_string(),
            Tag::String("minecraft:temperate".to_string())
        )));
    }

