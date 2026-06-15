#![allow(dead_code)]

use std::collections::VecDeque;

use crate::network::dispatch::{ConnectionProtocol, DisconnectionDetails, PacketFlow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolInfoModel {
    pub flow: PacketFlow,
    pub protocol: ConnectionProtocol,
    pub has_bundler: bool,
}

impl ProtocolInfoModel {
    pub const fn new(
        flow: PacketFlow,
        protocol: ConnectionProtocol,
        has_bundler: bool,
    ) -> Self {
        Self {
            flow,
            protocol,
            has_bundler,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketListenerModel {
    pub flow: PacketFlow,
    pub protocol: ConnectionProtocol,
    pub tickable: bool,
}

impl PacketListenerModel {
    pub const fn new(flow: PacketFlow, protocol: ConnectionProtocol, tickable: bool) -> Self {
        Self {
            flow,
            protocol,
            tickable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionEvent {
    Send {
        packet: String,
        flush: bool,
        listener: bool,
    },
    Flush,
    ListenerTick,
    DisconnectCallback(String),
    AutoReadDisabled,
    PipelineConfigured(Vec<&'static str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingAction {
    Send {
        packet: String,
        flush: bool,
        listener: bool,
    },
    Flush,
    InitiateServerbound {
        host: String,
        port: i32,
        outbound: ProtocolInfoModel,
        inbound: ProtocolInfoModel,
        listener: PacketListenerModel,
        intent: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConnectionModel {
    receiving: PacketFlow,
    send_login_disconnect: bool,
    pending_actions: VecDeque<PendingAction>,
    channel_known: bool,
    channel_open: bool,
    local_channel: bool,
    address: Option<String>,
    disconnect_listener: Option<PacketListenerModel>,
    packet_listener: Option<PacketListenerModel>,
    disconnection_details: Option<DisconnectionDetails>,
    delayed_disconnect: Option<DisconnectionDetails>,
    encrypted: bool,
    disconnection_handled: bool,
    read_only: bool,
    handling_fault: bool,
    received_packets: i32,
    sent_packets: i32,
    average_received_packets: f32,
    average_sent_packets: f32,
    tick_count: i32,
    bandwidth_monitor_ticks: i32,
    pipeline: Vec<&'static str>,
    events: Vec<ConnectionEvent>,
}

impl ConnectionModel {
    pub fn new(receiving: PacketFlow) -> Self {
        Self {
            receiving,
            send_login_disconnect: true,
            pending_actions: VecDeque::new(),
            channel_known: false,
            channel_open: false,
            local_channel: false,
            address: None,
            disconnect_listener: None,
            packet_listener: None,
            disconnection_details: None,
            delayed_disconnect: None,
            encrypted: false,
            disconnection_handled: false,
            read_only: false,
            handling_fault: false,
            received_packets: 0,
            sent_packets: 0,
            average_received_packets: 0.0,
            average_sent_packets: 0.0,
            tick_count: 0,
            bandwidth_monitor_ticks: 0,
            pipeline: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn configure_serialization(&mut self, inbound_direction: PacketFlow, local: bool) {
        let outbound_direction = inbound_direction.get_opposite();
        let configure_inbound = inbound_direction == PacketFlow::Serverbound;
        let configure_outbound = outbound_direction == PacketFlow::Serverbound;
        self.pipeline = vec![
            if local {
                "local_frame_decoder"
            } else {
                "splitter"
            },
            "flow_control",
            inbound_handler_name(configure_inbound),
            if local {
                "local_frame_encoder"
            } else {
                "prepender"
            },
            outbound_handler_name(configure_outbound),
        ];
        self.events
            .push(ConnectionEvent::PipelineConfigured(self.pipeline.clone()));
    }

    pub fn configure_packet_handler(&mut self) {
        self.pipeline.push("hackfix");
        self.pipeline.push("packet_handler");
    }

    pub fn activate(&mut self, address: impl Into<String>, local_channel: bool) {
        self.channel_known = true;
        self.channel_open = true;
        self.local_channel = local_channel;
        self.address = Some(address.into());
        if let Some(details) = self.delayed_disconnect.take() {
            self.disconnect_details(details);
        }
    }

    pub fn channel_inactive(&mut self) {
        self.disconnect("disconnect.endOfStream");
    }

    pub fn close_channel_without_details(&mut self) {
        self.channel_known = true;
        self.channel_open = false;
    }

    pub fn is_connected(&self) -> bool {
        self.channel_known && self.channel_open
    }

    pub fn is_connecting(&self) -> bool {
        !self.channel_known
    }

    pub fn is_memory_connection(&self) -> bool {
        self.local_channel
    }

    pub fn get_receiving(&self) -> PacketFlow {
        self.receiving
    }

    pub fn get_sending(&self) -> PacketFlow {
        self.receiving.get_opposite()
    }

    pub fn get_loggable_address(&self, log_ips: bool) -> String {
        match (&self.address, log_ips) {
            (None, _) => "local".to_string(),
            (Some(address), true) => address.clone(),
            (Some(_), false) => "IP hidden".to_string(),
        }
    }

    pub fn set_listener_for_serverbound_handshake(
        &mut self,
        listener: PacketListenerModel,
    ) -> Result<(), String> {
        if self.packet_listener.is_some() {
            return Err("Listener already set".to_string());
        }
        if self.receiving == PacketFlow::Serverbound
            && listener.flow == PacketFlow::Serverbound
            && listener.protocol == ConnectionProtocol::Handshaking
        {
            self.packet_listener = Some(listener);
            Ok(())
        } else {
            Err("Invalid initial listener".to_string())
        }
    }

    pub fn setup_inbound_protocol(
        &mut self,
        protocol: ProtocolInfoModel,
        listener: PacketListenerModel,
    ) -> Result<(), String> {
        self.validate_listener(protocol, listener)?;
        if protocol.flow != self.get_receiving() {
            return Err(format!("Invalid inbound protocol: {}", protocol.protocol.id()));
        }
        self.packet_listener = Some(listener);
        self.disconnect_listener = None;
        if protocol.has_bundler {
            self.add_after("decoder", "bundler");
        }
        Ok(())
    }

    pub fn setup_outbound_protocol(&mut self, protocol: ProtocolInfoModel) -> Result<(), String> {
        if protocol.flow != self.get_sending() {
            return Err(format!("Invalid outbound protocol: {}", protocol.protocol.id()));
        }
        if protocol.has_bundler {
            self.add_after("encoder", "unbundler");
        }
        self.send_login_disconnect = protocol.protocol == ConnectionProtocol::Login;
        Ok(())
    }

    pub fn initiate_serverbound_connection(
        &mut self,
        host: impl Into<String>,
        port: i32,
        outbound: ProtocolInfoModel,
        inbound: ProtocolInfoModel,
        listener: PacketListenerModel,
        intent: impl Into<String>,
    ) -> Result<(), String> {
        if outbound.protocol != inbound.protocol {
            return Err("Mismatched initial protocols".to_string());
        }
        let action = PendingAction::InitiateServerbound {
            host: host.into(),
            port,
            outbound,
            inbound,
            listener,
            intent: intent.into(),
        };
        self.disconnect_listener = Some(listener);
        self.run_or_queue(action)
    }

    pub fn send(
        &mut self,
        packet: impl Into<String>,
        listener: bool,
        flush: bool,
    ) -> Result<(), String> {
        let action = PendingAction::Send {
            packet: packet.into(),
            flush,
            listener,
        };
        self.run_or_queue(action)
    }

    pub fn flush_channel(&mut self) -> Result<(), String> {
        self.run_or_queue(PendingAction::Flush)
    }

    pub fn receive_packet(&mut self, should_handle: bool) -> Result<(), String> {
        if !self.is_connected() {
            return Ok(());
        }
        if self.packet_listener.is_none() {
            return Err("Received a packet before the packet listener was initialized".to_string());
        }
        if should_handle {
            self.received_packets += 1;
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        self.flush_queue().ok();
        if self.packet_listener.is_some_and(|listener| listener.tickable) {
            self.events.push(ConnectionEvent::ListenerTick);
        }
        if !self.is_connected() && !self.disconnection_handled {
            self.handle_disconnection();
        }
        if self.channel_known {
            self.events.push(ConnectionEvent::Flush);
        }
        if self.tick_count % 20 == 0 {
            self.tick_second();
        }
        self.tick_count += 1;
    }

    pub fn tick_second(&mut self) {
        self.average_sent_packets = lerp(0.75, self.sent_packets as f32, self.average_sent_packets);
        self.average_received_packets =
            lerp(0.75, self.received_packets as f32, self.average_received_packets);
        self.sent_packets = 0;
        self.received_packets = 0;
        if self.bandwidth_monitor_ticks >= 0 {
            self.bandwidth_monitor_ticks += 1;
        }
    }

    pub fn disconnect(&mut self, reason: impl Into<String>) {
        self.disconnect_details(DisconnectionDetails::new(reason));
    }

    pub fn disconnect_details(&mut self, details: DisconnectionDetails) {
        if !self.channel_known {
            self.delayed_disconnect = Some(details);
        } else if self.is_connected() {
            self.channel_open = false;
            self.disconnection_details = Some(details);
        }
    }

    pub fn exception_caught(&mut self, cause: ConnectionException) {
        match cause {
            ConnectionException::SkipPacket => {}
            ConnectionException::Timeout => {
                if self.is_connected() {
                    self.disconnect("disconnect.timeout");
                }
            }
            ConnectionException::Other(message) => {
                let first_fault = !self.handling_fault;
                self.handling_fault = true;
                if !self.is_connected() {
                    return;
                }
                let reason = format!("disconnect.genericReason: Internal Exception: {message}");
                let details = DisconnectionDetails::new(reason.clone());
                if first_fault {
                    if self.get_sending() == PacketFlow::Clientbound {
                        let packet = if self.send_login_disconnect {
                            "ClientboundLoginDisconnectPacket"
                        } else {
                            "ClientboundDisconnectPacket"
                        };
                        if self.send(packet, true, true).is_err() {
                            self.disconnect_details(details);
                            return;
                        }
                        self.disconnect_details(details);
                    } else {
                        self.disconnect_details(details);
                    }
                    self.set_read_only();
                } else {
                    self.disconnect_details(details);
                }
            }
        }
    }

    pub fn set_encryption_key(&mut self) {
        self.encrypted = true;
        self.add_before("splitter", "decrypt");
        self.add_before("prepender", "encrypt");
    }

    pub fn setup_compression(&mut self, threshold: i32, validate_decompressed: bool) {
        if threshold >= 0 {
            self.upsert_after("splitter", "decompress");
            self.upsert_after("prepender", "compress");
            if validate_decompressed {
                self.events.push(ConnectionEvent::PipelineConfigured(vec![
                    "decompress_validate",
                ]));
            }
        } else {
            self.pipeline
                .retain(|handler| *handler != "decompress" && *handler != "compress");
        }
    }

    pub fn set_read_only(&mut self) {
        if self.channel_known {
            self.read_only = true;
            self.events.push(ConnectionEvent::AutoReadDisabled);
        }
    }

    pub fn handle_disconnection(&mut self) {
        if self.channel_known && !self.channel_open && !self.disconnection_handled {
            self.disconnection_handled = true;
            let listener = self.packet_listener.or(self.disconnect_listener);
            if listener.is_some() {
                let reason = self
                    .disconnection_details
                    .as_ref()
                    .map(|details| details.reason.clone())
                    .unwrap_or_else(|| "multiplayer.disconnect.generic".to_string());
                self.events.push(ConnectionEvent::DisconnectCallback(reason));
            }
        }
    }

    pub fn pipeline(&self) -> &[&'static str] {
        &self.pipeline
    }

    pub fn events(&self) -> &[ConnectionEvent] {
        &self.events
    }

    pub fn pending_len(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn sent_packets(&self) -> i32 {
        self.sent_packets
    }

    pub fn average_sent_packets(&self) -> f32 {
        self.average_sent_packets
    }

    pub fn average_received_packets(&self) -> f32 {
        self.average_received_packets
    }

    pub fn disconnection_details(&self) -> Option<&DisconnectionDetails> {
        self.disconnection_details.as_ref()
    }

    pub fn send_login_disconnect(&self) -> bool {
        self.send_login_disconnect
    }

    pub fn encrypted(&self) -> bool {
        self.encrypted
    }

    fn validate_listener(
        &self,
        protocol: ProtocolInfoModel,
        listener: PacketListenerModel,
    ) -> Result<(), String> {
        if listener.flow != self.receiving {
            return Err(format!(
                "Trying to set listener for wrong side: connection is {}, but listener is {}",
                self.receiving, listener.flow
            ));
        }
        if protocol.protocol != listener.protocol {
            return Err(format!(
                "Listener protocol ({}) does not match requested one {:?}",
                listener.protocol.id(),
                protocol
            ));
        }
        Ok(())
    }

    fn run_or_queue(&mut self, action: PendingAction) -> Result<(), String> {
        if self.is_connected() {
            self.flush_queue()?;
            self.apply_action(action)
        } else {
            self.pending_actions.push_back(action);
            Ok(())
        }
    }

    fn flush_queue(&mut self) -> Result<(), String> {
        if !self.is_connected() {
            return Ok(());
        }
        while let Some(action) = self.pending_actions.pop_front() {
            self.apply_action(action)?;
        }
        Ok(())
    }

    fn apply_action(&mut self, action: PendingAction) -> Result<(), String> {
        match action {
            PendingAction::Send {
                packet,
                flush,
                listener,
            } => {
                self.sent_packets += 1;
                self.events.push(ConnectionEvent::Send {
                    packet,
                    flush,
                    listener,
                });
            }
            PendingAction::Flush => self.events.push(ConnectionEvent::Flush),
            PendingAction::InitiateServerbound {
                host,
                port,
                outbound,
                inbound,
                listener,
                intent,
            } => {
                self.setup_inbound_protocol(inbound, listener)?;
                self.apply_action(PendingAction::Send {
                    packet: format!("ClientIntentionPacket:{host}:{port}:{intent}"),
                    flush: true,
                    listener: false,
                })?;
                self.setup_outbound_protocol(outbound)?;
            }
        }
        Ok(())
    }

    fn add_after(&mut self, anchor: &'static str, handler: &'static str) {
        if let Some(index) = self.pipeline.iter().position(|name| *name == anchor) {
            self.pipeline.insert(index + 1, handler);
        } else {
            self.pipeline.push(handler);
        }
    }

    fn add_before(&mut self, anchor: &'static str, handler: &'static str) {
        if let Some(index) = self.pipeline.iter().position(|name| *name == anchor) {
            self.pipeline.insert(index, handler);
        } else {
            self.pipeline.insert(0, handler);
        }
    }

    fn upsert_after(&mut self, anchor: &'static str, handler: &'static str) {
        if !self.pipeline.contains(&handler) {
            self.add_after(anchor, handler);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionException {
    SkipPacket,
    Timeout,
    Other(String),
}

fn outbound_handler_name(configure_outbound: bool) -> &'static str {
    if configure_outbound {
        "encoder"
    } else {
        "outbound_config"
    }
}

fn inbound_handler_name(configure_inbound: bool) -> &'static str {
    if configure_inbound {
        "decoder"
    } else {
        "inbound_config"
    }
}

fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONNECTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/Connection.java");

    fn serverbound_listener(protocol: ConnectionProtocol) -> PacketListenerModel {
        PacketListenerModel::new(PacketFlow::Serverbound, protocol, false)
    }

    fn clientbound_listener(protocol: ConnectionProtocol) -> PacketListenerModel {
        PacketListenerModel::new(PacketFlow::Clientbound, protocol, false)
    }

    fn protocol(flow: PacketFlow, id: ConnectionProtocol) -> ProtocolInfoModel {
        ProtocolInfoModel::new(flow, id, false)
    }

    #[test]
    fn java_source_sentinels_cover_connection_surface() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "public class Connection extends SimpleChannelInboundHandler<Packet<?>>",
                "private volatile boolean sendLoginDisconnect = true;",
                "private final Queue<Consumer<Connection>> pendingActions = Queues.newConcurrentLinkedQueue();",
                "private static final ProtocolInfo<ServerHandshakePacketListener> INITIAL_PROTOCOL = HandshakeProtocols.SERVERBOUND;",
                "this.pendingActions.add(connection -> connection.sendPacket(packet, listener, flush));",
                "this.averageSentPackets = Mth.lerp(0.75F, this.sentPackets, this.averageSentPackets);",
                "this.channel.pipeline().addBefore(\"splitter\", \"decrypt\", new CipherDecoder(decryptCipher));",
                "this.channel.pipeline().addAfter(\"splitter\", \"decompress\", new CompressionDecoder(threshold, validateDecompressed));",
                "disconnectListener.onDisconnect(details);",
            ],
        );
    }

    #[test]
    fn serialization_pipeline_names_match_java_inbound_outbound_selection() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "boolean configureInbound = inboundDirection == PacketFlow.SERVERBOUND;",
                "boolean configureOutbound = outboundDirection == PacketFlow.SERVERBOUND;",
                "pipeline.addLast(\"splitter\", createFrameDecoder(monitor, local))",
                "inboundHandlerName(configureInbound)",
                "outboundHandlerName(configureOutbound)",
            ],
        );

        let mut server = ConnectionModel::new(PacketFlow::Serverbound);
        server.configure_serialization(PacketFlow::Serverbound, false);
        server.configure_packet_handler();
        assert_eq!(
            server.pipeline(),
            &[
                "splitter",
                "flow_control",
                "decoder",
                "prepender",
                "outbound_config",
                "hackfix",
                "packet_handler",
            ]
        );

        let mut client = ConnectionModel::new(PacketFlow::Clientbound);
        client.configure_serialization(PacketFlow::Clientbound, true);
        assert_eq!(
            client.pipeline(),
            &[
                "local_frame_decoder",
                "flow_control",
                "inbound_config",
                "local_frame_encoder",
                "encoder",
            ]
        );
    }

    #[test]
    fn queued_actions_drain_in_fifo_order_after_connection_opens() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "if (this.isConnected())",
                "this.pendingActions.add(action);",
                "while ((pendingAction = this.pendingActions.poll()) != null)",
            ],
        );
        let mut connection = ConnectionModel::new(PacketFlow::Serverbound);
        connection.send("first", false, false).unwrap();
        connection.flush_channel().unwrap();
        connection.send("second", true, true).unwrap();
        assert_eq!(connection.pending_len(), 3);
        assert_eq!(connection.sent_packets(), 0);

        connection.activate("127.0.0.1:25565", false);
        connection.tick();

        assert_eq!(
            connection.events(),
            &[
                ConnectionEvent::Send {
                    packet: "first".to_string(),
                    flush: false,
                    listener: false,
                },
                ConnectionEvent::Flush,
                ConnectionEvent::Send {
                    packet: "second".to_string(),
                    flush: true,
                    listener: true,
                },
                ConnectionEvent::Flush,
            ]
        );
        assert_eq!(connection.sent_packets(), 0);
        assert_eq!(connection.average_sent_packets(), 0.5);
    }

    #[test]
    fn listener_and_protocol_validation_follow_java_guards() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "throw new IllegalStateException(\"Listener already set\");",
                "throw new IllegalStateException(\"Invalid initial listener\");",
                "Trying to set listener for wrong side: connection is ",
                "Invalid inbound protocol: ",
                "Invalid outbound protocol: ",
            ],
        );
        let mut connection = ConnectionModel::new(PacketFlow::Serverbound);
        assert_eq!(
            connection.set_listener_for_serverbound_handshake(serverbound_listener(
                ConnectionProtocol::Handshaking
            )),
            Ok(())
        );
        assert_eq!(
            connection
                .set_listener_for_serverbound_handshake(serverbound_listener(
                    ConnectionProtocol::Handshaking
                ))
                .unwrap_err(),
            "Listener already set"
        );

        let mut invalid = ConnectionModel::new(PacketFlow::Clientbound);
        assert_eq!(
            invalid
                .set_listener_for_serverbound_handshake(clientbound_listener(
                    ConnectionProtocol::Handshaking
                ))
                .unwrap_err(),
            "Invalid initial listener"
        );

        let mut inbound = ConnectionModel::new(PacketFlow::Serverbound);
        inbound.configure_serialization(PacketFlow::Serverbound, false);
        assert!(inbound
            .setup_inbound_protocol(
                protocol(PacketFlow::Serverbound, ConnectionProtocol::Login),
                serverbound_listener(ConnectionProtocol::Status),
            )
            .unwrap_err()
            .starts_with("Listener protocol"));
        assert_eq!(
            inbound
                .setup_outbound_protocol(protocol(PacketFlow::Serverbound, ConnectionProtocol::Login))
                .unwrap_err(),
            "Invalid outbound protocol: login"
        );
    }

    #[test]
    fn inbound_and_outbound_protocol_setup_mutates_pipeline_and_login_disconnect_flag() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "ctx.pipeline().addAfter(\"decoder\", \"bundler\", newBundler)",
                "ctx.pipeline().addAfter(\"encoder\", \"unbundler\", newUnbundler)",
                "boolean isLoginProtocol = protocol.id() == ConnectionProtocol.LOGIN;",
                "this.sendLoginDisconnect = isLoginProtocol",
            ],
        );
        let mut connection = ConnectionModel::new(PacketFlow::Serverbound);
        connection.configure_serialization(PacketFlow::Serverbound, false);
        connection
            .setup_inbound_protocol(
                ProtocolInfoModel::new(
                    PacketFlow::Serverbound,
                    ConnectionProtocol::Configuration,
                    true,
                ),
                serverbound_listener(ConnectionProtocol::Configuration),
            )
            .unwrap();
        connection
            .setup_outbound_protocol(ProtocolInfoModel::new(
                PacketFlow::Clientbound,
                ConnectionProtocol::Login,
                true,
            ))
            .unwrap();

        assert!(connection.pipeline().contains(&"bundler"));
        assert!(connection.pipeline().contains(&"unbundler"));
        assert!(connection.send_login_disconnect());

        connection
            .setup_outbound_protocol(protocol(PacketFlow::Clientbound, ConnectionProtocol::Play))
            .unwrap();
        assert!(!connection.send_login_disconnect());
    }

    #[test]
    fn initiate_serverbound_connection_queues_handshake_until_connected() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "if (outbound.id() != inbound.id())",
                "this.disconnectListener = listener;",
                "new ClientIntentionPacket(SharedConstants.getCurrentVersion().protocolVersion(), hostName, port, intent)",
            ],
        );
        let mut connection = ConnectionModel::new(PacketFlow::Clientbound);
        assert_eq!(
            connection
                .initiate_serverbound_connection(
                    "localhost",
                    25565,
                    protocol(PacketFlow::Serverbound, ConnectionProtocol::Status),
                    protocol(PacketFlow::Clientbound, ConnectionProtocol::Login),
                    clientbound_listener(ConnectionProtocol::Login),
                    "LOGIN",
                )
                .unwrap_err(),
            "Mismatched initial protocols"
        );

        connection
            .initiate_serverbound_connection(
                "localhost",
                25565,
                protocol(PacketFlow::Serverbound, ConnectionProtocol::Login),
                protocol(PacketFlow::Clientbound, ConnectionProtocol::Login),
                clientbound_listener(ConnectionProtocol::Login),
                "LOGIN",
            )
            .unwrap();
        assert_eq!(connection.pending_len(), 1);

        connection.activate("remote", false);
        connection.tick();
        assert!(connection.events().iter().any(|event| {
            matches!(event, ConnectionEvent::Send { packet, .. } if packet == "ClientIntentionPacket:localhost:25565:LOGIN")
        }));
        assert!(connection.send_login_disconnect());
    }

    #[test]
    fn disconnect_and_disconnection_callback_match_delayed_and_generic_paths() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "this.delayedDisconnect = details;",
                "this.channel.close().awaitUninterruptibly();",
                "Objects.requireNonNullElseGet(",
                "Component.translatable(\"multiplayer.disconnect.generic\")",
            ],
        );
        let mut delayed = ConnectionModel::new(PacketFlow::Serverbound);
        delayed.disconnect("queued");
        assert!(delayed.is_connecting());
        delayed.activate("remote", false);
        assert!(!delayed.is_connected());
        assert_eq!(
            delayed.disconnection_details().map(|details| details.reason.as_str()),
            Some("queued")
        );

        let mut generic = ConnectionModel::new(PacketFlow::Serverbound);
        generic.activate("remote", false);
        generic
            .set_listener_for_serverbound_handshake(serverbound_listener(
                ConnectionProtocol::Handshaking,
            ))
            .unwrap();
        generic.close_channel_without_details();
        generic.handle_disconnection();
        assert_eq!(
            generic.events().last(),
            Some(&ConnectionEvent::DisconnectCallback(
                "multiplayer.disconnect.generic".to_string()
            ))
        );
    }

    #[test]
    fn exception_handling_picks_login_or_common_disconnect_and_sets_read_only() {
        assert_java_contains(
            CONNECTION_JAVA,
            &[
                "if (cause instanceof TimeoutException)",
                "new ClientboundLoginDisconnectPacket(reason) : new ClientboundDisconnectPacket(reason)",
                "LOGGER.debug(\"Double fault\", cause);",
                "this.setReadOnly();",
            ],
        );
        let mut login = ConnectionModel::new(PacketFlow::Serverbound);
        login.activate("remote", false);
        login.exception_caught(ConnectionException::Other("boom".to_string()));
        assert!(login.events().iter().any(|event| {
            matches!(event, ConnectionEvent::Send { packet, .. } if packet == "ClientboundLoginDisconnectPacket")
        }));
        assert!(login.events().contains(&ConnectionEvent::AutoReadDisabled));

        let mut play = ConnectionModel::new(PacketFlow::Serverbound);
        play.activate("remote", false);
        play.setup_outbound_protocol(protocol(PacketFlow::Clientbound, ConnectionProtocol::Play))
            .unwrap();
        play.exception_caught(ConnectionException::Other("boom".to_string()));
        assert!(play.events().iter().any(|event| {
            matches!(event, ConnectionEvent::Send { packet, .. } if packet == "ClientboundDisconnectPacket")
        }));
    }

    #[test]
    fn compression_and_encryption_handlers_use_java_anchor_points() {
        let mut connection = ConnectionModel::new(PacketFlow::Serverbound);
        connection.configure_serialization(PacketFlow::Serverbound, false);
        connection.set_encryption_key();
        connection.setup_compression(256, true);

        assert_eq!(
            connection.pipeline(),
            &[
                "decrypt",
                "splitter",
                "decompress",
                "flow_control",
                "decoder",
                "encrypt",
                "prepender",
                "compress",
                "outbound_config",
            ]
        );
        assert!(connection.encrypted());

        connection.setup_compression(-1, true);
        assert!(!connection.pipeline().contains(&"decompress"));
        assert!(!connection.pipeline().contains(&"compress"));
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
