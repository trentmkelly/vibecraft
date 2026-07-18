//! CSV writer with Minecraft's header and null-field conventions.

#![allow(dead_code)]

use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, Write};

const LINE_SEPARATOR: &str = "\r\n";
const FIELD_SEPARATOR: &str = ",";

/// Failure emitted while constructing or writing a [`CsvOutput`].
#[derive(Debug)]
pub enum CsvOutputError {
    Io(io::Error),
    InvalidColumnCount { expected: usize, actual: usize },
}

impl Display for CsvOutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidColumnCount { expected, actual } => write!(
                formatter,
                "Invalid number of columns, expected {expected}, but got {actual}"
            ),
        }
    }
}

impl Error for CsvOutputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidColumnCount { .. } => None,
        }
    }
}

impl From<io::Error> for CsvOutputError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// A header-first CSV writer matching `net.minecraft.util.CsvOutput`.
pub struct CsvOutput<W> {
    output: W,
    column_count: usize,
}

impl<W: Write> CsvOutput<W> {
    /// Writes one row, rejecting a value count different from the header count.
    pub fn write_row(
        &mut self,
        values: &[Option<&dyn Display>],
    ) -> Result<(), CsvOutputError> {
        if values.len() != self.column_count {
            return Err(CsvOutputError::InvalidColumnCount {
                expected: self.column_count,
                actual: values.len(),
            });
        }
        self.write_line(values.iter().copied())
    }

    pub fn into_inner(self) -> W {
        self.output
    }

    fn write_line<'a>(
        &mut self,
        values: impl Iterator<Item = Option<&'a dyn Display>>,
    ) -> Result<(), CsvOutputError> {
        let row = values.map(csv_field).collect::<Vec<_>>().join(FIELD_SEPARATOR);
        self.output.write_all(row.as_bytes())?;
        self.output.write_all(LINE_SEPARATOR.as_bytes())?;
        Ok(())
    }
}

/// Mutable-style builder used to collect the header row before creating a writer.
#[derive(Debug, Clone, Default)]
pub struct CsvOutputBuilder {
    headers: Vec<String>,
}

impl CsvOutputBuilder {
    pub const fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Adds a header, returning the builder as Java's fluent `addColumn` does.
    pub fn add_column(mut self, header: impl Into<String>) -> Self {
        self.headers.push(header.into());
        self
    }

    /// Writes the header immediately and returns the configured CSV output.
    pub fn build<W: Write>(self, output: W) -> Result<CsvOutput<W>, CsvOutputError> {
        let mut csv = CsvOutput {
            output,
            column_count: self.headers.len(),
        };
        csv.write_line(self.headers.iter().map(|header| Some(header as &dyn Display)))?;
        Ok(csv)
    }
}

pub fn builder() -> CsvOutputBuilder {
    CsvOutputBuilder::new()
}

fn csv_field(value: Option<&dyn Display>) -> String {
    let value = value.map_or_else(|| "[null]".to_string(), ToString::to_string);
    if [FIELD_SEPARATOR, "\"", "\r", "\n"]
        .iter()
        .any(|needle| value.contains(needle))
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::{builder, CsvOutputError};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn csv_output_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/CsvOutput.java");
        assert_eq!(JAVA.lines().count(), 51);
        for fragment in [
            "private static final String LINE_SEPARATOR = \"\\r\\n\"",
            "private static final String FIELD_SEPARATOR = \",\"",
            "private final int columnCount",
            "Invalid number of columns, expected ",
            "Collectors.joining(\",\") + \"\\r\\n\"",
            "StringEscapeUtils.escapeCsv(value != null ? value.toString() : \"[null]\")",
            "public CsvOutput.Builder addColumn(final String header)",
        ] {
            assert!(JAVA.contains(fragment), "missing CsvOutput source fragment: {fragment}");
        }
    }

    #[test]
    fn headers_rows_nulls_and_csv_escaping_match_java() {
        let mut csv = builder()
            .add_column("plain")
            .add_column("escaped,header")
            .add_column("nullable")
            .build(Vec::new())
            .unwrap_or_else(|error| panic!("header write should succeed: {error}"));
        let number = 42;
        let quoted = "say \"hi\", then\nnext";
        let values: [Option<&dyn Display>; 3] = [Some(&number), Some(&quoted), None];
        csv.write_row(&values)
            .unwrap_or_else(|error| panic!("row write should succeed: {error}"));
        let bytes = csv.into_inner();
        let text = String::from_utf8(bytes).unwrap_or_else(|error| panic!("CSV must be UTF-8: {error}"));
        assert_eq!(
            text,
            "plain,\"escaped,header\",nullable\r\n42,\"say \"\"hi\"\", then\nnext\",[null]\r\n"
        );
    }

    #[test]
    fn row_width_is_checked_before_writing() {
        let mut csv = builder()
            .add_column("one")
            .add_column("two")
            .build(Vec::new())
            .unwrap_or_else(|error| panic!("header write should succeed: {error}"));
        let value = "only one";
        let fields: [Option<&dyn Display>; 1] = [Some(&value)];
        assert!(matches!(
            csv.write_row(&fields),
            Err(CsvOutputError::InvalidColumnCount {
                expected: 2,
                actual: 1,
            })
        ));
        assert_eq!(
            csv.into_inner(),
            b"one,two\r\n",
            "a rejected row must not reach the writer"
        );
    }
}
