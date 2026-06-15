#[cfg(test)]
mod advancement_criteria;
#[cfg(test)]
mod advancement_display;
#[cfg(test)]
mod advancement_model;
#[cfg(test)]
mod advancement_progress;
#[cfg(test)]
mod advancement_rewards;
#[cfg(test)]
mod advancement_system;
#[cfg(test)]
mod advancement_tree;
#[cfg(test)]
mod advancement_tree_position;
#[cfg(test)]
mod advancement_trigger_registry;
#[cfg(test)]
mod advancement_visibility_evaluator;
mod ai_system;
mod attribute_system;
mod base_entity;
mod biome;
mod block_behavior;
#[cfg(test)]
mod block_behavior_tests;
mod block_behaviour_defaults;
#[cfg(test)]
mod block_behaviour_defaults_tests;
mod block_behaviour_properties;
mod block_catalog;
mod block_entity;
mod block_metadata;
mod block_placement;
mod block_properties;
#[cfg(test)]
mod block_regression;
mod block_scheduled_ticks;
mod block_shape_updates;
mod block_sounds;
mod block_states;
mod block_survival;
mod block_tags;
mod block_transforms;
mod block_update;
mod boss_fight;
#[cfg(test)]
mod bootstrap;
mod char_predicate;
mod chained_json_exception;
mod chat_component;
mod chat_formatting;
mod chat_trust;
#[cfg(test)]
mod chase;
mod chunk_manager;
mod chunk_task;
mod chunk_ticket;
mod chunk_watchdog;
mod cli;
mod collision_shape;
mod combat_damage;
mod combat_tracker;
mod command;
#[cfg(test)]
mod command_banlist;
#[cfg(test)]
mod command_angle_argument;
#[cfg(test)]
mod command_argument_signatures;
#[cfg(test)]
mod command_argument_visitor;
#[cfg(test)]
mod command_block_arguments;
#[cfg(test)]
mod command_block_position_arguments;
#[cfg(test)]
mod command_brigadier_exceptions;
#[cfg(test)]
mod command_pardon_ip;
#[cfg(test)]
mod command_build_context;
#[cfg(test)]
mod command_cacheable_function;
#[cfg(test)]
mod command_color_argument;
#[cfg(test)]
mod command_commands;
#[cfg(test)]
mod command_component_argument;
#[cfg(test)]
mod command_coordinate_arguments;
#[cfg(test)]
mod command_dimension_argument;
#[cfg(test)]
mod command_entity_anchor_argument;
#[cfg(test)]
mod command_entity_argument;
mod command_execution;
#[cfg(test)]
mod command_execution_source;
mod command_feedback;
#[cfg(test)]
mod command_function_instantiation_exception;
#[cfg(test)]
mod command_game_mode_argument;
#[cfg(test)]
mod command_game_profile_argument;
#[cfg(test)]
mod command_heightmap_type_argument;
#[cfg(test)]
mod command_hex_color_argument;
#[cfg(test)]
mod command_identifier_argument;
#[cfg(test)]
mod command_item_arguments;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod command_message_argument;
#[cfg(test)]
mod command_misc_argument_audits;
#[cfg(test)]
mod command_nbt_arguments;
#[cfg(test)]
mod command_nbt_path_argument;
#[cfg(test)]
mod command_objective_argument;
#[cfg(test)]
mod command_objective_criteria_argument;
#[cfg(test)]
mod command_operation_argument;
mod command_parity;
#[cfg(test)]
mod command_parser_utils;
#[cfg(test)]
mod command_particle_argument;
#[cfg(test)]
mod command_range_argument;
#[cfg(test)]
mod command_resource_argument;
#[cfg(test)]
mod command_resource_key_argument;
#[cfg(test)]
mod command_resource_or_id_argument;
#[cfg(test)]
mod command_resource_or_tag_argument;
#[cfg(test)]
mod command_resource_or_tag_key_argument;
#[cfg(test)]
mod command_resource_selector_argument;
#[cfg(test)]
mod command_result_callback;
#[cfg(test)]
mod command_score_holder_argument;
#[cfg(test)]
mod command_scoreboard_slot_argument;
#[cfg(test)]
mod command_selector;
#[cfg(test)]
mod command_shared_suggestion_provider;
#[cfg(test)]
mod command_signing_context;
#[cfg(test)]
mod command_slot_arguments;
#[cfg(test)]
mod command_source;
#[cfg(test)]
mod command_source_stack;
#[cfg(test)]
mod command_string_representable_argument;
#[cfg(test)]
mod command_style_argument;
#[cfg(test)]
mod command_swizzle_argument;
mod command_synchronization;
#[cfg(test)]
mod command_team_argument;
#[cfg(test)]
mod command_template_transform_arguments;
#[cfg(test)]
mod command_time_argument;
mod command_tree;
#[cfg(test)]
mod command_uuid_argument;
mod console;
mod container_block;
mod container_menus;
#[cfg(test)]
mod core_block_box;
#[cfg(test)]
mod core_block_math;
#[cfg(test)]
mod core_block_pos;
#[cfg(test)]
mod core_component_predicates;
#[cfg(test)]
mod core_components;
#[cfg(test)]
mod core_defaulted_registry;
#[cfg(test)]
mod core_direction;
#[cfg(test)]
mod core_dispenser;
#[cfg(test)]
mod core_holders;
#[cfg(test)]
mod core_layered_registry;
#[cfg(test)]
mod core_mapped_registry;
#[cfg(test)]
mod core_misc;
#[cfg(test)]
mod core_orientation;
#[cfg(test)]
mod core_particles;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod core_registries;
#[cfg(test)]
mod core_registry_codecs;
#[cfg(test)]
mod core_registry_helpers;
#[cfg(test)]
mod core_registry_interface;
#[cfg(test)]
mod core_registry_set_builder;
#[cfg(test)]
mod core_registry_synchronization;
#[cfg(test)]
mod core_section_pos;
mod crash;
mod crash_recovery_tests;
#[cfg(test)]
mod creative_inventory;
#[cfg(test)]
mod criterion_bee_nest;
#[cfg(test)]
mod criterion_block_interaction;
#[cfg(test)]
mod criterion_block_predicate;
#[cfg(test)]
mod criterion_bred_animals;
#[cfg(test)]
mod criterion_brewed_potion;
#[cfg(test)]
mod criterion_change_dimension;
#[cfg(test)]
mod criterion_channeled_lightning;
#[cfg(test)]
mod criterion_collection_predicates;
#[cfg(test)]
mod criterion_construct_beacon;
#[cfg(test)]
mod criterion_consume_item;
#[cfg(test)]
mod criterion_context_aware_predicate;
#[cfg(test)]
mod criterion_cured_zombie_villager;
#[cfg(test)]
mod criterion_damage_predicate;
#[cfg(test)]
mod criterion_damage_source_predicate;
#[cfg(test)]
mod criterion_data_component_matchers;
#[cfg(test)]
mod criterion_distance_predicate;
#[cfg(test)]
mod criterion_distance_trigger;
#[cfg(test)]
mod criterion_effects_changed;
#[cfg(test)]
mod criterion_enchanted_item;
#[cfg(test)]
mod criterion_enchantment_predicate;
#[cfg(test)]
mod criterion_enter_block;
#[cfg(test)]
mod criterion_entity_equipment_predicate;
#[cfg(test)]
mod criterion_entity_flags_predicate;
#[cfg(test)]
mod criterion_entity_hurt_player;
#[cfg(test)]
mod criterion_entity_predicate;
#[cfg(test)]
mod criterion_entity_sub_predicates;
#[cfg(test)]
mod criterion_entity_type_predicate;
#[cfg(test)]
mod criterion_fall_after_explosion;
#[cfg(test)]
mod criterion_filled_bucket;
#[cfg(test)]
mod criterion_fishing_hook_predicate;
#[cfg(test)]
mod criterion_fishing_rod_hooked;
#[cfg(test)]
mod criterion_fluid_predicate;
#[cfg(test)]
mod criterion_food_predicate;
#[cfg(test)]
mod criterion_game_type_predicate;
#[cfg(test)]
mod criterion_impossible_trigger;
#[cfg(test)]
mod criterion_input_predicate;
#[cfg(test)]
mod criterion_inventory_change;
#[cfg(test)]
mod criterion_item_durability;
#[cfg(test)]
mod criterion_item_predicate;
#[cfg(test)]
mod criterion_killed_by_arrow;
#[cfg(test)]
mod criterion_killed_trigger;
#[cfg(test)]
mod criterion_levitation;
#[cfg(test)]
mod criterion_light_predicate;
#[cfg(test)]
mod criterion_lightning_bolt_predicate;
#[cfg(test)]
mod criterion_lightning_strike;
#[cfg(test)]
mod criterion_location_predicate;
#[cfg(test)]
mod criterion_loot_table_trigger;
#[cfg(test)]
mod criterion_min_max_bounds;
#[cfg(test)]
mod criterion_movement_predicate;
#[cfg(test)]
mod criterion_nbt_predicate;
#[cfg(test)]
mod criterion_picked_up_item_trigger;
#[cfg(test)]
mod criterion_player_hurt_entity;
#[cfg(test)]
mod criterion_player_interact;
#[cfg(test)]
mod criterion_player_predicate;
#[cfg(test)]
mod criterion_player_trigger;
#[cfg(test)]
mod criterion_raider_predicate;
#[cfg(test)]
mod criterion_recipe_crafted;
#[cfg(test)]
mod criterion_recipe_unlocked;
#[cfg(test)]
mod criterion_sheep_predicate;
#[cfg(test)]
mod criterion_shot_crossbow;
#[cfg(test)]
mod criterion_simple_trigger;
#[cfg(test)]
mod criterion_single_component_item_predicate;
#[cfg(test)]
mod criterion_slide_down_block;
#[cfg(test)]
mod criterion_slime_predicate;
#[cfg(test)]
mod criterion_slots_predicate;
#[cfg(test)]
mod criterion_spear_mobs;
#[cfg(test)]
mod criterion_start_riding;
#[cfg(test)]
mod criterion_state_properties_predicate;
#[cfg(test)]
mod criterion_summoned_entity;
#[cfg(test)]
mod criterion_tag_predicate;
#[cfg(test)]
mod criterion_tame_animal;
#[cfg(test)]
mod criterion_target_block;
#[cfg(test)]
mod criterion_trade_trigger;
#[cfg(test)]
mod criterion_used_ender_eye;
#[cfg(test)]
mod criterion_used_totem;
#[cfg(test)]
mod criterion_using_item;
pub mod custom_boss_events;
mod damage_type;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_advancement_packs;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_advancements;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_info;
#[cfg(test)]
mod data_loot;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_block;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_chest;
#[cfg(test)]
mod data_loot_entity;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_gift;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_packs;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_piglin_barter;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_provider;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_shearing;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_loot_vanilla_entity;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_metadata;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_package;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_recipe_unlock_advancement;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_recipes;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_recipes_package;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_registries;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_shaped_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_shapeless_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_simple_cooking_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_single_item_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_smithing_transform_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_smithing_trim_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_special_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_structures;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_biomes;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_block_items;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_core;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_entity_types;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_medium_providers;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_registry_providers;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_trade_rebalance_trades;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_vanilla_blocks;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_vanilla_enchantments;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_vanilla_items;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_tags_villager_trades;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_transmute_recipe_builder;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_vanilla_recipe_provider;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_biome_defaults;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_bootstrap_context;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_carvers;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_dimension_types;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_jigsaw_pools;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_noise_data;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_processor_lists;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_structure_sets;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_structures;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod data_worldgen_surface_rule_data;
#[cfg(test)]
mod datapack_reload_tests;
#[cfg(test)]
mod dialog_system;
mod dispenser_cauldron;
mod enchantment_system;
#[cfg(test)]
mod entity_behavior_tests;
mod entity_category;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod entity_metadata;
mod entity_physics;
mod entity_syncher;
mod entity_validation;
#[cfg(test)]
mod entity_variants;
mod environment_attributes;
#[cfg(test)]
mod equipment_trim;
mod eula;
mod experience_system;
mod fire;
mod fluid;
#[cfg(test)]
mod fuzz_tests;
mod game_event;
mod game_rules;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod gametest_resources;
mod generated_reports;
mod gravity;
mod identifier_exception;
mod inhabited_time;
mod inventory;
mod inventory_transactions;
mod item_alchemy;
mod item_catalog;
mod item_consumable_components;
mod item_consume_effects;
mod item_custom_model_data_component;
mod item_death_protection_component;
mod item_disc_fragment;
mod item_dyed_color_component;
mod item_dye;
mod item_entity;
#[cfg(test)]
mod item_family_behavior;
mod item_flint_and_steel;
mod item_honeycomb;
mod item_instrument_component;
mod item_lore_component;
mod item_map_components;
mod item_misc_components;
mod item_ominous_bottle_component;
mod item_properties;
mod item_sign_applicator;
mod item_stack;
mod item_swing_animation_component;
mod item_tags;
mod item_tooltip_components;
mod item_tool_use;
mod item_weapon_component;
mod lighting;
mod living_entity;
mod localization_keys;
mod log;
mod logged_print_stream;
mod loot_system;
mod management_security;
mod management_server;
mod map_state;
#[cfg(test)]
mod mob_family;
mod mob_interaction;
#[cfg(test)]
mod movement_physics;
#[cfg(test)]
mod movement_validation;
mod network;
#[cfg(test)]
mod non_living_entity;
#[cfg(test)]
mod operational_coverage;
mod optionull;
#[cfg(test)]
mod parity_harness;
mod performance_benchmarks;
#[cfg(test)]
mod persistence_roundtrip_tests;
mod plant;
mod player;
mod player_access;
mod player_entity;
mod player_game_mode;
mod player_inventory;
mod player_list;
mod player_online_auth;
#[cfg(test)]
mod player_presentation;
#[cfg(test)]
mod player_profile_key;
mod portal;
mod post_processing;
#[cfg(test)]
mod potion_fluid_container;
mod presentation_data;
mod project_foundation_tests;
#[cfg(test)]
mod projectile_entity;
mod raid;
mod random_source;
mod random_sequences;
mod random_tick;
mod recipe_system;
mod reference_ids;
mod redstone;
mod registry;
mod reloadable_server_resources;
mod reloadable_server_registries;
mod resource_delegating_ops;
mod resource_dependant_name;
mod resource_file_to_id_converter;
mod resource_holder_set_codec;
mod resource_manager_registry_load_task;
mod resource_network_registry_load_task;
mod resource_registry_data_loader;
mod resource_registry_file_codec;
mod resource_registry_fixed_codec;
mod resource_registry_load_task;
mod resource_registry_ops;
mod resource_registry_validator;
mod report_type;
mod resources;
mod respawn;
mod running_on_different_thread_exception;
mod runtime;
mod scheduled_tick;
mod seed_validation;
#[cfg(test)]
mod server_advancement_manager;
mod server_function_library;
mod server_function_manager;
mod server_info;
mod server_interface;
mod server_links;
mod server_registry_layer;
mod server_tick_rate_manager;
mod services;
mod suppressed_exception_collector;
mod server_tick_task;
mod server_properties;
mod shared_constants;
mod spawning;
mod special_block;
#[cfg(test)]
mod statistics;
mod status_effect;
mod storage;
mod structure_resources;
mod suppress_forbidden;
mod system_report;
mod tracing_executor;
mod trial_system;
#[cfg(test)]
mod vehicle_entity;
mod vibration;
mod villager_system;
mod villager_trade_resources;
#[cfg(test)]
mod waypoint;
mod weather;
mod world;
mod world_border;
mod world_loader;
mod world_stem;
mod world_time;
mod world_version;
mod worldgen;
mod worldgen_comparison;
mod worldgen_resources;
