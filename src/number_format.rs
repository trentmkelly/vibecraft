use crate::chat_component::{Component, Style, TextColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberFormatModel {
    Blank,
    Styled(Style),
    Fixed(Component),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatTypeModel {
    Blank,
    Styled,
    Fixed,
}

impl NumberFormatModel {
    pub fn blank() -> Self {
        Self::Blank
    }

    pub fn styled(style: Style) -> Self {
        Self::Styled(style)
    }

    pub fn fixed(value: Component) -> Self {
        Self::Fixed(value)
    }

    pub fn no_style() -> Self {
        Self::Styled(Style::empty())
    }

    pub fn sidebar_default() -> Self {
        Self::Styled(Style::empty().with_color(legacy_color("red")))
    }

    pub fn player_list_default() -> Self {
        Self::Styled(Style::empty().with_color(legacy_color("yellow")))
    }

    pub fn format(&self, value: i32) -> Component {
        match self {
            Self::Blank => Component::empty(),
            Self::Styled(style) => Component::literal(value.to_string()).styled(style.clone()),
            Self::Fixed(component) => component.clone(),
        }
    }

    pub fn format_type(&self) -> NumberFormatTypeModel {
        match self {
            Self::Blank => NumberFormatTypeModel::Blank,
            Self::Styled(_) => NumberFormatTypeModel::Styled,
            Self::Fixed(_) => NumberFormatTypeModel::Fixed,
        }
    }
}

impl NumberFormatTypeModel {
    pub fn registry_name(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::Styled => "styled",
            Self::Fixed => "fixed",
        }
    }

    pub fn stream_registry_id(self) -> i32 {
        match self {
            Self::Blank => 0,
            Self::Styled => 1,
            Self::Fixed => 2,
        }
    }
}

pub fn bootstrap_number_format_types() -> Vec<(&'static str, NumberFormatTypeModel)> {
    vec![
        ("blank", NumberFormatTypeModel::Blank),
        ("styled", NumberFormatTypeModel::Styled),
        ("fixed", NumberFormatTypeModel::Fixed),
    ]
}

fn legacy_color(name: &str) -> TextColor {
    match TextColor::parse(name) {
        Some(color) => color,
        None => panic!("missing legacy color {name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::play::NumberFormat as PacketNumberFormat;
    use crate::storage::nbt::Tag;

    fn text(value: &str) -> Component {
        Component::literal(value)
    }

    #[test]
    fn blank_format_returns_empty_component_and_type() {
        let formatted = NumberFormatModel::blank().format(42);

        assert_eq!(formatted, Component::empty());
        assert_eq!(
            NumberFormatModel::blank().format_type(),
            NumberFormatTypeModel::Blank
        );
        assert_eq!(NumberFormatTypeModel::Blank.registry_name(), "blank");
        assert_eq!(NumberFormatTypeModel::Blank.stream_registry_id(), 0);
    }

    #[test]
    fn fixed_format_returns_a_copy_of_the_component_and_type() {
        let original = text("fixed").styled(Style::empty().with_bold(true));
        let format = NumberFormatModel::fixed(original.clone());
        let mut formatted = format.format(99);

        assert_eq!(formatted, original);
        formatted.siblings.push(Component::literal("mutated"));
        assert_eq!(format.format(99), original);
        assert_eq!(format.format_type(), NumberFormatTypeModel::Fixed);
        assert_eq!(NumberFormatTypeModel::Fixed.registry_name(), "fixed");
        assert_eq!(NumberFormatTypeModel::Fixed.stream_registry_id(), 2);
    }

    #[test]
    fn styled_format_formats_integer_string_with_style_and_defaults() {
        let style = Style::empty().with_color(legacy_color("green"));
        let formatted = NumberFormatModel::styled(style.clone()).format(-12);

        assert_eq!(formatted, Component::literal("-12").styled(style));
        assert_eq!(
            NumberFormatModel::styled(Style::empty()).format_type(),
            NumberFormatTypeModel::Styled
        );
        assert_eq!(NumberFormatTypeModel::Styled.registry_name(), "styled");
        assert_eq!(NumberFormatTypeModel::Styled.stream_registry_id(), 1);

        assert_eq!(
            NumberFormatModel::no_style(),
            NumberFormatModel::Styled(Style::empty())
        );
        assert_eq!(
            NumberFormatModel::sidebar_default(),
            NumberFormatModel::Styled(Style::empty().with_color(legacy_color("red")))
        );
        assert_eq!(
            NumberFormatModel::player_list_default(),
            NumberFormatModel::Styled(Style::empty().with_color(legacy_color("yellow")))
        );
    }

    #[test]
    fn number_format_types_bootstrap_in_java_registry_order() {
        assert_eq!(
            bootstrap_number_format_types(),
            vec![
                ("blank", NumberFormatTypeModel::Blank),
                ("styled", NumberFormatTypeModel::Styled),
                ("fixed", NumberFormatTypeModel::Fixed),
            ]
        );
    }

    #[test]
    fn chat_number_format_stream_ids_match_existing_play_packet_codec() {
        let cases = [
            (
                NumberFormatTypeModel::Blank,
                PacketNumberFormat::Blank,
                vec![0],
            ),
            (
                NumberFormatTypeModel::Styled,
                PacketNumberFormat::Styled {
                    style: Tag::Compound(Vec::new()),
                },
                vec![1, 10, 0],
            ),
            (
                NumberFormatTypeModel::Fixed,
                PacketNumberFormat::Fixed {
                    value: Tag::Compound(vec![("text".to_string(), Tag::String("x".to_string()))]),
                },
                vec![2, 10, 8, 0, 4, b't', b'e', b'x', b't', 0, 1, b'x', 0],
            ),
        ];

        for (format_type, packet_format, expected) in cases {
            let mut bytes = Vec::new();
            packet_format
                .write(&mut bytes)
                .unwrap_or_else(|err| panic!("{err}"));
            assert_eq!(bytes[0], format_type.stream_registry_id() as u8);
            assert_eq!(bytes, expected);
        }
    }
}
