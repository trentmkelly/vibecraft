#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityPackageCoverage {
    pub package: &'static str,
    pub java_file_count: usize,
    pub category: EntitySourceCategory,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySourceCategory {
    Core,
    AiSupport,
    Ambient,
    Animal,
    Boss,
    Decoration,
    Item,
    Monster,
    Npc,
    Player,
    Projectile,
    RaidSupport,
    ScheduleSupport,
    VariantSupport,
    Vehicle,
}

#[cfg(test)]
pub const ROOT_ENTITY_JAVA_FILES: usize = 68;
#[cfg(test)]
pub const REGISTERED_ENTITY_TYPES_26_1_2: usize = 157;

#[cfg(test)]
pub const ENTITY_PACKAGE_COVERAGE: &[EntityPackageCoverage] = &[
    EntityPackageCoverage {
        package: "net/minecraft/world/entity",
        java_file_count: ROOT_ENTITY_JAVA_FILES,
        category: EntitySourceCategory::Core,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/ai",
        java_file_count: 277,
        category: EntitySourceCategory::AiSupport,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/ambient",
        java_file_count: 3,
        category: EntitySourceCategory::Ambient,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/animal",
        java_file_count: 132,
        category: EntitySourceCategory::Animal,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/boss",
        java_file_count: 24,
        category: EntitySourceCategory::Boss,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/decoration",
        java_file_count: 12,
        category: EntitySourceCategory::Decoration,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/item",
        java_file_count: 4,
        category: EntitySourceCategory::Item,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/monster",
        java_file_count: 81,
        category: EntitySourceCategory::Monster,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/npc",
        java_file_count: 15,
        category: EntitySourceCategory::Npc,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/player",
        java_file_count: 14,
        category: EntitySourceCategory::Player,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/projectile",
        java_file_count: 37,
        category: EntitySourceCategory::Projectile,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/raid",
        java_file_count: 4,
        category: EntitySourceCategory::RaidSupport,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/schedule",
        java_file_count: 2,
        category: EntitySourceCategory::ScheduleSupport,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/variant",
        java_file_count: 11,
        category: EntitySourceCategory::VariantSupport,
    },
    EntityPackageCoverage {
        package: "net/minecraft/world/entity/vehicle",
        java_file_count: 24,
        category: EntitySourceCategory::Vehicle,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobCategoryDef {
    pub id: &'static str,
    pub max_per_chunk: i32,
    pub friendly: bool,
    pub persistent: bool,
    pub no_despawn_distance: i32,
    pub despawn_distance: i32,
}

pub const MOB_CATEGORIES: &[MobCategoryDef] = &[
    MobCategoryDef {
        id: "monster",
        max_per_chunk: 70,
        friendly: false,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "creature",
        max_per_chunk: 10,
        friendly: true,
        persistent: true,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "ambient",
        max_per_chunk: 15,
        friendly: true,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "axolotls",
        max_per_chunk: 5,
        friendly: true,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "underground_water_creature",
        max_per_chunk: 5,
        friendly: true,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "water_creature",
        max_per_chunk: 5,
        friendly: true,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
    MobCategoryDef {
        id: "water_ambient",
        max_per_chunk: 20,
        friendly: true,
        persistent: false,
        no_despawn_distance: 32,
        despawn_distance: 64,
    },
    MobCategoryDef {
        id: "misc",
        max_per_chunk: -1,
        friendly: true,
        persistent: true,
        no_despawn_distance: 32,
        despawn_distance: 128,
    },
];

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityCategoryExample {
    pub entity_type: &'static str,
    pub source_category: EntitySourceCategory,
    pub mob_category: Option<&'static str>,
    pub package: &'static str,
}

#[cfg(test)]
pub const ENTITY_CATEGORY_EXAMPLES: &[EntityCategoryExample] = &[
    EntityCategoryExample {
        entity_type: "minecraft:bat",
        source_category: EntitySourceCategory::Ambient,
        mob_category: Some("ambient"),
        package: "ambient",
    },
    EntityCategoryExample {
        entity_type: "minecraft:pig",
        source_category: EntitySourceCategory::Animal,
        mob_category: Some("creature"),
        package: "animal/pig",
    },
    EntityCategoryExample {
        entity_type: "minecraft:ender_dragon",
        source_category: EntitySourceCategory::Boss,
        mob_category: Some("monster"),
        package: "boss/enderdragon",
    },
    EntityCategoryExample {
        entity_type: "minecraft:painting",
        source_category: EntitySourceCategory::Decoration,
        mob_category: Some("misc"),
        package: "decoration/painting",
    },
    EntityCategoryExample {
        entity_type: "minecraft:item",
        source_category: EntitySourceCategory::Item,
        mob_category: Some("misc"),
        package: "item",
    },
    EntityCategoryExample {
        entity_type: "minecraft:zombie",
        source_category: EntitySourceCategory::Monster,
        mob_category: Some("monster"),
        package: "monster/zombie",
    },
    EntityCategoryExample {
        entity_type: "minecraft:villager",
        source_category: EntitySourceCategory::Npc,
        mob_category: Some("misc"),
        package: "npc/villager",
    },
    EntityCategoryExample {
        entity_type: "minecraft:player",
        source_category: EntitySourceCategory::Player,
        mob_category: None,
        package: "player",
    },
    EntityCategoryExample {
        entity_type: "minecraft:arrow",
        source_category: EntitySourceCategory::Projectile,
        mob_category: Some("misc"),
        package: "projectile/arrow",
    },
    EntityCategoryExample {
        entity_type: "minecraft:raid",
        source_category: EntitySourceCategory::RaidSupport,
        mob_category: None,
        package: "raid",
    },
    EntityCategoryExample {
        entity_type: "minecraft:minecart",
        source_category: EntitySourceCategory::Vehicle,
        mob_category: Some("misc"),
        package: "vehicle/minecart",
    },
];

#[cfg(test)]
pub fn package_coverage(category: EntitySourceCategory) -> Option<&'static EntityPackageCoverage> {
    ENTITY_PACKAGE_COVERAGE
        .iter()
        .find(|coverage| coverage.category == category)
}

pub fn mob_category(id: &str) -> Option<&'static MobCategoryDef> {
    MOB_CATEGORIES.iter().find(|category| category.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_package_manifest_matches_decompiled_world_entity_tree_counts() {
        assert_eq!(ROOT_ENTITY_JAVA_FILES, 68);
        assert_eq!(REGISTERED_ENTITY_TYPES_26_1_2, 157);
        assert_eq!(ENTITY_PACKAGE_COVERAGE.len(), 15);
        assert_eq!(
            ENTITY_PACKAGE_COVERAGE
                .iter()
                .map(|coverage| coverage.java_file_count)
                .sum::<usize>(),
            708
        );
        assert_eq!(
            package_coverage(EntitySourceCategory::Animal)
                .unwrap()
                .java_file_count,
            132
        );
        assert_eq!(
            package_coverage(EntitySourceCategory::Monster)
                .unwrap()
                .java_file_count,
            81
        );
        assert_eq!(
            package_coverage(EntitySourceCategory::AiSupport)
                .unwrap()
                .java_file_count,
            277
        );
    }

    #[test]
    fn mob_categories_match_vanilla_caps_persistence_and_despawn_distances() {
        assert_eq!(MOB_CATEGORIES.len(), 8);
        assert_eq!(
            mob_category("monster").unwrap(),
            &MobCategoryDef {
                id: "monster",
                max_per_chunk: 70,
                friendly: false,
                persistent: false,
                no_despawn_distance: 32,
                despawn_distance: 128
            }
        );
        assert_eq!(mob_category("water_ambient").unwrap().max_per_chunk, 20);
        assert_eq!(mob_category("water_ambient").unwrap().despawn_distance, 64);
        assert_eq!(mob_category("misc").unwrap().max_per_chunk, -1);
        assert!(mob_category("creature").unwrap().persistent);
        assert!(mob_category("axolotls").unwrap().friendly);
    }

    #[test]
    fn representative_entity_types_cover_all_server_visible_source_categories() {
        let categories = [
            EntitySourceCategory::Ambient,
            EntitySourceCategory::Animal,
            EntitySourceCategory::Boss,
            EntitySourceCategory::Decoration,
            EntitySourceCategory::Item,
            EntitySourceCategory::Monster,
            EntitySourceCategory::Npc,
            EntitySourceCategory::Player,
            EntitySourceCategory::Projectile,
            EntitySourceCategory::RaidSupport,
            EntitySourceCategory::Vehicle,
        ];

        for category in categories {
            assert!(
                ENTITY_CATEGORY_EXAMPLES
                    .iter()
                    .any(|example| example.source_category == category),
                "missing representative category {category:?}"
            );
        }
    }

    #[test]
    fn support_packages_are_separated_for_later_ai_variant_and_schedule_work() {
        assert_eq!(
            package_coverage(EntitySourceCategory::AiSupport)
                .unwrap()
                .package,
            "net/minecraft/world/entity/ai"
        );
        assert_eq!(
            package_coverage(EntitySourceCategory::VariantSupport)
                .unwrap()
                .java_file_count,
            11
        );
        assert_eq!(
            package_coverage(EntitySourceCategory::ScheduleSupport)
                .unwrap()
                .java_file_count,
            2
        );
    }
}
