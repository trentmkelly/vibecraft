#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectInstanceModel {
    pub effect: &'static str,
    pub duration: i32,
    pub amplifier: i32,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OminousBottleAmplifier {
    pub value: i32,
}

impl OminousBottleAmplifier {
    pub const EFFECT_DURATION: i32 = 120_000;
    pub const MIN_AMPLIFIER: i32 = 0;
    pub const MAX_AMPLIFIER: i32 = 4;
    pub const EFFECT_ID: &'static str = "minecraft:bad_omen";

    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn codec_new(value: i32) -> Result<Self, String> {
        if (Self::MIN_AMPLIFIER..=Self::MAX_AMPLIFIER).contains(&value) {
            Ok(Self::new(value))
        } else {
            Err(format!(
                "ominous bottle amplifier {value} outside {}..={}",
                Self::MIN_AMPLIFIER,
                Self::MAX_AMPLIFIER
            ))
        }
    }

    pub fn bad_omen_effect(self) -> EffectInstanceModel {
        EffectInstanceModel {
            effect: Self::EFFECT_ID,
            duration: Self::EFFECT_DURATION,
            amplifier: self.value,
            ambient: false,
            visible: false,
            show_icon: true,
        }
    }

    pub fn on_consume_effect(self) -> EffectInstanceModel {
        self.bad_omen_effect()
    }

    pub fn tooltip_effects(self) -> Vec<EffectInstanceModel> {
        vec![self.bad_omen_effect()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OMINOUS_BOTTLE_AMPLIFIER_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/OminousBottleAmplifier.java"
    );

    #[test]
    fn ominous_bottle_amplifier_effect_codec_range_and_tooltip_match_java() {
        for sentinel in [
            "public record OminousBottleAmplifier(int value) implements ConsumableListener, TooltipProvider",
            "public static final int EFFECT_DURATION = 120000;",
            "public static final int MIN_AMPLIFIER = 0;",
            "public static final int MAX_AMPLIFIER = 4;",
            "ExtraCodecs.intRange(0, 4).xmap(OminousBottleAmplifier::new, OminousBottleAmplifier::value)",
            "ByteBufCodecs.VAR_INT, OminousBottleAmplifier::value, OminousBottleAmplifier::new",
            "user.addEffect(new MobEffectInstance(MobEffects.BAD_OMEN, 120000, this.value, false, false, true));",
            "PotionContents.addPotionTooltip(effects, consumer, 1.0F, context.tickRate());",
        ] {
            assert!(
                OMINOUS_BOTTLE_AMPLIFIER_JAVA.contains(sentinel),
                "missing OminousBottleAmplifier sentinel {sentinel}"
            );
        }

        assert_eq!(OminousBottleAmplifier::EFFECT_DURATION, 120_000);
        assert_eq!(OminousBottleAmplifier::MIN_AMPLIFIER, 0);
        assert_eq!(OminousBottleAmplifier::MAX_AMPLIFIER, 4);
        assert_eq!(OminousBottleAmplifier::codec_new(0).unwrap().value, 0);
        assert_eq!(OminousBottleAmplifier::codec_new(4).unwrap().value, 4);
        assert!(OminousBottleAmplifier::codec_new(-1).is_err());
        assert!(OminousBottleAmplifier::codec_new(5).is_err());

        let amplifier = OminousBottleAmplifier::new(3);
        let expected = EffectInstanceModel {
            effect: "minecraft:bad_omen",
            duration: 120_000,
            amplifier: 3,
            ambient: false,
            visible: false,
            show_icon: true,
        };
        assert_eq!(amplifier.on_consume_effect(), expected);
        assert_eq!(amplifier.tooltip_effects(), vec![expected]);
    }
}
