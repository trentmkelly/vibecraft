#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnRule {
    Fallback,
    BiomeTag(&'static str),
    Biome(&'static str),
    StructureOrMoon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantModel {
    Normal,
    Warm,
    Cold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleVariant {
    pub id: &'static str,
    pub adult_texture: String,
    pub baby_texture: Option<String>,
    pub model: VariantModel,
    pub spawn_rule: SpawnRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WolfVariantDef {
    pub id: &'static str,
    pub wild_texture: String,
    pub tame_texture: String,
    pub angry_texture: String,
    pub baby_texture: String,
    pub tame_baby_texture: String,
    pub angry_baby_texture: String,
    pub spawn_rule: SpawnRule,
}

pub fn cat_variants_26_1_2() -> Vec<SimpleVariant> {
    [
        (
            "tabby",
            "entity/cat/cat_tabby",
            "entity/cat/cat_tabby_baby",
            SpawnRule::Fallback,
        ),
        (
            "black",
            "entity/cat/cat_black",
            "entity/cat/cat_black_baby",
            SpawnRule::Fallback,
        ),
        (
            "red",
            "entity/cat/cat_red",
            "entity/cat/cat_red_baby",
            SpawnRule::Fallback,
        ),
        (
            "siamese",
            "entity/cat/cat_siamese",
            "entity/cat/cat_siamese_baby",
            SpawnRule::Fallback,
        ),
        (
            "british_shorthair",
            "entity/cat/cat_british_shorthair",
            "entity/cat/cat_british_shorthair_baby",
            SpawnRule::Fallback,
        ),
        (
            "calico",
            "entity/cat/cat_calico",
            "entity/cat/cat_calico_baby",
            SpawnRule::Fallback,
        ),
        (
            "persian",
            "entity/cat/cat_persian",
            "entity/cat/cat_persian_baby",
            SpawnRule::Fallback,
        ),
        (
            "ragdoll",
            "entity/cat/cat_ragdoll",
            "entity/cat/cat_ragdoll_baby",
            SpawnRule::Fallback,
        ),
        (
            "white",
            "entity/cat/cat_white",
            "entity/cat/cat_white_baby",
            SpawnRule::Fallback,
        ),
        (
            "jellie",
            "entity/cat/cat_jellie",
            "entity/cat/cat_jellie_baby",
            SpawnRule::Fallback,
        ),
        (
            "all_black",
            "entity/cat/cat_all_black",
            "entity/cat/cat_all_black_baby",
            SpawnRule::StructureOrMoon,
        ),
    ]
    .into_iter()
    .map(
        |(id, adult_texture, baby_texture, spawn_rule)| SimpleVariant {
            id,
            adult_texture: adult_texture.to_string(),
            baby_texture: Some(baby_texture.to_string()),
            model: VariantModel::Normal,
            spawn_rule,
        },
    )
    .collect()
}

pub fn chicken_variants_26_1_2() -> Vec<SimpleVariant> {
    temperature_farm_variants("chicken", VariantModel::Normal, VariantModel::Cold)
}

pub fn cow_variants_26_1_2() -> Vec<SimpleVariant> {
    temperature_farm_variants("cow", VariantModel::Warm, VariantModel::Cold)
}

pub fn pig_variants_26_1_2() -> Vec<SimpleVariant> {
    temperature_farm_variants("pig", VariantModel::Normal, VariantModel::Cold)
}

pub fn frog_variants_26_1_2() -> Vec<SimpleVariant> {
    vec![
        SimpleVariant {
            id: "temperate",
            adult_texture: "entity/frog/frog_temperate".to_string(),
            baby_texture: None,
            model: VariantModel::Normal,
            spawn_rule: SpawnRule::Fallback,
        },
        SimpleVariant {
            id: "warm",
            adult_texture: "entity/frog/frog_warm".to_string(),
            baby_texture: None,
            model: VariantModel::Normal,
            spawn_rule: SpawnRule::BiomeTag("minecraft:spawns_warm_variant_frogs"),
        },
        SimpleVariant {
            id: "cold",
            adult_texture: "entity/frog/frog_cold".to_string(),
            baby_texture: None,
            model: VariantModel::Normal,
            spawn_rule: SpawnRule::BiomeTag("minecraft:spawns_cold_variant_frogs"),
        },
    ]
}

pub fn zombie_nautilus_variants_26_1_2() -> Vec<SimpleVariant> {
    vec![
        SimpleVariant {
            id: "temperate",
            adult_texture: "entity/nautilus/zombie_nautilus".to_string(),
            baby_texture: None,
            model: VariantModel::Normal,
            spawn_rule: SpawnRule::Fallback,
        },
        SimpleVariant {
            id: "warm",
            adult_texture: "entity/nautilus/zombie_nautilus_coral".to_string(),
            baby_texture: None,
            model: VariantModel::Warm,
            spawn_rule: SpawnRule::BiomeTag("minecraft:spawns_coral_variant_zombie_nautilus"),
        },
    ]
}

pub fn wolf_variants_26_1_2() -> Vec<WolfVariantDef> {
    [
        ("pale", "wolf", SpawnRule::Fallback),
        (
            "spotted",
            "wolf_spotted",
            SpawnRule::BiomeTag("minecraft:is_savanna"),
        ),
        ("snowy", "wolf_snowy", SpawnRule::Biome("minecraft:grove")),
        (
            "black",
            "wolf_black",
            SpawnRule::Biome("minecraft:old_growth_pine_taiga"),
        ),
        (
            "ashen",
            "wolf_ashen",
            SpawnRule::Biome("minecraft:snowy_taiga"),
        ),
        (
            "rusty",
            "wolf_rusty",
            SpawnRule::BiomeTag("minecraft:is_jungle"),
        ),
        ("woods", "wolf_woods", SpawnRule::Biome("minecraft:forest")),
        (
            "chestnut",
            "wolf_chestnut",
            SpawnRule::Biome("minecraft:old_growth_spruce_taiga"),
        ),
        (
            "striped",
            "wolf_striped",
            SpawnRule::BiomeTag("minecraft:is_badlands"),
        ),
    ]
    .into_iter()
    .map(|(id, file, spawn_rule)| WolfVariantDef {
        id,
        wild_texture: format!("entity/wolf/{file}"),
        tame_texture: format!("entity/wolf/{file}_tame"),
        angry_texture: format!("entity/wolf/{file}_angry"),
        baby_texture: format!("entity/wolf/{file}_baby"),
        tame_baby_texture: format!("entity/wolf/{file}_tame_baby"),
        angry_baby_texture: format!("entity/wolf/{file}_angry_baby"),
        spawn_rule,
    })
    .collect()
}

fn temperature_farm_variants(
    entity: &'static str,
    warm_model: VariantModel,
    cold_model: VariantModel,
) -> Vec<SimpleVariant> {
    [
        (
            "temperate",
            "temperate",
            VariantModel::Normal,
            SpawnRule::Fallback,
        ),
        (
            "warm",
            "warm",
            warm_model,
            SpawnRule::BiomeTag("minecraft:spawns_warm_variant_farm_animals"),
        ),
        (
            "cold",
            "cold",
            cold_model,
            SpawnRule::BiomeTag("minecraft:spawns_cold_variant_farm_animals"),
        ),
    ]
    .into_iter()
    .map(|(id, suffix, model, spawn_rule)| SimpleVariant {
        id,
        adult_texture: format!("entity/{entity}/{entity}_{suffix}"),
        baby_texture: Some(format!("entity/{entity}/{entity}_{suffix}_baby")),
        model,
        spawn_rule,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animal_variant_counts_and_defaults_match_26_1_2_bootstrap() {
        assert_eq!(cat_variants_26_1_2().len(), 11);
        assert_eq!(chicken_variants_26_1_2().len(), 3);
        assert_eq!(cow_variants_26_1_2().len(), 3);
        assert_eq!(frog_variants_26_1_2().len(), 3);
        assert_eq!(pig_variants_26_1_2().len(), 3);
        assert_eq!(wolf_variants_26_1_2().len(), 9);
        assert_eq!(zombie_nautilus_variants_26_1_2().len(), 2);

        assert_eq!(cat_variants_26_1_2()[0].id, "tabby");
        assert_eq!(chicken_variants_26_1_2()[0].id, "temperate");
        assert_eq!(cow_variants_26_1_2()[0].id, "temperate");
        assert_eq!(frog_variants_26_1_2()[0].id, "temperate");
        assert_eq!(pig_variants_26_1_2()[0].id, "temperate");
        assert_eq!(wolf_variants_26_1_2()[0].id, "pale");
        assert_eq!(zombie_nautilus_variants_26_1_2()[0].id, "temperate");
    }

    #[test]
    fn animal_variant_assets_and_spawn_rules_match_bootstrap_sources() {
        let cats = cat_variants_26_1_2();
        let all_black = cats
            .iter()
            .find(|variant| variant.id == "all_black")
            .unwrap();
        assert_eq!(all_black.adult_texture, "entity/cat/cat_all_black");
        assert_eq!(all_black.spawn_rule, SpawnRule::StructureOrMoon);

        let chicken_cold = chicken_variants_26_1_2()
            .into_iter()
            .find(|variant| variant.id == "cold")
            .unwrap();
        assert_eq!(chicken_cold.model, VariantModel::Cold);
        assert_eq!(
            chicken_cold.spawn_rule,
            SpawnRule::BiomeTag("minecraft:spawns_cold_variant_farm_animals")
        );

        let cow_warm = cow_variants_26_1_2()
            .into_iter()
            .find(|variant| variant.id == "warm")
            .unwrap();
        assert_eq!(cow_warm.model, VariantModel::Warm);

        let wolf_black = wolf_variants_26_1_2()
            .into_iter()
            .find(|variant| variant.id == "black")
            .unwrap();
        assert_eq!(wolf_black.tame_texture, "entity/wolf/wolf_black_tame");
        assert_eq!(
            wolf_black.spawn_rule,
            SpawnRule::Biome("minecraft:old_growth_pine_taiga")
        );

        let nautilus_warm = zombie_nautilus_variants_26_1_2()
            .into_iter()
            .find(|variant| variant.id == "warm")
            .unwrap();
        assert_eq!(
            nautilus_warm.adult_texture,
            "entity/nautilus/zombie_nautilus_coral"
        );
        assert_eq!(
            nautilus_warm.spawn_rule,
            SpawnRule::BiomeTag("minecraft:spawns_coral_variant_zombie_nautilus")
        );
    }
}
