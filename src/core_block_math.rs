#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl DirectionModel {
    const VALUES: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    fn normal(self) -> Vec3Model {
        match self {
            Self::Down => Vec3Model::new(0.0, -1.0, 0.0),
            Self::Up => Vec3Model::new(0.0, 1.0, 0.0),
            Self::North => Vec3Model::new(0.0, 0.0, -1.0),
            Self::South => Vec3Model::new(0.0, 0.0, 1.0),
            Self::West => Vec3Model::new(-1.0, 0.0, 0.0),
            Self::East => Vec3Model::new(1.0, 0.0, 0.0),
        }
    }

    fn approximate_nearest(vec: Vec3Model) -> Self {
        let mut result = Self::North;
        let mut highest_dot = f32::MIN_POSITIVE;
        for direction in Self::VALUES {
            let normal = direction.normal();
            let dot = vec.x * normal.x + vec.y * normal.y + vec.z * normal.z;
            if dot > highest_dot {
                highest_dot = dot;
                result = direction;
            }
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3Model {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3Model {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Matrix4Model {
    rows: [[f32; 4]; 4],
}

impl Matrix4Model {
    fn identity() -> Self {
        Self {
            rows: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::identity();
        matrix.rows[0][3] = x;
        matrix.rows[1][3] = y;
        matrix.rows[2][3] = z;
        matrix
    }

    fn rotation_x(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            rows: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, -sin, 0.0],
                [0.0, sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    fn rotation_y(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            rows: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            rows: [
                [x, 0.0, 0.0, 0.0],
                [0.0, y, 0.0, 0.0],
                [0.0, 0.0, z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    fn mul(self, that: Self) -> Self {
        let mut rows = [[0.0; 4]; 4];
        for (row_index, row) in rows.iter_mut().enumerate() {
            for (column_index, cell) in row.iter_mut().enumerate() {
                *cell = (0..4)
                    .map(|index| self.rows[row_index][index] * that.rows[index][column_index])
                    .sum();
            }
        }
        Self { rows }
    }

    fn transform_position(self, vec: Vec3Model) -> Vec3Model {
        Vec3Model::new(
            self.rows[0][0] * vec.x
                + self.rows[0][1] * vec.y
                + self.rows[0][2] * vec.z
                + self.rows[0][3],
            self.rows[1][0] * vec.x
                + self.rows[1][1] * vec.y
                + self.rows[1][2] * vec.z
                + self.rows[1][3],
            self.rows[2][0] * vec.x
                + self.rows[2][1] * vec.y
                + self.rows[2][2] * vec.z
                + self.rows[2][3],
        )
    }

    fn transform_direction(self, vec: Vec3Model) -> Vec3Model {
        Vec3Model::new(
            self.rows[0][0] * vec.x + self.rows[0][1] * vec.y + self.rows[0][2] * vec.z,
            self.rows[1][0] * vec.x + self.rows[1][1] * vec.y + self.rows[1][2] * vec.z,
            self.rows[2][0] * vec.x + self.rows[2][1] * vec.y + self.rows[2][2] * vec.z,
        )
    }

    fn is_identity(self) -> bool {
        self.approx_eq(Self::identity(), 0.0)
    }

    fn approx_eq(self, that: Self, epsilon: f32) -> bool {
        self.rows.iter().zip(that.rows).all(|(left, right)| {
            left.iter()
                .zip(right)
                .all(|(a, b)| (*a - b).abs() <= epsilon)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TransformationModel {
    matrix: Matrix4Model,
}

impl TransformationModel {
    fn identity() -> Self {
        Self {
            matrix: Matrix4Model::identity(),
        }
    }

    fn from_matrix(matrix: Matrix4Model) -> Self {
        Self { matrix }
    }

    fn translation(x: f32, y: f32, z: f32) -> Self {
        Self::from_matrix(Matrix4Model::translation(x, y, z))
    }

    fn rotation_x(radians: f32) -> Self {
        Self::from_matrix(Matrix4Model::rotation_x(radians))
    }

    fn rotation_y(radians: f32) -> Self {
        Self::from_matrix(Matrix4Model::rotation_y(radians))
    }

    fn scale(x: f32, y: f32, z: f32) -> Self {
        Self::from_matrix(Matrix4Model::scale(x, y, z))
    }

    fn compose(self, that: Self) -> Self {
        Self::from_matrix(self.matrix.mul(that.matrix))
    }
}

fn block_center_to_corner(transform: TransformationModel) -> TransformationModel {
    TransformationModel::from_matrix(
        Matrix4Model::translation(0.5, 0.5, 0.5)
            .mul(transform.matrix)
            .mul(Matrix4Model::translation(-0.5, -0.5, -0.5)),
    )
}

fn block_corner_to_center(transform: TransformationModel) -> TransformationModel {
    TransformationModel::from_matrix(
        Matrix4Model::translation(-0.5, -0.5, -0.5)
            .mul(transform.matrix)
            .mul(Matrix4Model::translation(0.5, 0.5, 0.5)),
    )
}

fn vanilla_uv_transform_local_to_global(direction: DirectionModel) -> TransformationModel {
    match direction {
        DirectionModel::South => TransformationModel::identity(),
        DirectionModel::East => TransformationModel::rotation_y(std::f32::consts::FRAC_PI_2),
        DirectionModel::West => TransformationModel::rotation_y(-std::f32::consts::FRAC_PI_2),
        DirectionModel::North => TransformationModel::rotation_y(std::f32::consts::PI),
        DirectionModel::Up => TransformationModel::rotation_x(-std::f32::consts::FRAC_PI_2),
        DirectionModel::Down => TransformationModel::rotation_x(std::f32::consts::FRAC_PI_2),
    }
}

fn vanilla_uv_transform_global_to_local(direction: DirectionModel) -> TransformationModel {
    match direction {
        DirectionModel::South => TransformationModel::identity(),
        DirectionModel::East => TransformationModel::rotation_y(-std::f32::consts::FRAC_PI_2),
        DirectionModel::West => TransformationModel::rotation_y(std::f32::consts::FRAC_PI_2),
        DirectionModel::North => TransformationModel::rotation_y(-std::f32::consts::PI),
        DirectionModel::Up => TransformationModel::rotation_x(std::f32::consts::FRAC_PI_2),
        DirectionModel::Down => TransformationModel::rotation_x(-std::f32::consts::FRAC_PI_2),
    }
}

fn get_face_transformation(
    transformation: TransformationModel,
    original_side: DirectionModel,
) -> TransformationModel {
    if transformation.matrix.is_identity() {
        return transformation;
    }

    let face_action = transformation.compose(vanilla_uv_transform_local_to_global(original_side));
    let transformed_normal = face_action
        .matrix
        .transform_direction(Vec3Model::new(0.0, 0.0, 1.0));
    let new_side = DirectionModel::approximate_nearest(transformed_normal);
    vanilla_uv_transform_global_to_local(new_side).compose(face_action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vec3_approx_eq(actual: Vec3Model, expected: Vec3Model) {
        assert!(
            (actual.x - expected.x).abs() <= 0.000_001
                && (actual.y - expected.y).abs() <= 0.000_001
                && (actual.z - expected.z).abs() <= 0.000_001,
            "actual={actual:?} expected={expected:?}"
        );
    }

    fn assert_matrix_approx_eq(actual: Matrix4Model, expected: Matrix4Model) {
        assert!(
            actual.approx_eq(expected, 0.000_001),
            "actual={actual:?} expected={expected:?}"
        );
    }

    #[test]
    fn block_center_corner_translation_wrappers_match_java_multiplication_order() {
        let translation = TransformationModel::translation(2.0, -3.0, 4.0);
        assert_eq!(block_center_to_corner(translation), translation);
        assert_eq!(block_corner_to_center(translation), translation);

        let scale = TransformationModel::scale(2.0, 3.0, 4.0);
        let center_to_corner = block_center_to_corner(scale);
        let corner_to_center = block_corner_to_center(scale);

        assert_vec3_approx_eq(
            center_to_corner
                .matrix
                .transform_position(Vec3Model::new(0.0, 0.0, 0.0)),
            Vec3Model::new(-0.5, -1.0, -1.5),
        );
        assert_vec3_approx_eq(
            corner_to_center
                .matrix
                .transform_position(Vec3Model::new(0.0, 0.0, 0.0)),
            Vec3Model::new(0.5, 1.0, 1.5),
        );
    }

    #[test]
    fn vanilla_uv_local_to_global_maps_local_south_normal_to_each_java_face() {
        for direction in DirectionModel::VALUES {
            let transform = vanilla_uv_transform_local_to_global(direction);
            let normal = transform
                .matrix
                .transform_direction(Vec3Model::new(0.0, 0.0, 1.0));
            assert_eq!(DirectionModel::approximate_nearest(normal), direction);
        }
    }

    #[test]
    fn vanilla_uv_global_to_local_is_inverse_of_local_to_global() {
        for direction in DirectionModel::VALUES {
            let round_trip = vanilla_uv_transform_global_to_local(direction)
                .compose(vanilla_uv_transform_local_to_global(direction));
            assert_matrix_approx_eq(round_trip.matrix, Matrix4Model::identity());
        }
    }

    #[test]
    fn face_transformation_returns_original_identity_instance_semantics() {
        let identity = TransformationModel::identity();
        assert_eq!(
            get_face_transformation(identity, DirectionModel::West),
            identity
        );
    }

    #[test]
    fn face_transformation_rotates_side_through_model_transform_then_relocalizes() {
        let rotate_to_east = TransformationModel::rotation_y(std::f32::consts::FRAC_PI_2);
        let west_face = get_face_transformation(rotate_to_east, DirectionModel::West);
        assert_matrix_approx_eq(west_face.matrix, Matrix4Model::identity());

        let north_face = get_face_transformation(rotate_to_east, DirectionModel::North);
        assert_matrix_approx_eq(north_face.matrix, Matrix4Model::identity());

        let up_to_north = TransformationModel::rotation_x(std::f32::consts::FRAC_PI_2);
        let up_face = get_face_transformation(up_to_north, DirectionModel::Up);
        assert_matrix_approx_eq(up_face.matrix, Matrix4Model::identity());
    }

    #[test]
    fn face_transformation_preserves_non_rotational_model_changes_after_face_cleanup() {
        let scale = TransformationModel::scale(2.0, 3.0, 4.0);
        let transformed = get_face_transformation(scale, DirectionModel::South);
        assert_matrix_approx_eq(transformed.matrix, scale.matrix);

        let translated = TransformationModel::translation(1.0, 2.0, 3.0);
        let translated_face = get_face_transformation(translated, DirectionModel::East);
        assert_vec3_approx_eq(
            translated_face
                .matrix
                .transform_position(Vec3Model::new(0.0, 0.0, 0.0)),
            Vec3Model::new(-3.0, 2.0, 1.0),
        );
    }
}
