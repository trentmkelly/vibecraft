//! Server-wide packet fan-out between player connections.
//!
//! Java broadcasts world changes through `PlayerList.broadcast*` and
//! `ChunkMap.TrackedEntity`, which encode a packet once and hand it to every
//! interested `Connection`. VibeCraft runs one thread per connection, so the
//! shared world tick (scheduled ticks, fluids, fire, ...) publishes its packets
//! on this bus and every session drains its own inbox from its own thread,
//! re-framing each packet with that connection's compression state.
//!
//! Packets travel as *plain payloads* (`VarInt id` + body, no length prefix and
//! no compression), the exact input of [`CompressionState::encode_packet`].

use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::network::compression::CompressionState;
use crate::network::varint::read_var_i32;

/// Per-connection inboxes keyed by the connection's registry token.
type Inboxes = HashMap<u64, VecDeque<Vec<u8>>>;

/// Shared publish/subscribe hub for plain packet payloads.
#[derive(Clone, Default)]
pub struct WorldPacketBus {
    inboxes: Arc<Mutex<Inboxes>>,
}

impl WorldPacketBus {
    fn lock(&self) -> MutexGuard<'_, Inboxes> {
        // A poisoned inbox map only means another session thread panicked
        // mid-push; the queues themselves are still structurally valid.
        self.inboxes.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Registers a connection. Dropping the returned [`Subscription`]
    /// unregisters it and discards anything still queued.
    pub fn subscribe(&self, token: u64) -> Subscription {
        self.lock().insert(token, VecDeque::new());
        Subscription {
            bus: self.clone(),
            token,
        }
    }

    /// Queues `payload` for every subscribed connection
    /// (Java `PlayerList.broadcastAll`).
    pub fn publish(&self, payload: &[u8]) {
        for inbox in self.lock().values_mut() {
            inbox.push_back(payload.to_vec());
        }
    }

    /// Publishes every packet frame in `frames` to all connections.
    ///
    /// `frames` must be the output of the shared world-tick writers run with
    /// [`CompressionState::disabled`], i.e. a sequence of
    /// `VarInt length` + plain payload. Returns an error on truncated input.
    pub fn publish_frames(&self, frames: &[u8]) -> io::Result<()> {
        for payload in split_plain_frames(frames)? {
            self.publish(payload);
        }
        Ok(())
    }
}

/// A connection's handle on the bus; unsubscribes on drop.
pub struct Subscription {
    bus: WorldPacketBus,
    token: u64,
}

impl Subscription {
    /// Writes all queued packets to `writer`, framed for this connection.
    /// Returns how many packets were written.
    pub fn drain_into<W: Write>(
        &self,
        writer: &mut W,
        compression: CompressionState,
    ) -> io::Result<usize> {
        let pending: Vec<Vec<u8>> = match self.bus.lock().get_mut(&self.token) {
            Some(inbox) => inbox.drain(..).collect(),
            None => Vec::new(),
        };
        for payload in &pending {
            writer.write_all(&compression.encode_packet(payload)?)?;
        }
        Ok(pending.len())
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.bus.lock().remove(&self.token);
    }
}

/// Splits a buffer of uncompressed frames (`VarInt length` + payload) into
/// their payload slices.
fn split_plain_frames(mut frames: &[u8]) -> io::Result<Vec<&[u8]>> {
    let mut payloads = Vec::new();
    while !frames.is_empty() {
        let length = read_var_i32(&mut frames)?;
        let length = usize::try_from(length)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "negative frame length"))?;
        if frames.len() < length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated broadcast frame",
            ));
        }
        let (payload, rest) = frames.split_at(length);
        payloads.push(payload);
        frames = rest;
    }
    Ok(payloads)
}

#[cfg(test)]
mod tests;
