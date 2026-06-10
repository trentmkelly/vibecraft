#![allow(dead_code)]

use std::sync::atomic::{AtomicI32, Ordering};

pub trait BandwidthSampleLogger {
    fn log_sample(&self, sample: i64);
}

pub struct BandwidthDebugMonitor<L> {
    bytes_received: AtomicI32,
    bandwidth_logger: L,
}

impl<L> BandwidthDebugMonitor<L> {
    pub fn new(bandwidth_logger: L) -> Self {
        Self {
            bytes_received: AtomicI32::new(0),
            bandwidth_logger,
        }
    }

    pub fn pending_bytes_received(&self) -> i32 {
        self.bytes_received.load(Ordering::Acquire)
    }
}

impl<L: BandwidthSampleLogger> BandwidthDebugMonitor<L> {
    pub fn on_receive(&self, bytes: i32) {
        self.bytes_received.fetch_add(bytes, Ordering::AcqRel);
    }

    pub fn tick(&self) {
        let bytes = self.bytes_received.swap(0, Ordering::AcqRel);
        self.bandwidth_logger.log_sample(i64::from(bytes));
    }
}

#[cfg(test)]
mod tests {
    use super::{BandwidthDebugMonitor, BandwidthSampleLogger};
    use std::sync::Mutex;

    const BANDWIDTH_DEBUG_MONITOR_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/BandwidthDebugMonitor.java"
    );

    #[derive(Default)]
    struct TestSampleLogger {
        samples: Mutex<Vec<i64>>,
    }

    impl TestSampleLogger {
        fn samples(&self) -> Vec<i64> {
            self.samples.lock().unwrap().clone()
        }
    }

    impl BandwidthSampleLogger for TestSampleLogger {
        fn log_sample(&self, sample: i64) {
            self.samples.lock().unwrap().push(sample);
        }
    }

    #[test]
    fn bandwidth_debug_monitor_logs_received_bytes_and_resets_each_tick() {
        for sentinel in [
            "private final AtomicInteger bytesReceived = new AtomicInteger();",
            "private final LocalSampleLogger bandwidthLogger;",
            "this.bytesReceived.getAndAdd(bytes);",
            "this.bandwidthLogger.logSample(this.bytesReceived.getAndSet(0));",
        ] {
            assert!(
                BANDWIDTH_DEBUG_MONITOR_JAVA.contains(sentinel),
                "missing BandwidthDebugMonitor sentinel {sentinel}"
            );
        }

        let logger = TestSampleLogger::default();
        let monitor = BandwidthDebugMonitor::new(logger);

        monitor.on_receive(10);
        monitor.on_receive(15);
        assert_eq!(monitor.pending_bytes_received(), 25);

        monitor.tick();
        assert_eq!(monitor.pending_bytes_received(), 0);

        monitor.on_receive(3);
        monitor.tick();
        monitor.tick();

        assert_eq!(monitor.bandwidth_logger.samples(), vec![25, 3, 0]);
    }
}
