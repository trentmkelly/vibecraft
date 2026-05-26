use crate::living_entity::EquipmentSlot;
use crate::map_state::DyeColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobSourceFamily {
    Ambient,
    Animal,
    WaterAnimal,
    Npc,
    Monster,
    Boss,
    Raid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobFamilyDef {
    pub id: &'static str,
    pub family: MobSourceFamily,
    pub package: &'static str,
    pub mob_category: &'static str,
    pub tameable: bool,
    pub breedable: bool,
    pub rideable: bool,
    pub bucketable: bool,
    pub shearable: bool,
    pub variant_backed: bool,
    pub converts_or_transforms: bool,
}

pub const AMBIENT_FAMILIES: &[MobFamilyDef] = &[family(
    "bat",
    MobSourceFamily::Ambient,
    "ambient",
    "ambient",
)];

pub const ANIMAL_FAMILIES: &[MobFamilyDef] = &[
    family("allay", MobSourceFamily::Animal, "animal/allay", "creature"),
    family(
        "armadillo",
        MobSourceFamily::Animal,
        "animal/armadillo",
        "creature",
    ),
    bucketable("axolotl", "animal/axolotl", "axolotls"),
    family("bee", MobSourceFamily::Animal, "animal/bee", "creature"),
    rideable("camel", "animal/camel", "creature"),
    breedable_variant("chicken", "animal/chicken", "creature"),
    breedable_variant("cow", "animal/cow", "creature"),
    bucketable("dolphin", "animal/dolphin", "water_creature"),
    rideable_variant("equine", "animal/equine", "creature"),
    tameable_variant("feline", "animal/feline", "creature"),
    bucketable_variant("fish", "animal/fish", "water_ambient"),
    family("fox", MobSourceFamily::Animal, "animal/fox", "creature"),
    bucketable_variant("frog", "animal/frog", "creature"),
    family("goat", MobSourceFamily::Animal, "animal/goat", "creature"),
    family("golem", MobSourceFamily::Animal, "animal/golem", "misc"),
    rideable("happy_ghast", "animal/happyghast", "creature"),
    bucketable_variant("nautilus", "animal/nautilus", "water_creature"),
    family("panda", MobSourceFamily::Animal, "animal/panda", "creature"),
    tameable("parrot", "animal/parrot", "creature"),
    breedable_variant("pig", "animal/pig", "creature"),
    family(
        "polar_bear",
        MobSourceFamily::Animal,
        "animal/polarbear",
        "creature",
    ),
    family(
        "rabbit",
        MobSourceFamily::Animal,
        "animal/rabbit",
        "creature",
    ),
    shearable("sheep", "animal/sheep", "creature"),
    family(
        "sniffer",
        MobSourceFamily::Animal,
        "animal/sniffer",
        "creature",
    ),
    bucketable("squid", "animal/squid", "water_ambient"),
    family(
        "turtle",
        MobSourceFamily::Animal,
        "animal/turtle",
        "creature",
    ),
    tameable_variant("wolf", "animal/wolf", "creature"),
];

pub const MONSTER_FAMILIES: &[MobFamilyDef] = &[
    monster("blaze", "monster"),
    transforming_monster("breeze", "monster/breeze"),
    monster("creaking", "monster/creaking"),
    monster("creeper", "monster"),
    monster("elder_guardian", "monster"),
    monster("enderman", "monster"),
    monster("endermite", "monster"),
    monster("ghast", "monster"),
    monster("giant", "monster"),
    monster("guardian", "monster"),
    transforming_monster("hoglin", "monster/hoglin"),
    monster("illager", "monster/illager"),
    monster("magma_cube", "monster"),
    monster("phantom", "monster"),
    transforming_monster("piglin", "monster/piglin"),
    monster("ravager", "monster"),
    monster("shulker", "monster"),
    transforming_monster("skeleton", "monster/skeleton"),
    monster("silverfish", "monster"),
    monster("slime", "monster"),
    monster("spider", "monster/spider"),
    monster("strider", "monster"),
    monster("vex", "monster"),
    monster("warden", "monster/warden"),
    monster("witch", "monster"),
    transforming_monster("zoglin", "monster"),
    transforming_monster("zombie", "monster/zombie"),
];

pub const NPC_FAMILIES: &[MobFamilyDef] = &[
    family("villager", MobSourceFamily::Npc, "npc/villager", "misc"),
    family(
        "wandering_trader",
        MobSourceFamily::Npc,
        "npc/wanderingtrader",
        "creature",
    ),
];

pub const BOSS_AND_RAID_FAMILIES: &[MobFamilyDef] = &[
    family(
        "ender_dragon",
        MobSourceFamily::Boss,
        "boss/enderdragon",
        "monster",
    ),
    family("wither", MobSourceFamily::Boss, "boss/wither", "monster"),
    family("raid", MobSourceFamily::Raid, "raid", "misc"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobRuntimeState {
    pub family_id: &'static str,
    pub persistent: bool,
    pub age: i32,
    pub variant: Option<&'static str>,
    pub owner_uuid: Option<String>,
    pub leashed_to: Option<i32>,
    pub anger_target: Option<i32>,
    pub conversion_timer: Option<i32>,
    pub passengers: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WolfTameRoll {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WolfInteractionOutcome {
    NoAction,
    ConsumedBoneTameSuccess,
    ConsumedBoneTameFailure,
    DyedCollar,
    EquippedArmor,
    RepairedArmor,
    ToggledSitting,
    Fed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WolfArmorState {
    pub item: &'static str,
    pub damage: i32,
    pub max_damage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WolfRuntimeState {
    pub mob: MobRuntimeState,
    pub collar_color: DyeColor,
    pub ordered_to_sit: bool,
    pub health: i32,
    pub max_health: i32,
    pub baby: bool,
    pub angry: bool,
    pub body_armor: Option<WolfArmorState>,
    pub target_entity_id: Option<i32>,
}

impl WolfRuntimeState {
    pub fn new() -> Self {
        Self {
            mob: MobRuntimeState::new("wolf"),
            collar_color: DyeColor::Red,
            ordered_to_sit: false,
            health: 8,
            max_health: 8,
            baby: false,
            angry: false,
            body_armor: None,
            target_entity_id: None,
        }
    }

    pub fn is_tame(&self) -> bool {
        self.mob.owner_uuid.is_some()
    }

    pub fn try_tame_with_bone(
        &mut self,
        owner_uuid: impl Into<String>,
        roll: WolfTameRoll,
    ) -> WolfInteractionOutcome {
        if self.is_tame() || self.angry {
            return WolfInteractionOutcome::NoAction;
        }
        match roll {
            WolfTameRoll::Success => {
                self.mob
                    .tame(
                        family_by_id("wolf").expect("wolf family exists"),
                        owner_uuid.into(),
                    )
                    .expect("wolf is tameable");
                self.max_health = 40;
                self.health = 40;
                self.ordered_to_sit = true;
                self.target_entity_id = None;
                WolfInteractionOutcome::ConsumedBoneTameSuccess
            }
            WolfTameRoll::Failure => WolfInteractionOutcome::ConsumedBoneTameFailure,
        }
    }

    pub fn dye_collar(&mut self, player_uuid: &str, dye: DyeColor) -> WolfInteractionOutcome {
        if self.mob.owner_uuid.as_deref() != Some(player_uuid) || self.collar_color == dye {
            return WolfInteractionOutcome::NoAction;
        }
        self.collar_color = dye;
        WolfInteractionOutcome::DyedCollar
    }

    pub fn equip_body_armor(
        &mut self,
        player_uuid: &str,
        item: &'static str,
        max_damage: i32,
    ) -> WolfInteractionOutcome {
        if self.mob.owner_uuid.as_deref() != Some(player_uuid)
            || self.baby
            || self.body_armor.is_some()
            || !wolf_body_armor_slot_accepts(item)
        {
            return WolfInteractionOutcome::NoAction;
        }
        self.body_armor = Some(WolfArmorState {
            item,
            damage: 0,
            max_damage: max_damage.max(1),
        });
        WolfInteractionOutcome::EquippedArmor
    }

    pub fn repair_body_armor(
        &mut self,
        player_uuid: &str,
        item: &'static str,
    ) -> WolfInteractionOutcome {
        if self.mob.owner_uuid.as_deref() != Some(player_uuid)
            || !self.ordered_to_sit
            || item != "minecraft:armadillo_scute"
        {
            return WolfInteractionOutcome::NoAction;
        }
        let Some(armor) = self.body_armor.as_mut() else {
            return WolfInteractionOutcome::NoAction;
        };
        if armor.damage <= 0 {
            return WolfInteractionOutcome::NoAction;
        }
        let repair_unit = (armor.max_damage as f32 * 0.125) as i32;
        armor.damage = (armor.damage - repair_unit).max(0);
        WolfInteractionOutcome::RepairedArmor
    }

    pub fn toggle_sitting(&mut self, player_uuid: &str) -> WolfInteractionOutcome {
        if self.mob.owner_uuid.as_deref() != Some(player_uuid) {
            return WolfInteractionOutcome::NoAction;
        }
        self.ordered_to_sit = !self.ordered_to_sit;
        self.target_entity_id = None;
        WolfInteractionOutcome::ToggledSitting
    }

    pub fn feed(&mut self, item: &'static str) -> WolfInteractionOutcome {
        if !wolf_food_accepts(item) || self.health >= self.max_health {
            return WolfInteractionOutcome::NoAction;
        }
        self.health = (self.health + 2).min(self.max_health);
        WolfInteractionOutcome::Fed
    }

    pub fn owner_was_hurt_by(&mut self, owner_uuid: &str, attacker_entity_id: i32) -> bool {
        if self.mob.owner_uuid.as_deref() != Some(owner_uuid) || self.ordered_to_sit {
            return false;
        }
        self.target_entity_id = Some(attacker_entity_id);
        true
    }

    pub fn hurt_owner_target(&mut self, owner_uuid: &str, target_entity_id: i32) -> bool {
        if self.mob.owner_uuid.as_deref() != Some(owner_uuid) || self.ordered_to_sit {
            return false;
        }
        self.target_entity_id = Some(target_entity_id);
        true
    }
}

pub fn wolf_body_armor_slot_accepts(item: &str) -> bool {
    item == "minecraft:wolf_armor"
}

pub fn wolf_equipment_slot_for_item(item: &str) -> Option<EquipmentSlot> {
    wolf_body_armor_slot_accepts(item).then_some(EquipmentSlot::Body)
}

pub fn wolf_food_accepts(item: &str) -> bool {
    matches!(
        item,
        "minecraft:beef"
            | "minecraft:chicken"
            | "minecraft:cooked_beef"
            | "minecraft:cooked_chicken"
            | "minecraft:cooked_mutton"
            | "minecraft:cooked_porkchop"
            | "minecraft:cooked_rabbit"
            | "minecraft:mutton"
            | "minecraft:porkchop"
            | "minecraft:rabbit"
            | "minecraft:rotten_flesh"
    )
}

impl MobRuntimeState {
    pub fn new(family_id: &'static str) -> Self {
        Self {
            family_id,
            persistent: false,
            age: 0,
            variant: None,
            owner_uuid: None,
            leashed_to: None,
            anger_target: None,
            conversion_timer: None,
            passengers: Vec::new(),
        }
    }

    pub fn set_variant(
        &mut self,
        family: &MobFamilyDef,
        variant: &'static str,
    ) -> Result<(), &'static str> {
        if !family.variant_backed {
            return Err("family does not use variants");
        }
        self.variant = Some(variant);
        Ok(())
    }

    pub fn tame(&mut self, family: &MobFamilyDef, owner_uuid: String) -> Result<(), &'static str> {
        if !family.tameable {
            return Err("family is not tameable");
        }
        self.owner_uuid = Some(owner_uuid);
        self.persistent = true;
        Ok(())
    }

    pub fn start_conversion(
        &mut self,
        family: &MobFamilyDef,
        ticks: i32,
    ) -> Result<(), &'static str> {
        if !family.converts_or_transforms {
            return Err("family does not convert");
        }
        self.conversion_timer = Some(ticks.max(0));
        Ok(())
    }

    pub fn tick_conversion(&mut self) -> bool {
        if let Some(timer) = self.conversion_timer.as_mut() {
            *timer = (*timer - 1).max(0);
            if *timer == 0 {
                self.conversion_timer = None;
                return true;
            }
        }
        false
    }

    pub fn add_passenger(&mut self, family: &MobFamilyDef, passenger_id: i32) -> bool {
        if !family.rideable || self.passengers.contains(&passenger_id) {
            return false;
        }
        self.passengers.push(passenger_id);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionCapability {
    Bucket,
    Shear,
    Breed,
    Tame,
    Ride,
}

pub fn supports_interaction(family: &MobFamilyDef, interaction: InteractionCapability) -> bool {
    match interaction {
        InteractionCapability::Bucket => family.bucketable,
        InteractionCapability::Shear => family.shearable,
        InteractionCapability::Breed => family.breedable,
        InteractionCapability::Tame => family.tameable,
        InteractionCapability::Ride => family.rideable,
    }
}

pub fn family_by_id(id: &str) -> Option<&'static MobFamilyDef> {
    all_families().into_iter().find(|family| family.id == id)
}

pub fn all_families() -> Vec<&'static MobFamilyDef> {
    AMBIENT_FAMILIES
        .iter()
        .chain(ANIMAL_FAMILIES)
        .chain(MONSTER_FAMILIES)
        .chain(NPC_FAMILIES)
        .chain(BOSS_AND_RAID_FAMILIES)
        .collect()
}

pub const MOB_FAMILY_CHECKLIST_SURFACE: &[&str] = &[
    "ambient mobs",
    "animals",
    "water mobs",
    "NPCs",
    "monsters",
    "bosses",
    "raids",
    "variants",
    "allay",
    "armadillo",
    "axolotl",
    "bee",
    "camel",
    "chicken",
    "cow",
    "dolphin",
    "equine",
    "feline",
    "fish",
    "fox",
    "frog",
    "goat",
    "golem",
    "happy ghast",
    "nautilus",
    "panda",
    "parrot",
    "pig",
    "polar bear",
    "rabbit",
    "sheep",
    "sniffer",
    "squid",
    "turtle",
    "wolf",
    "breeze",
    "creaking",
    "hoglin",
    "illager",
    "piglin",
    "skeleton",
    "spider",
    "warden",
    "zombie",
    "villagers",
    "wandering traders",
];

const fn family(
    id: &'static str,
    family: MobSourceFamily,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        id,
        family,
        package,
        mob_category,
        tameable: false,
        breedable: false,
        rideable: false,
        bucketable: false,
        shearable: false,
        variant_backed: false,
        converts_or_transforms: false,
    }
}

const fn tameable(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        tameable: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn tameable_variant(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        tameable: true,
        variant_backed: true,
        breedable: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn breedable_variant(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        breedable: true,
        variant_backed: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn rideable(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        rideable: true,
        breedable: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn rideable_variant(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        rideable: true,
        breedable: true,
        variant_backed: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn bucketable(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        bucketable: true,
        ..family(id, MobSourceFamily::WaterAnimal, package, mob_category)
    }
}

const fn bucketable_variant(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        bucketable: true,
        variant_backed: true,
        ..family(id, MobSourceFamily::WaterAnimal, package, mob_category)
    }
}

const fn shearable(
    id: &'static str,
    package: &'static str,
    mob_category: &'static str,
) -> MobFamilyDef {
    MobFamilyDef {
        shearable: true,
        breedable: true,
        ..family(id, MobSourceFamily::Animal, package, mob_category)
    }
}

const fn monster(id: &'static str, package: &'static str) -> MobFamilyDef {
    family(id, MobSourceFamily::Monster, package, "monster")
}

const fn transforming_monster(id: &'static str, package: &'static str) -> MobFamilyDef {
    MobFamilyDef {
        converts_or_transforms: true,
        ..family(id, MobSourceFamily::Monster, package, "monster")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mob_family_manifests_cover_decompiled_source_tree_groups() {
        assert_eq!(AMBIENT_FAMILIES.len(), 1);
        assert_eq!(ANIMAL_FAMILIES.len(), 27);
        assert_eq!(MONSTER_FAMILIES.len(), 27);
        assert_eq!(NPC_FAMILIES.len(), 2);
        assert_eq!(BOSS_AND_RAID_FAMILIES.len(), 3);

        for id in [
            "bat",
            "allay",
            "armadillo",
            "axolotl",
            "bee",
            "camel",
            "chicken",
            "cow",
            "dolphin",
            "equine",
            "feline",
            "fish",
            "fox",
            "frog",
            "goat",
            "golem",
            "happy_ghast",
            "nautilus",
            "panda",
            "parrot",
            "pig",
            "polar_bear",
            "rabbit",
            "sheep",
            "sniffer",
            "squid",
            "turtle",
            "wolf",
            "breeze",
            "creaking",
            "hoglin",
            "illager",
            "piglin",
            "skeleton",
            "spider",
            "warden",
            "zombie",
            "villager",
            "wandering_trader",
            "ender_dragon",
            "wither",
            "raid",
        ] {
            assert!(family_by_id(id).is_some(), "{id}");
        }
    }

    #[test]
    fn animal_capabilities_match_family_interfaces_visible_in_sources() {
        let axolotl = family_by_id("axolotl").unwrap();
        assert!(supports_interaction(axolotl, InteractionCapability::Bucket));
        assert_eq!(axolotl.mob_category, "axolotls");

        let wolf = family_by_id("wolf").unwrap();
        assert!(supports_interaction(wolf, InteractionCapability::Tame));
        assert!(supports_interaction(wolf, InteractionCapability::Breed));
        assert!(wolf.variant_backed);

        let camel = family_by_id("camel").unwrap();
        assert!(supports_interaction(camel, InteractionCapability::Ride));

        let sheep = family_by_id("sheep").unwrap();
        assert!(supports_interaction(sheep, InteractionCapability::Shear));
    }

    #[test]
    fn monster_npc_boss_and_raid_families_keep_source_categories_separate() {
        assert_eq!(family_by_id("breeze").unwrap().package, "monster/breeze");
        assert!(family_by_id("zombie").unwrap().converts_or_transforms);
        assert_eq!(
            family_by_id("villager").unwrap().family,
            MobSourceFamily::Npc
        );
        assert_eq!(
            family_by_id("ender_dragon").unwrap().family,
            MobSourceFamily::Boss
        );
        assert_eq!(family_by_id("raid").unwrap().family, MobSourceFamily::Raid);
    }

    #[test]
    fn runtime_state_applies_variant_taming_conversion_and_riding_guards() {
        let wolf = family_by_id("wolf").unwrap();
        let mut state = MobRuntimeState::new("wolf");
        state.set_variant(wolf, "black").unwrap();
        state.tame(wolf, "owner".to_string()).unwrap();
        assert_eq!(state.variant, Some("black"));
        assert_eq!(state.owner_uuid.as_deref(), Some("owner"));
        assert!(state.persistent);

        let zombie = family_by_id("zombie").unwrap();
        let mut zombie_state = MobRuntimeState::new("zombie");
        zombie_state.start_conversion(zombie, 2).unwrap();
        assert!(!zombie_state.tick_conversion());
        assert!(zombie_state.tick_conversion());
        assert_eq!(zombie_state.conversion_timer, None);

        let camel = family_by_id("camel").unwrap();
        let mut camel_state = MobRuntimeState::new("camel");
        assert!(camel_state.add_passenger(camel, 5));
        assert!(!camel_state.add_passenger(camel, 5));
        assert!(!camel_state.add_passenger(wolf, 7));
    }

    #[test]
    fn wolf_interactions_match_java_tame_collar_armor_and_owner_targets() {
        let owner = "00000000-0000-0000-0000-000000000001";
        let other = "00000000-0000-0000-0000-000000000002";
        let mut wolf = WolfRuntimeState::new();

        assert_wolf_taming_matches_java(&mut wolf, owner);
        assert_wolf_collar_dyeing_matches_java(&mut wolf, owner, other);
        assert_wolf_armor_handling_matches_java(&mut wolf, owner, other);
        assert_wolf_owner_targets_match_java(&mut wolf, owner, other);
    }

    fn assert_wolf_taming_matches_java(wolf: &mut WolfRuntimeState, owner: &'static str) {
        assert_eq!(wolf.collar_color, DyeColor::Red);
        assert_eq!(
            wolf.try_tame_with_bone(owner, WolfTameRoll::Failure),
            WolfInteractionOutcome::ConsumedBoneTameFailure
        );
        assert!(!wolf.is_tame());
        assert_eq!(
            wolf.try_tame_with_bone(owner, WolfTameRoll::Success),
            WolfInteractionOutcome::ConsumedBoneTameSuccess
        );
        assert!(wolf.is_tame());
        assert_eq!(wolf.max_health, 40);
        assert_eq!(wolf.health, 40);
        assert!(wolf.ordered_to_sit);
    }

    fn assert_wolf_collar_dyeing_matches_java(
        wolf: &mut WolfRuntimeState,
        owner: &'static str,
        other: &'static str,
    ) {
        assert_eq!(
            wolf.dye_collar(other, DyeColor::Blue),
            WolfInteractionOutcome::NoAction
        );
        assert_eq!(
            wolf.dye_collar(owner, DyeColor::Blue),
            WolfInteractionOutcome::DyedCollar
        );
        assert_eq!(wolf.collar_color, DyeColor::Blue);
        assert_eq!(
            wolf.dye_collar(owner, DyeColor::Blue),
            WolfInteractionOutcome::NoAction
        );
    }

    fn assert_wolf_armor_handling_matches_java(
        wolf: &mut WolfRuntimeState,
        owner: &'static str,
        other: &'static str,
    ) {
        assert_eq!(
            wolf.equip_body_armor(other, "minecraft:wolf_armor", 64),
            WolfInteractionOutcome::NoAction
        );
        assert_eq!(
            wolf_equipment_slot_for_item("minecraft:wolf_armor"),
            Some(EquipmentSlot::Body)
        );
        assert_eq!(
            wolf.equip_body_armor(owner, "minecraft:wolf_armor", 64),
            WolfInteractionOutcome::EquippedArmor
        );
        assert_eq!(
            wolf.equip_body_armor(owner, "minecraft:wolf_armor", 64),
            WolfInteractionOutcome::NoAction
        );

        wolf.body_armor.as_mut().unwrap().damage = 16;
        assert_eq!(
            wolf.repair_body_armor(owner, "minecraft:armadillo_scute"),
            WolfInteractionOutcome::RepairedArmor
        );
        assert_eq!(wolf.body_armor.as_ref().unwrap().damage, 8);
    }

    fn assert_wolf_owner_targets_match_java(
        wolf: &mut WolfRuntimeState,
        owner: &'static str,
        other: &'static str,
    ) {
        assert_eq!(
            wolf.toggle_sitting(owner),
            WolfInteractionOutcome::ToggledSitting
        );
        assert!(!wolf.ordered_to_sit);
        assert!(wolf.owner_was_hurt_by(owner, 42));
        assert_eq!(wolf.target_entity_id, Some(42));
        assert!(wolf.hurt_owner_target(owner, 43));
        assert_eq!(wolf.target_entity_id, Some(43));
        assert!(!wolf.owner_was_hurt_by(other, 99));
    }

    #[test]
    fn wolf_rejects_java_guarded_interactions() {
        let owner = "owner";
        let mut wild = WolfRuntimeState::new();
        wild.angry = true;
        assert_eq!(
            wild.try_tame_with_bone(owner, WolfTameRoll::Success),
            WolfInteractionOutcome::NoAction
        );

        let mut baby = WolfRuntimeState::new();
        baby.try_tame_with_bone(owner, WolfTameRoll::Success);
        baby.baby = true;
        assert_eq!(
            baby.equip_body_armor(owner, "minecraft:wolf_armor", 64),
            WolfInteractionOutcome::NoAction
        );

        let mut hurt = WolfRuntimeState::new();
        hurt.try_tame_with_bone(owner, WolfTameRoll::Success);
        hurt.health = 20;
        assert_eq!(
            hurt.feed("minecraft:bone"),
            WolfInteractionOutcome::NoAction
        );
        assert_eq!(
            hurt.feed("minecraft:cooked_beef"),
            WolfInteractionOutcome::Fed
        );
        assert_eq!(hurt.health, 22);
        assert!(!hurt.owner_was_hurt_by(owner, 7));
    }

    #[test]
    fn checklist_mob_family_surface_is_represented() {
        for family in [
            "ambient mobs",
            "animals",
            "water mobs",
            "NPCs",
            "monsters",
            "bosses",
            "raids",
            "variants",
            "allay",
            "armadillo",
            "axolotl",
            "bee",
            "camel",
            "chicken",
            "cow",
            "dolphin",
            "equine",
            "feline",
            "fish",
            "fox",
            "frog",
            "goat",
            "golem",
            "happy ghast",
            "nautilus",
            "panda",
            "parrot",
            "pig",
            "polar bear",
            "rabbit",
            "sheep",
            "sniffer",
            "squid",
            "turtle",
            "wolf",
            "breeze",
            "creaking",
            "hoglin",
            "illager",
            "piglin",
            "skeleton",
            "spider",
            "warden",
            "zombie",
            "villagers",
            "wandering traders",
        ] {
            assert!(MOB_FAMILY_CHECKLIST_SURFACE.contains(&family), "{family}");
        }
    }
}
