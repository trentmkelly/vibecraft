use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBlockTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub signal_strength: IntBoundsModel,
    pub projectile: Option<ContextAwarePredicateModel>,
}

impl TargetBlockTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        signal_strength: IntBoundsModel,
        projectile: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self {
            player,
            signal_strength,
            projectile,
        }
    }

    pub fn codec_fields() -> [CodecFieldModel; 3] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC", None),
            CodecFieldModel::optional(
                "signal_strength",
                "MinMaxBounds.Ints.CODEC",
                Some("MinMaxBounds.Ints.ANY"),
            ),
            CodecFieldModel::optional("projectile", "EntityPredicate.ADVANCEMENT_CODEC", None),
        ]
    }

    pub fn target_hit(
        redstone_signal_strength: IntBoundsModel,
        projectile: Option<ContextAwarePredicateModel>,
    ) -> TargetBlockCriterionModel {
        TargetBlockCriterionModel {
            trigger_id: id("minecraft:target_hit"),
            instance: Self::new(None, redstone_signal_strength, projectile),
        }
    }

    pub fn matches(
        &self,
        projectile: &LootContextModel,
        _hit_position: Vec3Model,
        signal_strength: i32,
    ) -> bool {
        if !self.signal_strength.matches(signal_strength) {
            return false;
        }

        self.projectile
            .as_ref()
            .is_none_or(|predicate| predicate.matches(projectile))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if let Some(problem) = self
            .player
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("player: {problem}"));
        }
        if let Some(problem) = self
            .projectile
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("projectile: {problem}"));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBlockTriggerModel {
    listeners: Vec<TargetBlockTriggerInstanceModel>,
}

impl TargetBlockTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = TargetBlockTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "TargetBlockTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        projectile: EntityContextModel,
        hit_position: Vec3Model,
        signal_strength: i32,
    ) -> Vec<&TargetBlockTriggerInstanceModel> {
        let projectile_context = EntityPredicateModel::create_context(player, projectile);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(&projectile_context, hit_position, signal_strength))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBlockCriterionModel {
    pub trigger_id: Identifier,
    pub instance: TargetBlockTriggerInstanceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecFieldModel {
    pub name: &'static str,
    pub codec: &'static str,
    pub default: Option<&'static str>,
    pub optional: bool,
}

impl CodecFieldModel {
    pub fn optional(
        name: &'static str,
        codec: &'static str,
        default: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            codec,
            default,
            optional: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn any() -> Self {
        Self::ANY
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

    fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| value <= max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Model {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3Model {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    entity_type: Identifier,
}

impl ServerPlayerModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_type: Identifier,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    player_type: Identifier,
    projectile_type: Identifier,
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(
        player: &ServerPlayerModel,
        projectile: EntityContextModel,
    ) -> LootContextModel {
        LootContextModel {
            player_type: player.entity_type.clone(),
            projectile_type: projectile.entity_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_entity_type: Option<Identifier>,
    validation_problem: Option<String>,
    panic_if_evaluated: bool,
}

impl ContextAwarePredicateModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_entity_type: Some(entity_type),
            validation_problem: None,
            panic_if_evaluated: false,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            required_entity_type: None,
            validation_problem: Some(problem.to_string()),
            panic_if_evaluated: false,
        }
    }

    pub fn panic_if_evaluated() -> Self {
        Self {
            required_entity_type: None,
            validation_problem: None,
            panic_if_evaluated: true,
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        assert!(
            !self.panic_if_evaluated,
            "projectile predicate should not be evaluated"
        );
        self.required_entity_type
            .as_ref()
            .is_none_or(|required| &context.projectile_type == required)
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> ServerPlayerModel {
        ServerPlayerModel::new(id("minecraft:player"))
    }

    fn projectile(entity_type: &str) -> EntityContextModel {
        EntityContextModel::new(id(entity_type))
    }

    fn hit(x: f64, y: f64, z: f64) -> Vec3Model {
        Vec3Model::new(x, y, z)
    }

    #[test]
    fn target_block_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            TargetBlockTriggerModel::codec_type(),
            "TargetBlockTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn target_block_codec_fields_match_java_record_codec() {
        assert_eq!(
            TargetBlockTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC", None),
                CodecFieldModel::optional(
                    "signal_strength",
                    "MinMaxBounds.Ints.CODEC",
                    Some("MinMaxBounds.Ints.ANY"),
                ),
                CodecFieldModel::optional("projectile", "EntityPredicate.ADVANCEMENT_CODEC", None,),
            ]
        );
    }

    #[test]
    fn target_hit_factory_uses_empty_player_and_target_hit_trigger_id() {
        let criterion = TargetBlockTriggerInstanceModel::target_hit(
            IntBoundsModel::between(5, 10),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:arrow",
            ))),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:target_hit"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(
            criterion.instance.signal_strength,
            IntBoundsModel::between(5, 10)
        );
        assert_eq!(
            criterion.instance.projectile,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:arrow"
            )))
        );
    }

    #[test]
    fn omitted_signal_strength_defaults_to_any_and_omitted_projectile_matches_any() {
        let instance = TargetBlockTriggerInstanceModel::new(None, IntBoundsModel::any(), None);
        let context =
            EntityPredicateModel::create_context(&player(), projectile("minecraft:snowball"));

        assert!(instance.matches(&context, hit(0.0, 0.0, 0.0), 0));
        assert!(instance.matches(&context, hit(0.0, 0.0, 0.0), 15));
    }

    #[test]
    fn signal_strength_must_match_before_projectile_predicate() {
        let instance = TargetBlockTriggerInstanceModel::new(
            None,
            IntBoundsModel::exactly(7),
            Some(ContextAwarePredicateModel::panic_if_evaluated()),
        );
        let context =
            EntityPredicateModel::create_context(&player(), projectile("minecraft:arrow"));

        assert!(!instance.matches(&context, hit(1.0, 2.0, 3.0), 6));
    }

    #[test]
    fn projectile_predicate_matches_created_projectile_loot_context() {
        let instance = TargetBlockTriggerInstanceModel::new(
            None,
            IntBoundsModel::between(3, 6),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:arrow",
            ))),
        );
        let trigger = TargetBlockTriggerModel::new([instance.clone()]);

        assert_eq!(
            trigger.trigger(
                &player(),
                projectile("minecraft:arrow"),
                hit(2.0, 3.0, 4.0),
                5
            ),
            vec![&instance]
        );
        assert!(trigger
            .trigger(
                &player(),
                projectile("minecraft:snowball"),
                hit(2.0, 3.0, 4.0),
                5
            )
            .is_empty());
        assert!(trigger
            .trigger(
                &player(),
                projectile("minecraft:arrow"),
                hit(2.0, 3.0, 4.0),
                7
            )
            .is_empty());
    }

    #[test]
    fn hit_position_is_not_used_by_matches() {
        let instance = TargetBlockTriggerInstanceModel::new(
            None,
            IntBoundsModel::exactly(4),
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:arrow",
            ))),
        );
        let context =
            EntityPredicateModel::create_context(&player(), projectile("minecraft:arrow"));

        assert_eq!(
            instance.matches(&context, hit(0.0, 0.0, 0.0), 4),
            instance.matches(&context, hit(99.0, -4.0, 12.5), 4)
        );
    }

    #[test]
    fn trigger_context_is_created_from_player_and_projectile() {
        let context =
            EntityPredicateModel::create_context(&player(), projectile("minecraft:arrow"));

        assert_eq!(context.player_type, id("minecraft:player"));
        assert_eq!(context.projectile_type, id("minecraft:arrow"));
    }

    #[test]
    fn validation_runs_inherited_player_label_and_projectile_label() {
        let instance = TargetBlockTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("bad player")),
            IntBoundsModel::any(),
            Some(ContextAwarePredicateModel::invalid("bad projectile")),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "player: bad player".to_string(),
                "projectile: bad projectile".to_string(),
            ]
        );
    }
}
