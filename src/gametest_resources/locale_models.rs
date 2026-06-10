use std::collections::BTreeMap;

pub const DEFAULT_LANGUAGE: &str = "en_us";

#[cfg(all(test, vibecraft_has_decompiled_sources))]
#[path = "tests_locale.rs"]
mod tests_locale;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeprecatedTranslationsInfoModel {
    pub removed: Vec<String>,
    pub renamed: BTreeMap<String, String>,
}

impl DeprecatedTranslationsInfoModel {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn load_from_json(raw: &str) -> Result<Self, String> {
        let removed = parse_string_array_field(raw, "removed")?;
        let renamed = parse_string_map_field(raw, "renamed")?;
        Ok(Self { removed, renamed })
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    pub fn load_default_resource() -> Result<Self, String> {
        Self::load_from_json(include_str!(
            "../../../decompiled-server-26.1.2/assets/minecraft/lang/deprecated.json"
        ))
    }

    #[cfg(not(vibecraft_has_decompiled_sources))]
    pub fn load_default_resource() -> Result<Self, String> {
        Err(
            "optional Java decompilation root is unavailable; cannot load deprecated translations"
                .to_string(),
        )
    }

    pub fn apply_to_map(&self, translations: &mut BTreeMap<String, String>) {
        for key in &self.removed {
            translations.remove(key);
        }

        for (from_key, to_key) in &self.renamed {
            if let Some(value) = translations.remove(from_key) {
                translations.insert(to_key.clone(), value);
            } else {
                translations.remove(to_key);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageModel {
    translations: BTreeMap<String, String>,
    default_right_to_left: bool,
}

impl LanguageModel {
    #[cfg(vibecraft_has_decompiled_sources)]
    pub fn load_default() -> Result<Self, String> {
        let mut translations = BTreeMap::new();
        load_translations_from_json(
            include_str!("../../../decompiled-server-26.1.2/assets/minecraft/lang/en_us.json"),
            &mut translations,
        )?;
        DeprecatedTranslationsInfoModel::load_default_resource()?.apply_to_map(&mut translations);
        Ok(Self {
            translations,
            default_right_to_left: false,
        })
    }

    #[cfg(not(vibecraft_has_decompiled_sources))]
    pub fn load_default() -> Result<Self, String> {
        Err("optional Java decompilation root is unavailable; cannot load en_us translations"
            .to_string())
    }

    pub fn injected(translations: BTreeMap<String, String>, default_right_to_left: bool) -> Self {
        Self {
            translations,
            default_right_to_left,
        }
    }

    pub fn get_or_default(&self, element_id: &str) -> String {
        self.get_or_default_with(element_id, element_id)
    }

    pub fn get_or_default_with(&self, element_id: &str, default_value: &str) -> String {
        self.translations
            .get(element_id)
            .cloned()
            .unwrap_or_else(|| default_value.to_string())
    }

    pub fn has(&self, element_id: &str) -> bool {
        self.translations.contains_key(element_id)
    }

    pub fn is_default_right_to_left(&self) -> bool {
        self.default_right_to_left
    }

    pub fn visual_order(&self, logical_order_text: &str) -> String {
        logical_order_text.to_string()
    }

    pub fn visual_order_lines<'a>(&self, lines: impl IntoIterator<Item = &'a str>) -> Vec<String> {
        lines
            .into_iter()
            .map(|line| self.visual_order(line))
            .collect()
    }
}

pub fn load_translations_from_json(
    raw: &str,
    output: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for (key, value) in parse_string_map_object(raw)? {
        output.insert(key, replace_unsupported_format_specifiers(&value));
    }
    Ok(())
}

pub fn replace_unsupported_format_specifiers(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let start = i;
        i += 1;
        let argument_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let argument = if i < chars.len() && chars[i] == '$' {
            i += 1;
            Some(argument_start..i)
        } else {
            i = argument_start;
            None
        };
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
            i += 1;
        }
        if i < chars.len() && matches!(chars[i], 'd' | 'f') {
            out.push('%');
            if let Some(argument) = argument {
                out.extend(chars[argument].iter().copied());
            }
            out.push('s');
            i += 1;
        } else {
            out.extend(chars[start..i].iter().copied());
        }
    }
    out
}

fn parse_string_array_field(raw: &str, field: &str) -> Result<Vec<String>, String> {
    let value = object_field(raw, field)?.ok_or_else(|| format!("missing field {field}"))?;
    parse_string_array(value)
}

fn parse_string_map_field(raw: &str, field: &str) -> Result<BTreeMap<String, String>, String> {
    let value = object_field(raw, field)?.ok_or_else(|| format!("missing field {field}"))?;
    parse_string_map_object(value)
}

fn object_field<'a>(raw: &'a str, field: &str) -> Result<Option<&'a str>, String> {
    let mut cursor = skip_ws(raw, 0);
    expect_char(raw, &mut cursor, '{')?;
    loop {
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, '}') {
            return Ok(None);
        }
        let key = parse_json_string(raw, &mut cursor)?;
        cursor = skip_ws(raw, cursor);
        expect_char(raw, &mut cursor, ':')?;
        cursor = skip_ws(raw, cursor);
        let start = cursor;
        skip_json_value(raw, &mut cursor)?;
        if key == field {
            return Ok(Some(&raw[start..cursor]));
        }
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, ',') {
            continue;
        }
        expect_char(raw, &mut cursor, '}')?;
        return Ok(None);
    }
}

fn parse_string_map_object(raw: &str) -> Result<BTreeMap<String, String>, String> {
    let mut cursor = skip_ws(raw, 0);
    expect_char(raw, &mut cursor, '{')?;
    let mut map = BTreeMap::new();
    loop {
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, '}') {
            return Ok(map);
        }
        let key = parse_json_string(raw, &mut cursor)?;
        cursor = skip_ws(raw, cursor);
        expect_char(raw, &mut cursor, ':')?;
        cursor = skip_ws(raw, cursor);
        let value = parse_json_string(raw, &mut cursor)?;
        map.insert(key, value);
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, ',') {
            continue;
        }
        expect_char(raw, &mut cursor, '}')?;
        return Ok(map);
    }
}

fn parse_string_array(raw: &str) -> Result<Vec<String>, String> {
    let mut cursor = skip_ws(raw, 0);
    expect_char(raw, &mut cursor, '[')?;
    let mut values = Vec::new();
    loop {
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, ']') {
            return Ok(values);
        }
        values.push(parse_json_string(raw, &mut cursor)?);
        cursor = skip_ws(raw, cursor);
        if consume_char(raw, &mut cursor, ',') {
            continue;
        }
        expect_char(raw, &mut cursor, ']')?;
        return Ok(values);
    }
}

fn skip_json_value(raw: &str, cursor: &mut usize) -> Result<(), String> {
    match peek_char(raw, *cursor) {
        Some('"') => {
            parse_json_string(raw, cursor)?;
            Ok(())
        }
        Some('{') => skip_balanced(raw, cursor, '{', '}'),
        Some('[') => skip_balanced(raw, cursor, '[', ']'),
        Some(_) => {
            while let Some(ch) = peek_char(raw, *cursor) {
                if matches!(ch, ',' | '}' | ']') {
                    break;
                }
                *cursor += ch.len_utf8();
            }
            Ok(())
        }
        None => Err("unexpected end while skipping value".to_string()),
    }
}

fn skip_balanced(raw: &str, cursor: &mut usize, open: char, close: char) -> Result<(), String> {
    expect_char(raw, cursor, open)?;
    let mut depth = 1;
    while depth > 0 {
        let Some(ch) = peek_char(raw, *cursor) else {
            return Err("unterminated JSON container".to_string());
        };
        if ch == '"' {
            parse_json_string(raw, cursor)?;
        } else {
            *cursor += ch.len_utf8();
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
            }
        }
    }
    Ok(())
}

fn parse_json_string(raw: &str, cursor: &mut usize) -> Result<String, String> {
    expect_char(raw, cursor, '"')?;
    let mut out = String::new();
    while let Some(ch) = peek_char(raw, *cursor) {
        *cursor += ch.len_utf8();
        match ch {
            '"' => return Ok(out),
            '\\' => out.push(parse_escape(raw, cursor)?),
            _ => out.push(ch),
        }
    }
    Err("unterminated JSON string".to_string())
}

fn parse_escape(raw: &str, cursor: &mut usize) -> Result<char, String> {
    let Some(ch) = peek_char(raw, *cursor) else {
        return Err("unterminated JSON escape".to_string());
    };
    *cursor += ch.len_utf8();
    match ch {
        '"' | '\\' | '/' => Ok(ch),
        'b' => Ok('\u{0008}'),
        'f' => Ok('\u{000C}'),
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        't' => Ok('\t'),
        'u' => {
            let end = *cursor + 4;
            let digits = raw
                .get(*cursor..end)
                .ok_or_else(|| "short unicode escape".to_string())?;
            *cursor = end;
            let value = u16::from_str_radix(digits, 16)
                .map_err(|_| format!("invalid unicode escape {digits}"))?;
            char::from_u32(u32::from(value)).ok_or_else(|| "invalid unicode scalar".to_string())
        }
        _ => Err(format!("invalid JSON escape {ch}")),
    }
}

fn skip_ws(raw: &str, mut cursor: usize) -> usize {
    while let Some(ch) = peek_char(raw, cursor) {
        if !ch.is_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }
    cursor
}

fn expect_char(raw: &str, cursor: &mut usize, expected: char) -> Result<(), String> {
    if consume_char(raw, cursor, expected) {
        Ok(())
    } else {
        Err(format!("expected {expected} at byte {cursor}"))
    }
}

fn consume_char(raw: &str, cursor: &mut usize, expected: char) -> bool {
    if peek_char(raw, *cursor) == Some(expected) {
        *cursor += expected.len_utf8();
        true
    } else {
        false
    }
}

fn peek_char(raw: &str, cursor: usize) -> Option<char> {
    raw.get(cursor..)?.chars().next()
}
