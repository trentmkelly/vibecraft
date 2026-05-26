//! Long-packed coordinate helpers used pervasively by the lighting engines.
//!
//! Mirrors the static helpers on `net.minecraft.core.SectionPos` and
//! `net.minecraft.core.BlockPos` for the 26.1.2 release.
//!
//! Vanilla packs section coordinates into a 64-bit long as
//! `(x:22, z:22, y:20)` with the layout
//!
//!   bits  0..19  -> y  (signed 20-bit, two's-complement)
//!   bits 20..41  -> z  (signed 22-bit, two's-complement)
//!   bits 42..63  -> x  (signed 22-bit, two's-complement)
//!
//! BlockPos uses `(x:26, z:26, y:12)`:
//!
//!   bits  0..11  -> y  (signed 12-bit)
//!   bits 12..37  -> z  (signed 26-bit)
//!   bits 38..63  -> x  (signed 26-bit)

use crate::lighting::data_layer::SECTION_SIZE;

// ---- Section packing (`SectionPos.asLong`, `x`, `y`, `z`) ----

const SECTION_PACKED_X_LENGTH: u64 = 22;
const SECTION_PACKED_Y_LENGTH: u64 = 20;
const SECTION_PACKED_Z_LENGTH: u64 = 22;
const SECTION_X_MASK: u64 = (1u64 << SECTION_PACKED_X_LENGTH) - 1;
const SECTION_Y_MASK: u64 = (1u64 << SECTION_PACKED_Y_LENGTH) - 1;
const SECTION_Z_MASK: u64 = (1u64 << SECTION_PACKED_Z_LENGTH) - 1;
const SECTION_Y_OFFSET: u64 = 0;
const SECTION_Z_OFFSET: u64 = SECTION_PACKED_Y_LENGTH;
const SECTION_X_OFFSET: u64 = SECTION_PACKED_Y_LENGTH + SECTION_PACKED_Z_LENGTH;

/// Pack a `(section_x, section_y, section_z)` into the Java-compatible 64-bit
/// section node identifier.
pub const fn section_pos_as_long(x: i32, y: i32, z: i32) -> i64 {
    let xu = (x as i64) as u64;
    let yu = (y as i64) as u64;
    let zu = (z as i64) as u64;
    let packed = ((xu & SECTION_X_MASK) << SECTION_X_OFFSET)
        | ((yu & SECTION_Y_MASK) << SECTION_Y_OFFSET)
        | ((zu & SECTION_Z_MASK) << SECTION_Z_OFFSET);
    packed as i64
}

#[inline]
pub const fn section_pos_x(section_node: i64) -> i32 {
    // Sign-extend the 22-bit field. Java does `(int)(node << 0 >> 42)`.
    ((section_node << (64 - SECTION_X_OFFSET - SECTION_PACKED_X_LENGTH))
        >> (64 - SECTION_PACKED_X_LENGTH)) as i32
}

#[inline]
pub const fn section_pos_y(section_node: i64) -> i32 {
    ((section_node << (64 - SECTION_PACKED_Y_LENGTH)) >> (64 - SECTION_PACKED_Y_LENGTH)) as i32
}

#[inline]
pub const fn section_pos_z(section_node: i64) -> i32 {
    ((section_node << (64 - SECTION_Z_OFFSET - SECTION_PACKED_Z_LENGTH))
        >> (64 - SECTION_PACKED_Z_LENGTH)) as i32
}

/// Java: `SectionPos.offset(long node, int dx, int dy, int dz)`.
#[inline]
pub fn section_pos_offset(section_node: i64, dx: i32, dy: i32, dz: i32) -> i64 {
    section_pos_as_long(
        section_pos_x(section_node) + dx,
        section_pos_y(section_node) + dy,
        section_pos_z(section_node) + dz,
    )
}

/// Java: `SectionPos.getZeroNode(long sectionNode)` — clears the Y field so the
/// result identifies the column.
#[inline]
pub const fn section_pos_get_zero_node(section_node: i64) -> i64 {
    // Java mask: -1048576L (i.e. ~PACKED_Y_MASK).
    (section_node as u64 & !SECTION_Y_MASK) as i64
}

/// Java: `SectionPos.getZeroNode(int x, int z)`.
#[inline]
pub const fn section_pos_zero_node(x: i32, z: i32) -> i64 {
    section_pos_get_zero_node(section_pos_as_long(x, 0, z))
}

/// Java: `SectionPos.blockToSectionCoord(int)` — arithmetic shift by 4.
#[inline]
pub const fn block_to_section_coord(block: i32) -> i32 {
    block >> 4
}

/// Java: `SectionPos.sectionToBlockCoord(int)`.
#[inline]
pub const fn section_to_block_coord(section: i32) -> i32 {
    section << 4
}

/// Java: `SectionPos.sectionRelative(int)` — `block & 15`.
#[inline]
pub const fn section_relative(block: i32) -> i32 {
    block & (SECTION_SIZE as i32 - 1)
}

// ---- Block packing (`BlockPos.asLong`, `getX`, `getY`, `getZ`) ----

const BLOCK_PACKED_HORIZONTAL_LENGTH: u64 = 26;
const BLOCK_PACKED_Y_LENGTH: u64 = 64 - 2 * BLOCK_PACKED_HORIZONTAL_LENGTH;
const BLOCK_X_MASK: u64 = (1u64 << BLOCK_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_Y_MASK: u64 = (1u64 << BLOCK_PACKED_Y_LENGTH) - 1;
const BLOCK_Z_MASK: u64 = (1u64 << BLOCK_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_Z_OFFSET: u64 = BLOCK_PACKED_Y_LENGTH;
const BLOCK_X_OFFSET: u64 = BLOCK_PACKED_Y_LENGTH + BLOCK_PACKED_HORIZONTAL_LENGTH;

#[inline]
pub const fn block_pos_as_long(x: i32, y: i32, z: i32) -> i64 {
    let xu = (x as i64) as u64;
    let yu = (y as i64) as u64;
    let zu = (z as i64) as u64;
    let packed = ((xu & BLOCK_X_MASK) << BLOCK_X_OFFSET)
        | (yu & BLOCK_Y_MASK)
        | ((zu & BLOCK_Z_MASK) << BLOCK_Z_OFFSET);
    packed as i64
}

#[inline]
pub const fn block_pos_x(block_node: i64) -> i32 {
    ((block_node << (64 - BLOCK_X_OFFSET - BLOCK_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_PACKED_HORIZONTAL_LENGTH)) as i32
}

#[inline]
pub const fn block_pos_y(block_node: i64) -> i32 {
    ((block_node << (64 - BLOCK_PACKED_Y_LENGTH)) >> (64 - BLOCK_PACKED_Y_LENGTH)) as i32
}

#[inline]
pub const fn block_pos_z(block_node: i64) -> i32 {
    ((block_node << (64 - BLOCK_Z_OFFSET - BLOCK_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_PACKED_HORIZONTAL_LENGTH)) as i32
}

/// Java: `BlockPos.offset(long, int, int, int)`.
#[inline]
pub fn block_pos_offset(block_node: i64, dx: i32, dy: i32, dz: i32) -> i64 {
    block_pos_as_long(
        block_pos_x(block_node) + dx,
        block_pos_y(block_node) + dy,
        block_pos_z(block_node) + dz,
    )
}

/// Java: `SectionPos.blockToSection(long)` — translate a packed block node into
/// its enclosing packed section node.
#[inline]
pub fn block_to_section(block_node: i64) -> i64 {
    section_pos_as_long(
        block_to_section_coord(block_pos_x(block_node)),
        block_to_section_coord(block_pos_y(block_node)),
        block_to_section_coord(block_pos_z(block_node)),
    )
}
