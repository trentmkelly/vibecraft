#![allow(dead_code)]

use crate::chat_component::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormattedText {
    Empty,
    Plain(String),
    Styled { text: String, style: Box<Style> },
    Composite(Vec<FormattedText>),
}

impl FormattedText {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn of(text: impl Into<String>) -> Self {
        Self::Plain(text.into())
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self::Styled {
            text: text.into(),
            style: Box::new(style),
        }
    }

    pub fn composite(parts: Vec<FormattedText>) -> Self {
        Self::Composite(parts)
    }

    pub fn visit<T>(&self, output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Plain(text) | Self::Styled { text, .. } => output(text),
            Self::Composite(parts) => {
                for part in parts {
                    if let Some(result) = part.visit(output) {
                        return Some(result);
                    }
                }
                None
            }
        }
    }

    pub fn visit_styled<T>(
        &self,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        parent_style: &Style,
    ) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Plain(text) => output(parent_style, text),
            Self::Styled { text, style } => output(&style.apply_to(parent_style), text),
            Self::Composite(parts) => {
                for part in parts {
                    if let Some(result) = part.visit_styled(output, parent_style) {
                        return Some(result);
                    }
                }
                None
            }
        }
    }

    pub fn get_string(&self) -> String {
        let mut builder = String::new();
        self.visit(&mut |contents| {
            builder.push_str(contents);
            None::<()>
        });
        builder
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::TextColor;

    #[test]
    fn formatted_text_matches_java_visitors_and_short_circuiting() {
        const FORMATTED_TEXT_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/FormattedText.java"
        );
        const STYLE_JAVA: &str =
            include_str!("../../decompiled-server-26.1.2/net/minecraft/network/chat/Style.java");

        for sentinel in [
            "Optional<Unit> STOP_ITERATION = Optional.of(Unit.INSTANCE);",
            "FormattedText EMPTY = new FormattedText()",
            "static FormattedText of(final String text)",
            "static FormattedText of(final String text, final Style style)",
            "return output.accept(style.applyTo(parentStyle), text);",
            "static FormattedText composite(final List<? extends FormattedText> parts)",
            "if (result.isPresent())",
            "default String getString()",
            "builder.append(contents);",
        ] {
            assert!(
                FORMATTED_TEXT_JAVA.contains(sentinel),
                "missing FormattedText sentinel {sentinel}"
            );
        }
        assert!(
            STYLE_JAVA.contains("public Style applyTo(final Style other)"),
            "missing Style.applyTo sentinel"
        );

        assert_eq!(FormattedText::empty().get_string(), "");
        assert_eq!(FormattedText::of("hello").get_string(), "hello");
        assert_eq!(
            FormattedText::composite(vec![
                FormattedText::of("a"),
                FormattedText::of("b"),
                FormattedText::of("c"),
            ])
            .get_string(),
            "abc"
        );

        let mut visited = Vec::new();
        let stop = FormattedText::composite(vec![
            FormattedText::of("first"),
            FormattedText::of("stop"),
            FormattedText::of("never"),
        ])
        .visit(&mut |contents| {
            visited.push(contents.to_string());
            (contents == "stop").then_some("stopped")
        });
        assert_eq!(stop, Some("stopped"));
        assert_eq!(visited, vec!["first", "stop"]);
    }

    #[test]
    fn formatted_text_styled_visit_applies_style_to_parent_like_java() {
        let parent = Style::empty()
            .with_color(TextColor::parse("red").unwrap_or_else(|| TextColor::from_rgb(0xFF5555)))
            .with_bold(true);
        let child = Style::empty().with_italic(true);
        let text = FormattedText::styled("styled", child);

        let mut seen = Vec::new();
        let result = text.visit_styled(
            &mut |style, contents| {
                seen.push((style.clone(), contents.to_string()));
                None::<()>
            },
            &parent,
        );
        assert_eq!(result, None);
        assert_eq!(seen.len(), 1);
        let (style, contents) = &seen[0];
        assert_eq!(contents, "styled");
        assert_eq!(
            style.color.as_ref().map(TextColor::serialize).as_deref(),
            Some("red")
        );
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.italic, Some(true));
    }
}
