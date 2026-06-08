use std::collections::{BTreeMap, BTreeSet};

use crate::criterion_food_predicate::{FoodDataModel, FoodPredicateModel};
use crate::criterion_game_type_predicate::{GameTypeModel, GameTypePredicateModel};
use crate::criterion_input_predicate::{InputModel, InputPredicateModel};
use crate::criterion_min_max_bounds::IntsBoundsModel;
use crate::registry::Identifier;

pub const LOOKING_AT_RANGE: f64 = 100.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPredicateModel {
    pub level: IntsBoundsModel,
    pub food: FoodPredicateModel,
    pub game_type: GameTypePredicateModel,
    pub stats: Vec<StatMatcherModel>,
    pub recipes: BTreeMap<Identifier, bool>,
    pub advancements: BTreeMap<Identifier, AdvancementPredicateModel>,
    pub looking_at: Option<EntityPredicateModel>,
    pub input: Option<InputPredicateModel>,
}

impl PlayerPredicateModel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        level: IntsBoundsModel,
        food: FoodPredicateModel,
        game_type: GameTypePredicateModel,
        stats: Vec<StatMatcherModel>,
        recipes: BTreeMap<Identifier, bool>,
        advancements: BTreeMap<Identifier, AdvancementPredicateModel>,
        looking_at: Option<EntityPredicateModel>,
        input: Option<InputPredicateModel>,
    ) -> Self {
        Self {
            level,
            food,
            game_type,
            stats,
            recipes,
            advancements,
            looking_at,
            input,
        }
    }

    pub fn builder() -> PlayerPredicateBuilderModel {
        PlayerPredicateBuilderModel::player()
    }

    pub fn matches(
        &self,
        entity: &EntityModel,
        level: &ServerLevelModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        let EntityModel::Player(player) = entity else {
            return false;
        };

        if !self.level.matches(player.experience_level) {
            return false;
        }

        if !self.food.matches(&player.food) {
            return false;
        }

        if !self.game_type.matches(player.game_type) {
            return false;
        }

        for stat in &self.stats {
            if !stat.matches(&player.stats) {
                return false;
            }
        }

        for (recipe, present) in &self.recipes {
            if player.recipes.contains(recipe) != *present {
                return false;
            }
        }

        for (advancement_id, predicate) in &self.advancements {
            let Some(progress) = level.advancements.get(advancement_id) else {
                return false;
            };
            if !predicate.test(progress) {
                return false;
            }
        }

        if let Some(looking_at) = &self.looking_at {
            let Some(target) = player.looking_at.as_ref() else {
                return false;
            };
            if target.distance > LOOKING_AT_RANGE
                || target.spectator
                || !target.line_of_sight
                || !looking_at.matches(&target.entity)
            {
                return false;
            }
        }

        self.input
            .as_ref()
            .is_none_or(|input| input.matches(&player.last_input))
    }

    pub fn codec_field_names() -> [&'static str; 8] {
        [
            "level",
            "food",
            "gamemode",
            "stats",
            "recipes",
            "advancements",
            "looking_at",
            "input",
        ]
    }

    pub fn entity_sub_predicate_type() -> Identifier {
        id("minecraft:player")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerPredicateBuilderModel {
    level: IntsBoundsModel,
    food: FoodPredicateModel,
    game_type: GameTypePredicateModel,
    stats: Vec<StatMatcherModel>,
    recipes: BTreeMap<Identifier, bool>,
    advancements: BTreeMap<Identifier, AdvancementPredicateModel>,
    looking_at: Option<EntityPredicateModel>,
    input: Option<InputPredicateModel>,
}

impl PlayerPredicateBuilderModel {
    pub fn player() -> Self {
        Self::default()
    }

    pub fn set_level(mut self, level: IntsBoundsModel) -> Self {
        self.level = level;
        self
    }

    pub fn set_food(mut self, food: FoodPredicateModel) -> Self {
        self.food = food;
        self
    }

    pub fn set_game_type(mut self, game_type: GameTypePredicateModel) -> Self {
        self.game_type = game_type;
        self
    }

    pub fn add_stat(
        mut self,
        stat_type: Identifier,
        value: Identifier,
        range: IntsBoundsModel,
    ) -> Self {
        self.stats
            .push(StatMatcherModel::new(stat_type, value, range));
        self
    }

    pub fn add_recipe(mut self, recipe: Identifier, present: bool) -> Self {
        self.recipes.insert(recipe, present);
        self
    }

    pub fn set_looking_at(mut self, looking_at: EntityPredicateBuilderModel) -> Self {
        self.looking_at = Some(looking_at.build());
        self
    }

    pub fn check_advancement_done(mut self, advancement: Identifier, is_done: bool) -> Self {
        self.advancements
            .insert(advancement, AdvancementPredicateModel::Done(is_done));
        self
    }

    pub fn check_advancement_criterions(
        mut self,
        advancement: Identifier,
        criterions: impl IntoIterator<Item = (String, bool)>,
    ) -> Self {
        self.advancements.insert(
            advancement,
            AdvancementPredicateModel::Criterions(criterions.into_iter().collect()),
        );
        self
    }

    pub fn has_input(mut self, input: InputPredicateModel) -> Self {
        self.input = Some(input);
        self
    }

    pub fn build(self) -> PlayerPredicateModel {
        PlayerPredicateModel::new(
            self.level,
            self.food,
            self.game_type,
            self.stats,
            self.recipes,
            self.advancements,
            self.looking_at,
            self.input,
        )
    }
}

impl Default for PlayerPredicateBuilderModel {
    fn default() -> Self {
        Self {
            level: IntsBoundsModel::ANY,
            food: FoodPredicateModel::ANY,
            game_type: GameTypePredicateModel::any(),
            stats: Vec::new(),
            recipes: BTreeMap::new(),
            advancements: BTreeMap::new(),
            looking_at: None,
            input: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatMatcherModel {
    stat_type: Identifier,
    value: Identifier,
    range: IntsBoundsModel,
}

impl StatMatcherModel {
    pub fn new(stat_type: Identifier, value: Identifier, range: IntsBoundsModel) -> Self {
        Self {
            stat_type,
            value,
            range,
        }
    }

    fn matches(&self, counter: &StatsCounterModel) -> bool {
        self.range
            .matches(counter.get_value(&self.stat_type, &self.value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementPredicateModel {
    Done(bool),
    Criterions(BTreeMap<String, bool>),
}

impl AdvancementPredicateModel {
    fn test(&self, progress: &AdvancementProgressModel) -> bool {
        match self {
            Self::Done(state) => progress.done == *state,
            Self::Criterions(criterions) => criterions.iter().all(|(name, state)| {
                progress
                    .criterions
                    .get(name)
                    .is_some_and(|done| done == state)
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementProgressModel {
    done: bool,
    criterions: BTreeMap<String, bool>,
}

impl AdvancementProgressModel {
    pub fn new(done: bool, criterions: impl IntoIterator<Item = (String, bool)>) -> Self {
        Self {
            done,
            criterions: criterions.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatsCounterModel {
    values: BTreeMap<(Identifier, Identifier), i32>,
}

impl StatsCounterModel {
    pub fn with_value(mut self, stat_type: Identifier, value: Identifier, count: i32) -> Self {
        self.values.insert((stat_type, value), count);
        self
    }

    fn get_value(&self, stat_type: &Identifier, value: &Identifier) -> i32 {
        *self
            .values
            .get(&(stat_type.clone(), value.clone()))
            .unwrap_or(&0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityModel {
    Player(ServerPlayerModel),
    Other(EntityContextModel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerPlayerModel {
    experience_level: i32,
    food: FoodDataModel,
    game_type: GameTypeModel,
    stats: StatsCounterModel,
    recipes: BTreeSet<Identifier>,
    looking_at: Option<LookingAtModel>,
    last_input: InputModel,
}

impl ServerPlayerModel {
    pub fn new() -> Self {
        Self {
            experience_level: 0,
            food: FoodDataModel::new(20, 5.0),
            game_type: GameTypeModel::Survival,
            stats: StatsCounterModel::default(),
            recipes: BTreeSet::new(),
            looking_at: None,
            last_input: InputModel::new(false, false, false, false, false, false, false),
        }
    }

    pub fn with_experience_level(mut self, experience_level: i32) -> Self {
        self.experience_level = experience_level;
        self
    }

    pub fn with_food(mut self, food: FoodDataModel) -> Self {
        self.food = food;
        self
    }

    pub fn with_game_type(mut self, game_type: GameTypeModel) -> Self {
        self.game_type = game_type;
        self
    }

    pub fn with_stats(mut self, stats: StatsCounterModel) -> Self {
        self.stats = stats;
        self
    }

    pub fn with_recipe(mut self, recipe: Identifier) -> Self {
        self.recipes.insert(recipe);
        self
    }

    pub fn with_looking_at(mut self, looking_at: LookingAtModel) -> Self {
        self.looking_at = Some(looking_at);
        self
    }

    pub fn with_last_input(mut self, input: InputModel) -> Self {
        self.last_input = input;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LookingAtModel {
    entity: EntityContextModel,
    distance: f64,
    line_of_sight: bool,
    spectator: bool,
}

impl LookingAtModel {
    pub fn new(entity: EntityContextModel) -> Self {
        Self {
            entity,
            distance: 10.0,
            line_of_sight: true,
            spectator: false,
        }
    }

    pub fn with_distance(mut self, distance: f64) -> Self {
        self.distance = distance;
        self
    }

    pub fn with_line_of_sight(mut self, line_of_sight: bool) -> Self {
        self.line_of_sight = line_of_sight;
        self
    }

    pub fn spectator(mut self) -> Self {
        self.spectator = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_type: Identifier,
    name: Option<String>,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier, name: Option<&str>) -> Self {
        Self {
            entity_type,
            name: name.map(str::to_string),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
}

impl EntityPredicateModel {
    fn matches(&self, entity: &EntityContextModel) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &entity.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                entity
                    .name
                    .as_ref()
                    .is_some_and(|entity_name| entity_name == name)
            })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilderModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
}

impl EntityPredicateBuilderModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn of(mut self, entity_type: Identifier) -> Self {
        self.required_type = Some(entity_type);
        self
    }

    pub fn named(mut self, name: &str) -> Self {
        self.required_name = Some(name.to_string());
        self
    }

    fn build(self) -> EntityPredicateModel {
        EntityPredicateModel {
            required_type: self.required_type,
            required_name: self.required_name,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerLevelModel {
    advancements: BTreeMap<Identifier, AdvancementProgressModel>,
}

impl ServerLevelModel {
    pub fn with_advancement(
        mut self,
        advancement: Identifier,
        progress: AdvancementProgressModel,
    ) -> Self {
        self.advancements.insert(advancement, progress);
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3Model;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::criterion_food_predicate::{FoodPredicateBuilderModel, IntBoundsModel};

    fn player_entity(player: ServerPlayerModel) -> EntityModel {
        EntityModel::Player(player)
    }

    fn other_entity() -> EntityModel {
        EntityModel::Other(EntityContextModel::new(id("minecraft:zombie"), None))
    }

    fn level() -> ServerLevelModel {
        ServerLevelModel::default()
    }

    fn input(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> InputModel {
        InputModel::new(forward, backward, left, right, jump, shift, sprint)
    }

    #[test]
    fn codec_fields_and_entity_sub_predicate_type_match_java() {
        assert_eq!(
            PlayerPredicateModel::codec_field_names(),
            [
                "level",
                "food",
                "gamemode",
                "stats",
                "recipes",
                "advancements",
                "looking_at",
                "input"
            ]
        );
        assert_eq!(
            PlayerPredicateModel::entity_sub_predicate_type(),
            id("minecraft:player")
        );
        assert_eq!(LOOKING_AT_RANGE, 100.0);
    }

    #[test]
    fn non_player_entities_never_match_and_default_builder_matches_any_player() {
        let predicate = PlayerPredicateModel::builder().build();

        assert!(!predicate.matches(&other_entity(), &level(), None));
        assert!(predicate.matches(&player_entity(ServerPlayerModel::new()), &level(), None));
    }

    #[test]
    fn level_food_and_game_type_are_checked_in_java_order() {
        let predicate = PlayerPredicateModel::builder()
            .set_level(IntsBoundsModel::between(10, 20))
            .set_food(
                FoodPredicateBuilderModel::food()
                    .with_level(IntBoundsModel::at_least(18))
                    .build(),
            )
            .set_game_type(GameTypePredicateModel::survival_like())
            .build();

        assert!(predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_experience_level(15)
                    .with_food(FoodDataModel::new(20, 5.0))
                    .with_game_type(GameTypeModel::Adventure)
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(ServerPlayerModel::new().with_experience_level(9)),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_experience_level(15)
                    .with_food(FoodDataModel::new(17, 5.0))
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_experience_level(15)
                    .with_game_type(GameTypeModel::Creative)
            ),
            &level(),
            None,
        ));
    }

    #[test]
    fn stat_matchers_and_recipe_expectations_use_player_counters_and_recipe_book() {
        let mined_stone = id("minecraft:mined_stone");
        let crafted = id("minecraft:recipes/building_blocks/stone_bricks");
        let absent = id("minecraft:recipes/misc/diamond");
        let predicate = PlayerPredicateModel::builder()
            .add_stat(
                id("minecraft:mined"),
                mined_stone.clone(),
                IntsBoundsModel::at_least(3),
            )
            .add_recipe(crafted.clone(), true)
            .add_recipe(absent, false)
            .build();
        let stats = StatsCounterModel::default().with_value(id("minecraft:mined"), mined_stone, 4);

        assert!(predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_stats(stats.clone())
                    .with_recipe(crafted)
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(ServerPlayerModel::new().with_stats(stats)),
            &level(),
            None,
        ));
    }

    #[test]
    fn advancement_done_and_criterion_predicates_match_server_progress() {
        let story = id("minecraft:story/mine_stone");
        let nether = id("minecraft:nether/root");
        let predicate = PlayerPredicateModel::builder()
            .check_advancement_done(story.clone(), true)
            .check_advancement_criterions(
                nether.clone(),
                [
                    ("entered_nether".to_string(), true),
                    ("returned".to_string(), false),
                ],
            )
            .build();
        let matching_level = level()
            .with_advancement(story.clone(), AdvancementProgressModel::new(true, []))
            .with_advancement(
                nether.clone(),
                AdvancementProgressModel::new(
                    false,
                    [
                        ("entered_nether".to_string(), true),
                        ("returned".to_string(), false),
                    ],
                ),
            );

        assert!(predicate.matches(
            &player_entity(ServerPlayerModel::new()),
            &matching_level,
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(ServerPlayerModel::new()),
            &level().with_advancement(story, AdvancementProgressModel::new(true, [])),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(ServerPlayerModel::new()),
            &level().with_advancement(nether, AdvancementProgressModel::new(false, [])),
            None,
        ));
    }

    #[test]
    fn looking_at_uses_range_entity_predicate_spectator_filter_and_line_of_sight() {
        let predicate = PlayerPredicateModel::builder()
            .set_looking_at(
                EntityPredicateBuilderModel::entity()
                    .of(id("minecraft:villager"))
                    .named("Target"),
            )
            .build();
        let target = EntityContextModel::new(id("minecraft:villager"), Some("Target"));

        assert!(predicate.matches(
            &player_entity(
                ServerPlayerModel::new().with_looking_at(LookingAtModel::new(target.clone()))
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_looking_at(LookingAtModel::new(target.clone()).with_distance(101.0))
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_looking_at(LookingAtModel::new(target.clone()).with_line_of_sight(false))
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new().with_looking_at(LookingAtModel::new(target).spectator())
            ),
            &level(),
            None,
        ));
    }

    #[test]
    fn input_predicate_is_checked_last_against_last_client_input() {
        let predicate = PlayerPredicateModel::builder()
            .has_input(InputPredicateModel::new(
                Some(true),
                None,
                None,
                None,
                Some(false),
                Some(true),
                Some(false),
            ))
            .build();

        assert!(predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_last_input(input(true, false, false, false, false, true, false))
            ),
            &level(),
            None,
        ));
        assert!(!predicate.matches(
            &player_entity(
                ServerPlayerModel::new()
                    .with_last_input(input(true, false, false, false, true, true, false))
            ),
            &level(),
            None,
        ));
    }

    #[test]
    fn builder_preserves_all_java_fields() {
        let predicate = PlayerPredicateModel::builder()
            .set_level(IntsBoundsModel::exactly(5))
            .set_food(FoodPredicateModel::ANY)
            .set_game_type(GameTypePredicateModel::of([GameTypeModel::Spectator]))
            .add_stat(
                id("minecraft:custom"),
                id("minecraft:jump"),
                IntsBoundsModel::at_least(1),
            )
            .add_recipe(id("minecraft:recipes/misc/map"), true)
            .check_advancement_done(id("minecraft:story/root"), true)
            .set_looking_at(EntityPredicateBuilderModel::entity().of(id("minecraft:pig")))
            .has_input(InputPredicateModel::default())
            .build();

        assert_eq!(predicate.level, IntsBoundsModel::exactly(5));
        assert_eq!(predicate.stats.len(), 1);
        assert_eq!(predicate.recipes.len(), 1);
        assert_eq!(predicate.advancements.len(), 1);
        assert!(predicate.looking_at.is_some());
        assert!(predicate.input.is_some());
    }
}
