use super::super::*;
use super::*;

#[test]
fn banner_pattern_layers_preserve_order_and_enforce_six_layer_cap() {
    let mut banner = BannerBlockEntity::from_block_state("minecraft:red_banner").unwrap();
    let layers = [
        ("minecraft:stripe_bottom", DyeColor::White),
        ("minecraft:stripe_top", DyeColor::Black),
        ("minecraft:stripe_left", DyeColor::Blue),
        ("minecraft:stripe_right", DyeColor::Yellow),
        ("minecraft:diagonal_left", DyeColor::Green),
        ("minecraft:diagonal_right", DyeColor::Purple),
    ];
    for (pattern, color) in layers {
        assert!(banner.add_pattern(pattern, color));
    }
    assert!(!banner.add_pattern("minecraft:globe", DyeColor::Cyan));
    banner.custom_name = Some("{\"text\":\"Six Layers\"}".to_string());

    let saved = banner.save_additional();
    let Tag::Compound(fields) = &saved else {
        panic!("banner save_additional should produce a compound");
    };
    assert_eq!(fields[0].0, "patterns");
    assert_eq!(fields[1].0, "CustomName");

    let Tag::List(saved_layers) = &fields[0].1 else {
        panic!("patterns should be a list");
    };
    assert_eq!(saved_layers.len(), BannerBlockEntity::MAX_PATTERNS);
    for (idx, (expected_pattern, expected_color)) in layers.iter().enumerate() {
        assert_eq!(
            saved_layers[idx],
            Tag::Compound(vec![
                (
                    "pattern".to_string(),
                    Tag::String((*expected_pattern).to_string())
                ),
                (
                    "color".to_string(),
                    Tag::String(expected_color.vanilla_name().to_string())
                ),
            ])
        );
    }

    let loaded = BannerBlockEntity::load_additional("minecraft:red_wall_banner", &saved)
        .expect("saved red banner should load");
    assert_eq!(loaded.base_color, DyeColor::Red);
    assert_eq!(loaded.patterns, banner.patterns);
    assert_eq!(loaded.custom_name, banner.custom_name);

    let overlong = Tag::Compound(vec![(
        "patterns".to_string(),
        Tag::List(
            (0..8)
                .map(|idx| {
                    BannerPatternLayer {
                        pattern: format!("minecraft:test_{idx}"),
                        color: DyeColor::White,
                    }
                    .to_tag()
                })
                .collect(),
        ),
    )]);
    let truncated =
        BannerBlockEntity::load_additional("minecraft:white_banner", &overlong).unwrap();
    assert_eq!(truncated.patterns.len(), BannerBlockEntity::MAX_PATTERNS);
    assert_eq!(truncated.patterns[5].pattern, "minecraft:test_5");
}

#[test]
fn decorated_pot_saves_sherds_item_loot_and_wobble_like_java() {
    let mut pot = DecoratedPotBlockEntity {
        decorations: PotDecorations::new(
            Some("minecraft:angler_pottery_sherd".to_string()),
            None,
            Some("minecraft:arms_up_pottery_sherd".to_string()),
            Some("minecraft:brick".to_string()),
        ),
        item: Some(PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 2,
        }),
        ..DecoratedPotBlockEntity::default()
    };

    assert_eq!(
        pot.decorations.ordered(),
        vec![
            "minecraft:angler_pottery_sherd".to_string(),
            "minecraft:brick".to_string(),
            "minecraft:arms_up_pottery_sherd".to_string(),
            "minecraft:brick".to_string(),
        ]
    );
    let saved = pot.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            (
                "sherds".to_string(),
                Tag::List(vec![
                    Tag::String("minecraft:angler_pottery_sherd".to_string()),
                    Tag::String("minecraft:brick".to_string()),
                    Tag::String("minecraft:arms_up_pottery_sherd".to_string()),
                    Tag::String("minecraft:brick".to_string()),
                ]),
            ),
            (
                "item".to_string(),
                Tag::Compound(vec![
                    (
                        "id".to_string(),
                        Tag::String("minecraft:diamond".to_string())
                    ),
                    ("count".to_string(), Tag::Int(2)),
                ]),
            ),
        ])
    );
    assert_eq!(DecoratedPotBlockEntity::load_additional(&saved), pot);

    pot.loot_table = Some("minecraft:chests/trial_chambers/reward".to_string());
    pot.loot_table_seed = 123;
    let loot_saved = pot.save_additional();
    assert!(
        matches!(&loot_saved, Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "LootTable") && fields.iter().all(|(key, _)| key != "item"))
    );
    let loaded_loot = DecoratedPotBlockEntity::load_additional(&loot_saved);
    assert_eq!(loaded_loot.loot_table, pot.loot_table);
    assert_eq!(loaded_loot.loot_table_seed, 123);
    assert_eq!(loaded_loot.item, None);

    assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);
    assert_eq!(DecoratedPotWobbleStyle::Negative.duration(), 10);
    assert!(pot.trigger_event(
        DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
        DecoratedPotWobbleStyle::Negative.id(),
        42,
    ));
    assert_eq!(pot.wobble_started_at_tick, 42);
    assert_eq!(
        pot.last_wobble_style,
        Some(DecoratedPotWobbleStyle::Negative)
    );
    assert!(!pot.trigger_event(99, DecoratedPotWobbleStyle::Positive.id(), 43));
    assert!(!pot.trigger_event(DecoratedPotBlockEntity::EVENT_POT_WOBBLES, 99, 43));
}

#[test]
fn decorated_pot_wobble_and_destruction_drops_preserve_sherds_and_item() {
    let mut pot = DecoratedPotBlockEntity {
        decorations: PotDecorations::new(
            Some("minecraft:arms_up_pottery_sherd".to_string()),
            Some("minecraft:blade_pottery_sherd".to_string()),
            None,
            Some("minecraft:brewer_pottery_sherd".to_string()),
        ),
        ..DecoratedPotBlockEntity::default()
    };

    pot.item = Some(PotItemStack {
        item_id: "minecraft:emerald".to_string(),
        count: 3,
    });
    assert!(pot.trigger_event(
        DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
        DecoratedPotWobbleStyle::Positive.id(),
        2400,
    ));
    assert_eq!(pot.wobble_started_at_tick, 2400);
    assert_eq!(
        pot.last_wobble_style,
        Some(DecoratedPotWobbleStyle::Positive)
    );
    assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);

    let drops = pot.destruction_drops();
    assert_eq!(
        drops.decoration_items,
        vec![
            "minecraft:arms_up_pottery_sherd".to_string(),
            "minecraft:blade_pottery_sherd".to_string(),
            "minecraft:brick".to_string(),
            "minecraft:brewer_pottery_sherd".to_string(),
        ]
    );
    assert_eq!(
        drops.stored_item,
        Some(PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 3,
        })
    );

    pot.item = Some(PotItemStack {
        item_id: "minecraft:air".to_string(),
        count: 0,
    });
    assert_eq!(pot.destruction_drops().stored_item, None);
}

#[test]
fn brushable_block_entity_brushes_resets_loot_and_update_tag_like_java() {
    assert_eq!(BrushableBlockEntity::BRUSH_COOLDOWN_TICKS, 10);
    assert_eq!(BrushableBlockEntity::BRUSH_RESET_TICKS, 40);
    assert_eq!(BrushableBlockEntity::REQUIRED_BRUSHES_TO_BREAK, 10);

    let mut brushable = BrushableBlockEntity::new();
    brushable.set_loot_table("minecraft:archaeology/desert_pyramid", 99);
    assert_eq!(
        brushable.save_additional(),
        Tag::Compound(vec![
            (
                "LootTable".to_string(),
                Tag::String("minecraft:archaeology/desert_pyramid".to_string())
            ),
            ("LootTableSeed".to_string(), Tag::Long(99)),
        ])
    );

    let generated_item = PotItemStack {
        item_id: "minecraft:diamond".to_string(),
        count: 1,
    };
    assert_eq!(
        brushable.brush(100, Direction::North, Some(generated_item.clone())),
        BrushResult::InProgress { dusted: 1 }
    );
    assert_eq!(brushable.hit_direction, Some(Direction::North));
    assert_eq!(brushable.brush_count, 1);
    assert_eq!(brushable.brush_count_resets_at_tick, 140);
    assert_eq!(brushable.cooldown_ends_at_tick, 110);
    assert_eq!(brushable.item, Some(generated_item.clone()));
    assert_eq!(brushable.loot_table, None);
    assert_eq!(
        brushable.get_update_tag(),
        Tag::Compound(vec![
            (
                "hit_direction".to_string(),
                Tag::String("north".to_string())
            ),
            ("item".to_string(), generated_item.to_tag()),
        ])
    );
    assert_eq!(
        brushable.brush(105, Direction::South, None),
        BrushResult::CoolingDown
    );
    assert_eq!(brushable.hit_direction, Some(Direction::North));

    assert_eq!(
        brushable.brush(110, Direction::South, None),
        BrushResult::InProgress { dusted: 1 }
    );
    assert_eq!(
        brushable.brush(120, Direction::South, None),
        BrushResult::InProgress { dusted: 2 }
    );
    assert_eq!(
        brushable.brush(130, Direction::South, None),
        BrushResult::InProgress { dusted: 2 }
    );
    assert_eq!(
        brushable.brush(140, Direction::South, None),
        BrushResult::InProgress { dusted: 2 }
    );
    assert_eq!(
        brushable.brush(150, Direction::South, None),
        BrushResult::InProgress { dusted: 3 }
    );
    assert_eq!(brushable.brush_count, 6);

    assert_eq!(brushable.check_reset(189), None);
    assert_eq!(brushable.check_reset(190), Some(2));
    assert_eq!(brushable.brush_count, 4);
    assert_eq!(brushable.brush_count_resets_at_tick, 194);
    assert_eq!(brushable.check_reset(194), Some(1));
    assert_eq!(brushable.brush_count, 2);
    assert_eq!(brushable.check_reset(198), Some(0));
    assert_eq!(brushable.brush_count, 0);
    assert_eq!(brushable.hit_direction, None);
    assert_eq!(brushable.cooldown_ends_at_tick, 0);

    brushable.hit_direction = Some(Direction::East);
    brushable.item = Some(generated_item.clone());
    let saved_item = brushable.save_additional();
    assert_eq!(
        BrushableBlockEntity::load_additional(&saved_item),
        brushable
    );
    assert_eq!(
        brushable.drop_content(),
        Some((generated_item, Direction::East))
    );
    assert_eq!(brushable.item, None);

    let mut completing = BrushableBlockEntity::new();
    for step in 0..9 {
        assert!(matches!(
            completing.brush(step * 10, Direction::Up, None),
            BrushResult::InProgress { .. }
        ));
    }
    assert_eq!(
        completing.brush(90, Direction::Up, None),
        BrushResult::Completed
    );
    assert_eq!(completing.brush_count, 10);
}

#[test]
fn copper_golem_statue_tracks_weather_pose_comparator_and_clone_components() {
    let mut statue = CopperGolemStatueBlockEntity::from_block_state(
        "minecraft:waxed_weathered_copper_golem_statue",
        CopperGolemStatuePose::Standing,
    )
    .unwrap();
    assert_eq!(statue.weather_state, CopperWeatherState::Weathered);
    assert_eq!(statue.weather_state.serialized_name(), "weathered");
    assert!(statue.waxed);
    assert_eq!(statue.comparator_output(), 1);

    statue.update_pose();
    assert_eq!(statue.pose, CopperGolemStatuePose::Sitting);
    assert_eq!(statue.comparator_output(), 2);
    statue.update_pose();
    assert_eq!(statue.pose, CopperGolemStatuePose::Running);
    assert_eq!(statue.comparator_output(), 3);
    statue.update_pose();
    assert_eq!(statue.pose, CopperGolemStatuePose::Star);
    assert_eq!(statue.comparator_output(), 4);
    statue.update_pose();
    assert_eq!(statue.pose, CopperGolemStatuePose::Standing);

    statue.custom_name = Some("{\"text\":\"Copper Buddy\"}".to_string());
    assert_eq!(
        statue.save_additional(),
        Tag::Compound(vec![(
            "CustomName".to_string(),
            Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
        )])
    );
    assert_eq!(
        statue.clone_item_components(),
        Tag::Compound(vec![
            (
                "minecraft:block_state".to_string(),
                Tag::Compound(vec![(
                    "copper_golem_pose".to_string(),
                    Tag::String("standing".to_string())
                )])
            ),
            (
                "minecraft:custom_name".to_string(),
                Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
            ),
        ])
    );

    let oxidized = CopperGolemStatueBlockEntity::from_block_state(
        "minecraft:oxidized_copper_golem_statue",
        CopperGolemStatuePose::Star,
    )
    .unwrap();
    assert_eq!(oxidized.weather_state, CopperWeatherState::Oxidized);
    assert!(!oxidized.waxed);
    assert_eq!(oxidized.comparator_output(), 4);
    assert_eq!(
        CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:copper_block",
            CopperGolemStatuePose::Standing,
        ),
        None
    );
}

#[test]
fn skull_block_entity_saves_profile_components_and_animation_like_java() {
    let profile = Tag::Compound(vec![
        ("name".to_string(), Tag::String("Steve".to_string())),
        (
            "id".to_string(),
            Tag::String("8667ba71-b85a-4004-af54-457a9734eed7".to_string()),
        ),
    ]);
    let mut skull = SkullBlockEntity::new();
    skull.profile = Some(profile.clone());
    skull.note_block_sound = Some("minecraft:block.note_block.basedrum".to_string());
    skull.custom_name = Some("{\"text\":\"Head\"}".to_string());

    let saved = skull.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![
            ("profile".to_string(), profile.clone()),
            (
                "note_block_sound".to_string(),
                Tag::String("minecraft:block.note_block.basedrum".to_string())
            ),
            (
                "custom_name".to_string(),
                Tag::String("{\"text\":\"Head\"}".to_string())
            ),
        ])
    );
    assert_eq!(SkullBlockEntity::load_additional(&saved), skull);
    assert_eq!(skull.get_update_tag(), saved);

    skull.animation_tick(true);
    skull.animation_tick(true);
    assert!(skull.is_animating);
    assert_eq!(skull.animation_tick_count, 2);
    assert_eq!(skull.animation(0.5), 2.5);
    skull.animation_tick(false);
    assert!(!skull.is_animating);
    assert_eq!(skull.animation(0.5), 2.0);

    let mut tag_with_components = saved.clone();
    SkullBlockEntity::remove_components_from_tag(&mut tag_with_components);
    assert_eq!(tag_with_components, Tag::Compound(vec![]));

    let mut from_components = SkullBlockEntity::new();
    from_components.apply_implicit_components(&BTreeMap::from([
        ("minecraft:profile".to_string(), profile.clone()),
        (
            "minecraft:note_block_sound".to_string(),
            Tag::String("minecraft:block.note_block.harp".to_string()),
        ),
        (
            "minecraft:custom_name".to_string(),
            Tag::String("{\"text\":\"Component Head\"}".to_string()),
        ),
    ]));
    assert_eq!(from_components.profile, Some(profile.clone()));
    assert_eq!(
        from_components.note_block_sound,
        Some("minecraft:block.note_block.harp".to_string())
    );
    assert_eq!(
        from_components.custom_name,
        Some("{\"text\":\"Component Head\"}".to_string())
    );
    assert_eq!(
        from_components.collect_implicit_components(),
        BTreeMap::from([
            ("minecraft:profile".to_string(), profile),
            (
                "minecraft:note_block_sound".to_string(),
                Tag::String("minecraft:block.note_block.harp".to_string())
            ),
            (
                "minecraft:custom_name".to_string(),
                Tag::String("{\"text\":\"Component Head\"}".to_string())
            ),
        ])
    );
}

#[test]
fn conduit_block_entity_scans_frame_applies_effects_and_tracks_target() {
    assert_eq!(ConduitBlockEntity::BLOCK_REFRESH_RATE, 40);
    assert_eq!(ConduitBlockEntity::MIN_ACTIVE_SIZE, 16);
    assert_eq!(ConduitBlockEntity::MIN_KILL_SIZE, 42);
    assert_eq!(ConduitBlockEntity::EFFECT_DURATION_TICKS, 260);
    assert_eq!(ConduitBlockEntity::KILL_RANGE, 8.0);
    assert!(ConduitBlockEntity::is_valid_frame_block(
        "minecraft:sea_lantern"
    ));
    assert!(!ConduitBlockEntity::is_valid_frame_block("minecraft:stone"));

    let origin = BlockPos {
        x: 10,
        y: 64,
        z: 10,
    };
    let mut conduit = ConduitBlockEntity::new();
    assert!(!conduit.update_shape(origin, |_| false, |_| "minecraft:sea_lantern"));
    assert!(conduit.effect_blocks.is_empty());

    assert!(conduit.update_shape(origin, |_| true, |_| "minecraft:prismarine"));
    assert_eq!(conduit.effect_blocks.len(), 42);
    assert_eq!(
        ConduitBlockEntity::effect_range(conduit.effect_blocks.len()),
        96
    );
    conduit.is_active = true;
    assert_eq!(
        conduit.apply_effects(),
        Some(ConduitEffectApplication {
            range: 96,
            duration_ticks: 260,
        })
    );

    let active_frame: Vec<BlockPos> = conduit.effect_blocks.iter().copied().take(16).collect();
    let active_frame_lookup = |pos: BlockPos| {
        if active_frame.contains(&pos) {
            "minecraft:dark_prismarine"
        } else {
            "minecraft:air"
        }
    };
    let mut minimum = ConduitBlockEntity::new();
    assert!(minimum.update_shape(origin, |_| true, active_frame_lookup));
    assert_eq!(minimum.effect_blocks.len(), 16);
    assert_eq!(
        ConduitBlockEntity::effect_range(minimum.effect_blocks.len()),
        32
    );
    minimum.is_hunting = minimum.effect_blocks.len() >= ConduitBlockEntity::MIN_KILL_SIZE;
    assert!(!minimum.update_destroy_target(
        origin,
        &[ConduitTarget {
            id: "guardian".to_string(),
            pos: BlockPos {
                x: 12,
                y: 64,
                z: 10
            },
            alive: true,
            enemy: true,
            in_water_or_rain: true,
        }]
    ));
    assert_eq!(minimum.destroy_target, None);

    let targets = vec![
        ConduitTarget {
            id: "outside".to_string(),
            pos: BlockPos {
                x: 18,
                y: 64,
                z: 10,
            },
            alive: true,
            enemy: true,
            in_water_or_rain: true,
        },
        ConduitTarget {
            id: "guardian".to_string(),
            pos: BlockPos {
                x: 17,
                y: 64,
                z: 10,
            },
            alive: true,
            enemy: true,
            in_water_or_rain: true,
        },
    ];
    let mut hunting = ConduitBlockEntity::new();
    let attacked =
        hunting.server_tick(40, origin, |_| true, |_| "minecraft:sea_lantern", &targets);
    assert_eq!(attacked.as_deref(), Some("guardian"));
    assert!(hunting.is_active);
    assert!(hunting.is_hunting);
    assert_eq!(hunting.destroy_target.as_deref(), Some("guardian"));
    assert_eq!(hunting.active_rotation(0.0), -0.0375);

    assert_eq!(
        hunting.server_tick(80, origin, |_| true, |_| "minecraft:sea_lantern", &targets),
        None
    );
    let dead_target = [ConduitTarget {
        id: "guardian".to_string(),
        pos: BlockPos {
            x: 17,
            y: 64,
            z: 10,
        },
        alive: false,
        enemy: true,
        in_water_or_rain: true,
    }];
    assert_eq!(
        hunting.server_tick(
            120,
            origin,
            |_| true,
            |_| "minecraft:sea_lantern",
            &dead_target
        ),
        None
    );
    assert_eq!(hunting.destroy_target, None);

    hunting.destroy_target = Some("guardian".to_string());
    let saved = hunting.save_additional();
    assert_eq!(
        saved,
        Tag::Compound(vec![(
            "Target".to_string(),
            Tag::String("guardian".to_string())
        )])
    );
    assert_eq!(
        ConduitBlockEntity::load_additional(&saved).destroy_target,
        Some("guardian".to_string())
    );
    assert_eq!(hunting.get_update_tag(), saved);
}

#[test]
fn campfire_block_entity_cooks_cools_saves_and_updates_items_like_java() {
    assert_eq!(CampfireBlockEntity::NUM_SLOTS, 4);
    assert_eq!(CampfireBlockEntity::DEFAULT_COOKING_TIME, 600);
    assert_eq!(CampfireBlockEntity::BURN_COOL_SPEED, 2);

    let mut campfire = CampfireBlockEntity::new(true);
    assert!(campfire.place_food(
        PotItemStack {
            item_id: "minecraft:cod".to_string(),
            count: 3,
        },
        Some(3),
    ));
    assert_eq!(
        campfire.items[0],
        Some(PotItemStack {
            item_id: "minecraft:cod".to_string(),
            count: 1,
        })
    );
    assert_eq!(campfire.cooking_time[0], 3);
    assert!((1..CampfireBlockEntity::NUM_SLOTS).all(|slot| campfire.items[slot].is_none()));

    assert_eq!(
        campfire.cook_tick(true, |item| PotItemStack {
            item_id: format!(
                "minecraft:cooked_{}",
                item.item_id.trim_start_matches("minecraft:")
            ),
            count: 1,
        }),
        vec![CampfireTickResult::Changed]
    );
    assert_eq!(campfire.cooking_progress[0], 1);
    assert_eq!(campfire.cooldown_tick(), vec![CampfireTickResult::Changed]);
    assert_eq!(campfire.cooking_progress[0], 0);

    campfire.cooking_progress[0] = 2;
    assert_eq!(
        campfire.cook_tick(true, |item| PotItemStack {
            item_id: format!(
                "minecraft:cooked_{}",
                item.item_id.trim_start_matches("minecraft:")
            ),
            count: 1,
        }),
        vec![CampfireTickResult::Cooked {
            slot: 0,
            item: PotItemStack {
                item_id: "minecraft:cooked_cod".to_string(),
                count: 1,
            },
        }]
    );
    assert_eq!(campfire.items[0], None);
    assert_eq!(campfire.cooking_progress[0], 0);
    assert_eq!(campfire.cooking_time[0], 0);

    assert!(campfire.place_food(
        PotItemStack {
            item_id: "minecraft:salmon".to_string(),
            count: 1,
        },
        None,
    ));
    campfire.cooking_progress[0] = 5;
    assert!(campfire.place_food(
        PotItemStack {
            item_id: "minecraft:beef".to_string(),
            count: 1,
        },
        Some(10),
    ));
    assert_eq!(campfire.cooking_time[0], 600);
    assert_eq!(campfire.cooking_time[1], 10);

    let saved = campfire.save_additional();
    assert_eq!(CampfireBlockEntity::load_additional(&saved), campfire);
    assert_eq!(
        campfire.get_update_tag(),
        Tag::Compound(vec![(
            "Items".to_string(),
            Tag::List(vec![
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(0)),
                    (
                        "id".to_string(),
                        Tag::String("minecraft:salmon".to_string())
                    ),
                    ("count".to_string(), Tag::Int(1)),
                ]),
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(1)),
                    ("id".to_string(), Tag::String("minecraft:beef".to_string())),
                    ("count".to_string(), Tag::Int(1)),
                ]),
            ])
        )])
    );

    campfire.clear_content();
    assert!(campfire.items.iter().all(Option::is_none));
    assert_eq!(campfire.cooldown_tick(), vec![CampfireTickResult::Changed]);
    assert_eq!(campfire.cooking_progress[0], 3);
}

