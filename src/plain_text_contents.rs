use crate::chat_component::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlainTextContentsModel {
    Empty,
    Literal(String),
}

impl PlainTextContentsModel {
    pub const CODEC_FIELD: &'static str = "text";

    pub fn create(text: impl Into<String>) -> Self {
        let text = text.into();
        if text.is_empty() {
            Self::Empty
        } else {
            Self::Literal(text)
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Empty => "",
            Self::Literal(text) => text,
        }
    }

    pub fn visit<T>(&self, output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Literal(text) => output(text),
        }
    }

    pub fn visit_styled<T>(
        &self,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Literal(text) => output(current_style, text),
        }
    }

    pub fn component_content_text(&self) -> String {
        self.text().to_string()
    }
}

impl std::fmt::Display for PlainTextContentsModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("empty"),
            Self::Literal(text) => write!(formatter, "literal{{{text}}}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::{Component, ComponentContent, TextColor};

    fn legacy_color(name: &str) -> TextColor {
        match TextColor::parse(name) {
            Some(color) => color,
            None => panic!("missing legacy color {name}"),
        }
    }

    #[test]
    fn plain_text_contents_create_reuses_empty_for_empty_text() {
        assert_eq!(
            PlainTextContentsModel::create(""),
            PlainTextContentsModel::Empty
        );
        assert_eq!(PlainTextContentsModel::create("").text(), "");
        assert_eq!(PlainTextContentsModel::create("").to_string(), "empty");
        assert_eq!(PlainTextContentsModel::CODEC_FIELD, "text");
    }

    #[test]
    fn plain_text_contents_literal_text_and_to_string_match_java() {
        let literal = PlainTextContentsModel::create("hello");

        assert_eq!(
            literal,
            PlainTextContentsModel::Literal("hello".to_string())
        );
        assert_eq!(literal.text(), "hello");
        assert_eq!(literal.to_string(), "literal{hello}");
    }

    #[test]
    fn plain_text_contents_visitors_skip_empty_and_forward_literal_text() {
        let mut visited = Vec::new();
        let empty = PlainTextContentsModel::create("");
        assert_eq!(
            empty.visit::<()>(&mut |text| {
                visited.push(text.to_string());
                Some(())
            }),
            None
        );
        assert!(visited.is_empty());

        let literal = PlainTextContentsModel::create("stop");
        let result = literal.visit(&mut |text| {
            visited.push(text.to_string());
            Some(text.len())
        });
        assert_eq!(result, Some(4));
        assert_eq!(visited, vec!["stop".to_string()]);
    }

    #[test]
    fn plain_text_contents_styled_visit_forwards_current_style() {
        let style = Style::empty().with_color(legacy_color("yellow"));
        let literal = PlainTextContentsModel::create("styled");
        let result = literal.visit_styled(
            &mut |visited_style, text| Some((visited_style.clone(), text.to_string())),
            &style,
        );

        assert_eq!(result, Some((style, "styled".to_string())));
    }

    #[test]
    fn plain_text_contents_maps_to_literal_component_content_text() {
        let literal = PlainTextContentsModel::create("chat");
        let component = Component::literal(literal.component_content_text());

        assert_eq!(
            component.content,
            ComponentContent::Literal("chat".to_string())
        );
        assert_eq!(component.to_json(), "{\"text\":\"chat\"}");
    }
}
