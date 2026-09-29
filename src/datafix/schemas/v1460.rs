//! Port of `net.minecraft.util.datafix.schemas.V1460`.

use super::v1451_6;
use super::v1458;
use super::v705;
use super::v99;
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::{dsl, ChoiceSet};
#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
fn register_entities(schema: &mut Schema) {
    schema.clear_entities();
    schema.register_entity(
        "minecraft:area_effect_cloud",
        dsl::optional_fields(vec![("Particle", dsl::reference(r::PARTICLE))]),
    );
    schema.register_simple_entity("minecraft:armor_stand");
    schema.register_entity(
        "minecraft:arrow",
        dsl::optional_fields(vec![("inBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:bat");
    schema.register_simple_entity("minecraft:blaze");
    schema.register_simple_entity("minecraft:boat");
    schema.register_simple_entity("minecraft:cave_spider");
    schema.register_entity(
        "minecraft:chest_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_simple_entity("minecraft:chicken");
    schema.register_entity(
        "minecraft:commandblock_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("LastOutput", dsl::reference(r::TEXT_COMPONENT)),
        ]),
    );
    schema.register_simple_entity("minecraft:cow");
    schema.register_simple_entity("minecraft:creeper");
    schema.register_entity(
        "minecraft:donkey",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:dragon_fireball");
    schema.register_simple_entity("minecraft:egg");
    schema.register_simple_entity("minecraft:elder_guardian");
    schema.register_simple_entity("minecraft:ender_crystal");
    schema.register_simple_entity("minecraft:ender_dragon");
    schema.register_entity(
        "minecraft:enderman",
        dsl::optional_fields(vec![("carriedBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:endermite");
    schema.register_simple_entity("minecraft:ender_pearl");
    schema.register_simple_entity("minecraft:evocation_fangs");
    schema.register_simple_entity("minecraft:evocation_illager");
    schema.register_simple_entity("minecraft:eye_of_ender_signal");
    schema.register_entity(
        "minecraft:falling_block",
        dsl::optional_fields(vec![
            ("BlockState", dsl::reference(r::BLOCK_STATE)),
            ("TileEntityData", dsl::reference(r::BLOCK_ENTITY)),
        ]),
    );
    schema.register_simple_entity("minecraft:fireball");
    schema.register_entity(
        "minecraft:fireworks_rocket",
        dsl::optional_fields(vec![("FireworksItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_entity(
        "minecraft:furnace_minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:ghast");
    schema.register_simple_entity("minecraft:giant");
    schema.register_simple_entity("minecraft:guardian");
    schema.register_entity(
        "minecraft:hopper_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_entity(
        "minecraft:horse",
        dsl::optional_fields(vec![
            ("ArmorItem", dsl::reference(r::ITEM_STACK)),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:husk");
    schema.register_simple_entity("minecraft:illusion_illager");
    schema.register_entity(
        "minecraft:item",
        dsl::optional_fields(vec![("Item", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_entity(
        "minecraft:item_frame",
        dsl::optional_fields(vec![("Item", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:leash_knot");
    schema.register_entity(
        "minecraft:llama",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
            ("DecorItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:llama_spit");
    schema.register_simple_entity("minecraft:magma_cube");
    schema.register_entity(
        "minecraft:minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:mooshroom");
    schema.register_entity(
        "minecraft:mule",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:ocelot");
    schema.register_simple_entity("minecraft:painting");
    schema.register_simple_entity("minecraft:parrot");
    schema.register_simple_entity("minecraft:pig");
    schema.register_simple_entity("minecraft:polar_bear");
    schema.register_entity(
        "minecraft:potion",
        dsl::optional_fields(vec![("Potion", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:rabbit");
    schema.register_simple_entity("minecraft:sheep");
    schema.register_simple_entity("minecraft:shulker");
    schema.register_simple_entity("minecraft:shulker_bullet");
    schema.register_simple_entity("minecraft:silverfish");
    schema.register_simple_entity("minecraft:skeleton");
    schema.register_entity(
        "minecraft:skeleton_horse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:slime");
    schema.register_simple_entity("minecraft:small_fireball");
    schema.register_simple_entity("minecraft:snowball");
    schema.register_simple_entity("minecraft:snowman");
    schema.register_entity(
        "minecraft:spawner_minecart",
        dsl::optional_fields_with_rest(
            vec![("DisplayState", dsl::reference(r::BLOCK_STATE))],
            dsl::reference(r::UNTAGGED_SPAWNER),
        ),
    );
    schema.register_entity(
        "minecraft:spectral_arrow",
        dsl::optional_fields(vec![("inBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:spider");
    schema.register_simple_entity("minecraft:squid");
    schema.register_simple_entity("minecraft:stray");
    schema.register_simple_entity("minecraft:tnt");
    schema.register_entity(
        "minecraft:tnt_minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_simple_entity("minecraft:vex");
    schema.register_entity(
        "minecraft:villager",
        dsl::optional_fields(vec![
            ("Inventory", dsl::list(dsl::reference(r::ITEM_STACK))),
            (
                "Offers",
                dsl::optional_fields(vec![(
                    "Recipes",
                    dsl::list(dsl::reference(r::VILLAGER_TRADE)),
                )]),
            ),
        ]),
    );
    schema.register_simple_entity("minecraft:villager_golem");
    schema.register_simple_entity("minecraft:vindication_illager");
    schema.register_simple_entity("minecraft:witch");
    schema.register_simple_entity("minecraft:wither");
    schema.register_simple_entity("minecraft:wither_skeleton");
    schema.register_simple_entity("minecraft:wither_skull");
    schema.register_simple_entity("minecraft:wolf");
    schema.register_simple_entity("minecraft:xp_bottle");
    schema.register_simple_entity("minecraft:xp_orb");
    schema.register_simple_entity("minecraft:zombie");
    schema.register_entity(
        "minecraft:zombie_horse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:zombie_pigman");
    schema.register_entity(
        "minecraft:zombie_villager",
        dsl::optional_fields(vec![(
            "Offers",
            dsl::optional_fields(vec![(
                "Recipes",
                dsl::list(dsl::reference(r::VILLAGER_TRADE)),
            )]),
        )]),
    );
}

fn register_block_entities(schema: &mut Schema) {
    schema.clear_block_entities();
    schema.register_block_entity(
        "minecraft:furnace",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_block_entity(
        "minecraft:chest",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_block_entity(
        "minecraft:trapped_chest",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_simple_block_entity("minecraft:ender_chest");
    schema.register_block_entity(
        "minecraft:jukebox",
        dsl::optional_fields(vec![("RecordItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_block_entity(
        "minecraft:dispenser",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_block_entity(
        "minecraft:dropper",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_block_entity("minecraft:sign", v99::sign());
    schema.register_block_entity("minecraft:mob_spawner", dsl::reference(r::UNTAGGED_SPAWNER));
    schema.register_block_entity(
        "minecraft:piston",
        dsl::optional_fields(vec![("blockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_block_entity(
        "minecraft:brewing_stand",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_block_entity("minecraft:enchanting_table", v1458::nameable());
    schema.register_simple_block_entity("minecraft:end_portal");
    schema.register_block_entity("minecraft:beacon", v1458::nameable());
    schema.register_block_entity(
        "minecraft:skull",
        dsl::optional_fields(vec![("custom_name", dsl::reference(r::TEXT_COMPONENT))]),
    );
    schema.register_simple_block_entity("minecraft:daylight_detector");
    schema.register_block_entity(
        "minecraft:hopper",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_simple_block_entity("minecraft:comparator");
    schema.register_block_entity("minecraft:banner", v1458::nameable());
    schema.register_simple_block_entity("minecraft:structure_block");
    schema.register_simple_block_entity("minecraft:end_gateway");
    schema.register_block_entity(
        "minecraft:command_block",
        dsl::optional_fields(vec![("LastOutput", dsl::reference(r::TEXT_COMPONENT))]),
    );
    schema.register_block_entity(
        "minecraft:shulker_box",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
    schema.register_simple_block_entity("minecraft:bed");
}

#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
fn register_types(schema: &mut Schema) {
    schema.clear_types();
    schema.register_type(
        r::LEVEL,
        dsl::optional_fields_with_rest(
            vec![(
                "CustomBossEvents",
                dsl::compound_list(dsl::optional_fields(vec![(
                    "Name",
                    dsl::reference(r::TEXT_COMPONENT),
                )])),
            )],
            dsl::reference(r::LIGHTWEIGHT_LEVEL),
        ),
    );
    schema.register_type(r::LIGHTWEIGHT_LEVEL, dsl::remainder());
    schema.register_type(r::RECIPE, dsl::namespaced_string());
    schema.register_type(
        r::PLAYER,
        dsl::optional_fields(vec![
            (
                "RootVehicle",
                dsl::optional_fields(vec![("Entity", dsl::reference(r::ENTITY_TREE))]),
            ),
            ("ender_pearls", dsl::list(dsl::reference(r::ENTITY_TREE))),
            ("Inventory", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("EnderItems", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("ShoulderEntityLeft", dsl::reference(r::ENTITY_TREE)),
            ("ShoulderEntityRight", dsl::reference(r::ENTITY_TREE)),
            (
                "recipeBook",
                dsl::optional_fields(vec![
                    ("recipes", dsl::list(dsl::reference(r::RECIPE))),
                    ("toBeDisplayed", dsl::list(dsl::reference(r::RECIPE))),
                ]),
            ),
        ]),
    );
    schema.register_type(
        r::CHUNK,
        dsl::fields(vec![(
            "Level",
            dsl::optional_fields(vec![
                ("Entities", dsl::list(dsl::reference(r::ENTITY_TREE))),
                (
                    "TileEntities",
                    dsl::list(dsl::or(dsl::reference(r::BLOCK_ENTITY), dsl::remainder())),
                ),
                (
                    "TileTicks",
                    dsl::list(dsl::fields(vec![("i", dsl::reference(r::BLOCK_NAME))])),
                ),
                (
                    "Sections",
                    dsl::list(dsl::optional_fields(vec![(
                        "Palette",
                        dsl::list(dsl::reference(r::BLOCK_STATE)),
                    )])),
                ),
            ]),
        )]),
    );
    schema.register_type(
        r::BLOCK_ENTITY,
        dsl::optional_fields_with_rest(
            vec![("components", dsl::reference(r::DATA_COMPONENTS))],
            dsl::namespaced_choice(ChoiceSet::BlockEntities),
        ),
    );
    schema.register_type(
        r::ENTITY_TREE,
        dsl::optional_fields_with_rest(
            vec![("Passengers", dsl::list(dsl::reference(r::ENTITY_TREE)))],
            dsl::reference(r::ENTITY),
        ),
    );
    schema.register_type(
        r::ENTITY,
        dsl::and(vec![
            dsl::reference(r::ENTITY_EQUIPMENT),
            dsl::optional_fields_with_rest(
                vec![("CustomName", dsl::reference(r::TEXT_COMPONENT))],
                dsl::namespaced_choice(ChoiceSet::Entities),
            ),
        ]),
    );
    schema.register_type(
        r::ITEM_STACK,
        dsl::hook(
            dsl::optional_fields(vec![
                ("id", dsl::reference(r::ITEM_NAME)),
                ("tag", v99::item_stack_tag()),
            ]),
            v705::add_names,
        ),
    );
    schema.register_type(
        r::HOTBAR,
        dsl::compound_list(dsl::list(dsl::reference(r::ITEM_STACK))),
    );
    schema.register_type(r::OPTIONS, dsl::remainder());
    schema.register_type(
        r::STRUCTURE,
        dsl::optional_fields(vec![
            (
                "entities",
                dsl::list(dsl::optional_fields(vec![(
                    "nbt",
                    dsl::reference(r::ENTITY_TREE),
                )])),
            ),
            (
                "blocks",
                dsl::list(dsl::optional_fields(vec![(
                    "nbt",
                    dsl::reference(r::BLOCK_ENTITY),
                )])),
            ),
            ("palette", dsl::list(dsl::reference(r::BLOCK_STATE))),
        ]),
    );
    schema.register_type(r::BLOCK_NAME, dsl::namespaced_string());
    schema.register_type(r::ITEM_NAME, dsl::namespaced_string());
    schema.register_type(r::BLOCK_STATE, dsl::remainder());
    schema.register_type(r::FLAT_BLOCK_STATE, dsl::remainder());
    schema.register_type(
        r::STATS,
        dsl::optional_fields(vec![(
            "stats",
            dsl::optional_fields(vec![
                (
                    "minecraft:mined",
                    dsl::compound_list_keyed(dsl::reference(r::BLOCK_NAME), dsl::int()),
                ),
                (
                    "minecraft:crafted",
                    dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int()),
                ),
                (
                    "minecraft:used",
                    dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int()),
                ),
                (
                    "minecraft:broken",
                    dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int()),
                ),
                (
                    "minecraft:picked_up",
                    dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int()),
                ),
                (
                    "minecraft:dropped",
                    dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int()),
                ),
                (
                    "minecraft:killed",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::int()),
                ),
                (
                    "minecraft:killed_by",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::int()),
                ),
                (
                    "minecraft:custom",
                    dsl::compound_list_keyed(dsl::namespaced_string(), dsl::int()),
                ),
            ]),
        )]),
    );
    schema.register_type(r::SAVED_DATA_COMMAND_STORAGE, dsl::remainder());
    schema.register_type(
        r::SAVED_DATA_CUSTOM_BOSS_EVENTS,
        dsl::optional_fields(vec![(
            "data",
            dsl::compound_list(dsl::optional_fields(vec![(
                "Name",
                dsl::reference(r::TEXT_COMPONENT),
            )])),
        )]),
    );
    schema.register_type(r::SAVED_DATA_ENDER_DRAGON_FIGHT, dsl::remainder());
    schema.register_type(r::SAVED_DATA_GAME_RULES, dsl::remainder());
    schema.register_type(r::SAVED_DATA_TICKETS, dsl::remainder());
    schema.register_type(
        r::SAVED_DATA_MAP_DATA,
        dsl::optional_fields(vec![(
            "data",
            dsl::optional_fields(vec![(
                "banners",
                dsl::list(dsl::optional_fields(vec![(
                    "Name",
                    dsl::reference(r::TEXT_COMPONENT),
                )])),
            )]),
        )]),
    );
    schema.register_type(r::SAVED_DATA_MAP_INDEX, dsl::remainder());
    schema.register_type(r::SAVED_DATA_RAIDS, dsl::remainder());
    schema.register_type(r::SAVED_DATA_RANDOM_SEQUENCES, dsl::remainder());
    schema.register_type(r::SAVED_DATA_SCHEDULED_EVENTS, dsl::remainder());
    schema.register_type(
        r::SAVED_DATA_SCOREBOARD,
        dsl::optional_fields(vec![(
            "data",
            dsl::optional_fields(vec![
                ("Objectives", dsl::list(dsl::reference(r::OBJECTIVE))),
                ("Teams", dsl::list(dsl::reference(r::TEAM))),
                (
                    "PlayerScores",
                    dsl::list(dsl::optional_fields(vec![(
                        "display",
                        dsl::reference(r::TEXT_COMPONENT),
                    )])),
                ),
            ]),
        )]),
    );
    schema.register_type(r::SAVED_DATA_STOPWATCHES, dsl::remainder());
    schema.register_type(
        r::SAVED_DATA_STRUCTURE_FEATURE_INDICES,
        dsl::optional_fields(vec![(
            "data",
            dsl::optional_fields(vec![(
                "Features",
                dsl::compound_list(dsl::reference(r::STRUCTURE_FEATURE)),
            )]),
        )]),
    );
    schema.register_type(r::SAVED_DATA_WANDERING_TRADER, dsl::remainder());
    schema.register_type(r::SAVED_DATA_WEATHER, dsl::remainder());
    schema.register_type(r::SAVED_DATA_WORLD_BORDER, dsl::remainder());
    schema.register_type(r::SAVED_DATA_WORLD_CLOCKS, dsl::remainder());
    schema.register_type(
        r::SAVED_DATA_WORLD_GEN_SETTINGS,
        dsl::fields(vec![("data", dsl::reference(r::WORLD_GEN_SETTINGS))]),
    );
    schema.register_type(r::DEBUG_PROFILE, dsl::remainder());
    schema.register_type(r::STRUCTURE_FEATURE, dsl::remainder());
    v1451_6::create_criterion_types(schema);
    schema.register_type(
        r::OBJECTIVE,
        dsl::hook_full(
            dsl::optional_fields(vec![
                ("CriteriaType", dsl::named_choice("type", "criterionTypes")),
                ("DisplayName", dsl::reference(r::TEXT_COMPONENT)),
            ]),
            v1451_6::unpack_objective_id,
            v1451_6::repack_objective_id,
        ),
    );
    schema.register_type(
        r::TEAM,
        dsl::optional_fields(vec![
            ("MemberNamePrefix", dsl::reference(r::TEXT_COMPONENT)),
            ("MemberNameSuffix", dsl::reference(r::TEXT_COMPONENT)),
            ("DisplayName", dsl::reference(r::TEXT_COMPONENT)),
        ]),
    );
    schema.register_type(
        r::UNTAGGED_SPAWNER,
        dsl::optional_fields(vec![
            (
                "SpawnPotentials",
                dsl::list(dsl::fields(vec![(
                    "Entity",
                    dsl::reference(r::ENTITY_TREE),
                )])),
            ),
            ("SpawnData", dsl::reference(r::ENTITY_TREE)),
        ]),
    );
    schema.register_type(
        r::ADVANCEMENTS,
        dsl::optional_fields(vec![
            (
                "minecraft:adventure/adventuring_time",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::BIOME), dsl::string()),
                )]),
            ),
            (
                "minecraft:adventure/kill_a_mob",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
            (
                "minecraft:adventure/kill_all_mobs",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
            (
                "minecraft:husbandry/bred_all_animals",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
        ]),
    );
    schema.register_type(r::BIOME, dsl::namespaced_string());
    schema.register_type(r::ENTITY_NAME, dsl::namespaced_string());
    schema.register_type(r::POI_CHUNK, dsl::remainder());
    schema.register_type(r::WORLD_GEN_SETTINGS, dsl::remainder());
    schema.register_type(
        r::ENTITY_CHUNK,
        dsl::optional_fields(vec![(
            "Entities",
            dsl::list(dsl::reference(r::ENTITY_TREE)),
        )]),
    );
    schema.register_type(r::DATA_COMPONENTS, dsl::remainder());
    schema.register_type(
        r::VILLAGER_TRADE,
        dsl::optional_fields(vec![
            ("buy", dsl::reference(r::ITEM_STACK)),
            ("buyB", dsl::reference(r::ITEM_STACK)),
            ("sell", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_type(r::PARTICLE, dsl::string());
    schema.register_type(r::TEXT_COMPONENT, dsl::string());
    schema.register_type(
        r::ENTITY_EQUIPMENT,
        dsl::and(vec![
            dsl::optional_fields(vec![(
                "ArmorItems",
                dsl::list(dsl::reference(r::ITEM_STACK)),
            )]),
            dsl::optional_fields(vec![(
                "HandItems",
                dsl::list(dsl::reference(r::ITEM_STACK)),
            )]),
            dsl::optional_fields(vec![("body_armor_item", dsl::reference(r::ITEM_STACK))]),
            dsl::optional_fields(vec![("saddle", dsl::reference(r::ITEM_STACK))]),
        ]),
    );
}

/// Applies `V1460` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
    register_block_entities(schema);
    register_types(schema);
}
