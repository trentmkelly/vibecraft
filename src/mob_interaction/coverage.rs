
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobInteractionCoverage {
    pub source: &'static str,
    pub covered_rules: &'static [&'static str],
}

pub const MOB_INTERACTION_COVERAGE: &[MobInteractionCoverage] = &[
    MobInteractionCoverage {
        source: "AgeableMob",
        covered_rules: &["ageable", "age lock", "baby variants"],
    },
    MobInteractionCoverage {
        source: "Animal",
        covered_rules: &["breedable", "love mode", "feeding"],
    },
    MobInteractionCoverage {
        source: "TamableAnimal",
        covered_rules: &["tameable", "owner", "sitting", "teleport"],
    },
    MobInteractionCoverage {
        source: "AbstractHorse",
        covered_rules: &["rideable", "saddle", "temper"],
    },
    MobInteractionCoverage {
        source: "Bucketable",
        covered_rules: &["bucketable", "bucket save/load"],
    },
    MobInteractionCoverage {
        source: "Shearable/MushroomCow",
        covered_rules: &["shearable", "transformation"],
    },
    MobInteractionCoverage {
        source: "Variant bootstraps",
        covered_rules: &["variant"],
    },
    MobInteractionCoverage {
        source: "AbstractVillager",
        covered_rules: &["trading"],
    },
    MobInteractionCoverage {
        source: "NeutralMob",
        covered_rules: &["anger"],
    },
    MobInteractionCoverage {
        source: "ConversionType",
        covered_rules: &["conversion", "transformation"],
    },
];

