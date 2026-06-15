#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityMetadataField {
    pub class_name: &'static str,
    pub index: u8,
    pub accessor: &'static str,
    pub serializer: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityMetadataClass {
    pub class_name: &'static str,
    pub parent: Option<&'static str>,
    pub fields: &'static [EntityMetadataField],
}

pub const ENTITY_METADATA: &[EntityMetadataField] = &[
    field("Entity", 0, "DATA_SHARED_FLAGS_ID", "BYTE"),
    field("Entity", 1, "DATA_AIR_SUPPLY_ID", "INT"),
    field("Entity", 2, "DATA_CUSTOM_NAME", "OPTIONAL_COMPONENT"),
    field("Entity", 3, "DATA_CUSTOM_NAME_VISIBLE", "BOOLEAN"),
    field("Entity", 4, "DATA_SILENT", "BOOLEAN"),
    field("Entity", 5, "DATA_NO_GRAVITY", "BOOLEAN"),
    field("Entity", 6, "DATA_POSE", "POSE"),
    field("Entity", 7, "DATA_TICKS_FROZEN", "INT"),
];

pub const LIVING_ENTITY_METADATA: &[EntityMetadataField] = &[
    field("LivingEntity", 8, "DATA_LIVING_ENTITY_FLAGS", "BYTE"),
    field("LivingEntity", 9, "DATA_HEALTH_ID", "FLOAT"),
    field("LivingEntity", 10, "DATA_EFFECT_PARTICLES", "PARTICLES"),
    field("LivingEntity", 11, "DATA_EFFECT_AMBIENCE_ID", "BOOLEAN"),
    field("LivingEntity", 12, "DATA_ARROW_COUNT_ID", "INT"),
    field("LivingEntity", 13, "DATA_STINGER_COUNT_ID", "INT"),
    field("LivingEntity", 14, "SLEEPING_POS_ID", "OPTIONAL_BLOCK_POS"),
];

pub const MOB_METADATA: &[EntityMetadataField] = &[field("Mob", 15, "DATA_MOB_FLAGS_ID", "BYTE")];

pub const PLAYER_METADATA: &[EntityMetadataField] = &[
    field("Player", 15, "DATA_PLAYER_ABSORPTION_ID", "FLOAT"),
    field("Player", 16, "DATA_SCORE_ID", "INT"),
    field(
        "Player",
        17,
        "DATA_SHOULDER_PARROT_LEFT",
        "OPTIONAL_UNSIGNED_INT",
    ),
    field(
        "Player",
        18,
        "DATA_SHOULDER_PARROT_RIGHT",
        "OPTIONAL_UNSIGNED_INT",
    ),
];

pub const AGEABLE_MOB_METADATA: &[EntityMetadataField] = &[
    field("AgeableMob", 16, "DATA_BABY_ID", "BOOLEAN"),
    field("AgeableMob", 17, "AGE_LOCKED", "BOOLEAN"),
];

pub const TAMABLE_ANIMAL_METADATA: &[EntityMetadataField] = &[
    field("TamableAnimal", 18, "DATA_FLAGS_ID", "BYTE"),
    field(
        "TamableAnimal",
        19,
        "DATA_OWNERUUID_ID",
        "OPTIONAL_LIVING_ENTITY_REFERENCE",
    ),
];

pub const PIG_METADATA: &[EntityMetadataField] = &[
    field("Pig", 18, "DATA_BOOST_TIME", "INT"),
    field("Pig", 19, "DATA_VARIANT_ID", "PIG_VARIANT"),
    field("Pig", 20, "DATA_SOUND_VARIANT_ID", "PIG_SOUND_VARIANT"),
];

pub const ALLAY_METADATA: &[EntityMetadataField] = &[
    field("Allay", 16, "DATA_DANCING", "BOOLEAN"),
    field("Allay", 17, "DATA_CAN_DUPLICATE", "BOOLEAN"),
];

pub const ARMADILLO_METADATA: &[EntityMetadataField] =
    &[field("Armadillo", 18, "ARMADILLO_STATE", "ARMADILLO_STATE")];

pub const AXOLOTL_METADATA: &[EntityMetadataField] = &[
    field("Axolotl", 18, "DATA_VARIANT", "INT"),
    field("Axolotl", 19, "DATA_PLAYING_DEAD", "BOOLEAN"),
    field("Axolotl", 20, "FROM_BUCKET", "BOOLEAN"),
];

pub const BEE_METADATA: &[EntityMetadataField] = &[
    field("Bee", 18, "DATA_FLAGS_ID", "BYTE"),
    field("Bee", 19, "DATA_ANGER_END_TIME", "LONG"),
];

pub const CAT_METADATA: &[EntityMetadataField] = &[
    field("Cat", 20, "DATA_VARIANT_ID", "CAT_VARIANT"),
    field("Cat", 21, "IS_LYING", "BOOLEAN"),
    field("Cat", 22, "RELAX_STATE_ONE", "BOOLEAN"),
    field("Cat", 23, "DATA_COLLAR_COLOR", "INT"),
    field("Cat", 24, "DATA_SOUND_VARIANT_ID", "CAT_SOUND_VARIANT"),
];

pub const CHICKEN_METADATA: &[EntityMetadataField] = &[
    field("Chicken", 18, "DATA_VARIANT_ID", "CHICKEN_VARIANT"),
    field(
        "Chicken",
        19,
        "DATA_SOUND_VARIANT_ID",
        "CHICKEN_SOUND_VARIANT",
    ),
];

pub const COW_METADATA: &[EntityMetadataField] = &[
    field("Cow", 18, "DATA_VARIANT_ID", "COW_VARIANT"),
    field("Cow", 19, "DATA_SOUND_VARIANT_ID", "COW_SOUND_VARIANT"),
];

pub const FROG_METADATA: &[EntityMetadataField] = &[
    field("Frog", 18, "DATA_VARIANT_ID", "FROG_VARIANT"),
    field("Frog", 19, "DATA_TONGUE_TARGET_ID", "OPTIONAL_UNSIGNED_INT"),
];

pub const SNIFFER_METADATA: &[EntityMetadataField] = &[
    field("Sniffer", 18, "DATA_STATE", "SNIFFER_STATE"),
    field("Sniffer", 19, "DATA_DROP_SEED_AT_TICK", "INT"),
];

pub const WOLF_METADATA: &[EntityMetadataField] = &[
    field("Wolf", 20, "DATA_INTERESTED_ID", "BOOLEAN"),
    field("Wolf", 21, "DATA_COLLAR_COLOR", "INT"),
    field("Wolf", 22, "DATA_ANGER_END_TIME", "LONG"),
    field("Wolf", 23, "DATA_VARIANT_ID", "WOLF_VARIANT"),
    field("Wolf", 24, "DATA_SOUND_VARIANT_ID", "WOLF_SOUND_VARIANT"),
];

pub const ZOMBIE_METADATA: &[EntityMetadataField] = &[
    field("Zombie", 16, "DATA_BABY_ID", "BOOLEAN"),
    field("Zombie", 17, "DATA_SPECIAL_TYPE_ID", "INT"),
    field("Zombie", 18, "DATA_DROWNED_CONVERSION_ID", "BOOLEAN"),
];

pub const BAT_METADATA: &[EntityMetadataField] = &[field("Bat", 16, "DATA_ID_FLAGS", "BYTE")];

pub const CREEPER_METADATA: &[EntityMetadataField] = &[
    field("Creeper", 16, "DATA_SWELL_DIR", "INT"),
    field("Creeper", 17, "DATA_IS_POWERED", "BOOLEAN"),
    field("Creeper", 18, "DATA_IS_IGNITED", "BOOLEAN"),
];

pub const SPIDER_METADATA: &[EntityMetadataField] = &[field("Spider", 16, "DATA_FLAGS_ID", "BYTE")];

pub const BLAZE_METADATA: &[EntityMetadataField] = &[field("Blaze", 16, "DATA_FLAGS_ID", "BYTE")];

pub const ENDER_MAN_METADATA: &[EntityMetadataField] = &[
    field("EnderMan", 16, "DATA_CARRY_STATE", "OPTIONAL_BLOCK_STATE"),
    field("EnderMan", 17, "DATA_CREEPY", "BOOLEAN"),
    field("EnderMan", 18, "DATA_STARED_AT", "BOOLEAN"),
];

pub const GUARDIAN_METADATA: &[EntityMetadataField] = &[
    field("Guardian", 16, "DATA_ID_MOVING", "BOOLEAN"),
    field("Guardian", 17, "DATA_ID_ATTACK_TARGET", "INT"),
];

pub const SLIME_METADATA: &[EntityMetadataField] = &[field("Slime", 16, "ID_SIZE", "INT")];

pub const PHANTOM_METADATA: &[EntityMetadataField] = &[field("Phantom", 16, "ID_SIZE", "INT")];

pub const GHAST_METADATA: &[EntityMetadataField] =
    &[field("Ghast", 16, "DATA_IS_CHARGING", "BOOLEAN")];

pub const WARDEN_METADATA: &[EntityMetadataField] =
    &[field("Warden", 16, "CLIENT_ANGER_LEVEL", "INT")];

pub const VEX_METADATA: &[EntityMetadataField] = &[field("Vex", 16, "DATA_FLAGS_ID", "BYTE")];

pub const ZOGLIN_METADATA: &[EntityMetadataField] =
    &[field("Zoglin", 16, "DATA_BABY_ID", "BOOLEAN")];

pub const CREAKING_METADATA: &[EntityMetadataField] = &[
    field("Creaking", 16, "CAN_MOVE", "BOOLEAN"),
    field("Creaking", 17, "IS_ACTIVE", "BOOLEAN"),
    field("Creaking", 18, "IS_TEARING_DOWN", "BOOLEAN"),
    field("Creaking", 19, "HOME_POS", "OPTIONAL_BLOCK_POS"),
];

pub const ABSTRACT_PIGLIN_METADATA: &[EntityMetadataField] = &[field(
    "AbstractPiglin",
    16,
    "DATA_IMMUNE_TO_ZOMBIFICATION",
    "BOOLEAN",
)];

pub const PIGLIN_METADATA: &[EntityMetadataField] = &[
    field("Piglin", 17, "DATA_BABY_ID", "BOOLEAN"),
    field("Piglin", 18, "DATA_IS_CHARGING_CROSSBOW", "BOOLEAN"),
    field("Piglin", 19, "DATA_IS_DANCING", "BOOLEAN"),
];

pub const SKELETON_METADATA: &[EntityMetadataField] =
    &[field("Skeleton", 16, "DATA_STRAY_CONVERSION_ID", "BOOLEAN")];

pub const BOGGED_METADATA: &[EntityMetadataField] =
    &[field("Bogged", 16, "DATA_SHEARED", "BOOLEAN")];

pub const ZOMBIE_VILLAGER_METADATA: &[EntityMetadataField] = &[
    field("ZombieVillager", 19, "DATA_CONVERTING_ID", "BOOLEAN"),
    field("ZombieVillager", 20, "DATA_VILLAGER_DATA", "VILLAGER_DATA"),
    field(
        "ZombieVillager",
        21,
        "DATA_VILLAGER_DATA_FINALIZED",
        "BOOLEAN",
    ),
];

pub const RAIDER_METADATA: &[EntityMetadataField] =
    &[field("Raider", 16, "IS_CELEBRATING", "BOOLEAN")];

pub const WITCH_METADATA: &[EntityMetadataField] =
    &[field("Witch", 17, "DATA_USING_ITEM", "BOOLEAN")];

pub const PILLAGER_METADATA: &[EntityMetadataField] =
    &[field("Pillager", 17, "IS_CHARGING_CROSSBOW", "BOOLEAN")];

pub const SPELLCASTER_ILLAGER_METADATA: &[EntityMetadataField] = &[field(
    "SpellcasterIllager",
    17,
    "DATA_SPELL_CASTING_ID",
    "BYTE",
)];

pub const ABSTRACT_GOLEM_METADATA: &[EntityMetadataField] = &[];

pub const IRON_GOLEM_METADATA: &[EntityMetadataField] =
    &[field("IronGolem", 16, "DATA_FLAGS_ID", "BYTE")];

pub const SNOW_GOLEM_METADATA: &[EntityMetadataField] =
    &[field("SnowGolem", 16, "DATA_PUMPKIN_ID", "BYTE")];

pub const COPPER_GOLEM_METADATA: &[EntityMetadataField] = &[
    field(
        "CopperGolem",
        16,
        "DATA_WEATHER_STATE",
        "WEATHERING_COPPER_STATE",
    ),
    field(
        "CopperGolem",
        17,
        "COPPER_GOLEM_STATE",
        "COPPER_GOLEM_STATE",
    ),
];

pub const ABSTRACT_VILLAGER_METADATA: &[EntityMetadataField] =
    &[field("AbstractVillager", 18, "DATA_UNHAPPY_COUNTER", "INT")];

pub const VILLAGER_METADATA: &[EntityMetadataField] = &[
    field("Villager", 19, "DATA_VILLAGER_DATA", "VILLAGER_DATA"),
    field("Villager", 20, "DATA_VILLAGER_DATA_FINALIZED", "BOOLEAN"),
];

pub const RABBIT_METADATA: &[EntityMetadataField] = &[field("Rabbit", 18, "DATA_TYPE_ID", "INT")];

pub const SHEEP_METADATA: &[EntityMetadataField] = &[field("Sheep", 18, "DATA_WOOL_ID", "BYTE")];

pub const TURTLE_METADATA: &[EntityMetadataField] = &[
    field("Turtle", 18, "HAS_EGG", "BOOLEAN"),
    field("Turtle", 19, "LAYING_EGG", "BOOLEAN"),
];

pub const PANDA_METADATA: &[EntityMetadataField] = &[
    field("Panda", 18, "UNHAPPY_COUNTER", "INT"),
    field("Panda", 19, "SNEEZE_COUNTER", "INT"),
    field("Panda", 20, "EAT_COUNTER", "INT"),
    field("Panda", 21, "MAIN_GENE_ID", "BYTE"),
    field("Panda", 22, "HIDDEN_GENE_ID", "BYTE"),
    field("Panda", 23, "DATA_ID_FLAGS", "BYTE"),
];

pub const FOX_METADATA: &[EntityMetadataField] = &[
    field("Fox", 18, "DATA_TYPE_ID", "INT"),
    field("Fox", 19, "DATA_FLAGS_ID", "BYTE"),
    field(
        "Fox",
        20,
        "DATA_TRUSTED_ID_0",
        "OPTIONAL_LIVING_ENTITY_REFERENCE",
    ),
    field(
        "Fox",
        21,
        "DATA_TRUSTED_ID_1",
        "OPTIONAL_LIVING_ENTITY_REFERENCE",
    ),
];

pub const POLAR_BEAR_METADATA: &[EntityMetadataField] =
    &[field("PolarBear", 18, "DATA_STANDING_ID", "BOOLEAN")];

pub const OCELOT_METADATA: &[EntityMetadataField] =
    &[field("Ocelot", 18, "DATA_TRUSTING", "BOOLEAN")];

pub const PARROT_METADATA: &[EntityMetadataField] =
    &[field("Parrot", 20, "DATA_VARIANT_ID", "INT")];

pub const ABSTRACT_HORSE_METADATA: &[EntityMetadataField] =
    &[field("AbstractHorse", 18, "DATA_ID_FLAGS", "BYTE")];

pub const ABSTRACT_CHESTED_HORSE_METADATA: &[EntityMetadataField] = &[field(
    "AbstractChestedHorse",
    19,
    "DATA_ID_CHEST",
    "BOOLEAN",
)];

pub const CAMEL_METADATA: &[EntityMetadataField] = &[
    field("Camel", 19, "DASH", "BOOLEAN"),
    field("Camel", 20, "LAST_POSE_CHANGE_TICK", "LONG"),
];

pub const ABSTRACT_FISH_METADATA: &[EntityMetadataField] =
    &[field("AbstractFish", 16, "FROM_BUCKET", "BOOLEAN")];

pub const PUFFERFISH_METADATA: &[EntityMetadataField] =
    &[field("Pufferfish", 17, "PUFF_STATE", "INT")];

pub const SALMON_METADATA: &[EntityMetadataField] = &[field("Salmon", 17, "DATA_TYPE", "INT")];

pub const TROPICAL_FISH_METADATA: &[EntityMetadataField] =
    &[field("TropicalFish", 17, "DATA_ID_TYPE_VARIANT", "INT")];

pub const TADPOLE_METADATA: &[EntityMetadataField] =
    &[field("Tadpole", 17, "AGE_LOCKED", "BOOLEAN")];

pub const GLOW_SQUID_METADATA: &[EntityMetadataField] =
    &[field("GlowSquid", 18, "DATA_DARK_TICKS_REMAINING", "INT")];

pub const DOLPHIN_METADATA: &[EntityMetadataField] = &[
    field("Dolphin", 18, "GOT_FISH", "BOOLEAN"),
    field("Dolphin", 19, "MOISTNESS_LEVEL", "INT"),
];

pub const ABSTRACT_NAUTILUS_METADATA: &[EntityMetadataField] =
    &[field("AbstractNautilus", 20, "DASH", "BOOLEAN")];

pub const ZOMBIE_NAUTILUS_METADATA: &[EntityMetadataField] = &[field(
    "ZombieNautilus",
    21,
    "DATA_VARIANT_ID",
    "ZOMBIE_NAUTILUS_VARIANT",
)];

pub const HORSE_METADATA: &[EntityMetadataField] =
    &[field("Horse", 19, "DATA_ID_TYPE_VARIANT", "INT")];

pub const LLAMA_METADATA: &[EntityMetadataField] = &[
    field("Llama", 20, "DATA_STRENGTH_ID", "INT"),
    field("Llama", 21, "DATA_VARIANT_ID", "INT"),
];

pub const GOAT_METADATA: &[EntityMetadataField] = &[
    field("Goat", 18, "DATA_IS_SCREAMING_GOAT", "BOOLEAN"),
    field("Goat", 19, "DATA_HAS_LEFT_HORN", "BOOLEAN"),
    field("Goat", 20, "DATA_HAS_RIGHT_HORN", "BOOLEAN"),
];

pub const HOGLIN_METADATA: &[EntityMetadataField] = &[field(
    "Hoglin",
    18,
    "DATA_IMMUNE_TO_ZOMBIFICATION",
    "BOOLEAN",
)];

pub const STRIDER_METADATA: &[EntityMetadataField] = &[
    field("Strider", 18, "DATA_BOOST_TIME", "INT"),
    field("Strider", 19, "DATA_SUFFOCATING", "BOOLEAN"),
];

pub const HAPPY_GHAST_METADATA: &[EntityMetadataField] = &[
    field("HappyGhast", 18, "IS_LEASH_HOLDER", "BOOLEAN"),
    field("HappyGhast", 19, "STAYS_STILL", "BOOLEAN"),
];

pub const MUSHROOM_COW_METADATA: &[EntityMetadataField] =
    &[field("MushroomCow", 18, "DATA_TYPE", "INT")];

pub const SHULKER_METADATA: &[EntityMetadataField] = &[
    field("Shulker", 16, "DATA_ATTACH_FACE_ID", "DIRECTION"),
    field("Shulker", 17, "DATA_PEEK_ID", "BYTE"),
    field("Shulker", 18, "DATA_COLOR_ID", "BYTE"),
];

pub const WITHER_BOSS_METADATA: &[EntityMetadataField] = &[
    field("WitherBoss", 16, "DATA_TARGET_A", "INT"),
    field("WitherBoss", 17, "DATA_TARGET_B", "INT"),
    field("WitherBoss", 18, "DATA_TARGET_C", "INT"),
    field("WitherBoss", 19, "DATA_ID_INV", "INT"),
];

pub const ENDER_DRAGON_METADATA: &[EntityMetadataField] =
    &[field("EnderDragon", 16, "DATA_PHASE", "INT")];

pub const END_CRYSTAL_METADATA: &[EntityMetadataField] = &[
    field("EndCrystal", 8, "DATA_BEAM_TARGET", "OPTIONAL_BLOCK_POS"),
    field("EndCrystal", 9, "DATA_SHOW_BOTTOM", "BOOLEAN"),
];

pub const EXPERIENCE_ORB_METADATA: &[EntityMetadataField] =
    &[field("ExperienceOrb", 8, "DATA_VALUE", "INT")];

pub const OMINOUS_ITEM_SPAWNER_METADATA: &[EntityMetadataField] =
    &[field("OminousItemSpawner", 8, "DATA_ITEM", "ITEM_STACK")];

pub const AVATAR_METADATA: &[EntityMetadataField] = &[
    field("Avatar", 15, "DATA_PLAYER_MAIN_HAND", "HUMANOID_ARM"),
    field("Avatar", 16, "DATA_PLAYER_MODE_CUSTOMISATION", "BYTE"),
];

pub const MANNEQUIN_METADATA: &[EntityMetadataField] = &[
    field("Mannequin", 17, "DATA_PROFILE", "RESOLVABLE_PROFILE"),
    field("Mannequin", 18, "DATA_IMMOVABLE", "BOOLEAN"),
    field("Mannequin", 19, "DATA_DESCRIPTION", "OPTIONAL_COMPONENT"),
];

pub const VEHICLE_ENTITY_METADATA: &[EntityMetadataField] = &[
    field("VehicleEntity", 8, "DATA_ID_HURT", "INT"),
    field("VehicleEntity", 9, "DATA_ID_HURTDIR", "INT"),
    field("VehicleEntity", 10, "DATA_ID_DAMAGE", "FLOAT"),
];

pub const ABSTRACT_BOAT_METADATA: &[EntityMetadataField] = &[
    field("AbstractBoat", 11, "DATA_ID_PADDLE_LEFT", "BOOLEAN"),
    field("AbstractBoat", 12, "DATA_ID_PADDLE_RIGHT", "BOOLEAN"),
    field("AbstractBoat", 13, "DATA_ID_BUBBLE_TIME", "INT"),
];

pub const ABSTRACT_MINECART_METADATA: &[EntityMetadataField] = &[
    field(
        "AbstractMinecart",
        11,
        "DATA_ID_CUSTOM_DISPLAY_BLOCK",
        "OPTIONAL_BLOCK_STATE",
    ),
    field("AbstractMinecart", 12, "DATA_ID_DISPLAY_OFFSET", "INT"),
];

pub const MINECART_COMMAND_BLOCK_METADATA: &[EntityMetadataField] = &[
    field("MinecartCommandBlock", 13, "DATA_ID_COMMAND_NAME", "STRING"),
    field(
        "MinecartCommandBlock",
        14,
        "DATA_ID_LAST_OUTPUT",
        "COMPONENT",
    ),
];

pub const MINECART_FURNACE_METADATA: &[EntityMetadataField] =
    &[field("MinecartFurnace", 13, "DATA_ID_FUEL", "BOOLEAN")];

pub const AREA_EFFECT_CLOUD_METADATA: &[EntityMetadataField] = &[
    field("AreaEffectCloud", 8, "DATA_RADIUS", "FLOAT"),
    field("AreaEffectCloud", 9, "DATA_WAITING", "BOOLEAN"),
    field("AreaEffectCloud", 10, "DATA_PARTICLE", "PARTICLE"),
];

pub const DISPLAY_METADATA: &[EntityMetadataField] = &[
    field(
        "Display",
        8,
        "DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID",
        "INT",
    ),
    field(
        "Display",
        9,
        "DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID",
        "INT",
    ),
    field(
        "Display",
        10,
        "DATA_POS_ROT_INTERPOLATION_DURATION_ID",
        "INT",
    ),
    field("Display", 11, "DATA_TRANSLATION_ID", "VECTOR3"),
    field("Display", 12, "DATA_SCALE_ID", "VECTOR3"),
    field("Display", 13, "DATA_LEFT_ROTATION_ID", "QUATERNION"),
    field("Display", 14, "DATA_RIGHT_ROTATION_ID", "QUATERNION"),
    field(
        "Display",
        15,
        "DATA_BILLBOARD_RENDER_CONSTRAINTS_ID",
        "BYTE",
    ),
    field("Display", 16, "DATA_BRIGHTNESS_OVERRIDE_ID", "INT"),
    field("Display", 17, "DATA_VIEW_RANGE_ID", "FLOAT"),
    field("Display", 18, "DATA_SHADOW_RADIUS_ID", "FLOAT"),
    field("Display", 19, "DATA_SHADOW_STRENGTH_ID", "FLOAT"),
    field("Display", 20, "DATA_WIDTH_ID", "FLOAT"),
    field("Display", 21, "DATA_HEIGHT_ID", "FLOAT"),
    field("Display", 22, "DATA_GLOW_COLOR_OVERRIDE_ID", "INT"),
];

pub const BLOCK_DISPLAY_METADATA: &[EntityMetadataField] = &[field(
    "BlockDisplay",
    23,
    "DATA_BLOCK_STATE_ID",
    "BLOCK_STATE",
)];

pub const ITEM_DISPLAY_METADATA: &[EntityMetadataField] = &[
    field("ItemDisplay", 23, "DATA_ITEM_STACK_ID", "ITEM_STACK"),
    field("ItemDisplay", 24, "DATA_ITEM_DISPLAY_ID", "BYTE"),
];

pub const TEXT_DISPLAY_METADATA: &[EntityMetadataField] = &[
    field("TextDisplay", 23, "DATA_TEXT_ID", "COMPONENT"),
    field("TextDisplay", 24, "DATA_LINE_WIDTH_ID", "INT"),
    field("TextDisplay", 25, "DATA_BACKGROUND_COLOR_ID", "INT"),
    field("TextDisplay", 26, "DATA_TEXT_OPACITY_ID", "BYTE"),
    field("TextDisplay", 27, "DATA_STYLE_FLAGS_ID", "BYTE"),
];

pub const INTERACTION_METADATA: &[EntityMetadataField] = &[
    field("Interaction", 8, "DATA_WIDTH_ID", "FLOAT"),
    field("Interaction", 9, "DATA_HEIGHT_ID", "FLOAT"),
    field("Interaction", 10, "DATA_RESPONSE_ID", "BOOLEAN"),
];

pub const HANGING_ENTITY_METADATA: &[EntityMetadataField] =
    &[field("HangingEntity", 8, "DATA_DIRECTION", "DIRECTION")];

pub const ITEM_FRAME_METADATA: &[EntityMetadataField] = &[
    field("ItemFrame", 9, "DATA_ITEM", "ITEM_STACK"),
    field("ItemFrame", 10, "DATA_ROTATION", "INT"),
];

pub const PAINTING_METADATA: &[EntityMetadataField] = &[field(
    "Painting",
    9,
    "DATA_PAINTING_VARIANT_ID",
    "PAINTING_VARIANT",
)];

pub const ARMOR_STAND_METADATA: &[EntityMetadataField] = &[
    field("ArmorStand", 15, "DATA_CLIENT_FLAGS", "BYTE"),
    field("ArmorStand", 16, "DATA_HEAD_POSE", "ROTATIONS"),
    field("ArmorStand", 17, "DATA_BODY_POSE", "ROTATIONS"),
    field("ArmorStand", 18, "DATA_LEFT_ARM_POSE", "ROTATIONS"),
    field("ArmorStand", 19, "DATA_RIGHT_ARM_POSE", "ROTATIONS"),
    field("ArmorStand", 20, "DATA_LEFT_LEG_POSE", "ROTATIONS"),
    field("ArmorStand", 21, "DATA_RIGHT_LEG_POSE", "ROTATIONS"),
];

pub const ITEM_ENTITY_METADATA: &[EntityMetadataField] =
    &[field("ItemEntity", 8, "DATA_ITEM", "ITEM_STACK")];

pub const PRIMED_TNT_METADATA: &[EntityMetadataField] = &[
    field("PrimedTnt", 8, "DATA_FUSE_ID", "INT"),
    field("PrimedTnt", 9, "DATA_BLOCK_STATE_ID", "BLOCK_STATE"),
];

pub const FALLING_BLOCK_ENTITY_METADATA: &[EntityMetadataField] = &[field(
    "FallingBlockEntity",
    8,
    "DATA_START_POS",
    "BLOCK_POS",
)];

pub const EYE_OF_ENDER_METADATA: &[EntityMetadataField] =
    &[field("EyeOfEnder", 8, "DATA_ITEM_STACK", "ITEM_STACK")];

pub const ABSTRACT_ARROW_METADATA: &[EntityMetadataField] = &[
    field("AbstractArrow", 8, "ID_FLAGS", "BYTE"),
    field("AbstractArrow", 9, "PIERCE_LEVEL", "BYTE"),
    field("AbstractArrow", 10, "IN_GROUND", "BOOLEAN"),
];

pub const ARROW_METADATA: &[EntityMetadataField] = &[field("Arrow", 11, "ID_EFFECT_COLOR", "INT")];

pub const THROWN_TRIDENT_METADATA: &[EntityMetadataField] = &[
    field("ThrownTrident", 11, "ID_LOYALTY", "BYTE"),
    field("ThrownTrident", 12, "ID_FOIL", "BOOLEAN"),
];

pub const FIREWORK_ROCKET_ENTITY_METADATA: &[EntityMetadataField] = &[
    field(
        "FireworkRocketEntity",
        8,
        "DATA_ID_FIREWORKS_ITEM",
        "ITEM_STACK",
    ),
    field(
        "FireworkRocketEntity",
        9,
        "DATA_ATTACHED_TO_TARGET",
        "OPTIONAL_UNSIGNED_INT",
    ),
    field("FireworkRocketEntity", 10, "DATA_SHOT_AT_ANGLE", "BOOLEAN"),
];

pub const FISHING_HOOK_METADATA: &[EntityMetadataField] = &[
    field("FishingHook", 8, "DATA_HOOKED_ENTITY", "INT"),
    field("FishingHook", 9, "DATA_BITING", "BOOLEAN"),
];

pub const THROWABLE_ITEM_PROJECTILE_METADATA: &[EntityMetadataField] = &[field(
    "ThrowableItemProjectile",
    8,
    "DATA_ITEM_STACK",
    "ITEM_STACK",
)];

pub const FIREBALL_METADATA: &[EntityMetadataField] =
    &[field("Fireball", 8, "DATA_ITEM_STACK", "ITEM_STACK")];

pub const WITHER_SKULL_METADATA: &[EntityMetadataField] =
    &[field("WitherSkull", 8, "DATA_DANGEROUS", "BOOLEAN")];

pub const ENTITY_METADATA_CLASSES: &[EntityMetadataClass] = &[
    class("Entity", None, ENTITY_METADATA),
    class("LivingEntity", Some("Entity"), LIVING_ENTITY_METADATA),
    class("Mob", Some("LivingEntity"), MOB_METADATA),
    class("PathfinderMob", Some("Mob"), &[]),
    class("Monster", Some("PathfinderMob"), &[]),
    class("AmbientCreature", Some("Mob"), &[]),
    class("Player", Some("LivingEntity"), PLAYER_METADATA),
    class("AgeableMob", Some("PathfinderMob"), AGEABLE_MOB_METADATA),
    class("Animal", Some("AgeableMob"), &[]),
    class("TamableAnimal", Some("Animal"), TAMABLE_ANIMAL_METADATA),
    class("AbstractCow", Some("Animal"), &[]),
    class("Allay", Some("PathfinderMob"), ALLAY_METADATA),
    class("Armadillo", Some("Animal"), ARMADILLO_METADATA),
    class("Axolotl", Some("Animal"), AXOLOTL_METADATA),
    class("Bee", Some("Animal"), BEE_METADATA),
    class("Cat", Some("TamableAnimal"), CAT_METADATA),
    class("Chicken", Some("Animal"), CHICKEN_METADATA),
    class("Cow", Some("AbstractCow"), COW_METADATA),
    class("Frog", Some("Animal"), FROG_METADATA),
    class("Pig", Some("Animal"), PIG_METADATA),
    class("Sniffer", Some("Animal"), SNIFFER_METADATA),
    class("Wolf", Some("TamableAnimal"), WOLF_METADATA),
    class("Zombie", Some("Monster"), ZOMBIE_METADATA),
    class("Bat", Some("AmbientCreature"), BAT_METADATA),
    class("Creeper", Some("Monster"), CREEPER_METADATA),
    class("Spider", Some("Monster"), SPIDER_METADATA),
    class("CaveSpider", Some("Spider"), &[]),
    class("Blaze", Some("Monster"), BLAZE_METADATA),
    class("EnderMan", Some("Monster"), ENDER_MAN_METADATA),
    class("Guardian", Some("Monster"), GUARDIAN_METADATA),
    class("ElderGuardian", Some("Guardian"), &[]),
    class("Slime", Some("Mob"), SLIME_METADATA),
    class("MagmaCube", Some("Slime"), &[]),
    class("Phantom", Some("Mob"), PHANTOM_METADATA),
    class("Ghast", Some("Mob"), GHAST_METADATA),
    class("Warden", Some("Monster"), WARDEN_METADATA),
    class("Vex", Some("Monster"), VEX_METADATA),
    class("Zoglin", Some("Monster"), ZOGLIN_METADATA),
    class("Creaking", Some("Monster"), CREAKING_METADATA),
    class("AbstractPiglin", Some("Monster"), ABSTRACT_PIGLIN_METADATA),
    class("Piglin", Some("AbstractPiglin"), PIGLIN_METADATA),
    class("PiglinBrute", Some("AbstractPiglin"), &[]),
    class("AbstractSkeleton", Some("Monster"), &[]),
    class("Skeleton", Some("AbstractSkeleton"), SKELETON_METADATA),
    class("WitherSkeleton", Some("AbstractSkeleton"), &[]),
    class("Stray", Some("Skeleton"), &[]),
    class("Parched", Some("AbstractSkeleton"), &[]),
    class("Bogged", Some("AbstractSkeleton"), BOGGED_METADATA),
    class("Husk", Some("Zombie"), &[]),
    class("Drowned", Some("Zombie"), &[]),
    class("ZombifiedPiglin", Some("Zombie"), &[]),
    class("ZombieVillager", Some("Zombie"), ZOMBIE_VILLAGER_METADATA),
    class("PatrollingMonster", Some("Monster"), &[]),
    class("Raider", Some("PatrollingMonster"), RAIDER_METADATA),
    class("Ravager", Some("Raider"), &[]),
    class("Witch", Some("Raider"), WITCH_METADATA),
    class("AbstractIllager", Some("Raider"), &[]),
    class("Pillager", Some("AbstractIllager"), PILLAGER_METADATA),
    class("Vindicator", Some("AbstractIllager"), &[]),
    class(
        "SpellcasterIllager",
        Some("AbstractIllager"),
        SPELLCASTER_ILLAGER_METADATA,
    ),
    class("Evoker", Some("SpellcasterIllager"), &[]),
    class("Illusioner", Some("SpellcasterIllager"), &[]),
    class(
        "AbstractGolem",
        Some("PathfinderMob"),
        ABSTRACT_GOLEM_METADATA,
    ),
    class("IronGolem", Some("AbstractGolem"), IRON_GOLEM_METADATA),
    class("SnowGolem", Some("AbstractGolem"), SNOW_GOLEM_METADATA),
    class("CopperGolem", Some("AbstractGolem"), COPPER_GOLEM_METADATA),
    class(
        "AbstractVillager",
        Some("AgeableMob"),
        ABSTRACT_VILLAGER_METADATA,
    ),
    class("Villager", Some("AbstractVillager"), VILLAGER_METADATA),
    class("WanderingTrader", Some("AbstractVillager"), &[]),
    class("Rabbit", Some("Animal"), RABBIT_METADATA),
    class("Sheep", Some("Animal"), SHEEP_METADATA),
    class("Turtle", Some("Animal"), TURTLE_METADATA),
    class("Panda", Some("Animal"), PANDA_METADATA),
    class("Fox", Some("Animal"), FOX_METADATA),
    class("PolarBear", Some("Animal"), POLAR_BEAR_METADATA),
    class("Ocelot", Some("Animal"), OCELOT_METADATA),
    class("ShoulderRidingEntity", Some("TamableAnimal"), &[]),
    class("Parrot", Some("ShoulderRidingEntity"), PARROT_METADATA),
    class("AbstractHorse", Some("Animal"), ABSTRACT_HORSE_METADATA),
    class(
        "AbstractChestedHorse",
        Some("AbstractHorse"),
        ABSTRACT_CHESTED_HORSE_METADATA,
    ),
    class("Mule", Some("AbstractChestedHorse"), &[]),
    class("Donkey", Some("AbstractChestedHorse"), &[]),
    class("Horse", Some("AbstractHorse"), HORSE_METADATA),
    class("SkeletonHorse", Some("AbstractHorse"), &[]),
    class("ZombieHorse", Some("AbstractHorse"), &[]),
    class("Llama", Some("AbstractChestedHorse"), LLAMA_METADATA),
    class("TraderLlama", Some("Llama"), &[]),
    class("Camel", Some("AbstractHorse"), CAMEL_METADATA),
    class("CamelHusk", Some("Camel"), &[]),
    class("Goat", Some("Animal"), GOAT_METADATA),
    class("Hoglin", Some("Animal"), HOGLIN_METADATA),
    class("Strider", Some("Animal"), STRIDER_METADATA),
    class("HappyGhast", Some("Animal"), HAPPY_GHAST_METADATA),
    class("MushroomCow", Some("AbstractCow"), MUSHROOM_COW_METADATA),
    class("WaterAnimal", Some("PathfinderMob"), &[]),
    class("AbstractFish", Some("WaterAnimal"), ABSTRACT_FISH_METADATA),
    class("AbstractSchoolingFish", Some("AbstractFish"), &[]),
    class("Cod", Some("AbstractSchoolingFish"), &[]),
    class("Pufferfish", Some("AbstractFish"), PUFFERFISH_METADATA),
    class("Salmon", Some("AbstractSchoolingFish"), SALMON_METADATA),
    class(
        "TropicalFish",
        Some("AbstractSchoolingFish"),
        TROPICAL_FISH_METADATA,
    ),
    class("Tadpole", Some("AbstractFish"), TADPOLE_METADATA),
    class("AgeableWaterCreature", Some("Animal"), &[]),
    class("Squid", Some("AgeableWaterCreature"), &[]),
    class("GlowSquid", Some("Squid"), GLOW_SQUID_METADATA),
    class("Dolphin", Some("AgeableWaterCreature"), DOLPHIN_METADATA),
    class(
        "AbstractNautilus",
        Some("TamableAnimal"),
        ABSTRACT_NAUTILUS_METADATA,
    ),
    class("Nautilus", Some("AbstractNautilus"), &[]),
    class(
        "ZombieNautilus",
        Some("AbstractNautilus"),
        ZOMBIE_NAUTILUS_METADATA,
    ),
    class(
        "AreaEffectCloud",
        Some("Entity"),
        AREA_EFFECT_CLOUD_METADATA,
    ),
    class("Display", Some("Entity"), DISPLAY_METADATA),
    class("BlockDisplay", Some("Display"), BLOCK_DISPLAY_METADATA),
    class("ItemDisplay", Some("Display"), ITEM_DISPLAY_METADATA),
    class("TextDisplay", Some("Display"), TEXT_DISPLAY_METADATA),
    class("Interaction", Some("Entity"), INTERACTION_METADATA),
    class("BlockAttachedEntity", Some("Entity"), &[]),
    class(
        "HangingEntity",
        Some("BlockAttachedEntity"),
        HANGING_ENTITY_METADATA,
    ),
    class("ItemFrame", Some("HangingEntity"), ITEM_FRAME_METADATA),
    class("GlowItemFrame", Some("ItemFrame"), &[]),
    class("Painting", Some("HangingEntity"), PAINTING_METADATA),
    class("ArmorStand", Some("LivingEntity"), ARMOR_STAND_METADATA),
    class("Avatar", Some("LivingEntity"), AVATAR_METADATA),
    class("Mannequin", Some("Avatar"), MANNEQUIN_METADATA),
    class("ItemEntity", Some("Entity"), ITEM_ENTITY_METADATA),
    class("PrimedTnt", Some("Entity"), PRIMED_TNT_METADATA),
    class(
        "FallingBlockEntity",
        Some("Entity"),
        FALLING_BLOCK_ENTITY_METADATA,
    ),
    class("EyeOfEnder", Some("Entity"), EYE_OF_ENDER_METADATA),
    class("ExperienceOrb", Some("Entity"), EXPERIENCE_ORB_METADATA),
    class(
        "OminousItemSpawner",
        Some("Entity"),
        OMINOUS_ITEM_SPAWNER_METADATA,
    ),
    class("VehicleEntity", Some("Entity"), VEHICLE_ENTITY_METADATA),
    class(
        "AbstractBoat",
        Some("VehicleEntity"),
        ABSTRACT_BOAT_METADATA,
    ),
    class("Boat", Some("AbstractBoat"), &[]),
    class("Raft", Some("AbstractBoat"), &[]),
    class("AbstractChestBoat", Some("AbstractBoat"), &[]),
    class("ChestBoat", Some("AbstractChestBoat"), &[]),
    class("ChestRaft", Some("AbstractChestBoat"), &[]),
    class(
        "AbstractMinecart",
        Some("VehicleEntity"),
        ABSTRACT_MINECART_METADATA,
    ),
    class("Minecart", Some("AbstractMinecart"), &[]),
    class("AbstractMinecartContainer", Some("AbstractMinecart"), &[]),
    class("MinecartChest", Some("AbstractMinecartContainer"), &[]),
    class("MinecartHopper", Some("AbstractMinecartContainer"), &[]),
    class("MinecartSpawner", Some("AbstractMinecart"), &[]),
    class("MinecartTNT", Some("AbstractMinecart"), &[]),
    class(
        "MinecartCommandBlock",
        Some("AbstractMinecart"),
        MINECART_COMMAND_BLOCK_METADATA,
    ),
    class(
        "MinecartFurnace",
        Some("AbstractMinecart"),
        MINECART_FURNACE_METADATA,
    ),
    class("Projectile", Some("Entity"), &[]),
    class("AbstractArrow", Some("Projectile"), ABSTRACT_ARROW_METADATA),
    class("Arrow", Some("AbstractArrow"), ARROW_METADATA),
    class("SpectralArrow", Some("AbstractArrow"), &[]),
    class(
        "ThrownTrident",
        Some("AbstractArrow"),
        THROWN_TRIDENT_METADATA,
    ),
    class(
        "FireworkRocketEntity",
        Some("Projectile"),
        FIREWORK_ROCKET_ENTITY_METADATA,
    ),
    class("FishingHook", Some("Projectile"), FISHING_HOOK_METADATA),
    class("ShulkerBullet", Some("Projectile"), &[]),
    class("LlamaSpit", Some("Projectile"), &[]),
    class("ThrowableProjectile", Some("Projectile"), &[]),
    class(
        "ThrowableItemProjectile",
        Some("ThrowableProjectile"),
        THROWABLE_ITEM_PROJECTILE_METADATA,
    ),
    class(
        "Fireball",
        Some("AbstractHurtingProjectile"),
        FIREBALL_METADATA,
    ),
    class("AbstractHurtingProjectile", Some("Projectile"), &[]),
    class(
        "WitherSkull",
        Some("AbstractHurtingProjectile"),
        WITHER_SKULL_METADATA,
    ),
    class("Shulker", Some("AbstractGolem"), SHULKER_METADATA),
    class("WitherBoss", Some("Monster"), WITHER_BOSS_METADATA),
    class("EnderDragon", Some("Mob"), ENDER_DRAGON_METADATA),
    class("EndCrystal", Some("Entity"), END_CRYSTAL_METADATA),
];

pub fn metadata_class(class_name: &str) -> Option<&'static EntityMetadataClass> {
    ENTITY_METADATA_CLASSES
        .iter()
        .find(|class| class.class_name == class_name)
}

pub fn inherited_metadata_fields(class_name: &str) -> Option<Vec<EntityMetadataField>> {
    let class = metadata_class(class_name)?;
    let mut fields = match class.parent {
        Some(parent) => inherited_metadata_fields(parent)?,
        None => Vec::new(),
    };
    fields.extend_from_slice(class.fields);
    Some(fields)
}

const fn field(
    class_name: &'static str,
    index: u8,
    accessor: &'static str,
    serializer: &'static str,
) -> EntityMetadataField {
    EntityMetadataField {
        class_name,
        index,
        accessor,
        serializer,
    }
}

const fn class(
    class_name: &'static str,
    parent: Option<&'static str>,
    fields: &'static [EntityMetadataField],
) -> EntityMetadataClass {
    EntityMetadataClass {
        class_name,
        parent,
        fields,
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests;
