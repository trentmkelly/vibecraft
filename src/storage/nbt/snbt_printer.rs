#![allow(dead_code)]

use std::collections::BTreeMap;

use super::{snbt_string, Tag};

const DEFAULT_INDENTATION: &str = "    ";

pub fn to_pretty_snbt(tag: &Tag) -> String {
    SnbtPrinter::new(DEFAULT_INDENTATION).visit(tag)
}

pub struct SnbtPrinter {
    indentation: String,
}

impl SnbtPrinter {
    pub fn new(indentation: impl Into<String>) -> Self {
        Self {
            indentation: indentation.into(),
        }
    }

    pub fn visit(&self, tag: &Tag) -> String {
        PrinterState {
            indentation: &self.indentation,
            depth: 0,
            path: Vec::new(),
        }
        .visit(tag)
    }
}

struct PrinterState<'a> {
    indentation: &'a str,
    depth: usize,
    path: Vec<String>,
}

impl PrinterState<'_> {
    fn visit(&self, tag: &Tag) -> String {
        match tag {
            Tag::End => String::new(),
            Tag::Byte(value) => format!("{value}b"),
            Tag::Short(value) => format!("{value}s"),
            Tag::Int(value) => value.to_string(),
            Tag::Long(value) => format!("{value}L"),
            Tag::Float(value) => format!("{}f", java_float_string(*value as f64)),
            Tag::Double(value) => format!("{}d", java_float_string(*value)),
            Tag::ByteArray(values) => {
                self.visit_primitive_array("B", values.iter().map(|value| format!("{value}B")))
            }
            Tag::String(value) => snbt_string::quote_and_escape_snbt_string(value),
            Tag::List(values) => self.visit_list(values),
            Tag::Compound(values) => self.visit_compound(values),
            Tag::IntArray(values) => {
                self.visit_primitive_array("I", values.iter().map(i32::to_string))
            }
            Tag::LongArray(values) => {
                self.visit_primitive_array("L", values.iter().map(|value| format!("{value}L")))
            }
        }
    }

    fn visit_primitive_array<I>(&self, type_name: &str, values: I) -> String
    where
        I: IntoIterator<Item = String>,
    {
        let mut rendered = format!("[{type_name};");
        let values = values.into_iter().collect::<Vec<_>>();
        for (index, value) in values.iter().enumerate() {
            rendered.push(' ');
            rendered.push_str(value);
            if index + 1 != values.len() {
                rendered.push(',');
            }
        }
        rendered.push(']');
        rendered
    }

    fn visit_list(&self, values: &[Tag]) -> String {
        if values.is_empty() {
            return "[]".to_string();
        }

        let mut path = self.path.clone();
        path.push("[]".to_string());
        let indentation = if no_indentation(&path_string(&path)) {
            ""
        } else {
            self.indentation
        };
        let mut rendered = "[".to_string();
        if !indentation.is_empty() {
            rendered.push('\n');
        }

        for (index, value) in values.iter().enumerate() {
            rendered.push_str(&indentation.repeat(self.depth + 1));
            rendered.push_str(
                &PrinterState {
                    indentation,
                    depth: self.depth + 1,
                    path: path.clone(),
                }
                .visit(value),
            );
            if index + 1 != values.len() {
                rendered.push(',');
                rendered.push_str(if indentation.is_empty() { " " } else { "\n" });
            }
        }

        if !indentation.is_empty() {
            rendered.push('\n');
            rendered.push_str(&indentation.repeat(self.depth));
        }
        rendered.push(']');
        rendered
    }

    fn visit_compound(&self, values: &[(String, Tag)]) -> String {
        if values.is_empty() {
            return "{}".to_string();
        }

        let mut path = self.path.clone();
        path.push("{}".to_string());
        let indentation = if no_indentation(&path_string(&path)) {
            ""
        } else {
            self.indentation
        };
        let entries = ordered_compound_entries(values, &path_string(&path));
        let mut rendered = "{".to_string();
        if !indentation.is_empty() {
            rendered.push('\n');
        }

        for (index, (key, value)) in entries.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push((*key).to_string());
            rendered.push_str(&indentation.repeat(self.depth + 1));
            rendered.push_str(&handle_escape_pretty(key));
            rendered.push_str(": ");
            rendered.push_str(
                &PrinterState {
                    indentation,
                    depth: self.depth + 1,
                    path: child_path,
                }
                .visit(value),
            );
            if index + 1 != entries.len() {
                rendered.push(',');
                rendered.push_str(if indentation.is_empty() { " " } else { "\n" });
            }
        }

        if !indentation.is_empty() {
            rendered.push('\n');
            rendered.push_str(&indentation.repeat(self.depth));
        }
        rendered.push('}');
        rendered
    }
}

fn ordered_compound_entries<'a>(
    values: &'a [(String, Tag)],
    path: &str,
) -> Vec<(&'a str, &'a Tag)> {
    let mut by_key = BTreeMap::new();
    for (key, value) in values {
        by_key.insert(key.as_str(), value);
    }

    let mut ordered = Vec::new();
    if let Some(priority) = key_order(path) {
        for &key in priority {
            if let Some(value) = by_key.remove(key) {
                ordered.push((key, value));
            }
        }
    }
    ordered.extend(by_key);
    ordered
}

fn key_order(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "{}" => Some(&[
            "DataVersion",
            "author",
            "size",
            "data",
            "entities",
            "palette",
            "palettes",
        ]),
        "{}.data.[].{}" => Some(&["pos", "state", "nbt"]),
        "{}.entities.[].{}" => Some(&["blockPos", "pos"]),
        _ => None,
    }
}

fn no_indentation(path: &str) -> bool {
    matches!(
        path,
        "{}.size.[]" | "{}.data.[].{}" | "{}.palette.[].{}" | "{}.entities.[].{}"
    )
}

fn path_string(path: &[String]) -> String {
    path.join(".")
}

fn handle_escape_pretty(input: &str) -> String {
    if is_simple_value(input) {
        input.to_string()
    } else {
        snbt_string::quote_and_escape_snbt_string(input)
    }
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

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const SNBT_PRINTER_TAG_VISITOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/nbt/SnbtPrinterTagVisitor.java");

    #[test]
    fn snbt_printer_matches_java_constants_and_scalar_suffixes() {
        for sentinel in [
            "public class SnbtPrinterTagVisitor implements TagVisitor",
            "map.put(\"{}\", Lists.newArrayList",
            "map.put(\"{}.data.[].{}\", Lists.newArrayList",
            "map.put(\"{}.entities.[].{}\", Lists.newArrayList",
            "private static final Set<String> NO_INDENTATION",
            "Pattern.compile(\"[A-Za-z0-9._+-]+\")",
            "this.result = tag.value() + \"L\";",
            "StringBuilder builder = new StringBuilder(\"[\").append(\"B\").append(\";\");",
        ] {
            assert!(
                SNBT_PRINTER_TAG_VISITOR_JAVA.contains(sentinel),
                "missing SnbtPrinterTagVisitor sentinel {sentinel}"
            );
        }

        assert_eq!(to_pretty_snbt(&Tag::End), "");
        assert_eq!(to_pretty_snbt(&Tag::Byte(-1)), "-1b");
        assert_eq!(to_pretty_snbt(&Tag::Short(2)), "2s");
        assert_eq!(to_pretty_snbt(&Tag::Int(3)), "3");
        assert_eq!(to_pretty_snbt(&Tag::Long(4)), "4L");
        assert_eq!(to_pretty_snbt(&Tag::Float(1.0)), "1.0f");
        assert_eq!(to_pretty_snbt(&Tag::Double(2.0)), "2.0d");
        assert_eq!(to_pretty_snbt(&Tag::Float(f32::INFINITY)), "Infinityf");
        assert_eq!(
            to_pretty_snbt(&Tag::Double(f64::NEG_INFINITY)),
            "-Infinityd"
        );
        assert_eq!(
            to_pretty_snbt(&Tag::String("A \"quoted\" name".to_string())),
            "'A \"quoted\" name'"
        );
    }

    #[test]
    fn snbt_printer_formats_primitive_arrays_with_java_spacing() {
        assert_eq!(to_pretty_snbt(&Tag::ByteArray(vec![])), "[B;]");
        assert_eq!(to_pretty_snbt(&Tag::ByteArray(vec![1, -2])), "[B; 1B, -2B]");
        assert_eq!(to_pretty_snbt(&Tag::IntArray(vec![3, -4])), "[I; 3, -4]");
        assert_eq!(to_pretty_snbt(&Tag::LongArray(vec![5, -6])), "[L; 5L, -6L]");
    }

    #[test]
    fn snbt_printer_sorts_keys_and_applies_root_structure_order() {
        let tag = Tag::Compound(vec![
            ("z".to_string(), Tag::Int(9)),
            ("palette".to_string(), Tag::List(vec![])),
            ("DataVersion".to_string(), Tag::Int(4790)),
            ("author".to_string(), Tag::String("vibecraft".to_string())),
            ("a".to_string(), Tag::Byte(1)),
            (
                "true".to_string(),
                Tag::String("simple key here".to_string()),
            ),
        ]);

        assert_eq!(
            to_pretty_snbt(&tag),
            "{\n    DataVersion: 4790,\n    author: \"vibecraft\",\n    palette: [],\n    a: 1b,\n    true: \"simple key here\",\n    z: 9\n}"
        );
    }

    #[test]
    fn snbt_printer_uses_path_based_no_indentation_for_structure_entries() {
        let tag = Tag::Compound(vec![(
            "data".to_string(),
            Tag::List(vec![Tag::Compound(vec![
                (
                    "nbt".to_string(),
                    Tag::Compound(vec![(
                        "id".to_string(),
                        Tag::String("minecraft:stone".to_string()),
                    )]),
                ),
                ("state".to_string(), Tag::Int(4)),
                (
                    "pos".to_string(),
                    Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]),
                ),
            ])]),
        )]);

        assert_eq!(
            to_pretty_snbt(&tag),
            "{\n    data: [\n        {pos: [1, 2, 3], state: 4, nbt: {id: \"minecraft:stone\"}}\n    ]\n}"
        );
    }

    #[test]
    fn snbt_printer_honors_custom_indentation_and_pretty_key_escaping() {
        let printer = SnbtPrinter::new("  ");
        let tag = Tag::Compound(vec![
            ("needs quote".to_string(), Tag::Int(1)),
            ("false".to_string(), Tag::Int(2)),
            (
                "a+b".to_string(),
                Tag::List(vec![Tag::String("x".to_string())]),
            ),
        ]);

        assert_eq!(
            printer.visit(&tag),
            "{\n  a+b: [\n    \"x\"\n  ],\n  false: 2,\n  \"needs quote\": 1\n}"
        );
    }
}
