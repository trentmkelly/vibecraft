//! Port of `net.minecraft.util.datafix.schemas.V99`, the root schema.

use crate::datafix::dynamic::{as_str, ensure_namespaced, get, set};
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::{dsl, ChoiceSet, Tmpl};
use crate::storage::nbt::Tag;

/// `V99.ITEM_TO_BLOCKENTITY`.
const ITEM_TO_BLOCK_ENTITY: &[(&str, &str)] = &[
    ("minecraft:furnace", "Furnace"),
    ("minecraft:lit_furnace", "Furnace"),
    ("minecraft:chest", "Chest"),
    ("minecraft:trapped_chest", "Chest"),
    ("minecraft:ender_chest", "EnderChest"),
    ("minecraft:jukebox", "RecordPlayer"),
    ("minecraft:dispenser", "Trap"),
    ("minecraft:dropper", "Dropper"),
    ("minecraft:sign", "Sign"),
    ("minecraft:mob_spawner", "MobSpawner"),
    ("minecraft:noteblock", "Music"),
    ("minecraft:brewing_stand", "Cauldron"),
    ("minecraft:enhanting_table", "EnchantTable"),
    ("minecraft:command_block", "CommandBlock"),
    ("minecraft:beacon", "Beacon"),
    ("minecraft:skull", "Skull"),
    ("minecraft:daylight_detector", "DLDetector"),
    ("minecraft:hopper", "Hopper"),
    ("minecraft:banner", "Banner"),
    ("minecraft:flower_pot", "FlowerPot"),
    ("minecraft:repeating_command_block", "CommandBlock"),
    ("minecraft:chain_command_block", "CommandBlock"),
    ("minecraft:standing_sign", "Sign"),
    ("minecraft:wall_sign", "Sign"),
    ("minecraft:piston_head", "Piston"),
    ("minecraft:daylight_detector_inverted", "DLDetector"),
    ("minecraft:unpowered_comparator", "Comparator"),
    ("minecraft:powered_comparator", "Comparator"),
    ("minecraft:wall_banner", "Banner"),
    ("minecraft:standing_banner", "Banner"),
    ("minecraft:structure_block", "Structure"),
    ("minecraft:end_portal", "Airportal"),
    ("minecraft:end_gateway", "EndGateway"),
    ("minecraft:shield", "Banner"),
];

/// `V99.ITEM_TO_ENTITY`.
pub const ITEM_TO_ENTITY: &[(&str, &str)] = &[
    ("minecraft:armor_stand", "ArmorStand"),
    ("minecraft:painting", "Painting"),
];

fn lookup(table: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(item, _)| *item == key)
        .map(|(_, value)| *value)
}

/// `V99.addNames`: the item stack pre-read hook that stamps the block entity /
/// entity id an item implies onto its `BlockEntityTag` / `EntityTag`.
pub fn add_names_with(
    input: &mut Tag,
    item_to_block_entity: &[(&str, &'static str)],
    item_to_entity: &[(&str, &'static str)],
) {
    // `input.get("id")` refers to the stack's own id as it was before the hook.
    let stack_id = get(input, "id").cloned();
    let Some(Tag::Compound(_)) = get(input, "tag") else {
        return;
    };
    let item_id = stack_id
        .as_ref()
        .and_then(as_str)
        .map(ensure_namespaced)
        .unwrap_or_else(|| "minecraft:air".to_string());
    let stack_id_or_empty = stack_id
        .as_ref()
        .and_then(as_str)
        .map(ensure_namespaced)
        .unwrap_or_default();
    let Some(item_tag) = crate::datafix::dynamic::get_mut(input, "tag") else {
        return;
    };
    if let Some(block_entity) = crate::datafix::dynamic::get_mut(item_tag, "BlockEntityTag") {
        if item_id != "minecraft:air" {
            if let Some(expected) = lookup(item_to_block_entity, &item_id) {
                set(block_entity, "id", Tag::String(expected.to_string()));
            }
        }
    }
    if let Some(entity) = crate::datafix::dynamic::get_mut(item_tag, "EntityTag") {
        if get(entity, "id").is_none() {
            if let Some(expected) = lookup(item_to_entity, &stack_id_or_empty) {
                set(entity, "id", Tag::String(expected.to_string()));
            }
        }
    }
}

/// `V99.ADD_NAMES`.
pub fn add_names(input: &mut Tag) {
    add_names_with(input, ITEM_TO_BLOCK_ENTITY, ITEM_TO_ENTITY);
}

fn item_stack() -> Tmpl {
    dsl::reference(r::ITEM_STACK)
}

fn block_name() -> Tmpl {
    dsl::reference(r::BLOCK_NAME)
}

fn text_component() -> Tmpl {
    dsl::reference(r::TEXT_COMPONENT)
}

fn item_list() -> Tmpl {
    dsl::list(item_stack())
}

fn register_throwable_projectile(schema: &mut Schema, name: &str) {
    schema.register_entity(name, dsl::optional_fields(vec![("inTile", block_name())]));
}

fn register_minecart(schema: &mut Schema, name: &str) {
    schema.register_entity(
        name,
        dsl::optional_fields(vec![("DisplayTile", block_name())]),
    );
}

fn register_inventory(schema: &mut Schema, name: &str) {
    schema.register_block_entity(name, dsl::optional_fields(vec![("Items", item_list())]));
}

fn minecart_with_items() -> Tmpl {
    dsl::optional_fields(vec![("DisplayTile", block_name()), ("Items", item_list())])
}

/// `V99.registerEntities`.
#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
pub fn register_entities(schema: &mut Schema) {
    schema.register_entity("Item", dsl::optional_fields(vec![("Item", item_stack())]));
    schema.register_simple_entity("XPOrb");
    register_throwable_projectile(schema, "ThrownEgg");
    schema.register_simple_entity("LeashKnot");
    schema.register_simple_entity("Painting");
    for arrow in ["Arrow", "TippedArrow", "SpectralArrow"] {
        schema.register_entity(arrow, dsl::optional_fields(vec![("inTile", block_name())]));
    }
    register_throwable_projectile(schema, "Snowball");
    register_throwable_projectile(schema, "Fireball");
    register_throwable_projectile(schema, "SmallFireball");
    register_throwable_projectile(schema, "ThrownEnderpearl");
    schema.register_simple_entity("EyeOfEnderSignal");
    schema.register_entity(
        "ThrownPotion",
        dsl::optional_fields(vec![("inTile", block_name()), ("Potion", item_stack())]),
    );
    register_throwable_projectile(schema, "ThrownExpBottle");
    schema.register_entity(
        "ItemFrame",
        dsl::optional_fields(vec![("Item", item_stack())]),
    );
    register_throwable_projectile(schema, "WitherSkull");
    schema.register_simple_entity("PrimedTnt");
    schema.register_entity(
        "FallingSand",
        dsl::optional_fields(vec![
            ("Block", block_name()),
            ("TileEntityData", dsl::reference(r::BLOCK_ENTITY)),
        ]),
    );
    schema.register_entity(
        "FireworksRocketEntity",
        dsl::optional_fields(vec![("FireworksItem", item_stack())]),
    );
    schema.register_simple_entity("Boat");
    schema.register_entity("Minecart", minecart_with_items());
    register_minecart(schema, "MinecartRideable");
    schema.register_entity("MinecartChest", minecart_with_items());
    register_minecart(schema, "MinecartFurnace");
    register_minecart(schema, "MinecartTNT");
    schema.register_entity(
        "MinecartSpawner",
        dsl::optional_fields_with_rest(
            vec![("DisplayTile", block_name())],
            dsl::reference(r::UNTAGGED_SPAWNER),
        ),
    );
    schema.register_entity("MinecartHopper", minecart_with_items());
    schema.register_entity(
        "MinecartCommandBlock",
        dsl::optional_fields(vec![
            ("DisplayTile", block_name()),
            ("LastOutput", text_component()),
        ]),
    );
    for simple in [
        "ArmorStand",
        "Creeper",
        "Skeleton",
        "Spider",
        "Giant",
        "Zombie",
        "Slime",
        "Ghast",
        "PigZombie",
    ] {
        schema.register_simple_entity(simple);
    }
    schema.register_entity(
        "Enderman",
        dsl::optional_fields(vec![("carried", block_name())]),
    );
    for simple in [
        "CaveSpider",
        "Silverfish",
        "Blaze",
        "LavaSlime",
        "EnderDragon",
        "WitherBoss",
        "Bat",
        "Witch",
        "Endermite",
        "Guardian",
        "Pig",
        "Sheep",
        "Cow",
        "Chicken",
        "Squid",
        "Wolf",
        "MushroomCow",
        "SnowMan",
        "Ozelot",
        "VillagerGolem",
    ] {
        schema.register_simple_entity(simple);
    }
    schema.register_entity(
        "EntityHorse",
        dsl::optional_fields(vec![
            ("Items", item_list()),
            ("ArmorItem", item_stack()),
            ("SaddleItem", item_stack()),
        ]),
    );
    schema.register_simple_entity("Rabbit");
    schema.register_entity(
        "Villager",
        dsl::optional_fields(vec![
            ("Inventory", item_list()),
            (
                "Offers",
                dsl::optional_fields(vec![(
                    "Recipes",
                    dsl::list(dsl::reference(r::VILLAGER_TRADE)),
                )]),
            ),
        ]),
    );
    schema.register_simple_entity("EnderCrystal");
    schema.register_entity(
        "AreaEffectCloud",
        dsl::optional_fields(vec![("Particle", dsl::reference(r::PARTICLE))]),
    );
    schema.register_simple_entity("ShulkerBullet");
    schema.register_simple_entity("DragonFireball");
    schema.register_simple_entity("Shulker");
}

/// `V99.registerBlockEntities`.
pub fn register_block_entities(schema: &mut Schema) {
    register_inventory(schema, "Furnace");
    register_inventory(schema, "Chest");
    schema.register_simple_block_entity("EnderChest");
    schema.register_block_entity(
        "RecordPlayer",
        dsl::optional_fields(vec![("RecordItem", item_stack())]),
    );
    register_inventory(schema, "Trap");
    register_inventory(schema, "Dropper");
    schema.register_block_entity("Sign", sign());
    schema.register_block_entity("MobSpawner", dsl::reference(r::UNTAGGED_SPAWNER));
    schema.register_simple_block_entity("Music");
    schema.register_simple_block_entity("Piston");
    register_inventory(schema, "Cauldron");
    schema.register_simple_block_entity("EnchantTable");
    schema.register_simple_block_entity("Airportal");
    schema.register_block_entity(
        "Control",
        dsl::optional_fields(vec![("LastOutput", text_component())]),
    );
    schema.register_simple_block_entity("Beacon");
    schema.register_block_entity(
        "Skull",
        dsl::optional_fields(vec![("custom_name", text_component())]),
    );
    schema.register_simple_block_entity("DLDetector");
    register_inventory(schema, "Hopper");
    schema.register_simple_block_entity("Comparator");
    schema.register_block_entity(
        "FlowerPot",
        dsl::optional_fields(vec![(
            "Item",
            dsl::or(dsl::int(), dsl::reference(r::ITEM_NAME)),
        )]),
    );
    schema.register_block_entity(
        "Banner",
        dsl::optional_fields(vec![("CustomName", text_component())]),
    );
    schema.register_simple_block_entity("Structure");
    schema.register_simple_block_entity("EndGateway");
}

/// `V99.sign`.
pub fn sign() -> Tmpl {
    dsl::optional_fields(vec![
        ("Text1", text_component()),
        ("Text2", text_component()),
        ("Text3", text_component()),
        ("Text4", text_component()),
        ("FilteredText1", text_component()),
        ("FilteredText2", text_component()),
        ("FilteredText3", text_component()),
        ("FilteredText4", text_component()),
    ])
}

/// `V99.itemStackTag`.
pub fn item_stack_tag() -> Tmpl {
    dsl::optional_fields(vec![
        ("EntityTag", dsl::reference(r::ENTITY_TREE)),
        ("BlockEntityTag", dsl::reference(r::BLOCK_ENTITY)),
        ("CanDestroy", dsl::list(block_name())),
        ("CanPlaceOn", dsl::list(block_name())),
        ("Items", item_list()),
        ("ChargedProjectiles", item_list()),
        ("pages", dsl::list(text_component())),
        ("filtered_pages", dsl::compound_list(text_component())),
        (
            "display",
            dsl::optional_fields(vec![
                ("Name", text_component()),
                ("Lore", dsl::list(text_component())),
            ]),
        ),
    ])
}

fn saved_data_text_list(field: &'static str) -> Tmpl {
    dsl::optional_fields(vec![(
        "data",
        dsl::optional_fields(vec![(
            field,
            dsl::list(dsl::optional_fields(vec![("Name", text_component())])),
        )]),
    )])
}

/// `V99.registerTypes`.
#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
pub fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::LEVEL,
        dsl::optional_fields_with_rest(
            vec![(
                "CustomBossEvents",
                dsl::compound_list(dsl::optional_fields(vec![("Name", text_component())])),
            )],
            dsl::reference(r::LIGHTWEIGHT_LEVEL),
        ),
    );
    schema.register_type(r::LIGHTWEIGHT_LEVEL, dsl::remainder());
    schema.register_type(
        r::PLAYER,
        dsl::optional_fields(vec![
            ("Inventory", item_list()),
            ("EnderItems", item_list()),
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
                    dsl::list(dsl::fields(vec![("i", block_name())])),
                ),
            ]),
        )]),
    );
    schema.register_type(
        r::BLOCK_ENTITY,
        dsl::optional_fields_with_rest(
            vec![("components", dsl::reference(r::DATA_COMPONENTS))],
            dsl::choice(ChoiceSet::BlockEntities),
        ),
    );
    schema.register_type(
        r::ENTITY_TREE,
        dsl::optional_fields_with_rest(
            vec![("Riding", dsl::reference(r::ENTITY_TREE))],
            dsl::reference(r::ENTITY),
        ),
    );
    schema.register_type(r::ENTITY_NAME, dsl::namespaced_string());
    schema.register_type(
        r::ENTITY,
        dsl::and(vec![
            dsl::reference(r::ENTITY_EQUIPMENT),
            dsl::optional_fields_with_rest(
                vec![("CustomName", dsl::string())],
                dsl::choice(ChoiceSet::Entities),
            ),
        ]),
    );
    schema.register_type(
        r::ITEM_STACK,
        dsl::hook(
            dsl::optional_fields(vec![
                ("id", dsl::or(dsl::int(), dsl::reference(r::ITEM_NAME))),
                ("tag", item_stack_tag()),
            ]),
            add_names,
        ),
    );
    schema.register_type(r::OPTIONS, dsl::remainder());
    schema.register_type(r::BLOCK_NAME, dsl::or(dsl::int(), dsl::namespaced_string()));
    schema.register_type(r::ITEM_NAME, dsl::namespaced_string());
    schema.register_type(r::STATS, dsl::remainder());
    for remainder in [
        r::SAVED_DATA_COMMAND_STORAGE,
        r::SAVED_DATA_ENDER_DRAGON_FIGHT,
        r::SAVED_DATA_GAME_RULES,
        r::SAVED_DATA_TICKETS,
        r::SAVED_DATA_MAP_INDEX,
        r::SAVED_DATA_RAIDS,
        r::SAVED_DATA_RANDOM_SEQUENCES,
        r::SAVED_DATA_SCHEDULED_EVENTS,
        r::SAVED_DATA_STOPWATCHES,
        r::SAVED_DATA_WANDERING_TRADER,
        r::SAVED_DATA_WEATHER,
        r::SAVED_DATA_WORLD_BORDER,
        r::SAVED_DATA_WORLD_CLOCKS,
    ] {
        schema.register_type(remainder, dsl::remainder());
    }
    schema.register_type(
        r::SAVED_DATA_CUSTOM_BOSS_EVENTS,
        dsl::optional_fields(vec![(
            "data",
            dsl::compound_list(dsl::optional_fields(vec![("Name", text_component())])),
        )]),
    );
    schema.register_type(r::SAVED_DATA_MAP_DATA, saved_data_text_list("banners"));
    schema.register_type(
        r::SAVED_DATA_SCOREBOARD,
        dsl::optional_fields(vec![(
            "data",
            dsl::optional_fields(vec![
                ("Objectives", dsl::list(dsl::reference(r::OBJECTIVE))),
                ("Teams", dsl::list(dsl::reference(r::TEAM))),
                (
                    "PlayerScores",
                    dsl::list(dsl::optional_fields(vec![("display", text_component())])),
                ),
            ]),
        )]),
    );
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
    schema.register_type(
        r::SAVED_DATA_WORLD_GEN_SETTINGS,
        dsl::fields(vec![("data", dsl::reference(r::WORLD_GEN_SETTINGS))]),
    );
    schema.register_type(r::DEBUG_PROFILE, dsl::remainder());
    schema.register_type(r::STRUCTURE_FEATURE, dsl::remainder());
    schema.register_type(r::OBJECTIVE, dsl::remainder());
    schema.register_type(
        r::TEAM,
        dsl::optional_fields(vec![
            ("MemberNamePrefix", text_component()),
            ("MemberNameSuffix", text_component()),
            ("DisplayName", text_component()),
        ]),
    );
    schema.register_type(r::UNTAGGED_SPAWNER, dsl::remainder());
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
            ("buy", item_stack()),
            ("buyB", item_stack()),
            ("sell", item_stack()),
        ]),
    );
    schema.register_type(r::PARTICLE, dsl::string());
    schema.register_type(r::TEXT_COMPONENT, dsl::string());
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
    schema.register_type(r::BLOCK_STATE, dsl::remainder());
    schema.register_type(r::FLAT_BLOCK_STATE, dsl::remainder());
    schema.register_type(
        r::ENTITY_EQUIPMENT,
        dsl::optional_fields(vec![("Equipment", item_list())]),
    );
}

/// The schema factory for `V99::new`.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
    register_block_entities(schema);
    register_types(schema);
}
