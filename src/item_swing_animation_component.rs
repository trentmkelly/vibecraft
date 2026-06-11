#![allow(dead_code)]

use crate::item_properties::SwingAnimationType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwingAnimationComponent {
    pub animation_type: SwingAnimationType,
    pub duration: i32,
}

impl SwingAnimationComponent {
    pub const DEFAULT_DURATION: i32 = 6;
    pub const DEFAULT_TYPE: SwingAnimationType = SwingAnimationType::Whack;

    pub fn new(animation_type: SwingAnimationType, duration: i32) -> Option<Self> {
        (duration > 0).then_some(Self {
            animation_type,
            duration,
        })
    }

    pub fn default_component() -> Self {
        Self {
            animation_type: Self::DEFAULT_TYPE,
            duration: Self::DEFAULT_DURATION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SWING_ANIMATION_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/SwingAnimation.java"
    );

    #[test]
    fn swing_animation_component_defaults_and_positive_duration_match_java() {
        for sentinel in [
            "public record SwingAnimation(SwingAnimationType type, int duration)",
            "public static final SwingAnimation DEFAULT = new SwingAnimation(SwingAnimationType.WHACK, 6);",
            "SwingAnimationType.CODEC.optionalFieldOf(\"type\", DEFAULT.type)",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"duration\", DEFAULT.duration)",
            "SwingAnimationType.STREAM_CODEC, SwingAnimation::type, ByteBufCodecs.VAR_INT, SwingAnimation::duration",
        ] {
            assert!(
                SWING_ANIMATION_JAVA.contains(sentinel),
                "missing SwingAnimation sentinel {sentinel}"
            );
        }

        assert_eq!(
            SwingAnimationComponent::default_component(),
            SwingAnimationComponent {
                animation_type: SwingAnimationType::Whack,
                duration: 6,
            }
        );
        assert_eq!(
            SwingAnimationComponent::new(SwingAnimationType::Stab, 12),
            Some(SwingAnimationComponent {
                animation_type: SwingAnimationType::Stab,
                duration: 12,
            })
        );
        assert_eq!(SwingAnimationComponent::new(SwingAnimationType::Whack, 0), None);
        assert_eq!(SwingAnimationComponent::new(SwingAnimationType::Whack, -1), None);
    }
}
