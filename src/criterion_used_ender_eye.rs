use crate::criterion_distance_predicate::DoubleBoundsModel;

#[derive(Debug, Clone, PartialEq)]
pub struct UsedEnderEyeTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub distance: DoubleBoundsModel,
}

impl UsedEnderEyeTriggerInstanceModel {
    pub fn new(player: Option<ContextAwarePredicateModel>, distance: DoubleBoundsModel) -> Self {
        Self { player, distance }
    }

    pub fn codec_fields() -> [CodecFieldModel; 2] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC", None),
            CodecFieldModel::optional(
                "distance",
                "MinMaxBounds.Doubles.CODEC",
                Some("MinMaxBounds.Doubles.ANY"),
            ),
        ]
    }

    pub fn matches(&self, sqr_distance: f64) -> bool {
        self.distance.matches_sqr(sqr_distance)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsedEnderEyeTriggerModel {
    listeners: Vec<UsedEnderEyeTriggerInstanceModel>,
}

impl UsedEnderEyeTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = UsedEnderEyeTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "UsedEnderEyeTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        feature: BlockPosModel,
    ) -> Vec<&UsedEnderEyeTriggerInstanceModel> {
        let xd = player.x - f64::from(feature.x);
        let zd = player.z - f64::from(feature.z);
        let sqr_distance = (xd * xd) + (zd * zd);

        self.listeners
            .iter()
            .filter(|listener| listener.matches(sqr_distance))
            .collect()
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerPlayerModel {
    x: f64,
    y: f64,
    z: f64,
}

impl ServerPlayerModel {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosModel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn used_ender_eye_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            UsedEnderEyeTriggerModel::codec_type(),
            "UsedEnderEyeTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn used_ender_eye_codec_fields_match_java_record_codec() {
        assert_eq!(
            UsedEnderEyeTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC", None),
                CodecFieldModel::optional(
                    "distance",
                    "MinMaxBounds.Doubles.CODEC",
                    Some("MinMaxBounds.Doubles.ANY"),
                ),
            ]
        );
    }

    #[test]
    fn any_distance_default_matches_any_squared_distance() {
        let instance = UsedEnderEyeTriggerInstanceModel::new(None, DoubleBoundsModel::any());

        assert!(instance.matches(0.0));
        assert!(instance.matches(1_000_000.0));
    }

    #[test]
    fn matches_uses_double_bounds_matches_sqr() {
        let instance =
            UsedEnderEyeTriggerInstanceModel::new(None, DoubleBoundsModel::between(5.0, 10.0));

        assert!(!instance.matches(24.0));
        assert!(instance.matches(25.0));
        assert!(instance.matches(100.0));
        assert!(!instance.matches(101.0));
    }

    #[test]
    fn trigger_computes_horizontal_squared_distance_from_player_to_feature() {
        let matching = UsedEnderEyeTriggerInstanceModel::new(None, DoubleBoundsModel::exactly(5.0));
        let too_far = UsedEnderEyeTriggerInstanceModel::new(None, DoubleBoundsModel::at_most(4.0));
        let trigger = UsedEnderEyeTriggerModel::new([matching.clone(), too_far]);
        let player = ServerPlayerModel::new(13.0, 99.0, 24.0);
        let feature = BlockPosModel::new(10, -64, 20);

        assert_eq!(trigger.trigger(&player, feature), vec![&matching]);
    }

    #[test]
    fn trigger_ignores_player_y_and_feature_y() {
        let instance = UsedEnderEyeTriggerInstanceModel::new(None, DoubleBoundsModel::exactly(5.0));
        let trigger = UsedEnderEyeTriggerModel::new([instance.clone()]);

        assert_eq!(
            trigger.trigger(
                &ServerPlayerModel::new(13.0, -1000.0, 24.0),
                BlockPosModel::new(10, 2048, 20),
            ),
            vec![&instance]
        );
    }
}
