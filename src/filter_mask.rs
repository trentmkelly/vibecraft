#![allow(dead_code)]

use crate::chat_component::{Component, HoverEvent, Style, TextColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterMask {
    words: Vec<u64>,
    mask_type: FilterMaskType,
}

impl FilterMask {
    pub fn pass_through() -> Self {
        Self {
            words: Vec::new(),
            mask_type: FilterMaskType::PassThrough,
        }
    }

    pub fn fully_filtered() -> Self {
        Self {
            words: Vec::new(),
            mask_type: FilterMaskType::FullyFiltered,
        }
    }

    pub fn partially_filtered(length: usize) -> Self {
        Self {
            words: vec![0; length.div_ceil(64)],
            mask_type: FilterMaskType::PartiallyFiltered,
        }
    }

    pub fn from_words(words: Vec<u64>) -> Self {
        Self {
            words: trim_trailing_zero_words(words),
            mask_type: FilterMaskType::PartiallyFiltered,
        }
    }

    pub fn mask_type(&self) -> FilterMaskType {
        self.mask_type
    }

    pub fn words(&self) -> &[u64] {
        &self.words
    }

    pub fn set_filtered(&mut self, index: usize) {
        let word = index / 64;
        if self.words.len() <= word {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1_u64 << (index % 64);
    }

    pub fn apply(&self, text: &str) -> Option<String> {
        match self.mask_type {
            FilterMaskType::PassThrough => Some(text.to_string()),
            FilterMaskType::FullyFiltered => None,
            FilterMaskType::PartiallyFiltered => {
                let mut units: Vec<u16> = text.encode_utf16().collect();
                for index in 0..units.len().min(self.bit_length()) {
                    if self.get(index) {
                        units[index] = b'#' as u16;
                    }
                }
                Some(String::from_utf16_lossy(&units))
            }
        }
    }

    pub fn apply_with_formatting(&self, text: &str) -> Option<Component> {
        match self.mask_type {
            FilterMaskType::PassThrough => Some(Component::literal(text)),
            FilterMaskType::FullyFiltered => None,
            FilterMaskType::PartiallyFiltered => Some(self.apply_partially_with_formatting(text)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mask_type == FilterMaskType::PassThrough
    }

    pub fn is_fully_filtered(&self) -> bool {
        self.mask_type == FilterMaskType::FullyFiltered
    }

    fn apply_partially_with_formatting(&self, text: &str) -> Component {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut result = Component::empty();
        let mut previous_index = 0;
        let mut filtered = self.get(0);

        loop {
            let mut next_index = if filtered {
                self.next_clear_bit(previous_index) as isize
            } else {
                self.next_set_bit(previous_index)
                    .map(|index| index as isize)
                    .unwrap_or(-1)
            };
            if next_index < 0 {
                next_index = units.len() as isize;
            }
            let next_index = next_index as usize;
            if next_index == previous_index {
                return result;
            }

            if filtered {
                result = result.append(
                    Component::literal("#".repeat(next_index - previous_index))
                        .styled(filtered_style()),
                );
            } else {
                result = result.append(Component::literal(utf16_slice_lossy(
                    &units,
                    previous_index,
                    next_index,
                )));
            }

            filtered = !filtered;
            previous_index = next_index;
        }
    }

    fn get(&self, index: usize) -> bool {
        self.words
            .get(index / 64)
            .map(|word| (word & (1_u64 << (index % 64))) != 0)
            .unwrap_or(false)
    }

    fn bit_length(&self) -> usize {
        self.words
            .iter()
            .rposition(|word| *word != 0)
            .map(|index| {
                let word = self.words[index];
                index * 64 + (u64::BITS as usize - word.leading_zeros() as usize)
            })
            .unwrap_or(0)
    }

    fn next_set_bit(&self, from_index: usize) -> Option<usize> {
        let mut word_index = from_index / 64;
        if word_index >= self.words.len() {
            return None;
        }

        let mut word = self.words[word_index] & (!0_u64 << (from_index % 64));
        loop {
            if word != 0 {
                return Some(word_index * 64 + word.trailing_zeros() as usize);
            }
            word_index += 1;
            word = *self.words.get(word_index)?;
        }
    }

    fn next_clear_bit(&self, from_index: usize) -> usize {
        let mut index = from_index;
        while self.get(index) {
            index += 1;
        }
        index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMaskType {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered,
}

impl FilterMaskType {
    pub fn id(self) -> i32 {
        match self {
            Self::PassThrough => 0,
            Self::FullyFiltered => 1,
            Self::PartiallyFiltered => 2,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::FullyFiltered,
            2 => Self::PartiallyFiltered,
            _ => Self::PassThrough,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::PassThrough => "pass_through",
            Self::FullyFiltered => "fully_filtered",
            Self::PartiallyFiltered => "partially_filtered",
        }
    }
}

pub fn filtered_style() -> Style {
    Style::empty()
        .with_color(TextColor::parse("dark_gray").unwrap_or_else(|| TextColor::from_rgb(0x555555)))
        .with_hover_event(HoverEvent::Text(Box::new(Component::translatable(
            "chat.filtered",
            Vec::new(),
        ))))
}

fn trim_trailing_zero_words(mut words: Vec<u64>) -> Vec<u64> {
    while words.last() == Some(&0) {
        words.pop();
    }
    words
}

fn utf16_slice_lossy(units: &[u16], start: usize, end: usize) -> String {
    let bounded_start = start.min(units.len());
    let bounded_end = end.min(units.len());
    String::from_utf16_lossy(&units[bounded_start..bounded_end])
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    #[test]
    fn filter_mask_matches_java_apply_and_formatting_behavior() {
        const FILTER_MASK_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/FilterMask.java"
        );

        for sentinel in [
            "public static final FilterMask FULLY_FILTERED = new FilterMask(new BitSet(0), FilterMask.Type.FULLY_FILTERED);",
            "public static final FilterMask PASS_THROUGH = new FilterMask(new BitSet(0), FilterMask.Type.PASS_THROUGH);",
            "Style.EMPTY",
            ".withColor(ChatFormatting.DARK_GRAY)",
            "new HoverEvent.ShowText(Component.translatable(\"chat.filtered\"))",
            "input.readEnum(FilterMask.Type.class)",
            "output.writeEnum(mask.type);",
            "output.writeBitSet(mask.mask);",
            "this.mask.set(index);",
            "chars[i] = '#';",
            "StringUtils.repeat('#', nextIndex - previousIndex)",
            "PASS_THROUGH(\"pass_through\", () -> FilterMask.PASS_THROUGH_CODEC)",
            "FULLY_FILTERED(\"fully_filtered\", () -> FilterMask.FULLY_FILTERED_CODEC)",
            "PARTIALLY_FILTERED(\"partially_filtered\", () -> FilterMask.PARTIALLY_FILTERED_CODEC)",
        ] {
            assert!(
                FILTER_MASK_JAVA.contains(sentinel),
                "missing FilterMask sentinel {sentinel}"
            );
        }

        assert_eq!(
            FilterMask::pass_through().apply("hello"),
            Some("hello".to_string())
        );
        assert_eq!(FilterMask::fully_filtered().apply("hello"), None);
        assert!(FilterMask::pass_through().is_empty());
        assert!(FilterMask::fully_filtered().is_fully_filtered());

        let mut mask = FilterMask::partially_filtered(8);
        mask.set_filtered(1);
        mask.set_filtered(2);
        mask.set_filtered(5);
        assert_eq!(mask.apply("abcdef"), Some("a##de#".to_string()));
        assert!(!mask.is_empty());
        assert!(!mask.is_fully_filtered());

        let Some(formatted) = mask.apply_with_formatting("abcdef") else {
            panic!("partial mask should format");
        };
        assert_eq!(
            formatted.to_json(),
            "{\"text\":\"\",\"extra\":[{\"text\":\"a\"},{\"text\":\"##\",\"color\":\"dark_gray\",\"hoverEvent\":{\"action\":\"show_text\",\"value\":{\"translate\":\"chat.filtered\"}}},{\"text\":\"de\"},{\"text\":\"#\",\"color\":\"dark_gray\",\"hoverEvent\":{\"action\":\"show_text\",\"value\":{\"translate\":\"chat.filtered\"}}}]}"
        );

        let mut unicode_mask = FilterMask::partially_filtered(4);
        unicode_mask.set_filtered(1);
        unicode_mask.set_filtered(2);
        assert_eq!(unicode_mask.apply("a😀b"), Some("a##b".to_string()));
    }

    #[test]
    fn filter_mask_type_matches_java_enum_wire_order_and_names() {
        assert_eq!(FilterMaskType::PassThrough.id(), 0);
        assert_eq!(FilterMaskType::FullyFiltered.id(), 1);
        assert_eq!(FilterMaskType::PartiallyFiltered.id(), 2);
        assert_eq!(FilterMaskType::from_id(-1), FilterMaskType::PassThrough);
        assert_eq!(FilterMaskType::from_id(99), FilterMaskType::PassThrough);
        assert_eq!(
            FilterMaskType::PassThrough.serialized_name(),
            "pass_through"
        );
        assert_eq!(
            FilterMaskType::FullyFiltered.serialized_name(),
            "fully_filtered"
        );
        assert_eq!(
            FilterMaskType::PartiallyFiltered.serialized_name(),
            "partially_filtered"
        );

        let mask = FilterMask::from_words(vec![0b101, 0]);
        assert_eq!(mask.mask_type(), FilterMaskType::PartiallyFiltered);
        assert_eq!(mask.words(), &[0b101]);
    }
}
