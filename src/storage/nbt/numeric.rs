#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbtNumericValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl NbtNumericValue {
    /// Number.intValue on the boxed primitive returned by NumericTag.box.
    /// This truncates floating values, whereas NumericTag.intValue floors them.
    pub fn boxed_int_value(self) -> i32 {
        match self {
            Self::Float(value) => value as i32,
            Self::Double(value) => value as i32,
            _ => self.int_value(),
        }
    }

    pub fn boxed_long_value(self) -> i64 {
        match self {
            Self::Float(value) => value as i64,
            Self::Double(value) => value as i64,
            _ => self.long_value(),
        }
    }

    pub fn boxed_byte_value(self) -> i8 {
        self.boxed_int_value() as i8
    }

    pub fn byte_value(self) -> i8 {
        self.int_floor_value() as u8 as i8
    }

    pub fn short_value(self) -> i16 {
        self.int_floor_value() as u16 as i16
    }

    pub fn int_value(self) -> i32 {
        self.int_floor_value()
    }

    pub fn long_value(self) -> i64 {
        match self {
            NbtNumericValue::Byte(value) => i64::from(value),
            NbtNumericValue::Short(value) => i64::from(value),
            NbtNumericValue::Int(value) => i64::from(value),
            NbtNumericValue::Long(value) => value,
            NbtNumericValue::Float(value) => value as i64,
            NbtNumericValue::Double(value) => value.floor() as i64,
        }
    }

    pub fn float_value(self) -> f32 {
        match self {
            NbtNumericValue::Byte(value) => value.into(),
            NbtNumericValue::Short(value) => value.into(),
            NbtNumericValue::Int(value) => value as f32,
            NbtNumericValue::Long(value) => value as f32,
            NbtNumericValue::Float(value) => value,
            NbtNumericValue::Double(value) => value as f32,
        }
    }

    pub fn double_value(self) -> f64 {
        match self {
            NbtNumericValue::Byte(value) => value.into(),
            NbtNumericValue::Short(value) => value.into(),
            NbtNumericValue::Int(value) => value.into(),
            NbtNumericValue::Long(value) => value as f64,
            NbtNumericValue::Float(value) => value.into(),
            NbtNumericValue::Double(value) => value,
        }
    }

    pub fn boolean_value(self) -> bool {
        self.byte_value() != 0
    }

    fn int_floor_value(self) -> i32 {
        match self {
            NbtNumericValue::Byte(value) => value.into(),
            NbtNumericValue::Short(value) => value.into(),
            NbtNumericValue::Int(value) => value,
            NbtNumericValue::Long(value) => value as i32,
            NbtNumericValue::Float(value) => value.floor() as i32,
            NbtNumericValue::Double(value) => value.floor() as i32,
        }
    }
}
