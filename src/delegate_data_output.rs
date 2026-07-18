//! Complete forwarding wrapper for Java's `DataOutput` interface.

#![allow(dead_code)]

use std::io;

/// Rust representation of every operation on Java's `java.io.DataOutput`.
///
/// Implementations own byte order, string encoding, and range validation exactly as
/// their Java counterpart does; [`DelegateDataOutput`] deliberately adds no policy.
pub trait DataOutput {
    fn write(&mut self, value: i32) -> io::Result<()>;
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn write_bytes_range(&mut self, bytes: &[u8], offset: usize, length: usize) -> io::Result<()>;
    fn write_boolean(&mut self, value: bool) -> io::Result<()>;
    fn write_byte(&mut self, value: i32) -> io::Result<()>;
    fn write_short(&mut self, value: i32) -> io::Result<()>;
    fn write_char(&mut self, value: i32) -> io::Result<()>;
    fn write_int(&mut self, value: i32) -> io::Result<()>;
    fn write_long(&mut self, value: i64) -> io::Result<()>;
    fn write_float(&mut self, value: f32) -> io::Result<()>;
    fn write_double(&mut self, value: f64) -> io::Result<()>;
    fn write_bytes_string(&mut self, value: &str) -> io::Result<()>;
    fn write_chars(&mut self, value: &str) -> io::Result<()>;
    fn write_utf(&mut self, value: &str) -> io::Result<()>;
}

/// A transparent `DataOutput` decorator, equivalent to Minecraft's Java wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateDataOutput<T> {
    parent: T,
}

impl<T> DelegateDataOutput<T> {
    pub const fn new(parent: T) -> Self {
        Self { parent }
    }

    pub fn into_inner(self) -> T {
        self.parent
    }

    pub fn parent(&self) -> &T {
        &self.parent
    }

    pub fn parent_mut(&mut self) -> &mut T {
        &mut self.parent
    }
}

impl<T: DataOutput> DataOutput for DelegateDataOutput<T> {
    fn write(&mut self, value: i32) -> io::Result<()> { self.parent.write(value) }
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> { self.parent.write_bytes(bytes) }
    fn write_bytes_range(&mut self, bytes: &[u8], offset: usize, length: usize) -> io::Result<()> { self.parent.write_bytes_range(bytes, offset, length) }
    fn write_boolean(&mut self, value: bool) -> io::Result<()> { self.parent.write_boolean(value) }
    fn write_byte(&mut self, value: i32) -> io::Result<()> { self.parent.write_byte(value) }
    fn write_short(&mut self, value: i32) -> io::Result<()> { self.parent.write_short(value) }
    fn write_char(&mut self, value: i32) -> io::Result<()> { self.parent.write_char(value) }
    fn write_int(&mut self, value: i32) -> io::Result<()> { self.parent.write_int(value) }
    fn write_long(&mut self, value: i64) -> io::Result<()> { self.parent.write_long(value) }
    fn write_float(&mut self, value: f32) -> io::Result<()> { self.parent.write_float(value) }
    fn write_double(&mut self, value: f64) -> io::Result<()> { self.parent.write_double(value) }
    fn write_bytes_string(&mut self, value: &str) -> io::Result<()> { self.parent.write_bytes_string(value) }
    fn write_chars(&mut self, value: &str) -> io::Result<()> { self.parent.write_chars(value) }
    fn write_utf(&mut self, value: &str) -> io::Result<()> { self.parent.write_utf(value) }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{DataOutput, DelegateDataOutput};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn delegate_data_output_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/DelegateDataOutput.java");
        assert_eq!(JAVA.lines().count(), 84);
        for fragment in ["implements DataOutput", "this.parent.write(b)", "this.parent.write(b, off, len)", "this.parent.writeBoolean(v)", "this.parent.writeDouble(v)", "this.parent.writeBytes(s)", "this.parent.writeChars(s)", "this.parent.writeUTF(s)"] {
            assert!(JAVA.contains(fragment), "missing DelegateDataOutput source fragment: {fragment}");
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Recorder(Vec<String>);
    macro_rules! record { ($name:ident, $type:ty) => { fn $name(&mut self, value: $type) -> io::Result<()> { self.0.push(format!("{}:{value:?}", stringify!($name))); Ok(()) } }; }
    impl DataOutput for Recorder {
        record!(write, i32); record!(write_boolean, bool); record!(write_byte, i32); record!(write_short, i32); record!(write_char, i32); record!(write_int, i32); record!(write_long, i64); record!(write_float, f32); record!(write_double, f64); record!(write_bytes_string, &str); record!(write_chars, &str); record!(write_utf, &str);
        fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> { self.0.push(format!("write_bytes:{bytes:?}")); Ok(()) }
        fn write_bytes_range(&mut self, bytes: &[u8], offset: usize, length: usize) -> io::Result<()> { self.0.push(format!("write_bytes_range:{bytes:?}:{offset}:{length}")); Ok(()) }
    }

    #[test]
    fn every_data_output_operation_is_forwarded_unchanged() {
        let mut output = DelegateDataOutput::new(Recorder::default());
        output.write(1).unwrap_or_else(|error| panic!("write: {error}")); output.write_bytes(&[2, 3]).unwrap_or_else(|error| panic!("bytes: {error}")); output.write_bytes_range(&[4, 5, 6], 1, 2).unwrap_or_else(|error| panic!("range: {error}")); output.write_boolean(true).unwrap_or_else(|error| panic!("bool: {error}")); output.write_byte(-7).unwrap_or_else(|error| panic!("byte: {error}")); output.write_short(8).unwrap_or_else(|error| panic!("short: {error}")); output.write_char(65).unwrap_or_else(|error| panic!("char: {error}")); output.write_int(-9).unwrap_or_else(|error| panic!("int: {error}")); output.write_long(10).unwrap_or_else(|error| panic!("long: {error}")); output.write_float(1.5).unwrap_or_else(|error| panic!("float: {error}")); output.write_double(2.5).unwrap_or_else(|error| panic!("double: {error}")); output.write_bytes_string("bytes").unwrap_or_else(|error| panic!("string: {error}")); output.write_chars("chars").unwrap_or_else(|error| panic!("chars: {error}")); output.write_utf("utf").unwrap_or_else(|error| panic!("utf: {error}"));
        assert_eq!(output.into_inner().0, ["write:1", "write_bytes:[2, 3]", "write_bytes_range:[4, 5, 6]:1:2", "write_boolean:true", "write_byte:-7", "write_short:8", "write_char:65", "write_int:-9", "write_long:10", "write_float:1.5", "write_double:2.5", "write_bytes_string:\"bytes\"", "write_chars:\"chars\"", "write_utf:\"utf\""]);
    }
}
