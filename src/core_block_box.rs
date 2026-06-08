use crate::core_misc::Vec3iModel;
use crate::core_orientation::{CoreAxisModel, CoreDirectionModel};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AabbModel {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

impl AabbModel {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockBoxModel {
    min: Vec3iModel,
    max: Vec3iModel,
}

impl BlockBoxModel {
    pub const STREAM_CODEC_FIELDS: [&'static str; 2] = ["min", "max"];

    pub fn new(min: Vec3iModel, max: Vec3iModel) -> Self {
        Self {
            min: block_pos_min(min, max),
            max: block_pos_max(min, max),
        }
    }

    pub fn of(pos: Vec3iModel) -> Self {
        Self::new(pos, pos)
    }

    pub fn of_two(a: Vec3iModel, b: Vec3iModel) -> Self {
        Self::new(a, b)
    }

    pub fn min(self) -> Vec3iModel {
        self.min
    }

    pub fn max(self) -> Vec3iModel {
        self.max
    }

    pub fn include(self, pos: Vec3iModel) -> Self {
        Self::new(block_pos_min(self.min, pos), block_pos_max(self.max, pos))
    }

    pub fn is_block(self) -> bool {
        self.min == self.max
    }

    pub fn contains(self, pos: Vec3iModel) -> bool {
        pos.get(CoreAxisModel::X) >= self.min.get(CoreAxisModel::X)
            && pos.get(CoreAxisModel::Y) >= self.min.get(CoreAxisModel::Y)
            && pos.get(CoreAxisModel::Z) >= self.min.get(CoreAxisModel::Z)
            && pos.get(CoreAxisModel::X) <= self.max.get(CoreAxisModel::X)
            && pos.get(CoreAxisModel::Y) <= self.max.get(CoreAxisModel::Y)
            && pos.get(CoreAxisModel::Z) <= self.max.get(CoreAxisModel::Z)
    }

    pub fn aabb(self) -> AabbModel {
        AabbModel::new(
            self.min.get(CoreAxisModel::X) as f64,
            self.min.get(CoreAxisModel::Y) as f64,
            self.min.get(CoreAxisModel::Z) as f64,
            self.max.get(CoreAxisModel::X) as f64 + 1.0,
            self.max.get(CoreAxisModel::Y) as f64 + 1.0,
            self.max.get(CoreAxisModel::Z) as f64 + 1.0,
        )
    }

    pub fn iter(self) -> Vec<Vec3iModel> {
        let mut positions = Vec::new();
        for z in self.min.get(CoreAxisModel::Z)..=self.max.get(CoreAxisModel::Z) {
            for y in self.min.get(CoreAxisModel::Y)..=self.max.get(CoreAxisModel::Y) {
                for x in self.min.get(CoreAxisModel::X)..=self.max.get(CoreAxisModel::X) {
                    positions.push(Vec3iModel::new(x, y, z));
                }
            }
        }
        positions
    }

    pub fn size_x(self) -> i32 {
        self.max.get(CoreAxisModel::X) - self.min.get(CoreAxisModel::X) + 1
    }

    pub fn size_y(self) -> i32 {
        self.max.get(CoreAxisModel::Y) - self.min.get(CoreAxisModel::Y) + 1
    }

    pub fn size_z(self) -> i32 {
        self.max.get(CoreAxisModel::Z) - self.min.get(CoreAxisModel::Z) + 1
    }

    pub fn extend(self, direction: CoreDirectionModel, amount: i32) -> Self {
        if amount == 0 {
            return self;
        }
        let moved_max = self.max.relative(direction, amount);
        let moved_min = self.min.relative(direction, amount);
        if axis_direction(direction) == AxisDirectionModel::Positive {
            Self::of_two(self.min, block_pos_max(self.min, moved_max))
        } else {
            Self::of_two(block_pos_min(moved_min, self.max), self.max)
        }
    }

    pub fn move_toward(self, direction: CoreDirectionModel, amount: i32) -> Self {
        if amount == 0 {
            self
        } else {
            Self::new(
                self.min.relative(direction, amount),
                self.max.relative(direction, amount),
            )
        }
    }

    pub fn offset(self, offset: Vec3iModel) -> Self {
        Self::new(self.min.offset_vec(offset), self.max.offset_vec(offset))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisDirectionModel {
    Positive,
    Negative,
}

fn axis_direction(direction: CoreDirectionModel) -> AxisDirectionModel {
    match direction {
        CoreDirectionModel::Up | CoreDirectionModel::South | CoreDirectionModel::East => {
            AxisDirectionModel::Positive
        }
        CoreDirectionModel::Down | CoreDirectionModel::North | CoreDirectionModel::West => {
            AxisDirectionModel::Negative
        }
    }
}

fn block_pos_min(a: Vec3iModel, b: Vec3iModel) -> Vec3iModel {
    Vec3iModel::new(
        a.get(CoreAxisModel::X).min(b.get(CoreAxisModel::X)),
        a.get(CoreAxisModel::Y).min(b.get(CoreAxisModel::Y)),
        a.get(CoreAxisModel::Z).min(b.get(CoreAxisModel::Z)),
    )
}

fn block_pos_max(a: Vec3iModel, b: Vec3iModel) -> Vec3iModel {
    Vec3iModel::new(
        a.get(CoreAxisModel::X).max(b.get(CoreAxisModel::X)),
        a.get(CoreAxisModel::Y).max(b.get(CoreAxisModel::Y)),
        a.get(CoreAxisModel::Z).max(b.get(CoreAxisModel::Z)),
    )
}

trait Vec3iOffsetModel {
    fn offset_vec(self, offset: Vec3iModel) -> Self;
}

impl Vec3iOffsetModel for Vec3iModel {
    fn offset_vec(self, offset: Vec3iModel) -> Self {
        self.offset(
            offset.get(CoreAxisModel::X),
            offset.get(CoreAxisModel::Y),
            offset.get(CoreAxisModel::Z),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i32, y: i32, z: i32) -> Vec3iModel {
        Vec3iModel::new(x, y, z)
    }

    #[test]
    fn block_box_constructor_and_factories_normalize_min_max() {
        let box_ = BlockBoxModel::new(pos(5, -1, 7), pos(2, 4, 3));
        assert_eq!(BlockBoxModel::STREAM_CODEC_FIELDS, ["min", "max"]);
        assert_eq!(box_.min(), pos(2, -1, 3));
        assert_eq!(box_.max(), pos(5, 4, 7));
        assert_eq!(BlockBoxModel::of(pos(1, 2, 3)).min(), pos(1, 2, 3));
        assert_eq!(
            BlockBoxModel::of_two(pos(10, 0, -2), pos(9, 3, -4)),
            BlockBoxModel::new(pos(9, 0, -4), pos(10, 3, -2))
        );
    }

    #[test]
    fn block_box_includes_blocks_contains_and_reports_full_block_aabb() {
        let box_ = BlockBoxModel::of(pos(1, 2, 3)).include(pos(-2, 5, 4));
        assert!(!box_.is_block());
        assert!(BlockBoxModel::of(pos(1, 2, 3)).is_block());
        assert_eq!(box_.min(), pos(-2, 2, 3));
        assert_eq!(box_.max(), pos(1, 5, 4));
        assert!(box_.contains(pos(-2, 2, 3)));
        assert!(box_.contains(pos(1, 5, 4)));
        assert!(box_.contains(pos(0, 3, 3)));
        assert!(!box_.contains(pos(-3, 3, 3)));
        assert!(!box_.contains(pos(0, 6, 3)));
        assert!(!box_.contains(pos(0, 3, 5)));
        assert_eq!(box_.aabb(), AabbModel::new(-2.0, 2.0, 3.0, 2.0, 6.0, 5.0));
    }

    #[test]
    fn block_box_iterates_like_block_pos_between_closed() {
        let box_ = BlockBoxModel::new(pos(0, 0, 0), pos(1, 1, 1));
        assert_eq!(
            box_.iter(),
            vec![
                pos(0, 0, 0),
                pos(1, 0, 0),
                pos(0, 1, 0),
                pos(1, 1, 0),
                pos(0, 0, 1),
                pos(1, 0, 1),
                pos(0, 1, 1),
                pos(1, 1, 1),
            ]
        );
        assert_eq!(box_.size_x(), 2);
        assert_eq!(box_.size_y(), 2);
        assert_eq!(box_.size_z(), 2);
    }

    #[test]
    fn block_box_extends_only_positive_or_negative_face_for_direction() {
        let box_ = BlockBoxModel::new(pos(0, 10, -3), pos(2, 12, -1));
        assert_eq!(box_.extend(CoreDirectionModel::East, 0), box_);
        assert_eq!(
            box_.extend(CoreDirectionModel::East, 3),
            BlockBoxModel::new(pos(0, 10, -3), pos(5, 12, -1))
        );
        assert_eq!(
            box_.extend(CoreDirectionModel::West, 3),
            BlockBoxModel::new(pos(-3, 10, -3), pos(2, 12, -1))
        );
        assert_eq!(
            box_.extend(CoreDirectionModel::Up, 2),
            BlockBoxModel::new(pos(0, 10, -3), pos(2, 14, -1))
        );
        assert_eq!(
            box_.extend(CoreDirectionModel::Down, 2),
            BlockBoxModel::new(pos(0, 8, -3), pos(2, 12, -1))
        );
        assert_eq!(
            box_.extend(CoreDirectionModel::South, 4),
            BlockBoxModel::new(pos(0, 10, -3), pos(2, 12, 3))
        );
        assert_eq!(
            box_.extend(CoreDirectionModel::North, 4),
            BlockBoxModel::new(pos(0, 10, -7), pos(2, 12, -1))
        );
    }

    #[test]
    fn block_box_move_and_offset_translate_both_corners() {
        let box_ = BlockBoxModel::new(pos(0, 1, 2), pos(3, 4, 5));
        assert_eq!(box_.move_toward(CoreDirectionModel::North, 0), box_);
        assert_eq!(
            box_.move_toward(CoreDirectionModel::North, 2),
            BlockBoxModel::new(pos(0, 1, 0), pos(3, 4, 3))
        );
        assert_eq!(
            box_.move_toward(CoreDirectionModel::Down, 3),
            BlockBoxModel::new(pos(0, -2, 2), pos(3, 1, 5))
        );
        assert_eq!(
            box_.offset(pos(-1, 5, 7)),
            BlockBoxModel::new(pos(-1, 6, 9), pos(2, 9, 12))
        );
    }
}
