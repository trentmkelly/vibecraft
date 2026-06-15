#![allow(dead_code)]

use super::numeric::NbtNumericValue;
use super::Tag;

pub const BUILTIN_TRUE: &str = "true";
pub const BUILTIN_FALSE: &str = "false";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuiltinKey {
    pub id: &'static str,
    pub arg_count: usize,
}

impl BuiltinKey {
    pub const fn new(id: &'static str, arg_count: usize) -> Self {
        Self { id, arg_count }
    }
}

impl std::fmt::Display for BuiltinKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}/{}", self.id, self.arg_count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnbtOperationError {
    ExpectedNumberOrBoolean,
    ExpectedStringUuid,
    UnknownBuiltin { id: String, arg_count: usize },
}

pub const BOOL_KEY: BuiltinKey = BuiltinKey::new("bool", 1);
pub const UUID_KEY: BuiltinKey = BuiltinKey::new("uuid", 1);

pub fn builtin_ids() -> Vec<&'static str> {
    let mut values = vec![BUILTIN_FALSE, BUILTIN_TRUE, BOOL_KEY.id, UUID_KEY.id];
    values.sort_unstable();
    values
}

pub fn run_builtin(id: &str, arguments: &[Tag]) -> Result<Tag, SnbtOperationError> {
    match (id, arguments.len()) {
        ("bool", 1) => run_bool(&arguments[0]),
        ("uuid", 1) => run_uuid(&arguments[0]),
        _ => Err(SnbtOperationError::UnknownBuiltin {
            id: id.to_string(),
            arg_count: arguments.len(),
        }),
    }
}

pub fn run_bool(argument: &Tag) -> Result<Tag, SnbtOperationError> {
    bool_from_tag(argument)
        .map(|value| Tag::Byte(i8::from(value)))
        .ok_or(SnbtOperationError::ExpectedNumberOrBoolean)
}

pub fn bool_from_tag(argument: &Tag) -> Option<bool> {
    match argument {
        Tag::Byte(value) if *value == 0 || *value == 1 => Some(*value != 0),
        _ => argument
            .numeric_value()
            .map(NbtNumericValue::double_value)
            .map(|value| value != 0.0),
    }
}

pub fn run_uuid(argument: &Tag) -> Result<Tag, SnbtOperationError> {
    let Tag::String(value) = argument else {
        return Err(SnbtOperationError::ExpectedStringUuid);
    };
    uuid_string_to_int_array(value)
        .map(Tag::IntArray)
        .ok_or(SnbtOperationError::ExpectedStringUuid)
}

pub fn uuid_string_to_int_array(value: &str) -> Option<Vec<i32>> {
    let hex = parse_uuid_hex(value)?;
    let most = u64::from_str_radix(&hex[..16], 16).ok()?;
    let least = u64::from_str_radix(&hex[16..], 16).ok()?;
    Some(vec![
        (most >> 32) as u32 as i32,
        most as u32 as i32,
        (least >> 32) as u32 as i32,
        least as u32 as i32,
    ])
}

fn parse_uuid_hex(value: &str) -> Option<String> {
    if value.len() != 36 {
        return None;
    }
    for index in [8, 13, 18, 23] {
        if value.as_bytes().get(index).copied() != Some(b'-') {
            return None;
        }
    }
    let hex = value.replace('-', "");
    if hex.len() == 32 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex)
    } else {
        None
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const SNBT_OPERATIONS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/nbt/SnbtOperations.java");
    const UUID_UTIL_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/core/UUIDUtil.java");

    #[test]
    fn snbt_operations_match_java_bool_uuid_builtins() {
        for sentinel in [
            "public static final String BUILTIN_TRUE = \"true\";",
            "public static final String BUILTIN_FALSE = \"false\";",
            "new SnbtOperations.BuiltinKey(\"bool\", 1)",
            "new SnbtOperations.BuiltinKey(\"uuid\", 1)",
            "ops.getBooleanValue(arg).result()",
            "ops.getNumberValue(arg).result()",
            "asNumber.get().doubleValue() != 0.0",
            "UUID.fromString(arg.get())",
            "ops.createIntList(IntStream.of(UUIDUtil.uuidToIntArray(uuid)))",
            "Stream.of(\"false\", \"true\")",
            "return this.id + \"/\" + this.argCount;",
        ] {
            assert!(
                SNBT_OPERATIONS_JAVA.contains(sentinel),
                "missing SnbtOperations sentinel {sentinel}"
            );
        }
        for sentinel in [
            "public static int[] uuidToIntArray(final UUID uuid)",
            "return new int[]{(int)(mostSignificantBits >> 32), (int)mostSignificantBits, (int)(leastSignificantBits >> 32), (int)leastSignificantBits};",
        ] {
            assert!(UUID_UTIL_JAVA.contains(sentinel), "missing UUID sentinel {sentinel}");
        }

        assert_eq!(BUILTIN_TRUE, "true");
        assert_eq!(BUILTIN_FALSE, "false");
        assert_eq!(BOOL_KEY.to_string(), "bool/1");
        assert_eq!(UUID_KEY.to_string(), "uuid/1");
        assert_eq!(builtin_ids(), vec!["bool", "false", "true", "uuid"]);

        assert_eq!(run_bool(&Tag::Byte(1)).unwrap(), Tag::Byte(1));
        assert_eq!(run_bool(&Tag::Byte(0)).unwrap(), Tag::Byte(0));
        assert_eq!(run_bool(&Tag::Int(-2)).unwrap(), Tag::Byte(1));
        assert_eq!(run_bool(&Tag::Double(0.0)).unwrap(), Tag::Byte(0));
        assert_eq!(
            run_bool(&Tag::String("no".to_string())).unwrap_err(),
            SnbtOperationError::ExpectedNumberOrBoolean
        );

        let uuid = "12345678-9abc-def0-8000-000000000001";
        assert_eq!(
            run_uuid(&Tag::String(uuid.to_string())).unwrap(),
            Tag::IntArray(vec![
                0x1234_5678,
                0x9ABC_DEF0_u32 as i32,
                0x8000_0000_u32 as i32,
                1
            ])
        );
        assert_eq!(
            run_uuid(&Tag::String("not-a-uuid".to_string())).unwrap_err(),
            SnbtOperationError::ExpectedStringUuid
        );
        assert_eq!(
            run_uuid(&Tag::Int(1)).unwrap_err(),
            SnbtOperationError::ExpectedStringUuid
        );
        assert_eq!(
            run_builtin("missing", &[Tag::Int(1)]).unwrap_err(),
            SnbtOperationError::UnknownBuiltin {
                id: "missing".to_string(),
                arg_count: 1,
            }
        );
    }
}
