use std::collections::{BTreeMap, BTreeSet};

use crate::network::play::ClientboundAdvancementsPacket;
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementDisplay {
    pub title: String,
    pub description: String,
    pub frame: AdvancementFrame,
    pub show_toast: bool,
    pub announce_chat: bool,
    pub hidden: bool,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementFrame {
    Task,
    Challenge,
    Goal,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementRewards {
    pub experience: i32,
    pub loot: Vec<Identifier>,
    pub recipes: Vec<Identifier>,
    pub function: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementDefinition {
    pub id: Identifier,
    pub parent: Option<Identifier>,
    pub display: Option<AdvancementDisplay>,
    pub rewards: AdvancementRewards,
    pub criteria: BTreeSet<String>,
    pub requirements: Vec<Vec<String>>,
    pub sends_telemetry_event: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementProgress {
    criteria: BTreeMap<String, Option<u64>>,
    requirements: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerAdvancementSet {
    progress: BTreeMap<Identifier, AdvancementProgress>,
    dirty: BTreeSet<Identifier>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerRecipeUnlocks {
    known: BTreeSet<Identifier>,
    highlight: BTreeSet<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDefinition {
    pub id: Identifier,
    pub special: bool,
    pub show_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeUnlockEvent {
    pub recipe: Identifier,
    pub show_notification: bool,
    pub highlight: bool,
    pub advancement_rewards: Vec<AdvancementRewardEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementRewardEvent {
    pub advancement: Identifier,
    pub experience: i32,
    pub loot: Vec<Identifier>,
    pub recipes: Vec<Identifier>,
    pub function: Option<Identifier>,
    pub announce_chat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeUnlockedCriterion {
    pub advancement: Identifier,
    pub criterion: String,
    pub recipe: Identifier,
}

impl AdvancementDefinition {
    pub fn all_of(
        id: &str,
        parent: Option<&str>,
        criteria: &[&str],
        rewards: AdvancementRewards,
        display: Option<AdvancementDisplay>,
    ) -> Result<Self, String> {
        if criteria.is_empty() {
            return Err("advancement criteria cannot be empty".to_string());
        }
        let criteria_set = criteria
            .iter()
            .map(|criterion| criterion.to_string())
            .collect();
        Ok(Self {
            id: Identifier::parse(id)?,
            parent: parent.map(Identifier::parse).transpose()?,
            display,
            rewards,
            criteria: criteria_set,
            requirements: criteria
                .iter()
                .map(|criterion| vec![criterion.to_string()])
                .collect(),
            sends_telemetry_event: false,
        })
    }

    pub fn any_of(
        id: &str,
        parent: Option<&str>,
        criteria: &[&str],
        display: Option<AdvancementDisplay>,
    ) -> Result<Self, String> {
        if criteria.is_empty() {
            return Err("advancement criteria cannot be empty".to_string());
        }
        Ok(Self {
            id: Identifier::parse(id)?,
            parent: parent.map(Identifier::parse).transpose()?,
            display,
            rewards: AdvancementRewards::default(),
            criteria: criteria
                .iter()
                .map(|criterion| criterion.to_string())
                .collect(),
            requirements: vec![criteria
                .iter()
                .map(|criterion| criterion.to_string())
                .collect()],
            sends_telemetry_event: false,
        })
    }

    pub fn visible_to(
        &self,
        player: &PlayerAdvancementSet,
        _tree: &[AdvancementDefinition],
    ) -> bool {
        let Some(display) = &self.display else {
            return false;
        };
        if !display.hidden {
            return true;
        }
        player.has_progress(&self.id)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| player.is_done(parent))
    }
}

impl AdvancementProgress {
    pub fn new(definition: &AdvancementDefinition) -> Self {
        Self {
            criteria: definition
                .criteria
                .iter()
                .map(|criterion| (criterion.clone(), None))
                .collect(),
            requirements: definition.requirements.clone(),
        }
    }

    pub fn grant(&mut self, criterion: &str, obtained_epoch_seconds: u64) -> bool {
        match self.criteria.get_mut(criterion) {
            Some(entry) if entry.is_none() => {
                *entry = Some(obtained_epoch_seconds);
                true
            }
            _ => false,
        }
    }

    pub fn revoke(&mut self, criterion: &str) -> bool {
        match self.criteria.get_mut(criterion) {
            Some(entry) if entry.is_some() => {
                *entry = None;
                true
            }
            _ => false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.requirements.iter().all(|group| {
            group
                .iter()
                .any(|criterion| self.criteria.get(criterion).is_some_and(Option::is_some))
        })
    }

    pub fn completed_criteria(&self) -> Vec<String> {
        self.criteria
            .iter()
            .filter_map(|(criterion, obtained)| obtained.is_some().then_some(criterion.clone()))
            .collect()
    }

    pub fn to_json_fragment(&self) -> String {
        let mut out = String::from("{\"criteria\":{");
        for (index, (criterion, obtained)) in self
            .criteria
            .iter()
            .filter_map(|(criterion, obtained)| obtained.map(|time| (criterion, time)))
            .enumerate()
        {
            if index > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(criterion);
            out.push_str("\":");
            out.push_str(&obtained.to_string());
        }
        out.push_str("},\"done\":");
        out.push_str(if self.is_done() { "true" } else { "false" });
        out.push('}');
        out
    }
}

impl PlayerAdvancementSet {
    pub fn ensure(&mut self, definition: &AdvancementDefinition) {
        self.progress
            .entry(definition.id.clone())
            .or_insert_with(|| AdvancementProgress::new(definition));
    }

    pub fn grant(
        &mut self,
        definition: &AdvancementDefinition,
        criterion: &str,
        obtained_epoch_seconds: u64,
    ) -> Option<AdvancementRewardEvent> {
        self.ensure(definition);
        let progress = self.progress.get_mut(&definition.id)?;
        let was_done = progress.is_done();
        if !progress.grant(criterion, obtained_epoch_seconds) {
            return None;
        }
        self.dirty.insert(definition.id.clone());
        (!was_done && progress.is_done()).then(|| AdvancementRewardEvent {
            advancement: definition.id.clone(),
            experience: definition.rewards.experience,
            loot: definition.rewards.loot.clone(),
            recipes: definition.rewards.recipes.clone(),
            function: definition.rewards.function.clone(),
            announce_chat: definition
                .display
                .as_ref()
                .is_some_and(|display| display.announce_chat),
        })
    }

    pub fn revoke(&mut self, definition: &AdvancementDefinition, criterion: &str) -> bool {
        self.ensure(definition);
        let changed = self
            .progress
            .get_mut(&definition.id)
            .is_some_and(|progress| progress.revoke(criterion));
        if changed {
            self.dirty.insert(definition.id.clone());
        }
        changed
    }

    pub fn is_done(&self, id: &Identifier) -> bool {
        self.progress
            .get(id)
            .is_some_and(AdvancementProgress::is_done)
    }

    pub fn has_progress(&self, id: &Identifier) -> bool {
        self.progress
            .get(id)
            .is_some_and(|progress| progress.criteria.values().any(Option::is_some))
    }

    pub fn to_packet(
        &mut self,
        definitions: &[AdvancementDefinition],
    ) -> ClientboundAdvancementsPacket {
        let added = definitions
            .iter()
            .filter(|definition| self.dirty.contains(&definition.id))
            .map(|definition| definition.id.clone())
            .collect();
        self.dirty.clear();
        ClientboundAdvancementsPacket {
            reset: false,
            added,
            removed: Vec::new(),
        }
    }

    pub fn to_vanilla_json(&self, data_version: i32) -> String {
        let mut out = String::from("{\"DataVersion\":");
        out.push_str(&data_version.to_string());
        out.push_str(",\"advancements\":{");
        for (index, (id, progress)) in self.progress.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(&id.to_string());
            out.push_str("\":");
            out.push_str(&progress.to_json_fragment());
        }
        out.push_str("}}");
        out
    }
}

impl PlayerRecipeUnlocks {
    pub fn contains(&self, recipe: &Identifier) -> bool {
        self.known.contains(recipe)
    }

    pub fn highlighted(&self, recipe: &Identifier) -> bool {
        self.highlight.contains(recipe)
    }

    pub fn unlock_recipes(
        &mut self,
        recipes: &[RecipeDefinition],
        advancements: &[AdvancementDefinition],
        triggers: &[RecipeUnlockedCriterion],
        player_advancements: &mut PlayerAdvancementSet,
        obtained_epoch_seconds: u64,
    ) -> Vec<RecipeUnlockEvent> {
        let mut events = Vec::new();
        for recipe in recipes {
            if recipe.special || self.known.contains(&recipe.id) {
                continue;
            }
            self.known.insert(recipe.id.clone());
            self.highlight.insert(recipe.id.clone());
            let advancement_rewards = trigger_recipe_unlocked(
                &recipe.id,
                advancements,
                triggers,
                player_advancements,
                obtained_epoch_seconds,
            );
            events.push(RecipeUnlockEvent {
                recipe: recipe.id.clone(),
                show_notification: recipe.show_notification,
                highlight: true,
                advancement_rewards,
            });
        }
        events
    }

    pub fn remove_recipes(&mut self, recipes: &[Identifier]) -> Vec<Identifier> {
        let mut removed = Vec::new();
        for recipe in recipes {
            if self.known.remove(recipe) {
                self.highlight.remove(recipe);
                removed.push(recipe.clone());
            }
        }
        removed
    }

    pub fn clear_highlight(&mut self, recipe: &Identifier) {
        self.highlight.remove(recipe);
    }

    pub fn pack(&self) -> (Vec<Identifier>, Vec<Identifier>) {
        (
            self.known.iter().cloned().collect(),
            self.highlight.iter().cloned().collect(),
        )
    }

    pub fn load_untrusted(
        known: Vec<Identifier>,
        highlight: Vec<Identifier>,
        validator: impl Fn(&Identifier) -> bool,
    ) -> Self {
        let known = known
            .into_iter()
            .filter(|recipe| validator(recipe))
            .collect::<BTreeSet<_>>();
        let highlight = highlight
            .into_iter()
            .filter(|recipe| known.contains(recipe))
            .collect();
        Self { known, highlight }
    }
}

pub fn trigger_recipe_unlocked(
    recipe: &Identifier,
    advancements: &[AdvancementDefinition],
    triggers: &[RecipeUnlockedCriterion],
    player_advancements: &mut PlayerAdvancementSet,
    obtained_epoch_seconds: u64,
) -> Vec<AdvancementRewardEvent> {
    let mut rewards = Vec::new();
    for trigger in triggers.iter().filter(|trigger| trigger.recipe == *recipe) {
        let Some(advancement) = advancements
            .iter()
            .find(|advancement| advancement.id == trigger.advancement)
        else {
            continue;
        };
        if let Some(reward) =
            player_advancements.grant(advancement, &trigger.criterion, obtained_epoch_seconds)
        {
            rewards.push(reward);
        }
    }
    rewards
}

pub fn assign_tree_layout(definitions: &mut [AdvancementDefinition]) {
    let mut child_counts: BTreeMap<Option<Identifier>, i32> = BTreeMap::new();
    for definition in definitions {
        let index = child_counts.entry(definition.parent.clone()).or_insert(0);
        if let Some(display) = &mut definition.display {
            display.x = definition.parent.as_ref().map_or(0, |_| 1);
            display.y = *index;
        }
        *index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(hidden: bool, announce_chat: bool) -> AdvancementDisplay {
        AdvancementDisplay {
            title: "Title".to_string(),
            description: "Description".to_string(),
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_chat,
            hidden,
            x: 0,
            y: 0,
        }
    }

    #[test]
    fn advancement_progress_grants_revokes_requirements_and_rewards() {
        let rewards = AdvancementRewards {
            experience: 5,
            loot: vec![Identifier::parse("minecraft:loot").unwrap()],
            recipes: vec![Identifier::parse("minecraft:stone").unwrap()],
            function: Some(Identifier::parse("minecraft:reward").unwrap()),
        };
        let definition = AdvancementDefinition::all_of(
            "minecraft:story/root",
            None,
            &["tick"],
            rewards,
            Some(display(false, true)),
        )
        .unwrap();
        let mut player = PlayerAdvancementSet::default();
        let reward = player.grant(&definition, "tick", 1_700_000_000).unwrap();
        assert_eq!(reward.experience, 5);
        assert!(reward.announce_chat);
        assert!(player.is_done(&definition.id));
        assert_eq!(
            player
                .progress
                .get(&definition.id)
                .unwrap()
                .completed_criteria(),
            vec!["tick".to_string()]
        );
        assert!(player.revoke(&definition, "tick"));
        assert!(!player.is_done(&definition.id));
    }

    #[test]
    fn advancement_visibility_layout_packet_and_persistence_follow_vanilla_shapes() {
        let mut definitions = vec![
            AdvancementDefinition::all_of(
                "minecraft:story/root",
                None,
                &["tick"],
                AdvancementRewards::default(),
                Some(AdvancementDisplay {
                    frame: AdvancementFrame::Challenge,
                    ..display(false, false)
                }),
            )
            .unwrap(),
            AdvancementDefinition::any_of(
                "minecraft:story/hidden",
                Some("minecraft:story/root"),
                &["stone", "iron"],
                Some(AdvancementDisplay {
                    frame: AdvancementFrame::Goal,
                    ..display(true, false)
                }),
            )
            .unwrap(),
        ];
        assign_tree_layout(&mut definitions);
        assert_eq!(definitions[0].display.as_ref().unwrap().x, 0);
        assert_eq!(definitions[1].display.as_ref().unwrap().x, 1);

        let mut player = PlayerAdvancementSet::default();
        assert!(!definitions[1].visible_to(&player, &definitions));
        player.grant(&definitions[0], "tick", 1).unwrap();
        assert!(definitions[1].visible_to(&player, &definitions));
        player.grant(&definitions[1], "iron", 2);
        assert!(player.is_done(&definitions[1].id));

        let packet = player.to_packet(&definitions);
        assert_eq!(packet.added.len(), 2);
        assert_eq!(packet.removed.len(), 0);
        assert!(!packet.reset);
        assert!(player.to_packet(&definitions).added.is_empty());

        let json = player.to_vanilla_json(4189);
        assert!(json.contains("\"DataVersion\":4189"));
        assert!(json.contains("\"minecraft:story/root\""));
        assert!(json.contains("\"done\":true"));
    }

    #[test]
    fn recipe_unlocks_skip_special_known_recipes_and_trigger_advancement_criteria() {
        let recipe = Identifier::parse("minecraft:oak_planks").unwrap();
        let special = Identifier::parse("minecraft:special").unwrap();
        let advancement = AdvancementDefinition::all_of(
            "minecraft:recipes/building_blocks/oak_planks",
            None,
            &["has_the_recipe"],
            AdvancementRewards {
                experience: 1,
                loot: Vec::new(),
                recipes: vec![recipe.clone()],
                function: None,
            },
            Some(display(false, true)),
        )
        .unwrap();
        let triggers = vec![RecipeUnlockedCriterion {
            advancement: advancement.id.clone(),
            criterion: "has_the_recipe".to_string(),
            recipe: recipe.clone(),
        }];
        let mut unlocks = PlayerRecipeUnlocks::default();
        let mut progress = PlayerAdvancementSet::default();

        let events = unlocks.unlock_recipes(
            &[
                RecipeDefinition {
                    id: recipe.clone(),
                    special: false,
                    show_notification: true,
                },
                RecipeDefinition {
                    id: special.clone(),
                    special: true,
                    show_notification: true,
                },
            ],
            &[advancement.clone()],
            &triggers,
            &mut progress,
            10,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].recipe, recipe);
        assert!(events[0].show_notification);
        assert!(events[0].highlight);
        assert_eq!(events[0].advancement_rewards[0].experience, 1);
        assert!(unlocks.contains(&events[0].recipe));
        assert!(unlocks.highlighted(&events[0].recipe));
        assert!(!unlocks.contains(&special));
        assert!(progress.is_done(&advancement.id));

        assert!(unlocks
            .unlock_recipes(
                &[RecipeDefinition {
                    id: events[0].recipe.clone(),
                    special: false,
                    show_notification: true,
                }],
                &[advancement],
                &triggers,
                &mut progress,
                11,
            )
            .is_empty());
    }

    #[test]
    fn recipe_unlocks_remove_clear_highlight_and_load_only_valid_recipes() {
        let stone = Identifier::parse("minecraft:stone").unwrap();
        let dirt = Identifier::parse("minecraft:dirt").unwrap();
        let bad = Identifier::parse("minecraft:bad").unwrap();
        let mut unlocks = PlayerRecipeUnlocks::load_untrusted(
            vec![stone.clone(), bad.clone()],
            vec![stone.clone(), dirt.clone()],
            |recipe| recipe != &bad,
        );
        assert!(unlocks.contains(&stone));
        assert!(!unlocks.contains(&bad));
        assert!(unlocks.highlighted(&stone));
        assert!(!unlocks.highlighted(&dirt));

        unlocks.clear_highlight(&stone);
        assert!(!unlocks.highlighted(&stone));
        assert_eq!(unlocks.remove_recipes(&[stone.clone(), dirt]), vec![stone]);
        let (known, highlight) = unlocks.pack();
        assert!(known.is_empty());
        assert!(highlight.is_empty());
    }
}
