use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpearMobsTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub count: Option<i32>,
}

impl SpearMobsTriggerInstanceModel {
    pub fn new(player: Option<ContextAwarePredicateModel>, count: Option<i32>) -> Self {
        Self { player, count }
    }

    pub fn codec_fields() -> [CodecFieldModel; 2] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
            CodecFieldModel::optional("count", "ExtraCodecs.POSITIVE_INT"),
        ]
    }

    pub fn decode(
        player: Option<ContextAwarePredicateModel>,
        count: Option<i32>,
    ) -> Result<Self, String> {
        if let Some(count) = count {
            if count <= 0 {
                return Err(format!("count must be positive: {count}"));
            }
        }

        Ok(Self::new(player, count))
    }

    pub fn spear_mobs(required_count: i32) -> SpearMobsCriterionModel {
        SpearMobsCriterionModel {
            trigger_id: id("minecraft:spear_mobs"),
            instance: Self::new(None, Some(required_count)),
        }
    }

    pub fn matches(&self, required_count: i32) -> bool {
        self.count.is_none_or(|count| required_count >= count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpearMobsTriggerModel {
    listeners: Vec<SpearMobsTriggerInstanceModel>,
}

impl SpearMobsTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = SpearMobsTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "SpearMobsTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        _player: &ServerPlayerModel,
        number: i32,
    ) -> Vec<SpearMobsTriggerInstanceModel> {
        self.listeners
            .iter()
            .filter(|instance| instance.matches(number))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpearMobsCriterionModel {
    pub trigger_id: Identifier,
    pub instance: SpearMobsTriggerInstanceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecFieldModel {
    pub name: &'static str,
    pub codec: &'static str,
    pub optional: bool,
}

impl CodecFieldModel {
    pub fn optional(name: &'static str, codec: &'static str) -> Self {
        Self {
            name,
            codec,
            optional: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    pub name: String,
}

impl ServerPlayerModel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spear_mobs_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            SpearMobsTriggerModel::codec_type(),
            "SpearMobsTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn spear_mobs_codec_fields_match_java_record_codec() {
        assert_eq!(
            SpearMobsTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("count", "ExtraCodecs.POSITIVE_INT"),
            ]
        );
    }

    #[test]
    fn spear_mobs_count_codec_accepts_only_positive_values_when_decoding() {
        assert_eq!(
            SpearMobsTriggerInstanceModel::decode(None, None).unwrap(),
            SpearMobsTriggerInstanceModel::new(None, None)
        );
        assert_eq!(
            SpearMobsTriggerInstanceModel::decode(None, Some(1)).unwrap(),
            SpearMobsTriggerInstanceModel::new(None, Some(1))
        );
        assert!(SpearMobsTriggerInstanceModel::decode(None, Some(0)).is_err());
        assert!(SpearMobsTriggerInstanceModel::decode(None, Some(-1)).is_err());
    }

    #[test]
    fn spear_mobs_factory_uses_empty_player_and_required_count() {
        let criterion = SpearMobsTriggerInstanceModel::spear_mobs(3);

        assert_eq!(criterion.trigger_id, id("minecraft:spear_mobs"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(criterion.instance.count, Some(3));
    }

    #[test]
    fn spear_mobs_matching_uses_omitted_count_as_match_any() {
        let instance = SpearMobsTriggerInstanceModel::new(None, None);

        assert!(instance.matches(0));
        assert!(instance.matches(5));
    }

    #[test]
    fn spear_mobs_matching_requires_actual_count_at_least_threshold() {
        let instance = SpearMobsTriggerInstanceModel::new(None, Some(3));

        assert!(!instance.matches(2));
        assert!(instance.matches(3));
        assert!(instance.matches(4));
    }

    #[test]
    fn spear_mobs_trigger_forwards_number_to_instance_matches() {
        let trigger = SpearMobsTriggerModel::new([
            SpearMobsTriggerInstanceModel::new(None, None),
            SpearMobsTriggerInstanceModel::new(None, Some(2)),
            SpearMobsTriggerInstanceModel::new(None, Some(4)),
        ]);
        let player = ServerPlayerModel::new("Alex");

        let matched = trigger.trigger(&player, 3);

        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].count, None);
        assert_eq!(matched[1].count, Some(2));
    }
}
