#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    create_gametest_batch, BlockPosModel, GameTestBatchModel, GameTestInfoStateModel, RotationModel,
};

pub const GAMETEST_RUNNER_SERVER_RUNTIME_TODO: &str = "gametest-runner-server-runtime";
pub const DEFAULT_GAMETESTS_PER_ROW: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestRunnerEvent {
    BatchStarting {
        index: i32,
        environment: String,
    },
    BatchFinished {
        index: i32,
        environment: String,
    },
    EnvironmentActivated {
        environment: String,
    },
    EnvironmentTornDown {
        environment: String,
    },
    TestAddedForRerun {
        original_id: String,
        copy_id: String,
    },
    TickerCleared,
    ForcedChunksCleared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureSpawnerModel {
    InPlace,
    NotSet,
}

impl StructureSpawnerModel {
    pub fn spawn_structure(self, test_info: &mut GameTestInfoStateModel) -> Option<String> {
        match self {
            Self::InPlace => {
                test_info.start_execution(1);
                Some(test_info.id.clone())
            }
            Self::NotSet => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestRunnerBuilderModel {
    pub batches: Vec<GameTestBatchModel>,
    pub batcher: &'static str,
    pub existing_structure_spawner: StructureSpawnerModel,
    pub new_structure_spawner: StructureSpawnerModel,
    pub halt_on_error: bool,
    pub clear_between_batches: bool,
}

impl GameTestRunnerBuilderModel {
    pub fn from_batches(batches: Vec<GameTestBatchModel>) -> Self {
        Self {
            batches,
            batcher: "GameTestBatchFactory.fromGameTestInfo",
            existing_structure_spawner: StructureSpawnerModel::InPlace,
            new_structure_spawner: StructureSpawnerModel::NotSet,
            halt_on_error: false,
            clear_between_batches: false,
        }
    }

    pub fn halt_on_error(mut self) -> Self {
        self.halt_on_error = true;
        self
    }

    pub fn clear_between_batches(mut self) -> Self {
        self.clear_between_batches = true;
        self
    }

    pub fn new_structure_spawner(mut self, spawner: StructureSpawnerModel) -> Self {
        self.new_structure_spawner = spawner;
        self
    }

    pub fn existing_structure_spawner(mut self, spawner: StructureSpawnerModel) -> Self {
        self.existing_structure_spawner = spawner;
        self
    }

    pub fn build(self) -> GameTestRunnerModel {
        GameTestRunnerModel::new(
            self.batches,
            self.existing_structure_spawner,
            self.new_structure_spawner,
            self.halt_on_error,
            self.clear_between_batches,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestRunnerModel {
    pub batches: Vec<GameTestBatchModel>,
    pub all_test_infos: Vec<String>,
    pub scheduled_for_rerun: Vec<String>,
    pub stopped: bool,
    pub current_environment: Option<String>,
    pub existing_structure_spawner: StructureSpawnerModel,
    pub new_structure_spawner: StructureSpawnerModel,
    pub halt_on_error: bool,
    pub clear_between_batches: bool,
    pub events: Vec<GameTestRunnerEvent>,
}

impl GameTestRunnerModel {
    pub fn new(
        batches: Vec<GameTestBatchModel>,
        existing_structure_spawner: StructureSpawnerModel,
        new_structure_spawner: StructureSpawnerModel,
        halt_on_error: bool,
        clear_between_batches: bool,
    ) -> Self {
        let all_test_infos = batches
            .iter()
            .flat_map(|batch| batch.game_test_infos.clone())
            .collect();
        Self {
            batches,
            all_test_infos,
            scheduled_for_rerun: Vec::new(),
            stopped: true,
            current_environment: None,
            existing_structure_spawner,
            new_structure_spawner,
            halt_on_error,
            clear_between_batches,
            events: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.stopped = false;
        self.run_batch(0);
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        self.end_current_environment();
    }

    pub fn rerun_test(&mut self, original: &GameTestInfoStateModel) {
        let copy = original.copy_reset();
        self.events.push(GameTestRunnerEvent::TestAddedForRerun {
            original_id: original.id.clone(),
            copy_id: copy.id.clone(),
        });
        self.all_test_infos.push(copy.id.clone());
        self.scheduled_for_rerun.push(copy.id);
        if self.stopped {
            self.run_scheduled_rerun_tests();
        }
    }

    pub fn run_batch(&mut self, batch_index: usize) {
        if batch_index >= self.batches.len() {
            self.end_current_environment();
            self.run_scheduled_rerun_tests();
            return;
        }

        if batch_index > 0 && self.clear_between_batches {
            self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
        }

        let batch = self.batches[batch_index].clone();
        self.end_current_environment();
        self.current_environment = Some(batch.environment.clone());
        self.events.push(GameTestRunnerEvent::EnvironmentActivated {
            environment: batch.environment.clone(),
        });
        self.events.push(GameTestRunnerEvent::BatchStarting {
            index: batch.index,
            environment: batch.environment,
        });
    }

    pub fn complete_batch(&mut self, batch_index: usize, failed: bool) {
        if let Some(batch) = self.batches.get(batch_index) {
            if failed && self.halt_on_error {
                self.end_current_environment();
                self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
                self.events.push(GameTestRunnerEvent::TickerCleared);
                self.stopped = true;
            } else {
                self.events.push(GameTestRunnerEvent::BatchFinished {
                    index: batch.index,
                    environment: batch.environment.clone(),
                });
                self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
                self.run_batch(batch_index + 1);
            }
        }
    }

    fn end_current_environment(&mut self) {
        if let Some(environment) = self.current_environment.take() {
            self.events
                .push(GameTestRunnerEvent::EnvironmentTornDown { environment });
        }
    }

    fn run_scheduled_rerun_tests(&mut self) {
        if self.scheduled_for_rerun.is_empty() {
            self.batches.clear();
            self.stopped = true;
        } else {
            let tests = std::mem::take(&mut self.scheduled_for_rerun);
            self.batches = vec![create_gametest_batch(0, tests, "minecraft:default")
                .expect("rerun batch must include copied tests")];
            self.stopped = false;
            self.run_batch(0);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureGridBoundsModel {
    pub min: BlockPosModel,
    pub max: BlockPosModel,
}

impl StructureGridBoundsModel {
    pub fn point(pos: BlockPosModel) -> Self {
        Self { min: pos, max: pos }
    }

    pub fn from_corner_and_size(
        north_west_corner: BlockPosModel,
        x_size: i32,
        z_size: i32,
    ) -> Self {
        Self {
            min: north_west_corner,
            max: BlockPosModel::new(
                north_west_corner.x() + x_size,
                north_west_corner.y(),
                north_west_corner.z() + z_size,
            ),
        }
    }

    pub fn minmax(self, other: Self) -> Self {
        Self {
            min: BlockPosModel::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
                self.min.z().min(other.min.z()),
            ),
            max: BlockPosModel::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
                self.max.z().max(other.max.z()),
            ),
        }
    }

    pub fn x_size(self) -> i32 {
        self.max.x() - self.min.x()
    }

    pub fn z_size(self) -> i32 {
        self.max.z() - self.min.z()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureGridSpawnerTestModel {
    pub id: String,
    pub prepared: bool,
    pub test_block_pos: Option<BlockPosModel>,
    pub structure_bounds: StructureGridBoundsModel,
    pub test_bounding_box: StructureGridBoundsModel,
    pub start_delay: Option<i32>,
}

impl StructureGridSpawnerTestModel {
    pub fn new(id: impl Into<String>, x_size: i32, z_size: i32) -> Self {
        let origin = BlockPosModel::ZERO;
        let bounds = StructureGridBoundsModel::from_corner_and_size(origin, x_size, z_size);
        Self {
            id: id.into(),
            prepared: true,
            test_block_pos: None,
            structure_bounds: bounds,
            test_bounding_box: bounds,
            start_delay: None,
        }
    }

    pub fn unprepared(mut self) -> Self {
        self.prepared = false;
        self
    }

    fn set_test_block_pos(&mut self, pos: BlockPosModel) {
        self.test_block_pos = Some(pos);
        let x_size = self.structure_bounds.x_size();
        let z_size = self.structure_bounds.z_size();
        self.structure_bounds = StructureGridBoundsModel::from_corner_and_size(pos, x_size, z_size);
        self.test_bounding_box = self.structure_bounds;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureGridSpawnerModel {
    pub tests_per_row: i32,
    pub current_row_count: i32,
    pub row_bounds: StructureGridBoundsModel,
    pub next_test_north_west_corner: BlockPosModel,
    pub first_test_north_west_corner: BlockPosModel,
    pub clear_on_batch: bool,
    pub max_x: f32,
    pub tests_in_last_batch: Vec<StructureGridSpawnerTestModel>,
    pub cleared_bounds: Vec<StructureGridBoundsModel>,
}

impl StructureGridSpawnerModel {
    pub const SPACE_BETWEEN_COLUMNS: i32 = 5;
    pub const SPACE_BETWEEN_ROWS: i32 = 6;

    pub fn new(
        first_test_north_west_corner: BlockPosModel,
        tests_per_row: i32,
        clear_on_batch: bool,
    ) -> Self {
        Self {
            tests_per_row,
            current_row_count: 0,
            row_bounds: StructureGridBoundsModel::point(first_test_north_west_corner),
            next_test_north_west_corner: first_test_north_west_corner,
            first_test_north_west_corner,
            clear_on_batch,
            max_x: -1.0,
            tests_in_last_batch: Vec::new(),
            cleared_bounds: Vec::new(),
        }
    }

    pub fn on_batch_start(&mut self) {
        if self.clear_on_batch {
            self.cleared_bounds.extend(
                self.tests_in_last_batch
                    .iter()
                    .map(|test| test.test_bounding_box),
            );
            self.tests_in_last_batch.clear();
            self.row_bounds = StructureGridBoundsModel::point(self.first_test_north_west_corner);
            self.next_test_north_west_corner = self.first_test_north_west_corner;
        }
    }

    pub fn spawn_structure(
        &mut self,
        test_info: &mut StructureGridSpawnerTestModel,
    ) -> Option<String> {
        let north_west_corner = self.next_test_north_west_corner;
        test_info.set_test_block_pos(north_west_corner);
        if !test_info.prepared {
            return None;
        }

        test_info.start_delay = Some(1);
        let structure_bounds = test_info.structure_bounds;
        self.row_bounds = self.row_bounds.minmax(structure_bounds);
        self.next_test_north_west_corner = BlockPosModel::new(
            self.next_test_north_west_corner.x()
                + structure_bounds.x_size()
                + Self::SPACE_BETWEEN_COLUMNS,
            self.next_test_north_west_corner.y(),
            self.next_test_north_west_corner.z(),
        );
        if self.next_test_north_west_corner.x() as f32 > self.max_x {
            self.max_x = self.next_test_north_west_corner.x() as f32;
        }

        self.current_row_count += 1;
        if self.current_row_count >= self.tests_per_row {
            self.current_row_count = 0;
            self.next_test_north_west_corner = BlockPosModel::new(
                self.first_test_north_west_corner.x(),
                self.next_test_north_west_corner.y(),
                self.next_test_north_west_corner.z()
                    + self.row_bounds.z_size()
                    + Self::SPACE_BETWEEN_ROWS,
            );
            self.row_bounds = StructureGridBoundsModel::point(self.next_test_north_west_corner);
        }

        self.tests_in_last_batch.push(test_info.clone());
        Some(test_info.id.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureUtilsBoxModel {
    pub min: BlockPosModel,
    pub max: BlockPosModel,
}

impl StructureUtilsBoxModel {
    pub fn from_corners(a: BlockPosModel, b: BlockPosModel) -> Self {
        Self {
            min: BlockPosModel::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z())),
            max: BlockPosModel::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z())),
        }
    }

    pub fn move_by(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            min: BlockPosModel::new(self.min.x() + dx, self.min.y() + dy, self.min.z() + dz),
            max: BlockPosModel::new(self.max.x() + dx, self.max.y() + dy, self.max.z() + dz),
        }
    }

    pub fn contains(self, pos: BlockPosModel) -> bool {
        (self.min.x()..=self.max.x()).contains(&pos.x())
            && (self.min.y()..=self.max.y()).contains(&pos.y())
            && (self.min.z()..=self.max.z()).contains(&pos.z())
    }

    pub fn positions(self) -> Vec<BlockPosModel> {
        let mut positions = Vec::new();
        for x in self.min.x()..=self.max.x() {
            for y in self.min.y()..=self.max.y() {
                for z in self.min.z()..=self.max.z() {
                    positions.push(BlockPosModel::new(x, y, z));
                }
            }
        }
        positions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureUtilsCreatedTestModel {
    pub id: String,
    pub structure_pos: BlockPosModel,
    pub size: BlockPosModel,
    pub rotation: RotationModel,
    pub status: &'static str,
    pub ignores_entities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureUtilsTestBlockModel {
    pub pos: BlockPosModel,
    pub structure_bounding_box: StructureUtilsBoxModel,
    pub structure_bounds_hit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureUtilsLevelModel {
    pub placed_blocks: Vec<(BlockPosModel, &'static str)>,
    pub neighbor_updates: Vec<BlockPosModel>,
    pub cleared_tick_areas: Vec<StructureUtilsBoxModel>,
    pub cleared_block_event_areas: Vec<StructureUtilsBoxModel>,
    pub discarded_entity_count: usize,
    pub test_blocks: Vec<StructureUtilsTestBlockModel>,
    pub created_tests: Vec<StructureUtilsCreatedTestModel>,
}

pub fn structure_utils_rotation_for_steps(rotation_steps: i32) -> Result<RotationModel, String> {
    match rotation_steps {
        0 => Ok(RotationModel::None),
        1 => Ok(RotationModel::Clockwise90),
        2 => Ok(RotationModel::Clockwise180),
        3 => Ok(RotationModel::Counterclockwise90),
        _ => Err(format!(
            "rotationSteps must be a value from 0-3. Got value {rotation_steps}"
        )),
    }
}

pub fn structure_utils_steps_for_rotation(rotation: RotationModel) -> i32 {
    match rotation {
        RotationModel::None => 0,
        RotationModel::Clockwise90 => 1,
        RotationModel::Clockwise180 => 2,
        RotationModel::Counterclockwise90 => 3,
    }
}

pub fn structure_utils_transformed_far_corner(
    structure_position: BlockPosModel,
    size: BlockPosModel,
    rotation: RotationModel,
) -> BlockPosModel {
    let far_corner = BlockPosModel::new(
        structure_position.x() + size.x() - 1,
        structure_position.y() + size.y() - 1,
        structure_position.z() + size.z() - 1,
    );
    structure_utils_transform(far_corner, rotation, structure_position)
}

pub fn structure_utils_structure_bounding_box(
    north_west_corner: BlockPosModel,
    size: BlockPosModel,
    rotation: RotationModel,
) -> StructureUtilsBoxModel {
    let far_corner = structure_utils_transformed_far_corner(north_west_corner, size, rotation);
    let bounding_box = StructureUtilsBoxModel::from_corners(north_west_corner, far_corner);
    let current_north_west_x = bounding_box.min.x().min(bounding_box.max.x());
    let current_north_west_z = bounding_box.min.z().min(bounding_box.max.z());
    bounding_box.move_by(
        north_west_corner.x() - current_north_west_x,
        0,
        north_west_corner.z() - current_north_west_z,
    )
}

pub fn structure_utils_create_new_empty_test(
    level: &mut StructureUtilsLevelModel,
    id: impl Into<String>,
    structure_pos: BlockPosModel,
    size: BlockPosModel,
    rotation: RotationModel,
) -> StructureUtilsCreatedTestModel {
    let structure_position =
        BlockPosModel::new(structure_pos.x(), structure_pos.y() + 1, structure_pos.z());
    let structure_bounding_box =
        structure_utils_structure_bounding_box(structure_position, size, rotation);
    structure_utils_clear_space_for_structure(level, structure_bounding_box);
    level
        .placed_blocks
        .push((structure_pos, "test_instance_block"));
    let test = StructureUtilsCreatedTestModel {
        id: id.into(),
        structure_pos,
        size,
        rotation,
        status: "CLEARED",
        ignores_entities: false,
    };
    level.created_tests.push(test.clone());
    test
}

pub fn structure_utils_clear_space_for_structure(
    level: &mut StructureUtilsLevelModel,
    structure_bounding_box: StructureUtilsBoxModel,
) {
    let ground_height = structure_bounding_box.min.y() - 1;
    for pos in structure_bounding_box.positions() {
        let block = if pos.y() < ground_height {
            "stone"
        } else {
            "air"
        };
        level.placed_blocks.push((pos, block));
        level.neighbor_updates.push(pos);
    }
    level.cleared_tick_areas.push(structure_bounding_box);
    level.cleared_block_event_areas.push(structure_bounding_box);
    level.discarded_entity_count += 1;
}

pub fn structure_utils_find_test_blocks(
    center_pos: BlockPosModel,
    search_radius: i32,
    level: &StructureUtilsLevelModel,
) -> Vec<BlockPosModel> {
    level
        .test_blocks
        .iter()
        .filter(|test| manhattan(test.pos, center_pos) <= search_radius)
        .map(|test| test.pos)
        .collect()
}

pub fn structure_utils_find_test_containing_pos(
    pos: BlockPosModel,
    search_radius: i32,
    level: &StructureUtilsLevelModel,
) -> Option<BlockPosModel> {
    level
        .test_blocks
        .iter()
        .filter(|test| manhattan(test.pos, pos) <= search_radius)
        .find(|test| test.structure_bounding_box.contains(pos))
        .map(|test| test.pos)
}

pub fn structure_utils_find_nearest_test(
    relative_to_pos: BlockPosModel,
    search_radius: i32,
    level: &StructureUtilsLevelModel,
) -> Option<BlockPosModel> {
    level
        .test_blocks
        .iter()
        .filter(|test| manhattan(test.pos, relative_to_pos) <= search_radius)
        .min_by_key(|test| manhattan(test.pos, relative_to_pos))
        .map(|test| test.pos)
}

pub fn structure_utils_looked_at_test_pos(
    pos: BlockPosModel,
    level: &StructureUtilsLevelModel,
) -> Option<BlockPosModel> {
    level
        .test_blocks
        .iter()
        .filter(|test| manhattan(test.pos, pos) <= 250)
        .filter(|test| test.structure_bounds_hit)
        .min_by_key(|test| squared_distance(test.pos, pos))
        .map(|test| test.pos)
}

fn structure_utils_transform(
    pos: BlockPosModel,
    rotation: RotationModel,
    pivot: BlockPosModel,
) -> BlockPosModel {
    let rel_x = pos.x() - pivot.x();
    let rel_z = pos.z() - pivot.z();
    match rotation {
        RotationModel::None => pos,
        RotationModel::Clockwise90 => {
            BlockPosModel::new(pivot.x() - rel_z, pos.y(), pivot.z() + rel_x)
        }
        RotationModel::Clockwise180 => {
            BlockPosModel::new(pivot.x() - rel_x, pos.y(), pivot.z() - rel_z)
        }
        RotationModel::Counterclockwise90 => {
            BlockPosModel::new(pivot.x() + rel_z, pos.y(), pivot.z() - rel_x)
        }
    }
}

fn manhattan(a: BlockPosModel, b: BlockPosModel) -> i32 {
    (a.x() - b.x()).abs() + (a.y() - b.y()).abs() + (a.z() - b.z()).abs()
}

fn squared_distance(a: BlockPosModel, b: BlockPosModel) -> i32 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let dz = a.z() - b.z();
    dx * dx + dy * dy + dz * dz
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFinderSourceModel {
    pub id: String,
    pub position: BlockPosModel,
    pub level: StructureUtilsLevelModel,
    pub failed_tests: Vec<TestFinderTestModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFinderTestModel {
    pub id: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFinderModel {
    pub source_id: String,
    pub tests: Vec<TestFinderTestModel>,
    pub test_positions: Vec<BlockPosModel>,
}

impl TestFinderModel {
    pub fn find_tests(&self) -> Vec<TestFinderTestModel> {
        self.tests.clone()
    }

    pub fn find_test_pos(&self) -> Vec<BlockPosModel> {
        self.test_positions.clone()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TestFinderBuilderModel {
    copies: Option<usize>,
}

impl TestFinderBuilderModel {
    pub fn create_multiple_copies(self, amount: usize) -> Self {
        Self {
            copies: Some(amount),
        }
    }

    pub fn radius(self, source: &TestFinderSourceModel, radius: i32) -> TestFinderModel {
        self.build(
            source,
            Vec::new(),
            structure_utils_find_test_blocks(source.position, radius, &source.level),
        )
    }

    pub fn nearest(self, source: &TestFinderSourceModel) -> TestFinderModel {
        self.build(
            source,
            Vec::new(),
            structure_utils_find_nearest_test(source.position, 15, &source.level)
                .into_iter()
                .collect(),
        )
    }

    pub fn all_nearby(self, source: &TestFinderSourceModel) -> TestFinderModel {
        self.build(
            source,
            Vec::new(),
            structure_utils_find_test_blocks(source.position, 250, &source.level),
        )
    }

    pub fn looked_at(self, source: &TestFinderSourceModel) -> TestFinderModel {
        self.build(
            source,
            Vec::new(),
            structure_utils_looked_at_test_pos(source.position, &source.level)
                .into_iter()
                .collect(),
        )
    }

    pub fn failed_tests(
        self,
        source: &TestFinderSourceModel,
        only_required_tests: bool,
    ) -> TestFinderModel {
        self.build(
            source,
            source
                .failed_tests
                .iter()
                .filter(|test| !only_required_tests || test.required)
                .cloned()
                .collect(),
            Vec::new(),
        )
    }

    pub fn by_resource_selection(
        self,
        source: &TestFinderSourceModel,
        holders: Vec<TestFinderTestModel>,
    ) -> TestFinderModel {
        self.build(source, holders, Vec::new())
    }

    fn build(
        self,
        source: &TestFinderSourceModel,
        tests: Vec<TestFinderTestModel>,
        test_positions: Vec<BlockPosModel>,
    ) -> TestFinderModel {
        TestFinderModel {
            source_id: source.id.clone(),
            tests: copy_items(tests, self.copies),
            test_positions: copy_items(test_positions, self.copies),
        }
    }
}

pub fn test_finder_builder() -> TestFinderBuilderModel {
    TestFinderBuilderModel::default()
}

fn copy_items<T: Clone>(items: Vec<T>, copies: Option<usize>) -> Vec<T> {
    if let Some(amount) = copies {
        let source = items;
        let mut copied = Vec::with_capacity(source.len() * amount);
        for _ in 0..amount {
            copied.extend(source.iter().cloned());
        }
        copied
    } else {
        items
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests_structure_utils {
    use super::*;

    const STRUCTURE_UTILS_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/StructureUtils.java"
    );

    #[test]
    fn structure_utils_matches_java_source_shape() {
        assert_eq!(STRUCTURE_UTILS_JAVA.lines().count(), 149);
        for sentinel in [
            "public static final int DEFAULT_Y_SEARCH_RADIUS = 10;",
            "public static @Nullable Path testStructuresTargetDir;",
            "public static @Nullable Path testStructuresSourceDir;",
            "public static Rotation getRotationForRotationSteps(final int rotationSteps)",
            "throw new IllegalArgumentException(\"rotationSteps must be a value from 0-3. Got value \" + rotationSteps);",
            "public static int getRotationStepsForRotation(final Rotation rotation)",
            "clearSpaceForStructure(structureBoundingBox, level);",
            "level.setBlockAndUpdate(structurePos, Blocks.TEST_INSTANCE_BLOCK.defaultBlockState());",
            "test.set(new TestInstanceBlockEntity.Data(Optional.of(key), size, rotation, false, TestInstanceBlockEntity.Status.CLEARED, Optional.empty()))",
            "BlockPos.betweenClosedStream(structureBoundingBox).forEach(pos -> clearBlock(groundHeight, pos, level));",
            "level.getBlockTicks().clearArea(structureBoundingBox);",
            "level.clearBlockEvents(structureBoundingBox);",
            "livingEntities.forEach(Entity::discard);",
            "BlockPos farCornerBeforeTransform = structurePosition.offset(size).offset(-1, -1, -1);",
            "BoundingBox.fromCorners(northWestCorner, farCorner);",
            "return findTestBlocks(pos, searchRadius, level).filter(testBlockPosToCheck -> doesStructureContain(testBlockPosToCheck, pos, level)).findFirst();",
            "Comparator<BlockPos> distanceToPlayer = Comparator.comparingInt(pos -> pos.distManhattan(relativeToPos));",
            "PoiManager.Occupancy.ANY",
            "blockEntity.getStructureBounds().clip(start, end).isPresent()",
            "if (pos.getY() < airIfAboveThisY)",
            "BlockInput blockInput = new BlockInput(blockState, Collections.emptySet(), null);",
            "blockInput.place(level, pos, 818);",
            "blockEntity.getStructureBoundingBox().isInside(pos)",
        ] {
            assert!(
                STRUCTURE_UTILS_JAVA.contains(sentinel),
                "missing StructureUtils sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn structure_utils_rotation_steps_match_java() {
        assert_eq!(
            structure_utils_rotation_for_steps(0),
            Ok(RotationModel::None)
        );
        assert_eq!(
            structure_utils_rotation_for_steps(1),
            Ok(RotationModel::Clockwise90)
        );
        assert_eq!(
            structure_utils_rotation_for_steps(2),
            Ok(RotationModel::Clockwise180)
        );
        assert_eq!(
            structure_utils_rotation_for_steps(3),
            Ok(RotationModel::Counterclockwise90)
        );
        assert_eq!(
            structure_utils_rotation_for_steps(4),
            Err("rotationSteps must be a value from 0-3. Got value 4".to_string())
        );
        assert_eq!(structure_utils_steps_for_rotation(RotationModel::None), 0);
        assert_eq!(
            structure_utils_steps_for_rotation(RotationModel::Clockwise90),
            1
        );
        assert_eq!(
            structure_utils_steps_for_rotation(RotationModel::Clockwise180),
            2
        );
        assert_eq!(
            structure_utils_steps_for_rotation(RotationModel::Counterclockwise90),
            3
        );
    }

    #[test]
    fn structure_utils_bounding_box_and_far_corner_match_java_transform_flow() {
        let origin = BlockPosModel::new(10, 64, 20);
        let size = BlockPosModel::new(3, 2, 4);

        assert_eq!(
            structure_utils_transformed_far_corner(origin, size, RotationModel::None),
            BlockPosModel::new(12, 65, 23)
        );
        assert_eq!(
            structure_utils_structure_bounding_box(origin, size, RotationModel::None),
            StructureUtilsBoxModel::from_corners(origin, BlockPosModel::new(12, 65, 23))
        );
        assert_eq!(
            structure_utils_transformed_far_corner(origin, size, RotationModel::Clockwise90),
            BlockPosModel::new(7, 65, 22)
        );
        assert_eq!(
            structure_utils_structure_bounding_box(origin, size, RotationModel::Clockwise90),
            StructureUtilsBoxModel::from_corners(
                BlockPosModel::new(10, 64, 20),
                BlockPosModel::new(13, 65, 22)
            )
        );
        assert_eq!(
            structure_utils_structure_bounding_box(origin, size, RotationModel::Clockwise180),
            StructureUtilsBoxModel::from_corners(
                BlockPosModel::new(10, 64, 20),
                BlockPosModel::new(12, 65, 23)
            )
        );
    }

    #[test]
    fn structure_utils_create_and_clear_space_match_java_side_effects() {
        let mut level = StructureUtilsLevelModel::default();
        let created = structure_utils_create_new_empty_test(
            &mut level,
            "minecraft:test",
            BlockPosModel::new(2, 10, 3),
            BlockPosModel::new(2, 2, 2),
            RotationModel::None,
        );

        assert_eq!(created.id, "minecraft:test");
        assert_eq!(created.status, "CLEARED");
        assert!(!created.ignores_entities);
        assert!(level
            .placed_blocks
            .contains(&(BlockPosModel::new(2, 10, 3), "test_instance_block")));
        assert_eq!(level.cleared_tick_areas.len(), 1);
        assert_eq!(level.cleared_block_event_areas, level.cleared_tick_areas);
        assert_eq!(level.discarded_entity_count, 1);

        let cleared = level.cleared_tick_areas[0];
        assert_eq!(cleared.min, BlockPosModel::new(2, 11, 3));
        assert_eq!(cleared.max, BlockPosModel::new(3, 12, 4));
        assert!(level
            .placed_blocks
            .iter()
            .any(|(pos, block)| *pos == BlockPosModel::new(2, 11, 3) && *block == "air"));
        assert!(level
            .neighbor_updates
            .contains(&BlockPosModel::new(3, 12, 4)));
    }

    #[test]
    fn structure_utils_find_test_blocks_containing_nearest_and_looked_at_match_java_queries() {
        let level = StructureUtilsLevelModel {
            test_blocks: vec![
                StructureUtilsTestBlockModel {
                    pos: BlockPosModel::new(0, 64, 0),
                    structure_bounding_box: StructureUtilsBoxModel::from_corners(
                        BlockPosModel::new(0, 64, 0),
                        BlockPosModel::new(4, 70, 4),
                    ),
                    structure_bounds_hit: false,
                },
                StructureUtilsTestBlockModel {
                    pos: BlockPosModel::new(8, 64, 0),
                    structure_bounding_box: StructureUtilsBoxModel::from_corners(
                        BlockPosModel::new(8, 64, 0),
                        BlockPosModel::new(12, 70, 4),
                    ),
                    structure_bounds_hit: true,
                },
                StructureUtilsTestBlockModel {
                    pos: BlockPosModel::new(20, 64, 0),
                    structure_bounding_box: StructureUtilsBoxModel::from_corners(
                        BlockPosModel::new(20, 64, 0),
                        BlockPosModel::new(24, 70, 4),
                    ),
                    structure_bounds_hit: true,
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            structure_utils_find_test_blocks(BlockPosModel::new(0, 64, 0), 10, &level),
            vec![BlockPosModel::new(0, 64, 0), BlockPosModel::new(8, 64, 0)]
        );
        assert_eq!(
            structure_utils_find_test_containing_pos(BlockPosModel::new(10, 66, 2), 20, &level),
            Some(BlockPosModel::new(8, 64, 0))
        );
        assert_eq!(
            structure_utils_find_nearest_test(BlockPosModel::new(7, 64, 0), 20, &level),
            Some(BlockPosModel::new(8, 64, 0))
        );
        assert_eq!(
            structure_utils_looked_at_test_pos(BlockPosModel::new(18, 64, 0), &level),
            Some(BlockPosModel::new(20, 64, 0))
        );
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests_test_finder {
    use super::*;

    const TEST_FINDER_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestFinder.java"
    );
    const TEST_INSTANCE_FINDER_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestInstanceFinder.java"
    );
    const TEST_POS_FINDER_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestPosFinder.java"
    );

    fn source() -> TestFinderSourceModel {
        TestFinderSourceModel {
            id: "source".to_string(),
            position: BlockPosModel::new(0, 64, 0),
            level: StructureUtilsLevelModel {
                test_blocks: vec![
                    StructureUtilsTestBlockModel {
                        pos: BlockPosModel::new(2, 64, 0),
                        structure_bounding_box: StructureUtilsBoxModel::from_corners(
                            BlockPosModel::new(2, 64, 0),
                            BlockPosModel::new(4, 70, 2),
                        ),
                        structure_bounds_hit: false,
                    },
                    StructureUtilsTestBlockModel {
                        pos: BlockPosModel::new(20, 64, 0),
                        structure_bounding_box: StructureUtilsBoxModel::from_corners(
                            BlockPosModel::new(20, 64, 0),
                            BlockPosModel::new(22, 70, 2),
                        ),
                        structure_bounds_hit: true,
                    },
                    StructureUtilsTestBlockModel {
                        pos: BlockPosModel::new(300, 64, 0),
                        structure_bounding_box: StructureUtilsBoxModel::from_corners(
                            BlockPosModel::new(300, 64, 0),
                            BlockPosModel::new(302, 70, 2),
                        ),
                        structure_bounds_hit: true,
                    },
                ],
                ..Default::default()
            },
            failed_tests: vec![
                TestFinderTestModel {
                    id: "minecraft:required".to_string(),
                    required: true,
                },
                TestFinderTestModel {
                    id: "minecraft:optional".to_string(),
                    required: false,
                },
            ],
        }
    }

    #[test]
    fn test_finder_and_functional_interfaces_match_java_source_shape() {
        assert_eq!(TEST_FINDER_JAVA.lines().count(), 130);
        assert_eq!(TEST_INSTANCE_FINDER_JAVA.lines().count(), 9);
        assert_eq!(TEST_POS_FINDER_JAVA.lines().count(), 9);
        for sentinel in [
            "public class TestFinder implements TestInstanceFinder, TestPosFinder",
            "private static final TestInstanceFinder NO_FUNCTIONS = Stream::empty;",
            "private static final TestPosFinder NO_STRUCTURES = Stream::empty;",
            "public Stream<BlockPos> findTestPos()",
            "public Stream<Holder.Reference<GameTestInstance>> findTests()",
            "public TestFinder.Builder createMultipleCopies(final int amount)",
            "private static <Q> UnaryOperator<Supplier<Stream<Q>>> createCopies(final int amount)",
            "for (int i = 0; i < amount; i++)",
            "copyList.addAll(sourceList);",
            "StructureUtils.findTestBlocks(pos, radius, source.getLevel())",
            "StructureUtils.findNearestTest(pos, 15, source.getLevel()).stream()",
            "StructureUtils.findTestBlocks(pos, 250, source.getLevel())",
            "StructureUtils.lookedAtTestPos(BlockPos.containing(source.getPosition()), source.getPlayer().getCamera(), source.getLevel())",
            "FailedTestTracker.getLastFailedTests().filter(test -> !onlyRequiredTests || test.value().required())",
            "this.build((CommandSourceStack)sourceStack.getSource(), holders::stream, TestFinder.NO_STRUCTURES)",
            "return this.failedTests(sourceStack, false);",
            "@FunctionalInterface",
            "Stream<Holder.Reference<GameTestInstance>> findTests();",
            "Stream<BlockPos> findTestPos();",
        ] {
            assert!(
                TEST_FINDER_JAVA.contains(sentinel)
                    || TEST_INSTANCE_FINDER_JAVA.contains(sentinel)
                    || TEST_POS_FINDER_JAVA.contains(sentinel),
                "missing TestFinder sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn test_finder_position_builders_delegate_to_structure_utils_like_java() {
        let source = source();

        assert_eq!(
            test_finder_builder().radius(&source, 25).find_test_pos(),
            vec![BlockPosModel::new(2, 64, 0), BlockPosModel::new(20, 64, 0)]
        );
        assert_eq!(
            test_finder_builder().nearest(&source).find_test_pos(),
            vec![BlockPosModel::new(2, 64, 0)]
        );
        assert_eq!(
            test_finder_builder().all_nearby(&source).find_test_pos(),
            vec![BlockPosModel::new(2, 64, 0), BlockPosModel::new(20, 64, 0)]
        );
        assert_eq!(
            test_finder_builder().looked_at(&source).find_test_pos(),
            vec![BlockPosModel::new(20, 64, 0)]
        );
    }

    #[test]
    fn test_finder_failed_and_resource_selection_match_java() {
        let source = source();
        assert_eq!(
            test_finder_builder()
                .failed_tests(&source, false)
                .find_tests(),
            source.failed_tests
        );
        assert_eq!(
            test_finder_builder()
                .failed_tests(&source, true)
                .find_tests(),
            vec![TestFinderTestModel {
                id: "minecraft:required".to_string(),
                required: true,
            }]
        );
        assert_eq!(
            test_finder_builder()
                .by_resource_selection(
                    &source,
                    vec![TestFinderTestModel {
                        id: "minecraft:selected".to_string(),
                        required: true,
                    }],
                )
                .find_tests(),
            vec![TestFinderTestModel {
                id: "minecraft:selected".to_string(),
                required: true,
            }]
        );
    }

    #[test]
    fn test_finder_create_multiple_copies_repeats_tests_and_positions_like_java() {
        let source = source();
        let copied_positions = test_finder_builder()
            .create_multiple_copies(3)
            .nearest(&source)
            .find_test_pos();
        assert_eq!(
            copied_positions,
            vec![
                BlockPosModel::new(2, 64, 0),
                BlockPosModel::new(2, 64, 0),
                BlockPosModel::new(2, 64, 0),
            ]
        );

        let copied_tests = test_finder_builder()
            .create_multiple_copies(2)
            .failed_tests(&source, true)
            .find_tests();
        assert_eq!(
            copied_tests,
            vec![
                TestFinderTestModel {
                    id: "minecraft:required".to_string(),
                    required: true,
                },
                TestFinderTestModel {
                    id: "minecraft:required".to_string(),
                    required: true,
                },
            ]
        );
    }
}
