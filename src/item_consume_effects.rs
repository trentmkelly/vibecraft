#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumeEffectModel {
    ApplyStatusEffects {
        effects: Vec<StatusEffectInstanceModel>,
        probability_millis: u16,
    },
    RemoveStatusEffects {
        effects: Vec<&'static str>,
    },
    ClearAllStatusEffects,
    TeleportRandomly {
        diameter: PositiveFloat,
    },
    PlaySound {
        sound: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEffectInstanceModel {
    pub effect: &'static str,
    pub duration_ticks: i32,
    pub amplifier: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveFloat(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct ConsumeEffectContext {
    pub level_min_y: f64,
    pub logical_height: f64,
    pub entity_position: (f64, f64, f64),
    pub entity_id: i32,
    pub item_id: &'static str,
    pub is_fox: bool,
    pub is_passenger: bool,
    pub active_effects: Vec<&'static str>,
    pub random_float: f32,
    pub random_doubles: Vec<f64>,
    pub teleport_successes: Vec<bool>,
    pub add_effect_results: Vec<bool>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConsumeEffectOutcome {
    pub changed: bool,
    pub applied_effects: Vec<StatusEffectInstanceModel>,
    pub removed_effects: Vec<&'static str>,
    pub cleared_all_effects: bool,
    pub played_sounds: Vec<PlayedSoundModel>,
    pub teleport_attempts: Vec<TeleportAttemptModel>,
    pub stopped_riding: bool,
    pub reset_fall_distance: bool,
    pub reset_current_impulse_context: bool,
    pub game_events: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayedSoundModel {
    pub sound: &'static str,
    pub source: &'static str,
    pub volume: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeleportAttemptModel {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ConsumeEffectModel {
    pub const APPLY_EFFECTS_TYPE: &'static str = "apply_effects";
    pub const REMOVE_EFFECTS_TYPE: &'static str = "remove_effects";
    pub const CLEAR_ALL_EFFECTS_TYPE: &'static str = "clear_all_effects";
    pub const TELEPORT_RANDOMLY_TYPE: &'static str = "teleport_randomly";
    pub const PLAY_SOUND_TYPE: &'static str = "play_sound";

    pub fn apply_status_effects(effects: Vec<StatusEffectInstanceModel>) -> Self {
        Self::ApplyStatusEffects {
            effects,
            probability_millis: 1000,
        }
    }

    pub fn apply_status_effects_with_probability(
        effects: Vec<StatusEffectInstanceModel>,
        probability: f32,
    ) -> Option<Self> {
        (0.0..=1.0).contains(&probability).then_some(Self::ApplyStatusEffects {
            effects,
            probability_millis: (probability * 1000.0).round() as u16,
        })
    }

    pub fn teleport_randomly_default() -> Self {
        Self::TeleportRandomly {
            diameter: PositiveFloat::known_positive(16.0),
        }
    }

    pub fn type_id(&self) -> &'static str {
        match self {
            Self::ApplyStatusEffects { .. } => Self::APPLY_EFFECTS_TYPE,
            Self::RemoveStatusEffects { .. } => Self::REMOVE_EFFECTS_TYPE,
            Self::ClearAllStatusEffects => Self::CLEAR_ALL_EFFECTS_TYPE,
            Self::TeleportRandomly { .. } => Self::TELEPORT_RANDOMLY_TYPE,
            Self::PlaySound { .. } => Self::PLAY_SOUND_TYPE,
        }
    }

    pub fn apply(&self, context: &ConsumeEffectContext) -> ConsumeEffectOutcome {
        match self {
            Self::ApplyStatusEffects {
                effects,
                probability_millis,
            } => apply_status_effects(effects, *probability_millis, context),
            Self::RemoveStatusEffects { effects } => remove_status_effects(effects, context),
            Self::ClearAllStatusEffects => clear_all_status_effects(context),
            Self::TeleportRandomly { diameter } => teleport_randomly(diameter.get(), context),
            Self::PlaySound { sound } => play_sound(sound),
        }
    }
}

impl StatusEffectInstanceModel {
    pub fn new(effect: &'static str, duration_ticks: i32, amplifier: i32) -> Self {
        Self {
            effect,
            duration_ticks,
            amplifier,
        }
    }
}

impl PositiveFloat {
    pub fn known_positive(value: f32) -> Self {
        debug_assert!(matches!(
            value.partial_cmp(&0.0),
            Some(std::cmp::Ordering::Greater)
        ));
        Self(value.to_bits())
    }

    pub fn new(value: f32) -> Option<Self> {
        matches!(value.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater))
            .then_some(Self(value.to_bits()))
    }

    pub fn get(self) -> f32 {
        f32::from_bits(self.0)
    }
}

impl ConsumeEffectContext {
    pub fn basic(item_id: &'static str, entity_id: i32) -> Self {
        Self {
            level_min_y: 0.0,
            logical_height: 384.0,
            entity_position: (0.0, 64.0, 0.0),
            entity_id,
            item_id,
            is_fox: false,
            is_passenger: false,
            active_effects: Vec::new(),
            random_float: 0.0,
            random_doubles: Vec::new(),
            teleport_successes: Vec::new(),
            add_effect_results: Vec::new(),
        }
    }
}

fn apply_status_effects(
    effects: &[StatusEffectInstanceModel],
    probability_millis: u16,
    context: &ConsumeEffectContext,
) -> ConsumeEffectOutcome {
    let probability = f32::from(probability_millis) / 1000.0;
    if context.random_float >= probability {
        return ConsumeEffectOutcome::default();
    }

    let mut outcome = ConsumeEffectOutcome::default();
    for (index, effect) in effects.iter().enumerate() {
        if context
            .add_effect_results
            .get(index)
            .copied()
            .unwrap_or(true)
        {
            outcome.changed = true;
        }
        outcome.applied_effects.push(effect.clone());
    }
    outcome
}

fn remove_status_effects(effects: &[&'static str], context: &ConsumeEffectContext) -> ConsumeEffectOutcome {
    let mut outcome = ConsumeEffectOutcome::default();
    for effect in effects {
        if context.active_effects.contains(effect) {
            outcome.changed = true;
            outcome.removed_effects.push(*effect);
        }
    }
    outcome
}

fn clear_all_status_effects(context: &ConsumeEffectContext) -> ConsumeEffectOutcome {
    ConsumeEffectOutcome {
        changed: !context.active_effects.is_empty(),
        cleared_all_effects: !context.active_effects.is_empty(),
        ..ConsumeEffectOutcome::default()
    }
}

fn play_sound(sound: &'static str) -> ConsumeEffectOutcome {
    ConsumeEffectOutcome {
        changed: true,
        played_sounds: vec![PlayedSoundModel {
            sound,
            source: "user",
            volume: 1.0,
            pitch: 1.0,
        }],
        ..ConsumeEffectOutcome::default()
    }
}

fn teleport_randomly(diameter: f32, context: &ConsumeEffectContext) -> ConsumeEffectOutcome {
    let mut outcome = ConsumeEffectOutcome::default();
    let mut random = context.random_doubles.iter().copied();

    for attempt in 0..16 {
        let rx = random.next().unwrap_or(0.5);
        let ry = random.next().unwrap_or(0.5);
        let rz = random.next().unwrap_or(0.5);
        let (base_x, base_y, base_z) = context.entity_position;
        let diameter = f64::from(diameter);
        let x = base_x + (rx - 0.5) * diameter;
        let y = (base_y + (ry - 0.5) * diameter)
            .clamp(context.level_min_y, context.level_min_y + context.logical_height - 1.0);
        let z = base_z + (rz - 0.5) * diameter;
        outcome.teleport_attempts.push(TeleportAttemptModel { x, y, z });
        if context.is_passenger {
            outcome.stopped_riding = true;
        }
        if context.teleport_successes.get(attempt).copied().unwrap_or(false) {
            outcome.changed = true;
            outcome.game_events.push("minecraft:teleport");
            outcome.played_sounds.push(if context.is_fox {
                PlayedSoundModel {
                    sound: "minecraft:entity.fox.teleport",
                    source: "neutral",
                    volume: 1.0,
                    pitch: 1.0,
                }
            } else {
                PlayedSoundModel {
                    sound: "minecraft:item.chorus_fruit.teleport",
                    source: "players",
                    volume: 1.0,
                    pitch: 1.0,
                }
            });
            outcome.reset_fall_distance = true;
            outcome.reset_current_impulse_context = true;
            break;
        }
    }
    outcome
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const CONSUME_EFFECT_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/ConsumeEffect.java");
    const APPLY_EFFECTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/ApplyStatusEffectsConsumeEffect.java");
    const CLEAR_ALL_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/ClearAllStatusEffectsConsumeEffect.java");
    const PLAY_SOUND_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/PlaySoundConsumeEffect.java");
    const REMOVE_EFFECTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/RemoveStatusEffectsConsumeEffect.java");
    const TELEPORT_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/consume_effects/TeleportRandomlyConsumeEffect.java");

    #[test]
    fn consume_effect_registry_type_ids_match_java() {
        for sentinel in [
            "\"apply_effects\", ApplyStatusEffectsConsumeEffect.CODEC",
            "\"remove_effects\", RemoveStatusEffectsConsumeEffect.CODEC",
            "\"clear_all_effects\", ClearAllStatusEffectsConsumeEffect.CODEC",
            "\"teleport_randomly\", TeleportRandomlyConsumeEffect.CODEC",
            "\"play_sound\", PlaySoundConsumeEffect.CODEC",
            "Codec<ConsumeEffect> CODEC = BuiltInRegistries.CONSUME_EFFECT_TYPE.byNameCodec().dispatch",
            "StreamCodec<RegistryFriendlyByteBuf, ConsumeEffect> STREAM_CODEC",
        ] {
            assert!(
                CONSUME_EFFECT_JAVA.contains(sentinel),
                "missing ConsumeEffect sentinel {sentinel}"
            );
        }

        assert_eq!(
            ConsumeEffectModel::apply_status_effects(Vec::new()).type_id(),
            "apply_effects"
        );
        assert_eq!(
            ConsumeEffectModel::RemoveStatusEffects { effects: vec![] }.type_id(),
            "remove_effects"
        );
        assert_eq!(
            ConsumeEffectModel::ClearAllStatusEffects.type_id(),
            "clear_all_effects"
        );
        assert_eq!(
            ConsumeEffectModel::teleport_randomly_default().type_id(),
            "teleport_randomly"
        );
        assert_eq!(
            ConsumeEffectModel::PlaySound {
                sound: "minecraft:item.honey_bottle.drink"
            }
            .type_id(),
            "play_sound"
        );
    }

    #[test]
    fn apply_status_effects_probability_and_any_applied_match_java() {
        for sentinel in [
            "public record ApplyStatusEffectsConsumeEffect(List<MobEffectInstance> effects, float probability) implements ConsumeEffect",
            "Codec.floatRange(0.0F, 1.0F).optionalFieldOf(\"probability\", 1.0F)",
            "if (user.getRandom().nextFloat() >= this.probability)",
            "if (user.addEffect(new MobEffectInstance(effect)))",
            "return anyApplied;",
        ] {
            assert!(
                APPLY_EFFECTS_JAVA.contains(sentinel),
                "missing ApplyStatusEffects sentinel {sentinel}"
            );
        }

        let effect = StatusEffectInstanceModel::new("minecraft:speed", 100, 2);
        let apply = ConsumeEffectModel::apply_status_effects_with_probability(vec![effect.clone()], 0.5)
            .unwrap_or_else(|| panic!("0.5 probability should be valid"));
        assert_eq!(
            apply.apply(&ConsumeEffectContext {
                random_float: 0.5,
                ..ConsumeEffectContext::basic("minecraft:test", 1)
            }),
            ConsumeEffectOutcome::default()
        );
        let applied = apply.apply(&ConsumeEffectContext {
            random_float: 0.49,
            add_effect_results: vec![true],
            ..ConsumeEffectContext::basic("minecraft:test", 1)
        });
        assert!(applied.changed);
        assert_eq!(applied.applied_effects, vec![effect]);
        assert!(ConsumeEffectModel::apply_status_effects_with_probability(Vec::new(), 1.01).is_none());
    }

    #[test]
    fn clear_remove_and_sound_effects_match_java_apply_results() {
        for (source, sentinels) in [
            (
                CLEAR_ALL_JAVA,
                vec![
                    "MapCodec.unit(INSTANCE)",
                    "StreamCodec.unit(INSTANCE)",
                    "return user.removeAllEffects();",
                ],
            ),
            (
                REMOVE_EFFECTS_JAVA,
                vec![
                    "RegistryCodecs.homogeneousList(Registries.MOB_EFFECT).fieldOf(\"effects\")",
                    "if (user.removeEffect(effect))",
                    "return hasRemovedAny;",
                ],
            ),
            (
                PLAY_SOUND_JAVA,
                vec![
                    "SoundEvent.CODEC.fieldOf(\"sound\")",
                    "level.playSound(null, user.blockPosition(), this.sound.value(), user.getSoundSource(), 1.0F, 1.0F);",
                    "return true;",
                ],
            ),
        ] {
            for sentinel in sentinels {
                assert!(source.contains(sentinel), "missing consume-effect sentinel {sentinel}");
            }
        }

        let context = ConsumeEffectContext {
            active_effects: vec!["minecraft:speed"],
            ..ConsumeEffectContext::basic("minecraft:test", 1)
        };
        assert!(ConsumeEffectModel::ClearAllStatusEffects.apply(&context).changed);
        let removed = ConsumeEffectModel::RemoveStatusEffects {
            effects: vec!["minecraft:strength", "minecraft:speed"],
        }
        .apply(&context);
        assert_eq!(removed.removed_effects, vec!["minecraft:speed"]);
        assert!(removed.changed);
        let sound = ConsumeEffectModel::PlaySound {
            sound: "minecraft:item.honey_bottle.drink",
        }
        .apply(&context);
        assert_eq!(sound.played_sounds[0].volume, 1.0);
        assert!(sound.changed);
    }

    #[test]
    fn teleport_randomly_uses_java_attempts_clamp_sound_and_reset_rules() {
        for sentinel in [
            "private static final float DEFAULT_DIAMETER = 16.0F;",
            "for (int attempt = 0; attempt < 16; attempt++)",
            "Mth.clamp(",
            "if (user.isPassenger())",
            "user.stopRiding();",
            "if (user.randomTeleport(xx, yy, zz, true))",
            "GameEvent.TELEPORT",
            "SoundEvents.FOX_TELEPORT",
            "SoundEvents.CHORUS_FRUIT_TELEPORT",
            "user.resetFallDistance();",
            "user.resetCurrentImpulseContext();",
        ] {
            assert!(
                TELEPORT_JAVA.contains(sentinel),
                "missing TeleportRandomly sentinel {sentinel}"
            );
        }

        let teleport = ConsumeEffectModel::teleport_randomly_default();
        let outcome = teleport.apply(&ConsumeEffectContext {
            level_min_y: -64.0,
            logical_height: 384.0,
            entity_position: (10.0, -60.0, 20.0),
            is_fox: true,
            is_passenger: true,
            random_doubles: vec![1.0, 0.0, 0.25],
            teleport_successes: vec![true],
            ..ConsumeEffectContext::basic("minecraft:chorus_fruit", 7)
        });

        assert_eq!(outcome.teleport_attempts.len(), 1);
        assert_eq!(outcome.teleport_attempts[0].x, 18.0);
        assert_eq!(outcome.teleport_attempts[0].y, -64.0);
        assert_eq!(outcome.teleport_attempts[0].z, 16.0);
        assert!(outcome.stopped_riding);
        assert_eq!(outcome.played_sounds[0].sound, "minecraft:entity.fox.teleport");
        assert_eq!(outcome.played_sounds[0].source, "neutral");
        assert_eq!(outcome.game_events, vec!["minecraft:teleport"]);
        assert!(outcome.reset_fall_distance);
        assert!(outcome.reset_current_impulse_context);
        assert!(PositiveFloat::new(0.0).is_none());
    }
}
