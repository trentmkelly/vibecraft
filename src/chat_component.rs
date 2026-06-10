#![allow(dead_code)]

#[path = "chat_package_info.rs"]
pub mod chat_package_info;
#[path = "chat_type.rs"]
pub mod chat_type;
#[path = "common_components.rs"]
pub mod common_components;
#[path = "component_contents.rs"]
pub mod component_contents;
#[path = "filter_mask.rs"]
pub mod filter_mask;
#[path = "formatted_text.rs"]
pub mod formatted_text;
#[path = "hover_event.rs"]
pub mod hover_event;
#[path = "number_format.rs"]
pub mod number_format;
#[path = "plain_text_contents.rs"]
pub mod plain_text_contents;
#[path = "resolution_context.rs"]
pub mod resolution_context;
#[path = "sub_string_source.rs"]
pub mod sub_string_source;
#[path = "throwing_component.rs"]
pub mod throwing_component;
#[path = "translatable_format_exception.rs"]
pub mod translatable_format_exception;

pub use resolution_context::ResolutionContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub content: ComponentContent,
    pub style: Style,
    pub siblings: Vec<Component>,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Literal(text.into()),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::literal("")
    }

    pub fn translatable(key: impl Into<String>, args: Vec<ComponentArgument>) -> Self {
        Self {
            content: ComponentContent::Translatable {
                key: key.into(),
                fallback: None,
                args,
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        if let ComponentContent::Translatable { fallback: slot, .. } = &mut self.content {
            *slot = Some(fallback.into());
        }
        self
    }

    pub fn styled(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn append(mut self, sibling: Component) -> Self {
        self.siblings.push(sibling);
        self
    }

    pub fn render_plain(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> String {
        let mut output = self.content.render_plain(translations, context);
        for sibling in &self.siblings {
            output.push_str(&sibling.render_plain(translations, context));
        }
        output
    }

    pub fn get_string(&self) -> String {
        self.render_plain(&TranslationTable::default(), &ResolutionContext::default())
    }

    pub fn to_json(&self) -> String {
        let mut fields = self.content.json_fields();
        fields.extend(self.style.json_fields());
        if !self.siblings.is_empty() {
            fields.push(format!(
                "\"extra\":{}",
                json_array(self.siblings.iter().map(Component::to_json).collect())
            ));
        }
        format!("{{{}}}", fields.join(","))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentContent {
    Literal(String),
    Translatable {
        key: String,
        fallback: Option<String>,
        args: Vec<ComponentArgument>,
    },
    Selector {
        selector: String,
        separator: Option<Box<Component>>,
    },
    Score {
        name: String,
        objective: String,
    },
    Keybind(String),
    Nbt {
        path: String,
        source: NbtSource,
        interpret: bool,
        plain: bool,
        separator: Option<Box<Component>>,
    },
    Object(ObjectContent),
}

impl ComponentContent {
    fn render_plain(&self, translations: &TranslationTable, context: &ResolutionContext) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Translatable {
                key,
                fallback,
                args,
            } => render_translation(
                translations
                    .lookup(key)
                    .or(fallback.as_deref())
                    .unwrap_or(key),
                args,
                translations,
                context,
            ),
            Self::Selector {
                selector,
                separator,
            } => context
                .selector(selector)
                .map(|values| {
                    let sep = separator
                        .as_ref()
                        .map(|component| component.render_plain(translations, context))
                        .unwrap_or_else(|| ", ".to_string());
                    values.join(&sep)
                })
                .unwrap_or_else(|| selector.clone()),
            Self::Score { name, objective } => context
                .score(name, objective)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            Self::Keybind(key) => context
                .keybind(key)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| key.clone()),
            Self::Nbt {
                path,
                source,
                separator,
                ..
            } => {
                let sep = separator
                    .as_ref()
                    .map(|component| component.render_plain(translations, context))
                    .unwrap_or_else(|| ", ".to_string());
                context.nbt(source, path).join(&sep)
            }
            Self::Object(object) => object.render_plain(context),
        }
    }

    fn json_fields(&self) -> Vec<String> {
        match self {
            Self::Literal(text) => vec![format!("\"text\":{}", json_string(text))],
            Self::Translatable {
                key,
                fallback,
                args,
            } => {
                let mut fields = vec![format!("\"translate\":{}", json_string(key))];
                if let Some(fallback) = fallback {
                    fields.push(format!("\"fallback\":{}", json_string(fallback)));
                }
                if !args.is_empty() {
                    fields.push(format!(
                        "\"with\":{}",
                        json_array(args.iter().map(ComponentArgument::to_json).collect())
                    ));
                }
                fields
            }
            Self::Selector {
                selector,
                separator,
            } => {
                let mut fields = vec![format!("\"selector\":{}", json_string(selector))];
                if let Some(separator) = separator {
                    fields.push(format!("\"separator\":{}", separator.to_json()));
                }
                fields
            }
            Self::Score { name, objective } => vec![format!(
                "\"score\":{{\"name\":{},\"objective\":{}}}",
                json_string(name),
                json_string(objective)
            )],
            Self::Keybind(key) => vec![format!("\"keybind\":{}", json_string(key))],
            Self::Nbt {
                path,
                source,
                interpret,
                plain,
                separator,
            } => {
                let mut fields = vec![
                    format!("\"nbt\":{}", json_string(path)),
                    source.json_field(),
                    format!("\"interpret\":{}", interpret),
                    format!("\"plain\":{}", plain),
                ];
                if let Some(separator) = separator {
                    fields.push(format!("\"separator\":{}", separator.to_json()));
                }
                fields
            }
            Self::Object(object) => object.json_fields(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentArgument {
    Component(Box<Component>),
    String(String),
    Number(i32),
    Long(i64),
    Boolean(bool),
    Null,
}

impl ComponentArgument {
    pub fn render_plain(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> String {
        match self {
            Self::Component(component) => component.render_plain(translations, context),
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::Long(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Null => "null".to_string(),
        }
    }

    fn to_json(&self) -> String {
        match self {
            Self::Component(component) => component.to_json(),
            Self::String(value) => json_string(value),
            Self::Number(value) => value.to_string(),
            Self::Long(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Null => "null".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtSource {
    Block(String),
    Entity(String),
    Storage(String),
}

impl NbtSource {
    fn json_field(&self) -> String {
        match self {
            Self::Block(value) => format!("\"block\":{}", json_string(value)),
            Self::Entity(value) => format!("\"entity\":{}", json_string(value)),
            Self::Storage(value) => format!("\"storage\":{}", json_string(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectContent {
    AtlasSprite { atlas: String, sprite: String },
    PlayerSprite { profile: String, hat: bool },
}

impl ObjectContent {
    fn render_plain(&self, context: &ResolutionContext) -> String {
        match self {
            Self::AtlasSprite { sprite, .. } => sprite.clone(),
            Self::PlayerSprite { profile, hat } => context
                .player_sprite(profile, *hat)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| profile.clone()),
        }
    }

    fn json_fields(&self) -> Vec<String> {
        match self {
            Self::AtlasSprite { atlas, sprite } => vec![format!(
                "\"object\":{{\"type\":\"atlas_sprite\",\"atlas\":{},\"sprite\":{}}}",
                json_string(atlas),
                json_string(sprite)
            )],
            Self::PlayerSprite { profile, hat } => vec![format!(
                "\"object\":{{\"type\":\"player_sprite\",\"profile\":{},\"hat\":{}}}",
                json_string(profile),
                hat
            )],
        }
    }
}

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
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: TextColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    pub fn with_click_event(mut self, click_event: ClickEvent) -> Self {
        self.click_event = Some(click_event);
        self
    }

    pub fn with_hover_event(mut self, hover_event: HoverEvent) -> Self {
        self.hover_event = Some(hover_event);
        self
    }

    pub fn with_font(mut self, font: FontDescription) -> Self {
        self.font = Some(font);
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

    fn json_fields(&self) -> Vec<String> {
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
            fields.push(format!("\"clickEvent\":{}", click_event.to_json()));
        }
        if let Some(hover_event) = &self.hover_event {
            fields.push(format!("\"hoverEvent\":{}", hover_event.to_json()));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickEvent {
    OpenUrl(String),
    OpenFile(String),
    RunCommand(String),
    SuggestCommand(String),
    ShowDialog(String),
    ChangePage(u32),
    CopyToClipboard(String),
    Custom { id: String, payload: Option<String> },
}

impl ClickEvent {
    pub fn action(&self) -> &'static str {
        match self {
            Self::OpenUrl(_) => "open_url",
            Self::OpenFile(_) => "open_file",
            Self::RunCommand(_) => "run_command",
            Self::SuggestCommand(_) => "suggest_command",
            Self::ShowDialog(_) => "show_dialog",
            Self::ChangePage(_) => "change_page",
            Self::CopyToClipboard(_) => "copy_to_clipboard",
            Self::Custom { .. } => "custom",
        }
    }

    pub fn allowed_from_server(&self) -> bool {
        !matches!(self, Self::OpenFile(_))
    }

    fn to_json(&self) -> String {
        let mut fields = vec![format!("\"action\":{}", json_string(self.action()))];
        match self {
            Self::OpenUrl(url) => fields.push(format!("\"url\":{}", json_string(url))),
            Self::OpenFile(path) => fields.push(format!("\"path\":{}", json_string(path))),
            Self::RunCommand(command) => {
                fields.push(format!("\"command\":{}", json_string(command)))
            }
            Self::SuggestCommand(command) => {
                fields.push(format!("\"command\":{}", json_string(command)))
            }
            Self::ShowDialog(dialog) => fields.push(format!("\"dialog\":{}", json_string(dialog))),
            Self::ChangePage(page) => fields.push(format!("\"page\":{}", page)),
            Self::CopyToClipboard(value) => {
                fields.push(format!("\"value\":{}", json_string(value)))
            }
            Self::Custom { id, payload } => {
                fields.push(format!("\"id\":{}", json_string(id)));
                if let Some(payload) = payload {
                    fields.push(format!("\"payload\":{}", json_string(payload)));
                }
            }
        }
        format!("{{{}}}", fields.join(","))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverEvent {
    Text(Box<Component>),
    Item {
        item: String,
        count: u32,
        components: Option<String>,
    },
    Entity {
        entity_type: String,
        uuid: String,
        name: Option<Box<Component>>,
    },
}

impl HoverEvent {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Text(_) => "show_text",
            Self::Item { .. } => "show_item",
            Self::Entity { .. } => "show_entity",
        }
    }

    fn to_json(&self) -> String {
        match self {
            Self::Text(value) => {
                format!("{{\"action\":\"show_text\",\"value\":{}}}", value.to_json())
            }
            Self::Item {
                item,
                count,
                components,
            } => {
                let mut fields = vec![
                    "\"action\":\"show_item\"".to_string(),
                    format!("\"id\":{}", json_string(item)),
                ];
                if *count != 1 {
                    fields.push(format!("\"count\":{}", count));
                }
                if let Some(components) = components {
                    fields.push(format!("\"components\":{}", components));
                }
                format!("{{{}}}", fields.join(","))
            }
            Self::Entity {
                entity_type,
                uuid,
                name,
            } => {
                let mut fields = vec![
                    "\"action\":\"show_entity\"".to_string(),
                    format!("\"id\":{}", json_string(entity_type)),
                    format!("\"uuid\":{}", json_string(uuid)),
                ];
                if let Some(name) = name {
                    fields.push(format!("\"name\":{}", name.to_json()));
                }
                format!("{{{}}}", fields.join(","))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontDescription {
    Resource(String),
    AtlasSprite { atlas: String, sprite: String },
    PlayerSprite { profile: String, hat: bool },
}

impl FontDescription {
    pub fn default_resource() -> Self {
        Self::Resource("minecraft:default".to_string())
    }

    pub fn codec_identifier(&self) -> Result<&str, String> {
        match self {
            Self::Resource(id) => Ok(id),
            Self::AtlasSprite { .. } | Self::PlayerSprite { .. } => {
                Err(format!("Unsupported font description type: {self:?}"))
            }
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            Self::Resource(id) => id.clone(),
            Self::AtlasSprite { atlas, sprite } => format!("{atlas}/{sprite}"),
            Self::PlayerSprite { profile, hat } => format!("{profile}:hat={hat}"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranslationTable {
    entries: Vec<(String, String)>,
}

impl TranslationTable {
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(entry, _)| entry == key)
            .map(|(_, value)| value.as_str())
    }
}

fn render_translation(
    template: &str,
    args: &[ComponentArgument],
    translations: &TranslationTable,
    context: &ResolutionContext,
) -> String {
    let mut result = String::new();
    let chars: Vec<char> = template.chars().collect();
    let mut cursor = 0;
    let mut next_arg = 0;
    while cursor < chars.len() {
        if chars[cursor] != '%' {
            result.push(chars[cursor]);
            cursor += 1;
            continue;
        }
        if cursor + 1 < chars.len() && chars[cursor + 1] == '%' {
            result.push('%');
            cursor += 2;
            continue;
        }
        let mut scan = cursor + 1;
        let mut explicit = String::new();
        while scan < chars.len() && chars[scan].is_ascii_digit() {
            explicit.push(chars[scan]);
            scan += 1;
        }
        let index = if !explicit.is_empty() && scan < chars.len() && chars[scan] == '$' {
            scan += 1;
            explicit
                .parse::<usize>()
                .ok()
                .and_then(|value| value.checked_sub(1))
        } else {
            scan = cursor + 1;
            let value = Some(next_arg);
            next_arg += 1;
            value
        };
        if scan < chars.len() && chars[scan] == 's' {
            if let Some(arg) = index.and_then(|index| args.get(index)) {
                result.push_str(&arg.render_plain(translations, context));
            }
            cursor = scan + 1;
        } else {
            result.push('%');
            cursor += 1;
        }
    }
    result
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

fn json_array(items: Vec<String>) -> String {
    format!("[{}]", items.join(","))
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_translatable_and_extra_components_render_like_vanilla_format_strings() {
        let translations = TranslationTable::default()
            .with("chat.type.text", "<%s> %s")
            .with("commands.op.success", "Made %s a server operator");
        let context = ResolutionContext::default();
        let component = Component::translatable(
            "chat.type.text",
            vec![
                ComponentArgument::Component(Box::new(Component::literal("Steve"))),
                ComponentArgument::Component(Box::new(
                    Component::literal("hello").append(Component::literal(" world")),
                )),
            ],
        );
        assert_eq!(
            component.render_plain(&translations, &context),
            "<Steve> hello world"
        );

        let percent = Component::translatable(
            "commands.op.success",
            vec![ComponentArgument::String("Alex".to_string())],
        )
        .with_fallback("Operator: %1$s %%");
        assert_eq!(
            percent.render_plain(&TranslationTable::default(), &context),
            "Operator: Alex %"
        );
    }

    #[test]
    fn selector_score_keybind_nbt_and_object_contents_resolve_from_context() {
        let context = ResolutionContext::default()
            .with_selector("@a", vec!["Steve", "Alex"])
            .with_score("Steve", "kills", 7)
            .with_keybind("key.jump", "Space")
            .with_nbt(
                NbtSource::Block("0 64 0".to_string()),
                "Items[0].id",
                vec!["minecraft:stone", "minecraft:dirt"],
            )
            .with_player_sprite("Steve", true, "steve-hat-sprite");
        let translations = TranslationTable::default();
        assert_eq!(
            Component {
                content: ComponentContent::Selector {
                    selector: "@a".to_string(),
                    separator: Some(Box::new(Component::literal(" | "))),
                },
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .render_plain(&translations, &context),
            "Steve | Alex"
        );
        assert_eq!(
            Component {
                content: ComponentContent::Score {
                    name: "Steve".to_string(),
                    objective: "kills".to_string(),
                },
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .render_plain(&translations, &context),
            "7"
        );
        assert_eq!(
            Component {
                content: ComponentContent::Keybind("key.jump".to_string()),
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .render_plain(&translations, &context),
            "Space"
        );
        assert_eq!(
            Component {
                content: ComponentContent::Nbt {
                    path: "Items[0].id".to_string(),
                    source: NbtSource::Block("0 64 0".to_string()),
                    interpret: false,
                    plain: true,
                    separator: None,
                },
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .render_plain(&translations, &context),
            "minecraft:stone, minecraft:dirt"
        );
        assert_eq!(
            Component {
                content: ComponentContent::Object(ObjectContent::PlayerSprite {
                    profile: "Steve".to_string(),
                    hat: true,
                }),
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .render_plain(&translations, &context),
            "steve-hat-sprite"
        );
    }

    #[test]
    fn text_colors_parse_legacy_names_and_hex_values() {
        const TEXT_COLOR_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/TextColor.java"
        );

        for sentinel in [
            "private static final String CUSTOM_COLOR_PREFIX = \"#\";",
            "Codec.STRING.comapFlatMap(TextColor::parseColor, TextColor::serialize)",
            "this.value = value & 16777215;",
            "return this.name != null ? this.name : this.formatValue();",
            "String.format(Locale.ROOT, \"#%06X\", this.value)",
            "return this.value == other.value;",
            "public static @Nullable TextColor fromLegacyFormat",
            "public static TextColor fromRgb(final int rgb)",
            "if (color.startsWith(\"#\"))",
            "Integer.parseInt(color.substring(1), 16)",
            "value >= 0 && value <= 16777215",
            "NAMED_COLORS.get(color)",
        ] {
            assert!(
                TEXT_COLOR_JAVA.contains(sentinel),
                "missing TextColor sentinel {sentinel}"
            );
        }

        assert_eq!(TextColor::parse("red").unwrap().serialize(), "red");
        assert_eq!(TextColor::parse("#00aB09").unwrap().serialize(), "#00AB09");
        assert!(TextColor::parse("#1000000").is_none());
        assert!(TextColor::parse("not_a_color").is_none());
        assert_eq!(TextColor::from_rgb(0x1FF_0000).value(), 0xFF_0000);
        assert_eq!(
            TextColor::parse("red").unwrap(),
            TextColor::from_rgb(0xFF_5555)
        );
        assert_eq!(TextColor::from_rgb(0x00_AB09).serialize(), "#00AB09");
    }

    #[test]
    fn style_serializes_color_flags_click_hover_insertion_shadow_and_font() {
        let mut style = Style::empty()
            .with_color(TextColor::parse("gold").unwrap())
            .with_bold(true)
            .with_click_event(ClickEvent::RunCommand("/help".to_string()))
            .with_hover_event(HoverEvent::Text(Box::new(Component::literal("Help"))))
            .with_font(FontDescription::Resource("minecraft:uniform".to_string()));
        style.italic = Some(false);
        style.underlined = Some(true);
        style.strikethrough = Some(false);
        style.obfuscated = Some(false);
        style.shadow_color = Some(0);
        style.insertion = Some("/help".to_string());

        let json = Component::literal("click").styled(style).to_json();
        assert!(json.contains("\"color\":\"gold\""));
        assert!(json.contains("\"bold\":true"));
        assert!(json.contains("\"italic\":false"));
        assert!(json.contains("\"shadow_color\":0"));
        assert!(json.contains("\"clickEvent\":{\"action\":\"run_command\",\"command\":\"/help\"}"));
        assert!(json
            .contains("\"hoverEvent\":{\"action\":\"show_text\",\"value\":{\"text\":\"Help\"}}"));
        assert!(json.contains("\"font\":\"minecraft:uniform\""));
    }

    #[test]
    fn click_event_actions_match_vanilla_server_safety_flags() {
        const CLICK_EVENT_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/ClickEvent.java"
        );

        for sentinel in [
            "OPEN_URL(\"open_url\", true, ClickEvent.OpenUrl.CODEC)",
            "OPEN_FILE(\"open_file\", false, ClickEvent.OpenFile.CODEC)",
            "RUN_COMMAND(\"run_command\", true, ClickEvent.RunCommand.CODEC)",
            "SUGGEST_COMMAND(\"suggest_command\", true, ClickEvent.SuggestCommand.CODEC)",
            "SHOW_DIALOG(\"show_dialog\", true, ClickEvent.ShowDialog.CODEC)",
            "CHANGE_PAGE(\"change_page\", true, ClickEvent.ChangePage.CODEC)",
            "COPY_TO_CLIPBOARD(\"copy_to_clipboard\", true, ClickEvent.CopyToClipboard.CODEC)",
            "CUSTOM(\"custom\", true, ClickEvent.Custom.CODEC)",
            "return !action.isAllowedFromServer()",
            "ExtraCodecs.POSITIVE_INT.fieldOf(\"page\")",
            "Codec.STRING.fieldOf(\"value\")",
            "Identifier.CODEC.fieldOf(\"id\")",
            "ExtraCodecs.NBT.optionalFieldOf(\"payload\")",
            "Codec.STRING.fieldOf(\"path\")",
            "ExtraCodecs.UNTRUSTED_URI.fieldOf(\"url\")",
            "ExtraCodecs.CHAT_STRING.fieldOf(\"command\")",
            "Dialog.CODEC.fieldOf(\"dialog\")",
        ] {
            assert!(
                CLICK_EVENT_JAVA.contains(sentinel),
                "missing ClickEvent sentinel {sentinel}"
            );
        }

        let events = [
            ClickEvent::OpenUrl("https://example.com".to_string()),
            ClickEvent::OpenFile("/tmp/server.log".to_string()),
            ClickEvent::RunCommand("/say hi".to_string()),
            ClickEvent::SuggestCommand("/help".to_string()),
            ClickEvent::ShowDialog("minecraft:confirm".to_string()),
            ClickEvent::ChangePage(2),
            ClickEvent::CopyToClipboard("seed".to_string()),
            ClickEvent::Custom {
                id: "minecraft:test".to_string(),
                payload: Some("{x:1}".to_string()),
            },
        ];
        assert_eq!(
            events.iter().map(ClickEvent::action).collect::<Vec<_>>(),
            vec![
                "open_url",
                "open_file",
                "run_command",
                "suggest_command",
                "show_dialog",
                "change_page",
                "copy_to_clipboard",
                "custom"
            ]
        );
        assert!(!events[1].allowed_from_server());
        assert!(events
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 1)
            .all(|(_, event)| event.allowed_from_server()));
    }

    #[test]
    fn hover_event_shapes_cover_text_item_and_entity_payloads() {
        let text = HoverEvent::Text(Box::new(Component::literal("hello")));
        let item = HoverEvent::Item {
            item: "minecraft:diamond".to_string(),
            count: 3,
            components: Some("{\"minecraft:custom_name\":{\"text\":\"Gem\"}}".to_string()),
        };
        let entity = HoverEvent::Entity {
            entity_type: "minecraft:zombie".to_string(),
            uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            name: Some(Box::new(Component::literal("Zombie"))),
        };
        assert!(text.to_json().contains("\"action\":\"show_text\""));
        assert!(item.to_json().contains("\"action\":\"show_item\""));
        assert!(entity.to_json().contains("\"action\":\"show_entity\""));
    }

    #[test]
    fn font_description_codec_matches_java_resource_only_contract() {
        const FONT_DESCRIPTION_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/FontDescription.java"
        );

        for sentinel in [
            "Identifier.CODEC",
            "flatComapMap(",
            "FontDescription.Resource::new",
            "fontDescription instanceof FontDescription.Resource resource",
            "DataResult.error(() -> \"Unsupported font description type: \" + fontDescription)",
            "FontDescription.Resource DEFAULT = new FontDescription.Resource(Identifier.withDefaultNamespace(\"default\"));",
            "record AtlasSprite(Identifier atlasId, Identifier spriteId) implements FontDescription",
            "record PlayerSprite(ResolvableProfile profile, boolean hat) implements FontDescription",
            "record Resource(Identifier id) implements FontDescription",
        ] {
            assert!(
                FONT_DESCRIPTION_JAVA.contains(sentinel),
                "missing FontDescription sentinel {sentinel}"
            );
        }

        assert_eq!(
            FontDescription::default_resource(),
            FontDescription::Resource("minecraft:default".to_string())
        );

        let resource = FontDescription::Resource("minecraft:uniform".to_string());
        assert_eq!(resource.codec_identifier(), Ok("minecraft:uniform"));
        assert_eq!(resource.serialize(), "minecraft:uniform");

        let atlas = FontDescription::AtlasSprite {
            atlas: "minecraft:blocks".to_string(),
            sprite: "minecraft:block/stone".to_string(),
        };
        assert!(
            matches!(atlas.codec_identifier(), Err(message) if message.starts_with("Unsupported font description type: "))
        );

        let player = FontDescription::PlayerSprite {
            profile: "Notch".to_string(),
            hat: true,
        };
        assert!(
            matches!(player.codec_identifier(), Err(message) if message.starts_with("Unsupported font description type: "))
        );
    }

    #[test]
    fn component_json_uses_vanilla_field_names_for_each_content_variant() {
        assert_eq!(Component::literal("Hi").to_json(), "{\"text\":\"Hi\"}");
        assert_eq!(
            Component::translatable(
                "chat.type.announcement",
                vec![ComponentArgument::String("Server".to_string())]
            )
            .to_json(),
            "{\"translate\":\"chat.type.announcement\",\"with\":[\"Server\"]}"
        );
        assert_eq!(
            Component {
                content: ComponentContent::Score {
                    name: "*".to_string(),
                    objective: "kills".to_string(),
                },
                style: Style::empty(),
                siblings: Vec::new(),
            }
            .to_json(),
            "{\"score\":{\"name\":\"*\",\"objective\":\"kills\"}}"
        );
        assert!(Component {
            content: ComponentContent::Nbt {
                path: "foo".to_string(),
                source: NbtSource::Storage("minecraft:test".to_string()),
                interpret: true,
                plain: false,
                separator: Some(Box::new(Component::literal("; "))),
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
        .to_json()
        .contains("\"storage\":\"minecraft:test\""));
    }
}
