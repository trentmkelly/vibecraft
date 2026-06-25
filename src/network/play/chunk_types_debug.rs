use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscriptionUpdate {
    pub subscription_id: i32,
    pub value_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSubscriptionEvent {
    pub subscription_id: i32,
    pub value_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugBlockValuePacket {
    pub block_pos: crate::block_update::BlockPos,
    pub update: DebugSubscriptionUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugChunkValuePacket {
    pub chunk_pos: ChunkPos,
    pub update: DebugSubscriptionUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugEntityValuePacket {
    pub entity_id: i32,
    pub update: DebugSubscriptionUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugEventPacket {
    pub event: DebugSubscriptionEvent,
}
