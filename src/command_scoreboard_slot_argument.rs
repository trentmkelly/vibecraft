use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardSlotArgumentModel;

impl ScoreboardSlotArgumentModel {
    pub fn display_slot() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<DisplaySlotModel, ScoreboardSlotParseError> {
        let name = reader.read_unquoted_string();
        DisplaySlotModel::by_name(&name)
            .ok_or(ScoreboardSlotParseError::InvalidValue { value: name })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(
            DisplaySlotModel::VALUES
                .iter()
                .map(|slot| slot.serialized_name().to_string()),
            builder,
        );
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["sidebar", "foo.bar"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, DisplaySlotModel>,
}

impl CommandContextModel {
    pub fn with_display_slot(mut self, name: impl Into<String>, value: DisplaySlotModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_display_slot(context: &CommandContextModel, name: &str) -> Option<DisplaySlotModel> {
    context.arguments.get(name).copied()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplaySlotModel {
    List,
    Sidebar,
    BelowName,
    TeamBlack,
    TeamDarkBlue,
    TeamDarkGreen,
    TeamDarkAqua,
    TeamDarkRed,
    TeamDarkPurple,
    TeamGold,
    TeamGray,
    TeamDarkGray,
    TeamBlue,
    TeamGreen,
    TeamAqua,
    TeamRed,
    TeamLightPurple,
    TeamYellow,
    TeamWhite,
}

impl DisplaySlotModel {
    pub const VALUES: [Self; 19] = [
        Self::List,
        Self::Sidebar,
        Self::BelowName,
        Self::TeamBlack,
        Self::TeamDarkBlue,
        Self::TeamDarkGreen,
        Self::TeamDarkAqua,
        Self::TeamDarkRed,
        Self::TeamDarkPurple,
        Self::TeamGold,
        Self::TeamGray,
        Self::TeamDarkGray,
        Self::TeamBlue,
        Self::TeamGreen,
        Self::TeamAqua,
        Self::TeamRed,
        Self::TeamLightPurple,
        Self::TeamYellow,
        Self::TeamWhite,
    ];

    pub fn id(self) -> i32 {
        match self {
            Self::List => 0,
            Self::Sidebar => 1,
            Self::BelowName => 2,
            Self::TeamBlack => 3,
            Self::TeamDarkBlue => 4,
            Self::TeamDarkGreen => 5,
            Self::TeamDarkAqua => 6,
            Self::TeamDarkRed => 7,
            Self::TeamDarkPurple => 8,
            Self::TeamGold => 9,
            Self::TeamGray => 10,
            Self::TeamDarkGray => 11,
            Self::TeamBlue => 12,
            Self::TeamGreen => 13,
            Self::TeamAqua => 14,
            Self::TeamRed => 15,
            Self::TeamLightPurple => 16,
            Self::TeamYellow => 17,
            Self::TeamWhite => 18,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Sidebar => "sidebar",
            Self::BelowName => "below_name",
            Self::TeamBlack => "sidebar.team.black",
            Self::TeamDarkBlue => "sidebar.team.dark_blue",
            Self::TeamDarkGreen => "sidebar.team.dark_green",
            Self::TeamDarkAqua => "sidebar.team.dark_aqua",
            Self::TeamDarkRed => "sidebar.team.dark_red",
            Self::TeamDarkPurple => "sidebar.team.dark_purple",
            Self::TeamGold => "sidebar.team.gold",
            Self::TeamGray => "sidebar.team.gray",
            Self::TeamDarkGray => "sidebar.team.dark_gray",
            Self::TeamBlue => "sidebar.team.blue",
            Self::TeamGreen => "sidebar.team.green",
            Self::TeamAqua => "sidebar.team.aqua",
            Self::TeamRed => "sidebar.team.red",
            Self::TeamLightPurple => "sidebar.team.light_purple",
            Self::TeamYellow => "sidebar.team.yellow",
            Self::TeamWhite => "sidebar.team.white",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|slot| slot.serialized_name() == name)
    }

    pub fn by_id(id: i32) -> Self {
        Self::VALUES
            .into_iter()
            .find(|slot| slot.id() == id)
            .unwrap_or(Self::List)
    }

    pub fn team_color_to_slot(color: ChatFormattingModel) -> Option<Self> {
        match color {
            ChatFormattingModel::Black => Some(Self::TeamBlack),
            ChatFormattingModel::DarkBlue => Some(Self::TeamDarkBlue),
            ChatFormattingModel::DarkGreen => Some(Self::TeamDarkGreen),
            ChatFormattingModel::DarkAqua => Some(Self::TeamDarkAqua),
            ChatFormattingModel::DarkRed => Some(Self::TeamDarkRed),
            ChatFormattingModel::DarkPurple => Some(Self::TeamDarkPurple),
            ChatFormattingModel::Gold => Some(Self::TeamGold),
            ChatFormattingModel::Gray => Some(Self::TeamGray),
            ChatFormattingModel::DarkGray => Some(Self::TeamDarkGray),
            ChatFormattingModel::Blue => Some(Self::TeamBlue),
            ChatFormattingModel::Green => Some(Self::TeamGreen),
            ChatFormattingModel::Aqua => Some(Self::TeamAqua),
            ChatFormattingModel::Red => Some(Self::TeamRed),
            ChatFormattingModel::LightPurple => Some(Self::TeamLightPurple),
            ChatFormattingModel::Yellow => Some(Self::TeamYellow),
            ChatFormattingModel::White => Some(Self::TeamWhite),
            ChatFormattingModel::Bold
            | ChatFormattingModel::Italic
            | ChatFormattingModel::Underline
            | ChatFormattingModel::Reset
            | ChatFormattingModel::Obfuscated
            | ChatFormattingModel::Strikethrough => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFormattingModel {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreboardSlotParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(DisplaySlotModel, usize), ScoreboardSlotParseError> {
        let mut reader = StringReaderModel::new(input);
        let slot = ScoreboardSlotArgumentModel::display_slot().parse(&mut reader)?;
        Ok((slot, reader.cursor()))
    }

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_display_slot_enum_order_ids_and_names_match_source() {
        assert_eq!(
            DisplaySlotModel::VALUES.map(DisplaySlotModel::id),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
        );
        assert_eq!(
            DisplaySlotModel::VALUES.map(DisplaySlotModel::serialized_name),
            [
                "list",
                "sidebar",
                "below_name",
                "sidebar.team.black",
                "sidebar.team.dark_blue",
                "sidebar.team.dark_green",
                "sidebar.team.dark_aqua",
                "sidebar.team.dark_red",
                "sidebar.team.dark_purple",
                "sidebar.team.gold",
                "sidebar.team.gray",
                "sidebar.team.dark_gray",
                "sidebar.team.blue",
                "sidebar.team.green",
                "sidebar.team.aqua",
                "sidebar.team.red",
                "sidebar.team.light_purple",
                "sidebar.team.yellow",
                "sidebar.team.white",
            ]
        );
    }

    #[test]
    fn java_factory_examples_and_context_lookup_match_source() {
        let argument = ScoreboardSlotArgumentModel::display_slot();
        assert_eq!(argument.examples(), ["sidebar", "foo.bar"]);

        let context =
            CommandContextModel::default().with_display_slot("slot", DisplaySlotModel::BelowName);
        assert_eq!(
            get_display_slot(&context, "slot"),
            Some(DisplaySlotModel::BelowName)
        );
        assert_eq!(get_display_slot(&context, "missing"), None);
    }

    #[test]
    fn java_parse_uses_exact_serialized_name_and_unquoted_cursor() {
        let (slot, cursor) = parse("sidebar.team.dark_blue trailing").unwrap();

        assert_eq!(slot, DisplaySlotModel::TeamDarkBlue);
        assert_eq!(cursor, "sidebar.team.dark_blue".len());
    }

    #[test]
    fn java_parse_rejects_unknown_names_without_cursor_reset() {
        let mut reader = StringReaderModel::new("foo.bar trailing");
        let error = ScoreboardSlotArgumentModel::display_slot().parse(&mut reader);

        assert_eq!(
            error,
            Err(ScoreboardSlotParseError::InvalidValue {
                value: "foo.bar".to_string()
            })
        );
        assert_eq!(reader.cursor(), "foo.bar".len());
    }

    #[test]
    fn java_parse_is_case_sensitive() {
        let mut reader = StringReaderModel::new("Sidebar");
        let error = ScoreboardSlotArgumentModel::display_slot().parse(&mut reader);

        assert_eq!(
            error,
            Err(ScoreboardSlotParseError::InvalidValue {
                value: "Sidebar".to_string()
            })
        );
        assert_eq!(reader.cursor(), "Sidebar".len());
    }

    #[test]
    fn java_suggestions_use_all_display_slot_serialized_names() {
        let mut builder = SuggestionsBuilderModel::new("dark");
        let suggestions =
            ScoreboardSlotArgumentModel::display_slot().list_suggestions(&mut builder);

        assert_eq!(
            suggestion_values(suggestions),
            vec![
                "sidebar.team.dark_blue".to_string(),
                "sidebar.team.dark_green".to_string(),
                "sidebar.team.dark_aqua".to_string(),
                "sidebar.team.dark_red".to_string(),
                "sidebar.team.dark_purple".to_string(),
                "sidebar.team.dark_gray".to_string(),
            ]
        );
    }

    #[test]
    fn java_by_id_uses_zero_out_of_bounds_strategy() {
        assert_eq!(DisplaySlotModel::by_id(0), DisplaySlotModel::List);
        assert_eq!(DisplaySlotModel::by_id(18), DisplaySlotModel::TeamWhite);
        assert_eq!(DisplaySlotModel::by_id(-1), DisplaySlotModel::List);
        assert_eq!(DisplaySlotModel::by_id(19), DisplaySlotModel::List);
    }

    #[test]
    fn java_team_color_to_slot_maps_only_real_colors() {
        let colors = [
            (ChatFormattingModel::Black, DisplaySlotModel::TeamBlack),
            (
                ChatFormattingModel::DarkBlue,
                DisplaySlotModel::TeamDarkBlue,
            ),
            (
                ChatFormattingModel::DarkGreen,
                DisplaySlotModel::TeamDarkGreen,
            ),
            (
                ChatFormattingModel::DarkAqua,
                DisplaySlotModel::TeamDarkAqua,
            ),
            (ChatFormattingModel::DarkRed, DisplaySlotModel::TeamDarkRed),
            (
                ChatFormattingModel::DarkPurple,
                DisplaySlotModel::TeamDarkPurple,
            ),
            (ChatFormattingModel::Gold, DisplaySlotModel::TeamGold),
            (ChatFormattingModel::Gray, DisplaySlotModel::TeamGray),
            (
                ChatFormattingModel::DarkGray,
                DisplaySlotModel::TeamDarkGray,
            ),
            (ChatFormattingModel::Blue, DisplaySlotModel::TeamBlue),
            (ChatFormattingModel::Green, DisplaySlotModel::TeamGreen),
            (ChatFormattingModel::Aqua, DisplaySlotModel::TeamAqua),
            (ChatFormattingModel::Red, DisplaySlotModel::TeamRed),
            (
                ChatFormattingModel::LightPurple,
                DisplaySlotModel::TeamLightPurple,
            ),
            (ChatFormattingModel::Yellow, DisplaySlotModel::TeamYellow),
            (ChatFormattingModel::White, DisplaySlotModel::TeamWhite),
        ];
        for (color, slot) in colors {
            assert_eq!(DisplaySlotModel::team_color_to_slot(color), Some(slot));
        }

        for format in [
            ChatFormattingModel::Bold,
            ChatFormattingModel::Italic,
            ChatFormattingModel::Underline,
            ChatFormattingModel::Reset,
            ChatFormattingModel::Obfuscated,
            ChatFormattingModel::Strikethrough,
        ] {
            assert_eq!(DisplaySlotModel::team_color_to_slot(format), None);
        }
    }
}
