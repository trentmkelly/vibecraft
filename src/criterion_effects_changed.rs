use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectsChangedTriggerInstance {
    pub player_predicate_present: bool,
    pub effects: Option<MobEffectsPredicateModel>,
    pub source: Option<ContextAwarePredicateModel>,
}

impl EffectsChangedTriggerInstance {
    pub fn new(
        player_predicate_present: bool,
        effects: Option<MobEffectsPredicateModel>,
        source: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present,
            effects,
            source,
        }
    }

    pub fn matches(&self, player: &PlayerEffectsModel, source: Option<&LootContextModel>) -> bool {
        if self
            .effects
            .as_ref()
            .is_some_and(|effects| !effects.matches(player))
        {
            return false;
        }

        self.source
            .as_ref()
            .is_none_or(|predicate| source.is_some_and(|source| predicate.matches(source)))
    }

    pub fn validate(&self) -> Vec<String> {
        if self.source.is_some() {
            vec!["source".to_string()]
        } else {
            Vec::new()
        }
    }

    pub fn has_effects(effects: MobEffectsPredicateBuilder) -> EffectsChangedCriterion {
        EffectsChangedCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(false, effects.build(), None),
        }
    }

    pub fn got_effects_from(source: ContextAwarePredicateModel) -> EffectsChangedCriterion {
        EffectsChangedCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(false, None, Some(source)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectsChangedCriterion {
    pub trigger_id: Identifier,
    pub instance: EffectsChangedTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:effects_changed").unwrap()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MobEffectsPredicateModel {
    effect_map: BTreeMap<Identifier, MobEffectInstancePredicateModel>,
}

impl MobEffectsPredicateModel {
    pub fn new(effect_map: BTreeMap<Identifier, MobEffectInstancePredicateModel>) -> Self {
        Self { effect_map }
    }

    pub fn matches(&self, player: &PlayerEffectsModel) -> bool {
        for (effect, predicate) in &self.effect_map {
            if !predicate.matches(player.effects.get(effect)) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MobEffectsPredicateBuilder {
    effect_map: BTreeMap<Identifier, MobEffectInstancePredicateModel>,
}

impl MobEffectsPredicateBuilder {
    pub fn effects() -> Self {
        Self::default()
    }

    pub fn and(mut self, effect: Identifier) -> Self {
        self.effect_map
            .insert(effect, MobEffectInstancePredicateModel::default());
        self
    }

    pub fn and_with(
        mut self,
        effect: Identifier,
        predicate: MobEffectInstancePredicateModel,
    ) -> Self {
        self.effect_map.insert(effect, predicate);
        self
    }

    pub fn build(self) -> Option<MobEffectsPredicateModel> {
        Some(MobEffectsPredicateModel::new(self.effect_map))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MobEffectInstancePredicateModel {
    amplifier: IntBoundsModel,
    duration: IntBoundsModel,
    ambient: Option<bool>,
    visible: Option<bool>,
}

impl MobEffectInstancePredicateModel {
    pub fn new(
        amplifier: IntBoundsModel,
        duration: IntBoundsModel,
        ambient: Option<bool>,
        visible: Option<bool>,
    ) -> Self {
        Self {
            amplifier,
            duration,
            ambient,
            visible,
        }
    }

    pub fn matches(&self, instance: Option<&MobEffectInstanceModel>) -> bool {
        let Some(instance) = instance else {
            return false;
        };

        if !self.amplifier.matches(instance.amplifier) {
            return false;
        }

        if !self.duration.matches(instance.duration) {
            return false;
        }

        if self
            .ambient
            .is_some_and(|ambient| ambient != instance.ambient)
        {
            return false;
        }

        self.visible
            .is_none_or(|visible| visible == instance.visible)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobEffectInstanceModel {
    amplifier: i32,
    duration: i32,
    ambient: bool,
    visible: bool,
}

impl MobEffectInstanceModel {
    pub fn new(amplifier: i32, duration: i32, ambient: bool, visible: bool) -> Self {
        Self {
            amplifier,
            duration,
            ambient,
            visible,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerEffectsModel {
    effects: BTreeMap<Identifier, MobEffectInstanceModel>,
}

impl PlayerEffectsModel {
    pub fn with_effect(mut self, effect: Identifier, instance: MobEffectInstanceModel) -> Self {
        self.effects.insert(effect, instance);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_entity_type: Option<Identifier>,
}

impl ContextAwarePredicateModel {
    pub fn any() -> Self {
        Self {
            required_entity_type: None,
        }
    }

    pub fn entity_type(required_entity_type: Identifier) -> Self {
        Self {
            required_entity_type: Some(required_entity_type),
        }
    }

    pub fn matches(&self, context: &LootContextModel) -> bool {
        self.required_entity_type
            .as_ref()
            .is_none_or(|required| required == &context.entity_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    entity_type: Identifier,
}

impl LootContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn player_with_speed() -> PlayerEffectsModel {
        PlayerEffectsModel::default().with_effect(
            id("minecraft:speed"),
            MobEffectInstanceModel::new(1, 200, false, true),
        )
    }

    #[test]
    fn omitted_effects_and_source_match_any_change() {
        let instance = EffectsChangedTriggerInstance::new(false, None, None);

        assert!(instance.matches(&PlayerEffectsModel::default(), None));
        assert!(instance.matches(
            &player_with_speed(),
            Some(&LootContextModel::new(id("minecraft:zombie")))
        ));
    }

    #[test]
    fn effects_predicate_is_checked_before_source_like_java() {
        let instance = EffectsChangedTriggerInstance::new(
            false,
            Some(MobEffectsPredicateModel::new(BTreeMap::from([(
                id("minecraft:speed"),
                MobEffectInstancePredicateModel::new(
                    IntBoundsModel::exactly(2),
                    IntBoundsModel::any(),
                    None,
                    None,
                ),
            )]))),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:zombie",
            ))),
        );

        assert!(!instance.matches(
            &player_with_speed(),
            Some(&LootContextModel::new(id("minecraft:zombie")))
        ));
    }

    #[test]
    fn mob_effect_instance_predicate_checks_presence_and_all_optional_fields() {
        let predicate = MobEffectInstancePredicateModel::new(
            IntBoundsModel::exactly(1),
            IntBoundsModel::between(100, 300),
            Some(false),
            Some(true),
        );

        assert!(predicate.matches(Some(&MobEffectInstanceModel::new(1, 200, false, true))));
        assert!(!predicate.matches(None));
        assert!(!predicate.matches(Some(&MobEffectInstanceModel::new(2, 200, false, true))));
        assert!(!predicate.matches(Some(&MobEffectInstanceModel::new(1, 99, false, true))));
        assert!(!predicate.matches(Some(&MobEffectInstanceModel::new(1, 200, true, true))));
        assert!(!predicate.matches(Some(&MobEffectInstanceModel::new(1, 200, false, false))));
    }

    #[test]
    fn source_predicate_requires_non_null_matching_context() {
        let instance = EffectsChangedTriggerInstance::new(
            false,
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:zombie",
            ))),
        );

        assert!(!instance.matches(&PlayerEffectsModel::default(), None));
        assert!(instance.matches(
            &PlayerEffectsModel::default(),
            Some(&LootContextModel::new(id("minecraft:zombie")))
        ));
        assert!(!instance.matches(
            &PlayerEffectsModel::default(),
            Some(&LootContextModel::new(id("minecraft:skeleton")))
        ));
    }

    #[test]
    fn factories_set_java_trigger_id_and_fields() {
        let has_effects = EffectsChangedTriggerInstance::has_effects(
            MobEffectsPredicateBuilder::effects()
                .and(id("minecraft:speed"))
                .and_with(
                    id("minecraft:strength"),
                    MobEffectInstancePredicateModel::new(
                        IntBoundsModel::exactly(0),
                        IntBoundsModel::any(),
                        None,
                        None,
                    ),
                ),
        );
        assert_eq!(has_effects.trigger_id, id("minecraft:effects_changed"));
        assert!(!has_effects.instance.player_predicate_present);
        assert!(has_effects.instance.effects.is_some());
        assert!(has_effects.instance.source.is_none());

        let got_from = EffectsChangedTriggerInstance::got_effects_from(
            ContextAwarePredicateModel::entity_type(id("minecraft:zombie")),
        );
        assert_eq!(got_from.trigger_id, id("minecraft:effects_changed"));
        assert!(got_from.instance.effects.is_none());
        assert!(got_from.instance.source.is_some());
    }

    #[test]
    fn validation_reports_source_predicate_label_only_when_present() {
        let without_source = EffectsChangedTriggerInstance::new(false, None, None);
        let with_source =
            EffectsChangedTriggerInstance::got_effects_from(ContextAwarePredicateModel::any());

        assert!(without_source.validate().is_empty());
        assert_eq!(with_source.instance.validate(), vec!["source".to_string()]);
    }
}
