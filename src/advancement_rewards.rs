use std::collections::BTreeMap;

use crate::advancement_system::AdvancementRewards;
use crate::registry::Identifier;

impl AdvancementRewards {
    pub fn java_empty() -> Self {
        Self::default()
    }

    pub fn java_builder() -> AdvancementRewardsBuilder {
        AdvancementRewardsBuilder::new()
    }

    pub fn grant_to_model(
        &self,
        player: &mut AdvancementRewardPlayerModel,
        environment: &AdvancementRewardEnvironment,
    ) {
        player.experience_points += self.experience;
        let mut inventory_changed = false;

        for loot_table in &self.loot {
            for item in environment.loot_items(loot_table) {
                if player.try_add_item(item.clone()) {
                    player.play_pickup_sound();
                    inventory_changed = true;
                } else {
                    player.drop_for_self(item.clone());
                }
            }
        }

        if inventory_changed {
            player.broadcast_container_changes += 1;
        }
        if !self.recipes.is_empty() {
            player.awarded_recipes.extend(self.recipes.iter().cloned());
        }
        if let Some(function) = &self.function {
            player.executed_functions.push(ExecutedRewardFunction {
                id: function.clone(),
                suppressed_output: true,
                permission_level: RewardFunctionPermissionLevel::GameMaster,
            });
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementRewardsBuilder {
    experience: i32,
    loot: Vec<Identifier>,
    recipes: Vec<Identifier>,
    function: Option<Identifier>,
}

impl AdvancementRewardsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn experience(amount: i32) -> Self {
        Self::new().add_experience(amount)
    }

    pub fn add_experience(mut self, amount: i32) -> Self {
        self.experience += amount;
        self
    }

    pub fn loot(id: Identifier) -> Self {
        Self::new().add_loot_table(id)
    }

    pub fn add_loot_table(mut self, id: Identifier) -> Self {
        self.loot.push(id);
        self
    }

    pub fn recipe(id: Identifier) -> Self {
        Self::new().add_recipe(id)
    }

    pub fn add_recipe(mut self, id: Identifier) -> Self {
        self.recipes.push(id);
        self
    }

    pub fn function(id: Identifier) -> Self {
        Self::new().runs(id)
    }

    pub fn runs(mut self, function: Identifier) -> Self {
        self.function = Some(function);
        self
    }

    pub fn build(self) -> AdvancementRewards {
        AdvancementRewards {
            experience: self.experience,
            loot: self.loot,
            recipes: self.recipes,
            function: self.function,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementRewardEnvironment {
    loot_tables: BTreeMap<Identifier, Vec<RewardItemStack>>,
}

impl AdvancementRewardEnvironment {
    pub fn with_loot_table(mut self, id: Identifier, items: Vec<RewardItemStack>) -> Self {
        self.loot_tables.insert(id, items);
        self
    }

    fn loot_items(&self, id: &Identifier) -> &[RewardItemStack] {
        self.loot_tables.get(id).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementRewardPlayerModel {
    pub experience_points: i32,
    pub inventory_capacity: usize,
    pub inventory_items: Vec<RewardItemStack>,
    pub dropped_items: Vec<DroppedRewardItem>,
    pub pickup_sounds: usize,
    pub broadcast_container_changes: usize,
    pub awarded_recipes: Vec<Identifier>,
    pub executed_functions: Vec<ExecutedRewardFunction>,
}

impl AdvancementRewardPlayerModel {
    pub fn new(inventory_capacity: usize) -> Self {
        Self {
            experience_points: 0,
            inventory_capacity,
            inventory_items: Vec::new(),
            dropped_items: Vec::new(),
            pickup_sounds: 0,
            broadcast_container_changes: 0,
            awarded_recipes: Vec::new(),
            executed_functions: Vec::new(),
        }
    }

    fn try_add_item(&mut self, item: RewardItemStack) -> bool {
        if self.inventory_items.len() >= self.inventory_capacity {
            return false;
        }
        self.inventory_items.push(item);
        true
    }

    fn play_pickup_sound(&mut self) {
        self.pickup_sounds += 1;
    }

    fn drop_for_self(&mut self, item: RewardItemStack) {
        self.dropped_items.push(DroppedRewardItem {
            item,
            no_pickup_delay: true,
            target_player: true,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardItemStack {
    pub item: Identifier,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroppedRewardItem {
    pub item: RewardItemStack,
    pub no_pickup_delay: bool,
    pub target_player: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedRewardFunction {
    pub id: Identifier,
    pub suppressed_output: bool,
    pub permission_level: RewardFunctionPermissionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardFunctionPermissionLevel {
    GameMaster,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advancement_system::AdvancementDefinition;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(value: &str, count: i32) -> RewardItemStack {
        RewardItemStack {
            item: id(value),
            count,
        }
    }

    #[test]
    fn advancement_rewards_builder_matches_java_accumulation_and_replacement() {
        assert_eq!(
            AdvancementRewards::java_empty(),
            AdvancementRewards::default()
        );

        let rewards = AdvancementRewards::java_builder()
            .add_experience(5)
            .add_experience(7)
            .add_loot_table(id("minecraft:chests/simple_dungeon"))
            .add_loot_table(id("minecraft:gameplay/fishing"))
            .add_recipe(id("minecraft:oak_planks"))
            .runs(id("minecraft:first"))
            .runs(id("minecraft:second"))
            .build();

        assert_eq!(rewards.experience, 12);
        assert_eq!(
            rewards.loot,
            vec![
                id("minecraft:chests/simple_dungeon"),
                id("minecraft:gameplay/fishing")
            ]
        );
        assert_eq!(rewards.recipes, vec![id("minecraft:oak_planks")]);
        assert_eq!(rewards.function, Some(id("minecraft:second")));

        assert_eq!(
            AdvancementRewardsBuilder::experience(3).build().experience,
            3
        );
        assert_eq!(
            AdvancementRewardsBuilder::loot(id("minecraft:bonus"))
                .build()
                .loot,
            vec![id("minecraft:bonus")]
        );
        assert_eq!(
            AdvancementRewardsBuilder::recipe(id("minecraft:stick"))
                .build()
                .recipes,
            vec![id("minecraft:stick")]
        );
        assert_eq!(
            AdvancementRewardsBuilder::function(id("minecraft:reward"))
                .build()
                .function,
            Some(id("minecraft:reward"))
        );
    }

    #[test]
    fn advancement_rewards_grant_matches_java_action_order_and_side_effects() {
        let rewards = AdvancementRewards {
            experience: 25,
            loot: vec![id("minecraft:advancements/story/root")],
            recipes: vec![id("minecraft:oak_planks")],
            function: Some(id("minecraft:reward")),
        };
        let environment = AdvancementRewardEnvironment::default().with_loot_table(
            id("minecraft:advancements/story/root"),
            vec![stack("minecraft:stone", 2), stack("minecraft:diamond", 1)],
        );
        let mut player = AdvancementRewardPlayerModel::new(1);

        rewards.grant_to_model(&mut player, &environment);

        assert_eq!(player.experience_points, 25);
        assert_eq!(player.inventory_items, vec![stack("minecraft:stone", 2)]);
        assert_eq!(player.pickup_sounds, 1);
        assert_eq!(player.broadcast_container_changes, 1);
        assert_eq!(
            player.dropped_items,
            vec![DroppedRewardItem {
                item: stack("minecraft:diamond", 1),
                no_pickup_delay: true,
                target_player: true,
            }]
        );
        assert_eq!(player.awarded_recipes, vec![id("minecraft:oak_planks")]);
        assert_eq!(
            player.executed_functions,
            vec![ExecutedRewardFunction {
                id: id("minecraft:reward"),
                suppressed_output: true,
                permission_level: RewardFunctionPermissionLevel::GameMaster,
            }]
        );
    }

    #[test]
    fn advancement_rewards_json_defaults_and_fields_match_java_codec() {
        let defaults = AdvancementDefinition::from_json(
            "minecraft:test/default_rewards",
            r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}},"rewards":{}}"#,
        )
        .unwrap();
        assert_eq!(defaults.rewards, AdvancementRewards::java_empty());

        let parsed = AdvancementDefinition::from_json(
            "minecraft:test/rewards",
            r#"{
                "criteria":{"tick":{"trigger":"minecraft:tick"}},
                "rewards":{
                    "experience": 40,
                    "loot":["minecraft:advancements/story/root"],
                    "recipes":["minecraft:oak_planks"],
                    "function":"minecraft:reward"
                }
            }"#,
        )
        .unwrap();
        assert_eq!(
            parsed.rewards,
            AdvancementRewards {
                experience: 40,
                loot: vec![id("minecraft:advancements/story/root")],
                recipes: vec![id("minecraft:oak_planks")],
                function: Some(id("minecraft:reward")),
            }
        );
    }
}
