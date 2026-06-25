use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTestInstanceBlockStatus {
    pub status: ComponentJson,
    pub size: Option<Vec3iData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3iData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
