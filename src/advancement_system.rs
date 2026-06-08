use std::collections::{BTreeMap, BTreeSet};

use crate::advancement_criteria::AdvancementRequirementsModel;
use crate::chat_formatting::ChatFormatting;
use crate::network::play::{
    AdvancementHolderData, AdvancementProgressData, ClientboundAdvancementsPacket,
    CriterionProgressData,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementDisplay {
    pub icon: Identifier,
    pub title: String,
    pub description: String,
    pub background: Option<Identifier>,
    pub frame: AdvancementFrame,
    pub show_toast: bool,
    pub announce_chat: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementFrame {
    Task,
    Challenge,
    Goal,
}

impl AdvancementFrame {
    pub const VALUES: [Self; 3] = [Self::Task, Self::Challenge, Self::Goal];

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Challenge => "challenge",
            Self::Goal => "goal",
        }
    }

    pub fn chat_color(self) -> ChatFormatting {
        match self {
            Self::Task | Self::Goal => ChatFormatting::Green,
            Self::Challenge => ChatFormatting::DarkPurple,
        }
    }

    pub fn display_name_translation_key(self) -> String {
        format!("advancements.toast.{}", self.serialized_name())
    }

    pub fn announcement_translation_key(self) -> String {
        format!("chat.type.advancement.{}", self.serialized_name())
    }

    pub fn create_announcement(self, player_display_name: &str, advancement_name: &str) -> String {
        format!(
            "{} {} {}",
            self.announcement_translation_key(),
            player_display_name,
            advancement_name
        )
    }

    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "task" => Some(Self::Task),
            "challenge" => Some(Self::Challenge),
            "goal" => Some(Self::Goal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionProgressModel {
    obtained_epoch_millis: Option<i64>,
}

impl CriterionProgressModel {
    pub fn new() -> Self {
        Self {
            obtained_epoch_millis: None,
        }
    }

    pub fn from_obtained_epoch_millis(obtained_epoch_millis: i64) -> Self {
        Self {
            obtained_epoch_millis: Some(obtained_epoch_millis),
        }
    }

    pub fn is_done(&self) -> bool {
        self.obtained_epoch_millis.is_some()
    }

    pub fn grant_at_epoch_millis(&mut self, obtained_epoch_millis: i64) {
        self.obtained_epoch_millis = Some(obtained_epoch_millis);
    }

    pub fn revoke(&mut self) {
        self.obtained_epoch_millis = None;
    }

    pub fn get_obtained_epoch_millis(&self) -> Option<i64> {
        self.obtained_epoch_millis
    }

    pub fn to_network_data(&self) -> CriterionProgressData {
        CriterionProgressData {
            obtained_epoch_millis: self.obtained_epoch_millis,
        }
    }

    pub fn from_network_data(data: CriterionProgressData) -> Self {
        Self {
            obtained_epoch_millis: data.obtained_epoch_millis,
        }
    }
}

impl std::fmt::Display for CriterionProgressModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.obtained_epoch_millis {
            Some(epoch_millis) => write!(formatter, "CriterionProgress{{obtained={epoch_millis}}}"),
            None => write!(formatter, "CriterionProgress{{obtained=false}}"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementRewards {
    pub experience: i32,
    pub loot: Vec<Identifier>,
    pub recipes: Vec<Identifier>,
    pub function: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub fn from_json(id: &str, raw: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|err| format!("invalid advancement JSON for {id}: {err}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| format!("advancement {id} must be a JSON object"))?;
        let criteria_object = object
            .get("criteria")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("advancement {id} missing criteria object"))?;
        if criteria_object.is_empty() {
            return Err(format!("advancement {id} criteria cannot be empty"));
        }
        let criteria = criteria_object.keys().cloned().collect::<BTreeSet<_>>();
        let requirements = match object.get("requirements") {
            Some(value) => parse_advancement_requirements(id, value, &criteria)?,
            None => criteria
                .iter()
                .map(|criterion| vec![criterion.clone()])
                .collect(),
        };
        let rewards = object
            .get("rewards")
            .map(parse_advancement_rewards)
            .transpose()?
            .unwrap_or_default();
        let display = object
            .get("display")
            .map(parse_advancement_display)
            .transpose()?;
        Ok(Self {
            id: Identifier::parse(id)?,
            parent: object
                .get("parent")
                .map(|value| {
                    json_string(value, "parent").and_then(|parent| Identifier::parse(&parent))
                })
                .transpose()?,
            display,
            rewards,
            criteria,
            requirements,
            sends_telemetry_event: object
                .get("sends_telemetry_event")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        })
    }

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

fn parse_advancement_requirements(
    id: &str,
    value: &serde_json::Value,
    criteria: &BTreeSet<String>,
) -> Result<Vec<Vec<String>>, String> {
    let outer = value
        .as_array()
        .ok_or_else(|| format!("advancement {id} requirements must be an array"))?;
    let mut requirements = Vec::new();
    for group in outer {
        let inner = group
            .as_array()
            .ok_or_else(|| format!("advancement {id} requirement group must be an array"))?;
        let mut parsed_group = Vec::new();
        for criterion in inner {
            let criterion = json_string(criterion, "requirement criterion")?;
            if !criteria.contains(&criterion) {
                return Err(format!(
                    "advancement {id} requirement references unknown criterion {criterion}"
                ));
            }
            parsed_group.push(criterion);
        }
        requirements.push(parsed_group);
    }
    let requirements = AdvancementRequirementsModel::new(requirements);
    requirements
        .validate(criteria)
        .map_err(|err| format!("advancement {id} {err}"))?;
    Ok(requirements.requirements().to_vec())
}

fn parse_advancement_rewards(value: &serde_json::Value) -> Result<AdvancementRewards, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "advancement rewards must be an object".to_string())?;
    Ok(AdvancementRewards {
        experience: object
            .get("experience")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
            .try_into()
            .map_err(|_| "advancement reward experience out of i32 range".to_string())?,
        loot: parse_identifier_array(object.get("loot"), "loot")?,
        recipes: parse_identifier_array(object.get("recipes"), "recipes")?,
        function: object
            .get("function")
            .map(|value| json_string(value, "function").and_then(|id| Identifier::parse(&id)))
            .transpose()?,
    })
}

fn parse_advancement_display(value: &serde_json::Value) -> Result<AdvancementDisplay, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "advancement display must be an object".to_string())?;
    Ok(AdvancementDisplay {
        icon: parse_display_icon(
            object
                .get("icon")
                .ok_or_else(|| "advancement display missing icon".to_string())?,
        )?,
        title: stringify_component(
            object
                .get("title")
                .ok_or_else(|| "advancement display missing title".to_string())?,
        )?,
        description: stringify_component(
            object
                .get("description")
                .ok_or_else(|| "advancement display missing description".to_string())?,
        )?,
        background: object
            .get("background")
            .map(|value| json_string(value, "background").and_then(|id| Identifier::parse(&id)))
            .transpose()?,
        frame: match object
            .get("frame")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("task")
        {
            "task" => AdvancementFrame::Task,
            "challenge" => AdvancementFrame::Challenge,
            "goal" => AdvancementFrame::Goal,
            frame => return Err(format!("unknown advancement frame {frame}")),
        },
        show_toast: object
            .get("show_toast")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        announce_chat: object
            .get("announce_to_chat")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        hidden: object
            .get("hidden")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        x: 0.0,
        y: 0.0,
    })
}

fn parse_display_icon(value: &serde_json::Value) -> Result<Identifier, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "advancement display icon must be an object".to_string())?;
    let id = object
        .get("id")
        .ok_or_else(|| "advancement display icon missing id".to_string())?;
    Identifier::parse(&json_string(id, "icon id")?)
}

fn parse_identifier_array(
    value: Option<&serde_json::Value>,
    field: &str,
) -> Result<Vec<Identifier>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| format!("advancement rewards {field} must be an array"))?;
    array
        .iter()
        .map(|value| json_string(value, field).and_then(|id| Identifier::parse(&id)))
        .collect()
}

fn json_string(value: &serde_json::Value, field: &str) -> Result<String, String> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| format!("{field} must be a string"))
}

fn stringify_component(value: &serde_json::Value) -> Result<String, String> {
    if let Some(text) = value.as_str() {
        return Ok(text.to_string());
    }
    let object = value
        .as_object()
        .ok_or_else(|| "component must be a string or object".to_string())?;
    if let Some(text) = object.get("text").and_then(serde_json::Value::as_str) {
        return Ok(text.to_string());
    }
    if let Some(translate) = object.get("translate").and_then(serde_json::Value::as_str) {
        return Ok(translate.to_string());
    }
    Err("component object must contain text or translate".to_string())
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

    pub fn drain_packet(
        &mut self,
        definitions: &[AdvancementDefinition],
    ) -> ClientboundAdvancementsPacket {
        let added = definitions
            .iter()
            .filter(|definition| self.dirty.contains(&definition.id))
            .map(|definition| {
                AdvancementHolderData::minimal(
                    definition.id.clone(),
                    definition.parent.clone(),
                    definition.requirements.clone(),
                    definition.sends_telemetry_event,
                )
            })
            .collect();
        let progress = self
            .dirty
            .iter()
            .filter_map(|id| self.progress.get(id).map(|progress| (id, progress)))
            .map(|(id, progress)| {
                (
                    id.clone(),
                    AdvancementProgressData {
                        criteria: progress
                            .criteria
                            .iter()
                            .map(|(criterion, obtained)| {
                                (
                                    criterion.clone(),
                                    CriterionProgressData {
                                        obtained_epoch_millis: obtained
                                            .map(|seconds| seconds.saturating_mul(1000) as i64),
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        self.dirty.clear();
        ClientboundAdvancementsPacket {
            reset: false,
            added,
            removed: Vec::new(),
            progress,
            show_advancements: true,
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
            display.x = definition.parent.as_ref().map_or(0.0, |_| 1.0);
            display.y = *index as f32;
        }
        *index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(hidden: bool, announce_chat: bool) -> AdvancementDisplay {
        AdvancementDisplay {
            icon: Identifier::parse("minecraft:stone").unwrap(),
            title: "Title".to_string(),
            description: "Description".to_string(),
            background: None,
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_chat,
            hidden,
            x: 0.0,
            y: 0.0,
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
        assert_eq!(definitions[0].display.as_ref().unwrap().x, 0.0);
        assert_eq!(definitions[1].display.as_ref().unwrap().x, 1.0);

        let mut player = PlayerAdvancementSet::default();
        assert!(!definitions[1].visible_to(&player, &definitions));
        player.grant(&definitions[0], "tick", 1).unwrap();
        assert!(definitions[1].visible_to(&player, &definitions));
        player.grant(&definitions[1], "iron", 2);
        assert!(player.is_done(&definitions[1].id));

        let packet = player.drain_packet(&definitions);
        assert_eq!(packet.added.len(), 2);
        assert_eq!(packet.removed.len(), 0);
        assert!(!packet.reset);
        assert!(player.drain_packet(&definitions).added.is_empty());

        let json = player.to_vanilla_json(4189);
        assert!(json.contains("\"DataVersion\":4189"));
        assert!(json.contains("\"minecraft:story/root\""));
        assert!(json.contains("\"done\":true"));
    }

    #[test]
    fn advancement_json_loader_decodes_vanilla_codec_fields() {
        let mut root = AdvancementDefinition::from_json(
            "minecraft:story/root",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/advancement/story/root.json"
            ),
        )
        .unwrap();
        assert_eq!(root.parent, None);
        assert_eq!(
            root.criteria,
            BTreeSet::from(["crafting_table".to_string()])
        );
        assert_eq!(root.requirements, vec![vec!["crafting_table".to_string()]]);
        assert!(root.sends_telemetry_event);
        let root_display = root.display.as_ref().unwrap();
        assert_eq!(root_display.title, "advancements.story.root.title");
        assert!(!root_display.show_toast);
        assert!(!root_display.announce_chat);
        assert_eq!(root_display.frame, AdvancementFrame::Task);

        let mine_stone = AdvancementDefinition::from_json(
            "minecraft:story/mine_stone",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/advancement/story/mine_stone.json"
            ),
        )
        .unwrap();
        assert_eq!(
            mine_stone.parent,
            Some(Identifier::parse("minecraft:story/root").unwrap())
        );
        assert_eq!(
            mine_stone.display.as_ref().unwrap().description,
            "advancements.story.mine_stone.description"
        );
        assert!(mine_stone.display.as_ref().unwrap().show_toast);
        assert!(mine_stone.display.as_ref().unwrap().announce_chat);

        let recipe = AdvancementDefinition::from_json(
            "minecraft:recipes/decorations/crafting_table",
            include_str!(
                "../../decompiled-server-26.1.2/data/minecraft/advancement/recipes/decorations/crafting_table.json"
            ),
        )
        .unwrap();
        assert_eq!(
            recipe.rewards.recipes,
            vec![Identifier::parse("minecraft:crafting_table").unwrap()]
        );
        assert!(recipe.display.is_none());

        let reward = AdvancementDefinition::from_json(
            "minecraft:test/reward",
            r#"{
                "criteria": {
                    "tick": {
                        "trigger": "minecraft:tick"
                    }
                },
                "rewards": {
                    "experience": 25,
                    "loot": [
                        "minecraft:advancements/test_reward"
                    ]
                }
            }"#,
        )
        .unwrap();
        assert_eq!(reward.rewards.experience, 25);
        assert_eq!(
            reward.rewards.loot,
            vec![Identifier::parse("minecraft:advancements/test_reward").unwrap()]
        );

        assign_tree_layout(std::slice::from_mut(&mut root));
        assert_eq!(root.display.as_ref().unwrap().x, 0.0);
    }

    #[test]
    fn advancement_json_loader_rejects_invalid_requirements() {
        assert!(
            AdvancementDefinition::from_json("minecraft:test/empty", r#"{"criteria":{}}"#)
                .unwrap_err()
                .contains("criteria cannot be empty")
        );

        assert!(AdvancementDefinition::from_json(
            "minecraft:test/bad_requirement",
            r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}},"requirements":[["missing"]]}"#
        )
        .unwrap_err()
        .contains("unknown criterion"));
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
            std::slice::from_ref(&advancement),
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

    #[test]
    fn advancement_type_model_matches_java_enum_fields_and_announcement() {
        assert_eq!(
            AdvancementFrame::VALUES.map(AdvancementFrame::serialized_name),
            ["task", "challenge", "goal"]
        );
        assert_eq!(AdvancementFrame::Task.chat_color(), ChatFormatting::Green);
        assert_eq!(
            AdvancementFrame::Challenge.chat_color(),
            ChatFormatting::DarkPurple
        );
        assert_eq!(AdvancementFrame::Goal.chat_color(), ChatFormatting::Green);
        assert_eq!(
            AdvancementFrame::Challenge.display_name_translation_key(),
            "advancements.toast.challenge"
        );
        assert_eq!(
            AdvancementFrame::Goal.announcement_translation_key(),
            "chat.type.advancement.goal"
        );
        assert_eq!(
            AdvancementFrame::Task.create_announcement("Steve", "Stone Age"),
            "chat.type.advancement.task Steve Stone Age"
        );
        assert_eq!(
            AdvancementFrame::from_serialized_name("challenge"),
            Some(AdvancementFrame::Challenge)
        );
        assert_eq!(AdvancementFrame::from_serialized_name("unknown"), None);
    }

    #[test]
    fn criterion_progress_model_matches_java_done_grant_revoke_and_network() {
        let mut progress = CriterionProgressModel::new();
        assert!(!progress.is_done());
        assert_eq!(progress.get_obtained_epoch_millis(), None);
        assert_eq!(progress.to_string(), "CriterionProgress{obtained=false}");
        assert_eq!(
            progress.to_network_data(),
            CriterionProgressData {
                obtained_epoch_millis: None
            }
        );

        progress.grant_at_epoch_millis(1_700_000_000_123);
        assert!(progress.is_done());
        assert_eq!(
            progress.get_obtained_epoch_millis(),
            Some(1_700_000_000_123)
        );
        assert_eq!(
            progress.to_string(),
            "CriterionProgress{obtained=1700000000123}"
        );
        assert_eq!(
            CriterionProgressModel::from_network_data(progress.to_network_data()),
            progress
        );

        progress.revoke();
        assert!(!progress.is_done());
        assert_eq!(
            CriterionProgressModel::from_obtained_epoch_millis(42).to_network_data(),
            CriterionProgressData {
                obtained_epoch_millis: Some(42)
            }
        );
    }

    #[test]
    fn advancement_holder_model_matches_java_id_only_identity_methods() {
        let id = Identifier::parse("minecraft:story/root").unwrap();
        let other_id = Identifier::parse("minecraft:story/mine_stone").unwrap();
        let holder =
            AdvancementHolderData::minimal(id.clone(), None, vec![vec!["a".to_string()]], false);
        let same_id_different_value = AdvancementHolderData::minimal(
            id.clone(),
            Some(other_id.clone()),
            vec![vec!["b".to_string()]],
            true,
        );
        let different_id = AdvancementHolderData::minimal(
            other_id.clone(),
            None,
            vec![vec!["a".to_string()]],
            false,
        );

        assert_ne!(holder, same_id_different_value);
        assert!(holder.java_equals_by_id(&same_id_different_value));
        assert!(!holder.java_equals_by_id(&different_id));
        assert_eq!(holder.java_hash_key(), &id);
        assert_eq!(holder.java_to_string(), "minecraft:story/root");
    }
}
