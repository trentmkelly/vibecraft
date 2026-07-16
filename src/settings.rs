//! Java `Settings<T>` property loading, storage, and mutable-value semantics.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsModel {
    properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableValue<T> {
    key: String,
    value: T,
}

impl SettingsModel {
    pub fn new(properties: BTreeMap<String, String>) -> Self {
        Self { properties }
    }

    pub fn load_from_file(path: &Path) -> Self {
        let properties = fs::read(path)
            .ok()
            .map(|bytes| decode_java_properties(&bytes))
            .map(|text| parse_java_properties(&text))
            .unwrap_or_default();
        Self { properties }
    }

    pub fn store(&self, path: &Path) -> std::io::Result<()> {
        let mut output = String::from("#Minecraft server properties\n#");
        output.push_str(&Local::now().format("%a %b %d %H:%M:%S %Z %Y").to_string());
        output.push('\n');
        for (key, value) in &self.properties {
            output.push_str(&escape_property_key(key));
            output.push('=');
            output.push_str(&escape_property_value(value));
            output.push('\n');
        }
        fs::write(path, output)
    }

    pub fn properties(&self) -> &BTreeMap<String, String> {
        &self.properties
    }

    pub fn clone_properties(&self) -> BTreeMap<String, String> {
        self.properties.clone()
    }

    pub fn get<T, D, S>(&mut self, key: &str, deserialize: D, serialize: S, default: T) -> T
    where
        T: Clone,
        D: Fn(&str) -> Option<T>,
        S: Fn(&T) -> String,
    {
        let value = self
            .properties
            .get(key)
            .and_then(|value| deserialize(value))
            .unwrap_or(default);
        self.properties.insert(key.to_string(), serialize(&value));
        value
    }

    pub fn get_legacy<T, D>(&mut self, key: &str, deserialize: D) -> Option<T>
    where
        D: Fn(&str) -> T,
    {
        self.properties.remove(key).map(|value| deserialize(&value))
    }

    pub fn get_mutable<T, D, S>(
        &mut self,
        key: &str,
        deserialize: D,
        serialize: S,
        default: T,
    ) -> MutableValue<T>
    where
        T: Clone,
        D: Fn(&str) -> Option<T>,
        S: Fn(&T) -> String,
    {
        let value = self
            .properties
            .get(key)
            .and_then(|value| deserialize(value))
            .unwrap_or(default);
        self.properties.insert(key.to_string(), serialize(&value));
        MutableValue {
            key: key.to_string(),
            value,
        }
    }

    pub fn update_mutable<T, S>(
        &self,
        mutable: &MutableValue<T>,
        value: T,
        serialize: S,
    ) -> Self
    where
        T: Clone,
        S: Fn(&T) -> String,
    {
        let mut properties = self.clone_properties();
        properties.insert(mutable.key.clone(), serialize(&value));
        Self { properties }
    }

    pub fn dispatch_number_or_string<T, I, S>(
        value: &str,
        integer: I,
        string: S,
    ) -> Option<T>
    where
        I: Fn(i32) -> Option<T>,
        S: Fn(&str) -> Option<T>,
    {
        value
            .parse::<i32>()
            .ok()
            .and_then(integer)
            .or_else(|| string(value))
    }
}

impl<T> MutableValue<T> {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

pub fn load_properties(path: &Path) -> BTreeMap<String, String> {
    SettingsModel::load_from_file(path).properties
}

pub fn parse_java_properties(text: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for line in logical_lines(text) {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let (key, value) = split_property_line(line);
        if !key.is_empty() {
            result.insert(unescape_property(key), unescape_property(value));
        }
    }
    result
}

pub fn store_properties(
    path: &Path,
    properties: &BTreeMap<String, String>,
    comment: &str,
) -> std::io::Result<()> {
    let settings = SettingsModel::new(properties.clone());
    let mut output = String::new();
    output.push('#');
    output.push_str(comment);
    output.push('\n');
    output.push('#');
    output.push_str(&Local::now().format("%a %b %d %H:%M:%S %Z %Y").to_string());
    output.push('\n');
    for (key, value) in settings.properties().iter() {
        output.push_str(&escape_property_key(key));
        output.push('=');
        output.push_str(&escape_property_value(value));
        output.push('\n');
    }
    fs::write(path, output)
}

fn decode_java_properties(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| {
        bytes
            .iter()
            .map(|byte| char::from(*byte))
            .collect::<String>()
    })
}

fn logical_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let raw = if current.is_empty() { raw } else { raw.trim_start() };
        let continuation = trailing_backslash_count(raw) % 2 == 1;
        let part = if continuation {
            &raw[..raw.len().saturating_sub(1)]
        } else {
            raw
        };
        current.push_str(part);
        if continuation {
            continue;
        }
        lines.push(std::mem::take(&mut current));
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn trailing_backslash_count(value: &str) -> usize {
    value.bytes().rev().take_while(|byte| *byte == b'\\').count()
}

fn split_property_line(line: &str) -> (&str, &str) {
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
        } else if character == '=' || character == ':' || character.is_ascii_whitespace() {
            let mut value_start = index + character.len_utf8();
            while value_start < line.len() {
                let next = line[value_start..].chars().next();
                if !matches!(next, Some(' ' | '\t' | '\x0c' | '=' | ':')) {
                    break;
                }
                value_start += next.map_or(0, char::len_utf8);
            }
            return (&line[..index], &line[value_start..]);
        }
    }
    (line, "")
}

fn unescape_property(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('f') => output.push('\x0c'),
            Some('u') => {
                let digits: String = chars.by_ref().take(4).collect();
                if digits.len() == 4 {
                    if let Ok(codepoint) = u16::from_str_radix(&digits, 16) {
                        if let Some(decoded) = char::from_u32(u32::from(codepoint)) {
                            output.push(decoded);
                            continue;
                        }
                    }
                }
                output.push('u');
                output.push_str(&digits);
            }
            Some(next) => output.push(next),
            None => output.push('\\'),
        }
    }
    output
}

fn escape_property_key(value: &str) -> String {
    escape_property(value, true)
}

fn escape_property_value(value: &str) -> String {
    escape_property(value, false)
}

fn escape_property(value: &str, key: bool) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\x0c' => output.push_str("\\f"),
            ' ' if key || index == 0 => output.push_str("\\ "),
            ':' | '=' if key => {
                output.push('\\');
                output.push(character);
            }
            '#' | '!' if key && index == 0 => {
                output.push('\\');
                output.push(character);
            }
            character if character.is_control() => output.push_str(&format!("\\u{:04x}", character as u32)),
            _ => output.push(character),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/dedicated/Settings.java");

    fn path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("vibecraft-settings-{name}-{}.properties", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_matches_settings_loading_storage_and_mutable_surface() {
        for fragment in [
            "public static Properties loadFromFile(final Path file)",
            "trying ISO_8859_1",
            "public void store(final Path output)",
            "protected <V> @Nullable V getLegacy(final String key",
            "protected <V> V get(",
            "protected <V> Settings<T>.MutableValue<V> getMutable",
            "protected Properties cloneProperties()",
            "public T update(final RegistryAccess registryAccess, final V value)",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn java_properties_parser_handles_comments_separators_escapes_and_continuations() {
        let parsed = parse_java_properties(
            "# comment\n! ignored\nkey:value\nother value\ncontinued=hello\\\n  world\nescaped\\ key=one\\:two\\=three\\u0021\n",
        );
        assert_eq!(parsed["key"], "value");
        assert_eq!(parsed["other"], "value");
        assert_eq!(parsed["continued"], "helloworld");
        assert_eq!(parsed["escaped key"], "one:two=three!");
    }

    #[test]
    fn settings_get_legacy_default_and_mutable_update_match_java() {
        let mut settings = SettingsModel::new(BTreeMap::new());
        let legacy = settings.get_legacy("old", |value| value == "true");
        assert_eq!(legacy, None);
        let value = settings.get("count", |value| value.parse::<i32>().ok(), |value| value.to_string(), 4);
        assert_eq!(value, 4);
        let mutable = settings.get_mutable(
            "motd",
            |value: &str| Some(value.to_string()),
            |value: &String| value.clone(),
            "Default".to_string(),
        );
        assert_eq!(mutable.get(), "Default");
        let updated = settings.update_mutable(&mutable, "Changed".to_string(), |value: &String| value.clone());
        assert_eq!(updated.properties()["motd"], "Changed");
        assert_eq!(SettingsModel::dispatch_number_or_string("2", Some, |_| None), Some(2));
        assert_eq!(SettingsModel::dispatch_number_or_string("hard", |_| None, |value| Some(value.to_string())), Some("hard".to_string()));
    }

    #[test]
    fn settings_load_falls_back_to_iso_8859_1_and_stores_vanilla_header() {
        let source = path("encoding");
        fs::write(&source, [b'm', b'o', b't', b'd', b'=', 0xe9]).expect("write Latin-1 fixture");
        let settings = SettingsModel::load_from_file(&source);
        assert_eq!(settings.properties()["motd"], "é");
        settings.store(&source).expect("store properties");
        let saved = fs::read_to_string(&source).expect("read stored properties");
        assert!(saved.starts_with("#Minecraft server properties\n#"));
        assert!(saved.contains("motd=é\n"));
        let _ignored = fs::remove_file(source);
    }
}
