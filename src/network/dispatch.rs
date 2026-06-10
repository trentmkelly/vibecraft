#![allow(dead_code)]

use std::collections::VecDeque;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

impl PacketFlow {
    pub fn id(self) -> &'static str {
        match self {
            Self::Serverbound => "serverbound",
            Self::Clientbound => "clientbound",
        }
    }

    pub fn get_opposite(self) -> Self {
        match self {
            Self::Serverbound => Self::Clientbound,
            Self::Clientbound => Self::Serverbound,
        }
    }
}

impl std::fmt::Display for PacketFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Serverbound => "SERVERBOUND",
            Self::Clientbound => "CLIENTBOUND",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl ConnectionProtocol {
    pub fn id(self) -> &'static str {
        match self {
            Self::Handshaking => "handshake",
            Self::Play => "play",
            Self::Status => "status",
            Self::Login => "login",
            Self::Configuration => "configuration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectionDetails {
    pub reason: String,
    pub report: Option<PathBuf>,
    pub bug_report_link: Option<String>,
}

impl DisconnectionDetails {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            report: None,
            bug_report_link: None,
        }
    }

    pub fn with_report_and_bug_link(
        reason: impl Into<String>,
        report: Option<PathBuf>,
        bug_report_link: Option<String>,
    ) -> Self {
        Self {
            reason: reason.into(),
            report,
            bug_report_link,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketErrorReport {
    pub protocol: &'static str,
    pub flow: PacketFlow,
    pub packet_state: ProtocolState,
    pub packet_id: i32,
    pub cause: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportConnectionDetails {
    pub protocol: &'static str,
    pub flow: String,
    pub listener_details: Vec<(String, String)>,
}

pub trait JavaPacketListener {
    fn flow(&self) -> PacketFlow;
    fn protocol(&self) -> ConnectionProtocol;
    fn on_disconnect(&mut self, details: DisconnectionDetails);
    fn is_accepting_messages(&self) -> bool;

    fn on_packet_error(
        &self,
        packet: &DecodedPacket,
        cause: impl std::fmt::Display,
    ) -> PacketErrorReport {
        PacketErrorReport {
            protocol: self.protocol().id(),
            flow: self.flow(),
            packet_state: packet.state,
            packet_id: packet.id,
            cause: cause.to_string(),
        }
    }

    fn create_disconnection_info(
        &self,
        reason: impl Into<String>,
        _cause: Option<&str>,
    ) -> DisconnectionDetails {
        DisconnectionDetails::new(reason)
    }

    fn should_handle_message(&self, _packet: &DecodedPacket) -> bool {
        self.is_accepting_messages()
    }

    fn fill_crash_report(&self) -> CrashReportConnectionDetails {
        let mut details = CrashReportConnectionDetails {
            protocol: self.protocol().id(),
            flow: self.flow().to_string(),
            listener_details: Vec::new(),
        };
        self.fill_listener_specific_crash_details(&mut details);
        details
    }

    fn fill_listener_specific_crash_details(&self, _details: &mut CrashReportConnectionDetails) {}
}

pub trait ClientboundPacketListener: JavaPacketListener {
    fn clientbound_flow(&self) -> PacketFlow {
        PacketFlow::Clientbound
    }
}

pub trait ServerboundPacketListener: JavaPacketListener {
    fn serverbound_flow(&self) -> PacketFlow {
        PacketFlow::Serverbound
    }
}

pub trait TickablePacketListener: JavaPacketListener {
    fn tick(&mut self);
}

pub trait SkipPacketException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipPacketFailure {
    Message(String),
    Cause(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipPacketDecoderException {
    pub failure: SkipPacketFailure,
}

impl SkipPacketDecoderException {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            failure: SkipPacketFailure::Message(message.into()),
        }
    }

    pub fn cause(cause: impl Into<String>) -> Self {
        Self {
            failure: SkipPacketFailure::Cause(cause.into()),
        }
    }
}

impl SkipPacketException for SkipPacketDecoderException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipPacketEncoderException {
    pub failure: SkipPacketFailure,
}

impl SkipPacketEncoderException {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            failure: SkipPacketFailure::Message(message.into()),
        }
    }

    pub fn cause(cause: impl Into<String>) -> Self {
        Self {
            failure: SkipPacketFailure::Cause(cause.into()),
        }
    }
}

impl SkipPacketException for SkipPacketEncoderException {}

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
        ClientboundPacketListener, ConnectionProtocol, DecodedPacket, DisconnectionDetails,
        DispatchOutcome, JavaPacketListener, MainThreadPacketQueue, PacketDirection, PacketFlow,
        ProtocolState, RecordingListener, ServerboundPacketListener, SkipPacketDecoderException,
        SkipPacketEncoderException, SkipPacketException, SkipPacketFailure, TickablePacketListener,
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

    #[derive(Debug)]
    struct TestJavaListener {
        flow: PacketFlow,
        protocol: ConnectionProtocol,
        accepting: bool,
        disconnected: Option<DisconnectionDetails>,
        ticks: u32,
    }

    impl TestJavaListener {
        fn new(flow: PacketFlow) -> Self {
            Self {
                flow,
                protocol: ConnectionProtocol::Configuration,
                accepting: true,
                disconnected: None,
                ticks: 0,
            }
        }
    }

    impl JavaPacketListener for TestJavaListener {
        fn flow(&self) -> PacketFlow {
            self.flow
        }

        fn protocol(&self) -> ConnectionProtocol {
            self.protocol
        }

        fn on_disconnect(&mut self, details: DisconnectionDetails) {
            self.disconnected = Some(details);
        }

        fn is_accepting_messages(&self) -> bool {
            self.accepting
        }
    }

    impl ClientboundPacketListener for TestJavaListener {}
    impl ServerboundPacketListener for TestJavaListener {}

    impl TickablePacketListener for TestJavaListener {
        fn tick(&mut self) {
            self.ticks += 1;
        }
    }

    #[test]
    fn packet_flow_and_connection_protocol_match_java_ids() {
        const PACKET_FLOW_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/PacketFlow.java"
        );
        const CONNECTION_PROTOCOL_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/ConnectionProtocol.java"
        );

        for sentinel in [
            "SERVERBOUND(\"serverbound\")",
            "CLIENTBOUND(\"clientbound\")",
            "return this == CLIENTBOUND ? SERVERBOUND : CLIENTBOUND;",
            "public String id()",
        ] {
            assert!(
                PACKET_FLOW_JAVA.contains(sentinel),
                "missing PacketFlow sentinel {sentinel}"
            );
        }
        for sentinel in [
            "HANDSHAKING(\"handshake\")",
            "PLAY(\"play\")",
            "STATUS(\"status\")",
            "LOGIN(\"login\")",
            "CONFIGURATION(\"configuration\")",
            "public String id()",
        ] {
            assert!(
                CONNECTION_PROTOCOL_JAVA.contains(sentinel),
                "missing ConnectionProtocol sentinel {sentinel}"
            );
        }

        assert_eq!(PacketFlow::Serverbound.id(), "serverbound");
        assert_eq!(PacketFlow::Clientbound.id(), "clientbound");
        assert_eq!(
            PacketFlow::Clientbound.get_opposite(),
            PacketFlow::Serverbound
        );
        assert_eq!(
            PacketFlow::Serverbound.get_opposite(),
            PacketFlow::Clientbound
        );
        assert_eq!(PacketFlow::Clientbound.to_string(), "CLIENTBOUND");

        assert_eq!(ConnectionProtocol::Handshaking.id(), "handshake");
        assert_eq!(ConnectionProtocol::Play.id(), "play");
        assert_eq!(ConnectionProtocol::Status.id(), "status");
        assert_eq!(ConnectionProtocol::Login.id(), "login");
        assert_eq!(ConnectionProtocol::Configuration.id(), "configuration");
    }

    #[test]
    fn java_packet_listener_defaults_match_source_contract() {
        const PACKET_LISTENER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/PacketListener.java"
        );
        const DISCONNECTION_DETAILS_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/DisconnectionDetails.java"
        );

        for sentinel in [
            "PacketFlow flow();",
            "ConnectionProtocol protocol();",
            "void onDisconnect(DisconnectionDetails details);",
            "throw PacketUtils.makeReportedException(cause, packet, this);",
            "return new DisconnectionDetails(reason);",
            "return this.isAcceptingMessages();",
            "connection.setDetail(\"Protocol\", () -> this.protocol().id());",
            "connection.setDetail(\"Flow\", () -> this.flow().toString());",
        ] {
            assert!(
                PACKET_LISTENER_JAVA.contains(sentinel),
                "missing PacketListener sentinel {sentinel}"
            );
        }
        assert!(DISCONNECTION_DETAILS_JAVA.contains(
            "public record DisconnectionDetails(Component reason, Optional<Path> report, Optional<URI> bugReportLink)"
        ));
        assert!(DISCONNECTION_DETAILS_JAVA
            .contains("this(reason, Optional.empty(), Optional.empty());"));

        let packet = packet(7);
        let mut listener = TestJavaListener::new(PacketFlow::Clientbound);
        assert!(listener.should_handle_message(&packet));
        listener.accepting = false;
        assert!(!listener.should_handle_message(&packet));

        let info = listener.create_disconnection_info("{\"text\":\"bye\"}", Some("ignored"));
        assert_eq!(info, DisconnectionDetails::new("{\"text\":\"bye\"}"));
        listener.on_disconnect(info.clone());
        assert_eq!(listener.disconnected, Some(info));

        let error = listener.on_packet_error(&packet, "boom");
        assert_eq!(error.protocol, "configuration");
        assert_eq!(error.flow, PacketFlow::Clientbound);
        assert_eq!(error.packet_id, 7);
        assert_eq!(error.cause, "boom");

        let crash = listener.fill_crash_report();
        assert_eq!(crash.protocol, "configuration");
        assert_eq!(crash.flow, "CLIENTBOUND");
        assert!(crash.listener_details.is_empty());
    }

    #[test]
    fn clientbound_serverbound_and_tickable_listener_surfaces_match_java() {
        const CLIENTBOUND_LISTENER_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/ClientboundPacketListener.java");
        const SERVERBOUND_LISTENER_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/ServerboundPacketListener.java");
        const TICKABLE_LISTENER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/TickablePacketListener.java"
        );

        assert!(CLIENTBOUND_LISTENER_JAVA.contains("return PacketFlow.CLIENTBOUND;"));
        assert!(SERVERBOUND_LISTENER_JAVA.contains("return PacketFlow.SERVERBOUND;"));
        assert!(TICKABLE_LISTENER_JAVA.contains("void tick();"));

        let mut listener = TestJavaListener::new(PacketFlow::Clientbound);
        assert_eq!(listener.clientbound_flow(), PacketFlow::Clientbound);
        listener.flow = PacketFlow::Serverbound;
        assert_eq!(listener.serverbound_flow(), PacketFlow::Serverbound);
        listener.tick();
        listener.tick();
        assert_eq!(listener.ticks, 2);
    }

    fn assert_skip_marker<T: SkipPacketException>(_value: &T) {}

    #[test]
    fn skip_packet_exceptions_match_java_marker_wrappers() {
        const SKIP_PACKET_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/SkipPacketException.java"
        );
        const SKIP_DECODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/SkipPacketDecoderException.java"
        );
        const SKIP_ENCODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/SkipPacketEncoderException.java"
        );

        assert!(SKIP_PACKET_JAVA.contains("public interface SkipPacketException"));
        for sentinel in [
            "extends DecoderException",
            "implements IdDispatchCodec.DontDecorateException, SkipPacketException",
            "public SkipPacketDecoderException(final String message)",
            "public SkipPacketDecoderException(final Throwable cause)",
        ] {
            assert!(
                SKIP_DECODER_JAVA.contains(sentinel),
                "missing SkipPacketDecoderException sentinel {sentinel}"
            );
        }
        for sentinel in [
            "extends EncoderException",
            "implements IdDispatchCodec.DontDecorateException, SkipPacketException",
            "public SkipPacketEncoderException(final String message)",
            "public SkipPacketEncoderException(final Throwable cause)",
        ] {
            assert!(
                SKIP_ENCODER_JAVA.contains(sentinel),
                "missing SkipPacketEncoderException sentinel {sentinel}"
            );
        }

        let decoder_message = SkipPacketDecoderException::message("bad decode");
        let decoder_cause = SkipPacketDecoderException::cause("io");
        let encoder_message = SkipPacketEncoderException::message("bad encode");
        let encoder_cause = SkipPacketEncoderException::cause("overflow");

        assert_skip_marker(&decoder_message);
        assert_skip_marker(&encoder_message);
        assert_eq!(
            decoder_message.failure,
            SkipPacketFailure::Message("bad decode".to_string())
        );
        assert_eq!(
            decoder_cause.failure,
            SkipPacketFailure::Cause("io".to_string())
        );
        assert_eq!(
            encoder_message.failure,
            SkipPacketFailure::Message("bad encode".to_string())
        );
        assert_eq!(
            encoder_cause.failure,
            SkipPacketFailure::Cause("overflow".to_string())
        );
    }
}
