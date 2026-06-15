#![allow(dead_code)]

use crate::block_entity::BrewingRecipe;
use crate::status_effect::status_effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlchemyEffectInstance {
    pub effect: &'static str,
    pub duration_ticks: i32,
    pub amplifier: i32,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionDef {
    pub id: &'static str,
    pub name: &'static str,
    pub effects: &'static [AlchemyEffectInstance],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionContentsModel {
    pub potion: Option<&'static str>,
    pub custom_color: Option<i32>,
    pub custom_effects: Vec<AlchemyEffectInstance>,
    pub custom_name: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionStack {
    pub item: &'static str,
    pub potion: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotionMix {
    pub from: &'static str,
    pub ingredient: &'static str,
    pub to: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerMix {
    pub from: &'static str,
    pub ingredient: &'static str,
    pub to: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionBrewingModel {
    pub containers: Vec<&'static str>,
    pub potion_mixes: Vec<PotionMix>,
    pub container_mixes: Vec<ContainerMix>,
}

pub const BREWING_TIME_SECONDS: i32 = 20;
pub const BASE_POTION_COLOR: i32 = -13083194;

const fn eff(effect: &'static str, duration_ticks: i32) -> AlchemyEffectInstance {
    AlchemyEffectInstance {
        effect,
        duration_ticks,
        amplifier: 0,
        visible: true,
    }
}

const fn amp(
    effect: &'static str,
    duration_ticks: i32,
    amplifier: i32,
) -> AlchemyEffectInstance {
    AlchemyEffectInstance {
        effect,
        duration_ticks,
        amplifier,
        visible: true,
    }
}

const NIGHT_VISION: &[AlchemyEffectInstance] = &[eff("minecraft:night_vision", 3600)];
const LONG_NIGHT_VISION: &[AlchemyEffectInstance] = &[eff("minecraft:night_vision", 9600)];
const INVISIBILITY: &[AlchemyEffectInstance] = &[eff("minecraft:invisibility", 3600)];
const LONG_INVISIBILITY: &[AlchemyEffectInstance] = &[eff("minecraft:invisibility", 9600)];
const LEAPING: &[AlchemyEffectInstance] = &[eff("minecraft:jump_boost", 3600)];
const LONG_LEAPING: &[AlchemyEffectInstance] = &[eff("minecraft:jump_boost", 9600)];
const STRONG_LEAPING: &[AlchemyEffectInstance] = &[amp("minecraft:jump_boost", 1800, 1)];
const FIRE_RESISTANCE: &[AlchemyEffectInstance] = &[eff("minecraft:fire_resistance", 3600)];
const LONG_FIRE_RESISTANCE: &[AlchemyEffectInstance] =
    &[eff("minecraft:fire_resistance", 9600)];
const SWIFTNESS: &[AlchemyEffectInstance] = &[eff("minecraft:speed", 3600)];
const LONG_SWIFTNESS: &[AlchemyEffectInstance] = &[eff("minecraft:speed", 9600)];
const STRONG_SWIFTNESS: &[AlchemyEffectInstance] = &[amp("minecraft:speed", 1800, 1)];
const SLOWNESS: &[AlchemyEffectInstance] = &[eff("minecraft:slowness", 1800)];
const LONG_SLOWNESS: &[AlchemyEffectInstance] = &[eff("minecraft:slowness", 4800)];
const STRONG_SLOWNESS: &[AlchemyEffectInstance] = &[amp("minecraft:slowness", 400, 3)];
const TURTLE_MASTER: &[AlchemyEffectInstance] = &[
    amp("minecraft:slowness", 400, 3),
    amp("minecraft:resistance", 400, 2),
];
const LONG_TURTLE_MASTER: &[AlchemyEffectInstance] = &[
    amp("minecraft:slowness", 800, 3),
    amp("minecraft:resistance", 800, 2),
];
const STRONG_TURTLE_MASTER: &[AlchemyEffectInstance] = &[
    amp("minecraft:slowness", 400, 5),
    amp("minecraft:resistance", 400, 3),
];
const WATER_BREATHING: &[AlchemyEffectInstance] = &[eff("minecraft:water_breathing", 3600)];
const LONG_WATER_BREATHING: &[AlchemyEffectInstance] =
    &[eff("minecraft:water_breathing", 9600)];
const HEALING: &[AlchemyEffectInstance] = &[eff("minecraft:instant_health", 1)];
const STRONG_HEALING: &[AlchemyEffectInstance] =
    &[amp("minecraft:instant_health", 1, 1)];
const HARMING: &[AlchemyEffectInstance] = &[eff("minecraft:instant_damage", 1)];
const STRONG_HARMING: &[AlchemyEffectInstance] =
    &[amp("minecraft:instant_damage", 1, 1)];
const POISON: &[AlchemyEffectInstance] = &[eff("minecraft:poison", 900)];
const LONG_POISON: &[AlchemyEffectInstance] = &[eff("minecraft:poison", 1800)];
const STRONG_POISON: &[AlchemyEffectInstance] = &[amp("minecraft:poison", 432, 1)];
const REGENERATION: &[AlchemyEffectInstance] = &[eff("minecraft:regeneration", 900)];
const LONG_REGENERATION: &[AlchemyEffectInstance] = &[eff("minecraft:regeneration", 1800)];
const STRONG_REGENERATION: &[AlchemyEffectInstance] =
    &[amp("minecraft:regeneration", 450, 1)];
const STRENGTH: &[AlchemyEffectInstance] = &[eff("minecraft:strength", 3600)];
const LONG_STRENGTH: &[AlchemyEffectInstance] = &[eff("minecraft:strength", 9600)];
const STRONG_STRENGTH: &[AlchemyEffectInstance] = &[amp("minecraft:strength", 1800, 1)];
const WEAKNESS: &[AlchemyEffectInstance] = &[eff("minecraft:weakness", 1800)];
const LONG_WEAKNESS: &[AlchemyEffectInstance] = &[eff("minecraft:weakness", 4800)];
const LUCK: &[AlchemyEffectInstance] = &[eff("minecraft:luck", 6000)];
const SLOW_FALLING: &[AlchemyEffectInstance] = &[eff("minecraft:slow_falling", 1800)];
const LONG_SLOW_FALLING: &[AlchemyEffectInstance] =
    &[eff("minecraft:slow_falling", 4800)];
const WIND_CHARGED: &[AlchemyEffectInstance] = &[eff("minecraft:wind_charged", 3600)];
const WEAVING: &[AlchemyEffectInstance] = &[eff("minecraft:weaving", 3600)];
const OOZING: &[AlchemyEffectInstance] = &[eff("minecraft:oozing", 3600)];
const INFESTED: &[AlchemyEffectInstance] = &[eff("minecraft:infested", 3600)];

pub const POTIONS: &[PotionDef] = &[
    PotionDef {
        id: "water",
        name: "water",
        effects: &[],
    },
    PotionDef {
        id: "mundane",
        name: "mundane",
        effects: &[],
    },
    PotionDef {
        id: "thick",
        name: "thick",
        effects: &[],
    },
    PotionDef {
        id: "awkward",
        name: "awkward",
        effects: &[],
    },
    PotionDef {
        id: "night_vision",
        name: "night_vision",
        effects: NIGHT_VISION,
    },
    PotionDef {
        id: "long_night_vision",
        name: "night_vision",
        effects: LONG_NIGHT_VISION,
    },
    PotionDef {
        id: "invisibility",
        name: "invisibility",
        effects: INVISIBILITY,
    },
    PotionDef {
        id: "long_invisibility",
        name: "invisibility",
        effects: LONG_INVISIBILITY,
    },
    PotionDef {
        id: "leaping",
        name: "leaping",
        effects: LEAPING,
    },
    PotionDef {
        id: "long_leaping",
        name: "leaping",
        effects: LONG_LEAPING,
    },
    PotionDef {
        id: "strong_leaping",
        name: "leaping",
        effects: STRONG_LEAPING,
    },
    PotionDef {
        id: "fire_resistance",
        name: "fire_resistance",
        effects: FIRE_RESISTANCE,
    },
    PotionDef {
        id: "long_fire_resistance",
        name: "fire_resistance",
        effects: LONG_FIRE_RESISTANCE,
    },
    PotionDef {
        id: "swiftness",
        name: "swiftness",
        effects: SWIFTNESS,
    },
    PotionDef {
        id: "long_swiftness",
        name: "swiftness",
        effects: LONG_SWIFTNESS,
    },
    PotionDef {
        id: "strong_swiftness",
        name: "swiftness",
        effects: STRONG_SWIFTNESS,
    },
    PotionDef {
        id: "slowness",
        name: "slowness",
        effects: SLOWNESS,
    },
    PotionDef {
        id: "long_slowness",
        name: "slowness",
        effects: LONG_SLOWNESS,
    },
    PotionDef {
        id: "strong_slowness",
        name: "slowness",
        effects: STRONG_SLOWNESS,
    },
    PotionDef {
        id: "turtle_master",
        name: "turtle_master",
        effects: TURTLE_MASTER,
    },
    PotionDef {
        id: "long_turtle_master",
        name: "turtle_master",
        effects: LONG_TURTLE_MASTER,
    },
    PotionDef {
        id: "strong_turtle_master",
        name: "turtle_master",
        effects: STRONG_TURTLE_MASTER,
    },
    PotionDef {
        id: "water_breathing",
        name: "water_breathing",
        effects: WATER_BREATHING,
    },
    PotionDef {
        id: "long_water_breathing",
        name: "water_breathing",
        effects: LONG_WATER_BREATHING,
    },
    PotionDef {
        id: "healing",
        name: "healing",
        effects: HEALING,
    },
    PotionDef {
        id: "strong_healing",
        name: "healing",
        effects: STRONG_HEALING,
    },
    PotionDef {
        id: "harming",
        name: "harming",
        effects: HARMING,
    },
    PotionDef {
        id: "strong_harming",
        name: "harming",
        effects: STRONG_HARMING,
    },
    PotionDef {
        id: "poison",
        name: "poison",
        effects: POISON,
    },
    PotionDef {
        id: "long_poison",
        name: "poison",
        effects: LONG_POISON,
    },
    PotionDef {
        id: "strong_poison",
        name: "poison",
        effects: STRONG_POISON,
    },
    PotionDef {
        id: "regeneration",
        name: "regeneration",
        effects: REGENERATION,
    },
    PotionDef {
        id: "long_regeneration",
        name: "regeneration",
        effects: LONG_REGENERATION,
    },
    PotionDef {
        id: "strong_regeneration",
        name: "regeneration",
        effects: STRONG_REGENERATION,
    },
    PotionDef {
        id: "strength",
        name: "strength",
        effects: STRENGTH,
    },
    PotionDef {
        id: "long_strength",
        name: "strength",
        effects: LONG_STRENGTH,
    },
    PotionDef {
        id: "strong_strength",
        name: "strength",
        effects: STRONG_STRENGTH,
    },
    PotionDef {
        id: "weakness",
        name: "weakness",
        effects: WEAKNESS,
    },
    PotionDef {
        id: "long_weakness",
        name: "weakness",
        effects: LONG_WEAKNESS,
    },
    PotionDef {
        id: "luck",
        name: "luck",
        effects: LUCK,
    },
    PotionDef {
        id: "slow_falling",
        name: "slow_falling",
        effects: SLOW_FALLING,
    },
    PotionDef {
        id: "long_slow_falling",
        name: "slow_falling",
        effects: LONG_SLOW_FALLING,
    },
    PotionDef {
        id: "wind_charged",
        name: "wind_charged",
        effects: WIND_CHARGED,
    },
    PotionDef {
        id: "weaving",
        name: "weaving",
        effects: WEAVING,
    },
    PotionDef {
        id: "oozing",
        name: "oozing",
        effects: OOZING,
    },
    PotionDef {
        id: "infested",
        name: "infested",
        effects: INFESTED,
    },
];

pub const VANILLA_CONTAINER_MIXES: &[ContainerMix] = &[
    ContainerMix {
        from: "minecraft:potion",
        ingredient: "minecraft:gunpowder",
        to: "minecraft:splash_potion",
    },
    ContainerMix {
        from: "minecraft:splash_potion",
        ingredient: "minecraft:dragon_breath",
        to: "minecraft:lingering_potion",
    },
];

pub const VANILLA_POTION_MIXES: &[PotionMix] = &[
    PotionMix {
        from: "water",
        ingredient: "minecraft:glowstone_dust",
        to: "thick",
    },
    PotionMix {
        from: "water",
        ingredient: "minecraft:redstone",
        to: "mundane",
    },
    PotionMix {
        from: "water",
        ingredient: "minecraft:nether_wart",
        to: "awkward",
    },
    start_mundane("minecraft:breeze_rod"),
    start_awkw("minecraft:breeze_rod", "wind_charged"),
    start_mundane("minecraft:slime_block"),
    start_awkw("minecraft:slime_block", "oozing"),
    start_mundane("minecraft:stone"),
    start_awkw("minecraft:stone", "infested"),
    start_mundane("minecraft:cobweb"),
    start_awkw("minecraft:cobweb", "weaving"),
    mix("awkward", "minecraft:golden_carrot", "night_vision"),
    mix("night_vision", "minecraft:redstone", "long_night_vision"),
    mix("night_vision", "minecraft:fermented_spider_eye", "invisibility"),
    mix(
        "long_night_vision",
        "minecraft:fermented_spider_eye",
        "long_invisibility",
    ),
    mix("invisibility", "minecraft:redstone", "long_invisibility"),
    start_mundane("minecraft:magma_cream"),
    start_awkw("minecraft:magma_cream", "fire_resistance"),
    mix(
        "fire_resistance",
        "minecraft:redstone",
        "long_fire_resistance",
    ),
    start_mundane("minecraft:rabbit_foot"),
    start_awkw("minecraft:rabbit_foot", "leaping"),
    mix("leaping", "minecraft:redstone", "long_leaping"),
    mix("leaping", "minecraft:glowstone_dust", "strong_leaping"),
    mix("leaping", "minecraft:fermented_spider_eye", "slowness"),
    mix(
        "long_leaping",
        "minecraft:fermented_spider_eye",
        "long_slowness",
    ),
    mix("slowness", "minecraft:redstone", "long_slowness"),
    mix("slowness", "minecraft:glowstone_dust", "strong_slowness"),
    mix("awkward", "minecraft:turtle_helmet", "turtle_master"),
    mix("turtle_master", "minecraft:redstone", "long_turtle_master"),
    mix(
        "turtle_master",
        "minecraft:glowstone_dust",
        "strong_turtle_master",
    ),
    mix("swiftness", "minecraft:fermented_spider_eye", "slowness"),
    mix(
        "long_swiftness",
        "minecraft:fermented_spider_eye",
        "long_slowness",
    ),
    start_mundane("minecraft:sugar"),
    start_awkw("minecraft:sugar", "swiftness"),
    mix("swiftness", "minecraft:redstone", "long_swiftness"),
    mix("swiftness", "minecraft:glowstone_dust", "strong_swiftness"),
    mix("awkward", "minecraft:pufferfish", "water_breathing"),
    mix(
        "water_breathing",
        "minecraft:redstone",
        "long_water_breathing",
    ),
    start_mundane("minecraft:glistering_melon_slice"),
    start_awkw("minecraft:glistering_melon_slice", "healing"),
    mix("healing", "minecraft:glowstone_dust", "strong_healing"),
    mix("healing", "minecraft:fermented_spider_eye", "harming"),
    mix(
        "strong_healing",
        "minecraft:fermented_spider_eye",
        "strong_harming",
    ),
    mix("harming", "minecraft:glowstone_dust", "strong_harming"),
    mix("poison", "minecraft:fermented_spider_eye", "harming"),
    mix("long_poison", "minecraft:fermented_spider_eye", "harming"),
    mix(
        "strong_poison",
        "minecraft:fermented_spider_eye",
        "strong_harming",
    ),
    start_mundane("minecraft:spider_eye"),
    start_awkw("minecraft:spider_eye", "poison"),
    mix("poison", "minecraft:redstone", "long_poison"),
    mix("poison", "minecraft:glowstone_dust", "strong_poison"),
    start_mundane("minecraft:ghast_tear"),
    start_awkw("minecraft:ghast_tear", "regeneration"),
    mix("regeneration", "minecraft:redstone", "long_regeneration"),
    mix(
        "regeneration",
        "minecraft:glowstone_dust",
        "strong_regeneration",
    ),
    start_mundane("minecraft:blaze_powder"),
    start_awkw("minecraft:blaze_powder", "strength"),
    mix("strength", "minecraft:redstone", "long_strength"),
    mix("strength", "minecraft:glowstone_dust", "strong_strength"),
    mix("water", "minecraft:fermented_spider_eye", "weakness"),
    mix("weakness", "minecraft:redstone", "long_weakness"),
    mix("awkward", "minecraft:phantom_membrane", "slow_falling"),
    mix("slow_falling", "minecraft:redstone", "long_slow_falling"),
];

const fn mix(from: &'static str, ingredient: &'static str, to: &'static str) -> PotionMix {
    PotionMix {
        from,
        ingredient,
        to,
    }
}

const fn start_mundane(ingredient: &'static str) -> PotionMix {
    mix("water", ingredient, "mundane")
}

const fn start_awkw(ingredient: &'static str, to: &'static str) -> PotionMix {
    mix("awkward", ingredient, to)
}

pub fn potion(id: &str) -> Option<&'static PotionDef> {
    POTIONS.iter().find(|potion| potion.id == id)
}

pub fn is_potion_instant(potion_id: &str) -> bool {
    potion(potion_id).is_some_and(PotionDef::has_instant_effects)
}

impl PotionDef {
    pub fn has_instant_effects(&self) -> bool {
        self.effects.iter().any(|effect| {
            status_effect(effect.effect).is_some_and(|definition| definition.instantaneous)
        })
    }
}

impl PotionContentsModel {
    pub fn empty() -> Self {
        Self {
            potion: None,
            custom_color: None,
            custom_effects: Vec::new(),
            custom_name: None,
        }
    }

    pub fn new(potion: &'static str) -> Self {
        Self {
            potion: Some(potion),
            ..Self::empty()
        }
    }

    pub fn is(&self, potion_id: &str) -> bool {
        self.potion == Some(potion_id) && self.custom_effects.is_empty()
    }

    pub fn all_effects(&self) -> Vec<AlchemyEffectInstance> {
        let mut effects = self
            .potion
            .and_then(potion)
            .map(|potion| potion.effects.to_vec())
            .unwrap_or_default();
        effects.extend(self.custom_effects.iter().copied());
        effects
    }

    pub fn for_each_effect_scaled(&self, duration_scale: f32) -> Vec<AlchemyEffectInstance> {
        self.all_effects()
            .into_iter()
            .map(|mut effect| {
                effect.duration_ticks = ((effect.duration_ticks as f32) * duration_scale) as i32;
                effect
            })
            .collect()
    }

    pub fn with_potion(&self, potion: &'static str) -> Self {
        Self {
            potion: Some(potion),
            custom_color: self.custom_color,
            custom_effects: self.custom_effects.clone(),
            custom_name: self.custom_name,
        }
    }

    pub fn with_effect_added(&self, effect: AlchemyEffectInstance) -> Self {
        let mut custom_effects = self.custom_effects.clone();
        custom_effects.push(effect);
        Self {
            custom_effects,
            ..self.clone()
        }
    }

    pub fn get_color(&self) -> i32 {
        self.get_color_or(BASE_POTION_COLOR)
    }

    pub fn get_color_or(&self, default_color: i32) -> i32 {
        self.custom_color
            .or_else(|| color_optional(&self.all_effects()))
            .unwrap_or(default_color)
    }

    pub fn translation_key(&self, prefix: &str) -> String {
        let suffix = self
            .custom_name
            .or_else(|| self.potion.and_then(potion).map(|potion| potion.name))
            .unwrap_or("empty");
        format!("{prefix}{suffix}")
    }

    pub fn has_effects(&self) -> bool {
        !self.all_effects().is_empty()
    }
}

pub fn color_optional(effects: &[AlchemyEffectInstance]) -> Option<i32> {
    let mut red = 0;
    let mut green = 0;
    let mut blue = 0;
    let mut total_weight = 0;
    for effect in effects.iter().filter(|effect| effect.visible) {
        let color = status_effect(effect.effect)?.color;
        let weight = effect.amplifier + 1;
        red += weight * ((color >> 16) & 0xff) as i32;
        green += weight * ((color >> 8) & 0xff) as i32;
        blue += weight * (color & 0xff) as i32;
        total_weight += weight;
    }
    if total_weight == 0 {
        None
    } else {
        Some(
            (0xff << 24)
                | ((red / total_weight) << 16)
                | ((green / total_weight) << 8)
                | (blue / total_weight),
        )
    }
}

impl PotionBrewingModel {
    pub fn empty() -> Self {
        Self {
            containers: Vec::new(),
            potion_mixes: Vec::new(),
            container_mixes: Vec::new(),
        }
    }

    pub fn vanilla() -> Self {
        Self {
            containers: vec![
                "minecraft:potion",
                "minecraft:splash_potion",
                "minecraft:lingering_potion",
            ],
            potion_mixes: VANILLA_POTION_MIXES.to_vec(),
            container_mixes: VANILLA_CONTAINER_MIXES.to_vec(),
        }
    }

    pub fn is_ingredient(&self, ingredient: &str) -> bool {
        self.is_container_ingredient(ingredient) || self.is_potion_ingredient(ingredient)
    }

    pub fn is_container(&self, input: &PotionStack) -> bool {
        self.containers.contains(&input.item)
    }

    pub fn is_container_ingredient(&self, ingredient: &str) -> bool {
        self.container_mixes
            .iter()
            .any(|mix| mix.ingredient == ingredient)
    }

    pub fn is_potion_ingredient(&self, ingredient: &str) -> bool {
        self.potion_mixes
            .iter()
            .any(|mix| mix.ingredient == ingredient)
    }

    pub fn is_brewable_potion(&self, potion: &str) -> bool {
        self.potion_mixes.iter().any(|mix| mix.to == potion)
    }

    pub fn has_mix(&self, source: &PotionStack, ingredient: &str) -> bool {
        self.is_container(source)
            && (self.has_container_mix(source, ingredient)
                || self.has_potion_mix(source, ingredient))
    }

    pub fn has_container_mix(&self, source: &PotionStack, ingredient: &str) -> bool {
        self.container_mixes
            .iter()
            .any(|mix| source.item == mix.from && ingredient == mix.ingredient)
    }

    pub fn has_potion_mix(&self, source: &PotionStack, ingredient: &str) -> bool {
        let Some(potion) = source.potion else {
            return false;
        };
        self.potion_mixes
            .iter()
            .any(|mix| potion == mix.from && ingredient == mix.ingredient)
    }

    pub fn mix(&self, ingredient: &str, source: &PotionStack) -> PotionStack {
        let Some(potion) = source.potion else {
            return *source;
        };
        for mix in &self.container_mixes {
            if source.item == mix.from && ingredient == mix.ingredient {
                return PotionStack {
                    item: mix.to,
                    potion: Some(potion),
                };
            }
        }
        for mix in &self.potion_mixes {
            if potion == mix.from && ingredient == mix.ingredient {
                return PotionStack {
                    item: source.item,
                    potion: Some(mix.to),
                };
            }
        }
        *source
    }
}

pub fn is_brewing_ingredient(item_id: &str) -> bool {
    PotionBrewingModel::vanilla().is_ingredient(item_id)
}

pub fn vanilla_brewing_recipes() -> Vec<BrewingRecipe> {
    let brewing = PotionBrewingModel::vanilla();
    let mut recipes = Vec::with_capacity(
        brewing.potion_mixes.len() * brewing.containers.len() + brewing.container_mixes.len(),
    );
    for mix in &brewing.potion_mixes {
        for container in &brewing.containers {
            recipes.push(BrewingRecipe::new(
                container,
                mix.from,
                mix.ingredient,
                container,
                mix.to,
            ));
        }
    }
    for mix in &brewing.container_mixes {
        for potion in POTIONS.iter().map(|potion| potion.id) {
            recipes.push(BrewingRecipe::new(
                mix.from,
                potion,
                mix.ingredient,
                mix.to,
                potion,
            ));
        }
    }
    recipes
}

#[cfg(test)]
mod tests {
    use super::*;

    const POTION_SOURCE: &str = vibecraft_java_source!("/net/minecraft/world/item/alchemy/Potion.java");
    const POTIONS_SOURCE: &str = vibecraft_java_source!("/net/minecraft/world/item/alchemy/Potions.java");
    const POTION_CONTENTS_SOURCE: &str = vibecraft_java_source!("/net/minecraft/world/item/alchemy/PotionContents.java");
    const POTION_BREWING_SOURCE: &str = vibecraft_java_source!("/net/minecraft/world/item/alchemy/PotionBrewing.java");

    #[test]
    fn potion_registry_effects_names_and_instant_flag_match_java() {
        assert!(POTION_SOURCE.contains("public boolean hasInstantEffects()"));
        assert!(POTIONS_SOURCE.contains("new Potion(\"turtle_master\", new MobEffectInstance(MobEffects.SLOWNESS, 400, 3), new MobEffectInstance(MobEffects.RESISTANCE, 400, 2))"));
        assert_eq!(POTIONS.len(), 46);
        assert_eq!(POTIONS.first().map(|potion| potion.id), Some("water"));
        assert_eq!(POTIONS.last().map(|potion| potion.id), Some("infested"));
        assert_eq!(
            potion("long_night_vision").map(|potion| potion.name),
            Some("night_vision")
        );
        assert_eq!(
            potion("strong_turtle_master").map(|potion| potion.effects),
            Some(STRONG_TURTLE_MASTER)
        );
        assert!(potion("healing").is_some_and(PotionDef::has_instant_effects));
        assert!(!potion("regeneration").is_some_and(PotionDef::has_instant_effects));
    }

    #[test]
    fn potion_contents_effects_color_name_and_scaling_match_java() {
        assert!(POTION_CONTENTS_SOURCE.contains("BASE_POTION_COLOR = -13083194"));
        assert!(POTION_CONTENTS_SOURCE.contains("int amplifier = effect.getAmplifier() + 1"));
        assert!(POTION_CONTENTS_SOURCE.contains("effect.withScaledDuration(durationScale)"));

        let contents = PotionContentsModel::new("strong_swiftness").with_effect_added(amp(
            "minecraft:slowness",
            100,
            3,
        ));
        assert!(contents.has_effects());
        assert_eq!(contents.translation_key("item.minecraft.potion.effect."), "item.minecraft.potion.effect.swiftness");
        assert_eq!(
            contents.all_effects(),
            vec![
                amp("minecraft:speed", 1800, 1),
                amp("minecraft:slowness", 100, 3)
            ]
        );
        assert_eq!(contents.for_each_effect_scaled(0.5)[0].duration_ticks, 900);
        assert_eq!(
            color_optional(&[
                amp("minecraft:speed", 20, 1),
                amp("minecraft:slowness", 20, 3)
            ]),
            Some(0xff6d_c3ea_u32 as i32)
        );
        assert_eq!(PotionContentsModel::empty().get_color(), BASE_POTION_COLOR);
        assert_eq!(
            PotionContentsModel {
                custom_color: Some(0x112233),
                ..PotionContentsModel::new("swiftness")
            }
            .get_color_or(0),
            0x112233
        );
        assert!(PotionContentsModel::new("water").is("water"));
        assert!(!contents.is("strong_swiftness"));
    }

    #[test]
    fn potion_brewing_vanilla_graph_matches_java_add_vanilla_mixes() {
        assert!(POTION_BREWING_SOURCE.contains("public static final int BREWING_TIME_SECONDS = 20"));
        assert!(POTION_BREWING_SOURCE.contains("builder.addContainerRecipe(Items.POTION, Items.GUNPOWDER, Items.SPLASH_POTION);"));
        assert!(POTION_BREWING_SOURCE.contains("builder.addStartMix(Items.BREEZE_ROD, Potions.WIND_CHARGED);"));
        assert_eq!(BREWING_TIME_SECONDS, 20);

        let brewing = PotionBrewingModel::vanilla();
        assert_eq!(
            brewing.containers,
            vec![
                "minecraft:potion",
                "minecraft:splash_potion",
                "minecraft:lingering_potion"
            ]
        );
        assert_eq!(brewing.container_mixes.len(), 2);
        assert_eq!(brewing.potion_mixes.len(), 63);
        assert!(brewing.is_ingredient("minecraft:nether_wart"));
        assert!(brewing.is_container_ingredient("minecraft:dragon_breath"));
        assert!(brewing.is_potion_ingredient("minecraft:breeze_rod"));
        assert!(!brewing.is_ingredient("minecraft:apple"));
        assert!(brewing.is_brewable_potion("wind_charged"));
        assert!(!brewing.is_brewable_potion("water"));

        let water = PotionStack {
            item: "minecraft:potion",
            potion: Some("water"),
        };
        assert!(brewing.has_mix(&water, "minecraft:nether_wart"));
        assert_eq!(
            brewing.mix("minecraft:nether_wart", &water),
            PotionStack {
                item: "minecraft:potion",
                potion: Some("awkward")
            }
        );
        assert_eq!(
            brewing.mix(
                "minecraft:gunpowder",
                &PotionStack {
                    item: "minecraft:potion",
                    potion: Some("awkward")
                },
            ),
            PotionStack {
                item: "minecraft:splash_potion",
                potion: Some("awkward")
            }
        );
        assert_eq!(
            brewing.mix(
                "minecraft:dragon_breath",
                &PotionStack {
                    item: "minecraft:splash_potion",
                    potion: Some("strong_harming")
                },
            ),
            PotionStack {
                item: "minecraft:lingering_potion",
                potion: Some("strong_harming")
            }
        );
    }

    #[test]
    fn vanilla_brewing_recipes_adapt_java_graph_to_block_entity_recipe_surface() {
        let recipes = vanilla_brewing_recipes();
        assert_eq!(
            recipes.len(),
            VANILLA_POTION_MIXES.len() * 3 + VANILLA_CONTAINER_MIXES.len() * POTIONS.len()
        );
        assert!(recipes.iter().any(|recipe| {
            recipe.source_item == "minecraft:potion"
                && recipe.source_potion == "awkward"
                && recipe.ingredient == "minecraft:blaze_powder"
                && recipe.result_item == "minecraft:potion"
                && recipe.result_potion == "strength"
        }));
        assert!(recipes.iter().any(|recipe| {
            recipe.source_item == "minecraft:splash_potion"
                && recipe.source_potion == "strong_harming"
                && recipe.ingredient == "minecraft:dragon_breath"
                && recipe.result_item == "minecraft:lingering_potion"
                && recipe.result_potion == "strong_harming"
        }));
    }
}
