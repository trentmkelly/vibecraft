#![allow(dead_code)]

use std::collections::BTreeMap;

pub const MAX_FOOD: i32 = 20;
pub const MAX_SATURATION: f32 = 20.0;
pub const START_SATURATION: f32 = 5.0;
pub const SATURATION_FLOOR: f32 = 2.5;
pub const EXHAUSTION_DROP: f32 = 4.0;
pub const HEALTH_TICK_COUNT: i32 = 80;
pub const HEALTH_TICK_COUNT_SATURATED: i32 = 10;
pub const HEAL_LEVEL: i32 = 18;
pub const SPRINT_LEVEL: i32 = 6;
pub const STARVE_LEVEL: i32 = 0;
pub const FOOD_SATURATION_POOR: f32 = 0.1;
pub const FOOD_SATURATION_LOW: f32 = 0.3;
pub const FOOD_SATURATION_NORMAL: f32 = 0.6;
pub const FOOD_SATURATION_GOOD: f32 = 0.8;
pub const FOOD_SATURATION_MAX: f32 = 1.0;
pub const FOOD_SATURATION_SUPERNATURAL: f32 = 1.2;
pub const EXHAUSTION_HEAL: f32 = 6.0;
pub const EXHAUSTION_JUMP: f32 = 0.05;
pub const EXHAUSTION_SPRINT_JUMP: f32 = 0.2;
pub const EXHAUSTION_MINE: f32 = 0.005;
pub const EXHAUSTION_ATTACK: f32 = 0.1;
pub const EXHAUSTION_WALK: f32 = 0.0;
pub const EXHAUSTION_CROUCH: f32 = 0.0;
pub const EXHAUSTION_SPRINT: f32 = 0.1;
pub const EXHAUSTION_SWIM: f32 = 0.01;

pub fn saturation_by_modifier(nutrition: i32, modifier: f32) -> f32 {
    nutrition as f32 * modifier * 2.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodProperties {
    nutrition: i32,
    saturation: f32,
    can_always_eat: bool,
}

impl FoodProperties {
    pub const fn new(nutrition: i32, saturation: f32, can_always_eat: bool) -> Self {
        Self {
            nutrition,
            saturation,
            can_always_eat,
        }
    }

    pub fn builder() -> FoodPropertiesBuilder {
        FoodPropertiesBuilder::default()
    }

    pub fn nutrition(&self) -> i32 {
        self.nutrition
    }

    pub fn saturation(&self) -> f32 {
        self.saturation
    }

    pub fn can_always_eat(&self) -> bool {
        self.can_always_eat
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FoodPropertiesBuilder {
    nutrition: i32,
    saturation_modifier: f32,
    can_always_eat: bool,
}

impl FoodPropertiesBuilder {
    pub fn nutrition(mut self, nutrition: i32) -> Self {
        self.nutrition = nutrition;
        self
    }

    pub fn saturation_modifier(mut self, saturation_modifier: f32) -> Self {
        self.saturation_modifier = saturation_modifier;
        self
    }

    pub fn always_edible(mut self) -> Self {
        self.can_always_eat = true;
        self
    }

    pub fn build(self) -> FoodProperties {
        FoodProperties::new(
            self.nutrition,
            saturation_by_modifier(self.nutrition, self.saturation_modifier),
            self.can_always_eat,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodDefinition {
    pub name: &'static str,
    pub nutrition: i32,
    pub saturation_modifier: f32,
    pub can_always_eat: bool,
}

impl FoodDefinition {
    pub fn properties(&self) -> FoodProperties {
        let builder = FoodProperties::builder()
            .nutrition(self.nutrition)
            .saturation_modifier(self.saturation_modifier);
        if self.can_always_eat {
            builder.always_edible().build()
        } else {
            builder.build()
        }
    }
}

const fn food(name: &'static str, nutrition: i32, saturation_modifier: f32) -> FoodDefinition {
    FoodDefinition {
        name,
        nutrition,
        saturation_modifier,
        can_always_eat: false,
    }
}

const fn always_food(
    name: &'static str,
    nutrition: i32,
    saturation_modifier: f32,
) -> FoodDefinition {
    FoodDefinition {
        name,
        nutrition,
        saturation_modifier,
        can_always_eat: true,
    }
}

const fn stew(name: &'static str, nutrition: i32) -> FoodDefinition {
    food(name, nutrition, FOOD_SATURATION_NORMAL)
}

const fn always_stew(name: &'static str, nutrition: i32) -> FoodDefinition {
    always_food(name, nutrition, FOOD_SATURATION_NORMAL)
}

pub const FOODS: &[FoodDefinition] = &[
    food("APPLE", 4, FOOD_SATURATION_LOW),
    food("BAKED_POTATO", 5, FOOD_SATURATION_NORMAL),
    food("BEEF", 3, FOOD_SATURATION_LOW),
    food("BEETROOT", 1, FOOD_SATURATION_NORMAL),
    stew("BEETROOT_SOUP", 6),
    food("BREAD", 5, FOOD_SATURATION_NORMAL),
    food("CARROT", 3, FOOD_SATURATION_NORMAL),
    food("CHICKEN", 2, FOOD_SATURATION_LOW),
    always_food("CHORUS_FRUIT", 4, FOOD_SATURATION_LOW),
    food("COD", 2, FOOD_SATURATION_POOR),
    food("COOKED_BEEF", 8, FOOD_SATURATION_GOOD),
    food("COOKED_CHICKEN", 6, FOOD_SATURATION_NORMAL),
    food("COOKED_COD", 5, FOOD_SATURATION_NORMAL),
    food("COOKED_MUTTON", 6, FOOD_SATURATION_GOOD),
    food("COOKED_PORKCHOP", 8, FOOD_SATURATION_GOOD),
    food("COOKED_RABBIT", 5, FOOD_SATURATION_NORMAL),
    food("COOKED_SALMON", 6, FOOD_SATURATION_GOOD),
    food("COOKIE", 2, FOOD_SATURATION_POOR),
    food("DRIED_KELP", 1, FOOD_SATURATION_LOW),
    always_food(
        "ENCHANTED_GOLDEN_APPLE",
        4,
        FOOD_SATURATION_SUPERNATURAL,
    ),
    always_food("GOLDEN_APPLE", 4, FOOD_SATURATION_SUPERNATURAL),
    food("GOLDEN_CARROT", 6, FOOD_SATURATION_SUPERNATURAL),
    always_food("HONEY_BOTTLE", 6, FOOD_SATURATION_POOR),
    food("MELON_SLICE", 2, FOOD_SATURATION_LOW),
    stew("MUSHROOM_STEW", 6),
    food("MUTTON", 2, FOOD_SATURATION_LOW),
    food("POISONOUS_POTATO", 2, FOOD_SATURATION_LOW),
    food("PORKCHOP", 3, FOOD_SATURATION_LOW),
    food("POTATO", 1, FOOD_SATURATION_LOW),
    food("PUFFERFISH", 1, FOOD_SATURATION_POOR),
    food("PUMPKIN_PIE", 8, FOOD_SATURATION_LOW),
    food("RABBIT", 3, FOOD_SATURATION_LOW),
    stew("RABBIT_STEW", 10),
    food("ROTTEN_FLESH", 4, FOOD_SATURATION_POOR),
    food("SALMON", 2, FOOD_SATURATION_POOR),
    food("SPIDER_EYE", 2, FOOD_SATURATION_GOOD),
    always_stew("SUSPICIOUS_STEW", 6),
    food("SWEET_BERRIES", 2, FOOD_SATURATION_POOR),
    food("GLOW_BERRIES", 2, FOOD_SATURATION_POOR),
    food("TROPICAL_FISH", 1, FOOD_SATURATION_POOR),
];

pub fn food_definition(name: &str) -> Option<&'static FoodDefinition> {
    FOODS.iter().find(|food| food.name == name)
}

pub fn food_properties(name: &str) -> Option<FoodProperties> {
    food_definition(name).map(FoodDefinition::properties)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodTickInput {
    pub difficulty: Difficulty,
    pub natural_regen: bool,
    pub is_hurt: bool,
    pub health: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoodTickOutcome {
    None,
    Heal { amount: f32 },
    Starve { amount: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodData {
    food_level: i32,
    saturation_level: f32,
    exhaustion_level: f32,
    tick_timer: i32,
}

impl Default for FoodData {
    fn default() -> Self {
        Self {
            food_level: MAX_FOOD,
            saturation_level: START_SATURATION,
            exhaustion_level: 0.0,
            tick_timer: 0,
        }
    }
}

impl FoodData {
    pub fn add(&mut self, food: i32, saturation: f32) {
        self.food_level = (food + self.food_level).clamp(0, MAX_FOOD);
        self.saturation_level = (saturation + self.saturation_level).clamp(0.0, self.food_level as f32);
    }

    pub fn eat(&mut self, food: i32, saturation_modifier: f32) {
        self.add(food, saturation_by_modifier(food, saturation_modifier));
    }

    pub fn eat_properties(&mut self, food_properties: FoodProperties) {
        self.add(food_properties.nutrition(), food_properties.saturation());
    }

    pub fn tick(&mut self, input: FoodTickInput) -> FoodTickOutcome {
        if self.exhaustion_level > EXHAUSTION_DROP {
            self.exhaustion_level -= EXHAUSTION_DROP;
            if self.saturation_level > 0.0 {
                self.saturation_level = (self.saturation_level - 1.0).max(0.0);
            } else if input.difficulty != Difficulty::Peaceful {
                self.food_level = (self.food_level - 1).max(0);
            }
        }

        if input.natural_regen
            && self.saturation_level > 0.0
            && input.is_hurt
            && self.food_level >= MAX_FOOD
        {
            self.tick_timer += 1;
            if self.tick_timer >= HEALTH_TICK_COUNT_SATURATED {
                let saturation_spent = self.saturation_level.min(EXHAUSTION_HEAL);
                self.add_exhaustion(saturation_spent);
                self.tick_timer = 0;
                return FoodTickOutcome::Heal {
                    amount: saturation_spent / EXHAUSTION_HEAL,
                };
            }
        } else if input.natural_regen && self.food_level >= HEAL_LEVEL && input.is_hurt {
            self.tick_timer += 1;
            if self.tick_timer >= HEALTH_TICK_COUNT {
                self.add_exhaustion(EXHAUSTION_HEAL);
                self.tick_timer = 0;
                return FoodTickOutcome::Heal { amount: 1.0 };
            }
        } else if self.food_level <= STARVE_LEVEL {
            self.tick_timer += 1;
            if self.tick_timer >= HEALTH_TICK_COUNT {
                if starvation_damages(input.difficulty, input.health) {
                    self.tick_timer = 0;
                    return FoodTickOutcome::Starve { amount: 1.0 };
                }
                self.tick_timer = 0;
            }
        } else {
            self.tick_timer = 0;
        }

        FoodTickOutcome::None
    }

    pub fn read_additional_save_data(&mut self, input: &BTreeMap<String, FoodSaveValue>) {
        self.food_level = input
            .get("foodLevel")
            .and_then(FoodSaveValue::as_int)
            .unwrap_or(MAX_FOOD);
        self.tick_timer = input
            .get("foodTickTimer")
            .and_then(FoodSaveValue::as_int)
            .unwrap_or(0);
        self.saturation_level = input
            .get("foodSaturationLevel")
            .and_then(FoodSaveValue::as_float)
            .unwrap_or(START_SATURATION);
        self.exhaustion_level = input
            .get("foodExhaustionLevel")
            .and_then(FoodSaveValue::as_float)
            .unwrap_or(0.0);
    }

    pub fn add_additional_save_data(&self) -> BTreeMap<String, FoodSaveValue> {
        BTreeMap::from([
            ("foodExhaustionLevel".to_string(), FoodSaveValue::Float(self.exhaustion_level)),
            ("foodLevel".to_string(), FoodSaveValue::Int(self.food_level)),
            ("foodSaturationLevel".to_string(), FoodSaveValue::Float(self.saturation_level)),
            ("foodTickTimer".to_string(), FoodSaveValue::Int(self.tick_timer)),
        ])
    }

    pub fn food_level(&self) -> i32 {
        self.food_level
    }

    pub fn has_enough_food(&self) -> bool {
        self.food_level() > SPRINT_LEVEL
    }

    pub fn needs_food(&self) -> bool {
        self.food_level < MAX_FOOD
    }

    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion_level = (self.exhaustion_level + amount).min(40.0);
    }

    pub fn saturation_level(&self) -> f32 {
        self.saturation_level
    }

    pub fn exhaustion_level(&self) -> f32 {
        self.exhaustion_level
    }

    pub fn tick_timer(&self) -> i32 {
        self.tick_timer
    }

    pub fn set_food_level(&mut self, food: i32) {
        self.food_level = food;
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        self.saturation_level = saturation;
    }

    pub fn set_exhaustion(&mut self, exhaustion: f32) {
        self.exhaustion_level = exhaustion;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoodSaveValue {
    Int(i32),
    Float(f32),
}

impl FoodSaveValue {
    fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(value) => Some(*value),
            Self::Float(_) => None,
        }
    }

    fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Int(_) => None,
        }
    }
}

pub fn starvation_damages(difficulty: Difficulty, health: f32) -> bool {
    health > 10.0 || difficulty == Difficulty::Hard || health > 1.0 && difficulty == Difficulty::Normal
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const FOOD_CONSTANTS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/food/FoodConstants.java");
    const FOOD_DATA_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/food/FoodData.java");
    const FOOD_PROPERTIES_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/food/FoodProperties.java");
    const FOODS_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/food/Foods.java");

    fn assert_contains(source: &str, needle: &str) {
        assert!(
            source.contains(needle),
            "Java source no longer contains sentinel: {needle}"
        );
    }

    fn tick_input(difficulty: Difficulty, natural_regen: bool, is_hurt: bool, health: f32) -> FoodTickInput {
        FoodTickInput {
            difficulty,
            natural_regen,
            is_hurt,
            health,
        }
    }

    #[test]
    fn food_constants_match_java_literals() {
        for needle in [
            "public static final int MAX_FOOD = 20;",
            "public static final float MAX_SATURATION = 20.0F;",
            "public static final float START_SATURATION = 5.0F;",
            "public static final float EXHAUSTION_DROP = 4.0F;",
            "public static final int HEALTH_TICK_COUNT = 80;",
            "public static final int HEALTH_TICK_COUNT_SATURATED = 10;",
            "public static final int HEAL_LEVEL = 18;",
            "public static final int SPRINT_LEVEL = 6;",
            "public static final float EXHAUSTION_SWIM = 0.01F;",
            "return nutrition * modifier * 2.0F;",
        ] {
            assert_contains(FOOD_CONSTANTS_JAVA, needle);
        }
        assert_eq!(MAX_FOOD, 20);
        assert_eq!(saturation_by_modifier(6, FOOD_SATURATION_NORMAL), 7.2000003);
    }

    #[test]
    fn food_properties_builder_matches_java_shape() {
        for needle in [
            "public record FoodProperties(int nutrition, float saturation, boolean canAlwaysEat)",
            "ExtraCodecs.NON_NEGATIVE_INT.fieldOf(\"nutrition\")",
            "Codec.BOOL.optionalFieldOf(\"can_always_eat\", false)",
            "float saturation = FoodConstants.saturationByModifier(this.nutrition, this.saturationModifier);",
            "player.getFoodData().eat(this);",
        ] {
            assert_contains(FOOD_PROPERTIES_JAVA, needle);
        }

        let food = FoodProperties::builder()
            .nutrition(4)
            .saturation_modifier(1.2)
            .always_edible()
            .build();
        assert_eq!(food.nutrition(), 4);
        assert_eq!(food.saturation(), 9.6);
        assert!(food.can_always_eat());
    }

    #[test]
    fn foods_table_matches_java_constants_and_builder_inputs() {
        assert_eq!(FOODS.len(), 40);
        assert_eq!(FOODS_JAVA.matches("public static final FoodProperties ").count(), FOODS.len());
        for definition in FOODS {
            assert_contains(
                FOODS_JAVA,
                &format!("public static final FoodProperties {}", definition.name),
            );
        }

        assert_eq!(
            food_properties("COOKED_BEEF"),
            Some(FoodProperties::new(8, 12.8, false))
        );
        assert_eq!(
            food_properties("GOLDEN_CARROT"),
            Some(FoodProperties::new(6, 14.400001, false))
        );
        assert_eq!(
            food_properties("SUSPICIOUS_STEW"),
            Some(FoodProperties::new(6, 7.2000003, true))
        );
        assert_eq!(food_properties("UNKNOWN"), None);
    }

    #[test]
    fn food_data_eating_exhaustion_and_save_data_match_java() {
        for needle in [
            "private int foodLevel = 20;",
            "private float saturationLevel = 5.0F;",
            "this.foodLevel = Mth.clamp(food + this.foodLevel, 0, 20);",
            "this.saturationLevel = Mth.clamp(saturation + this.saturationLevel, 0.0F, this.foodLevel);",
            "this.exhaustionLevel = Math.min(this.exhaustionLevel + amount, 40.0F);",
            "input.getIntOr(\"foodLevel\", 20)",
            "output.putFloat(\"foodExhaustionLevel\", this.exhaustionLevel);",
        ] {
            assert_contains(FOOD_DATA_JAVA, needle);
        }

        let mut food = FoodData::default();
        food.set_food_level(17);
        food.set_saturation(1.0);
        food.eat(5, FOOD_SATURATION_NORMAL);
        assert_eq!(food.food_level(), 20);
        assert_eq!(food.saturation_level(), 7.0);

        food.set_exhaustion(39.0);
        food.add_exhaustion(3.0);
        assert_eq!(food.exhaustion_level(), 40.0);
        food.add_exhaustion(-2.5);
        assert_eq!(food.exhaustion_level(), 37.5);

        let saved = food.add_additional_save_data();
        assert_eq!(saved.get("foodLevel"), Some(&FoodSaveValue::Int(20)));
        assert_eq!(saved.get("foodTickTimer"), Some(&FoodSaveValue::Int(0)));
        assert_eq!(
            saved.get("foodSaturationLevel"),
            Some(&FoodSaveValue::Float(7.0))
        );
        assert_eq!(
            saved.get("foodExhaustionLevel"),
            Some(&FoodSaveValue::Float(37.5))
        );

        let mut loaded = FoodData::default();
        loaded.read_additional_save_data(&BTreeMap::from([
            ("foodLevel".to_string(), FoodSaveValue::Int(9)),
            ("foodTickTimer".to_string(), FoodSaveValue::Int(12)),
            ("foodSaturationLevel".to_string(), FoodSaveValue::Float(3.5)),
            ("foodExhaustionLevel".to_string(), FoodSaveValue::Float(1.25)),
        ]));
        assert_eq!(loaded.food_level(), 9);
        assert_eq!(loaded.tick_timer(), 12);
        assert_eq!(loaded.saturation_level(), 3.5);
        assert_eq!(loaded.exhaustion_level(), 1.25);
    }

    #[test]
    fn tick_drains_exhaustion_then_regenerates_or_starves_like_java() {
        for needle in [
            "if (this.exhaustionLevel > 4.0F)",
            "this.exhaustionLevel -= 4.0F;",
            "this.saturationLevel = Math.max(this.saturationLevel - 1.0F, 0.0F);",
            "this.tickTimer >= 10",
            "player.heal(saturationSpent / 6.0F);",
            "this.tickTimer >= 80",
            "player.heal(1.0F);",
            "player.hurtServer(level, player.damageSources().starve(), 1.0F);",
        ] {
            assert_contains(FOOD_DATA_JAVA, needle);
        }

        let mut fast = FoodData::default();
        for _ in 0..9 {
            assert_eq!(
                fast.tick(tick_input(Difficulty::Normal, true, true, 19.0)),
                FoodTickOutcome::None
            );
        }
        assert_eq!(
            fast.tick(tick_input(Difficulty::Normal, true, true, 19.0)),
            FoodTickOutcome::Heal {
                amount: START_SATURATION / EXHAUSTION_HEAL
            }
        );
        assert_eq!(fast.exhaustion_level(), START_SATURATION);
        assert_eq!(fast.tick_timer(), 0);

        let mut slow = FoodData::default();
        slow.set_food_level(18);
        slow.set_saturation(0.0);
        for _ in 0..79 {
            assert_eq!(
                slow.tick(tick_input(Difficulty::Easy, true, true, 7.0)),
                FoodTickOutcome::None
            );
        }
        assert_eq!(
            slow.tick(tick_input(Difficulty::Easy, true, true, 7.0)),
            FoodTickOutcome::Heal { amount: 1.0 }
        );
        assert_eq!(slow.exhaustion_level(), EXHAUSTION_HEAL);

        let mut starving = FoodData::default();
        starving.set_food_level(0);
        starving.set_saturation(0.0);
        for _ in 0..79 {
            assert_eq!(
                starving.tick(tick_input(Difficulty::Normal, false, false, 1.0)),
                FoodTickOutcome::None
            );
        }
        assert_eq!(
            starving.tick(tick_input(Difficulty::Normal, false, false, 1.0)),
            FoodTickOutcome::None
        );
        assert_eq!(starving.tick_timer(), 0);

        let mut hard_starving = FoodData::default();
        hard_starving.set_food_level(0);
        for _ in 0..79 {
            let _ = hard_starving.tick(tick_input(Difficulty::Hard, false, false, 1.0));
        }
        assert_eq!(
            hard_starving.tick(tick_input(Difficulty::Hard, false, false, 1.0)),
            FoodTickOutcome::Starve { amount: 1.0 }
        );

        let mut drained = FoodData::default();
        drained.set_saturation(0.0);
        drained.set_exhaustion(4.1);
        assert_eq!(
            drained.tick(tick_input(Difficulty::Peaceful, false, false, 20.0)),
            FoodTickOutcome::None
        );
        assert_eq!(drained.food_level(), MAX_FOOD);
        assert!((drained.exhaustion_level() - 0.099999905).abs() < f32::EPSILON);
    }
}
