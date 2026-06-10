#![allow(dead_code)]

pub struct HandlerNames;

impl HandlerNames {
    pub const DECOMPRESS: &'static str = "decompress";
    pub const COMPRESS: &'static str = "compress";
    pub const DECODER: &'static str = "decoder";
    pub const ENCODER: &'static str = "encoder";
    pub const INBOUND_CONFIG: &'static str = "inbound_config";
    pub const OUTBOUND_CONFIG: &'static str = "outbound_config";
    pub const SPLITTER: &'static str = "splitter";
    pub const PREPENDER: &'static str = "prepender";
    pub const DECRYPT: &'static str = "decrypt";
    pub const ENCRYPT: &'static str = "encrypt";
    pub const UNBUNDLER: &'static str = "unbundler";
    pub const BUNDLER: &'static str = "bundler";
    pub const PACKET_HANDLER: &'static str = "packet_handler";
    pub const TIMEOUT: &'static str = "timeout";
    pub const LEGACY_QUERY: &'static str = "legacy_query";
    pub const LATENCY: &'static str = "latency";
}
