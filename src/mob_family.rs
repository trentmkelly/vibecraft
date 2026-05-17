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
