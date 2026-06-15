#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipDisplayComponent {
    pub hide_tooltip: bool,
    pub hidden_components: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TooltipContextModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TooltipFlagModel {
    pub advanced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipComponentModel {
    pub translation_key: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TooltipComponentGetterModel {
    pub components: Vec<&'static str>,
}

pub trait TooltipProviderModel {
    fn add_to_tooltip(
        &self,
        context: &TooltipContextModel,
        consumer: &mut Vec<TooltipComponentModel>,
        flag: TooltipFlagModel,
        components: &TooltipComponentGetterModel,
    );
}

impl TooltipDisplayComponent {
    pub fn default_component() -> Self {
        Self {
            hide_tooltip: false,
            hidden_components: Vec::new(),
        }
    }

    pub fn with_hidden(&self, component: &'static str, hidden: bool) -> Self {
        let already_hidden = self.hidden_components.contains(&component);
        if already_hidden == hidden {
            return self.clone();
        }

        let mut hidden_components = self.hidden_components.clone();
        if hidden {
            hidden_components.push(component);
        } else {
            hidden_components.retain(|existing| *existing != component);
        }
        Self {
            hide_tooltip: self.hide_tooltip,
            hidden_components,
        }
    }

    pub fn shows(&self, component: &'static str) -> bool {
        !self.hide_tooltip && !self.hidden_components.contains(&component)
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const TOOLTIP_DISPLAY_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/TooltipDisplay.java");
    const TOOLTIP_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/TooltipProvider.java");

    struct StaticTooltipProvider;

    impl TooltipProviderModel for StaticTooltipProvider {
        fn add_to_tooltip(
            &self,
            _context: &TooltipContextModel,
            consumer: &mut Vec<TooltipComponentModel>,
            _flag: TooltipFlagModel,
            _components: &TooltipComponentGetterModel,
        ) {
            consumer.push(TooltipComponentModel {
                translation_key: "item.test.tooltip",
            });
        }
    }

    #[test]
    fn tooltip_display_defaults_hidden_set_and_shows_match_java() {
        for sentinel in [
            "public record TooltipDisplay(boolean hideTooltip, SequencedSet<DataComponentType<?>> hiddenComponents)",
            "Codec.BOOL.optionalFieldOf(\"hide_tooltip\", false).forGetter(TooltipDisplay::hideTooltip)",
            "COMPONENT_SET_CODEC.optionalFieldOf(\"hidden_components\", ReferenceSortedSets.emptySet()).forGetter(TooltipDisplay::hiddenComponents)",
            "public static final TooltipDisplay DEFAULT = new TooltipDisplay(false, ReferenceSortedSets.emptySet());",
            "public TooltipDisplay withHidden(final DataComponentType<?> component, final boolean hidden)",
            "return !this.hideTooltip && !this.hiddenComponents.contains(component);",
        ] {
            assert!(
                TOOLTIP_DISPLAY_JAVA.contains(sentinel),
                "missing TooltipDisplay sentinel {sentinel}"
            );
        }

        let default = TooltipDisplayComponent::default_component();
        assert!(!default.hide_tooltip);
        assert!(default.hidden_components.is_empty());
        assert!(default.shows("minecraft:lore"));

        let hidden = default.with_hidden("minecraft:lore", true);
        assert_eq!(hidden.hidden_components, vec!["minecraft:lore"]);
        assert!(!hidden.shows("minecraft:lore"));
        assert!(hidden.shows("minecraft:dyed_color"));
        assert_eq!(hidden.with_hidden("minecraft:lore", true), hidden);
        assert_eq!(
            hidden.with_hidden("minecraft:lore", false),
            TooltipDisplayComponent::default_component()
        );

        let globally_hidden = TooltipDisplayComponent {
            hide_tooltip: true,
            hidden_components: Vec::new(),
        };
        assert!(!globally_hidden.shows("minecraft:dyed_color"));
    }

    #[test]
    fn tooltip_provider_signature_is_modeled_like_java_interface() {
        for sentinel in [
            "public interface TooltipProvider",
            "void addToTooltip(Item.TooltipContext context, Consumer<Component> consumer, TooltipFlag flag, DataComponentGetter components);",
        ] {
            assert!(
                TOOLTIP_PROVIDER_JAVA.contains(sentinel),
                "missing TooltipProvider sentinel {sentinel}"
            );
        }

        let provider = StaticTooltipProvider;
        let mut tooltip = Vec::new();
        provider.add_to_tooltip(
            &TooltipContextModel,
            &mut tooltip,
            TooltipFlagModel { advanced: true },
            &TooltipComponentGetterModel::default(),
        );
        assert_eq!(
            tooltip,
            vec![TooltipComponentModel {
                translation_key: "item.test.tooltip",
            }]
        );
    }
}
