use std::collections::BTreeMap;

use crate::advancement_criteria::{AdvancementRequirementStrategy, AdvancementRequirementsModel};
use crate::advancement_display::DisplayInfoModel;
use crate::advancement_rewards::AdvancementRewardsBuilder;
use crate::advancement_system::AdvancementFrame;
use crate::advancement_system::AdvancementRewards;
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct JavaAdvancementModel {
    pub parent: Option<Identifier>,
    pub display: Option<DisplayInfoModel>,
    pub rewards: AdvancementRewards,
    pub criteria: BTreeMap<String, JavaCriterionValidationModel>,
    pub requirements: AdvancementRequirementsModel,
    pub sends_telemetry_event: bool,
    pub name: Option<DecoratedAdvancementName>,
}

impl JavaAdvancementModel {
    pub fn new(
        parent: Option<Identifier>,
        display: Option<DisplayInfoModel>,
        rewards: AdvancementRewards,
        criteria: BTreeMap<String, JavaCriterionValidationModel>,
        requirements: AdvancementRequirementsModel,
        sends_telemetry_event: bool,
    ) -> Self {
        let name = display.as_ref().map(Self::decorate_name);
        Self {
            parent,
            display,
            rewards,
            criteria,
            requirements,
            sends_telemetry_event,
            name,
        }
    }

    pub fn validate(self) -> Result<Self, String> {
        if self.criteria.is_empty() {
            return Err("Advancement criteria cannot be empty".to_string());
        }
        self.requirements.validate(
            &self
                .criteria
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
        )?;
        Ok(self)
    }

    pub fn name_for_holder(holder: &JavaAdvancementHolderModel) -> String {
        holder
            .value
            .name
            .as_ref()
            .map_or_else(|| holder.id.to_string(), |name| name.text.clone())
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn validate_criteria(&self) -> Vec<String> {
        self.criteria
            .iter()
            .flat_map(|(name, criterion)| {
                criterion
                    .validation_problems
                    .iter()
                    .map(move |problem| format!("{name}: {problem}"))
            })
            .collect()
    }

    pub fn write_network_payload(&self) -> JavaAdvancementNetworkPayload {
        JavaAdvancementNetworkPayload {
            parent: self.parent.clone(),
            display: self
                .display
                .as_ref()
                .map(DisplayInfoModel::to_network_payload),
            requirements: self.requirements.requirements().to_vec(),
            sends_telemetry_event: self.sends_telemetry_event,
        }
    }

    pub fn read_network_payload(payload: JavaAdvancementNetworkPayload) -> Self {
        Self::new(
            payload.parent,
            payload.display.map(DisplayInfoModel::from_network_payload),
            AdvancementRewards::java_empty(),
            BTreeMap::new(),
            AdvancementRequirementsModel::new(payload.requirements),
            payload.sends_telemetry_event,
        )
    }

    fn decorate_name(display: &DisplayInfoModel) -> DecoratedAdvancementName {
        let color = display.frame.chat_color();
        DecoratedAdvancementName {
            text: format!("[{}]", display.title),
            color: format!("{color:?}"),
            hover_text: format!("{}\n{}", display.title, display.description),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaCriterionValidationModel {
    validation_problems: Vec<String>,
}

impl JavaCriterionValidationModel {
    pub fn valid() -> Self {
        Self {
            validation_problems: Vec::new(),
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            validation_problems: vec![problem.to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaAdvancementHolderModel {
    pub id: Identifier,
    pub value: JavaAdvancementModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedAdvancementName {
    pub text: String,
    pub color: String,
    pub hover_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaAdvancementNetworkPayload {
    pub parent: Option<Identifier>,
    pub display: Option<crate::advancement_display::DisplayInfoNetworkPayload>,
    pub requirements: Vec<Vec<String>>,
    pub sends_telemetry_event: bool,
}

#[derive(Debug, Clone)]
pub struct JavaAdvancementBuilderModel {
    parent: Option<Identifier>,
    display: Option<DisplayInfoModel>,
    rewards: AdvancementRewards,
    criteria: BTreeMap<String, JavaCriterionValidationModel>,
    requirements: Option<AdvancementRequirementsModel>,
    requirements_strategy: AdvancementRequirementStrategy,
    sends_telemetry_event: bool,
}

impl JavaAdvancementBuilderModel {
    pub fn advancement() -> Self {
        Self::new().sends_telemetry_event()
    }

    pub fn recipe_advancement() -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        Self {
            parent: None,
            display: None,
            rewards: AdvancementRewards::java_empty(),
            criteria: BTreeMap::new(),
            requirements: None,
            requirements_strategy: AdvancementRequirementStrategy::And,
            sends_telemetry_event: false,
        }
    }

    pub fn parent(mut self, parent: Identifier) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn display(mut self, display: DisplayInfoModel) -> Self {
        self.display = Some(display);
        self
    }

    pub fn rewards(mut self, rewards: AdvancementRewards) -> Self {
        self.rewards = rewards;
        self
    }

    pub fn rewards_from_builder(self, rewards: AdvancementRewardsBuilder) -> Self {
        self.rewards(rewards.build())
    }

    pub fn add_criterion(mut self, name: &str, criterion: JavaCriterionValidationModel) -> Self {
        self.criteria.insert(name.to_string(), criterion);
        self
    }

    pub fn requirements_strategy(mut self, strategy: AdvancementRequirementStrategy) -> Self {
        self.requirements_strategy = strategy;
        self
    }

    pub fn requirements(mut self, requirements: AdvancementRequirementsModel) -> Self {
        self.requirements = Some(requirements);
        self
    }

    pub fn sends_telemetry_event(mut self) -> Self {
        self.sends_telemetry_event = true;
        self
    }

    pub fn build(self, id: Identifier) -> Result<JavaAdvancementHolderModel, String> {
        let requirements = self.requirements.unwrap_or_else(|| {
            self.requirements_strategy
                .create(self.criteria.keys().cloned())
        });
        let value = JavaAdvancementModel::new(
            self.parent,
            self.display,
            self.rewards,
            self.criteria,
            requirements,
            self.sends_telemetry_event,
        )
        .validate()?;
        Ok(JavaAdvancementHolderModel { id, value })
    }

    pub fn save(
        self,
        output: &mut Vec<JavaAdvancementHolderModel>,
        name: &str,
    ) -> Result<JavaAdvancementHolderModel, String> {
        let advancement = self.build(Identifier::parse(name)?)?;
        output.push(advancement.clone());
        Ok(advancement)
    }
}

impl Default for JavaAdvancementBuilderModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advancement_display::DisplayInfoFields;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn display(frame: AdvancementFrame) -> DisplayInfoModel {
        DisplayInfoModel::new(DisplayInfoFields {
            icon: id("minecraft:stone"),
            title: "Stone Age".to_string(),
            description: "Mine stone".to_string(),
            background: None,
            frame,
            show_toast: true,
            announce_chat: true,
            hidden: false,
        })
    }

    #[test]
    fn advancement_builder_defaults_match_java_advancement_and_recipe_builders() {
        let advancement = JavaAdvancementBuilderModel::advancement()
            .display(display(AdvancementFrame::Task))
            .add_criterion("tick", JavaCriterionValidationModel::valid())
            .build(id("minecraft:story/root"))
            .unwrap();
        assert!(advancement.value.sends_telemetry_event);
        assert!(advancement.value.is_root());
        assert_eq!(
            advancement.value.requirements.requirements(),
            &[vec!["tick".to_string()]]
        );
        assert_eq!(
            JavaAdvancementModel::name_for_holder(&advancement),
            "[Stone Age]"
        );
        assert_eq!(advancement.value.name.as_ref().unwrap().color, "Green");
        assert_eq!(
            advancement.value.name.as_ref().unwrap().hover_text,
            "Stone Age\nMine stone"
        );

        let recipe = JavaAdvancementBuilderModel::recipe_advancement()
            .add_criterion("has_recipe", JavaCriterionValidationModel::valid())
            .build(id("minecraft:recipes/stone"))
            .unwrap();
        assert!(!recipe.value.sends_telemetry_event);
        assert_eq!(
            JavaAdvancementModel::name_for_holder(&recipe),
            "minecraft:recipes/stone"
        );
    }

    #[test]
    fn advancement_builder_requirements_rewards_parent_save_and_validation_match_java() {
        let mut saved = Vec::new();
        let holder = JavaAdvancementBuilderModel::recipe_advancement()
            .parent(id("minecraft:story/root"))
            .requirements_strategy(AdvancementRequirementStrategy::Or)
            .rewards_from_builder(AdvancementRewards::java_builder().add_experience(5))
            .add_criterion("stone", JavaCriterionValidationModel::valid())
            .add_criterion("iron", JavaCriterionValidationModel::valid())
            .save(&mut saved, "minecraft:story/mine_stone")
            .unwrap();

        assert_eq!(saved, vec![holder.clone()]);
        assert_eq!(holder.value.parent, Some(id("minecraft:story/root")));
        assert_eq!(holder.value.rewards.experience, 5);
        assert_eq!(
            holder.value.requirements.requirements(),
            &[vec!["iron".to_string(), "stone".to_string()]]
        );

        let bad_empty = JavaAdvancementBuilderModel::advancement()
            .build(id("minecraft:bad"))
            .unwrap_err();
        assert_eq!(bad_empty, "Advancement criteria cannot be empty");

        let bad_requirements = JavaAdvancementBuilderModel::recipe_advancement()
            .add_criterion("stone", JavaCriterionValidationModel::valid())
            .requirements(AdvancementRequirementsModel::new(vec![vec![
                "missing".to_string()
            ]]))
            .build(id("minecraft:bad_requirements"))
            .unwrap_err();
        assert!(bad_requirements.contains("Missing: [stone]. Unknown: [missing]"));
    }

    #[test]
    fn advancement_network_read_write_shape_matches_java_stream_codec() {
        let mut display = display(AdvancementFrame::Challenge);
        display.set_location(2.0, 3.5);
        let advancement = JavaAdvancementBuilderModel::advancement()
            .parent(id("minecraft:story/root"))
            .display(display)
            .add_criterion("tick", JavaCriterionValidationModel::valid())
            .build(id("minecraft:story/child"))
            .unwrap();

        let payload = advancement.value.write_network_payload();
        assert_eq!(payload.parent, Some(id("minecraft:story/root")));
        assert_eq!(
            payload.display.as_ref().unwrap().frame,
            AdvancementFrame::Challenge
        );
        assert_eq!(payload.requirements, vec![vec!["tick".to_string()]]);
        assert!(payload.sends_telemetry_event);

        let decoded = JavaAdvancementModel::read_network_payload(payload);
        assert_eq!(decoded.parent, Some(id("minecraft:story/root")));
        assert_eq!(decoded.rewards, AdvancementRewards::java_empty());
        assert!(decoded.criteria.is_empty());
        assert!(decoded
            .display
            .as_ref()
            .is_some_and(|display| !display.announce_chat));
        assert_eq!(
            decoded.requirements.requirements(),
            &[vec!["tick".to_string()]]
        );
        assert!(decoded.sends_telemetry_event);
    }

    #[test]
    fn advancement_validate_walks_criterion_instances_by_name() {
        let advancement = JavaAdvancementBuilderModel::recipe_advancement()
            .add_criterion("good", JavaCriterionValidationModel::valid())
            .add_criterion(
                "bad",
                JavaCriterionValidationModel::invalid("unknown loot table"),
            )
            .build(id("minecraft:test"))
            .unwrap();

        assert_eq!(
            advancement.value.validate_criteria(),
            vec!["bad: unknown loot table".to_string()]
        );
    }
}
