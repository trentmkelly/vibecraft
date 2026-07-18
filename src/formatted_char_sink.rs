//! Callback contract used while visiting styled Unicode code points.

#![allow(dead_code)]

use crate::chat_component::Style;

/// Rust equivalent of Java's `FormattedCharSink` functional interface.
/// Returning `false` asks the caller to stop visiting characters.
pub trait FormattedCharSink {
    fn accept(&mut self, position: i32, style: &Style, codepoint: i32) -> bool;
}

impl<F> FormattedCharSink for F
where
    F: FnMut(i32, &Style, i32) -> bool,
{
    fn accept(&mut self, position: i32, style: &Style, codepoint: i32) -> bool {
        self(position, style, codepoint)
    }
}

#[cfg(test)]
mod tests {
    use crate::chat_component::Style;

    use super::FormattedCharSink;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn formatted_char_sink_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/FormattedCharSink.java");
        assert_eq!(JAVA.lines().count(), 8);
        assert!(JAVA.contains("@FunctionalInterface"));
        assert!(JAVA.contains("boolean accept(int position, Style style, int codepoint)"));
    }

    #[test]
    fn closure_implements_the_stopping_callback_contract() {
        let style = Style::empty();
        let mut visited = Vec::new();
        let mut sink = |position, received_style: &Style, codepoint| {
            visited.push((position, received_style.clone(), codepoint));
            false
        };
        assert!(!sink.accept(3, &style, 0x1F_642));
        assert_eq!(visited, vec![(3, style, 0x1F_642)]);
    }
}
