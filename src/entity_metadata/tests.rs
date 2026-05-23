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
fn common_monster_metadata_uses_mob_and_monster_offsets() {
    let creeper = inherited_metadata_fields("Creeper").unwrap();
    assert_eq!(creeper[15].accessor, "DATA_MOB_FLAGS_ID");
    assert_eq!(creeper[16].accessor, "DATA_SWELL_DIR");
    assert_eq!(creeper[18].accessor, "DATA_IS_IGNITED");

    let enderman = inherited_metadata_fields("EnderMan").unwrap();
    assert_eq!(enderman[16].serializer, "OPTIONAL_BLOCK_STATE");
    assert_eq!(enderman[18].accessor, "DATA_STARED_AT");

    let guardian = inherited_metadata_fields("Guardian").unwrap();
    assert_eq!(guardian[16].accessor, "DATA_ID_MOVING");
    assert_eq!(guardian[17].accessor, "DATA_ID_ATTACK_TARGET");
    assert_eq!(
        inherited_metadata_fields("ElderGuardian").unwrap(),
        guardian
    );

    let slime = inherited_metadata_fields("MagmaCube").unwrap();
    assert_eq!(slime[16].accessor, "ID_SIZE");

    let creaking = inherited_metadata_fields("Creaking").unwrap();
    assert_eq!(creaking[16].accessor, "CAN_MOVE");
    assert_eq!(creaking[19].serializer, "OPTIONAL_BLOCK_POS");
}

#[test]
fn monster_subclass_branches_continue_after_parent_accessors() {
    let piglin = inherited_metadata_fields("Piglin").unwrap();
    assert_eq!(piglin[16].accessor, "DATA_IMMUNE_TO_ZOMBIFICATION");
    assert_eq!(piglin[17].accessor, "DATA_BABY_ID");
    assert_eq!(piglin[19].accessor, "DATA_IS_DANCING");

    let zombie_villager = inherited_metadata_fields("ZombieVillager").unwrap();
    assert_eq!(zombie_villager[18].accessor, "DATA_DROWNED_CONVERSION_ID");
    assert_eq!(zombie_villager[19].accessor, "DATA_CONVERTING_ID");
    assert_eq!(zombie_villager[20].serializer, "VILLAGER_DATA");

    let skeleton = inherited_metadata_fields("Skeleton").unwrap();
    assert_eq!(skeleton[16].accessor, "DATA_STRAY_CONVERSION_ID");
    assert_eq!(inherited_metadata_fields("Stray").unwrap(), skeleton);

    let bogged = inherited_metadata_fields("Bogged").unwrap();
    assert_eq!(bogged[16].accessor, "DATA_SHEARED");
}

#[test]
fn raider_golem_and_villager_metadata_offsets_match_decompiled_parents() {
    let witch = inherited_metadata_fields("Witch").unwrap();
    assert_eq!(witch[16].accessor, "IS_CELEBRATING");
    assert_eq!(witch[17].accessor, "DATA_USING_ITEM");

    let pillager = inherited_metadata_fields("Pillager").unwrap();
    assert_eq!(pillager[16].accessor, "IS_CELEBRATING");
    assert_eq!(pillager[17].accessor, "IS_CHARGING_CROSSBOW");

    let evoker = inherited_metadata_fields("Evoker").unwrap();
    assert_eq!(evoker[17].accessor, "DATA_SPELL_CASTING_ID");
    assert_eq!(inherited_metadata_fields("Illusioner").unwrap(), evoker);

    let copper_golem = inherited_metadata_fields("CopperGolem").unwrap();
    assert_eq!(copper_golem[16].serializer, "WEATHERING_COPPER_STATE");
    assert_eq!(copper_golem[17].serializer, "COPPER_GOLEM_STATE");

    let villager = inherited_metadata_fields("Villager").unwrap();
    assert_eq!(villager[18].accessor, "DATA_UNHAPPY_COUNTER");
    assert_eq!(villager[19].serializer, "VILLAGER_DATA");
    assert_eq!(villager[20].serializer, "BOOLEAN");
}

#[test]
fn additional_animal_metadata_covers_common_visual_state() {
    let panda = inherited_metadata_fields("Panda").unwrap();
    assert_eq!(panda[18].accessor, "UNHAPPY_COUNTER");
    assert_eq!(panda[23].accessor, "DATA_ID_FLAGS");

    let fox = inherited_metadata_fields("Fox").unwrap();
    assert_eq!(fox[18].accessor, "DATA_TYPE_ID");
    assert_eq!(fox[21].serializer, "OPTIONAL_LIVING_ENTITY_REFERENCE");

    let parrot = inherited_metadata_fields("Parrot").unwrap();
    assert_eq!(parrot[18].accessor, "DATA_FLAGS_ID");
    assert_eq!(parrot[20].accessor, "DATA_VARIANT_ID");

    let camel = inherited_metadata_fields("Camel").unwrap();
    assert_eq!(camel[18].accessor, "DATA_ID_FLAGS");
    assert_eq!(camel[19].accessor, "DASH");
    assert_eq!(camel[20].serializer, "LONG");

    let horse = inherited_metadata_fields("Horse").unwrap();
    assert_eq!(horse[18].accessor, "DATA_ID_FLAGS");
    assert_eq!(horse[19].accessor, "DATA_ID_TYPE_VARIANT");

    let llama = inherited_metadata_fields("Llama").unwrap();
    assert_eq!(llama[19].accessor, "DATA_ID_CHEST");
    assert_eq!(llama[20].accessor, "DATA_STRENGTH_ID");
    assert_eq!(llama[21].accessor, "DATA_VARIANT_ID");

    let goat = inherited_metadata_fields("Goat").unwrap();
    assert_eq!(goat[18].accessor, "DATA_IS_SCREAMING_GOAT");
    assert_eq!(goat[20].accessor, "DATA_HAS_RIGHT_HORN");

    let strider = inherited_metadata_fields("Strider").unwrap();
    assert_eq!(strider[18].accessor, "DATA_BOOST_TIME");
    assert_eq!(strider[19].accessor, "DATA_SUFFOCATING");
}

#[test]
fn aquatic_entity_metadata_follows_water_and_animal_branches() {
    let cod = inherited_metadata_fields("Cod").unwrap();
    assert_eq!(cod[15].accessor, "DATA_MOB_FLAGS_ID");
    assert_eq!(cod[16].accessor, "FROM_BUCKET");

    let pufferfish = inherited_metadata_fields("Pufferfish").unwrap();
    assert_eq!(pufferfish[16].accessor, "FROM_BUCKET");
    assert_eq!(pufferfish[17].accessor, "PUFF_STATE");

    let tropical_fish = inherited_metadata_fields("TropicalFish").unwrap();
    assert_eq!(tropical_fish[17].accessor, "DATA_ID_TYPE_VARIANT");

    let tadpole = inherited_metadata_fields("Tadpole").unwrap();
    assert_eq!(tadpole[17].accessor, "AGE_LOCKED");

    let glow_squid = inherited_metadata_fields("GlowSquid").unwrap();
    assert_eq!(glow_squid[16].accessor, "DATA_BABY_ID");
    assert_eq!(glow_squid[18].accessor, "DATA_DARK_TICKS_REMAINING");

    let dolphin = inherited_metadata_fields("Dolphin").unwrap();
    assert_eq!(dolphin[18].accessor, "GOT_FISH");
    assert_eq!(dolphin[19].accessor, "MOISTNESS_LEVEL");

    let zombie_nautilus = inherited_metadata_fields("ZombieNautilus").unwrap();
    assert_eq!(zombie_nautilus[20].accessor, "DASH");
    assert_eq!(zombie_nautilus[21].serializer, "ZOMBIE_NAUTILUS_VARIANT");
}

#[test]
fn special_entities_and_bosses_use_decompiled_root_offsets() {
    let orb = inherited_metadata_fields("ExperienceOrb").unwrap();
    assert_eq!(orb[8].accessor, "DATA_VALUE");

    let spawner = inherited_metadata_fields("OminousItemSpawner").unwrap();
    assert_eq!(spawner[8].serializer, "ITEM_STACK");

    let crystal = inherited_metadata_fields("EndCrystal").unwrap();
    assert_eq!(crystal[8].serializer, "OPTIONAL_BLOCK_POS");
    assert_eq!(crystal[9].accessor, "DATA_SHOW_BOTTOM");

    let dragon = inherited_metadata_fields("EnderDragon").unwrap();
    assert_eq!(dragon[15].accessor, "DATA_MOB_FLAGS_ID");
    assert_eq!(dragon[16].accessor, "DATA_PHASE");

    let wither = inherited_metadata_fields("WitherBoss").unwrap();
    assert_eq!(wither[16].accessor, "DATA_TARGET_A");
    assert_eq!(wither[19].accessor, "DATA_ID_INV");

    let shulker = inherited_metadata_fields("Shulker").unwrap();
    assert_eq!(shulker[16].serializer, "DIRECTION");
    assert_eq!(shulker[18].accessor, "DATA_COLOR_ID");
}

#[test]
fn avatar_mannequin_and_vehicle_metadata_inherit_parent_fields() {
    let mannequin = inherited_metadata_fields("Mannequin").unwrap();
    assert_eq!(mannequin[15].accessor, "DATA_PLAYER_MAIN_HAND");
    assert_eq!(mannequin[16].accessor, "DATA_PLAYER_MODE_CUSTOMISATION");
    assert_eq!(mannequin[17].serializer, "RESOLVABLE_PROFILE");
    assert_eq!(mannequin[19].serializer, "OPTIONAL_COMPONENT");

    let boat = inherited_metadata_fields("Boat").unwrap();
    assert_eq!(boat[8].accessor, "DATA_ID_HURT");
    assert_eq!(boat[10].serializer, "FLOAT");
    assert_eq!(boat[11].accessor, "DATA_ID_PADDLE_LEFT");
    assert_eq!(boat[13].accessor, "DATA_ID_BUBBLE_TIME");

    let minecart = inherited_metadata_fields("Minecart").unwrap();
    assert_eq!(minecart[11].serializer, "OPTIONAL_BLOCK_STATE");
    assert_eq!(minecart[12].accessor, "DATA_ID_DISPLAY_OFFSET");

    let command_minecart = inherited_metadata_fields("MinecartCommandBlock").unwrap();
    assert_eq!(command_minecart[13].accessor, "DATA_ID_COMMAND_NAME");
    assert_eq!(command_minecart[14].serializer, "COMPONENT");

    let furnace_minecart = inherited_metadata_fields("MinecartFurnace").unwrap();
    assert_eq!(furnace_minecart[13].accessor, "DATA_ID_FUEL");
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
