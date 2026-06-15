//! Java `DiscFragmentItem` hover-text behavior.
//!
//! TODO(live-item-tooltip-rendering): route this through the eventual item tooltip
//! rendering pipeline when the server exposes item tooltips.

#![allow(
    dead_code,
    reason = "tooltip-only item model awaits TODO(live-item-tooltip-rendering)"
)]

use crate::chat_component::{Component, Style, TextColor};

pub fn disc_fragment_display_name(item_id: &str) -> Option<Component> {
    let path = item_id.strip_prefix("minecraft:")?;
    Some(
        Component::translatable(format!("item.minecraft.{path}.desc"), Vec::new())
            .styled(Style::empty().with_color(TextColor::parse("gray")?)),
    )
}

pub fn append_disc_fragment_hover_text(item_id: &str, tooltip: &mut Vec<Component>) -> bool {
    let Some(display_name) = disc_fragment_display_name(item_id) else {
        return false;
    };
    tooltip.push(display_name);
    true
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::ComponentContent;

    const DISC_FRAGMENT_ITEM_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/item/DiscFragmentItem.java");

    #[test]
    fn disc_fragment_hover_text_matches_java_append_hover_text() {
        assert!(DISC_FRAGMENT_ITEM_JAVA.contains("builder.accept(this.getDisplayName().withStyle(ChatFormatting.GRAY));"));
        assert!(DISC_FRAGMENT_ITEM_JAVA.contains("return Component.translatable(this.descriptionId + \".desc\");"));

        let mut tooltip = Vec::new();
        assert!(append_disc_fragment_hover_text(
            "minecraft:disc_fragment_5",
            &mut tooltip
        ));
        assert_eq!(tooltip.len(), 1);
        assert_eq!(
            tooltip[0].content,
            ComponentContent::Translatable {
                key: "item.minecraft.disc_fragment_5.desc".to_string(),
                fallback: None,
                args: Vec::new()
            }
        );
        assert_eq!(tooltip[0].style.color.as_ref().unwrap().serialize(), "gray");
        assert!(!append_disc_fragment_hover_text("disc_fragment_5", &mut tooltip));
    }
}
