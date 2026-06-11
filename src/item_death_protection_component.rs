#![allow(dead_code)]

use crate::item_consume_effects::{ConsumeEffectModel, StatusEffectInstanceModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathProtectionComponent {
    pub death_effects: Vec<ConsumeEffectModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathProtectionApplyContext {
    pub item_id: &'static str,
    pub entity_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedConsumeEffect {
    pub context: DeathProtectionApplyContext,
    pub effect: ConsumeEffectModel,
}

impl DeathProtectionComponent {
    pub fn new(death_effects: Vec<ConsumeEffectModel>) -> Self {
        Self { death_effects }
    }

    pub fn totem_of_undying() -> Self {
        Self::new(vec![
            ConsumeEffectModel::ClearAllStatusEffects,
            ConsumeEffectModel::apply_status_effects(vec![
                StatusEffectInstanceModel::new("minecraft:regeneration", 900, 1),
                StatusEffectInstanceModel::new("minecraft:absorption", 100, 1),
                StatusEffectInstanceModel::new("minecraft:fire_resistance", 800, 0),
            ]),
        ])
    }

    pub fn apply_effects(&self, context: DeathProtectionApplyContext) -> Vec<AppliedConsumeEffect> {
        self.death_effects
            .iter()
            .cloned()
            .map(|effect| AppliedConsumeEffect {
                context: context.clone(),
                effect,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_properties::ItemComponent;

    const DEATH_PROTECTION_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/DeathProtection.java"
    );

    #[test]
    fn death_protection_totem_default_matches_java_effect_list() {
        for sentinel in [
            "public record DeathProtection(List<ConsumeEffect> deathEffects)",
            "ConsumeEffect.CODEC.listOf().optionalFieldOf(\"death_effects\", List.of())",
            "ConsumeEffect.STREAM_CODEC.apply(ByteBufCodecs.list())",
            "public static final DeathProtection TOTEM_OF_UNDYING = new DeathProtection",
            "new ClearAllStatusEffectsConsumeEffect()",
            "new MobEffectInstance(MobEffects.REGENERATION, 900, 1)",
            "new MobEffectInstance(MobEffects.ABSORPTION, 100, 1)",
            "new MobEffectInstance(MobEffects.FIRE_RESISTANCE, 800, 0)",
        ] {
            assert!(
                DEATH_PROTECTION_JAVA.contains(sentinel),
                "missing DeathProtection sentinel {sentinel}"
            );
        }

        assert_eq!(
            DeathProtectionComponent::totem_of_undying(),
            DeathProtectionComponent::new(vec![
                ConsumeEffectModel::ClearAllStatusEffects,
                ConsumeEffectModel::apply_status_effects(vec![
                    StatusEffectInstanceModel::new("minecraft:regeneration", 900, 1),
                    StatusEffectInstanceModel::new("minecraft:absorption", 100, 1),
                    StatusEffectInstanceModel::new("minecraft:fire_resistance", 800, 0),
                ]),
            ])
        );
    }

    #[test]
    fn death_protection_apply_effects_preserves_java_order_and_context() {
        let component = DeathProtectionComponent::totem_of_undying();
        let context = DeathProtectionApplyContext {
            item_id: "minecraft:totem_of_undying",
            entity_id: 42,
        };
        let applied = component.apply_effects(context.clone());

        assert_eq!(applied.len(), 2);
        assert_eq!(applied[0].context, context);
        assert_eq!(applied[0].effect, ConsumeEffectModel::ClearAllStatusEffects);
        assert!(matches!(
            applied[1].effect,
            ConsumeEffectModel::ApplyStatusEffects { .. }
        ));
    }

    #[test]
    fn death_protection_item_component_key_matches_data_component_registry() {
        assert_eq!(
            ItemComponent::DeathProtection(DeathProtectionComponent::totem_of_undying()).key(),
            "minecraft:death_protection"
        );
    }
}
