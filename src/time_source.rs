//! Time-source traits matching Minecraft's `TimeSource`.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    pub const fn convert_nanoseconds(self, nanoseconds: i64) -> i64 {
        match self {
            Self::Nanoseconds => nanoseconds,
            Self::Microseconds => nanoseconds / 1_000,
            Self::Milliseconds => nanoseconds / 1_000_000,
            Self::Seconds => nanoseconds / 1_000_000_000,
            Self::Minutes => nanoseconds / 60_000_000_000,
            Self::Hours => nanoseconds / 3_600_000_000_000,
            Self::Days => nanoseconds / 86_400_000_000_000,
        }
    }
}

pub trait TimeSource {
    fn get(&self, time_unit: TimeUnit) -> i64;
}

pub trait LongSupplier {
    fn get_as_long(&self) -> i64;
}

pub struct NanoTimeSource<F> {
    supplier: F,
}

impl<F> NanoTimeSource<F> {
    pub const fn new(supplier: F) -> Self {
        Self { supplier }
    }
}

impl<F> LongSupplier for NanoTimeSource<F>
where
    F: Fn() -> i64,
{
    fn get_as_long(&self) -> i64 {
        (self.supplier)()
    }
}

impl<F> TimeSource for NanoTimeSource<F>
where
    F: Fn() -> i64,
{
    fn get(&self, time_unit: TimeUnit) -> i64 {
        time_unit.convert_nanoseconds(self.get_as_long())
    }
}

#[cfg(test)]
mod tests {
    use super::{LongSupplier, NanoTimeSource, TimeSource, TimeUnit};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn time_source_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/TimeSource.java");
        assert_eq!(JAVA.lines().count(), 16);
        for fragment in [
            "long get(TimeUnit timeUnit)",
            "interface NanoTimeSource extends LongSupplier, TimeSource",
            "default long get(final TimeUnit timeUnit)",
            "timeUnit.convert(this.getAsLong(), TimeUnit.NANOSECONDS)",
        ] {
            assert!(JAVA.contains(fragment), "missing TimeSource source fragment: {fragment}");
        }
    }

    #[test]
    fn nano_time_source_converts_units_like_java_timeunit() {
        let source = NanoTimeSource::new(|| 3_600_000_123_456);
        assert_eq!(source.get_as_long(), 3_600_000_123_456);
        assert_eq!(source.get(TimeUnit::Nanoseconds), 3_600_000_123_456);
        assert_eq!(source.get(TimeUnit::Microseconds), 3_600_000_123);
        assert_eq!(source.get(TimeUnit::Milliseconds), 3_600_000);
        assert_eq!(source.get(TimeUnit::Seconds), 3_600);
        assert_eq!(source.get(TimeUnit::Minutes), 60);
        assert_eq!(source.get(TimeUnit::Hours), 1);
        assert_eq!(source.get(TimeUnit::Days), 0);

        let negative = NanoTimeSource::new(|| -1_500_000_000);
        assert_eq!(negative.get(TimeUnit::Seconds), -1);
    }
}
