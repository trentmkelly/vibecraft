#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseEffectsComponent {
    pub can_sprint: bool,
    pub interact_vibrations: bool,
    pub speed_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableListenerContext {
    pub level_id: &'static str,
    pub user_id: i32,
    pub stack_item: &'static str,
    pub consumable_id: &'static str,
}

pub trait ConsumableListenerModel {
    fn on_consume(&self, context: &ConsumableListenerContext);
}

impl UseEffectsComponent {
    pub const DEFAULT_CAN_SPRINT: bool = false;
    pub const DEFAULT_INTERACT_VIBRATIONS: bool = true;
    pub const DEFAULT_SPEED_MULTIPLIER: f32 = 0.2;

    pub fn new(
        can_sprint: bool,
        interact_vibrations: bool,
        speed_multiplier: f32,
    ) -> Option<Self> {
        (0.0..=1.0).contains(&speed_multiplier).then_some(Self {
            can_sprint,
            interact_vibrations,
            speed_multiplier,
        })
    }

    pub fn default_component() -> Self {
        Self {
            can_sprint: Self::DEFAULT_CAN_SPRINT,
            interact_vibrations: Self::DEFAULT_INTERACT_VIBRATIONS,
            speed_multiplier: Self::DEFAULT_SPEED_MULTIPLIER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const USE_EFFECTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/UseEffects.java"
    );
    const CONSUMABLE_LISTENER_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/ConsumableListener.java"
    );

    struct RecordingListener {
        seen: RefCell<Vec<ConsumableListenerContext>>,
    }

    impl RecordingListener {
        fn new() -> Self {
            Self {
                seen: RefCell::new(Vec::new()),
            }
        }
    }

    impl ConsumableListenerModel for RecordingListener {
        fn on_consume(&self, context: &ConsumableListenerContext) {
            self.seen.borrow_mut().push(context.clone());
        }
    }

    #[test]
    fn use_effects_defaults_range_and_fields_match_java() {
        for sentinel in [
            "public record UseEffects(boolean canSprint, boolean interactVibrations, float speedMultiplier)",
            "public static final UseEffects DEFAULT = new UseEffects(false, true, 0.2F);",
            "Codec.BOOL.optionalFieldOf(\"can_sprint\", DEFAULT.canSprint)",
            "Codec.BOOL.optionalFieldOf(\"interact_vibrations\", DEFAULT.interactVibrations)",
            "Codec.floatRange(0.0F, 1.0F).optionalFieldOf(\"speed_multiplier\", DEFAULT.speedMultiplier)",
            "ByteBufCodecs.BOOL",
            "ByteBufCodecs.FLOAT",
        ] {
            assert!(
                USE_EFFECTS_JAVA.contains(sentinel),
                "missing UseEffects sentinel {sentinel}"
            );
        }

        assert_eq!(
            UseEffectsComponent::default_component(),
            UseEffectsComponent {
                can_sprint: false,
                interact_vibrations: true,
                speed_multiplier: 0.2,
            }
        );
        assert_eq!(
            UseEffectsComponent::new(true, false, 1.0),
            Some(UseEffectsComponent {
                can_sprint: true,
                interact_vibrations: false,
                speed_multiplier: 1.0,
            })
        );
        assert_eq!(UseEffectsComponent::new(false, true, -0.01), None);
        assert_eq!(UseEffectsComponent::new(false, true, 1.01), None);
    }

    #[test]
    fn consumable_listener_signature_is_modeled_like_java_interface() {
        for sentinel in [
            "public interface ConsumableListener",
            "void onConsume(final Level level, final LivingEntity user, final ItemStack stack, final Consumable consumable);",
        ] {
            assert!(
                CONSUMABLE_LISTENER_JAVA.contains(sentinel),
                "missing ConsumableListener sentinel {sentinel}"
            );
        }

        let listener = RecordingListener::new();
        let context = ConsumableListenerContext {
            level_id: "minecraft:overworld",
            user_id: 42,
            stack_item: "minecraft:ominous_bottle",
            consumable_id: "minecraft:drink",
        };
        listener.on_consume(&context);
        assert_eq!(listener.seen.borrow().as_slice(), &[context]);
    }
}
