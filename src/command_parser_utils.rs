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

    pub fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = cursor;
        self
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

    fn skip(&mut self) {
        self.cursor += 1;
    }
}

pub fn read_while(reader: &mut StringReaderModel, predicate: impl Fn(char) -> bool) -> String {
    let start = reader.cursor;
    while reader.can_read() && predicate(reader.peek()) {
        reader.skip();
    }
    reader.input[start..reader.cursor].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_while_returns_consumed_prefix_and_advances_cursor() {
        let mut reader = StringReaderModel::new("abc 123");

        assert_eq!(
            read_while(&mut reader, |ch| ch.is_ascii_alphabetic()),
            "abc"
        );
        assert_eq!(reader.cursor(), 3);
    }

    #[test]
    fn read_while_starts_at_current_cursor() {
        let mut reader = StringReaderModel::new("abc123;").with_cursor(3);

        assert_eq!(read_while(&mut reader, |ch| ch.is_ascii_digit()), "123");
        assert_eq!(reader.cursor(), 6);
    }

    #[test]
    fn read_while_returns_empty_and_keeps_cursor_when_first_char_fails() {
        let mut reader = StringReaderModel::new(" abc");

        assert_eq!(read_while(&mut reader, |ch| ch.is_ascii_alphabetic()), "");
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn read_while_at_end_returns_empty() {
        let mut reader = StringReaderModel::new("abc").with_cursor(3);

        assert_eq!(read_while(&mut reader, |_| true), "");
        assert_eq!(reader.cursor(), 3);
    }
}
