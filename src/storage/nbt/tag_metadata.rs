pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

pub const OBJECT_HEADER_BYTES: usize = 8;
pub const ARRAY_HEADER_BYTES: usize = 12;
pub const OBJECT_REFERENCE_BYTES: usize = 4;
pub const STRING_SIZE_BYTES: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NbtTagTypeInfo {
    pub id: u8,
    pub name: &'static str,
    pub pretty_name: &'static str,
    pub static_payload_size: Option<usize>,
    pub self_size_in_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtTagTypeLookup {
    Known(NbtTagTypeInfo),
    Invalid {
        id: i32,
        name: String,
        pretty_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootVisitResult {
    Continue,
    Halt,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootParseAction {
    ParsePayload,
    Stop,
    SkipPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedNbtExceptionModel {
    pub crash_report: String,
}

impl ReportedNbtExceptionModel {
    pub fn new(crash_report: impl Into<String>) -> Self {
        Self {
            crash_report: crash_report.into(),
        }
    }
}

pub const TAG_TYPES: [NbtTagTypeInfo; 13] = [
    NbtTagTypeInfo {
        id: TAG_END,
        name: "END",
        pretty_name: "TAG_End",
        static_payload_size: Some(0),
        self_size_in_bytes: Some(8),
    },
    NbtTagTypeInfo {
        id: TAG_BYTE,
        name: "BYTE",
        pretty_name: "TAG_Byte",
        static_payload_size: Some(1),
        self_size_in_bytes: Some(9),
    },
    NbtTagTypeInfo {
        id: TAG_SHORT,
        name: "SHORT",
        pretty_name: "TAG_Short",
        static_payload_size: Some(2),
        self_size_in_bytes: Some(10),
    },
    NbtTagTypeInfo {
        id: TAG_INT,
        name: "INT",
        pretty_name: "TAG_Int",
        static_payload_size: Some(4),
        self_size_in_bytes: Some(12),
    },
    NbtTagTypeInfo {
        id: TAG_LONG,
        name: "LONG",
        pretty_name: "TAG_Long",
        static_payload_size: Some(8),
        self_size_in_bytes: Some(16),
    },
    NbtTagTypeInfo {
        id: TAG_FLOAT,
        name: "FLOAT",
        pretty_name: "TAG_Float",
        static_payload_size: Some(4),
        self_size_in_bytes: Some(12),
    },
    NbtTagTypeInfo {
        id: TAG_DOUBLE,
        name: "DOUBLE",
        pretty_name: "TAG_Double",
        static_payload_size: Some(8),
        self_size_in_bytes: Some(16),
    },
    NbtTagTypeInfo {
        id: TAG_BYTE_ARRAY,
        name: "BYTE[]",
        pretty_name: "TAG_Byte_Array",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
    NbtTagTypeInfo {
        id: TAG_STRING,
        name: "STRING",
        pretty_name: "TAG_String",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
    NbtTagTypeInfo {
        id: TAG_LIST,
        name: "LIST",
        pretty_name: "TAG_List",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
    NbtTagTypeInfo {
        id: TAG_COMPOUND,
        name: "COMPOUND",
        pretty_name: "TAG_Compound",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
    NbtTagTypeInfo {
        id: TAG_INT_ARRAY,
        name: "INT[]",
        pretty_name: "TAG_Int_Array",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
    NbtTagTypeInfo {
        id: TAG_LONG_ARRAY,
        name: "LONG[]",
        pretty_name: "TAG_Long_Array",
        static_payload_size: None,
        self_size_in_bytes: None,
    },
];

pub fn tag_type(type_id: i32) -> NbtTagTypeLookup {
    usize::try_from(type_id)
        .ok()
        .and_then(|index| TAG_TYPES.get(index).copied())
        .map(NbtTagTypeLookup::Known)
        .unwrap_or_else(|| NbtTagTypeLookup::Invalid {
            id: type_id,
            name: format!("INVALID[{type_id}]"),
            pretty_name: format!("UNKNOWN_{type_id}"),
        })
}

pub fn root_parse_action(result: RootVisitResult) -> RootParseAction {
    match result {
        RootVisitResult::Continue => RootParseAction::ParsePayload,
        RootVisitResult::Halt => RootParseAction::Stop,
        RootVisitResult::Break => RootParseAction::SkipPayload,
    }
}
