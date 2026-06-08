use crate::core_misc::Vec3iModel;
use crate::core_orientation::{CoreDirectionModel, PositionModel, Vec3PositionModel};
use crate::lighting::positions::{
    block_pos_as_long, block_pos_x, block_pos_y, block_pos_z, block_to_section,
    block_to_section_coord, section_pos_as_long, section_pos_get_zero_node, section_pos_offset,
    section_pos_x, section_pos_y, section_pos_z, section_pos_zero_node, section_relative,
    section_to_block_coord,
};

pub const SECTION_BITS: i32 = 4;
pub const SECTION_SIZE: i32 = 16;
pub const SECTION_MASK: i32 = 15;
pub const SECTION_HALF_SIZE: i32 = 8;
pub const SECTION_MAX_INDEX: i32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPosModel {
    x: i32,
    z: i32,
}

impl ChunkPosModel {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn as_long(self) -> i64 {
        ((self.x as i64) & 0xffff_ffff) | (((self.z as i64) & 0xffff_ffff) << 32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl SectionPosModel {
    pub const STREAM_CODEC_KIND: &'static str = "long";

    pub fn of(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn of_block_pos(pos: Vec3iModel) -> Self {
        Self::of(
            block_to_section_coord(pos.get(crate::core_orientation::CoreAxisModel::X)),
            block_to_section_coord(pos.get(crate::core_orientation::CoreAxisModel::Y)),
            block_to_section_coord(pos.get(crate::core_orientation::CoreAxisModel::Z)),
        )
    }

    pub fn of_chunk_pos(pos: ChunkPosModel, section_y: i32) -> Self {
        Self::of(pos.x, section_y, pos.z)
    }

    pub fn of_position(pos: impl PositionModel) -> Self {
        Self::of(
            block_to_section_coord(floor_to_i32(pos.x())),
            block_to_section_coord(floor_to_i32(pos.y())),
            block_to_section_coord(floor_to_i32(pos.z())),
        )
    }

    pub fn of_long(section_node: i64) -> Self {
        Self::of(
            section_pos_x(section_node),
            section_pos_y(section_node),
            section_pos_z(section_node),
        )
    }

    pub fn offset_long(section_node: i64, direction: CoreDirectionModel) -> i64 {
        let (x, y, z) = direction.step();
        section_pos_offset(section_node, x, y, z)
    }

    pub fn offset_long_by(section_node: i64, x: i32, y: i32, z: i32) -> i64 {
        section_pos_offset(section_node, x, y, z)
    }

    pub fn pos_to_section_coord(pos: f64) -> i32 {
        block_to_section_coord(floor_to_i32(pos))
    }

    pub fn block_to_section_coord(block_coord: i32) -> i32 {
        block_to_section_coord(block_coord)
    }

    pub fn block_to_section_coord_f64(coord: f64) -> i32 {
        block_to_section_coord(floor_to_i32(coord))
    }

    pub fn section_relative(block_coord: i32) -> i32 {
        section_relative(block_coord)
    }

    pub fn section_relative_pos(pos: Vec3iModel) -> i16 {
        let x = section_relative(pos.get(crate::core_orientation::CoreAxisModel::X));
        let y = section_relative(pos.get(crate::core_orientation::CoreAxisModel::Y));
        let z = section_relative(pos.get(crate::core_orientation::CoreAxisModel::Z));
        ((x << 8) | (z << 4) | y) as i16
    }

    pub fn section_relative_x(relative: i16) -> i32 {
        ((relative as u16) >> 8 & 15) as i32
    }

    pub fn section_relative_y(relative: i16) -> i32 {
        ((relative as u16) & 15) as i32
    }

    pub fn section_relative_z(relative: i16) -> i32 {
        ((relative as u16) >> 4 & 15) as i32
    }

    pub fn section_to_block_coord(section_coord: i32) -> i32 {
        section_to_block_coord(section_coord)
    }

    pub fn section_to_block_coord_with_offset(section_coord: i32, offset: i32) -> i32 {
        section_to_block_coord(section_coord) + offset
    }

    pub fn block_to_section_long(block_node: i64) -> i64 {
        block_to_section(block_node)
    }

    pub fn zero_node(x: i32, z: i32) -> i64 {
        section_pos_zero_node(x, z)
    }

    pub fn zero_node_from_long(section_node: i64) -> i64 {
        section_pos_get_zero_node(section_node)
    }

    pub fn section_to_chunk(section_node: i64) -> i64 {
        ChunkPosModel::new(section_pos_x(section_node), section_pos_z(section_node)).as_long()
    }

    pub fn as_long_coords(x: i32, y: i32, z: i32) -> i64 {
        section_pos_as_long(x, y, z)
    }

    pub fn as_long(self) -> i64 {
        section_pos_as_long(self.x, self.y, self.z)
    }

    pub fn x_from_long(section_node: i64) -> i32 {
        section_pos_x(section_node)
    }

    pub fn y_from_long(section_node: i64) -> i32 {
        section_pos_y(section_node)
    }

    pub fn z_from_long(section_node: i64) -> i32 {
        section_pos_z(section_node)
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    pub fn z(self) -> i32 {
        self.z
    }

    pub fn min_block_x(self) -> i32 {
        section_to_block_coord(self.x)
    }

    pub fn min_block_y(self) -> i32 {
        section_to_block_coord(self.y)
    }

    pub fn min_block_z(self) -> i32 {
        section_to_block_coord(self.z)
    }

    pub fn max_block_x(self) -> i32 {
        section_to_block_coord(self.x) + SECTION_MASK
    }

    pub fn max_block_y(self) -> i32 {
        section_to_block_coord(self.y) + SECTION_MASK
    }

    pub fn max_block_z(self) -> i32 {
        section_to_block_coord(self.z) + SECTION_MASK
    }

    pub fn relative_to_block_x(self, relative: i16) -> i32 {
        self.min_block_x() + Self::section_relative_x(relative)
    }

    pub fn relative_to_block_y(self, relative: i16) -> i32 {
        self.min_block_y() + Self::section_relative_y(relative)
    }

    pub fn relative_to_block_z(self, relative: i16) -> i32 {
        self.min_block_z() + Self::section_relative_z(relative)
    }

    pub fn relative_to_block_pos(self, relative: i16) -> Vec3iModel {
        Vec3iModel::new(
            self.relative_to_block_x(relative),
            self.relative_to_block_y(relative),
            self.relative_to_block_z(relative),
        )
    }

    pub fn origin(self) -> Vec3iModel {
        Vec3iModel::new(self.min_block_x(), self.min_block_y(), self.min_block_z())
    }

    pub fn center(self) -> Vec3iModel {
        self.origin()
            .offset(SECTION_HALF_SIZE, SECTION_HALF_SIZE, SECTION_HALF_SIZE)
    }

    pub fn chunk(self) -> ChunkPosModel {
        ChunkPosModel::new(self.x, self.z)
    }

    pub fn offset(self, x: i32, y: i32, z: i32) -> Self {
        if x == 0 && y == 0 && z == 0 {
            self
        } else {
            Self::of(self.x + x, self.y + y, self.z + z)
        }
    }

    pub fn blocks_inside(self) -> Vec<Vec3iModel> {
        let mut blocks = Vec::with_capacity((SECTION_SIZE * SECTION_SIZE * SECTION_SIZE) as usize);
        for z in self.min_block_z()..=self.max_block_z() {
            for y in self.min_block_y()..=self.max_block_y() {
                for x in self.min_block_x()..=self.max_block_x() {
                    blocks.push(Vec3iModel::new(x, y, z));
                }
            }
        }
        blocks
    }

    pub fn cube(center: Self, radius: i32) -> Vec<Self> {
        Self::between_closed_stream(
            center.x - radius,
            center.y - radius,
            center.z - radius,
            center.x + radius,
            center.y + radius,
            center.z + radius,
        )
    }

    pub fn around_chunk(
        center: ChunkPosModel,
        radius: i32,
        min_section: i32,
        max_section: i32,
    ) -> Vec<Self> {
        Self::between_closed_stream(
            center.x - radius,
            min_section,
            center.z - radius,
            center.x + radius,
            max_section,
            center.z + radius,
        )
    }

    pub fn between_closed_stream(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        let mut sections = Vec::new();
        for z in min_z..=max_z {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    sections.push(Self::of(x, y, z));
                }
            }
        }
        sections
    }

    pub fn around_and_at_block_pos(block_x: i32, block_y: i32, block_z: i32) -> Vec<i64> {
        let min_section_x = block_to_section_coord(block_x - 1);
        let max_section_x = block_to_section_coord(block_x + 1);
        let min_section_y = block_to_section_coord(block_y - 1);
        let max_section_y = block_to_section_coord(block_y + 1);
        let min_section_z = block_to_section_coord(block_z - 1);
        let max_section_z = block_to_section_coord(block_z + 1);
        let mut sections = Vec::new();
        for x in min_section_x..=max_section_x {
            for y in min_section_y..=max_section_y {
                for z in min_section_z..=max_section_z {
                    sections.push(section_pos_as_long(x, y, z));
                }
            }
        }
        sections
    }
}

fn floor_to_i32(value: f64) -> i32 {
    value.floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_pos_constants_and_constructors_match_java() {
        assert_eq!(SECTION_BITS, 4);
        assert_eq!(SECTION_SIZE, 16);
        assert_eq!(SECTION_MASK, 15);
        assert_eq!(SECTION_HALF_SIZE, 8);
        assert_eq!(SECTION_MAX_INDEX, 15);
        assert_eq!(SectionPosModel::STREAM_CODEC_KIND, "long");

        assert_eq!(
            SectionPosModel::of(1, -2, 3),
            SectionPosModel { x: 1, y: -2, z: 3 }
        );
        assert_eq!(
            SectionPosModel::of_block_pos(Vec3iModel::new(31, -1, -16)),
            SectionPosModel::of(1, -1, -1)
        );
        assert_eq!(
            SectionPosModel::of_chunk_pos(ChunkPosModel::new(-3, 4), 9),
            SectionPosModel::of(-3, 9, 4)
        );
        assert_eq!(
            SectionPosModel::of_position(Vec3PositionModel {
                x: 32.9,
                y: -0.1,
                z: -16.0,
            }),
            SectionPosModel::of(2, -1, -1)
        );
    }

    #[test]
    fn section_pos_packing_offsets_and_chunk_conversion_match_java() {
        for (x, y, z) in [
            (0, 0, 0),
            (1, 2, 3),
            (-1, -2, -3),
            (1_000_000, -64, -1_000_000),
            (-2_000_000, 319, 2_000_000),
        ] {
            let packed = SectionPosModel::as_long_coords(x, y, z);
            assert_eq!(SectionPosModel::x_from_long(packed), x);
            assert_eq!(SectionPosModel::y_from_long(packed), y);
            assert_eq!(SectionPosModel::z_from_long(packed), z);
            assert_eq!(
                SectionPosModel::of_long(packed),
                SectionPosModel::of(x, y, z)
            );
            assert_eq!(SectionPosModel::of(x, y, z).as_long(), packed);
        }

        let packed = SectionPosModel::as_long_coords(4, -5, 6);
        assert_eq!(
            SectionPosModel::offset_long(packed, CoreDirectionModel::North),
            SectionPosModel::as_long_coords(4, -5, 5)
        );
        assert_eq!(
            SectionPosModel::offset_long_by(packed, -2, 3, 7),
            SectionPosModel::as_long_coords(2, -2, 13)
        );
        assert_eq!(
            SectionPosModel::block_to_section_long(block_pos_as_long(33, -65, -16)),
            SectionPosModel::as_long_coords(2, -5, -1)
        );
        assert_eq!(
            SectionPosModel::zero_node_from_long(SectionPosModel::as_long_coords(8, -9, 10)),
            SectionPosModel::as_long_coords(8, 0, 10)
        );
        assert_eq!(
            SectionPosModel::zero_node(8, 10),
            SectionPosModel::as_long_coords(8, 0, 10)
        );
        assert_eq!(
            SectionPosModel::section_to_chunk(SectionPosModel::as_long_coords(-3, 99, 7)),
            ChunkPosModel::new(-3, 7).as_long()
        );
    }

    #[test]
    fn section_pos_coordinate_and_relative_helpers_match_java() {
        assert_eq!(SectionPosModel::pos_to_section_coord(-0.1), -1);
        assert_eq!(SectionPosModel::block_to_section_coord(31), 1);
        assert_eq!(SectionPosModel::block_to_section_coord(-1), -1);
        assert_eq!(SectionPosModel::block_to_section_coord_f64(16.0), 1);
        assert_eq!(SectionPosModel::block_to_section_coord_f64(15.999), 0);
        assert_eq!(SectionPosModel::section_relative(31), 15);
        assert_eq!(SectionPosModel::section_relative(-1), 15);
        assert_eq!(SectionPosModel::section_to_block_coord(-3), -48);
        assert_eq!(
            SectionPosModel::section_to_block_coord_with_offset(-3, 15),
            -33
        );

        let relative = SectionPosModel::section_relative_pos(Vec3iModel::new(-1, 32, 17));
        assert_eq!(relative, 0x0f10);
        assert_eq!(SectionPosModel::section_relative_x(relative), 15);
        assert_eq!(SectionPosModel::section_relative_y(relative), 0);
        assert_eq!(SectionPosModel::section_relative_z(relative), 1);

        let section = SectionPosModel::of(-1, 2, 1);
        assert_eq!(section.relative_to_block_x(relative), -1);
        assert_eq!(section.relative_to_block_y(relative), 32);
        assert_eq!(section.relative_to_block_z(relative), 17);
        assert_eq!(
            section.relative_to_block_pos(relative),
            Vec3iModel::new(-1, 32, 17)
        );
    }

    #[test]
    fn section_pos_block_bounds_origin_center_offset_and_chunk_match_java() {
        let section = SectionPosModel::of(-2, 3, 4);
        assert_eq!(section.x(), -2);
        assert_eq!(section.y(), 3);
        assert_eq!(section.z(), 4);
        assert_eq!(section.min_block_x(), -32);
        assert_eq!(section.min_block_y(), 48);
        assert_eq!(section.min_block_z(), 64);
        assert_eq!(section.max_block_x(), -17);
        assert_eq!(section.max_block_y(), 63);
        assert_eq!(section.max_block_z(), 79);
        assert_eq!(section.origin(), Vec3iModel::new(-32, 48, 64));
        assert_eq!(section.center(), Vec3iModel::new(-24, 56, 72));
        assert_eq!(section.chunk(), ChunkPosModel::new(-2, 4));
        assert_eq!(section.offset(0, 0, 0), section);
        assert_eq!(section.offset(1, -2, 3), SectionPosModel::of(-1, 1, 7));
    }

    #[test]
    fn section_pos_stream_helpers_match_cursor_order() {
        assert_eq!(
            SectionPosModel::between_closed_stream(0, 0, 0, 1, 1, 1),
            vec![
                SectionPosModel::of(0, 0, 0),
                SectionPosModel::of(1, 0, 0),
                SectionPosModel::of(0, 1, 0),
                SectionPosModel::of(1, 1, 0),
                SectionPosModel::of(0, 0, 1),
                SectionPosModel::of(1, 0, 1),
                SectionPosModel::of(0, 1, 1),
                SectionPosModel::of(1, 1, 1),
            ]
        );
        assert_eq!(
            SectionPosModel::cube(SectionPosModel::of(2, 3, 4), 1).len(),
            27
        );
        assert_eq!(
            SectionPosModel::around_chunk(ChunkPosModel::new(5, -6), 1, -1, 0).len(),
            18
        );

        let blocks = SectionPosModel::of(1, -1, 0).blocks_inside();
        assert_eq!(blocks.len(), 4096);
        assert_eq!(blocks.first().copied(), Some(Vec3iModel::new(16, -16, 0)));
        assert_eq!(blocks.last().copied(), Some(Vec3iModel::new(31, -1, 15)));
    }

    #[test]
    fn section_pos_around_and_at_block_pos_matches_java_nested_loop() {
        assert_eq!(
            SectionPosModel::around_and_at_block_pos(7, 8, 9),
            vec![SectionPosModel::as_long_coords(0, 0, 0)]
        );

        let around_edge = SectionPosModel::around_and_at_block_pos(15, 16, -1);
        let decoded = around_edge
            .iter()
            .map(|packed| {
                (
                    section_pos_x(*packed),
                    section_pos_y(*packed),
                    section_pos_z(*packed),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            decoded,
            vec![
                (0, 0, -1),
                (0, 0, 0),
                (0, 1, -1),
                (0, 1, 0),
                (1, 0, -1),
                (1, 0, 0),
                (1, 1, -1),
                (1, 1, 0),
            ]
        );

        let block = block_pos_as_long(33, -65, -16);
        assert_eq!(block_pos_x(block), 33);
        assert_eq!(block_pos_y(block), -65);
        assert_eq!(block_pos_z(block), -16);
    }
}
