#![allow(dead_code)]

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

const LATEST_ENTRY_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressedThrowable {
    class_name: String,
    message: Option<String>,
}

impl SuppressedThrowable {
    pub fn new(class_name: impl Into<String>, message: Option<impl Into<String>>) -> Self {
        Self {
            class_name: class_name.into(),
            message: message.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LongEntry {
    timestamp_ms: i64,
    location: String,
    class_name: String,
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CountEntry {
    location: String,
    class_name: String,
    count: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SuppressedExceptionCollector {
    latest_entries: VecDeque<LongEntry>,
    entry_counts: Vec<CountEntry>,
}

impl SuppressedExceptionCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, location: impl Into<String>, throwable: SuppressedThrowable) {
        self.add_entry_at(current_time_ms(), location, throwable);
    }

    pub fn add_entry_at(
        &mut self,
        timestamp_ms: i64,
        location: impl Into<String>,
        throwable: SuppressedThrowable,
    ) {
        let location = location.into();
        self.latest_entries.push_back(LongEntry {
            timestamp_ms,
            location: location.clone(),
            class_name: throwable.class_name.clone(),
            message: throwable.message.clone(),
        });

        while self.latest_entries.len() > LATEST_ENTRY_COUNT {
            self.latest_entries.pop_front();
        }

        let count = self.remove_count_entry(&location, &throwable.class_name) + 1;
        self.entry_counts.insert(
            0,
            CountEntry {
                location,
                class_name: throwable.class_name,
                count,
            },
        );
    }

    pub fn dump(&self) -> String {
        self.dump_at(current_time_ms())
    }

    pub fn dump_at(&self, current_time_ms: i64) -> String {
        let mut result = String::new();
        if !self.latest_entries.is_empty() {
            result.push_str("\n\t\tLatest entries:\n");

            for entry in &self.latest_entries {
                result.push_str("\t\t\t");
                result.push_str(&entry.location);
                result.push(':');
                result.push_str(&entry.class_name);
                result.push_str(": ");
                result.push_str(entry.message.as_deref().unwrap_or("null"));
                result.push_str(" (");
                result.push_str(&(current_time_ms - entry.timestamp_ms).to_string());
                result.push_str("ms ago)\n");
            }
        }

        if !self.entry_counts.is_empty() {
            if result.is_empty() {
                result.push('\n');
            }

            result.push_str("\t\tEntry counts:\n");
            for entry in &self.entry_counts {
                result.push_str("\t\t\t");
                result.push_str(&entry.location);
                result.push(':');
                result.push_str(&entry.class_name);
                result.push_str(" x ");
                result.push_str(&entry.count.to_string());
                result.push('\n');
            }
        }

        if result.is_empty() {
            "~~NONE~~".to_string()
        } else {
            result
        }
    }

    pub fn latest_entry_count(&self) -> usize {
        self.latest_entries.len()
    }

    fn remove_count_entry(&mut self, location: &str, class_name: &str) -> i32 {
        let Some(index) = self
            .entry_counts
            .iter()
            .position(|entry| entry.location == location && entry.class_name == class_name)
        else {
            return 0;
        };
        self.entry_counts.remove(index).count
    }
}

fn current_time_ms() -> i64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/SuppressedExceptionCollector.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_suppressed_exception_collector_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("private static final int LATEST_ENTRY_COUNT = 8;"));
        assert!(JAVA_SOURCE.contains("private final Queue<SuppressedExceptionCollector.LongEntry> latestEntries"));
        assert!(JAVA_SOURCE.contains("Object2IntLinkedOpenHashMap<SuppressedExceptionCollector.ShortEntry> entryCounts"));
        assert!(JAVA_SOURCE.contains("this.latestEntries.add(new SuppressedExceptionCollector.LongEntry"));
        assert!(JAVA_SOURCE.contains("while (this.latestEntries.size() > 8)"));
        assert!(JAVA_SOURCE.contains("this.latestEntries.remove();"));
        assert!(JAVA_SOURCE.contains("this.entryCounts.putAndMoveToFirst(key, currentValue + 1);"));
        assert!(JAVA_SOURCE.contains("result.append(\"\\n\\t\\tLatest entries:\\n\");"));
        assert!(JAVA_SOURCE.contains("result.append(\"\\t\\tEntry counts:\\n\");"));
        assert!(JAVA_SOURCE.contains("return result.isEmpty() ? \"~~NONE~~\" : result.toString();"));
    }

    #[test]
    fn server_utility_suppressed_exception_collector_empty_dump_matches_java() {
        let collector = SuppressedExceptionCollector::new();
        assert_eq!(collector.dump_at(100), "~~NONE~~");
    }

    #[test]
    fn server_utility_suppressed_exception_collector_dumps_latest_and_counts() {
        let mut collector = SuppressedExceptionCollector::new();
        collector.add_entry_at(
            1000,
            "chunk[0,0]",
            SuppressedThrowable::new("class java.lang.IllegalStateException", Some("bad state")),
        );
        collector.add_entry_at(
            1100,
            "chunk[1,0]",
            SuppressedThrowable::new("class java.lang.RuntimeException", None::<String>),
        );
        collector.add_entry_at(
            1200,
            "chunk[0,0]",
            SuppressedThrowable::new("class java.lang.IllegalStateException", Some("bad again")),
        );

        assert_eq!(
            collector.dump_at(1500),
            "\n\t\tLatest entries:\n\
             \t\t\tchunk[0,0]:class java.lang.IllegalStateException: bad state (500ms ago)\n\
             \t\t\tchunk[1,0]:class java.lang.RuntimeException: null (400ms ago)\n\
             \t\t\tchunk[0,0]:class java.lang.IllegalStateException: bad again (300ms ago)\n\
             \t\tEntry counts:\n\
             \t\t\tchunk[0,0]:class java.lang.IllegalStateException x 2\n\
             \t\t\tchunk[1,0]:class java.lang.RuntimeException x 1\n"
        );
    }

    #[test]
    fn server_utility_suppressed_exception_collector_keeps_only_latest_eight() {
        let mut collector = SuppressedExceptionCollector::new();
        for index in 0..10 {
            collector.add_entry_at(
                index,
                format!("loc-{index}"),
                SuppressedThrowable::new("class java.lang.Exception", Some(format!("msg-{index}"))),
            );
        }

        let dump = collector.dump_at(20);
        let latest = dump
            .split("\t\tEntry counts:\n")
            .next()
            .unwrap_or_else(|| panic!("dump should include latest section"));
        assert_eq!(collector.latest_entry_count(), 8);
        assert!(!latest.contains("loc-0:class java.lang.Exception"));
        assert!(!latest.contains("loc-1:class java.lang.Exception"));
        assert!(latest.contains("loc-2:class java.lang.Exception: msg-2 (18ms ago)"));
        assert!(latest.contains("loc-9:class java.lang.Exception: msg-9 (11ms ago)"));
        assert!(dump.contains("loc-9:class java.lang.Exception x 1"));
        assert!(dump.contains("loc-0:class java.lang.Exception x 1"));
    }
}
