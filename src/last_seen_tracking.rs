#![allow(dead_code)]

use crate::chat_trust::{LastSeenMessages, LastSeenMessagesUpdateModel, MessageSignature};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenTrackedEntryModel {
    pub signature: MessageSignature,
    pub pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesTrackerModel {
    tracked_messages: Vec<Option<LastSeenTrackedEntryModel>>,
    tail: usize,
    offset: i32,
    last_tracked_message: Option<MessageSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenTrackerUpdateModel {
    pub last_seen: LastSeenMessages,
    pub update: LastSeenMessagesUpdateModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesValidatorModel {
    last_seen_count: usize,
    tracked_messages: Vec<Option<LastSeenTrackedEntryModel>>,
    last_pending_message: Option<MessageSignature>,
}

impl LastSeenTrackedEntryModel {
    pub fn new(signature: MessageSignature, pending: bool) -> Self {
        Self { signature, pending }
    }

    pub fn acknowledge(&self) -> Self {
        if self.pending {
            Self {
                signature: self.signature.clone(),
                pending: false,
            }
        } else {
            self.clone()
        }
    }
}

impl LastSeenMessagesTrackerModel {
    pub fn new(last_seen_count: usize) -> Self {
        Self {
            tracked_messages: vec![None; last_seen_count],
            tail: 0,
            offset: 0,
            last_tracked_message: None,
        }
    }

    pub fn add_pending(&mut self, message: MessageSignature, was_shown: bool) -> bool {
        if self.last_tracked_message.as_ref() == Some(&message) {
            return false;
        }

        self.last_tracked_message = Some(message.clone());
        self.add_entry(was_shown.then(|| LastSeenTrackedEntryModel::new(message, true)));
        true
    }

    pub fn ignore_pending(&mut self, pending_message: &MessageSignature) {
        for entry in &mut self.tracked_messages {
            if matches!(entry, Some(tracked) if tracked.pending && tracked.signature == *pending_message)
            {
                *entry = None;
                break;
            }
        }
    }

    pub fn get_and_clear_offset(&mut self) -> i32 {
        let original = self.offset;
        self.offset = 0;
        original
    }

    pub fn generate_and_apply_update(&mut self) -> LastSeenTrackerUpdateModel {
        let offset = self.get_and_clear_offset();
        let mut acknowledged = vec![false; self.tracked_messages.len()];
        let mut last_seen_entries = Vec::new();

        let tracked_len = self.tracked_messages.len();
        for (ack_index, acknowledged_slot) in acknowledged.iter_mut().enumerate().take(tracked_len)
        {
            let index = (self.tail + ack_index) % tracked_len;
            if let Some(message) = &self.tracked_messages[index] {
                *acknowledged_slot = true;
                last_seen_entries.push(message.signature.clone());
                self.tracked_messages[index] = Some(message.acknowledge());
            }
        }

        let last_seen = LastSeenMessages {
            entries: last_seen_entries,
        };
        let update = LastSeenMessagesUpdateModel {
            offset,
            acknowledged,
            checksum: last_seen.compute_checksum(),
        };
        LastSeenTrackerUpdateModel { last_seen, update }
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }

    fn add_entry(&mut self, entry: Option<LastSeenTrackedEntryModel>) {
        if self.tracked_messages.is_empty() {
            return;
        }
        let index = self.tail;
        self.tail = (index + 1) % self.tracked_messages.len();
        self.offset += 1;
        self.tracked_messages[index] = entry;
    }
}

impl LastSeenMessagesValidatorModel {
    pub fn new(last_seen_count: usize) -> Self {
        Self {
            last_seen_count,
            tracked_messages: vec![None; last_seen_count],
            last_pending_message: None,
        }
    }

    pub fn add_pending(&mut self, message: MessageSignature) {
        if self.last_pending_message.as_ref() != Some(&message) {
            self.tracked_messages
                .push(Some(LastSeenTrackedEntryModel::new(message.clone(), true)));
            self.last_pending_message = Some(message);
        }
    }

    pub fn tracked_messages_count(&self) -> usize {
        self.tracked_messages.len()
    }

    pub fn apply_offset(&mut self, offset: i32) -> Result<(), String> {
        let max_offset = self.tracked_messages.len() as i32 - self.last_seen_count as i32;
        if offset >= 0 && offset <= max_offset {
            self.tracked_messages.drain(0..offset as usize);
            Ok(())
        } else {
            Err(format!(
                "Advanced last seen window by {offset} messages, but expected at most {max_offset}"
            ))
        }
    }

    pub fn apply_update(
        &mut self,
        update: &LastSeenMessagesUpdateModel,
    ) -> Result<LastSeenMessages, String> {
        self.apply_offset(update.offset)?;
        let acknowledged_length = bitset_length(&update.acknowledged);
        if acknowledged_length > self.last_seen_count {
            return Err(format!(
                "Last seen update contained {acknowledged_length} messages, but maximum window size is {}",
                self.last_seen_count
            ));
        }

        let mut last_seen_entries = Vec::new();
        for i in 0..self.last_seen_count {
            let acknowledged = update.acknowledged.get(i).copied().unwrap_or(false);
            let message = self.tracked_messages.get(i).cloned().flatten();
            if acknowledged {
                let Some(message) = message else {
                    return Err(format!(
                        "Last seen update acknowledged unknown or previously ignored message at index {i}"
                    ));
                };
                self.tracked_messages[i] = Some(message.acknowledge());
                last_seen_entries.push(message.signature);
            } else if let Some(message) = message {
                if !message.pending {
                    return Err(format!(
                        "Last seen update ignored previously acknowledged message at index {i} and signature {:?}",
                        message.signature
                    ));
                }
                self.tracked_messages[i] = None;
            } else {
                self.tracked_messages[i] = None;
            }
        }

        let last_seen = LastSeenMessages {
            entries: last_seen_entries,
        };
        if update.verify_checksum(&last_seen) {
            Ok(last_seen)
        } else {
            Err(
                "Checksum mismatch on last seen update: the client and server must have desynced"
                    .to_string(),
            )
        }
    }
}

fn bitset_length(bits: &[bool]) -> usize {
    bits.iter()
        .rposition(|bit| *bit)
        .map(|index| index + 1)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(value: u8) -> MessageSignature {
        MessageSignature(vec![value; MessageSignature::BYTES])
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn last_seen_tracking_matches_java_tracker_validator_and_entry_contracts() {
        const TRACKER_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessagesTracker.java"
        );
        const VALIDATOR_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessagesValidator.java"
        );
        const TRACKED_ENTRY_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenTrackedEntry.java"
        );

        for sentinel in [
            "private final @Nullable LastSeenTrackedEntry[] trackedMessages;",
            "if (Objects.equals(message, this.lastTrackedMessage))",
            "this.addEntry(wasShown ? new LastSeenTrackedEntry(message, true) : null);",
            "this.tail = (index + 1) % this.trackedMessages.length;",
            "this.offset++;",
            "entry != null && entry.pending() && pendingMessage.equals(entry.signature())",
            "int originalOffset = this.offset;",
            "int index = (this.tail + i) % this.trackedMessages.length;",
            "acknowledged.set(i, true);",
            "this.trackedMessages[index] = message.acknowledge();",
            "new LastSeenMessages.Update(offset, acknowledged, lastSeen.computeChecksum())",
        ] {
            assert!(
                TRACKER_JAVA.contains(sentinel),
                "missing tracker sentinel {sentinel}"
            );
        }

        for sentinel in [
            "private final ObjectList<LastSeenTrackedEntry> trackedMessages = new ObjectArrayList();",
            "this.trackedMessages.add(null);",
            "if (!message.equals(this.lastPendingMessage))",
            "int maxOffset = this.trackedMessages.size() - this.lastSeenCount;",
            "this.trackedMessages.removeElements(0, offset);",
            "update.acknowledged().length() > this.lastSeenCount",
            "Last seen update acknowledged unknown or previously ignored message at index",
            "Last seen update ignored previously acknowledged message at index",
            "if (!update.verifyChecksum(lastSeen))",
        ] {
            assert!(
                VALIDATOR_JAVA.contains(sentinel),
                "missing validator sentinel {sentinel}"
            );
        }

        assert!(TRACKED_ENTRY_JAVA.contains(
            "return this.pending ? new LastSeenTrackedEntry(this.signature, false) : this;"
        ));

        let pending = LastSeenTrackedEntryModel::new(sig(1), true);
        assert_eq!(
            pending.acknowledge(),
            LastSeenTrackedEntryModel::new(sig(1), false)
        );
        let acknowledged_entry = LastSeenTrackedEntryModel::new(sig(2), false);
        assert_eq!(acknowledged_entry.acknowledge(), acknowledged_entry);

        let mut tracker = LastSeenMessagesTrackerModel::new(3);
        assert!(tracker.add_pending(sig(1), true));
        assert!(!tracker.add_pending(sig(1), true));
        assert!(tracker.add_pending(sig(2), false));
        assert!(tracker.add_pending(sig(3), true));
        assert_eq!(tracker.offset(), 3);
        tracker.ignore_pending(&sig(3));
        let update = tracker.generate_and_apply_update();
        assert_eq!(update.update.offset, 3);
        assert_eq!(update.update.acknowledged, vec![true, false, false]);
        assert_eq!(update.last_seen.entries, vec![sig(1)]);
        assert_eq!(tracker.offset(), 0);

        let mut validator = LastSeenMessagesValidatorModel::new(3);
        validator.add_pending(sig(1));
        validator.add_pending(sig(1));
        validator.add_pending(sig(2));
        validator.add_pending(sig(3));
        assert_eq!(validator.tracked_messages_count(), 6);
        let valid_update = LastSeenMessagesUpdateModel {
            offset: 3,
            acknowledged: vec![true, false, true],
            checksum: LastSeenMessages {
                entries: vec![sig(1), sig(3)],
            }
            .compute_checksum(),
        };
        assert_eq!(
            validator.apply_update(&valid_update),
            Ok(LastSeenMessages {
                entries: vec![sig(1), sig(3)]
            })
        );

        let ignored_acknowledged = validator.apply_update(&LastSeenMessagesUpdateModel {
            offset: 0,
            acknowledged: vec![false, false, true],
            checksum: LastSeenMessagesUpdateModel::IGNORE_CHECKSUM,
        });
        assert!(
            matches!(ignored_acknowledged, Err(message) if message.contains("previously acknowledged"))
        );

        let mut bad_offset = LastSeenMessagesValidatorModel::new(2);
        assert!(bad_offset.apply_offset(1).is_err());

        let mut too_long = LastSeenMessagesValidatorModel::new(2);
        too_long.add_pending(sig(5));
        let too_long_result = too_long.apply_update(&LastSeenMessagesUpdateModel {
            offset: 0,
            acknowledged: vec![false, false, true],
            checksum: LastSeenMessagesUpdateModel::IGNORE_CHECKSUM,
        });
        assert!(matches!(too_long_result, Err(message) if message.contains("maximum window size")));

        let mut unknown = LastSeenMessagesValidatorModel::new(2);
        let unknown_result = unknown.apply_update(&LastSeenMessagesUpdateModel {
            offset: 0,
            acknowledged: vec![true],
            checksum: LastSeenMessagesUpdateModel::IGNORE_CHECKSUM,
        });
        assert!(
            matches!(unknown_result, Err(message) if message.contains("unknown or previously ignored"))
        );

        let mut bad_checksum = LastSeenMessagesValidatorModel::new(1);
        bad_checksum.add_pending(sig(7));
        let bad_checksum_result = bad_checksum.apply_update(&LastSeenMessagesUpdateModel {
            offset: 1,
            acknowledged: vec![true],
            checksum: 99,
        });
        assert!(
            matches!(bad_checksum_result, Err(message) if message.contains("Checksum mismatch"))
        );
    }
}
