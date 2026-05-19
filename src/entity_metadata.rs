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
        assert!(metadata_class("Missing").is_none());
    }
}
