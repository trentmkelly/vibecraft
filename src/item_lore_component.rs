#![allow(dead_code)]

use crate::chat_component::{Component, Style, TextColor};

/// `net.minecraft.world.item.component.ItemLore` — the item lore component.
///
/// Vanilla stores the raw lines and derives tooltip lines by merging every line's
/// style with dark-purple italic lore defaults. Existing line style wins for any
/// field it explicitly set; unset fields inherit the lore style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLoreComponent {
    pub lines: Vec<Component>,
    pub styled_lines: Vec<Component>,
}

impl ItemLoreComponent {
    pub const MAX_LINES: usize = 256;

    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            styled_lines: Vec::new(),
        }
    }

    pub fn new(lines: Vec<Component>) -> Result<Self, ItemLoreError> {
        if lines.len() > Self::MAX_LINES {
            return Err(ItemLoreError::TooManyLines {
                got: lines.len(),
                max: Self::MAX_LINES,
            });
        }
        let styled_lines = lines.iter().cloned().map(merge_lore_style).collect();
        Ok(Self {
            lines,
            styled_lines,
        })
    }

    pub fn with_line_added(&self, component: Component) -> Result<Self, ItemLoreError> {
        let mut lines = self.lines.clone();
        lines.push(component);
        Self::new(lines)
    }

    pub fn add_to_tooltip(&self, consumer: &mut impl FnMut(Component)) {
        for line in &self.styled_lines {
            consumer(line.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemLoreError {
    TooManyLines { got: usize, max: usize },
}

fn merge_lore_style(mut component: Component) -> Component {
    let style = component.style.apply_to(&lore_style());
    component.style = style;
    component
}

fn lore_style() -> Style {
    Style::empty()
        .with_color(TextColor::from_rgb(0xAA_00_AA))
        .with_italic(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::{ComponentContent, TextColor};

    #[test]
    fn item_lore_defaults_to_empty_and_adds_dark_purple_italic_tooltip_lines() {
        let first = ItemLoreComponent::empty()
            .with_line_added(Component::literal("first"))
            .unwrap_or_else(|err| panic!("single lore line should be valid: {err:?}"));
        let lore = first
            .with_line_added(Component::literal("second"))
            .unwrap_or_else(|err| panic!("two lore lines should be valid: {err:?}"));

        assert_eq!(lore.lines.len(), 2);
        assert_eq!(lore.styled_lines[0].get_string(), "first");
        assert_eq!(
            lore.styled_lines[0].style.color.as_ref().map(TextColor::value),
            Some(0xAA_00_AA)
        );
        assert_eq!(lore.styled_lines[0].style.italic, Some(true));

        let mut tooltip = Vec::new();
        lore.add_to_tooltip(&mut |line| tooltip.push(line));
        assert_eq!(tooltip, lore.styled_lines);
    }

    #[test]
    fn item_lore_merge_preserves_explicit_line_style_like_java_component_utils() {
        let line = Component::literal("custom").styled(
            Style::empty()
                .with_color(TextColor::from_rgb(0xFF_55_55))
                .with_bold(true),
        );
        let lore = ItemLoreComponent::new(vec![line.clone()])
            .unwrap_or_else(|err| panic!("single lore line should be valid: {err:?}"));

        assert_eq!(lore.lines, vec![line]);
        assert_eq!(
            lore.styled_lines[0].style.color.as_ref().map(TextColor::value),
            Some(0xFF_55_55)
        );
        assert_eq!(lore.styled_lines[0].style.bold, Some(true));
        assert_eq!(lore.styled_lines[0].style.italic, Some(true));
    }

    #[test]
    fn item_lore_rejects_more_than_java_max_lines() {
        let too_many = vec![Component::empty(); ItemLoreComponent::MAX_LINES + 1];
        assert_eq!(
            ItemLoreComponent::new(too_many),
            Err(ItemLoreError::TooManyLines {
                got: 257,
                max: 256
            })
        );
    }

    #[test]
    fn item_lore_preserves_non_literal_component_content() {
        let component = Component::translatable("item.minecraft.diamond", Vec::new());
        let lore = ItemLoreComponent::new(vec![component])
            .unwrap_or_else(|err| panic!("single lore line should be valid: {err:?}"));

        assert!(matches!(
            lore.styled_lines[0].content,
            ComponentContent::Translatable { .. }
        ));
    }
}
