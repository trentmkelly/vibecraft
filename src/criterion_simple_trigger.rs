use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::registry::Identifier;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimpleCriterionTriggerModel {
    players: HashMap<PlayerAdvancementsKey, HashSet<CriterionListenerModel>>,
}

impl SimpleCriterionTriggerModel {
    pub fn add_player_listener(
        &mut self,
        player: &PlayerAdvancementsModel,
        listener: CriterionListenerModel,
    ) {
        self.players
            .entry(player.key.clone())
            .or_default()
            .insert(listener);
    }

    pub fn remove_player_listener(
        &mut self,
        player: &PlayerAdvancementsModel,
        listener: &CriterionListenerModel,
    ) {
        let Some(listeners) = self.players.get_mut(&player.key) else {
            return;
        };

        listeners.remove(listener);
        if listeners.is_empty() {
            self.players.remove(&player.key);
        }
    }

    pub fn remove_player_listeners(&mut self, player: &PlayerAdvancementsModel) {
        self.players.remove(&player.key);
    }

    pub fn listener_count(&self, player: &PlayerAdvancementsModel) -> usize {
        self.players.get(&player.key).map_or(0, HashSet::len)
    }

    pub fn tracked_player_count(&self) -> usize {
        self.players.len()
    }

    pub fn trigger(
        &self,
        player: &mut ServerPlayerModel,
        matcher: impl Fn(&SimpleTriggerInstanceModel) -> bool,
    ) -> Vec<AwardModel> {
        let Some(all_listeners) = self.players.get(&player.advancements.key) else {
            return Vec::new();
        };
        if all_listeners.is_empty() {
            return Vec::new();
        }

        let player_context = EntityPredicateModel::create_context(player);
        let listeners = all_listeners
            .iter()
            .filter(|listener| matcher(&listener.trigger))
            .filter(|listener| {
                listener
                    .trigger
                    .player
                    .as_ref()
                    .is_none_or(|predicate| predicate.matches(&player_context))
            })
            .cloned()
            .collect::<Vec<_>>();

        for listener in &listeners {
            listener.run(&mut player.advancements);
        }

        listeners
            .into_iter()
            .map(|listener| AwardModel {
                advancement: listener.advancement,
                criterion: listener.criterion,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub match_group: &'static str,
}

impl SimpleTriggerInstanceModel {
    pub fn new(player: Option<ContextAwarePredicateModel>, match_group: &'static str) -> Self {
        Self {
            player,
            match_group,
        }
    }

    pub fn validate(&self, validator: &ValidationContextSourceModel) -> Vec<String> {
        self.player
            .as_ref()
            .map(|predicate| {
                ValidatableModel::validate(&validator.entity_context, "player", predicate)
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CriterionListenerModel {
    pub trigger: SimpleTriggerInstanceModel,
    pub advancement: Identifier,
    pub criterion: String,
}

impl CriterionListenerModel {
    pub fn new(
        trigger: SimpleTriggerInstanceModel,
        advancement: Identifier,
        criterion: &str,
    ) -> Self {
        Self {
            trigger,
            advancement,
            criterion: criterion.to_string(),
        }
    }

    fn run(&self, player: &mut PlayerAdvancementsModel) {
        player.award(self.advancement.clone(), self.criterion.clone());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAdvancementsModel {
    key: PlayerAdvancementsKey,
    awards: Vec<AwardModel>,
}

impl PlayerAdvancementsModel {
    pub fn new(object_id: u64, _logical_name: &'static str) -> Self {
        Self {
            key: PlayerAdvancementsKey { object_id },
            awards: Vec::new(),
        }
    }

    pub fn awards(&self) -> &[AwardModel] {
        &self.awards
    }

    fn award(&mut self, advancement: Identifier, criterion: String) {
        self.awards.push(AwardModel {
            advancement,
            criterion,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    entity_id: &'static str,
    advancements: PlayerAdvancementsModel,
}

impl ServerPlayerModel {
    pub fn new(
        entity_id: &'static str,
        object_id: u64,
        logical_advancements_name: &'static str,
    ) -> Self {
        Self {
            entity_id,
            advancements: PlayerAdvancementsModel::new(object_id, logical_advancements_name),
        }
    }

    pub fn advancements(&self) -> &PlayerAdvancementsModel {
        &self.advancements
    }
}

#[derive(Debug, Clone)]
struct PlayerAdvancementsKey {
    object_id: u64,
}

impl PartialEq for PlayerAdvancementsKey {
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl Eq for PlayerAdvancementsKey {}

impl Hash for PlayerAdvancementsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object_id.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwardModel {
    pub advancement: Identifier,
    pub criterion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextAwarePredicateModel {
    requirement: PlayerPredicateRequirement,
}

impl ContextAwarePredicateModel {
    pub fn player_entity(entity_id: &'static str) -> Self {
        Self {
            requirement: PlayerPredicateRequirement::EntityId(entity_id),
        }
    }

    pub fn invalid(problem: &'static str) -> Self {
        Self {
            requirement: PlayerPredicateRequirement::Invalid(problem),
        }
    }

    pub fn panic_if_evaluated() -> Self {
        Self {
            requirement: PlayerPredicateRequirement::PanicIfEvaluated,
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        match self.requirement {
            PlayerPredicateRequirement::EntityId(expected) => context.entity_id == expected,
            PlayerPredicateRequirement::Invalid(_) => true,
            PlayerPredicateRequirement::PanicIfEvaluated => {
                panic!("player predicate should not have been evaluated")
            }
        }
    }

    fn validate(&self, label: &str) -> Vec<String> {
        match self.requirement {
            PlayerPredicateRequirement::Invalid(problem) => {
                vec![format!("{label}: {problem}")]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlayerPredicateRequirement {
    EntityId(&'static str),
    Invalid(&'static str),
    PanicIfEvaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootContextModel {
    entity_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(player: &ServerPlayerModel) -> LootContextModel {
        LootContextModel {
            entity_id: player.entity_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationContextSourceModel {
    entity_context: EntityValidationContextModel,
}

impl ValidationContextSourceModel {
    pub fn new(entity_context: EntityValidationContextModel) -> Self {
        Self { entity_context }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityValidationContextModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatableModel;

impl ValidatableModel {
    fn validate(
        _context: &EntityValidationContextModel,
        label: &str,
        predicate: &ContextAwarePredicateModel,
    ) -> Vec<String> {
        predicate.validate(label)
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instance(match_group: &'static str) -> SimpleTriggerInstanceModel {
        SimpleTriggerInstanceModel::new(None, match_group)
    }

    fn listener(
        match_group: &'static str,
        advancement: &str,
        criterion: &str,
    ) -> CriterionListenerModel {
        CriterionListenerModel::new(instance(match_group), id(advancement), criterion)
    }

    #[test]
    fn player_listener_sets_are_keyed_by_advancements_identity() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let first = PlayerAdvancementsModel::new(1, "same logical player");
        let same_logical_different_identity =
            PlayerAdvancementsModel::new(2, "same logical player");
        let first_listener = listener("hit", "minecraft:story/root", "a");
        let second_listener = listener("hit", "minecraft:story/root", "b");

        trigger.add_player_listener(&first, first_listener.clone());
        trigger.add_player_listener(&same_logical_different_identity, second_listener);
        trigger.add_player_listener(&first, first_listener);

        assert_eq!(trigger.tracked_player_count(), 2);
        assert_eq!(trigger.listener_count(&first), 1);
        assert_eq!(trigger.listener_count(&same_logical_different_identity), 1);
    }

    #[test]
    fn listener_removal_deletes_empty_player_entry() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let player = PlayerAdvancementsModel::new(1, "player");
        let first = listener("hit", "minecraft:story/root", "a");
        let second = listener("hit", "minecraft:story/root", "b");

        trigger.add_player_listener(&player, first.clone());
        trigger.add_player_listener(&player, second.clone());
        trigger.remove_player_listener(&player, &first);
        assert_eq!(trigger.listener_count(&player), 1);
        trigger.remove_player_listener(&player, &second);

        assert_eq!(trigger.listener_count(&player), 0);
        assert_eq!(trigger.tracked_player_count(), 0);
    }

    #[test]
    fn remove_player_listeners_removes_all_for_that_player_only() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let first = PlayerAdvancementsModel::new(1, "first");
        let second = PlayerAdvancementsModel::new(2, "second");

        trigger.add_player_listener(&first, listener("hit", "minecraft:story/root", "a"));
        trigger.add_player_listener(&second, listener("hit", "minecraft:story/root", "b"));
        trigger.remove_player_listeners(&first);

        assert_eq!(trigger.listener_count(&first), 0);
        assert_eq!(trigger.listener_count(&second), 1);
    }

    #[test]
    fn trigger_runs_only_matcher_and_player_predicate_accepted_listeners() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let mut player = ServerPlayerModel::new("player-1", 1, "player");
        trigger.add_player_listener(
            player.advancements(),
            CriterionListenerModel::new(
                SimpleTriggerInstanceModel::new(
                    Some(ContextAwarePredicateModel::player_entity("player-1")),
                    "hit",
                ),
                id("minecraft:story/root"),
                "accepted",
            ),
        );
        trigger.add_player_listener(
            player.advancements(),
            CriterionListenerModel::new(
                SimpleTriggerInstanceModel::new(
                    Some(ContextAwarePredicateModel::player_entity("other")),
                    "hit",
                ),
                id("minecraft:story/root"),
                "wrong_player",
            ),
        );
        trigger.add_player_listener(
            player.advancements(),
            listener("miss", "minecraft:story/root", "wrong_matcher"),
        );

        let awards = trigger.trigger(&mut player, |instance| instance.match_group == "hit");

        assert_eq!(
            awards,
            vec![AwardModel {
                advancement: id("minecraft:story/root"),
                criterion: "accepted".to_string()
            }]
        );
        assert_eq!(player.advancements().awards(), awards);
    }

    #[test]
    fn matcher_is_checked_before_player_predicate() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let mut player = ServerPlayerModel::new("player-1", 1, "player");
        trigger.add_player_listener(
            player.advancements(),
            CriterionListenerModel::new(
                SimpleTriggerInstanceModel::new(
                    Some(ContextAwarePredicateModel::panic_if_evaluated()),
                    "miss",
                ),
                id("minecraft:story/root"),
                "criterion",
            ),
        );

        assert!(trigger
            .trigger(&mut player, |instance| instance.match_group == "hit")
            .is_empty());
    }

    #[test]
    fn trigger_copies_matching_listeners_before_awarding() {
        let mut trigger = SimpleCriterionTriggerModel::default();
        let mut player = ServerPlayerModel::new("player-1", 1, "player");
        trigger.add_player_listener(&player.advancements, listener("hit", "minecraft:a", "one"));
        trigger.add_player_listener(&player.advancements, listener("hit", "minecraft:b", "two"));

        let awards = trigger.trigger(&mut player, |instance| instance.match_group == "hit");
        trigger.remove_player_listeners(player.advancements());

        assert_eq!(awards.len(), 2);
        assert_eq!(player.advancements().awards().len(), 2);
    }

    #[test]
    fn simple_instance_validation_uses_player_label_and_entity_context() {
        let instance = SimpleTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("missing parameter")),
            "hit",
        );
        let validator = ValidationContextSourceModel::new(EntityValidationContextModel);

        assert_eq!(
            instance.validate(&validator),
            vec!["player: missing parameter".to_string()]
        );
        assert!(SimpleTriggerInstanceModel::new(None, "hit")
            .validate(&validator)
            .is_empty());
    }
}
