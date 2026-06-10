#![allow(dead_code)]

use crate::chat_component::{Component, ResolutionContext, Style, TextColor, TranslationTable};

use super::{snbt_string, Tag};

const INLINE_LIST_THRESHOLD: usize = 8;
const MAX_DEPTH: usize = 64;
const MAX_LENGTH: usize = 128;

pub fn to_plain_text_component(tag: &Tag, indentation: &str, sort_keys: bool) -> Component {
    TextComponentTagVisitor::new(indentation, TextComponentStyling::Plain, sort_keys).visit(tag)
}

pub fn to_rich_text_component(tag: &Tag, indentation: &str, sort_keys: bool) -> Component {
    TextComponentTagVisitor::new(indentation, TextComponentStyling::Rich, sort_keys).visit(tag)
}

pub fn to_plain_text(tag: &Tag, indentation: &str, sort_keys: bool) -> String {
    to_plain_text_component(tag, indentation, sort_keys)
        .render_plain(&TranslationTable::default(), &ResolutionContext::default())
}

pub struct TextComponentTagVisitor {
    indentation: String,
    styling: TextComponentStyling,
    sort_keys: bool,
    indent_depth: usize,
    depth: usize,
    result: Component,
}

impl TextComponentTagVisitor {
    pub fn new(
        indentation: impl Into<String>,
        styling: TextComponentStyling,
        sort_keys: bool,
    ) -> Self {
        Self {
            indentation: indentation.into(),
            styling,
            sort_keys,
            indent_depth: 0,
            depth: 0,
            result: Component::empty(),
        }
    }

    pub fn visit(mut self, tag: &Tag) -> Component {
        self.visit_tag(tag);
        self.result
    }

    fn visit_tag(&mut self, tag: &Tag) {
        match tag {
            Tag::End => {}
            Tag::String(value) => self.visit_string(value),
            Tag::Byte(value) => {
                self.append_styled(&value.to_string(), self.styling.number_style());
                self.append_token(Token::ByteSuffix);
            }
            Tag::Short(value) => {
                self.append_styled(&value.to_string(), self.styling.number_style());
                self.append_token(Token::ShortSuffix);
            }
            Tag::Int(value) => self.append_styled(&value.to_string(), self.styling.number_style()),
            Tag::Long(value) => {
                self.append_styled(&value.to_string(), self.styling.number_style());
                self.append_token(Token::LongSuffix);
            }
            Tag::Float(value) => {
                self.append_styled(
                    &java_float_string(*value as f64),
                    self.styling.number_style(),
                );
                self.append_token(Token::FloatSuffix);
            }
            Tag::Double(value) => {
                self.append_styled(&java_float_string(*value), self.styling.number_style());
                self.append_token(Token::DoubleSuffix);
            }
            Tag::ByteArray(values) => self.visit_byte_array(values),
            Tag::List(values) => self.visit_list(values),
            Tag::Compound(values) => self.visit_compound(values),
            Tag::IntArray(values) => self.visit_int_array(values),
            Tag::LongArray(values) => self.visit_long_array(values),
        }
    }

    fn visit_string(&mut self, value: &str) {
        let quoted = snbt_string::quote_and_escape_snbt_string(value);
        let quote = &quoted[..1];
        self.append_literal(quote);
        self.append_styled(&quoted[1..quoted.len() - 1], self.styling.string_style());
        self.append_literal(quote);
    }

    fn visit_byte_array(&mut self, values: &[i8]) {
        self.append_token(Token::ListOpen);
        self.append_token(Token::ByteArrayPrefix);
        self.append_token(Token::ListTypeSeparator);
        for (index, value) in values.iter().take(MAX_LENGTH).enumerate() {
            self.append_literal(" ");
            self.append_styled(&value.to_string(), self.styling.number_style());
            self.append_token(Token::ByteSuffix);
            if index + 1 != values.len() {
                self.append_token(Token::ElementSeparator);
            }
        }
        if values.len() > MAX_LENGTH {
            self.append_token(Token::Folded);
        }
        self.append_token(Token::ListClose);
    }

    fn visit_int_array(&mut self, values: &[i32]) {
        self.append_token(Token::ListOpen);
        self.append_token(Token::IntArrayPrefix);
        self.append_token(Token::ListTypeSeparator);
        for (index, value) in values.iter().take(MAX_LENGTH).enumerate() {
            self.append_literal(" ");
            self.append_styled(&value.to_string(), self.styling.number_style());
            if index + 1 != values.len() {
                self.append_token(Token::ElementSeparator);
            }
        }
        if values.len() > MAX_LENGTH {
            self.append_token(Token::Folded);
        }
        self.append_token(Token::ListClose);
    }

    fn visit_long_array(&mut self, values: &[i64]) {
        self.append_token(Token::ListOpen);
        self.append_token(Token::LongArrayPrefix);
        self.append_token(Token::ListTypeSeparator);
        for (index, value) in values.iter().take(MAX_LENGTH).enumerate() {
            self.append_literal(" ");
            self.append_styled(&value.to_string(), self.styling.number_style());
            self.append_token(Token::LongSuffix);
            if index + 1 != values.len() {
                self.append_token(Token::ElementSeparator);
            }
        }
        if values.len() > MAX_LENGTH {
            self.append_token(Token::Folded);
        }
        self.append_token(Token::ListClose);
    }

    fn visit_list(&mut self, values: &[Tag]) {
        if values.is_empty() {
            self.append_token(Token::ListOpen);
            self.append_token(Token::ListClose);
        } else if self.depth >= MAX_DEPTH {
            self.append_token(Token::ListOpen);
            self.append_token(Token::Folded);
            self.append_token(Token::ListClose);
        } else if !should_wrap_list_elements(values) {
            self.append_token(Token::ListOpen);
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    self.append_token(Token::ElementSeparator);
                    self.append_literal(" ");
                }
                self.append_sub_tag(value, false);
            }
            self.append_token(Token::ListClose);
        } else {
            self.visit_wrapped_list(values);
        }
    }

    fn visit_wrapped_list(&mut self, values: &[Tag]) {
        self.append_token(Token::ListOpen);
        if !self.indentation.is_empty() {
            self.append_literal("\n");
        }

        let entry_indent = self.indentation.repeat(self.indent_depth + 1);
        let element_spacing = if self.indentation.is_empty() {
            " "
        } else {
            "\n"
        };

        for (index, value) in values.iter().take(MAX_LENGTH).enumerate() {
            self.append_literal(&entry_indent);
            self.append_sub_tag(value, true);
            if index + 1 != values.len() {
                self.append_token(Token::ElementSeparator);
                self.append_literal(element_spacing);
            }
        }

        if values.len() > MAX_LENGTH {
            self.append_literal(&entry_indent);
            self.append_token(Token::Folded);
        }

        if !self.indentation.is_empty() {
            self.append_literal("\n");
            self.append_literal(&self.indentation.repeat(self.indent_depth));
        }
        self.append_token(Token::ListClose);
    }

    fn visit_compound(&mut self, values: &[(String, Tag)]) {
        if values.is_empty() {
            self.append_token(Token::StructOpen);
            self.append_token(Token::StructClose);
        } else if self.depth >= MAX_DEPTH {
            self.append_token(Token::StructOpen);
            self.append_token(Token::Folded);
            self.append_token(Token::StructClose);
        } else {
            self.append_token(Token::StructOpen);
            let entries = self.compound_entries(values);
            if !self.indentation.is_empty() {
                self.append_literal("\n");
            }

            let entry_indent = self.indentation.repeat(self.indent_depth + 1);
            let element_spacing = if self.indentation.is_empty() {
                " "
            } else {
                "\n"
            };

            for (index, (key, value)) in entries.iter().enumerate() {
                self.append_literal(&entry_indent);
                self.append_component(self.handle_escape_pretty(key));
                self.append_token(Token::NameValueSeparator);
                self.append_literal(" ");
                self.append_sub_tag(value, true);
                if index + 1 != entries.len() {
                    self.append_token(Token::ElementSeparator);
                    self.append_literal(element_spacing);
                }
            }

            if !self.indentation.is_empty() {
                self.append_literal("\n");
                self.append_literal(&self.indentation.repeat(self.indent_depth));
            }
            self.append_token(Token::StructClose);
        }
    }

    fn compound_entries<'a>(&self, values: &'a [(String, Tag)]) -> Vec<(&'a str, &'a Tag)> {
        let mut entries = values
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect::<Vec<_>>();
        if self.sort_keys {
            entries.sort_by(|left, right| left.0.cmp(right.0));
        }
        entries
    }

    fn append_sub_tag(&mut self, tag: &Tag, indent: bool) {
        if indent {
            self.indent_depth += 1;
        }
        self.depth += 1;
        self.visit_tag(tag);
        self.depth -= 1;
        if indent {
            self.indent_depth -= 1;
        }
    }

    fn handle_escape_pretty(&self, input: &str) -> Component {
        if is_simple_value(input) {
            return Component::literal(input).styled(self.styling.key_style());
        }

        let quoted = snbt_string::quote_and_escape_snbt_string(input);
        let quote = &quoted[..1];
        Component::literal(quote)
            .append(
                Component::literal(&quoted[1..quoted.len() - 1]).styled(self.styling.key_style()),
            )
            .append(Component::literal(quote))
    }

    fn append_literal(&mut self, value: &str) {
        self.append_component(Component::literal(value));
    }

    fn append_styled(&mut self, value: &str, style: Style) {
        self.append_component(Component::literal(value).styled(style));
    }

    fn append_token(&mut self, token: Token) {
        self.append_component(self.styling.token(token));
    }

    fn append_component(&mut self, component: Component) {
        self.result.siblings.push(component);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextComponentStyling {
    Plain,
    Rich,
}

impl TextComponentStyling {
    fn key_style(self) -> Style {
        match self {
            Self::Plain => Style::empty(),
            Self::Rich => Style::empty().with_color(text_color("aqua")),
        }
    }

    fn string_style(self) -> Style {
        match self {
            Self::Plain => Style::empty(),
            Self::Rich => Style::empty().with_color(text_color("green")),
        }
    }

    fn number_style(self) -> Style {
        match self {
            Self::Plain => Style::empty(),
            Self::Rich => Style::empty().with_color(text_color("gold")),
        }
    }

    fn token(self, token: Token) -> Component {
        let style = match (self, token) {
            (Self::Rich, Token::Folded) => Style::empty().with_color(text_color("gray")),
            (
                Self::Rich,
                Token::ByteSuffix
                | Token::ByteArrayPrefix
                | Token::ShortSuffix
                | Token::IntArrayPrefix
                | Token::LongSuffix
                | Token::LongArrayPrefix
                | Token::FloatSuffix
                | Token::DoubleSuffix,
            ) => Style::empty().with_color(text_color("red")),
            _ => Style::empty(),
        };
        Component::literal(token.text()).styled(style)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Folded,
    ElementSeparator,
    ListClose,
    ListOpen,
    ListTypeSeparator,
    StructClose,
    StructOpen,
    NameValueSeparator,
    ByteSuffix,
    ByteArrayPrefix,
    ShortSuffix,
    IntArrayPrefix,
    LongSuffix,
    LongArrayPrefix,
    FloatSuffix,
    DoubleSuffix,
}

impl Token {
    fn text(self) -> &'static str {
        match self {
            Self::Folded => "<...>",
            Self::ElementSeparator => ",",
            Self::ListClose => "]",
            Self::ListOpen => "[",
            Self::ListTypeSeparator => ";",
            Self::StructClose => "}",
            Self::StructOpen => "{",
            Self::NameValueSeparator => ":",
            Self::ByteSuffix => "b",
            Self::ByteArrayPrefix => "B",
            Self::ShortSuffix => "s",
            Self::IntArrayPrefix => "I",
            Self::LongSuffix => "L",
            Self::LongArrayPrefix => "L",
            Self::FloatSuffix => "f",
            Self::DoubleSuffix => "d",
        }
    }
}

fn should_wrap_list_elements(values: &[Tag]) -> bool {
    values.len() < INLINE_LIST_THRESHOLD
        && values.iter().any(|value| {
            !matches!(
                value,
                Tag::Byte(_)
                    | Tag::Short(_)
                    | Tag::Int(_)
                    | Tag::Long(_)
                    | Tag::Float(_)
                    | Tag::Double(_)
            )
        })
}

fn is_simple_value(input: &str) -> bool {
    !input.is_empty()
        && input
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-'))
}

fn java_float_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value == f64::INFINITY {
        return "Infinity".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_string();
    }

    let mut rendered = value.to_string();
    if let Some(exponent_index) = rendered.find('e') {
        if !rendered[..exponent_index].contains('.') {
            rendered.insert_str(exponent_index, ".0");
        }
        if let Some(exponent_index) = rendered.find('e') {
            rendered.replace_range(exponent_index..=exponent_index, "E");
        }
    } else if !rendered.contains('.') {
        rendered.push_str(".0");
    }
    rendered
}

fn text_color(name: &str) -> TextColor {
    TextColor::parse(name).unwrap_or_else(|| TextColor::from_rgb(0))
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const TEXT_COMPONENT_TAG_VISITOR_JAVA: &str = include_str!(
        "../../../../decompiled-server-26.1.2/net/minecraft/nbt/TextComponentTagVisitor.java"
    );

    #[test]
    fn text_component_visitor_matches_java_constants_and_tokens() {
        for sentinel in [
            "private static final int INLINE_LIST_THRESHOLD = 8;",
            "private static final int MAX_DEPTH = 64;",
            "private static final int MAX_LENGTH = 128;",
            "Pattern.compile(\"[A-Za-z0-9._+-]+\")",
            "this.append(String.valueOf(tag.value()), this.styling.numberStyle()).append(TextComponentTagVisitor.Token.LONG_SUFFIX);",
            "BYTE_SUFFIX(\"b\")",
            "LONG_SUFFIX(\"L\")",
            "FOLDED(\"<...>\")",
            "Style.EMPTY.withColor(ChatFormatting.AQUA)",
            "Style.EMPTY.withColor(ChatFormatting.GREEN)",
            "Style.EMPTY.withColor(ChatFormatting.GOLD)",
            "Style.EMPTY.withColor(ChatFormatting.RED)",
        ] {
            assert!(
                TEXT_COMPONENT_TAG_VISITOR_JAVA.contains(sentinel),
                "missing TextComponentTagVisitor sentinel {sentinel}"
            );
        }

        assert_eq!(to_plain_text(&Tag::End, "  ", true), "");
        assert_eq!(to_plain_text(&Tag::Long(4), "  ", true), "4L");
        assert_eq!(to_plain_text(&Tag::Float(1.0), "  ", true), "1.0f");
        assert_eq!(to_plain_text(&Tag::Double(2.0), "  ", true), "2.0d");
    }

    #[test]
    fn text_component_visitor_plain_output_matches_java_layout_rules() {
        let tag = Tag::Compound(vec![
            ("z".to_string(), Tag::Int(3)),
            ("a key".to_string(), Tag::String("A \"quote\"".to_string())),
            (
                "list".to_string(),
                Tag::List(vec![
                    Tag::Compound(vec![("inner".to_string(), Tag::Byte(1))]),
                    Tag::Compound(vec![("inner".to_string(), Tag::Byte(2))]),
                ]),
            ),
        ]);

        assert_eq!(
            to_plain_text(&tag, "  ", true),
            "{\n  \"a key\": 'A \"quote\"',\n  list: [\n    {\n      inner: 1b\n    },\n    {\n      inner: 2b\n    }\n  ],\n  z: 3\n}"
        );
        assert_eq!(
            to_plain_text(&Tag::List(vec![Tag::Int(1); 8]), "  ", true),
            "[1, 1, 1, 1, 1, 1, 1, 1]"
        );
    }

    #[test]
    fn text_component_visitor_folds_long_arrays_lists_and_deep_compounds() {
        let bytes = Tag::ByteArray((0..130).map(|value| value as i8).collect());
        let rendered = to_plain_text(&bytes, "", true);
        assert!(rendered.starts_with("[B; 0b,"));
        assert!(rendered.ends_with(",<...>]"));

        let long_numeric_list = Tag::List((0..130).map(Tag::Int).collect());
        assert!(to_plain_text(&long_numeric_list, "", true).ends_with(", 129]"));

        let mut deep = Tag::Compound(vec![("leaf".to_string(), Tag::Int(1))]);
        for index in 0..64 {
            deep = Tag::Compound(vec![(format!("level{index}"), deep)]);
        }
        assert!(to_plain_text(&deep, "", true).contains("{<...>}"));
    }

    #[test]
    fn text_component_visitor_rich_styles_match_java_highlighting_roles() {
        let component = to_rich_text_component(
            &Tag::Compound(vec![
                ("name".to_string(), Tag::String("value".to_string())),
                ("count".to_string(), Tag::Long(5)),
            ]),
            "",
            true,
        );
        let json = component.to_json();
        assert!(json.contains("\"color\":\"aqua\""));
        assert!(json.contains("\"color\":\"green\""));
        assert!(json.contains("\"color\":\"gold\""));
        assert!(json.contains("\"color\":\"red\""));
        assert!(json.contains("\"text\":\"L\",\"color\":\"red\""));
    }
}
