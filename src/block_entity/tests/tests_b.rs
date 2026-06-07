use super::super::*;
use super::*;

#[test]
fn command_block_chain_executes_facing_order_and_respects_conditional_flag() {
    let mut root = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
    root.set_command("say root");
    root.powered = true;

    let mut chain_one = CommandBlockEntity::new(CommandBlockMode::Sequence, false);
    chain_one.set_command("say first");
    chain_one.powered = true;

    let mut chain_two = CommandBlockEntity::new(CommandBlockMode::Sequence, true);
    chain_two.set_command("say second");
    chain_two.powered = true;

    let mut entries = vec![
        CommandBlockChainEntry {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            facing: Direction::East,
            block: root,
        },
        CommandBlockChainEntry {
            pos: BlockPos { x: 1, y: 64, z: 0 },
            facing: Direction::East,
            block: chain_one,
        },
        CommandBlockChainEntry {
            pos: BlockPos { x: 2, y: 64, z: 0 },
            facing: Direction::East,
            block: chain_two,
        },
    ];

    let steps = execute_command_block_chain(
        &mut entries,
        BlockPos { x: 0, y: 64, z: 0 },
        CommandBlockExecutionContext {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            level: "minecraft:overworld".to_string(),
            game_time: 200,
            command_blocks_enabled: true,
            has_permission: true,
            previous_success: true,
        },
    );
    assert_eq!(
        steps,
        vec![
            CommandBlockChainStep {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                command: "say root".to_string(),
                success_count: 1,
            },
            CommandBlockChainStep {
                pos: BlockPos { x: 1, y: 64, z: 0 },
                command: "say first".to_string(),
                success_count: 1,
            },
            CommandBlockChainStep {
                pos: BlockPos { x: 2, y: 64, z: 0 },
                command: "say second".to_string(),
                success_count: 1,
            },
        ]
    );

    entries[1].block.command.clear();
    entries[0].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
    entries[1].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
    entries[2].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
    let conditional_steps = execute_command_block_chain(
        &mut entries,
        BlockPos { x: 0, y: 64, z: 0 },
        CommandBlockExecutionContext {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            level: "minecraft:overworld".to_string(),
            game_time: 201,
            command_blocks_enabled: true,
            has_permission: true,
            previous_success: true,
        },
    );
    assert_eq!(
        conditional_steps
            .iter()
            .map(|step| step.success_count)
            .collect::<Vec<_>>(),
        vec![1, 0, 0]
    );
}

#[test]
fn jukebox_block_entity_tracks_disc_playback_ticks_and_outputs() {
    let mut jukebox = JukeboxBlockEntity::new();
    let disc = stack("minecraft:music_disc_13", 1);
    assert_jukebox_plays_and_persists_disc(&mut jukebox, &disc);
    assert_jukebox_rejects_second_disc_and_pops_item(&mut jukebox, disc);
    assert_jukebox_inert_item_paths();
    assert_jukebox_set_without_playing_path();
}

fn assert_jukebox_plays_and_persists_disc(jukebox: &mut JukeboxBlockEntity, disc: &PotItemStack) {
    assert!(jukebox.can_place_item(disc));
    assert_eq!(
        jukebox.set_the_item(Some(disc.clone())),
        JukeboxSongEvent::Started
    );
    assert!(jukebox.is_playing);
    assert_eq!(jukebox.redstone_signal(), 15);
    assert_eq!(jukebox.comparator_output(), 1);
    assert!(jukebox.tick());
    assert_eq!(jukebox.ticks_since_song_started, 1);

    let saved = jukebox.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            ("RecordItem".to_string(), disc.to_tag()),
            ("ticks_since_song_started".to_string(), Tag::Long(1)),
        ])
    );
    let loaded = JukeboxBlockEntity::load_additional(&saved);
    assert_eq!(loaded.item, Some(disc.clone()));
    assert_eq!(loaded.ticks_since_song_started, 1);
    assert!(!loaded.is_playing);
    assert_eq!(loaded.comparator_output(), 1);
}

fn assert_jukebox_rejects_second_disc_and_pops_item(
    jukebox: &mut JukeboxBlockEntity,
    disc: PotItemStack,
) {
    assert!(!jukebox.can_place_item(&PotItemStack {
        item_id: "minecraft:music_disc_5".to_string(),
        count: 1,
    }));
    assert_eq!(jukebox.pop_out_the_item(), Some(disc));
    assert!(!jukebox.is_playing);
    assert_eq!(jukebox.redstone_signal(), 0);
    assert_eq!(jukebox.comparator_output(), 0);
    assert_eq!(jukebox.save_additional(), Tag::Compound(Vec::new()));
}

fn assert_jukebox_inert_item_paths() {
    let mut inert = JukeboxBlockEntity::new();
    assert_eq!(
        inert.set_the_item(Some(PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 1,
        })),
        JukeboxSongEvent::Stopped
    );
    assert!(!inert.tick());
    assert_eq!(inert.comparator_output(), 0);
    assert!(!inert.can_place_item(&PotItemStack {
        item_id: "minecraft:diamond".to_string(),
        count: 1,
    }));
    assert!(inert.can_take_item(true));
    assert!(!inert.can_take_item(false));
}

fn assert_jukebox_set_without_playing_path() {
    let mut without_playing = JukeboxBlockEntity::new();
    assert_eq!(
        without_playing.set_song_item_without_playing(PotItemStack {
            item_id: "minecraft:music_disc_5".to_string(),
            count: 1,
        }),
        JukeboxSongEvent::ItemChanged
    );
    assert!(!without_playing.is_playing);
    assert_eq!(without_playing.redstone_signal(), 0);
    assert_eq!(without_playing.comparator_output(), 15);
}

#[test]
fn enchanting_table_saves_name_scans_bookshelves_and_animates_book_like_java() {
    let mut table = EnchantingTableBlockEntity::new();
    assert_enchanting_table_names_components_and_update_tag(&mut table);
    assert_enchanting_table_bookshelf_offsets_and_scan_cap();
    assert_enchanting_table_book_animation(&mut table);
}

fn assert_enchanting_table_names_components_and_update_tag(table: &mut EnchantingTableBlockEntity) {
    assert_eq!(
        table.display_name(),
        EnchantingTableBlockEntity::DEFAULT_NAME
    );
    table.custom_name = Some("\"Arcana\"".to_string());
    let saved = table.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![(
            "CustomName".to_string(),
            Tag::String("\"Arcana\"".to_string())
        )])
    );
    assert_eq!(
        EnchantingTableBlockEntity::load_additional(&saved).custom_name,
        Some("\"Arcana\"".to_string())
    );
    assert_eq!(table.get_update_tag(), saved);
    assert_eq!(
        table.collect_implicit_components(),
        Tag::Compound(vec![(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Arcana\"".to_string())
        )])
    );
    let mut from_components = EnchantingTableBlockEntity::new();
    from_components.apply_implicit_components(&table.collect_implicit_components());
    assert_eq!(from_components.custom_name, table.custom_name);
    assert_eq!(
        EnchantingTableBlockEntity::remove_components_from_tag(&saved),
        Tag::Compound(vec![])
    );
}

fn assert_enchanting_table_bookshelf_offsets_and_scan_cap() {
    let offsets = EnchantingTableBlockEntity::bookshelf_offsets();
    assert_eq!(offsets.len(), 32);
    assert!(offsets.contains(&BlockPos { x: -2, y: 0, z: 0 }));
    assert!(offsets.contains(&BlockPos { x: 2, y: 1, z: 2 }));
    let power = |pos: BlockPos| pos.x == 2 || pos.z == -2;
    let transmit = |_pos: BlockPos| true;
    assert_eq!(
        EnchantingTableBlockEntity::count_valid_bookshelves(power, transmit),
        15
    );
}

fn assert_enchanting_table_book_animation(table: &mut EnchantingTableBlockEntity) {
    table.book_animation_tick(Some((1.0, 0.0)), Some(2.0), false);
    assert_eq!(table.time, 1);
    assert_eq!(table.o_open, 0.0);
    assert_eq!(table.open, 0.1);
    assert_eq!(table.t_rot, 0.0);
    assert!(table.flip > 0.0);
    let previous_flip = table.flip;
    table.book_animation_tick(None, None, false);
    assert_eq!(table.time, 2);
    assert_eq!(table.o_open, 0.1);
    assert_eq!(table.open, 0.0);
    assert!(table.t_rot > 0.0);
    assert!(table.flip >= previous_flip);

    table.open = 0.8;
    let previous_target = table.flip_t;
    table.book_animation_tick(Some((0.0, 1.0)), Some(3.0), false);
    assert_eq!(table.flip_t, previous_target);
    table.book_animation_tick(Some((0.0, 1.0)), Some(3.0), true);
    assert_eq!(table.flip_t, previous_target + 3.0);

    table.rot = std::f32::consts::TAU;
    table.t_rot = -std::f32::consts::TAU;
    table.book_animation_tick(None, None, false);
    assert!(table.rot < std::f32::consts::PI);
    assert!(table.t_rot > -std::f32::consts::PI);
}

#[test]
fn shelf_block_entity_saves_three_items_align_flag_and_swaps_slots() {
    let mut shelf = ShelfBlockEntity::new();
    assert_eq!(shelf.items.len(), ShelfBlockEntity::MAX_ITEMS);
    assert_eq!(shelf.comparator_output(), 0);
    assert!(shelf.set_item_no_update(
        0,
        Some(PotItemStack {
            item_id: "minecraft:book".to_string(),
            count: 1,
        })
    ));
    assert!(shelf.set_item_no_update(
        2,
        Some(PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 3,
        })
    ));
    assert!(!shelf.set_item_no_update(
        3,
        Some(PotItemStack {
            item_id: "minecraft:apple".to_string(),
            count: 1,
        })
    ));
    shelf.align_items_to_bottom = true;
    assert_eq!(shelf.filled_slot_count(), 2);
    assert_eq!(shelf.comparator_output(), 2);

    let saved = shelf.save_additional();
    assert_eq!(ShelfBlockEntity::load_additional(&saved), shelf);
    assert_eq!(shelf.get_update_tag(), saved);
    assert_eq!(
        shelf.swap_item_no_update(
            0,
            Some(PotItemStack {
                item_id: "minecraft:stick".to_string(),
                count: 4,
            })
        ),
        Some(PotItemStack {
            item_id: "minecraft:book".to_string(),
            count: 1,
        })
    );
    assert_eq!(
        shelf.get_item(0),
        Some(&PotItemStack {
            item_id: "minecraft:stick".to_string(),
            count: 4,
        })
    );
    assert_eq!(
        shelf.remove_item_no_update(2),
        Some(PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 3,
        })
    );
    assert_eq!(shelf.comparator_output(), 1);

    let out_of_range = Tag::Compound(vec![(
        "Items".to_string(),
        Tag::List(vec![Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:apple".to_string())),
            ("count".to_string(), Tag::Int(1)),
            ("Slot".to_string(), Tag::Byte(9)),
        ])]),
    )]);
    assert_eq!(
        ShelfBlockEntity::load_additional(&out_of_range),
        ShelfBlockEntity::new()
    );
}

#[test]
fn beacon_tier_effect_payment_and_beam_state_match_vanilla_rules() {
    assert_beacon_base_tiers_match_java();
    let mut beacon = assert_beacon_effect_applications_match_java();
    assert_beacon_payment_and_persistence_match_java(&mut beacon);
    assert_beacon_beam_sections_match_java();
}

fn assert_beacon_base_tiers_match_java() {
    let pos = BlockPos { x: 0, y: 64, z: 0 };
    let full_four_tier = |block: BlockPos| {
        let dy = pos.y - block.y;
        (1..=4).contains(&dy) && block.x.abs() <= dy && block.z.abs() <= dy
    };
    assert_eq!(BeaconBlockEntity::update_base(pos, 0, full_four_tier), 4);
    let broken_second_tier = |block: BlockPos| {
        let dy = pos.y - block.y;
        (1..=4).contains(&dy)
            && block.x.abs() <= dy
            && block.z.abs() <= dy
            && !(dy == 2 && block.x == 2 && block.z == 0)
    };
    assert_eq!(
        BeaconBlockEntity::update_base(pos, 0, broken_second_tier),
        1
    );
    assert_eq!(
        BeaconBlockEntity::update_base(BlockPos { x: 0, y: 1, z: 0 }, 1, |_| true),
        0
    );
}

fn assert_beacon_effect_applications_match_java() -> BeaconBlockEntity {
    let mut beacon = BeaconBlockEntity::new();
    beacon.levels = 4;
    beacon.set_primary_power(Some("minecraft:speed"));
    beacon.set_secondary_power(Some("minecraft:speed"));
    assert_eq!(beacon.comparator_output(), 4);
    assert_eq!(
        beacon.effect_applications(),
        vec![BeaconEffectApplication {
            effect: "minecraft:speed".to_string(),
            duration_ticks: 340,
            amplifier: 1,
            range: 50,
        }]
    );
    beacon.set_secondary_power(Some("minecraft:regeneration"));
    assert_eq!(
        beacon.effect_applications(),
        vec![
            BeaconEffectApplication {
                effect: "minecraft:speed".to_string(),
                duration_ticks: 340,
                amplifier: 0,
                range: 50,
            },
            BeaconEffectApplication {
                effect: "minecraft:regeneration".to_string(),
                duration_ticks: 340,
                amplifier: 0,
                range: 50,
            },
        ]
    );

    beacon.set_primary_power(Some("minecraft:night_vision"));
    assert_eq!(beacon.primary_power, None);
    beacon
}

fn assert_beacon_payment_and_persistence_match_java(beacon: &mut BeaconBlockEntity) {
    assert!(BeaconBlockEntity::can_pay_with(&PotItemStack {
        item_id: "minecraft:amethyst_shard".to_string(),
        count: 1,
    }));
    assert!(!beacon.set_payment_item(Some(PotItemStack {
        item_id: "minecraft:apple".to_string(),
        count: 1,
    })));
    assert!(beacon.set_payment_item(Some(PotItemStack {
        item_id: "minecraft:emerald".to_string(),
        count: 1,
    })));

    beacon.primary_power = Some("minecraft:haste".to_string());
    beacon.secondary_power = Some("minecraft:regeneration".to_string());
    beacon.custom_name = Some("\"Beacon\"".to_string());
    beacon.lock_key = Some("secret".to_string());
    let saved = beacon.save_additional();
    let loaded = BeaconBlockEntity::load_additional(&saved);
    assert_eq!(loaded.levels, 4);
    assert_eq!(loaded.primary_power.as_deref(), Some("minecraft:haste"));
    assert_eq!(
        loaded.secondary_power.as_deref(),
        Some("minecraft:regeneration")
    );
    assert_eq!(loaded.display_name(), "\"Beacon\"");
    assert_eq!(loaded.lock_key.as_deref(), Some("secret"));
    assert_eq!(loaded.get_update_tag(), loaded.save_additional());
}

fn assert_beacon_beam_sections_match_java() {
    let mut section = BeaconBeamSection::new(0xFF00_FF00u32 as i32);
    assert_eq!(section.color(), 0xFF00_FF00u32 as i32);
    assert_eq!(section.height(), 1);
    section.increase_height();
    assert_eq!(section.height(), 2);

    let sections = BeaconBlockEntity::scan_beam([
        BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
        BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
        BeaconBeamBlock::TintedGlass(0xFF00_00FFu32 as i32),
        BeaconBeamBlock::Transparent,
    ]);
    assert_eq!(
        sections,
        vec![
            BeaconBeamSection {
                color: 0xFFFF_0000u32 as i32,
                height: 1,
            },
            BeaconBeamSection {
                color: 0xFFFF_0000u32 as i32,
                height: 1,
            },
            BeaconBeamSection {
                color: 0xFF7F_007Fu32 as i32,
                height: 2,
            },
        ]
    );
    let mut beacon = BeaconBlockEntity::new();
    beacon.beam_sections = sections.clone();
    assert_eq!(beacon.beam_sections(), sections.as_slice());
    assert_eq!(
        BeaconBlockEntity::scan_beam([BeaconBeamBlock::Blocking]),
        Vec::<BeaconBeamSection>::new()
    );
}

#[test]
fn lectern_block_entity_tracks_book_pages_and_comparator_signal() {
    let mut lectern = LecternBlockEntity::new();
    assert_eq!(lectern.save_additional(), Tag::Compound(Vec::new()));
    assert!(!lectern.has_book());
    assert_eq!(lectern.get_redstone_signal(), 0);

    let book = PotItemStack {
        item_id: "minecraft:written_book".to_string(),
        count: 1,
    };
    lectern.set_book(Some(book.clone()), 5);
    assert!(lectern.has_book());
    assert_eq!(lectern.page, 0);
    assert_eq!(lectern.page_count, 5);
    assert_eq!(lectern.get_redstone_signal(), 1);
    assert!(lectern.set_page(2));
    assert_eq!(lectern.get_redstone_signal(), 8);
    assert!(lectern.set_page(99));
    assert_eq!(lectern.page, 4);
    assert_eq!(lectern.get_redstone_signal(), 15);
    assert!(!lectern.set_page(4));

    let saved = lectern.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            ("Book".to_string(), book.to_tag()),
            ("Page".to_string(), Tag::Int(4)),
        ])
    );
    assert_eq!(LecternBlockEntity::load_additional(&saved, 5), lectern);
    assert_eq!(
        LecternBlockEntity::load_additional(
            &Tag::Compound(vec![
                ("Book".to_string(), book.to_tag()),
                ("Page".to_string(), Tag::Int(-3)),
            ]),
            5,
        )
        .page,
        0
    );

    assert_eq!(lectern.remove_book_no_update(), Some(book));
    assert!(!lectern.has_book());
    assert_eq!(lectern.page, 0);
    assert_eq!(lectern.page_count, 0);
    assert_eq!(lectern.get_redstone_signal(), 0);

    let mut single_page = LecternBlockEntity::new();
    single_page.set_book(
        Some(PotItemStack {
            item_id: "minecraft:writable_book".to_string(),
            count: 1,
        }),
        1,
    );
    assert_eq!(single_page.get_redstone_signal(), 15);
    single_page.clear_content();
    assert_eq!(single_page.save_additional(), Tag::Compound(Vec::new()));
}

#[test]
fn sign_block_entities_track_front_back_text_filtering_wax_and_hanging_shape() {
    let mut sign = SignBlockEntityModel::default();
    assert_sign_defaults_and_blank_edit_rejection(&mut sign);
    assert_sign_front_text_edit_click_and_wax(&mut sign);
    assert_sign_back_text_filtering_and_editor_timeout(&mut sign);
    let loaded = assert_sign_save_load_preserves_text(&sign);
    assert_hanging_sign_shape_and_load(loaded);
}

fn assert_sign_defaults_and_blank_edit_rejection(sign: &mut SignBlockEntityModel) {
    assert_eq!(SignBlockEntityModel::MAX_TEXT_LINE_WIDTH, 90);
    assert_eq!(SignBlockEntityModel::TEXT_LINE_HEIGHT, 10);
    assert_eq!(
        SignBlockEntityModel::INTERACTION_FAILED_SOUND,
        "minecraft:block.waxed_sign_interact_fail"
    );
    assert_eq!(sign.front_text.color, DyeColor::Black);
    assert!(!sign.front_text.has_message(false));
    assert!(!sign.update_sign_text(
        "player-a",
        true,
        std::array::from_fn(|_| SignLine::default()),
        false,
    ));
}

fn assert_sign_front_text_edit_click_and_wax(sign: &mut SignBlockEntityModel) {
    sign.set_allowed_player_editor(Some("player-a".to_string()));
    assert!(!sign.player_is_too_far_away_to_edit("player-a", 4.0));
    let front_lines = [
        SignLine::new("raw one", "filtered one").with_click_command("/say front"),
        SignLine::new("raw two", "filtered two"),
        SignLine::new("", ""),
        SignLine::new("raw four", "filtered four"),
    ];
    assert!(sign.update_sign_text("player-a", true, front_lines, false));
    assert!(sign.player_who_may_edit.is_none());
    sign.front_text.color = DyeColor::Blue;
    sign.front_text.has_glowing_text = true;
    assert_eq!(sign.front_text.lines[0].visible_text(false), "raw one");
    assert_eq!(sign.front_text.lines[0].visible_text(true), "filtered one");
    assert!(!sign.can_execute_click_commands(true, false));
    assert!(sign.set_waxed(true));
    assert!(sign.can_execute_click_commands(true, false));
    assert_eq!(
        sign.executable_click_commands(true, false),
        vec!["/say front".to_string()]
    );
}

fn assert_sign_back_text_filtering_and_editor_timeout(sign: &mut SignBlockEntityModel) {
    sign.set_allowed_player_editor(Some("player-b".to_string()));
    let filtered_back_lines = [
        SignLine::new("unsafe", "safe"),
        SignLine::new("raw", "clean"),
        SignLine::new("", ""),
        SignLine::new("last", "filtered last"),
    ];
    assert!(!sign.update_sign_text("player-b", false, filtered_back_lines.clone(), true));
    assert_eq!(sign.back_text.lines[0].raw, "");
    assert!(sign.set_waxed(false));
    assert!(sign.update_sign_text("player-b", false, filtered_back_lines, true));
    assert_eq!(sign.back_text.lines[0].raw, "safe");
    assert_eq!(sign.back_text.lines[0].filtered, "safe");

    sign.set_allowed_player_editor(Some("player-c".to_string()));
    assert!(sign.tick_editing_player("player-c", 4.01));
    assert!(sign.player_who_may_edit.is_none());
}

fn assert_sign_save_load_preserves_text(sign: &SignBlockEntityModel) -> SignBlockEntityModel {
    let saved = sign.save_additional();
    let loaded = SignBlockEntityModel::load_additional(&saved);
    assert_eq!(loaded.front_text.color, DyeColor::Blue);
    assert!(loaded.front_text.has_glowing_text);
    assert_eq!(loaded.front_text.lines[0].raw, "raw one");
    assert_eq!(loaded.front_text.lines[0].filtered, "filtered one");
    assert_eq!(
        loaded.front_text.lines[0].click_command.as_deref(),
        Some("/say front")
    );
    assert_eq!(loaded.back_text.lines[3].raw, "filtered last");
    assert!(!loaded.is_waxed);
    loaded
}

fn assert_hanging_sign_shape_and_load(loaded: SignBlockEntityModel) {
    let mut hanging = HangingSignBlockEntityModel::new(HangingSignAttachment::CeilingMiddle);
    hanging.sign = loaded;
    assert_eq!(HangingSignBlockEntityModel::MAX_TEXT_LINE_WIDTH, 60);
    assert_eq!(HangingSignBlockEntityModel::TEXT_LINE_HEIGHT, 9);
    assert_eq!(
        HangingSignBlockEntityModel::INTERACTION_FAILED_SOUND,
        "minecraft:block.waxed_hanging_sign_interact_fail"
    );
    assert_eq!(
        HangingSignAttachment::from_block_state("minecraft:oak_wall_hanging_sign[facing=north]"),
        HangingSignAttachment::Wall
    );
    assert_eq!(
        HangingSignAttachment::from_block_state(
            "minecraft:oak_hanging_sign[attached=true,rotation=8]"
        ),
        HangingSignAttachment::CeilingMiddle
    );
    assert_eq!(
        HangingSignAttachment::from_block_state("minecraft:oak_hanging_sign[attached=false]"),
        HangingSignAttachment::Ceiling
    );
    let saved = hanging.save_additional();
    assert!(
        !matches!(&saved, Tag::Compound(entries) if entries.iter().any(|(name, _)| name == "attachment"))
    );
    let loaded_hanging = HangingSignBlockEntityModel::load_additional(&saved);
    assert_eq!(loaded_hanging.attachment, HangingSignAttachment::Ceiling);
    assert_eq!(loaded_hanging.sign.front_text.lines[0].raw, "raw one");
}


#[test]
fn brewing_stand_ticks_fuel_recipes_sided_slots_and_save_load_like_java() {
    let recipes = brewing_test_recipes();
    let mut stand = BrewingStandBlockEntity::new();
    assert_brewing_stand_layout_and_sided_slots(&stand);
    assert_brewing_stand_slot_restrictions(&stand, &recipes);
    assert_brewing_stand_fuel_tick_brew_and_save_load(&mut stand, &recipes);
    assert_brewing_stand_cancelled_when_ingredient_changes(&recipes);
}

fn brewing_test_recipes() -> [BrewingRecipe; 3] {
    [
        BrewingRecipe::new(
            "minecraft:potion",
            "water",
            "minecraft:nether_wart",
            "minecraft:potion",
            "awkward",
        ),
        BrewingRecipe::new(
            "minecraft:potion",
            "awkward",
            "minecraft:blaze_powder",
            "minecraft:potion",
            "strength",
        ),
        BrewingRecipe::new(
            "minecraft:potion",
            "awkward",
            "minecraft:gunpowder",
            "minecraft:splash_potion",
            "awkward",
        ),
    ]
}

fn assert_brewing_stand_layout_and_sided_slots(stand: &BrewingStandBlockEntity) {
    assert_eq!(stand.items.len(), BrewingStandBlockEntity::CONTAINER_SIZE);
    assert_eq!(stand.potion_bits(), [false, false, false]);
    assert_eq!(
        BrewingStandBlockEntity::slots_for_face(Direction::Up),
        &[BrewingStandBlockEntity::INGREDIENT_SLOT]
    );
    assert_eq!(
        BrewingStandBlockEntity::slots_for_face(Direction::Down),
        &[0, 1, 2, BrewingStandBlockEntity::INGREDIENT_SLOT]
    );
    assert_eq!(
        BrewingStandBlockEntity::slots_for_face(Direction::North),
        &[0, 1, 2, BrewingStandBlockEntity::FUEL_SLOT]
    );
}

fn assert_brewing_stand_slot_restrictions(
    stand: &BrewingStandBlockEntity,
    recipes: &[BrewingRecipe],
) {
    let water = PotItemStack {
        item_id: brewing_stack_id("minecraft:potion", "water"),
        count: 1,
    };
    let nether_wart = PotItemStack {
        item_id: "minecraft:nether_wart".to_string(),
        count: 1,
    };
    let blaze_powder = PotItemStack {
        item_id: "minecraft:blaze_powder".to_string(),
        count: 2,
    };
    assert!(stand.can_place_item(0, &water, recipes));
    assert!(stand.can_place_item(
        BrewingStandBlockEntity::INGREDIENT_SLOT,
        &nether_wart,
        recipes
    ));
    assert!(stand.can_place_item(BrewingStandBlockEntity::FUEL_SLOT, &blaze_powder, recipes));
    assert!(!BrewingStandBlockEntity::can_take_item_through_face(
        BrewingStandBlockEntity::INGREDIENT_SLOT,
        &nether_wart,
        Direction::Down
    ));
    assert!(BrewingStandBlockEntity::can_take_item_through_face(
        BrewingStandBlockEntity::INGREDIENT_SLOT,
        &PotItemStack {
            item_id: "minecraft:glass_bottle".to_string(),
            count: 1,
        },
        Direction::Down
    ));
}

fn assert_brewing_stand_fuel_tick_brew_and_save_load(
    stand: &mut BrewingStandBlockEntity,
    recipes: &[BrewingRecipe],
) {
    let water = PotItemStack {
        item_id: brewing_stack_id("minecraft:potion", "water"),
        count: 1,
    };
    let nether_wart = stack("minecraft:nether_wart", 1);
    let blaze_powder = stack("minecraft:blaze_powder", 2);
    stand.set_item(0, Some(water.clone()));
    stand.set_item(1, Some(water));
    stand.set_item(BrewingStandBlockEntity::INGREDIENT_SLOT, Some(nether_wart));
    stand.set_item(BrewingStandBlockEntity::FUEL_SLOT, Some(blaze_powder));
    assert_eq!(stand.potion_bits(), [true, true, false]);
    assert_eq!(
        stand.server_tick(recipes),
        BrewingStandTickResult::FuelLoaded
    );
    assert_eq!(stand.fuel, BrewingStandBlockEntity::FUEL_USES);
    assert_eq!(
        stand.items[BrewingStandBlockEntity::FUEL_SLOT]
            .as_ref()
            .map(|stack| stack.count),
        Some(1)
    );
    assert_eq!(stand.server_tick(recipes), BrewingStandTickResult::Started);
    assert_eq!(stand.fuel, BrewingStandBlockEntity::FUEL_USES - 1);
    assert_eq!(stand.brew_time, BrewingStandBlockEntity::BREW_TIME);
    assert_eq!(stand.ingredient.as_deref(), Some("minecraft:nether_wart"));
    for _ in 1..BrewingStandBlockEntity::BREW_TIME {
        assert_eq!(stand.server_tick(recipes), BrewingStandTickResult::Brewing);
    }
    assert_eq!(stand.server_tick(recipes), BrewingStandTickResult::Brewed);
    assert_eq!(stand.items[BrewingStandBlockEntity::INGREDIENT_SLOT], None);
    assert_eq!(
        stand.items[0].as_ref().map(|stack| stack.item_id.as_str()),
        Some("minecraft:potion#awkward")
    );
    assert_eq!(
        stand.items[1].as_ref().map(|stack| stack.item_id.as_str()),
        Some("minecraft:potion#awkward")
    );

    let saved = stand.save_additional();
    assert_eq!(BrewingStandBlockEntity::load_additional(&saved), *stand);
}

fn assert_brewing_stand_cancelled_when_ingredient_changes(recipes: &[BrewingRecipe]) {
    let mut cancelled = BrewingStandBlockEntity::new();
    cancelled.set_item(
        0,
        Some(PotItemStack {
            item_id: brewing_stack_id("minecraft:potion", "awkward"),
            count: 1,
        }),
    );
    cancelled.set_item(
        BrewingStandBlockEntity::INGREDIENT_SLOT,
        Some(PotItemStack {
            item_id: "minecraft:blaze_powder".to_string(),
            count: 1,
        }),
    );
    cancelled.fuel = 1;
    assert_eq!(
        cancelled.server_tick(recipes),
        BrewingStandTickResult::Started
    );
    cancelled.set_item(
        BrewingStandBlockEntity::INGREDIENT_SLOT,
        Some(PotItemStack {
            item_id: "minecraft:gunpowder".to_string(),
            count: 1,
        }),
    );
    assert_eq!(
        cancelled.server_tick(recipes),
        BrewingStandTickResult::Cancelled
    );
    assert_eq!(cancelled.brew_time, 0);
}

#[test]
fn crafter_block_entity_tracks_disabled_slots_triggered_pulse_and_output_like_java() {
    let mut crafter = CrafterBlockEntity::new();
    assert_crafter_layout_disabled_slots_and_input_rules(&mut crafter);
    let recipe = crafter_test_recipe();
    assert_crafter_pulse_result_and_tick_lifecycle(&mut crafter, &recipe);
    assert_crafter_save_load_round_trip(&crafter);
    assert_crafter_no_recipe_pulse_result(recipe);
}

fn assert_crafter_layout_disabled_slots_and_input_rules(crafter: &mut CrafterBlockEntity) {
    assert_eq!(CrafterBlockEntity::CONTAINER_WIDTH, 3);
    assert_eq!(CrafterBlockEntity::CONTAINER_HEIGHT, 3);
    assert_eq!(CrafterBlockEntity::CONTAINER_SIZE, 9);
    assert_eq!(CrafterBlockEntity::DATA_TRIGGERED, 9);
    assert_eq!(CrafterBlockEntity::NUM_DATA, 10);
    assert_eq!(CrafterBlockEntity::SLOT_DISABLED, 1);
    assert_eq!(CrafterBlockEntity::SLOT_ENABLED, 0);
    assert_eq!(CrafterBlockEntity::DISPLAY_NAME, "container.crafter");
    assert!(!crafter.triggered);
    assert_eq!(crafter.crafting_ticks_remaining, 0);
    assert_eq!(crafter.redstone_signal(), 0);
    assert!(crafter.set_slot_state(8, false));
    assert!(crafter.is_slot_disabled(8));
    assert_eq!(crafter.redstone_signal(), 1);
    assert!(crafter.set_item(
        8,
        Some(PotItemStack {
            item_id: "minecraft:stone".to_string(),
            count: 1,
        }),
    ));
    assert!(!crafter.is_slot_disabled(8));
    assert!(crafter.set_item(8, None));
    assert!(crafter.set_item(
        0,
        Some(PotItemStack {
            item_id: "minecraft:oak_planks".to_string(),
            count: 2,
        }),
    ));
    assert!(crafter.set_item(
        1,
        Some(PotItemStack {
            item_id: "minecraft:oak_planks".to_string(),
            count: 1,
        }),
    ));
    assert!(!crafter.can_place_item(
        0,
        &PotItemStack {
            item_id: "minecraft:oak_planks".to_string(),
            count: 1,
        },
    ));
    assert!(crafter.can_place_item(
        2,
        &PotItemStack {
            item_id: "minecraft:oak_planks".to_string(),
            count: 1,
        },
    ));
    assert_eq!(crafter.redstone_signal(), 2);
}

fn crafter_test_recipe() -> CrafterRecipe {
    CrafterRecipe {
        pattern: [
            Some("minecraft:oak_planks"),
            Some("minecraft:oak_planks"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        result: PotItemStack {
            item_id: "minecraft:stick".to_string(),
            count: 4,
        },
        remaining_items: vec![PotItemStack {
            item_id: "minecraft:bowl".to_string(),
            count: 1,
        }],
    }
}

fn assert_crafter_pulse_result_and_tick_lifecycle(
    crafter: &mut CrafterBlockEntity,
    recipe: &CrafterRecipe,
) {
    assert_eq!(
        crafter.pulse_craft(std::slice::from_ref(recipe)),
        CrafterPulseResult::NotTriggered
    );
    crafter.set_triggered(true);
    assert_eq!(
        crafter.pulse_craft(std::slice::from_ref(recipe)),
        CrafterPulseResult::Crafted {
            result: PotItemStack {
                item_id: "minecraft:stick".to_string(),
                count: 4,
            },
            remaining_items: vec![PotItemStack {
                item_id: "minecraft:bowl".to_string(),
                count: 1,
            }],
        }
    );
    assert_eq!(
        crafter.crafting_ticks_remaining,
        CrafterBlockEntity::MAX_CRAFTING_TICKS
    );
    assert_eq!(
        crafter.items[0],
        Some(PotItemStack {
            item_id: "minecraft:oak_planks".to_string(),
            count: 1,
        })
    );
    assert_eq!(crafter.items[1], None);
    for _ in 1..CrafterBlockEntity::MAX_CRAFTING_TICKS {
        assert_eq!(crafter.server_tick(), None);
    }
    assert_eq!(crafter.server_tick(), Some(("crafting", false)));
    assert_eq!(crafter.crafting_ticks_remaining, 0);
}

fn assert_crafter_save_load_round_trip(crafter: &CrafterBlockEntity) {
    let saved = crafter.save_additional();
    let loaded = CrafterBlockEntity::load_additional(&saved);
    assert_eq!(loaded, *crafter);
}

fn assert_crafter_no_recipe_pulse_result(recipe: CrafterRecipe) {
    let mut no_recipe = CrafterBlockEntity::new();
    no_recipe.set_triggered(true);
    assert_eq!(
        no_recipe.pulse_craft(&[recipe]),
        CrafterPulseResult::NoRecipe
    );
}

#[test]
fn spawner_block_entity_tracks_spawn_data_rules_delay_and_nbt_like_java() {
    let mut spawner = SpawnerBlockEntity::default();
    assert_spawner_defaults_and_idle_tick(&mut spawner);
    assert_spawner_entity_id_and_delay_paths(&mut spawner);
    assert_spawner_spawn_rule_nearby_cap_and_success_paths(&mut spawner);
    assert_spawner_save_update_tag_and_events(&mut spawner);
}

fn assert_spawner_defaults_and_idle_tick(spawner: &mut SpawnerBlockEntity) {
    assert_eq!(spawner.spawn_delay, 20);
    assert_eq!(spawner.min_spawn_delay, 200);
    assert_eq!(spawner.max_spawn_delay, 800);
    assert_eq!(spawner.spawn_count, 4);
    assert_eq!(spawner.max_nearby_entities, 6);
    assert_eq!(spawner.required_player_range, 16);
    assert_eq!(spawner.spawn_range, 4);
    assert_eq!(
        spawner.server_tick(false, true, 0, 0),
        SpawnerTickResult::Idle
    );
}

fn assert_spawner_entity_id_and_delay_paths(spawner: &mut SpawnerBlockEntity) {
    spawner.set_entity_id("minecraft:zombie");
    assert_eq!(
        spawner
            .next_spawn_data
            .as_ref()
            .and_then(SpawnDataModel::entity_id),
        Some("minecraft:zombie")
    );

    spawner.spawn_potentials = spawner_test_potentials();
    spawner.spawn_delay = -1;
    assert_eq!(
        spawner.server_tick(true, true, 0, 12),
        SpawnerTickResult::Delay
    );
    assert_eq!(spawner.spawn_delay, 212);

    spawner.spawn_delay = 2;
    assert_eq!(
        spawner.server_tick(true, true, 0, 0),
        SpawnerTickResult::CountDown
    );
    assert_eq!(spawner.spawn_delay, 1);
}

fn spawner_test_potentials() -> Vec<SpawnDataModel> {
    vec![
        SpawnDataModel {
            weight: 1,
            ..SpawnDataModel::new("minecraft:zombie")
        },
        SpawnDataModel {
            weight: 3,
            custom_spawn_rules: Some(SpawnerCustomSpawnRules {
                block_light_limit: (0, 7),
                sky_light_limit: (0, 15),
                requires_no_sky_access: true,
            }),
            equipment: Some(Tag::Compound(vec![(
                "mainhand".to_string(),
                Tag::String("minecraft:iron_sword".to_string()),
            )])),
            ..SpawnDataModel::new("minecraft:skeleton")
        },
    ]
}

fn assert_spawner_spawn_rule_nearby_cap_and_success_paths(spawner: &mut SpawnerBlockEntity) {
    spawner.spawn_delay = 0;
    spawner.next_spawn_data = Some(spawner.spawn_potentials[1].clone());
    assert_eq!(
        spawner.server_tick_with_context(SpawnerSpawnContext {
            player_in_range: true,
            spawner_blocks_work: true,
            nearby_entities: 0,
            block_light: 8,
            sky_light: 0,
            no_sky_access: true,
            collision_free: true,
            spawn_rules_ok: true,
            obstruction_free: true,
            delay_roll: 5,
            potential_roll: 0,
        }),
        SpawnerTickResult::SpawnRulesFailed
    );
    assert_eq!(spawner.spawn_delay, 0);

    assert_eq!(
        spawner.server_tick_with_context(SpawnerSpawnContext {
            player_in_range: true,
            spawner_blocks_work: true,
            nearby_entities: spawner.max_nearby_entities,
            block_light: 0,
            sky_light: 0,
            no_sky_access: true,
            collision_free: true,
            spawn_rules_ok: true,
            obstruction_free: true,
            delay_roll: 7,
            potential_roll: 0,
        }),
        SpawnerTickResult::Delay
    );
    assert_eq!(spawner.spawn_delay, 207);

    spawner.spawn_delay = 0;
    spawner.next_spawn_data = Some(spawner.spawn_potentials[1].clone());
    assert_eq!(
        spawner.server_tick_with_context(SpawnerSpawnContext {
            player_in_range: true,
            spawner_blocks_work: true,
            nearby_entities: 0,
            block_light: 7,
            sky_light: 0,
            no_sky_access: true,
            collision_free: true,
            spawn_rules_ok: true,
            obstruction_free: true,
            delay_roll: 11,
            potential_roll: 1,
        }),
        SpawnerTickResult::Spawned {
            entity_id: "minecraft:skeleton".to_string(),
            count: 4,
        }
    );
    assert_eq!(spawner.spawn_delay, 211);
    assert_eq!(
        spawner
            .next_spawn_data
            .as_ref()
            .and_then(SpawnDataModel::entity_id),
        Some("minecraft:skeleton")
    );
}

fn assert_spawner_save_update_tag_and_events(spawner: &mut SpawnerBlockEntity) {
    let saved = spawner.save_additional();
    let update_tag = spawner.update_tag();
    assert!(compound_entries(&saved)
        .unwrap()
        .iter()
        .any(|(name, _)| name == "SpawnPotentials"));
    assert!(!compound_entries(&update_tag)
        .unwrap()
        .iter()
        .any(|(name, _)| name == "SpawnPotentials"));
    assert_eq!(SpawnerBlockEntity::load_additional(&saved), *spawner);

    assert!(spawner.on_event_triggered(true, SpawnerBlockEntity::EVENT_SPAWN));
    assert_eq!(spawner.spawn_delay, spawner.min_spawn_delay);
    assert!(!spawner.on_event_triggered(true, 99));
}

#[test]
fn crafter_handle_slot_state_changed_gates_match_java() {
    // Java `ServerGamePacketListenerImpl.handleContainerSlotStateChanged` (crafter
    // branch): apply the toggle only when the sender is not a spectator and the
    // packet's container id matches the player's open menu container id.
    let mut crafter = CrafterBlockEntity::new();
    let open = 5;

    // Happy path: not spectator, matching container id, empty slot -> disabled.
    assert!(crafter.handle_slot_state_changed(false, open, open, 3, false));
    assert!(crafter.is_slot_disabled(3));

    // Re-enable through the handler.
    assert!(crafter.handle_slot_state_changed(false, open, open, 3, true));
    assert!(!crafter.is_slot_disabled(3));

    // Spectator is rejected.
    assert!(!crafter.handle_slot_state_changed(true, open, open, 4, false));
    assert!(!crafter.is_slot_disabled(4));

    // Container-id mismatch is rejected.
    assert!(!crafter.handle_slot_state_changed(false, open, open + 1, 4, false));
    assert!(!crafter.is_slot_disabled(4));

    // Out-of-range / negative slot is rejected (matches slotCanBeDisabled bounds).
    assert!(!crafter.handle_slot_state_changed(false, open, open, -1, false));
    assert!(!crafter.handle_slot_state_changed(false, open, open, 9, false));
}
