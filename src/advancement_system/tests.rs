use super::*;

fn display(hidden: bool, announce_chat: bool) -> AdvancementDisplay {
    AdvancementDisplay {
        icon: Identifier::parse("minecraft:stone").unwrap(),
        title: "Title".to_string(),
        description: "Description".to_string(),
        title_json: serde_json::json!("Title"),
        description_json: serde_json::json!("Description"),
        background: None,
        frame: AdvancementFrame::Task,
        show_toast: true,
        announce_chat,
        hidden,
        x: 0.0,
        y: 0.0,
    }
}

#[test]
fn advancement_progress_grants_revokes_requirements_and_rewards() {
    let rewards = AdvancementRewards {
        experience: 5,
        loot: vec![Identifier::parse("minecraft:loot").unwrap()],
        recipes: vec![Identifier::parse("minecraft:stone").unwrap()],
        function: Some(Identifier::parse("minecraft:reward").unwrap()),
    };
    let definition = AdvancementDefinition::all_of(
        "minecraft:story/root",
        None,
        &["tick"],
        rewards,
        Some(display(false, true)),
    )
    .unwrap();
    let mut player = PlayerAdvancementSet::default();
    let reward = player.grant(&definition, "tick", 1_700_000_000).unwrap();
    assert_eq!(reward.experience, 5);
    assert!(reward.announce_chat);
    assert!(player.is_done(&definition.id));
    assert_eq!(
        player
            .progress
            .get(&definition.id)
            .unwrap()
            .completed_criteria(),
        vec!["tick".to_string()]
    );
    assert!(player.revoke(&definition, "tick"));
    assert!(!player.is_done(&definition.id));
}

#[test]
fn advancement_visibility_layout_packet_and_persistence_follow_vanilla_shapes() {
    let mut definitions = vec![
        AdvancementDefinition::all_of(
            "minecraft:story/root",
            None,
            &["tick"],
            AdvancementRewards::default(),
            Some(AdvancementDisplay {
                frame: AdvancementFrame::Challenge,
                ..display(false, false)
            }),
        )
        .unwrap(),
        AdvancementDefinition::any_of(
            "minecraft:story/hidden",
            Some("minecraft:story/root"),
            &["stone", "iron"],
            Some(AdvancementDisplay {
                frame: AdvancementFrame::Goal,
                ..display(true, false)
            }),
        )
        .unwrap(),
    ];
    assign_tree_layout(&mut definitions);
    assert_eq!(definitions[0].display.as_ref().unwrap().x, 0.0);
    assert_eq!(definitions[1].display.as_ref().unwrap().x, 1.0);

    let mut player = PlayerAdvancementSet::default();
    assert!(!definitions[1].visible_to(&player, &definitions));
    player.grant(&definitions[0], "tick", 1).unwrap();
    assert!(definitions[1].visible_to(&player, &definitions));
    player.grant(&definitions[1], "iron", 2);
    assert!(player.is_done(&definitions[1].id));

    let packet = player.drain_packet(&definitions);
    assert_eq!(packet.added.len(), 2);
    assert_eq!(packet.removed.len(), 0);
    assert!(!packet.reset);
    assert!(player.drain_packet(&definitions).added.is_empty());

    let json = player.to_vanilla_json(4189);
    assert!(json.contains("\"DataVersion\":4189"));
    assert!(json.contains("\"minecraft:story/root\""));
    assert!(json.contains("\"done\":true"));
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn advancement_json_loader_decodes_vanilla_codec_fields() {
    let vanilla_data = |parts: &[&str]| {
        let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
            panic!("VIBECRAFT_DECOMPILED_SOURCE_ROOT is required for Java-source parity tests");
        };
        std::fs::read_to_string(
            parts
                .iter()
                .fold(std::path::PathBuf::from(source_root), |path, part| {
                    path.join(part)
                }),
        )
        .expect("vanilla advancement JSON should be readable")
    };

    let mut root = AdvancementDefinition::from_json(
        "minecraft:story/root",
        &vanilla_data(&[
            "data",
            "minecraft",
            "advancement",
            "story",
            "root.json",
        ]),
    )
    .unwrap();
    assert_eq!(root.parent, None);
    assert_eq!(
        root.criteria,
        BTreeSet::from(["crafting_table".to_string()])
    );
    assert_eq!(root.requirements, vec![vec!["crafting_table".to_string()]]);
    assert!(root.sends_telemetry_event);
    let root_display = root.display.as_ref().unwrap();
    assert_eq!(root_display.title, "advancements.story.root.title");
    assert!(!root_display.show_toast);
    assert!(!root_display.announce_chat);
    assert_eq!(root_display.frame, AdvancementFrame::Task);

    let mine_stone = AdvancementDefinition::from_json(
        "minecraft:story/mine_stone",
        &vanilla_data(&[
            "data",
            "minecraft",
            "advancement",
            "story",
            "mine_stone.json",
        ]),
    )
    .unwrap();
    assert_eq!(
        mine_stone.parent,
        Some(Identifier::parse("minecraft:story/root").unwrap())
    );
    assert_eq!(
        mine_stone.display.as_ref().unwrap().description,
        "advancements.story.mine_stone.description"
    );
    assert!(mine_stone.display.as_ref().unwrap().show_toast);
    assert!(mine_stone.display.as_ref().unwrap().announce_chat);

    let recipe = AdvancementDefinition::from_json(
        "minecraft:recipes/decorations/crafting_table",
        &vanilla_data(&[
            "data",
            "minecraft",
            "advancement",
            "recipes",
            "decorations",
            "crafting_table.json",
        ]),
    )
    .unwrap();
    assert_eq!(
        recipe.rewards.recipes,
        vec![Identifier::parse("minecraft:crafting_table").unwrap()]
    );
    assert!(recipe.display.is_none());

    let reward = AdvancementDefinition::from_json(
        "minecraft:test/reward",
        r#"{
            "criteria": {
                "tick": {
                    "trigger": "minecraft:tick"
                }
            },
            "rewards": {
                "experience": 25,
                "loot": [
                    "minecraft:advancements/test_reward"
                ]
            }
        }"#,
    )
    .unwrap();
    assert_eq!(reward.rewards.experience, 25);
    assert_eq!(
        reward.rewards.loot,
        vec![Identifier::parse("minecraft:advancements/test_reward").unwrap()]
    );

    assign_tree_layout(std::slice::from_mut(&mut root));
    assert_eq!(root.display.as_ref().unwrap().x, 0.0);
}

#[test]
fn advancement_json_loader_rejects_invalid_requirements() {
    assert!(
        AdvancementDefinition::from_json("minecraft:test/empty", r#"{"criteria":{}}"#)
            .unwrap_err()
            .contains("criteria cannot be empty")
    );

    assert!(AdvancementDefinition::from_json(
        "minecraft:test/bad_requirement",
        r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}},"requirements":[["missing"]]}"#
    )
    .unwrap_err()
    .contains("unknown criterion"));
}

#[test]
fn recipe_unlocks_skip_special_known_recipes_and_trigger_advancement_criteria() {
    let recipe = Identifier::parse("minecraft:oak_planks").unwrap();
    let special = Identifier::parse("minecraft:special").unwrap();
    let advancement = AdvancementDefinition::all_of(
        "minecraft:recipes/building_blocks/oak_planks",
        None,
        &["has_the_recipe"],
        AdvancementRewards {
            experience: 1,
            loot: Vec::new(),
            recipes: vec![recipe.clone()],
            function: None,
        },
        Some(display(false, true)),
    )
    .unwrap();
    let triggers = vec![RecipeUnlockedCriterion {
        advancement: advancement.id.clone(),
        criterion: "has_the_recipe".to_string(),
        recipe: recipe.clone(),
    }];
    let mut unlocks = PlayerRecipeUnlocks::default();
    let mut progress = PlayerAdvancementSet::default();

    let events = unlocks.unlock_recipes(
        &[
            RecipeDefinition {
                id: recipe.clone(),
                special: false,
                show_notification: true,
            },
            RecipeDefinition {
                id: special.clone(),
                special: true,
                show_notification: true,
            },
        ],
        std::slice::from_ref(&advancement),
        &triggers,
        &mut progress,
        10,
    );

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].recipe, recipe);
    assert!(events[0].show_notification);
    assert!(events[0].highlight);
    assert_eq!(events[0].advancement_rewards[0].experience, 1);
    assert!(unlocks.contains(&events[0].recipe));
    assert!(unlocks.highlighted(&events[0].recipe));
    assert!(!unlocks.contains(&special));
    assert!(progress.is_done(&advancement.id));

    assert!(unlocks
        .unlock_recipes(
            &[RecipeDefinition {
                id: events[0].recipe.clone(),
                special: false,
                show_notification: true,
            }],
            &[advancement],
            &triggers,
            &mut progress,
            11,
        )
        .is_empty());
}

#[test]
fn recipe_unlocks_remove_clear_highlight_and_load_only_valid_recipes() {
    let stone = Identifier::parse("minecraft:stone").unwrap();
    let dirt = Identifier::parse("minecraft:dirt").unwrap();
    let bad = Identifier::parse("minecraft:bad").unwrap();
    let mut unlocks = PlayerRecipeUnlocks::load_untrusted(
        vec![stone.clone(), bad.clone()],
        vec![stone.clone(), dirt.clone()],
        |recipe| recipe != &bad,
    );
    assert!(unlocks.contains(&stone));
    assert!(!unlocks.contains(&bad));
    assert!(unlocks.highlighted(&stone));
    assert!(!unlocks.highlighted(&dirt));

    unlocks.clear_highlight(&stone);
    assert!(!unlocks.highlighted(&stone));
    assert_eq!(unlocks.remove_recipes(&[stone.clone(), dirt]), vec![stone]);
    let (known, highlight) = unlocks.pack();
    assert!(known.is_empty());
    assert!(highlight.is_empty());
}

#[test]
fn advancement_type_model_matches_java_enum_fields_and_announcement() {
    assert_eq!(
        AdvancementFrame::VALUES.map(AdvancementFrame::serialized_name),
        ["task", "challenge", "goal"]
    );
    assert_eq!(AdvancementFrame::Task.chat_color(), ChatFormatting::Green);
    assert_eq!(
        AdvancementFrame::Challenge.chat_color(),
        ChatFormatting::DarkPurple
    );
    assert_eq!(AdvancementFrame::Goal.chat_color(), ChatFormatting::Green);
    assert_eq!(
        AdvancementFrame::Challenge.display_name_translation_key(),
        "advancements.toast.challenge"
    );
    assert_eq!(
        AdvancementFrame::Goal.announcement_translation_key(),
        "chat.type.advancement.goal"
    );
    assert_eq!(
        AdvancementFrame::Task.create_announcement("Steve", "Stone Age"),
        "chat.type.advancement.task Steve Stone Age"
    );
    assert_eq!(
        AdvancementFrame::from_serialized_name("challenge"),
        Some(AdvancementFrame::Challenge)
    );
    assert_eq!(AdvancementFrame::from_serialized_name("unknown"), None);
}

#[test]
fn criterion_progress_model_matches_java_done_grant_revoke_and_network() {
    let mut progress = CriterionProgressModel::new();
    assert!(!progress.is_done());
    assert_eq!(progress.get_obtained_epoch_millis(), None);
    assert_eq!(progress.to_string(), "CriterionProgress{obtained=false}");
    assert_eq!(
        progress.to_network_data(),
        CriterionProgressData {
            obtained_epoch_millis: None
        }
    );

    progress.grant_at_epoch_millis(1_700_000_000_123);
    assert!(progress.is_done());
    assert_eq!(
        progress.get_obtained_epoch_millis(),
        Some(1_700_000_000_123)
    );
    assert_eq!(
        progress.to_string(),
        "CriterionProgress{obtained=1700000000123}"
    );
    assert_eq!(
        CriterionProgressModel::from_network_data(progress.to_network_data()),
        progress
    );

    progress.revoke();
    assert!(!progress.is_done());
    assert_eq!(
        CriterionProgressModel::from_obtained_epoch_millis(42).to_network_data(),
        CriterionProgressData {
            obtained_epoch_millis: Some(42)
        }
    );
}

#[test]
fn advancement_holder_model_matches_java_id_only_identity_methods() {
    let id = Identifier::parse("minecraft:story/root").unwrap();
    let other_id = Identifier::parse("minecraft:story/mine_stone").unwrap();
    let holder =
        AdvancementHolderData::minimal(id.clone(), None, vec![vec!["a".to_string()]], false);
    let same_id_different_value = AdvancementHolderData::minimal(
        id.clone(),
        Some(other_id.clone()),
        vec![vec!["b".to_string()]],
        true,
    );
    let different_id = AdvancementHolderData::minimal(
        other_id.clone(),
        None,
        vec![vec!["a".to_string()]],
        false,
    );

    assert_ne!(holder, same_id_different_value);
    assert!(holder.java_equals_by_id(&same_id_different_value));
    assert!(!holder.java_equals_by_id(&different_id));
    assert_eq!(holder.java_hash_key(), &id);
    assert_eq!(holder.java_to_string(), "minecraft:story/root");
}
