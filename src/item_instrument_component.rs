#![allow(dead_code)]

use crate::chat_component::{Component, Style, TextColor};
use crate::presentation_data::{InstrumentDef, INSTRUMENTS};

/// `net.minecraft.world.item.Instrument` — direct registry payload for goat horns.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentModel {
    pub id: &'static str,
    pub sound_event: &'static str,
    pub use_duration: f32,
    pub range: f32,
    pub description: Component,
}

impl InstrumentModel {
    pub fn new(
        id: &'static str,
        sound_event: &'static str,
        use_duration: f32,
        range: f32,
        description: Component,
    ) -> Result<Self, InstrumentError> {
        if !is_positive_float(use_duration) {
            return Err(InstrumentError::NonPositiveUseDuration(use_duration));
        }
        if !is_positive_float(range) {
            return Err(InstrumentError::NonPositiveRange(range));
        }
        Ok(Self {
            id,
            sound_event,
            use_duration,
            range,
            description,
        })
    }

    pub fn from_vanilla_id(id: &str) -> Option<Self> {
        let definition = INSTRUMENTS.iter().find(|instrument| instrument.id == id)?;
        Some(Self::from_definition(definition))
    }

    pub fn from_definition(definition: &InstrumentDef) -> Self {
        Self {
            id: definition.id,
            sound_event: definition.sound_event,
            use_duration: definition.use_duration_seconds,
            range: definition.range,
            description: Component::translatable(instrument_translation_key(definition.id), Vec::new()),
        }
    }
}

/// `net.minecraft.world.item.component.InstrumentComponent` — holder component.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentComponent {
    pub instrument: &'static str,
}

impl InstrumentComponent {
    pub fn new(instrument: &'static str) -> Self {
        Self { instrument }
    }

    pub fn instrument(&self) -> Option<InstrumentModel> {
        InstrumentModel::from_vanilla_id(self.instrument)
    }

    pub fn add_to_tooltip(&self, consumer: &mut impl FnMut(Component)) {
        if let Some(instrument) = self.instrument() {
            consumer(merge_gray_style(instrument.description));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstrumentError {
    NonPositiveUseDuration(f32),
    NonPositiveRange(f32),
}

fn instrument_translation_key(id: &str) -> String {
    let short_id = id.strip_prefix("minecraft:").unwrap_or(id);
    format!("instrument.minecraft.{short_id}")
}

fn merge_gray_style(mut component: Component) -> Component {
    component.style = component.style.apply_to(&gray_style());
    component
}

fn gray_style() -> Style {
    Style::empty().with_color(TextColor::from_rgb(0xAA_AA_AA))
}

fn is_positive_float(value: f32) -> bool {
    matches!(value.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTRUMENT_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/world/item/Instrument.java");
    const INSTRUMENT_COMPONENT_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/InstrumentComponent.java"
    );

    #[test]
    fn instrument_direct_codec_shape_and_vanilla_registry_match_java() {
        for sentinel in [
            "public record Instrument(Holder<SoundEvent> soundEvent, float useDuration, float range, Component description)",
            "ExtraCodecs.POSITIVE_FLOAT.fieldOf(\"use_duration\")",
            "ExtraCodecs.POSITIVE_FLOAT.fieldOf(\"range\")",
            "ComponentSerialization.CODEC.fieldOf(\"description\")",
            "RegistryFileCodec.create(Registries.INSTRUMENT, DIRECT_CODEC)",
            "ByteBufCodecs.holder(Registries.INSTRUMENT, DIRECT_STREAM_CODEC)",
        ] {
            assert!(
                INSTRUMENT_JAVA.contains(sentinel),
                "missing Instrument sentinel {sentinel}"
            );
        }

        assert_eq!(INSTRUMENTS.len(), 8);
        let ponder = InstrumentModel::from_vanilla_id("minecraft:ponder_goat_horn")
            .unwrap_or_else(|| panic!("ponder goat horn should exist"));
        assert_eq!(ponder.sound_event, "minecraft:item.goat_horn.sound.0");
        assert_eq!(ponder.use_duration, 7.0);
        assert_eq!(ponder.range, 256.0);
        assert_eq!(
            ponder.description.to_json(),
            "{\"translate\":\"instrument.minecraft.ponder_goat_horn\"}"
        );

        assert_eq!(
            InstrumentModel::new(
                "minecraft:test",
                "minecraft:sound",
                0.0,
                1.0,
                Component::empty()
            ),
            Err(InstrumentError::NonPositiveUseDuration(0.0))
        );
        assert_eq!(
            InstrumentModel::new(
                "minecraft:test",
                "minecraft:sound",
                1.0,
                -1.0,
                Component::empty()
            ),
            Err(InstrumentError::NonPositiveRange(-1.0))
        );
    }

    #[test]
    fn instrument_component_tooltip_merges_gray_style_like_java() {
        for sentinel in [
            "public record InstrumentComponent(Holder<Instrument> instrument) implements TooltipProvider",
            "Instrument.CODEC.xmap(InstrumentComponent::new, InstrumentComponent::instrument)",
            "Instrument.STREAM_CODEC",
            "consumer.accept(ComponentUtils.mergeStyles(this.instrument.value().description(), Style.EMPTY.withColor(ChatFormatting.GRAY)))",
        ] {
            assert!(
                INSTRUMENT_COMPONENT_JAVA.contains(sentinel),
                "missing InstrumentComponent sentinel {sentinel}"
            );
        }

        let component = InstrumentComponent::new("minecraft:sing_goat_horn");
        let mut tooltip = Vec::new();
        component.add_to_tooltip(&mut |line| tooltip.push(line));

        assert_eq!(tooltip.len(), 1);
        assert_eq!(
            tooltip[0].to_json(),
            "{\"translate\":\"instrument.minecraft.sing_goat_horn\",\"color\":\"#AAAAAA\"}"
        );
    }

    #[test]
    fn instrument_component_preserves_explicit_description_style_when_merging() {
        let mut instrument =
            InstrumentModel::from_vanilla_id("minecraft:seek_goat_horn").unwrap_or_else(|| {
                panic!("seek goat horn should exist")
            });
        instrument.description = instrument
            .description
            .styled(Style::empty().with_color(TextColor::from_rgb(0xFF_55_55)));

        let merged = merge_gray_style(instrument.description);
        assert_eq!(merged.style.color.as_ref().map(TextColor::value), Some(0xFF_55_55));
    }
}
