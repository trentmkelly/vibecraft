//! Priority-ordered FIFO iteration matching Minecraft's `SequencedPriorityIterator`.

#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

#[derive(Debug)]
pub struct SequencedPriorityIterator<T> {
    queues_by_priority: BTreeMap<i32, VecDeque<T>>,
}

impl<T> SequencedPriorityIterator<T> {
    pub fn new() -> Self {
        Self {
            queues_by_priority: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, data: T, priority: i32) {
        self.queues_by_priority
            .entry(priority)
            .or_default()
            .push_back(data);
    }

    pub fn is_empty(&self) -> bool {
        self.queues_by_priority.values().all(VecDeque::is_empty)
    }
}

impl<T> Iterator for SequencedPriorityIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let priority = self
            .queues_by_priority
            .iter()
            .rev()
            .find_map(|(priority, queue)| (!queue.is_empty()).then_some(*priority))?;
        let queue = self.queues_by_priority.get_mut(&priority)?;
        let result = queue.pop_front();
        if queue.is_empty() {
            self.queues_by_priority.remove(&priority);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::SequencedPriorityIterator;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn sequenced_priority_iterator_matches_java_source() {
        const JAVA: &str =
            vibecraft_java_source!("/net/minecraft/util/SequencedPriorityIterator.java");
        assert_eq!(JAVA.lines().count(), 70);
        for fragment in [
            "public final class SequencedPriorityIterator<T> extends AbstractIterator<T>",
            "private static final int MIN_PRIO = Integer.MIN_VALUE",
            "public void add(final T data, final int priority)",
            "queue.addLast(data)",
            "protected @Nullable T computeNext()",
            "removeFirst()",
            "switchCacheToNextHighestPrioQueue",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing SequencedPriorityIterator source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn sequenced_priority_iterator_preserves_priority_and_fifo_order() {
        let mut iterator = SequencedPriorityIterator::new();
        assert!(iterator.next().is_none());
        iterator.add("low-1", -1);
        iterator.add("high-1", 5);
        iterator.add("high-2", 5);
        iterator.add("middle", 2);
        iterator.add("low-2", -1);
        assert!(!iterator.is_empty());
        let output: Vec<_> = (&mut iterator).collect();
        assert_eq!(output, ["high-1", "high-2", "middle", "low-1", "low-2"]);
        assert!(iterator.is_empty());
        assert_eq!(iterator.next(), None);
    }
}
