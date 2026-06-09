use super::{create_gametest_batch, BlockPosModel, GameTestBatchModel, GameTestInfoStateModel};

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
