#![allow(dead_code)]

use crate::network::play::RemoteDebugSampleType as PacketRemoteDebugSampleType;

pub trait SampleLogger {
    fn log_full_sample(&mut self, sample: &[i64]) -> Result<(), String>;
    fn log_sample(&mut self, sample: i64);
    fn log_partial_sample(&mut self, sample: i64, dimension: usize) -> Result<(), String>;
}

pub trait SampleStorage {
    fn capacity(&self) -> usize;
    fn size(&self) -> usize;
    fn get(&self, index: usize) -> Result<i64, String>;
    fn get_dimension(&self, index: usize, dimension: usize) -> Result<i64, String>;
    fn reset(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleAccumulator {
    defaults: Vec<i64>,
    sample: Vec<i64>,
}

impl SampleAccumulator {
    pub fn new(dimensions: usize, defaults: Vec<i64>) -> Result<Self, String> {
        if defaults.len() != dimensions {
            return Err(format!(
                "defaults have incorrect length of {}",
                defaults.len()
            ));
        }
        Ok(Self {
            defaults,
            sample: vec![0; dimensions],
        })
    }

    fn log_full_sample(&mut self, sample: &[i64]) {
        self.sample[..sample.len()].copy_from_slice(sample);
    }

    fn log_sample(&mut self, sample: i64) {
        self.sample[0] = sample;
    }

    fn log_partial_sample(&mut self, sample: i64, dimension: usize) -> Result<(), String> {
        if dimension >= 1 && dimension < self.sample.len() {
            self.sample[dimension] = sample;
            Ok(())
        } else {
            Err(format!(
                "{} out of bounds for dimensions {}",
                dimension,
                self.sample.len()
            ))
        }
    }

    fn reset_sample(&mut self) {
        self.sample.copy_from_slice(&self.defaults);
    }

    fn snapshot(&self) -> Vec<i64> {
        self.sample.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSampleLogger {
    accumulator: SampleAccumulator,
    samples: Vec<Vec<i64>>,
    start: usize,
    size: usize,
}

impl LocalSampleLogger {
    pub const CAPACITY: usize = 240;

    pub fn new(dimensions: usize) -> Self {
        Self::with_defaults(dimensions, vec![0; dimensions]).unwrap_or_else(|err| panic!("{err}"))
    }

    pub fn with_defaults(dimensions: usize, defaults: Vec<i64>) -> Result<Self, String> {
        Ok(Self {
            accumulator: SampleAccumulator::new(dimensions, defaults)?,
            samples: vec![vec![0; dimensions]; Self::CAPACITY],
            start: 0,
            size: 0,
        })
    }

    fn use_sample(&mut self) {
        let next_index = self.wrap_index(self.start + self.size);
        self.samples[next_index].copy_from_slice(&self.accumulator.sample);
        if self.size < Self::CAPACITY {
            self.size += 1;
        } else {
            self.start = self.wrap_index(self.start + 1);
        }
    }

    fn wrap_index(&self, index: usize) -> usize {
        index % Self::CAPACITY
    }
}

impl SampleLogger for LocalSampleLogger {
    fn log_full_sample(&mut self, sample: &[i64]) -> Result<(), String> {
        self.accumulator.log_full_sample(sample);
        self.use_sample();
        self.accumulator.reset_sample();
        Ok(())
    }

    fn log_sample(&mut self, sample: i64) {
        self.accumulator.log_sample(sample);
        self.use_sample();
        self.accumulator.reset_sample();
    }

    fn log_partial_sample(&mut self, sample: i64, dimension: usize) -> Result<(), String> {
        self.accumulator.log_partial_sample(sample, dimension)
    }
}

impl SampleStorage for LocalSampleLogger {
    fn capacity(&self) -> usize {
        self.samples.len()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn get(&self, index: usize) -> Result<i64, String> {
        self.get_dimension(index, 0)
    }

    fn get_dimension(&self, index: usize, dimension: usize) -> Result<i64, String> {
        if index >= self.size {
            return Err(format!("{} out of bounds for length {}", index, self.size));
        }
        let sample_array = &self.samples[self.wrap_index(self.start + index)];
        sample_array.get(dimension).copied().ok_or_else(|| {
            format!(
                "{} out of bounds for dimensions {}",
                dimension,
                sample_array.len()
            )
        })
    }

    fn reset(&mut self) {
        self.start = 0;
        self.size = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteDebugSampleType {
    TickTime,
}

impl RemoteDebugSampleType {
    pub fn subscription(self) -> DebugSubscription {
        match self {
            Self::TickTime => DebugSubscription::DedicatedServerTickTime,
        }
    }

    pub fn packet_type(self) -> PacketRemoteDebugSampleType {
        match self {
            Self::TickTime => PacketRemoteDebugSampleType::TickTime,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscription {
    DedicatedServerTickTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpsDebugDimensions {
    FullTick,
    TickServerMethod,
    ScheduledTasks,
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSamplePacket {
    pub sample: Vec<i64>,
    pub sample_type: RemoteDebugSampleType,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServerDebugSubscribers {
    tick_time_subscribers: usize,
    broadcasts: Vec<(DebugSubscription, DebugSamplePacket)>,
}

impl ServerDebugSubscribers {
    pub fn subscribe(&mut self, subscription: DebugSubscription) {
        match subscription {
            DebugSubscription::DedicatedServerTickTime => self.tick_time_subscribers += 1,
        }
    }

    pub fn has_any_subscriber_for(&self, subscription: DebugSubscription) -> bool {
        match subscription {
            DebugSubscription::DedicatedServerTickTime => self.tick_time_subscribers > 0,
        }
    }

    pub fn broadcast_to_all(&mut self, subscription: DebugSubscription, packet: DebugSamplePacket) {
        self.broadcasts.push((subscription, packet));
    }

    pub fn broadcasts(&self) -> &[(DebugSubscription, DebugSamplePacket)] {
        &self.broadcasts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSampleLogger {
    accumulator: SampleAccumulator,
    subscribers: ServerDebugSubscribers,
    sample_type: RemoteDebugSampleType,
}

impl RemoteSampleLogger {
    pub fn new(
        dimensions: usize,
        subscribers: ServerDebugSubscribers,
        sample_type: RemoteDebugSampleType,
    ) -> Self {
        Self::with_defaults(dimensions, subscribers, sample_type, vec![0; dimensions])
            .unwrap_or_else(|err| panic!("{err}"))
    }

    pub fn with_defaults(
        dimensions: usize,
        subscribers: ServerDebugSubscribers,
        sample_type: RemoteDebugSampleType,
        defaults: Vec<i64>,
    ) -> Result<Self, String> {
        Ok(Self {
            accumulator: SampleAccumulator::new(dimensions, defaults)?,
            subscribers,
            sample_type,
        })
    }

    pub fn subscribers(&self) -> &ServerDebugSubscribers {
        &self.subscribers
    }

    fn use_sample(&mut self) {
        let subscription = self.sample_type.subscription();
        if self.subscribers.has_any_subscriber_for(subscription) {
            self.subscribers.broadcast_to_all(
                subscription,
                DebugSamplePacket {
                    sample: self.accumulator.snapshot(),
                    sample_type: self.sample_type,
                },
            );
        }
    }
}

impl SampleLogger for RemoteSampleLogger {
    fn log_full_sample(&mut self, sample: &[i64]) -> Result<(), String> {
        self.accumulator.log_full_sample(sample);
        self.use_sample();
        self.accumulator.reset_sample();
        Ok(())
    }

    fn log_sample(&mut self, sample: i64) {
        self.accumulator.log_sample(sample);
        self.use_sample();
        self.accumulator.reset_sample();
    }

    fn log_partial_sample(&mut self, sample: i64, dimension: usize) -> Result<(), String> {
        self.accumulator.log_partial_sample(sample, dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got Err({err:?})"),
        }
    }

    #[test]
    fn abstract_sample_logger_validates_defaults_and_resets_after_use() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/AbstractSampleLogger.java");
        assert!(source.contains("defaults have incorrect length of "));
        assert!(source.contains("System.arraycopy(sample, 0, this.sample, 0, sample.length);"));
        assert!(source.contains("this.useSample();\n      this.resetSample();"));
        assert!(source.contains("dimension >= 1 && dimension < this.sample.length"));

        assert_eq!(
            LocalSampleLogger::with_defaults(2, vec![0]).map(|_| "ok".to_string()),
            Err("defaults have incorrect length of 1".to_string())
        );

        let mut logger = must_ok(LocalSampleLogger::with_defaults(3, vec![10, 20, 30]));
        assert_eq!(
            logger.log_partial_sample(99, 0),
            Err("0 out of bounds for dimensions 3".to_string())
        );
        must_ok(logger.log_partial_sample(77, 1));
        logger.log_sample(5);
        assert_eq!(logger.get_dimension(0, 0), Ok(5));
        assert_eq!(logger.get_dimension(0, 1), Ok(77));
        assert_eq!(logger.get_dimension(0, 2), Ok(0));

        logger.log_sample(6);
        assert_eq!(logger.get_dimension(1, 1), Ok(20));
    }

    #[test]
    fn local_sample_logger_uses_java_ring_buffer_and_storage_bounds() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/LocalSampleLogger.java");
        assert!(source.contains("public static final int CAPACITY = 240"));
        assert!(source.contains("this.start = this.wrapIndex(this.start + 1);"));
        assert!(source.contains("return this.get(index, 0);"));
        assert!(source.contains("this.start = 0;\n      this.size = 0;"));

        let mut logger = LocalSampleLogger::new(2);
        for value in 0..245 {
            must_ok(logger.log_full_sample(&[value, value + 1000]));
        }

        assert_eq!(logger.capacity(), 240);
        assert_eq!(logger.size(), 240);
        assert_eq!(logger.get(0), Ok(5));
        assert_eq!(logger.get_dimension(239, 1), Ok(1244));
        assert_eq!(
            logger.get_dimension(240, 0),
            Err("240 out of bounds for length 240".to_string())
        );
        assert_eq!(
            logger.get_dimension(0, 2),
            Err("2 out of bounds for dimensions 2".to_string())
        );

        logger.reset();
        assert_eq!(logger.size(), 0);
    }

    #[test]
    fn remote_sample_type_and_tps_dimensions_match_java_order() {
        let remote_source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/RemoteDebugSampleType.java");
        let tps_source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/TpsDebugDimensions.java");
        assert!(remote_source.contains("TICK_TIME(DebugSubscriptions.DEDICATED_SERVER_TICK_TIME)"));
        assert!(remote_source.contains("public DebugSubscription<?> subscription()"));
        assert!(tps_source.contains("FULL_TICK,\n   TICK_SERVER_METHOD,\n   SCHEDULED_TASKS,\n   IDLE;"));

        assert_eq!(
            RemoteDebugSampleType::TickTime.subscription(),
            DebugSubscription::DedicatedServerTickTime
        );
        assert_eq!(
            RemoteDebugSampleType::TickTime.packet_type(),
            PacketRemoteDebugSampleType::TickTime
        );
        assert_eq!(
            [
                TpsDebugDimensions::FullTick,
                TpsDebugDimensions::TickServerMethod,
                TpsDebugDimensions::ScheduledTasks,
                TpsDebugDimensions::Idle,
            ]
            .len(),
            4
        );
    }

    #[test]
    fn remote_sample_logger_broadcasts_only_when_subscribed_and_clones_sample() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/RemoteSampleLogger.java");
        assert!(source.contains("this.subscribers.hasAnySubscriberFor(this.sampleType.subscription())"));
        assert!(source.contains("new ClientboundDebugSamplePacket((long[])this.sample.clone(), this.sampleType)"));

        let mut unsubscribed = RemoteSampleLogger::new(
            2,
            ServerDebugSubscribers::default(),
            RemoteDebugSampleType::TickTime,
        );
        unsubscribed.log_sample(1);
        assert!(unsubscribed.subscribers().broadcasts().is_empty());

        let mut subscribers = ServerDebugSubscribers::default();
        subscribers.subscribe(DebugSubscription::DedicatedServerTickTime);
        let mut subscribed =
            RemoteSampleLogger::new(2, subscribers, RemoteDebugSampleType::TickTime);
        must_ok(subscribed.log_partial_sample(8, 1));
        subscribed.log_sample(7);
        let broadcasts = subscribed.subscribers().broadcasts();
        assert_eq!(broadcasts.len(), 1);
        assert_eq!(broadcasts[0].0, DebugSubscription::DedicatedServerTickTime);
        assert_eq!(
            broadcasts[0].1,
            DebugSamplePacket {
                sample: vec![7, 8],
                sample_type: RemoteDebugSampleType::TickTime,
            }
        );

        subscribed.log_sample(9);
        assert_eq!(subscribed.subscribers().broadcasts()[0].1.sample, vec![7, 8]);
        assert_eq!(subscribed.subscribers().broadcasts()[1].1.sample, vec![9, 0]);
    }

    #[test]
    fn sample_interfaces_are_represented_by_rust_traits() {
        let logger_source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/SampleLogger.java");
        let storage_source =
            vibecraft_java_source!("/net/minecraft/util/debugchart/SampleStorage.java");
        assert!(logger_source.contains("void logFullSample(final long[] sample);"));
        assert!(logger_source.contains("void logSample(final long sample);"));
        assert!(logger_source.contains("void logPartialSample(final long sample, final int dimension);"));
        assert!(storage_source.contains("int capacity();"));
        assert!(storage_source.contains("long get(final int index, final int dimension);"));
        assert!(storage_source.contains("void reset();"));

        let mut logger = LocalSampleLogger::new(1);
        logger.log_sample(4);
        let storage: &dyn SampleStorage = &logger;
        assert_eq!(storage.capacity(), 240);
        assert_eq!(storage.size(), 1);
        assert_eq!(storage.get(0), Ok(4));
    }
}
