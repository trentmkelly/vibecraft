//! Path allow-list matching from Minecraft's `PathAllowList`.
//!
//! The parser follows `PathAllowList.ConfigEntry.parse`: blank lines and lines
//! whose first character is `#` are ignored, while all other text is kept
//! verbatim. Prefix rules compare the path's string form; glob and regex rules
//! use full-string matching, like `PathMatcher.matches`.
//!
//! Java's filesystem matcher is compiled on demand per filesystem provider and
//! the Java implementation disables the whole list if any matcher fails to
//! compile. This Rust implementation has one host filesystem and applies the
//! same fail-closed rule on first match. Regex rules use Rust's `regex` crate;
//! Common Java regex tokens are translated to Java's default ASCII and line
//! terminator behavior. Unsupported Java constructs and regex features with
//! different semantics are rejected so they cannot accidentally allow a path.
//! Rust's engine still does not implement every Java Pattern feature, so regex
//! rules outside this supported subset fail closed rather than claiming parity.

use std::io::{self, BufRead};
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    Prefix(String),
    Glob(String),
    Regex(String),
}

#[derive(Debug, Clone)]
enum Matcher {
    Prefix(String),
    Regex(Regex),
}

/// A list of path rules. Any matching entry allows the path.
#[derive(Debug, Clone)]
pub struct PathAllowList {
    entries: Vec<Entry>,
    compiled: OnceLock<Option<Vec<Matcher>>>,
}

impl PartialEq for PathAllowList {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Eq for PathAllowList {}

impl PathAllowList {
    /// Reads Java `PathAllowList.readPlain` syntax from a buffered source.
    ///
    /// Syntax errors are returned as `InvalidData`, as Rust cannot propagate
    /// Java's `IllegalArgumentException`. Matcher compilation is deferred until
    /// `matches` and disables all rules on failure, matching Java's
    /// `getForFileSystem` behavior.
    pub fn read_plain(reader: impl BufRead) -> io::Result<Self> {
        let mut text = String::new();
        let mut reader = reader;
        reader.read_to_string(&mut text)?;

        let mut entries = Vec::new();
        for line in text.split(['\r', '\n']) {
            if line.is_empty() || line.chars().all(java_is_whitespace) || line.starts_with('#') {
                continue;
            }
            entries.push(parse_entry(line)?);
        }

        Ok(Self {
            entries,
            compiled: OnceLock::new(),
        })
    }

    /// Constructs an allow-list that rejects every path.
    pub fn deny_all() -> Self {
        Self {
            entries: Vec::new(),
            compiled: OnceLock::new(),
        }
    }

    /// Returns whether any configured rule matches this path.
    pub fn matches(&self, path: &Path) -> bool {
        let Some(matchers) = self.compiled.get_or_init(|| compile_entries(&self.entries)) else {
            return false;
        };
        let Some(path_text) = path.to_str() else {
            // Java paths are strings. Refuse lossy matching for non-UTF-8 Rust
            // paths rather than allowing a replacement-character collision.
            return false;
        };
        matchers.iter().any(|matcher| match matcher {
            Matcher::Prefix(prefix) => path_text.starts_with(prefix),
            Matcher::Regex(regex) => regex.is_match(path_text),
        })
    }
}

/// Mirrors `Character.isWhitespace`, which differs from Rust's Unicode
/// whitespace predicate for non-breaking spaces and NEL.
fn java_is_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{001F}'
            | '\u{0020}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

fn parse_entry(line: &str) -> io::Result<Entry> {
    if !line.starts_with('[') {
        return Ok(Entry::Prefix(line.to_owned()));
    }

    let Some(close) = line[1..].find(']') else {
        return Err(invalid_definition(format!(
            "Unterminated type in line '{line}'"
        )));
    };
    let close = close + 1;
    let kind = &line[1..close];
    let contents = &line[close + 1..];
    match kind {
        "glob" => Ok(Entry::Glob(contents.to_owned())),
        "regex" => Ok(Entry::Regex(contents.to_owned())),
        "prefix" => Ok(Entry::Prefix(contents.to_owned())),
        _ => Err(invalid_definition(format!(
            "Unsupported definition type in line '{line}'"
        ))),
    }
}

fn invalid_definition(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn compile_entries(entries: &[Entry]) -> Option<Vec<Matcher>> {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Prefix(prefix) => Some(Matcher::Prefix(prefix.clone())),
            Entry::Glob(glob) => glob_to_regex(glob)
                .and_then(|pattern| Regex::new(&pattern).ok())
                .map(Matcher::Regex),
            Entry::Regex(pattern) => {
                let translated = translate_java_regex(pattern)?;
                Regex::new(&format!(r"\A(?:{translated})\z"))
                    .ok()
                    .map(Matcher::Regex)
            }
        })
        .collect()
}

/// Translates Java's common predefined classes and dot, rejecting syntax whose
/// meaning differs between Java Pattern and Rust regex.
fn translate_java_regex(pattern: &str) -> Option<String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut output = String::with_capacity(pattern.len());
    let mut index = 0;
    let mut in_class = false;
    while index < chars.len() {
        let current = chars[index];
        if current == '\\' {
            let next = *chars.get(index + 1)?;
            if next.is_ascii_digit() || matches!(next, 'k' | 'p' | 'P') {
                return None;
            }
            if !in_class && matches!(next, 'b' | 'B') {
                return None;
            }
            if in_class && matches!(next, 'd' | 'D' | 's' | 'S' | 'w' | 'W') {
                return None;
            }
            let replacement = match next {
                'd' => Some("[0-9]"),
                'D' => Some("[^0-9]"),
                's' => Some(r"[ \t\n\x0B\f\r]"),
                'S' => Some(r"[^ \t\n\x0B\f\r]"),
                'w' => Some("[A-Za-z0-9_]"),
                'W' => Some("[^A-Za-z0-9_]"),
                _ => None,
            };
            if let Some(replacement) = replacement {
                output.push_str(replacement);
            } else {
                output.push('\\');
                output.push(next);
            }
            index += 2;
            continue;
        }
        if in_class
            && matches!(current, '&' | '-' | '~' | '|')
            && chars.get(index + 1) == Some(&current)
        {
            // Rust treats these pairs as set operators. Java only gives &&
            // that meaning; reject the unsupported syntax before compilation.
            return None;
        }
        if current == '[' {
            if in_class {
                // Java permits nested character classes; their union and
                // intersection rules differ from Rust's class syntax.
                return None;
            }
            in_class = true;
        } else if current == ']' {
            in_class = false;
        } else if !in_class {
            if current == '('
                && chars.get(index + 1) == Some(&'?')
                && chars.get(index + 2) != Some(&':')
            {
                return None;
            }
            if matches!(current, '*' | '+' | '?' | '}') && chars.get(index + 1) == Some(&'+') {
                return None;
            }
            if current == '.' {
                output.push_str(r"[^\n\r\x{85}\x{2028}\x{2029}]");
                index += 1;
                continue;
            }
        }
        output.push(current);
        index += 1;
    }
    Some(output)
}

/// Converts Java's Unix glob operators into an anchored Rust regex.
///
/// Supported operators are `*`, `**`, `?`, character classes (including
/// `[!...]`), and brace alternation. Malformed classes/braces and dangling
/// escapes fail compilation for the entire allow-list, as in Java.
fn glob_to_regex(glob: &str) -> Option<String> {
    let chars: Vec<char> = glob.chars().collect();
    let mut output = String::from(r"\A(?:");
    let mut index = 0;
    let mut in_group = false;
    while index < chars.len() {
        match chars[index] {
            '*' => {
                if chars.get(index + 1) == Some(&'*') {
                    // Java's regex dot excludes all line terminators by default.
                    output.push_str(r"[^\n\r\x{85}\x{2028}\x{2029}]*");
                    index += 2;
                } else {
                    output.push_str("[^/]*");
                    index += 1;
                }
            }
            '?' => {
                output.push_str("[^/]");
                index += 1;
            }
            '{' => {
                if in_group {
                    return None;
                }
                in_group = true;
                output.push_str("(?:(?:");
                index += 1;
            }
            '}' => {
                if in_group {
                    in_group = false;
                    output.push_str("))");
                } else {
                    output.push_str(&regex::escape("}"));
                }
                index += 1;
            }
            ',' if in_group => {
                output.push_str(")|(?:");
                index += 1;
            }
            '[' => {
                output.push_str(&glob_class(&chars, &mut index)?);
            }
            '\\' => {
                let escaped = chars.get(index + 1)?;
                output.push_str(&regex::escape(&escaped.to_string()));
                index += 2;
            }
            ch => {
                output.push_str(&regex::escape(&ch.to_string()));
                index += 1;
            }
        }
    }
    if in_group {
        return None;
    }
    output.push_str(r")\z");
    Some(output)
}

/// Reads one Java glob character class and writes its Rust equivalent.
/// Range validation follows OpenJDK `sun.nio.fs.Globs.toRegexPattern`:
/// <https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/sun/nio/fs/Globs.java>.
fn glob_class(chars: &[char], index: &mut usize) -> Option<String> {
    *index += 1;
    let mut negated = false;
    let mut contents = String::new();
    if chars.get(*index) == Some(&'^') {
        contents.push_str(r"\^");
        *index += 1;
    } else {
        if chars.get(*index) == Some(&'!') {
            negated = true;
            *index += 1;
        }
        if chars.get(*index) == Some(&'-') {
            contents.push_str(r"\-");
            *index += 1;
        }
    }

    let mut closed = false;
    let mut range_start = None;
    while *index < chars.len() {
        let ch = chars[*index];
        *index += 1;
        match ch {
            ']' => {
                closed = true;
                break;
            }
            '/' => return None,
            '-' => {
                // OpenJDK Globs checks each range before generating a regex;
                // delegating to Rust would interpret repeated '-' as subtraction.
                let start = range_start.take()?;
                let end = *chars.get(*index)?;
                *index += 1;
                if end == ']' {
                    contents.push_str(r"\-");
                    closed = true;
                    break;
                }
                if end < start {
                    return None;
                }
                contents.push('-');
                contents.push_str(&glob_class_literal(end));
            }
            ch => {
                contents.push_str(&glob_class_literal(ch));
                range_start = Some(ch);
            }
        }
    }
    if !closed {
        return None;
    }
    if negated {
        Some(format!("[^{contents}/]"))
    } else {
        // Even a range spanning '/' must exclude the filesystem separator.
        Some(format!("[[{contents}]&&[^/]]"))
    }
}

fn glob_class_literal(ch: char) -> String {
    match ch {
        // Escape Rust's additional set operators instead of turning Java's
        // literal ampersands, tildes, or pipes into class algebra.
        '&' | '~' | '|' => format!("\\{ch}"),
        ch => regex::escape(&ch.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::PathAllowList;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn parses_prefixes_without_trimming_and_ignores_only_java_comments() {
        let allow =
            PathAllowList::read_plain(Cursor::new("# comment\n  \nroot\n #not-comment\n")).unwrap();
        assert!(allow.matches(Path::new("root/file")));
        assert!(!allow.matches(Path::new(" root/file")));
        assert!(allow.matches(Path::new(" #not-comment/file")));
    }

    #[test]
    fn matches_globs_and_regexes_against_the_entire_path() {
        let allow = PathAllowList::read_plain(Cursor::new(
            "[glob]data/{world,player}-[0-9].dat\n[regex]cache/.*\\.bin\n",
        ))
        .unwrap();
        assert!(allow.matches(Path::new("data/world-3.dat")));
        assert!(allow.matches(Path::new("data/player-8.dat")));
        assert!(!allow.matches(Path::new("data/nested/world-3.dat")));
        assert!(allow.matches(Path::new("cache/a.bin")));
        assert!(!allow.matches(Path::new("prefix/cache/a.bin")));
    }

    #[test]
    fn malformed_definitions_are_invalid_data() {
        for definition in ["[glob", "[unknown]path"] {
            let error = PathAllowList::read_plain(Cursor::new(definition)).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn any_matcher_compilation_failure_disables_the_entire_list() {
        let allow = PathAllowList::read_plain(Cursor::new("allowed\n[glob][broken\n")).unwrap();
        assert!(!allow.matches(Path::new("allowed/path")));

        let unsupported =
            PathAllowList::read_plain(Cursor::new("allowed\n[regex](?=unsupported)\n")).unwrap();
        assert!(!unsupported.matches(Path::new("allowed/path")));

        let unicode_boundary =
            PathAllowList::read_plain(Cursor::new("allowed\n[regex]\\bword\\b\n")).unwrap();
        assert!(!unicode_boundary.matches(Path::new("allowed/path")));

        let unsupported_flags =
            PathAllowList::read_plain(Cursor::new("allowed\n[regex](?U)word\n")).unwrap();
        assert!(!unsupported_flags.matches(Path::new("allowed/path")));
    }

    #[test]
    fn java_regex_predefined_classes_keep_ascii_semantics() {
        let allow = PathAllowList::read_plain(Cursor::new("[regex]\\d+\n")).unwrap();
        assert!(allow.matches(Path::new("123")));
        assert!(!allow.matches(Path::new("١٢٣")));

        let dot = PathAllowList::read_plain(Cursor::new("[regex]x.y\n")).unwrap();
        assert!(!dot.matches(Path::new("x\ry")));
        assert!(!dot.matches(Path::new("x\u{0085}y")));
    }

    #[test]
    fn deny_all_rejects_paths() {
        assert!(!PathAllowList::deny_all().matches(Path::new("anything")));
    }

    #[test]
    fn matches_java_character_class_and_brace_edge_cases() {
        let allow = PathAllowList::read_plain(Cursor::new(
            "[glob][a-z]\n[glob][!a]\n[glob][^a]\n[glob]literal}\n",
        ))
        .unwrap();
        assert!(allow.matches(Path::new("m")));
        assert!(allow.matches(Path::new("b")));
        assert!(allow.matches(Path::new("^")));
        assert!(allow.matches(Path::new("a")));
        assert!(allow.matches(Path::new("literal}")));

        let slash_class = PathAllowList::read_plain(Cursor::new("[glob][!a]\n")).unwrap();
        assert!(!slash_class.matches(Path::new("/")));
        let explicit_separator = PathAllowList::read_plain(Cursor::new("[glob][/]\n")).unwrap();
        assert!(!explicit_separator.matches(Path::new("anything")));
        let nested_group = PathAllowList::read_plain(Cursor::new("[glob]{a,{b,c}}\n")).unwrap();
        assert!(!nested_group.matches(Path::new("a")));
        let ampersand_class = PathAllowList::read_plain(Cursor::new("[glob][a&&b]\n")).unwrap();
        assert!(ampersand_class.matches(Path::new("a")));
        assert!(ampersand_class.matches(Path::new("b")));
        assert!(ampersand_class.matches(Path::new("&")));
    }

    #[test]
    fn java_blank_line_detection_does_not_treat_non_breaking_space_as_blank() {
        let allow = PathAllowList::read_plain(Cursor::new("\u{00A0}\n\u{0085}\n")).unwrap();
        assert!(allow.matches(Path::new("\u{00A0}/child")));
        assert!(allow.matches(Path::new("\u{0085}/child")));
    }

    #[test]
    fn regex_class_set_operators_disable_the_whole_policy() {
        for pattern in ["[a--b]", "[a~~b]", "[a||b]", "[a&&b]"] {
            let allow =
                PathAllowList::read_plain(Cursor::new(format!("allowed\n[regex]{pattern}\n")))
                    .unwrap();
            assert!(!allow.matches(Path::new("allowed/file")), "{pattern}");
        }
    }

    #[test]
    fn glob_classes_validate_java_ranges_and_preserve_literal_set_symbols() {
        for pattern in ["[a--b]", "[--a]", "[z-a]", "[a-b-c]"] {
            let allow =
                PathAllowList::read_plain(Cursor::new(format!("allowed\n[glob]{pattern}\n")))
                    .unwrap();
            assert!(!allow.matches(Path::new("allowed/file")), "{pattern}");
        }
        for (pattern, literal) in [("[a~~b]", "~"), ("[a||b]", "|"), ("[a&&b]", "&")] {
            let allow =
                PathAllowList::read_plain(Cursor::new(format!("[glob]{pattern}\n"))).unwrap();
            assert!(allow.matches(Path::new(literal)), "{pattern}");
            assert!(allow.matches(Path::new("a")), "{pattern}");
            assert!(allow.matches(Path::new("b")), "{pattern}");
        }
        let spanning_separator = PathAllowList::read_plain(Cursor::new("[glob][.-0]\n")).unwrap();
        assert!(spanning_separator.matches(Path::new(".")));
        assert!(spanning_separator.matches(Path::new("0")));
        assert!(!spanning_separator.matches(Path::new("/")));
        let trailing_hyphen = PathAllowList::read_plain(Cursor::new("[glob][a-]\n")).unwrap();
        assert!(trailing_hyphen.matches(Path::new("-")));
    }

    #[test]
    fn equality_and_cloning_use_definitions_not_compilation_cache() {
        let first = PathAllowList::read_plain(Cursor::new("root\n")).unwrap();
        let second = first.clone();
        assert_eq!(first, second);
        assert!(first.matches(Path::new("root/file")));
        assert_eq!(first, second);
    }
}
