#![allow(dead_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPacket {
    pub state: ProtocolState,
    pub direction: PacketDirection,
    pub id: i32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    Serverbound,
    Clientbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    Handled,
    Disconnect(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseStep {
    SendDisconnect(String),
    SetReadOnly,
    HandleDisconnection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosePlan {
    steps: Vec<CloseStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedPacket {
    sequence: u64,
    packet: DecodedPacket,
}

#[derive(Debug, Default)]
pub struct MainThreadPacketQueue {
    next_sequence: u64,
    queue: VecDeque<QueuedPacket>,
}

pub trait PacketListener {
    fn handle_packet(&mut self, packet: DecodedPacket) -> DispatchOutcome;
}

impl MainThreadPacketQueue {
    pub fn enqueue(&mut self, packet: DecodedPacket) {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.queue.push_back(QueuedPacket { sequence, packet });
    }

    pub fn drain_into<L: PacketListener>(&mut self, listener: &mut L) -> Vec<DispatchOutcome> {
        let mut outcomes = Vec::with_capacity(self.queue.len());
        while let Some(queued) = self.queue.pop_front() {
            outcomes.push(listener.handle_packet(queued.packet));
        }
        outcomes
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
}

impl ClosePlan {
    pub fn disconnect(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            steps: vec![
                CloseStep::SendDisconnect(reason),
                CloseStep::SetReadOnly,
                CloseStep::HandleDisconnection,
            ],
        }
    }

    pub fn steps(&self) -> &[CloseStep] {
        &self.steps
    }
}

#[derive(Debug, Default)]
pub struct RecordingListener {
    accepted: Vec<(ProtocolState, i32)>,
    disconnect_on: Option<i32>,
}

impl RecordingListener {
    pub fn disconnecting_on(packet_id: i32) -> Self {
        Self {
            accepted: Vec::new(),
            disconnect_on: Some(packet_id),
        }
    }

    pub fn accepted(&self) -> &[(ProtocolState, i32)] {
        &self.accepted
    }
}

impl PacketListener for RecordingListener {
    fn handle_packet(&mut self, packet: DecodedPacket) -> DispatchOutcome {
        if self.disconnect_on == Some(packet.id) {
            DispatchOutcome::Disconnect(format!(
                "unexpected packet {} in {:?}",
                packet.id, packet.state
            ))
        } else {
            self.accepted.push((packet.state, packet.id));
            DispatchOutcome::Handled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DecodedPacket, DispatchOutcome, MainThreadPacketQueue, PacketDirection, ProtocolState,
        RecordingListener,
    };

    fn packet(id: i32) -> DecodedPacket {
        DecodedPacket {
            state: ProtocolState::Configuration,
            direction: PacketDirection::Serverbound,
            id,
            payload: vec![id as u8],
        }
    }

    #[test]
    fn dispatch_queue_preserves_packet_order_for_main_thread_handoff() {
        let mut queue = MainThreadPacketQueue::default();
        queue.enqueue(packet(1));
        queue.enqueue(packet(2));
        queue.enqueue(packet(3));
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.next_sequence(), 3);

        let mut listener = RecordingListener::default();
        let outcomes = queue.drain_into(&mut listener);
        assert_eq!(outcomes, vec![DispatchOutcome::Handled; 3]);
        assert_eq!(
            listener.accepted(),
            &[
                (ProtocolState::Configuration, 1),
                (ProtocolState::Configuration, 2),
                (ProtocolState::Configuration, 3)
            ]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn dispatch_queue_reports_disconnect_outcomes_without_panicking() {
        let mut queue = MainThreadPacketQueue::default();
        queue.enqueue(packet(1));
        queue.enqueue(packet(99));

        let mut listener = RecordingListener::disconnecting_on(99);
        let outcomes = queue.drain_into(&mut listener);
        assert_eq!(
            outcomes,
            vec![
                DispatchOutcome::Handled,
                DispatchOutcome::Disconnect("unexpected packet 99 in Configuration".to_string())
            ]
        );
    }

    #[test]
    fn close_plan_matches_vanilla_disconnect_ordering() {
        let plan = super::ClosePlan::disconnect("{\"text\":\"bye\"}");
        assert_eq!(
            plan.steps(),
            &[
                super::CloseStep::SendDisconnect("{\"text\":\"bye\"}".to_string()),
                super::CloseStep::SetReadOnly,
                super::CloseStep::HandleDisconnection,
            ]
        );
    }
}
