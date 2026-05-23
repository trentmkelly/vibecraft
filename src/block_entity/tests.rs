use super::*;

fn pos() -> BlockPos {
    BlockPos {
        x: 18,
        y: 64,
        z: 35,
    }
}

fn stack(item_id: &str, count: i32) -> PotItemStack {
    PotItemStack {
        item_id: item_id.to_string(),
        count,
    }
}

fn assert_menu(
    menu: &BlockEntityMenuOpen,
    container_id: i32,
    menu_type: &'static str,
    initial_slots: &[Option<PotItemStack>],
) {
    assert_eq!(menu.container_id, container_id);
    assert_eq!(menu.menu_type, menu_type);
    assert_eq!(menu.initial_slots, initial_slots);
}

#[test]
fn block_entity_registry_matches_26_1_2_type_surface() {
    assert_eq!(BLOCK_ENTITY_TYPES.len(), 49);
    assert_eq!(type_info(BlockEntityTypeId::Furnace).key, "furnace");
    assert_eq!(
        type_info(BlockEntityTypeId::CopperGolemStatue).key,
        "copper_golem_statue"
    );
    assert_eq!(
        type_info(BlockEntityTypeId::CopperGolemStatue).valid_blocks,
        &[
            "minecraft:copper_golem_statue",
            "minecraft:exposed_copper_golem_statue",
            "minecraft:weathered_copper_golem_statue",
            "minecraft:oxidized_copper_golem_statue",
            "minecraft:waxed_copper_golem_statue",
            "minecraft:waxed_exposed_copper_golem_statue",
            "minecraft:waxed_weathered_copper_golem_statue",
            "minecraft:waxed_oxidized_copper_golem_statue",
        ]
    );
    assert!(is_valid_block_state(
        BlockEntityTypeId::Sign,
        "minecraft:oak_wall_sign"
    ));
    assert!(!is_valid_block_state(
        BlockEntityTypeId::Sign,
        "minecraft:stone"
    ));
}

#[test]
fn op_only_custom_data_matches_vanilla_guarded_types() {
    assert!(only_op_can_set_nbt(BlockEntityTypeId::CommandBlock));
    assert!(only_op_can_set_nbt(BlockEntityTypeId::Lectern));
    assert!(only_op_can_set_nbt(BlockEntityTypeId::Sign));
    assert!(only_op_can_set_nbt(BlockEntityTypeId::HangingSign));
    assert!(only_op_can_set_nbt(BlockEntityTypeId::MobSpawner));
    assert!(only_op_can_set_nbt(BlockEntityTypeId::TrialSpawner));
    assert!(!only_op_can_set_nbt(BlockEntityTypeId::Vault));
}

#[test]
fn validates_block_state_on_creation_and_load() {
    assert!(BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").is_ok());
    assert_eq!(
        BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:furnace"),
        Err(BlockEntityError::InvalidBlockState {
            ty: BlockEntityTypeId::Chest,
            block_state: "minecraft:furnace".to_string()
        })
    );
}

#[test]
fn detects_block_entity_support_for_block_states() {
    assert!(has_block_entity_for_block("minecraft:chest"));
    assert!(has_block_entity_for_block("minecraft:oak_sign"));
    assert!(has_block_entity_for_block("minecraft:oak_hanging_sign"));
    assert!(has_block_entity_for_block("minecraft:lectern"));
    assert!(has_block_entity_for_block("minecraft:command_block"));
    assert!(has_block_entity_for_block(
        "minecraft:wither_skeleton_skull"
    ));
    assert!(has_block_entity_for_block("minecraft:red_banner"));
    assert!(has_block_entity_for_block("minecraft:conduit"));
    assert!(has_block_entity_for_block("minecraft:bell"));
    assert!(has_block_entity_for_block("minecraft:crimson_hanging_sign"));
    assert!(has_block_entity_for_block("minecraft:brown_banner"));
    assert!(has_block_entity_for_block("minecraft:waxed_copper_chest"));
    assert!(has_block_entity_for_block("minecraft:dark_oak_wall_sign"));
    assert!(has_block_entity_for_block("minecraft:spawner"));
    assert!(has_block_entity_for_block("minecraft:vault"));
    assert!(has_block_entity_for_block("minecraft:trial_spawner"));
    assert!(has_block_entity_for_block(
        "minecraft:calibrated_sculk_sensor"
    ));
    assert!(has_block_entity_for_block("minecraft:chiseled_bookshelf"));
    assert!(has_block_entity_for_block("minecraft:suspicious_sand"));
    assert!(has_block_entity_for_block("minecraft:green_bed"));
    assert!(has_block_entity_for_block("minecraft:black_shulker_box"));
    assert!(has_block_entity_for_block("minecraft:crimson_shelf"));
    assert!(has_block_entity_for_block(
        "minecraft:waxed_oxidized_copper_golem_statue"
    ));
    assert!(has_block_entity_for_block(
        "minecraft:warped_wall_hanging_sign"
    ));
    assert!(has_block_entity_for_block("minecraft:campfire"));
    assert!(!has_block_entity_for_block("minecraft:candle"));
    assert!(!has_block_entity_for_block("minecraft:cauldron"));
    assert!(!has_block_entity_for_block("minecraft:stone"));
    assert!(!has_block_entity_for_block("minecraft:dirt"));
}

#[test]
fn bed_block_entity_is_color_only_placeholder() {
    assert_eq!(
        BedBlockEntity::from_block_state("minecraft:white_bed"),
        Some(BedBlockEntity {
            color: DyeColor::White
        })
    );
    assert_eq!(
        BedBlockEntity::from_block_state("minecraft:light_blue_bed"),
        Some(BedBlockEntity {
            color: DyeColor::LightBlue
        })
    );
    assert_eq!(
        BedBlockEntity::from_block_state("minecraft:black_bed"),
        Some(BedBlockEntity {
            color: DyeColor::Black
        })
    );
    assert_eq!(BedBlockEntity::from_block_state("minecraft:stone"), None);

    let bed = BlockEntity::new(BlockEntityTypeId::Bed, pos(), "minecraft:red_bed").unwrap();
    assert_eq!(bed.ty, BlockEntityTypeId::Bed);
    assert_eq!(
        BedBlockEntity::from_block_state(&bed.block_state),
        Some(BedBlockEntity {
            color: DyeColor::Red
        })
    );
    assert_eq!(
        BedBlockEntity::from_block_state(&bed.block_state)
            .unwrap()
            .save_additional(),
        Tag::Compound(Vec::new())
    );
    assert_eq!(
        bed.save_with_full_metadata(),
        Tag::Compound(vec![
            ("components".to_string(), Tag::Compound(Vec::new())),
            ("id".to_string(), Tag::String("bed".to_string())),
            ("x".to_string(), Tag::Int(pos().x)),
            ("y".to_string(), Tag::Int(pos().y)),
            ("z".to_string(), Tag::Int(pos().z)),
        ])
    );
}

#[test]
fn end_portal_block_entity_is_zero_data_portal_placeholder() {
    let portal =
        BlockEntity::new(BlockEntityTypeId::EndPortal, pos(), "minecraft:end_portal").unwrap();
    assert_eq!(portal.ty, BlockEntityTypeId::EndPortal);
    assert_eq!(type_info(BlockEntityTypeId::EndPortal).key, "end_portal");
    assert_eq!(
        type_info(BlockEntityTypeId::EndPortal).valid_blocks,
        &["minecraft:end_portal"]
    );
    assert_eq!(
        EndPortalBlockEntity.save_additional(),
        Tag::Compound(Vec::new())
    );
    assert_eq!(
        portal.save_with_full_metadata(),
        Tag::Compound(vec![
            ("components".to_string(), Tag::Compound(Vec::new())),
            ("id".to_string(), Tag::String("end_portal".to_string())),
            ("x".to_string(), Tag::Int(pos().x)),
            ("y".to_string(), Tag::Int(pos().y)),
            ("z".to_string(), Tag::Int(pos().z)),
        ])
    );
}

#[test]
fn end_gateway_block_entity_saves_ticks_cooldown_and_exit_like_java() {
    let mut gateway = TheEndGatewayBlockEntity::new();
    assert!(gateway.is_spawning());
    assert!(!gateway.is_cooling_down());
    assert_eq!(gateway.spawn_percent(0.0), 0.0);
    assert_eq!(TheEndGatewayBlockEntity::SPAWN_TIME, 200);
    assert_eq!(TheEndGatewayBlockEntity::COOLDOWN_TIME, 40);
    assert_eq!(TheEndGatewayBlockEntity::ATTENTION_INTERVAL, 2400);
    assert_eq!(TheEndGatewayBlockEntity::GATEWAY_HEIGHT_ABOVE_SURFACE, 10);

    gateway.age = 199;
    assert_eq!(gateway.spawn_percent(0.5), 0.9975);
    assert!(gateway.portal_tick());
    assert_eq!(gateway.age, 200);
    assert!(!gateway.is_spawning());

    gateway.set_exit_position(
        BlockPos {
            x: 12,
            y: 80,
            z: -7,
        },
        true,
    );
    let saved = gateway.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            ("Age".to_string(), Tag::Long(200)),
            (
                "exit_portal".to_string(),
                Tag::List(vec![Tag::Int(12), Tag::Int(80), Tag::Int(-7)])
            ),
            ("ExactTeleport".to_string(), Tag::Byte(1)),
        ])
    );
    assert_eq!(TheEndGatewayBlockEntity::load_additional(&saved), gateway);
    assert_eq!(gateway.get_update_tag(), saved);

    gateway.trigger_cooldown();
    assert!(gateway.is_cooling_down());
    assert_eq!(gateway.teleport_cooldown, 40);
    assert_eq!(gateway.cooldown_percent(0.0), 0.0);
    gateway.beam_animation_tick();
    assert_eq!(gateway.age, 201);
    assert_eq!(gateway.teleport_cooldown, 39);
    assert!((gateway.cooldown_percent(0.0) - 0.025).abs() < f32::EPSILON * 4.0);
    assert!(gateway.trigger_event(TheEndGatewayBlockEntity::EVENT_COOLDOWN));
    assert_eq!(gateway.teleport_cooldown, 40);
    assert!(!gateway.trigger_event(99));

    let mut attention = TheEndGatewayBlockEntity {
        age: 2399,
        ..TheEndGatewayBlockEntity::new()
    };
    assert!(attention.portal_tick());
    assert_eq!(attention.age, 2400);
    assert_eq!(attention.teleport_cooldown, 40);
}

#[test]
fn jigsaw_block_entity_saves_priorities_joint_and_generation_plan_like_java() {
    let mut jigsaw = JigsawBlockEntity::new();
    assert_eq!(jigsaw.name, JigsawBlockEntity::EMPTY_ID);
    assert_eq!(jigsaw.target, JigsawBlockEntity::EMPTY_ID);
    assert_eq!(jigsaw.pool, JigsawBlockEntity::EMPTY_ID);
    assert_eq!(jigsaw.final_state, JigsawBlockEntity::DEFAULT_FINAL_STATE);
    assert_eq!(jigsaw.joint, JigsawJointType::Rollable);

    jigsaw.name = "minecraft:house/start".to_string();
    jigsaw.target = "minecraft:house/door".to_string();
    jigsaw.pool = "minecraft:village/plains/houses".to_string();
    jigsaw.final_state = "minecraft:oak_planks".to_string();
    jigsaw.joint = JigsawJointType::Aligned;
    jigsaw.placement_priority = 7;
    jigsaw.selection_priority = -3;

    let saved = jigsaw.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            (
                "name".to_string(),
                Tag::String("minecraft:house/start".to_string())
            ),
            (
                "target".to_string(),
                Tag::String("minecraft:house/door".to_string())
            ),
            (
                "pool".to_string(),
                Tag::String("minecraft:village/plains/houses".to_string())
            ),
            (
                "final_state".to_string(),
                Tag::String("minecraft:oak_planks".to_string())
            ),
            ("joint".to_string(), Tag::String("aligned".to_string())),
            ("placement_priority".to_string(), Tag::Int(7)),
            ("selection_priority".to_string(), Tag::Int(-3)),
        ])
    );
    assert_eq!(JigsawBlockEntity::load_additional(&saved), jigsaw);
    assert_eq!(jigsaw.get_update_tag(), saved);

    let plan =
        jigsaw.generation_plan(BlockPos { x: 4, y: 70, z: 8 }, Direction::North, 5, true);
    assert_eq!(
        plan,
        JigsawGenerationPlan {
            pool: "minecraft:village/plains/houses".to_string(),
            target: "minecraft:house/door".to_string(),
            levels: 5,
            keep_jigsaws: true,
            start_pos: BlockPos { x: 4, y: 70, z: 7 },
        }
    );

    let defaults = JigsawBlockEntity::load_additional(&Tag::Compound(vec![(
        "joint".to_string(),
        Tag::String("unknown".to_string()),
    )]));
    assert_eq!(defaults.name, JigsawBlockEntity::EMPTY_ID);
    assert_eq!(defaults.joint, JigsawJointType::Rollable);
    assert_eq!(defaults.placement_priority, 0);
    assert_eq!(defaults.selection_priority, 0);
}

#[test]
fn comparator_block_entity_persists_output_and_uses_compare_subtract_logic() {
    let mut compare = ComparatorBlockEntity::new(ComparatorMode::Compare);
    assert_eq!(
        compare.save_additional(),
        Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(0))])
    );
    assert!(compare.update_output(12, 7));
    assert_eq!(compare.output_signal, 12);
    assert!(!compare.update_output(12, 7));
    assert_eq!(compare.calculate_output(3, 10), 0);

    let saved = compare.save_additional();
    assert_eq!(
        ComparatorBlockEntity::load_additional(ComparatorMode::Compare, &saved),
        compare
    );
    assert_eq!(
        ComparatorBlockEntity::load_additional(
            ComparatorMode::Subtract,
            &Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(99))]),
        )
        .output_signal,
        99
    );

    let mut subtract = ComparatorBlockEntity::new(ComparatorMode::Subtract);
    assert!(subtract.update_output(12, 7));
    assert_eq!(subtract.output_signal, 5);
    assert!(subtract.update_output(3, 10));
    assert_eq!(subtract.output_signal, 0);
}

#[test]
fn daylight_detector_updates_power_with_vanilla_solar_math_and_tick_cadence() {
    let mut normal = DaylightDetectorBlockEntity::new(false);
    assert_eq!(normal.save_additional(), Tag::Compound(Vec::new()));
    assert!(!normal.tick(19, 15, 0.0));
    assert_eq!(normal.power, 0);
    assert!(normal.tick(20, 15, 0.0));
    assert_eq!(normal.power, 15);
    assert!(normal.update_signal(15, 180.0));
    assert_eq!(normal.power, 0);
    assert!(!normal.update_signal(-4, 0.0));
    assert_eq!(normal.power, 0);

    assert_eq!(
        DaylightDetectorBlockEntity::calculate_power(false, 10, 90.0),
        3
    );
    assert_eq!(
        DaylightDetectorBlockEntity::calculate_power(false, 99, 0.0),
        15
    );

    let mut inverted = DaylightDetectorBlockEntity::new(true);
    assert!(inverted.update_signal(4, 90.0));
    assert_eq!(inverted.power, 11);
    assert!(inverted.update_signal(99, 0.0));
    assert_eq!(inverted.power, 0);
}

#[test]
fn block_entity_comparator_outputs_cover_boundary_states() {
    assert_eq!(inventory_comparator_output(&[]), 0);
    assert_eq!(inventory_comparator_output(&[None]), 0);
    assert_eq!(
        inventory_comparator_output(&[Some(stack("minecraft:stone", 1))]),
        1
    );
    assert_eq!(
        inventory_comparator_output(&[Some(stack("minecraft:stone", 64))]),
        MAX_SIGNAL
    );

    let mut furnace = AbstractFurnaceBlockEntity::new(FurnaceBlockEntityKind::Furnace);
    assert_eq!(furnace.comparator_output(), 0);
    furnace.set_item(
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        Some(stack("minecraft:iron_ore", 1)),
        None,
    );
    assert_eq!(furnace.comparator_output(), 1);
    for slot in 0..AbstractFurnaceBlockEntity::SLOT_COUNT {
        furnace.set_item(slot, Some(stack("minecraft:stone", 64)), None);
    }
    assert_eq!(furnace.comparator_output(), MAX_SIGNAL);

    let mut brewing = BrewingStandBlockEntity::new();
    assert_eq!(brewing.comparator_output(), 0);
    brewing.set_item(0, Some(stack("minecraft:potion", 1)));
    assert_eq!(brewing.comparator_output(), 1);
    for slot in 0..BrewingStandBlockEntity::CONTAINER_SIZE {
        brewing.set_item(slot, Some(stack("minecraft:potion", 64)));
    }
    assert_eq!(brewing.comparator_output(), MAX_SIGNAL);

    for kind in [
        ContainerBlockEntityKind::Chest,
        ContainerBlockEntityKind::TrappedChest,
        ContainerBlockEntityKind::Barrel,
        ContainerBlockEntityKind::ShulkerBox,
        ContainerBlockEntityKind::Dispenser,
        ContainerBlockEntityKind::Dropper,
        ContainerBlockEntityKind::Hopper,
    ] {
        let mut container = ContainerBlockEntityModel::new(kind);
        assert_eq!(container.comparator_output(), 0, "{kind:?} empty");
        container.set_item(0, Some(stack("minecraft:stone", 1)));
        assert!(container.comparator_output() > 0, "{kind:?} partial");
        for slot in 0..container.items.len() {
            container.set_item(slot, Some(stack("minecraft:stone", 64)));
        }
        assert_eq!(container.comparator_output(), MAX_SIGNAL, "{kind:?} full");
    }

    let mut trapped = ContainerBlockEntityModel::new(ContainerBlockEntityKind::TrappedChest);
    trapped.viewer_count = 0;
    assert_eq!(trapped.trapped_chest_signal(), 0);
    trapped.viewer_count = i32::from(MAX_SIGNAL);
    assert_eq!(trapped.trapped_chest_signal(), MAX_SIGNAL);
    trapped.viewer_count = i32::from(MAX_SIGNAL) + 1;
    assert_eq!(trapped.trapped_chest_signal(), MAX_SIGNAL);

    let mut jukebox = JukeboxBlockEntity::new();
    assert_eq!(jukebox.comparator_output(), 0);
    assert_eq!(jukebox.redstone_signal(), 0);
    jukebox.set_the_item(Some(stack("minecraft:music_disc_13", 1)));
    assert_eq!(jukebox.comparator_output(), 1);
    assert_eq!(jukebox.redstone_signal(), MAX_SIGNAL);
    jukebox.set_the_item(Some(stack("minecraft:music_disc_5", 1)));
    assert_eq!(jukebox.comparator_output(), MAX_SIGNAL);

    let mut shelf = ShelfBlockEntity::new();
    assert_eq!(shelf.comparator_output(), 0);
    shelf.set_item_no_update(0, Some(stack("minecraft:book", 1)));
    assert_eq!(shelf.comparator_output(), 1);
    for slot in 0..ShelfBlockEntity::MAX_ITEMS {
        shelf.set_item_no_update(slot, Some(stack("minecraft:book", 1)));
    }
    assert_eq!(shelf.comparator_output(), 3);

    let mut beacon = BeaconBlockEntity::new();
    beacon.levels = -1;
    assert_eq!(beacon.comparator_output(), 0);
    beacon.levels = BeaconBlockEntity::MAX_LEVELS;
    assert_eq!(beacon.comparator_output(), 4);
    beacon.levels = 99;
    assert_eq!(beacon.comparator_output(), 4);

    let mut lectern = LecternBlockEntity::new();
    assert_eq!(lectern.get_redstone_signal(), 0);
    lectern.set_book(Some(stack("minecraft:written_book", 1)), 4);
    assert_eq!(lectern.get_redstone_signal(), 1);
    lectern.set_page(3);
    assert_eq!(lectern.get_redstone_signal(), MAX_SIGNAL);

    let mut crafter = CrafterBlockEntity::new();
    assert_eq!(crafter.redstone_signal(), 0);
    for slot in 0..CrafterBlockEntity::CONTAINER_SIZE {
        crafter.set_slot_state(slot, false);
    }
    assert_eq!(crafter.redstone_signal(), 9);

    let mut pot = DecoratedPotBlockEntity::default();
    assert_eq!(pot.comparator_output(), 0);
    pot.item = Some(stack("minecraft:diamond", 1));
    assert_eq!(pot.comparator_output(), 1);
    pot.item = Some(stack("minecraft:diamond", 64));
    assert_eq!(pot.comparator_output(), MAX_SIGNAL);

    let mut statue = CopperGolemStatueBlockEntity::from_block_state(
        "minecraft:copper_golem_statue",
        CopperGolemStatuePose::Standing,
    )
    .unwrap();
    assert_eq!(statue.comparator_output(), 1);
    statue.update_pose();
    assert_eq!(statue.comparator_output(), 2);
    statue.update_pose();
    assert_eq!(statue.comparator_output(), 3);
    statue.update_pose();
    assert_eq!(statue.comparator_output(), 4);

    let mut heart = CreakingHeartBlockEntity::new();
    assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 0);
    heart.set_creaking_uuid("protector".to_string());
    assert_eq!(heart.compute_analog_output_signal(None), 0);
    assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 15);
    assert_eq!(
        heart.compute_analog_output_signal(Some(f64::from(
            CreakingHeartBlockEntity::CREAKING_ROAMING_RADIUS
        ))),
        0
    );
}

#[test]
fn gui_block_entities_open_with_menu_id_initial_slots_and_close_state() {
    let mut furnace = AbstractFurnaceBlockEntity::furnace();
    furnace.set_item(
        AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
        Some(stack("minecraft:iron_ore", 3)),
        None,
    );
    assert_menu(
        &furnace.open_menu(1),
        1,
        "furnace",
        &[Some(stack("minecraft:iron_ore", 3)), None, None],
    );

    let mut blast = AbstractFurnaceBlockEntity::blast_furnace();
    blast.set_item(
        AbstractFurnaceBlockEntity::FUEL_SLOT,
        Some(stack("minecraft:coal", 2)),
        None,
    );
    assert_menu(
        &blast.open_menu(2),
        2,
        "blast_furnace",
        &[None, Some(stack("minecraft:coal", 2)), None],
    );
    assert_menu(
        &AbstractFurnaceBlockEntity::smoker().open_menu(3),
        3,
        "smoker",
        &[None, None, None],
    );

    let ender_slots = vec![Some(stack("minecraft:ender_pearl", 16)); 27];
    let ender_menu = open_ender_chest_menu(4, ender_slots.clone());
    assert_menu(&ender_menu, 4, "generic_9x3", &ender_slots);
    assert_eq!(ender_menu.close(), BlockEntityMenuClose { container_id: 4 });

    for (index, (kind, menu_type, slot_count)) in [
        (ContainerBlockEntityKind::Chest, "generic_9x3", 27),
        (ContainerBlockEntityKind::TrappedChest, "generic_9x3", 27),
        (ContainerBlockEntityKind::Barrel, "generic_9x3", 27),
        (ContainerBlockEntityKind::ShulkerBox, "shulker_box", 27),
        (ContainerBlockEntityKind::Dispenser, "generic_3x3", 9),
        (ContainerBlockEntityKind::Dropper, "generic_3x3", 9),
        (ContainerBlockEntityKind::Hopper, "hopper", 5),
    ]
    .into_iter()
    .enumerate()
    {
        let mut container = ContainerBlockEntityModel::new(kind);
        container.lock_key = Some("key".to_string());
        assert_eq!(container.open_menu(20 + index as i32, None, false), None);
        container.set_item(0, Some(stack("minecraft:apple", 5)));

        let menu = container
            .open_menu(20 + index as i32, Some("key"), false)
            .unwrap();
        let mut expected = vec![None; slot_count];
        expected[0] = Some(stack("minecraft:apple", 5));
        assert_menu(&menu, 20 + index as i32, menu_type, &expected);
        assert_eq!(container.viewer_count, 1);
        let close = container.close_menu(menu);
        assert_eq!(
            close,
            BlockEntityMenuClose {
                container_id: 20 + index as i32
            }
        );
        assert_eq!(container.viewer_count, 0);
    }

    let enchantment = EnchantingTableBlockEntity::new();
    assert_menu(&enchantment.open_menu(40), 40, "enchantment", &[None, None]);

    let mut brewing = BrewingStandBlockEntity::new();
    brewing.set_item(0, Some(stack("minecraft:potion", 1)));
    assert_menu(
        &brewing.open_menu(41),
        41,
        "brewing_stand",
        &[Some(stack("minecraft:potion", 1)), None, None, None, None],
    );

    let mut beacon = BeaconBlockEntity::new();
    assert!(beacon.set_payment_item(Some(stack("minecraft:emerald", 1))));
    assert_menu(
        &beacon.open_menu(42),
        42,
        "beacon",
        &[Some(stack("minecraft:emerald", 1))],
    );

    let mut lectern = LecternBlockEntity::new();
    lectern.set_book(Some(stack("minecraft:written_book", 1)), 3);
    assert_menu(
        &lectern.open_menu(43),
        43,
        "lectern",
        &[Some(stack("minecraft:written_book", 1))],
    );

    let mut crafter = CrafterBlockEntity::new();
    crafter.set_item(8, Some(stack("minecraft:redstone", 4)));
    let mut crafter_slots = vec![None; CrafterBlockEntity::CONTAINER_SIZE + 1];
    crafter_slots[8] = Some(stack("minecraft:redstone", 4));
    assert_menu(&crafter.open_menu(44), 44, "crafter_3x3", &crafter_slots);
}

#[test]
fn command_block_entity_persists_base_fields_and_models_execution_gate() {
    let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, true);
    command.set_command("say hello");
    command.custom_name = Some("\"Runner\"".to_string());
    command.track_output = true;
    command.last_output = Some("\"previous\"".to_string());
    command.last_execution = 41;
    command.powered = true;
    command.automatic = true;
    command.mark_condition_met(true);

    let saved = command.save_additional();
    assert_eq!(
        CommandBlockEntity::load_additional(CommandBlockMode::Redstone, true, &saved),
        command
    );
    assert_eq!(
        command.execution_action(true, true),
        SpecialBlockAction::ExecuteCommand { success_count: 1 }
    );
    assert_eq!(command.perform_command(42, true, true, true), true);
    assert_eq!(command.success_count, 1);
    assert_eq!(command.last_execution, 42);
    assert_eq!(command.perform_command(42, true, true, true), false);

    let mut blocked = CommandBlockEntity::new(CommandBlockMode::Auto, false);
    blocked.set_command("say no");
    assert_eq!(
        blocked.execution_action(false, true),
        SpecialBlockAction::ExecuteCommand { success_count: 0 }
    );
    assert!(blocked.set_automatic(true, true));
    assert!(!blocked.set_automatic(true, true));
    assert!(blocked.can_use(true));
    assert!(!blocked.can_use(false));

    let mut searge = CommandBlockEntity::new(CommandBlockMode::Auto, false);
    searge.set_command("Searge");
    assert!(searge.perform_command(9, false, false, false));
    assert_eq!(searge.success_count, 1);
    assert_eq!(
        searge.last_output.as_deref(),
        Some(CommandBlockEntity::SEARGE_OUTPUT)
    );

    let loaded_without_tracking = CommandBlockEntity::load_additional(
        CommandBlockMode::Auto,
        false,
        &Tag::Compound(vec![
            ("TrackOutput".to_string(), Tag::Byte(0)),
            (
                "LastOutput".to_string(),
                Tag::String("\"ignored\"".to_string()),
            ),
            ("UpdateLastExecution".to_string(), Tag::Byte(0)),
            ("LastExecution".to_string(), Tag::Long(99)),
        ]),
    );
    assert_eq!(loaded_without_tracking.last_output, None);
    assert_eq!(
        loaded_without_tracking.last_execution,
        CommandBlockEntity::NO_LAST_EXECUTION
    );
}

#[test]
fn command_block_execution_uses_block_source_and_captures_output() {
    let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
    command.set_command("say hello");
    command.powered = true;

    let execution = command
        .execute_from_context(
            CommandBlockExecutionContext {
                pos: BlockPos { x: 4, y: 64, z: -2 },
                level: "minecraft:overworld".to_string(),
                game_time: 100,
                command_blocks_enabled: true,
                has_permission: true,
                previous_success: true,
            },
            Some("{\"text\":\"hello\"}".to_string()),
        )
        .expect("powered redstone command block should execute on the leading edge");

    assert_eq!(execution.command, "say hello");
    assert_eq!(execution.success_count, 1);
    assert_eq!(execution.output.as_deref(), Some("{\"text\":\"hello\"}"));
    assert_eq!(command.last_output.as_deref(), Some("{\"text\":\"hello\"}"));
    assert_eq!(command.last_execution, 100);
    assert_eq!(execution.source.source, "CommandBlockEntity");
    assert_eq!(execution.source.level, "minecraft:overworld");
    assert_eq!(execution.source.permission_level, 2);
    assert_eq!(
        execution.source.position,
        Vec3 {
            x: 4.5,
            y: 64.5,
            z: -1.5,
        }
    );

    assert!(
        command
            .execute_from_context(
                CommandBlockExecutionContext {
                    pos: BlockPos { x: 4, y: 64, z: -2 },
                    level: "minecraft:overworld".to_string(),
                    game_time: 100,
                    command_blocks_enabled: true,
                    has_permission: true,
                    previous_success: true,
                },
                Some("{\"text\":\"again\"}".to_string()),
            )
            .is_none(),
        "command block should not run twice in the same game tick"
    );

    let mut denied = CommandBlockEntity::new(CommandBlockMode::Auto, false);
    denied.set_command("say denied");
    let denied_execution = denied
        .execute_from_context(
            CommandBlockExecutionContext {
                pos: BlockPos { x: 0, y: 70, z: 0 },
                level: "minecraft:overworld".to_string(),
                game_time: 101,
                command_blocks_enabled: true,
                has_permission: false,
                previous_success: true,
            },
            Some("{\"text\":\"denied\"}".to_string()),
        )
        .expect("permission-denied command block records a zero-success execution");
    assert_eq!(denied_execution.success_count, 0);
    assert_eq!(denied.success_count, 0);
    assert_eq!(denied.last_output.as_deref(), Some("{\"text\":\"denied\"}"));
}

#[test]
fn command_block_editor_packet_and_client_update_require_permission() {
    let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
    command.set_command("say old");
    command.last_output = Some("{\"text\":\"old\"}".to_string());

    assert_eq!(command.open_editor_packet(pos(), false), None);
    let packet = command
        .open_editor_packet(pos(), true)
        .expect("operators can open command block editor data");
    assert_eq!(packet.pos, pos());
    assert_eq!(packet.ty, BlockEntityTypeId::CommandBlock);
    let packet_entries = compound_entries(&packet.tag).unwrap();
    assert_eq!(
        get_string(packet_entries, "Command"),
        Some("say old"),
        "editor packet carries the current command string"
    );
    assert_eq!(
        get_string(packet_entries, "LastOutput"),
        Some("{\"text\":\"old\"}")
    );

    assert!(!command.apply_client_update(
        CommandBlockUpdate {
            command: "say denied".to_string(),
            mode: CommandBlockMode::Auto,
            track_output: false,
            conditional: true,
            automatic: true,
        },
        false,
        true,
    ));
    assert_eq!(command.command, "say old");

    assert!(command.apply_client_update(
        CommandBlockUpdate {
            command: "say new".to_string(),
            mode: CommandBlockMode::Auto,
            track_output: false,
            conditional: true,
            automatic: true,
        },
        true,
        true,
    ));
    assert_eq!(command.command, "say new");
    assert_eq!(command.mode, CommandBlockMode::Auto);
    assert!(command.conditional);
    assert!(command.automatic);
    assert!(!command.track_output);
    assert_eq!(command.last_output, None);
}


mod tests_b;
mod tests_c;
mod tests_d;
mod tests_b2;
mod tests_c2;
