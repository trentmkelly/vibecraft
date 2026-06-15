use crate::chat_component::{formatted_text::FormattedText, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormattedCharSequenceDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Eq)]
pub struct FormattedCharSequenceModel {
    pub text: String,
    pub style: Style,
    pub direction: FormattedCharSequenceDirection,
    pub reverse_modifier: Option<fn(u32) -> u32>,
}

#[derive(Debug, Clone)]
pub struct SubStringSourceModel {
    plain_text: String,
    char_styles: Vec<Style>,
    reverse_char_modifier: fn(u32) -> u32,
}

impl PartialEq for FormattedCharSequenceModel {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.style == other.style
            && self.direction == other.direction
            && self.reverse_modifier.is_some() == other.reverse_modifier.is_some()
    }
}

impl FormattedCharSequenceModel {
    pub fn forward(text: String, style: Style) -> Self {
        Self {
            text,
            style,
            direction: FormattedCharSequenceDirection::Forward,
            reverse_modifier: None,
        }
    }

    pub fn backward(text: String, style: Style, reverse_modifier: fn(u32) -> u32) -> Self {
        Self {
            text,
            style,
            direction: FormattedCharSequenceDirection::Backward,
            reverse_modifier: Some(reverse_modifier),
        }
    }

    pub fn codepoints(&self) -> Vec<u32> {
        let mut codepoints = self.text.chars().map(|ch| ch as u32).collect::<Vec<_>>();
        if self.direction == FormattedCharSequenceDirection::Backward {
            codepoints.reverse();
            if let Some(modifier) = self.reverse_modifier {
                for codepoint in &mut codepoints {
                    *codepoint = modifier(*codepoint);
                }
            }
        }
        codepoints
    }
}

impl SubStringSourceModel {
    pub fn create(text: &FormattedText) -> Self {
        Self::create_with(text, identity_codepoint, |plain| plain.to_string())
    }

    pub fn create_with(
        text: &FormattedText,
        reverse_char_modifier: fn(u32) -> u32,
        shaper: impl Fn(&str) -> String,
    ) -> Self {
        let mut plain_text = String::new();
        let mut char_styles = Vec::new();
        text.visit_styled(
            &mut |style, contents| {
                for ch in contents.chars() {
                    plain_text.push(ch);
                    for _ in 0..ch.len_utf16() {
                        char_styles.push(style.clone());
                    }
                }
                None::<()>
            },
            &Style::empty(),
        );
        Self {
            plain_text: shaper(&plain_text),
            char_styles,
            reverse_char_modifier,
        }
    }

    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    pub fn char_style_count(&self) -> usize {
        self.char_styles.len()
    }

    pub fn substring(
        &self,
        start: usize,
        length: usize,
        reverse: bool,
    ) -> Vec<FormattedCharSequenceModel> {
        if length == 0 {
            return Vec::new();
        }

        let end = start + length;
        let mut parts = Vec::new();
        let mut current_run_style = self.char_styles[start].clone();
        let mut current_run_start = start;

        for actual_index in start + 1..end {
            let char_style = &self.char_styles[actual_index];
            if char_style != &current_run_style {
                parts.push(self.sequence_for_run(
                    current_run_start,
                    actual_index,
                    current_run_style,
                    reverse,
                ));
                current_run_style = char_style.clone();
                current_run_start = actual_index;
            }
        }

        if current_run_start < end {
            parts.push(self.sequence_for_run(current_run_start, end, current_run_style, reverse));
        }

        if reverse {
            parts.reverse();
        }
        parts
    }

    fn sequence_for_run(
        &self,
        start: usize,
        end: usize,
        style: Style,
        reverse: bool,
    ) -> FormattedCharSequenceModel {
        let text = self.utf16_slice(start, end);
        if reverse {
            FormattedCharSequenceModel::backward(text, style, self.reverse_char_modifier)
        } else {
            FormattedCharSequenceModel::forward(text, style)
        }
    }

    fn utf16_slice(&self, start: usize, end: usize) -> String {
        let byte_start = byte_index_for_utf16_index(&self.plain_text, start);
        let byte_end = byte_index_for_utf16_index(&self.plain_text, end);
        self.plain_text[byte_start..byte_end].to_string()
    }
}

fn identity_codepoint(codepoint: u32) -> u32 {
    codepoint
}

fn byte_index_for_utf16_index(text: &str, target: usize) -> usize {
    let mut utf16_index = 0;
    for (byte_index, ch) in text.char_indices() {
        if utf16_index == target {
            return byte_index;
        }
        utf16_index += ch.len_utf16();
    }
    text.len()
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::TextColor;

    const SUB_STRING_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/SubStringSource.java");
    const FORMATTED_CHAR_SEQUENCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/util/FormattedCharSequence.java");
    const STRING_DECOMPOSER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/util/StringDecomposer.java");

    fn color(name: &str) -> TextColor {
        TextColor::parse(name).unwrap_or_else(|| panic!("missing color {name}"))
    }

    fn shift_codepoint(codepoint: u32) -> u32 {
        codepoint + 1
    }

    #[test]
    fn sub_string_source_java_source_contract_is_tracked() {
        for sentinel in [
            "private final String plainText;",
            "private final List<Style> charStyles;",
            "private final Int2IntFunction reverseCharModifier;",
            "return this.plainText;",
            "if (length == 0)",
            "Style currentRunStyle = this.charStyles.get(start);",
            "if (!charStyle.equals(currentRunStyle))",
            "FormattedCharSequence.backward(currentRunText, currentRunStyle, this.reverseCharModifier)",
            "FormattedCharSequence.forward(currentRunText, currentRunStyle)",
            "return reverse ? Lists.reverse(parts) : parts;",
            "StringDecomposer.iterateFormatted(contents, style",
            "plainText.appendCodePoint(codepoint);",
            "Character.charCount(codepoint)",
            "return new SubStringSource(shaper.apply(plainText.toString()), charStyles, reverseCharModifier);",
        ] {
            assert!(
                SUB_STRING_SOURCE_JAVA.contains(sentinel),
                "missing SubStringSource sentinel: {sentinel}"
            );
        }
        assert!(FORMATTED_CHAR_SEQUENCE_JAVA.contains(
            "static FormattedCharSequence backward(final String plainText, final Style style, final Int2IntFunction modifier)"
        ));
        assert!(STRING_DECOMPOSER_JAVA.contains("public static boolean iterateBackwards"));
    }

    #[test]
    fn sub_string_source_groups_same_style_runs_and_reverses_run_order() {
        let red = Style::empty().with_color(color("red"));
        let blue = Style::empty().with_color(color("blue"));
        let text = FormattedText::composite(vec![
            FormattedText::styled("ab", red.clone()),
            FormattedText::styled("cd", blue.clone()),
            FormattedText::styled("e", blue.clone()),
        ]);
        let source = SubStringSourceModel::create(&text);

        assert_eq!(source.plain_text(), "abcde");
        let forward = source.substring(1, 4, false);
        assert_eq!(
            forward,
            vec![
                FormattedCharSequenceModel::forward("b".to_string(), red),
                FormattedCharSequenceModel::forward("cde".to_string(), blue),
            ]
        );

        let reversed = source.substring(1, 4, true);
        assert_eq!(reversed.len(), 2);
        assert_eq!(reversed[0].text, "cde");
        assert_eq!(
            reversed[0].direction,
            FormattedCharSequenceDirection::Backward
        );
        assert_eq!(
            reversed[0].codepoints(),
            vec!['e' as u32, 'd' as u32, 'c' as u32]
        );
        assert_eq!(reversed[1].text, "b");
    }

    #[test]
    fn sub_string_source_counts_styles_per_utf16_code_unit_and_shapes_plain_text() {
        let gold = Style::empty().with_color(color("gold"));
        let text = FormattedText::styled("a😀b", gold.clone());
        let source = SubStringSourceModel::create_with(&text, shift_codepoint, |plain| {
            plain.replace('a', "A")
        });

        assert_eq!(source.plain_text(), "A😀b");
        assert_eq!(
            source.char_style_count(),
            4,
            "Java stores one style entry for each UTF-16 code unit"
        );
        assert_eq!(source.substring(0, 1, false)[0].text, "A");
        assert_eq!(source.substring(1, 2, false)[0].text, "😀");

        let reversed = source.substring(0, 4, true);
        assert_eq!(reversed.len(), 1);
        assert_eq!(
            reversed[0].codepoints(),
            vec!['b' as u32 + 1, '😀' as u32 + 1, 'A' as u32 + 1]
        );
        assert_eq!(reversed[0].style, gold);
    }

    #[test]
    fn sub_string_source_zero_length_substring_is_empty() {
        let source = SubStringSourceModel::create(&FormattedText::of("text"));
        assert!(source.substring(0, 0, false).is_empty());
        assert!(source.substring(0, 0, true).is_empty());
    }
}
