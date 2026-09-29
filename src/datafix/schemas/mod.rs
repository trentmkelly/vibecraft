//! Ports of `net.minecraft.util.datafix.schemas.*`: the per-version type registries.
//!
//! Each module exposes `build`, which applies that version's registrations on
//! top of the schema derived from its parent (the Java `super.registerX` calls).

pub mod v100;
pub mod v102;
pub mod v1022;
pub mod v106;
pub mod v107;
pub mod v1125;
pub mod v135;
pub mod v143;
pub mod v1451;
pub mod v1451_1;
pub mod v1451_2;
pub mod v1451_3;
pub mod v1451_4;
pub mod v1451_5;
pub mod v1451_6;
pub mod v1458;
pub mod v1460;
pub mod v1466;
pub mod v1470;
pub mod v1481;
pub mod v1483;
pub mod v1486;
pub mod v1488;
pub mod v1510;
pub mod v501;
pub mod v700;
pub mod v701;
pub mod v702;
pub mod v703;
pub mod v704;
mod v704_item_to_block_entity;
pub mod v705;
mod v705_item_to_entity;
pub mod v808;
pub mod v99;
