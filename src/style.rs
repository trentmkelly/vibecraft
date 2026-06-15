use super::{json_string, ClickEvent, FontDescription, HoverEvent};
use crate::chat_formatting::ChatFormatting;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Style {
    pub color: Option<TextColor>,
    pub shadow_color: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub insertion: Option<String>,
    pub font: Option<FontDescription>,
}

impl Style {
    pub const MAP_CODEC_FIELDS: [&'static str; 11] = [
        "color",
        "shadow_color",
        "bold",
        "italic",
        "underlined",
        "strikethrough",
        "obfuscated",
        "click_event",
        "hover_event",
        "insertion",
        "font",
    ];

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    pub fn get_color(&self) -> Option<&TextColor> {
        self.color.as_ref()
    }

    pub fn get_shadow_color(&self) -> Option<u32> {
        self.shadow_color
    }

    pub fn is_bold(&self) -> bool {
        self.bold == Some(true)
    }

    pub fn is_italic(&self) -> bool {
        self.italic == Some(true)
    }

    pub fn is_underlined(&self) -> bool {
        self.underlined == Some(true)
    }

    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough == Some(true)
    }

    pub fn is_obfuscated(&self) -> bool {
        self.obfuscated == Some(true)
    }

    pub fn get_click_event(&self) -> Option<&ClickEvent> {
        self.click_event.as_ref()
    }

    pub fn get_hover_event(&self) -> Option<&HoverEvent> {
        self.hover_event.as_ref()
    }

    pub fn get_insertion(&self) -> Option<&str> {
        self.insertion.as_deref()
    }

    pub fn get_font(&self) -> FontDescription {
        self.font
            .clone()
            .unwrap_or_else(FontDescription::default_resource)
    }

    pub fn with_color(mut self, color: TextColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_optional_color(mut self, color: Option<TextColor>) -> Self {
        self.color = color;
        self
    }

    pub fn with_legacy_color(self, color: Option<ChatFormatting>) -> Self {
        self.with_optional_color(color.and_then(TextColor::from_legacy_format))
    }

    pub fn with_rgb_color(self, color: u32) -> Self {
        self.with_color(TextColor::from_rgb(color))
    }

    pub fn with_shadow_color(mut self, shadow_color: u32) -> Self {
        self.shadow_color = Some(shadow_color);
        self
    }

    pub fn with_optional_shadow_color(mut self, shadow_color: Option<u32>) -> Self {
        self.shadow_color = shadow_color;
        self
    }

    pub fn without_shadow(self) -> Self {
        self.with_shadow_color(0)
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    pub fn with_optional_bold(mut self, bold: Option<bool>) -> Self {
        self.bold = bold;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    pub fn with_optional_italic(mut self, italic: Option<bool>) -> Self {
        self.italic = italic;
        self
    }

    pub fn with_underlined(mut self, underlined: bool) -> Self {
        self.underlined = Some(underlined);
        self
    }

    pub fn with_optional_underlined(mut self, underlined: Option<bool>) -> Self {
        self.underlined = underlined;
        self
    }

    pub fn with_strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = Some(strikethrough);
        self
    }

    pub fn with_optional_strikethrough(mut self, strikethrough: Option<bool>) -> Self {
        self.strikethrough = strikethrough;
        self
    }

    pub fn with_obfuscated(mut self, obfuscated: bool) -> Self {
        self.obfuscated = Some(obfuscated);
        self
    }

    pub fn with_optional_obfuscated(mut self, obfuscated: Option<bool>) -> Self {
        self.obfuscated = obfuscated;
        self
    }

    pub fn with_click_event(mut self, click_event: ClickEvent) -> Self {
        self.click_event = Some(click_event);
        self
    }

    pub fn with_optional_click_event(mut self, click_event: Option<ClickEvent>) -> Self {
        self.click_event = click_event;
        self
    }

    pub fn with_hover_event(mut self, hover_event: HoverEvent) -> Self {
        self.hover_event = Some(hover_event);
        self
    }

    pub fn with_optional_hover_event(mut self, hover_event: Option<HoverEvent>) -> Self {
        self.hover_event = hover_event;
        self
    }

    pub fn with_insertion(mut self, insertion: impl Into<String>) -> Self {
        self.insertion = Some(insertion.into());
        self
    }

    pub fn with_optional_insertion(mut self, insertion: Option<String>) -> Self {
        self.insertion = insertion;
        self
    }

    pub fn with_font(mut self, font: FontDescription) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_optional_font(mut self, font: Option<FontDescription>) -> Self {
        self.font = font;
        self
    }

    pub fn apply_format(self, format: ChatFormatting) -> Self {
        match format {
            ChatFormatting::Obfuscated => self.with_obfuscated(true),
            ChatFormatting::Bold => self.with_bold(true),
            ChatFormatting::Strikethrough => self.with_strikethrough(true),
            ChatFormatting::Underline => self.with_underlined(true),
            ChatFormatting::Italic => self.with_italic(true),
            ChatFormatting::Reset => Self::empty(),
            color => self.with_legacy_color(Some(color)),
        }
    }

    pub fn apply_legacy_format(self, format: ChatFormatting) -> Self {
        match format {
            ChatFormatting::Obfuscated => self.with_obfuscated(true),
            ChatFormatting::Bold => self.with_bold(true),
            ChatFormatting::Strikethrough => self.with_strikethrough(true),
            ChatFormatting::Underline => self.with_underlined(true),
            ChatFormatting::Italic => self.with_italic(true),
            ChatFormatting::Reset => Self::empty(),
            color => self
                .with_optional_obfuscated(Some(false))
                .with_optional_bold(Some(false))
                .with_optional_strikethrough(Some(false))
                .with_optional_underlined(Some(false))
                .with_optional_italic(Some(false))
                .with_legacy_color(Some(color)),
        }
    }

    pub fn apply_formats(mut self, formats: impl IntoIterator<Item = ChatFormatting>) -> Self {
        for format in formats {
            if format == ChatFormatting::Reset {
                return Self::empty();
            }
            self = self.apply_format(format);
        }
        self
    }

    pub fn apply_to(&self, other: &Style) -> Style {
        if *self == Style::empty() {
            return other.clone();
        }
        if *other == Style::empty() {
            return self.clone();
        }
        Style {
            color: self.color.clone().or_else(|| other.color.clone()),
            shadow_color: self.shadow_color.or(other.shadow_color),
            bold: self.bold.or(other.bold),
            italic: self.italic.or(other.italic),
            underlined: self.underlined.or(other.underlined),
            strikethrough: self.strikethrough.or(other.strikethrough),
            obfuscated: self.obfuscated.or(other.obfuscated),
            click_event: self
                .click_event
                .clone()
                .or_else(|| other.click_event.clone()),
            hover_event: self
                .hover_event
                .clone()
                .or_else(|| other.hover_event.clone()),
            insertion: self.insertion.clone().or_else(|| other.insertion.clone()),
            font: self.font.clone().or_else(|| other.font.clone()),
        }
    }

    pub fn to_java_string(&self) -> String {
        let mut fields = Vec::new();
        if let Some(color) = &self.color {
            fields.push(format!("color={}", color.serialize()));
        }
        if let Some(color) = self.shadow_color {
            fields.push(format!("shadowColor={color}"));
        }
        push_java_flag(&mut fields, "bold", self.bold);
        push_java_flag(&mut fields, "italic", self.italic);
        push_java_flag(&mut fields, "underlined", self.underlined);
        push_java_flag(&mut fields, "strikethrough", self.strikethrough);
        push_java_flag(&mut fields, "obfuscated", self.obfuscated);
        if let Some(click_event) = &self.click_event {
            fields.push(format!("clickEvent={click_event:?}"));
        }
        if let Some(hover_event) = &self.hover_event {
            fields.push(format!("hoverEvent={hover_event:?}"));
        }
        if let Some(insertion) = &self.insertion {
            fields.push(format!("insertion={insertion}"));
        }
        if let Some(font) = &self.font {
            fields.push(format!("font={}", font.serialize()));
        }
        format!("{{{}}}", fields.join(","))
    }

    pub(super) fn json_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        if let Some(color) = &self.color {
            fields.push(format!("\"color\":{}", json_string(&color.serialize())));
        }
        if let Some(color) = self.shadow_color {
            fields.push(format!("\"shadow_color\":{}", color));
        }
        push_bool(&mut fields, "bold", self.bold);
        push_bool(&mut fields, "italic", self.italic);
        push_bool(&mut fields, "underlined", self.underlined);
        push_bool(&mut fields, "strikethrough", self.strikethrough);
        push_bool(&mut fields, "obfuscated", self.obfuscated);
        if let Some(click_event) = &self.click_event {
            fields.push(format!("\"click_event\":{}", click_event.to_json()));
        }
        if let Some(hover_event) = &self.hover_event {
            fields.push(format!("\"hover_event\":{}", hover_event.to_json()));
        }
        if let Some(insertion) = &self.insertion {
            fields.push(format!("\"insertion\":{}", json_string(insertion)));
        }
        if let Some(font) = &self.font {
            fields.push(format!("\"font\":{}", json_string(&font.serialize())));
        }
        fields
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TextColor {
    value: u32,
    name: Option<&'static str>,
}

impl PartialEq for TextColor {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl TextColor {
    pub fn from_rgb(value: u32) -> Self {
        Self {
            value: value & 0xFF_FFFF,
            name: None,
        }
    }

    pub fn from_legacy_format(format: ChatFormatting) -> Option<Self> {
        let value = format.get_color()?;
        Some(Self {
            value,
            name: Some(match format {
                ChatFormatting::Black => "black",
                ChatFormatting::DarkBlue => "dark_blue",
                ChatFormatting::DarkGreen => "dark_green",
                ChatFormatting::DarkAqua => "dark_aqua",
                ChatFormatting::DarkRed => "dark_red",
                ChatFormatting::DarkPurple => "dark_purple",
                ChatFormatting::Gold => "gold",
                ChatFormatting::Gray => "gray",
                ChatFormatting::DarkGray => "dark_gray",
                ChatFormatting::Blue => "blue",
                ChatFormatting::Green => "green",
                ChatFormatting::Aqua => "aqua",
                ChatFormatting::Red => "red",
                ChatFormatting::LightPurple => "light_purple",
                ChatFormatting::Yellow => "yellow",
                ChatFormatting::White => "white",
                _ => return None,
            }),
        })
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn parse(value: &str) -> Option<Self> {
        if let Some(hex) = value.strip_prefix('#') {
            let parsed = u32::from_str_radix(hex, 16).ok()?;
            if hex.len() == 6 && parsed <= 0xFF_FFFF {
                Some(Self::from_rgb(parsed))
            } else {
                None
            }
        } else {
            legacy_color(value).map(|rgb| Self {
                value: rgb.0,
                name: Some(rgb.1),
            })
        }
    }

    pub fn serialize(&self) -> String {
        self.name
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("#{:06X}", self.value))
    }
}

fn legacy_color(name: &str) -> Option<(u32, &'static str)> {
    match name {
        "black" => Some((0x000000, "black")),
        "dark_blue" => Some((0x0000AA, "dark_blue")),
        "dark_green" => Some((0x00AA00, "dark_green")),
        "dark_aqua" => Some((0x00AAAA, "dark_aqua")),
        "dark_red" => Some((0xAA0000, "dark_red")),
        "dark_purple" => Some((0xAA00AA, "dark_purple")),
        "gold" => Some((0xFFAA00, "gold")),
        "gray" => Some((0xAAAAAA, "gray")),
        "dark_gray" => Some((0x555555, "dark_gray")),
        "blue" => Some((0x5555FF, "blue")),
        "green" => Some((0x55FF55, "green")),
        "aqua" => Some((0x55FFFF, "aqua")),
        "red" => Some((0xFF5555, "red")),
        "light_purple" => Some((0xFF55FF, "light_purple")),
        "yellow" => Some((0xFFFF55, "yellow")),
        "white" => Some((0xFFFFFF, "white")),
        _ => None,
    }
}

fn push_bool(fields: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        fields.push(format!("\"{name}\":{value}"));
    }
}

fn push_java_flag(fields: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        if value {
            fields.push(name.to_string());
        } else {
            fields.push(format!("!{name}"));
        }
    }
}
