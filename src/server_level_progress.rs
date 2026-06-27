#![allow(dead_code)]

use crate::storage::region::ChunkPos;

pub const PREPARE_SERVER_WEIGHT: i32 = 10;
pub const EXPECTED_PLAYER_CHUNKS: i32 = 49;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelLoadStage {
    StartServer,
    PrepareGlobalSpawn,
    LoadInitialChunks,
    LoadPlayerChunks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelLoadEvent {
    Start {
        stage: LevelLoadStage,
        total_chunks: i32,
    },
    Update {
        stage: LevelLoadStage,
        current_chunks: i32,
        total_chunks: i32,
    },
    Finish {
        stage: LevelLoadStage,
    },
    UpdateFocus {
        dimension: String,
        chunk_pos: ChunkPos,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecordingLevelLoadListener {
    pub events: Vec<LevelLoadEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedLevelLoadListenerModel {
    first: RecordingLevelLoadListener,
    second: RecordingLevelLoadListener,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLoadStatusViewModel {
    dimension: Option<String>,
    center_chunk: Option<ChunkPos>,
    radius: i32,
    statuses: Vec<ChunkLoadStatusCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLoadStatusCell {
    pub x: i32,
    pub z: i32,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelLoadProgressTrackerModel {
    include_player_chunks: bool,
    total_weight: i32,
    finalized_weight: i32,
    segment_weight: i32,
    segment_fraction: f32,
    progress: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoggingLevelLoadListenerModel {
    include_player_chunks: bool,
    progress_tracker: LevelLoadProgressTrackerModel,
    closed: bool,
    start_time: i64,
    next_log_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelLoadLog {
    SelectingGlobalWorldSpawn,
    LoadingPersistentChunks(i32),
    LoadingPlayerSpawnChunks(i32),
    PreparingSpawn(i32),
    TimeElapsed(i64),
}

impl RecordingLevelLoadListener {
    pub fn start(&mut self, stage: LevelLoadStage, total_chunks: i32) {
        self.events.push(LevelLoadEvent::Start {
            stage,
            total_chunks,
        });
    }

    pub fn update(&mut self, stage: LevelLoadStage, current_chunks: i32, total_chunks: i32) {
        self.events.push(LevelLoadEvent::Update {
            stage,
            current_chunks,
            total_chunks,
        });
    }

    pub fn finish(&mut self, stage: LevelLoadStage) {
        self.events.push(LevelLoadEvent::Finish { stage });
    }

    pub fn update_focus(&mut self, dimension: impl Into<String>, chunk_pos: ChunkPos) {
        self.events.push(LevelLoadEvent::UpdateFocus {
            dimension: dimension.into(),
            chunk_pos,
        });
    }
}

impl ComposedLevelLoadListenerModel {
    pub fn new(first: RecordingLevelLoadListener, second: RecordingLevelLoadListener) -> Self {
        Self { first, second }
    }

    pub fn start(&mut self, stage: LevelLoadStage, total_chunks: i32) {
        self.first.start(stage, total_chunks);
        self.second.start(stage, total_chunks);
    }

    pub fn update(&mut self, stage: LevelLoadStage, current_chunks: i32, total_chunks: i32) {
        self.first.update(stage, current_chunks, total_chunks);
        self.second.update(stage, current_chunks, total_chunks);
    }

    pub fn finish(&mut self, stage: LevelLoadStage) {
        self.first.finish(stage);
        self.second.finish(stage);
    }

    pub fn update_focus(&mut self, dimension: impl Into<String>, chunk_pos: ChunkPos) {
        let dimension = dimension.into();
        self.first.update_focus(dimension.clone(), chunk_pos);
        self.second.update_focus(dimension, chunk_pos);
    }

    pub fn listeners(&self) -> (&RecordingLevelLoadListener, &RecordingLevelLoadListener) {
        (&self.first, &self.second)
    }
}

impl ChunkLoadStatusViewModel {
    pub fn new(radius: i32) -> Self {
        Self {
            dimension: None,
            center_chunk: None,
            radius,
            statuses: Vec::new(),
        }
    }

    pub fn move_to(&mut self, dimension: impl Into<String>, center_chunk: ChunkPos) {
        self.dimension = Some(dimension.into());
        self.center_chunk = Some(center_chunk);
    }

    pub fn set(&mut self, x: i32, z: i32, status: &'static str) {
        if let Some(cell) = self
            .statuses
            .iter_mut()
            .find(|cell| cell.x == x && cell.z == z)
        {
            cell.status = status;
        } else {
            self.statuses.push(ChunkLoadStatusCell { x, z, status });
        }
    }

    pub fn get(&self, x: i32, z: i32) -> Option<&'static str> {
        self.statuses
            .iter()
            .find(|cell| cell.x == x && cell.z == z)
            .map(|cell| cell.status)
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    pub fn dimension(&self) -> Option<&str> {
        self.dimension.as_deref()
    }

    pub fn center_chunk(&self) -> Option<ChunkPos> {
        self.center_chunk
    }
}

impl LevelLoadProgressTrackerModel {
    pub fn new(include_player_chunks: bool) -> Self {
        Self {
            include_player_chunks,
            total_weight: 0,
            finalized_weight: 0,
            segment_weight: 0,
            segment_fraction: 0.0,
            progress: 0.0,
        }
    }

    pub fn start(&mut self, stage: LevelLoadStage, total_chunks: i32) {
        if self.tracks_stage(stage) {
            match stage {
                LevelLoadStage::LoadInitialChunks => {
                    let player_chunks_weight = if self.include_player_chunks {
                        EXPECTED_PLAYER_CHUNKS
                    } else {
                        0
                    };
                    self.total_weight = PREPARE_SERVER_WEIGHT + total_chunks + player_chunks_weight;
                    self.begin_segment(PREPARE_SERVER_WEIGHT);
                    self.finish_segment();
                    self.begin_segment(total_chunks);
                }
                LevelLoadStage::LoadPlayerChunks => {
                    self.begin_segment(EXPECTED_PLAYER_CHUNKS);
                }
                LevelLoadStage::StartServer | LevelLoadStage::PrepareGlobalSpawn => {}
            }
        }
    }

    pub fn update(&mut self, stage: LevelLoadStage, current_chunks: i32, total_chunks: i32) {
        if self.tracks_stage(stage) {
            self.segment_fraction = if total_chunks == 0 {
                0.0
            } else {
                current_chunks as f32 / total_chunks as f32
            };
            self.update_progress();
        }
    }

    pub fn finish(&mut self, stage: LevelLoadStage) {
        if self.tracks_stage(stage) {
            self.finish_segment();
        }
    }

    pub fn get(&self) -> f32 {
        self.progress
    }

    pub fn update_focus(&self, _dimension: &str, _chunk_pos: ChunkPos) {}

    fn begin_segment(&mut self, weight: i32) {
        self.segment_weight = weight;
        self.segment_fraction = 0.0;
        self.update_progress();
    }

    fn finish_segment(&mut self) {
        self.finalized_weight += self.segment_weight;
        self.segment_weight = 0;
        self.update_progress();
    }

    fn tracks_stage(&self, stage: LevelLoadStage) -> bool {
        match stage {
            LevelLoadStage::LoadInitialChunks => true,
            LevelLoadStage::LoadPlayerChunks => self.include_player_chunks,
            LevelLoadStage::StartServer | LevelLoadStage::PrepareGlobalSpawn => false,
        }
    }

    fn update_progress(&mut self) {
        if self.total_weight == 0 {
            self.progress = 0.0;
        } else {
            let current_weight =
                self.finalized_weight as f32 + self.segment_fraction * self.segment_weight as f32;
            self.progress = current_weight / self.total_weight as f32;
        }
    }
}

impl LoggingLevelLoadListenerModel {
    pub fn new(include_player_chunks: bool) -> Self {
        Self {
            include_player_chunks,
            progress_tracker: LevelLoadProgressTrackerModel::new(include_player_chunks),
            closed: false,
            start_time: i64::MAX,
            next_log_time: i64::MAX,
        }
    }

    pub fn for_dedicated_server() -> Self {
        Self::new(false)
    }

    pub fn for_singleplayer() -> Self {
        Self::new(true)
    }

    pub fn start(
        &mut self,
        stage: LevelLoadStage,
        total_chunks: i32,
        now_millis: i64,
    ) -> Vec<LevelLoadLog> {
        if self.closed {
            return Vec::new();
        }

        if self.start_time == i64::MAX {
            self.start_time = now_millis;
            self.next_log_time = now_millis;
        }

        self.progress_tracker.start(stage, total_chunks);
        match stage {
            LevelLoadStage::PrepareGlobalSpawn => vec![LevelLoadLog::SelectingGlobalWorldSpawn],
            LevelLoadStage::LoadInitialChunks => {
                vec![LevelLoadLog::LoadingPersistentChunks(total_chunks)]
            }
            LevelLoadStage::LoadPlayerChunks => {
                vec![LevelLoadLog::LoadingPlayerSpawnChunks(total_chunks)]
            }
            LevelLoadStage::StartServer => Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        stage: LevelLoadStage,
        current_chunks: i32,
        total_chunks: i32,
        now_millis: i64,
    ) -> Vec<LevelLoadLog> {
        if self.closed {
            return Vec::new();
        }

        self.progress_tracker
            .update(stage, current_chunks, total_chunks);
        if now_millis > self.next_log_time {
            self.next_log_time += 500;
            let percent = (self.progress_tracker.get() * 100.0).floor() as i32;
            vec![LevelLoadLog::PreparingSpawn(percent)]
        } else {
            Vec::new()
        }
    }

    pub fn finish(&mut self, stage: LevelLoadStage, now_millis: i64) -> Vec<LevelLoadLog> {
        if self.closed {
            return Vec::new();
        }

        self.progress_tracker.finish(stage);
        let final_stage = if self.include_player_chunks {
            LevelLoadStage::LoadPlayerChunks
        } else {
            LevelLoadStage::LoadInitialChunks
        };
        if stage == final_stage {
            self.next_log_time = i64::MAX;
            self.closed = true;
            vec![LevelLoadLog::TimeElapsed(now_millis - self.start_time)]
        } else {
            Vec::new()
        }
    }

    pub fn update_focus(&self, _dimension: &str, _chunk_pos: ChunkPos) {}

    pub fn progress(&self) -> f32 {
        self.progress_tracker.get()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn start_time(&self) -> i64 {
        self.start_time
    }

    pub fn next_log_time(&self) -> i64 {
        self.next_log_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(x: i32, z: i32) -> ChunkPos {
        ChunkPos { x, z }
    }

    #[test]
    fn chunk_load_status_view_tracks_move_get_and_radius_contract() {
        let mut view = ChunkLoadStatusViewModel::new(12);

        assert_eq!(view.radius(), 12);
        assert_eq!(view.dimension(), None);
        assert_eq!(view.center_chunk(), None);
        assert_eq!(view.get(1, 2), None);

        view.move_to("minecraft:overworld", chunk(4, -3));
        view.set(1, 2, "minecraft:empty");
        view.set(3, 4, "minecraft:full");
        view.set(1, 2, "minecraft:light");

        assert_eq!(view.dimension(), Some("minecraft:overworld"));
        assert_eq!(view.center_chunk(), Some(chunk(4, -3)));
        assert_eq!(view.get(1, 2), Some("minecraft:light"));
        assert_eq!(view.get(3, 4), Some("minecraft:full"));
        assert_eq!(view.get(9, 9), None);
    }

    #[test]
    fn composed_level_load_listener_forwards_to_first_then_second() {
        let mut composed = ComposedLevelLoadListenerModel::new(
            RecordingLevelLoadListener::default(),
            RecordingLevelLoadListener::default(),
        );

        composed.start(LevelLoadStage::LoadInitialChunks, 9);
        composed.update(LevelLoadStage::LoadInitialChunks, 3, 9);
        composed.finish(LevelLoadStage::LoadInitialChunks);
        composed.update_focus("minecraft:the_nether", chunk(-2, 7));

        let expected = vec![
            LevelLoadEvent::Start {
                stage: LevelLoadStage::LoadInitialChunks,
                total_chunks: 9,
            },
            LevelLoadEvent::Update {
                stage: LevelLoadStage::LoadInitialChunks,
                current_chunks: 3,
                total_chunks: 9,
            },
            LevelLoadEvent::Finish {
                stage: LevelLoadStage::LoadInitialChunks,
            },
            LevelLoadEvent::UpdateFocus {
                dimension: "minecraft:the_nether".to_string(),
                chunk_pos: chunk(-2, 7),
            },
        ];
        let (first, second) = composed.listeners();
        assert_eq!(first.events, expected);
        assert_eq!(second.events, expected);
    }

    #[test]
    fn progress_tracker_matches_java_initial_and_player_chunk_weights() {
        assert_eq!(PREPARE_SERVER_WEIGHT, 10);
        assert_eq!(EXPECTED_PLAYER_CHUNKS, 49);

        let mut dedicated = LevelLoadProgressTrackerModel::new(false);
        assert_eq!(dedicated.get(), 0.0);
        dedicated.start(LevelLoadStage::PrepareGlobalSpawn, 500);
        assert_eq!(dedicated.get(), 0.0);
        dedicated.start(LevelLoadStage::LoadInitialChunks, 90);
        assert_eq!(dedicated.get(), 0.1);
        dedicated.update(LevelLoadStage::LoadInitialChunks, 45, 90);
        assert_eq!(dedicated.get(), 0.55);
        dedicated.finish(LevelLoadStage::LoadInitialChunks);
        assert_eq!(dedicated.get(), 1.0);
        dedicated.start(LevelLoadStage::LoadPlayerChunks, 49);
        assert_eq!(dedicated.get(), 1.0);

        let mut singleplayer = LevelLoadProgressTrackerModel::new(true);
        singleplayer.start(LevelLoadStage::LoadInitialChunks, 90);
        assert!((singleplayer.get() - (10.0 / 149.0)).abs() < f32::EPSILON);
        singleplayer.update(LevelLoadStage::LoadInitialChunks, 90, 90);
        assert!((singleplayer.get() - (100.0 / 149.0)).abs() < f32::EPSILON);
        singleplayer.finish(LevelLoadStage::LoadInitialChunks);
        assert!((singleplayer.get() - (100.0 / 149.0)).abs() < f32::EPSILON);
        singleplayer.start(LevelLoadStage::LoadPlayerChunks, 49);
        assert!((singleplayer.get() - (100.0 / 149.0)).abs() < f32::EPSILON);
        singleplayer.update(LevelLoadStage::LoadPlayerChunks, 24, 49);
        assert!((singleplayer.get() - (124.0 / 149.0)).abs() < 0.000_001);
        singleplayer.finish(LevelLoadStage::LoadPlayerChunks);
        assert_eq!(singleplayer.get(), 1.0);
    }

    #[test]
    fn progress_tracker_zero_total_update_keeps_segment_fraction_zero() {
        let mut tracker = LevelLoadProgressTrackerModel::new(false);
        tracker.start(LevelLoadStage::LoadInitialChunks, 0);
        assert_eq!(tracker.get(), 1.0);
        tracker.update(LevelLoadStage::LoadInitialChunks, 1, 0);
        assert_eq!(tracker.get(), 1.0);
    }

    #[test]
    fn logging_listener_matches_java_messages_timing_and_close_rules() {
        let mut dedicated = LoggingLevelLoadListenerModel::for_dedicated_server();
        assert_eq!(
            dedicated.start(LevelLoadStage::PrepareGlobalSpawn, 0, 1_000),
            vec![LevelLoadLog::SelectingGlobalWorldSpawn]
        );
        assert_eq!(dedicated.start_time(), 1_000);
        assert_eq!(dedicated.next_log_time(), 1_000);
        assert_eq!(
            dedicated.start(LevelLoadStage::LoadInitialChunks, 90, 1_050),
            vec![LevelLoadLog::LoadingPersistentChunks(90)]
        );
        assert_eq!(
            dedicated.update(LevelLoadStage::LoadInitialChunks, 44, 90, 1_000),
            Vec::<LevelLoadLog>::new()
        );
        assert_eq!(
            dedicated.update(LevelLoadStage::LoadInitialChunks, 45, 90, 1_001),
            vec![LevelLoadLog::PreparingSpawn(55)]
        );
        assert_eq!(dedicated.next_log_time(), 1_500);
        assert_eq!(
            dedicated.update(LevelLoadStage::LoadInitialChunks, 90, 90, 1_500),
            Vec::<LevelLoadLog>::new()
        );
        assert_eq!(
            dedicated.update(LevelLoadStage::LoadInitialChunks, 90, 90, 1_501),
            vec![LevelLoadLog::PreparingSpawn(100)]
        );
        assert_eq!(
            dedicated.finish(LevelLoadStage::LoadInitialChunks, 2_345),
            vec![LevelLoadLog::TimeElapsed(1_345)]
        );
        assert!(dedicated.is_closed());
        assert_eq!(dedicated.next_log_time(), i64::MAX);
        assert_eq!(
            dedicated.start(LevelLoadStage::LoadPlayerChunks, 49, 2_400),
            Vec::<LevelLoadLog>::new()
        );
    }

    #[test]
    fn singleplayer_logging_waits_for_player_chunks_before_closing() {
        let mut singleplayer = LoggingLevelLoadListenerModel::for_singleplayer();

        assert_eq!(
            singleplayer.start(LevelLoadStage::LoadInitialChunks, 90, 10),
            vec![LevelLoadLog::LoadingPersistentChunks(90)]
        );
        assert_eq!(
            singleplayer.finish(LevelLoadStage::LoadInitialChunks, 20),
            Vec::<LevelLoadLog>::new()
        );
        assert!(!singleplayer.is_closed());
        assert_eq!(
            singleplayer.start(LevelLoadStage::LoadPlayerChunks, 49, 30),
            vec![LevelLoadLog::LoadingPlayerSpawnChunks(49)]
        );
        assert_eq!(
            singleplayer.finish(LevelLoadStage::LoadPlayerChunks, 40),
            vec![LevelLoadLog::TimeElapsed(30)]
        );
        assert!(singleplayer.is_closed());
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn progress_sources_match_java_26_1_2() {
        const CHUNK_LOAD_STATUS_VIEW: &str =
            vibecraft_java_source!("/net/minecraft/server/level/progress/ChunkLoadStatusView.java");
        const LEVEL_LOAD_LISTENER: &str =
            vibecraft_java_source!("/net/minecraft/server/level/progress/LevelLoadListener.java");
        const LEVEL_LOAD_PROGRESS_TRACKER: &str = vibecraft_java_source!(
            "/net/minecraft/server/level/progress/LevelLoadProgressTracker.java"
        );
        const LOGGING_LEVEL_LOAD_LISTENER: &str = vibecraft_java_source!(
            "/net/minecraft/server/level/progress/LoggingLevelLoadListener.java"
        );

        for sentinel in [
            "public interface ChunkLoadStatusView",
            "void moveTo(ResourceKey<Level> dimension, ChunkPos centerChunk);",
            "@Nullable ChunkStatus get(int x, int z);",
            "int radius();",
        ] {
            assert!(
                CHUNK_LOAD_STATUS_VIEW.contains(sentinel),
                "ChunkLoadStatusView.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public interface LevelLoadListener",
            "static LevelLoadListener compose(final LevelLoadListener first, final LevelLoadListener second)",
            "first.start(stage, totalChunks);",
            "second.start(stage, totalChunks);",
            "first.update(stage, currentChunks, totalChunks);",
            "second.update(stage, currentChunks, totalChunks);",
            "first.finish(stage);",
            "second.finish(stage);",
            "first.updateFocus(dimension, chunkPos);",
            "second.updateFocus(dimension, chunkPos);",
            "START_SERVER,",
            "PREPARE_GLOBAL_SPAWN,",
            "LOAD_INITIAL_CHUNKS,",
            "LOAD_PLAYER_CHUNKS;",
        ] {
            assert!(
                LEVEL_LOAD_LISTENER.contains(sentinel),
                "LevelLoadListener.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "private static final int PREPARE_SERVER_WEIGHT = 10;",
            "private static final int EXPECTED_PLAYER_CHUNKS = Mth.square(7);",
            "private final boolean includePlayerChunks;",
            "case LOAD_INITIAL_CHUNKS:",
            "int playerChunksWeight = this.includePlayerChunks ? EXPECTED_PLAYER_CHUNKS : 0;",
            "this.totalWeight = 10 + totalChunks + playerChunksWeight;",
            "this.beginSegment(10);",
            "this.finishSegment();",
            "this.beginSegment(totalChunks);",
            "case LOAD_PLAYER_CHUNKS:",
            "this.beginSegment(EXPECTED_PLAYER_CHUNKS);",
            "this.segmentFraction = totalChunks == 0 ? 0.0F : (float)currentChunks / totalChunks;",
            "this.finalizedWeight = this.finalizedWeight + this.segmentWeight;",
            "case LOAD_INITIAL_CHUNKS -> true;",
            "case LOAD_PLAYER_CHUNKS -> this.includePlayerChunks;",
            "float currentWeight = this.finalizedWeight + this.segmentFraction * this.segmentWeight;",
            "this.progress = currentWeight / this.totalWeight;",
            "public float get()",
        ] {
            assert!(
                LEVEL_LOAD_PROGRESS_TRACKER.contains(sentinel),
                "LevelLoadProgressTracker.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class LoggingLevelLoadListener implements LevelLoadListener",
            "private final boolean includePlayerChunks;",
            "private final LevelLoadProgressTracker progressTracker;",
            "private boolean closed;",
            "private long startTime = Long.MAX_VALUE;",
            "private long nextLogTime = Long.MAX_VALUE;",
            "public static LoggingLevelLoadListener forDedicatedServer()",
            "return new LoggingLevelLoadListener(false);",
            "public static LoggingLevelLoadListener forSingleplayer()",
            "return new LoggingLevelLoadListener(true);",
            "if (!this.closed)",
            "if (this.startTime == Long.MAX_VALUE)",
            "this.nextLogTime = now;",
            "LOGGER.info(\"Selecting global world spawn...\");",
            "LOGGER.info(\"Loading {} persistent chunks...\", totalChunks);",
            "LOGGER.info(\"Loading {} chunks for player spawn...\", totalChunks);",
            "if (Util.getMillis() > this.nextLogTime)",
            "this.nextLogTime += 500L;",
            "int percent = Mth.floor(this.progressTracker.get() * 100.0F);",
            "LOGGER.info(Component.translatable(\"menu.preparingSpawn\", percent).getString());",
            "? LevelLoadListener.Stage.LOAD_PLAYER_CHUNKS",
            ": LevelLoadListener.Stage.LOAD_INITIAL_CHUNKS;",
            "LOGGER.info(\"Time elapsed: {} ms\", Util.getMillis() - this.startTime);",
            "this.nextLogTime = Long.MAX_VALUE;",
            "this.closed = true;",
        ] {
            assert!(
                LOGGING_LEVEL_LOAD_LISTENER.contains(sentinel),
                "LoggingLevelLoadListener.java is missing sentinel: {sentinel}"
            );
        }
    }
}
