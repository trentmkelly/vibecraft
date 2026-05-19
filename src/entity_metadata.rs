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
    class("ItemEntity", Some("Entity"), ITEM_ENTITY_METADATA),
    class("PrimedTnt", Some("Entity"), PRIMED_TNT_METADATA),
    class(
        "FallingBlockEntity",
        Some("Entity"),
        FALLING_BLOCK_ENTITY_METADATA,
    ),
    class("EyeOfEnder", Some("Entity"), EYE_OF_ENDER_METADATA),
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

#[cfg(test)]
mod tests {
    use super::{inherited_metadata_fields, metadata_class};

    #[test]
    fn root_entity_metadata_indexes_match_java_define_id_order() {
        let fields = inherited_metadata_fields("Entity").unwrap();
        assert_eq!(fields.len(), 8);
        assert_eq!(fields[0].accessor, "DATA_SHARED_FLAGS_ID");
        assert_eq!(fields[0].index, 0);
        assert_eq!(fields[0].serializer, "BYTE");
        assert_eq!(fields[6].accessor, "DATA_POSE");
        assert_eq!(fields[6].index, 6);
        assert_eq!(fields[7].accessor, "DATA_TICKS_FROZEN");
        assert_eq!(fields[7].serializer, "INT");
    }

    #[test]
    fn living_mob_player_and_common_subclasses_inherit_vanilla_indexes() {
        let living = inherited_metadata_fields("LivingEntity").unwrap();
        assert_eq!(living[8].accessor, "DATA_LIVING_ENTITY_FLAGS");
        assert_eq!(living[8].index, 8);
        assert_eq!(living[14].accessor, "SLEEPING_POS_ID");
        assert_eq!(living[14].serializer, "OPTIONAL_BLOCK_POS");

        let mob = inherited_metadata_fields("Mob").unwrap();
        assert_eq!(mob[15].accessor, "DATA_MOB_FLAGS_ID");
        assert_eq!(mob[15].index, 15);

        let player = inherited_metadata_fields("Player").unwrap();
        assert_eq!(player[15].accessor, "DATA_PLAYER_ABSORPTION_ID");
        assert_eq!(player[18].accessor, "DATA_SHOULDER_PARROT_RIGHT");

        let pig = inherited_metadata_fields("Pig").unwrap();
        assert_eq!(pig[16].accessor, "DATA_BABY_ID");
        assert_eq!(pig[18].accessor, "DATA_BOOST_TIME");
        assert_eq!(pig[20].serializer, "PIG_SOUND_VARIANT");

        let zombie = inherited_metadata_fields("Zombie").unwrap();
        assert_eq!(zombie[15].accessor, "DATA_MOB_FLAGS_ID");
        assert_eq!(zombie[16].accessor, "DATA_BABY_ID");
        assert_eq!(zombie[18].accessor, "DATA_DROWNED_CONVERSION_ID");
    }

    #[test]
    fn common_animal_subclasses_follow_decompiled_accessor_order() {
        let allay = inherited_metadata_fields("Allay").unwrap();
        assert_eq!(allay[15].accessor, "DATA_MOB_FLAGS_ID");
        assert_eq!(allay[16].accessor, "DATA_DANCING");
        assert_eq!(allay[17].accessor, "DATA_CAN_DUPLICATE");

        let cat = inherited_metadata_fields("Cat").unwrap();
        assert_eq!(cat[18].accessor, "DATA_FLAGS_ID");
        assert_eq!(cat[19].accessor, "DATA_OWNERUUID_ID");
        assert_eq!(cat[20].accessor, "DATA_VARIANT_ID");
        assert_eq!(cat[24].serializer, "CAT_SOUND_VARIANT");

        let wolf = inherited_metadata_fields("Wolf").unwrap();
        assert_eq!(wolf[20].accessor, "DATA_INTERESTED_ID");
        assert_eq!(wolf[22].serializer, "LONG");
        assert_eq!(wolf[24].serializer, "WOLF_SOUND_VARIANT");

        let axolotl = inherited_metadata_fields("Axolotl").unwrap();
        assert_eq!(axolotl[18].accessor, "DATA_VARIANT");
        assert_eq!(axolotl[20].accessor, "FROM_BUCKET");
    }

    #[test]
    fn variant_animals_use_post_ageable_indexes() {
        for class_name in ["Chicken", "Cow", "Frog", "Pig", "Sniffer"] {
            let fields = inherited_metadata_fields(class_name).unwrap();
            assert_eq!(fields[16].accessor, "DATA_BABY_ID");
            assert_eq!(fields[17].accessor, "AGE_LOCKED");
            assert_eq!(fields[18].class_name, class_name);
        }

        let armadillo = inherited_metadata_fields("Armadillo").unwrap();
        assert_eq!(armadillo[18].serializer, "ARMADILLO_STATE");

        let bee = inherited_metadata_fields("Bee").unwrap();
        assert_eq!(bee[18].accessor, "DATA_FLAGS_ID");
        assert_eq!(bee[19].serializer, "LONG");
    }

    #[test]
    fn hierarchy_nodes_without_new_accessors_are_explicit() {
        assert_eq!(metadata_class("PathfinderMob").unwrap().fields, &[]);
        assert_eq!(metadata_class("Monster").unwrap().fields, &[]);
        assert_eq!(metadata_class("Animal").unwrap().fields, &[]);
        assert_eq!(metadata_class("Projectile").unwrap().fields, &[]);
        assert_eq!(metadata_class("BlockAttachedEntity").unwrap().fields, &[]);
        assert!(metadata_class("Missing").is_none());
    }

    #[test]
    fn non_living_entity_metadata_starts_after_root_entity_fields() {
        let item = inherited_metadata_fields("ItemEntity").unwrap();
        assert_eq!(item[8].accessor, "DATA_ITEM");
        assert_eq!(item[8].serializer, "ITEM_STACK");

        let tnt = inherited_metadata_fields("PrimedTnt").unwrap();
        assert_eq!(tnt[8].accessor, "DATA_FUSE_ID");
        assert_eq!(tnt[9].serializer, "BLOCK_STATE");

        let falling_block = inherited_metadata_fields("FallingBlockEntity").unwrap();
        assert_eq!(falling_block[8].accessor, "DATA_START_POS");
        assert_eq!(falling_block[8].serializer, "BLOCK_POS");

        let area_effect_cloud = inherited_metadata_fields("AreaEffectCloud").unwrap();
        assert_eq!(area_effect_cloud[8].accessor, "DATA_RADIUS");
        assert_eq!(area_effect_cloud[10].serializer, "PARTICLE");

        let interaction = inherited_metadata_fields("Interaction").unwrap();
        assert_eq!(interaction[8].accessor, "DATA_WIDTH_ID");
        assert_eq!(interaction[10].accessor, "DATA_RESPONSE_ID");
    }

    #[test]
    fn display_entity_subclasses_continue_after_display_base_indexes() {
        let display = inherited_metadata_fields("Display").unwrap();
        assert_eq!(
            display[8].accessor,
            "DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID"
        );
        assert_eq!(display[11].serializer, "VECTOR3");
        assert_eq!(display[14].serializer, "QUATERNION");
        assert_eq!(display[22].accessor, "DATA_GLOW_COLOR_OVERRIDE_ID");

        let block_display = inherited_metadata_fields("BlockDisplay").unwrap();
        assert_eq!(block_display[23].accessor, "DATA_BLOCK_STATE_ID");
        assert_eq!(block_display[23].serializer, "BLOCK_STATE");

        let item_display = inherited_metadata_fields("ItemDisplay").unwrap();
        assert_eq!(item_display[23].accessor, "DATA_ITEM_STACK_ID");
        assert_eq!(item_display[24].accessor, "DATA_ITEM_DISPLAY_ID");

        let text_display = inherited_metadata_fields("TextDisplay").unwrap();
        assert_eq!(text_display[23].accessor, "DATA_TEXT_ID");
        assert_eq!(text_display[27].accessor, "DATA_STYLE_FLAGS_ID");
    }

    #[test]
    fn hanging_and_decorative_entities_include_parent_accessors() {
        let item_frame = inherited_metadata_fields("ItemFrame").unwrap();
        assert_eq!(item_frame[8].accessor, "DATA_DIRECTION");
        assert_eq!(item_frame[9].accessor, "DATA_ITEM");
        assert_eq!(item_frame[10].accessor, "DATA_ROTATION");

        let glow_item_frame = inherited_metadata_fields("GlowItemFrame").unwrap();
        assert_eq!(glow_item_frame, item_frame);

        let painting = inherited_metadata_fields("Painting").unwrap();
        assert_eq!(painting[8].serializer, "DIRECTION");
        assert_eq!(painting[9].serializer, "PAINTING_VARIANT");

        let armor_stand = inherited_metadata_fields("ArmorStand").unwrap();
        assert_eq!(armor_stand[14].accessor, "SLEEPING_POS_ID");
        assert_eq!(armor_stand[15].accessor, "DATA_CLIENT_FLAGS");
        assert_eq!(armor_stand[21].accessor, "DATA_RIGHT_LEG_POSE");
    }

    #[test]
    fn projectile_metadata_matches_java_subclass_offsets() {
        let arrow = inherited_metadata_fields("Arrow").unwrap();
        assert_eq!(arrow[8].accessor, "ID_FLAGS");
        assert_eq!(arrow[10].accessor, "IN_GROUND");
        assert_eq!(arrow[11].accessor, "ID_EFFECT_COLOR");

        let trident = inherited_metadata_fields("ThrownTrident").unwrap();
        assert_eq!(trident[11].accessor, "ID_LOYALTY");
        assert_eq!(trident[12].accessor, "ID_FOIL");

        let firework = inherited_metadata_fields("FireworkRocketEntity").unwrap();
        assert_eq!(firework[8].serializer, "ITEM_STACK");
        assert_eq!(firework[9].serializer, "OPTIONAL_UNSIGNED_INT");
        assert_eq!(firework[10].serializer, "BOOLEAN");

        let thrown_item = inherited_metadata_fields("ThrowableItemProjectile").unwrap();
        assert_eq!(thrown_item[8].accessor, "DATA_ITEM_STACK");

        let fireball = inherited_metadata_fields("Fireball").unwrap();
        assert_eq!(fireball[8].serializer, "ITEM_STACK");

        let wither_skull = inherited_metadata_fields("WitherSkull").unwrap();
        assert_eq!(wither_skull[8].accessor, "DATA_DANGEROUS");
    }
}
