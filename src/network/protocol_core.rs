#![allow(dead_code)]

use super::dispatch::PacketFlow;
use super::dispatch::{ConnectionProtocol, ProtocolInfoDetails, ProtocolPacketDetails};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketTypeModel {
    pub flow: PacketFlow,
    pub id: String,
}

impl PacketTypeModel {
    pub fn new(flow: PacketFlow, id: impl Into<String>) -> Self {
        Self {
            flow,
            id: id.into(),
        }
    }
}

impl std::fmt::Display for PacketTypeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.flow.id(), self.id)
    }
}

pub trait PacketModel {
    fn packet_type(&self) -> &PacketTypeModel;
    fn handle(&mut self);

    fn is_skippable(&self) -> bool {
        false
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingPacketModel {
    packet_type: PacketTypeModel,
    skippable: bool,
    terminal: bool,
    handled: bool,
}

impl RecordingPacketModel {
    pub fn new(packet_type: PacketTypeModel) -> Self {
        Self {
            packet_type,
            skippable: false,
            terminal: false,
            handled: false,
        }
    }

    pub fn skippable(mut self) -> Self {
        self.skippable = true;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }

    pub fn handled(&self) -> bool {
        self.handled
    }
}

impl PacketModel for RecordingPacketModel {
    fn packet_type(&self) -> &PacketTypeModel {
        &self.packet_type
    }

    fn handle(&mut self) {
        self.handled = true;
    }

    fn is_skippable(&self) -> bool {
        self.skippable
    }

    fn is_terminal(&self) -> bool {
        self.terminal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningOnDifferentThread;

pub fn ensure_running_on_same_thread(
    is_same_thread: bool,
    mut schedule_if_possible: impl FnMut(),
) -> Result<(), RunningOnDifferentThread> {
    if is_same_thread {
        Ok(())
    } else {
        schedule_if_possible();
        Err(RunningOnDifferentThread)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketCrashDetails {
    pub packet_type: Option<String>,
    pub is_terminal: Option<String>,
    pub is_skippable: Option<String>,
    pub listener_details_filled: bool,
}

pub fn fill_packet_crash_report(packet: Option<&dyn PacketModel>) -> PacketCrashDetails {
    let (packet_type, is_terminal, is_skippable) = packet
        .map(|packet| {
            (
                Some(packet.packet_type().to_string()),
                Some(packet.is_terminal().to_string()),
                Some(packet.is_skippable().to_string()),
            )
        })
        .unwrap_or((None, None, None));
    PacketCrashDetails {
        packet_type,
        is_terminal,
        is_skippable,
        listener_details_filled: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedPacketException {
    pub reused_existing_report: bool,
    pub title: String,
    pub details: PacketCrashDetails,
}

pub fn make_reported_packet_exception(
    already_reported: bool,
    packet: &dyn PacketModel,
) -> ReportedPacketException {
    ReportedPacketException {
        reused_existing_report: already_reported,
        title: "Main thread packet handler".to_string(),
        details: fill_packet_crash_report(Some(packet)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCodecEntryModel {
    pub packet_type: PacketTypeModel,
    pub serializer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCodecBuilderModel {
    flow: PacketFlow,
    entries: Vec<ProtocolCodecEntryModel>,
}

impl ProtocolCodecBuilderModel {
    pub fn new(flow: PacketFlow) -> Self {
        Self {
            flow,
            entries: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        packet_type: PacketTypeModel,
        serializer: impl Into<String>,
    ) -> Result<&mut Self, String> {
        if packet_type.flow != self.flow {
            return Err(format!(
                "Invalid packet flow for packet {packet_type}, expected {}",
                self.flow
            ));
        }
        self.entries.push(ProtocolCodecEntryModel {
            packet_type,
            serializer: serializer.into(),
        });
        Ok(self)
    }

    pub fn build(&self) -> Vec<ProtocolCodecEntryModel> {
        self.entries.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfoBuilderEntryModel {
    pub packet_type: PacketTypeModel,
    pub serializer: String,
    pub modifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltProtocolInfoModel {
    pub protocol: ConnectionProtocol,
    pub flow: PacketFlow,
    pub codec_entries: Vec<ProtocolCodecEntryModel>,
    pub bundler_info: Option<String>,
    pub details: ProtocolInfoDetails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfoBuilderModel {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    entries: Vec<ProtocolInfoBuilderEntryModel>,
    bundler_info: Option<String>,
}

impl ProtocolInfoBuilderModel {
    pub fn new(protocol: ConnectionProtocol, flow: PacketFlow) -> Self {
        Self {
            protocol,
            flow,
            entries: Vec::new(),
            bundler_info: None,
        }
    }

    pub fn add_packet(
        &mut self,
        packet_type: PacketTypeModel,
        serializer: impl Into<String>,
    ) -> &mut Self {
        self.entries.push(ProtocolInfoBuilderEntryModel {
            packet_type,
            serializer: serializer.into(),
            modifier: None,
        });
        self
    }

    pub fn add_packet_with_modifier(
        &mut self,
        packet_type: PacketTypeModel,
        serializer: impl Into<String>,
        modifier: impl Into<String>,
    ) -> &mut Self {
        self.entries.push(ProtocolInfoBuilderEntryModel {
            packet_type,
            serializer: serializer.into(),
            modifier: Some(modifier.into()),
        });
        self
    }

    pub fn with_bundle_packet(
        &mut self,
        bundler_packet: PacketTypeModel,
        delimiter_packet: PacketTypeModel,
    ) -> &mut Self {
        self.entries.push(ProtocolInfoBuilderEntryModel {
            packet_type: delimiter_packet,
            serializer: "StreamCodec.unit(delimiterPacket)".to_string(),
            modifier: None,
        });
        self.bundler_info = Some(format!("BundlerInfo.createForPacket({bundler_packet})"));
        self
    }

    pub fn build_details(&self) -> ProtocolInfoDetails {
        ProtocolInfoDetails {
            id: self.protocol,
            flow: self.flow,
            packets: self
                .entries
                .iter()
                .enumerate()
                .map(|(network_id, entry)| ProtocolPacketDetails {
                    packet_type: Box::leak(entry.packet_type.to_string().into_boxed_str()),
                    network_id: network_id as i32,
                })
                .collect(),
        }
    }

    pub fn build_bound(&self, context: &str) -> Result<BuiltProtocolInfoModel, String> {
        let mut codec_builder = ProtocolCodecBuilderModel::new(self.flow);
        for entry in &self.entries {
            let serializer = match &entry.modifier {
                Some(modifier) => format!("{modifier}({},{context})", entry.serializer),
                None => entry.serializer.clone(),
            };
            codec_builder.add(entry.packet_type.clone(), serializer)?;
        }
        Ok(BuiltProtocolInfoModel {
            protocol: self.protocol,
            flow: self.flow,
            codec_entries: codec_builder.build(),
            bundler_info: self.bundler_info.clone(),
            details: self.build_details(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUnboundProtocolModel {
    builder: ProtocolInfoBuilderModel,
    context: String,
}

impl SimpleUnboundProtocolModel {
    pub fn bind(&self, context_wrapper: &str) -> Result<BuiltProtocolInfoModel, String> {
        self.builder
            .build_bound(&format!("{context_wrapper}:{}", self.context))
    }

    pub fn details(&self) -> ProtocolInfoDetails {
        self.builder.build_details()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnboundProtocolModel {
    builder: ProtocolInfoBuilderModel,
}

impl UnboundProtocolModel {
    pub fn bind(&self, context_wrapper: &str, context: &str) -> Result<BuiltProtocolInfoModel, String> {
        self.builder.build_bound(&format!("{context_wrapper}:{context}"))
    }

    pub fn details(&self) -> ProtocolInfoDetails {
        self.builder.build_details()
    }
}

pub fn build_simple_unbound_protocol(
    builder: ProtocolInfoBuilderModel,
    context: impl Into<String>,
) -> SimpleUnboundProtocolModel {
    SimpleUnboundProtocolModel {
        builder,
        context: context.into(),
    }
}

pub fn build_unbound_protocol(builder: ProtocolInfoBuilderModel) -> UnboundProtocolModel {
    UnboundProtocolModel { builder }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const PACKET_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/Packet.java");
    const PACKET_TYPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/PacketType.java");
    const PACKET_UTILS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/PacketUtils.java");
    const PROTOCOL_CODEC_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/ProtocolCodecBuilder.java");
    const PROTOCOL_INFO_BUILDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/ProtocolInfoBuilder.java");
    const SIMPLE_UNBOUND_PROTOCOL_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/SimpleUnboundProtocol.java");
    const UNBOUND_PROTOCOL_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/UnboundProtocol.java");

    #[test]
    fn packet_and_packet_type_match_java_surface() {
        for sentinel in [
            "PacketType<? extends Packet<T>> type();",
            "void handle(T listener);",
            "default boolean isSkippable()",
            "return false;",
            "default boolean isTerminal()",
            "return StreamCodec.ofMember(writer, reader);",
        ] {
            assert!(
                PACKET_JAVA.contains(sentinel),
                "missing Packet Java sentinel {sentinel}"
            );
        }
        for sentinel in [
            "public record PacketType<T extends Packet<?>>(PacketFlow flow, Identifier id)",
            "return this.flow.id() + \"/\" + this.id;",
        ] {
            assert!(
                PACKET_TYPE_JAVA.contains(sentinel),
                "missing PacketType Java sentinel {sentinel}"
            );
        }

        let packet_type = PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:add_entity");
        assert_eq!(packet_type.to_string(), "clientbound/minecraft:add_entity");
        let mut packet = RecordingPacketModel::new(packet_type);
        assert!(!packet.is_skippable());
        assert!(!packet.is_terminal());
        assert!(!packet.handled());
        packet.handle();
        assert!(packet.handled());

        let special = RecordingPacketModel::new(PacketTypeModel::new(
            PacketFlow::Serverbound,
            "minecraft:finish_configuration",
        ))
        .skippable()
        .terminal();
        assert!(special.is_skippable());
        assert!(special.is_terminal());
    }

    #[test]
    fn packet_utils_match_java_thread_and_crash_report_contract() {
        for sentinel in [
            "if (!packetProcessor.isSameThread())",
            "packetProcessor.scheduleIfPossible(listener, packet);",
            "throw RunningOnDifferentThreadException.RUNNING_ON_DIFFERENT_THREAD;",
            "if (cause instanceof ReportedException re)",
            "CrashReport.forThrowable(cause, \"Main thread packet handler\")",
            "CrashReportCategory details = report.addCategory(\"Incoming Packet\");",
            "details.setDetail(\"Type\", () -> packet.type().toString());",
            "details.setDetail(\"Is Terminal\", () -> Boolean.toString(packet.isTerminal()));",
            "details.setDetail(\"Is Skippable\", () -> Boolean.toString(packet.isSkippable()));",
            "listener.fillCrashReport(report);",
        ] {
            assert!(
                PACKET_UTILS_JAVA.contains(sentinel),
                "missing PacketUtils Java sentinel {sentinel}"
            );
        }

        let mut scheduled = 0;
        assert_eq!(
            ensure_running_on_same_thread(true, || scheduled += 1),
            Ok(())
        );
        assert_eq!(scheduled, 0);
        assert_eq!(
            ensure_running_on_same_thread(false, || scheduled += 1),
            Err(RunningOnDifferentThread)
        );
        assert_eq!(scheduled, 1);

        let packet = RecordingPacketModel::new(PacketTypeModel::new(
            PacketFlow::Serverbound,
            "minecraft:chat",
        ))
        .skippable();
        assert_eq!(
            fill_packet_crash_report(Some(&packet)),
            PacketCrashDetails {
                packet_type: Some("serverbound/minecraft:chat".to_string()),
                is_terminal: Some("false".to_string()),
                is_skippable: Some("true".to_string()),
                listener_details_filled: true,
            }
        );
        assert_eq!(
            fill_packet_crash_report(None),
            PacketCrashDetails {
                packet_type: None,
                is_terminal: None,
                is_skippable: None,
                listener_details_filled: true,
            }
        );
        assert_eq!(
            make_reported_packet_exception(true, &packet),
            ReportedPacketException {
                reused_existing_report: true,
                title: "Main thread packet handler".to_string(),
                details: fill_packet_crash_report(Some(&packet)),
            }
        );
    }

    #[test]
    fn protocol_codec_builder_enforces_java_flow_and_builds_dispatch_entries() {
        for sentinel in [
            "IdDispatchCodec.builder(Packet::type)",
            "private final PacketFlow flow;",
            "if (type.flow() != this.flow)",
            "throw new IllegalArgumentException(\"Invalid packet flow for packet \" + type + \", expected \" + this.flow.name());",
            "this.dispatchBuilder.add(type, serializer);",
            "return this.dispatchBuilder.build();",
        ] {
            assert!(
                PROTOCOL_CODEC_BUILDER_JAVA.contains(sentinel),
                "missing ProtocolCodecBuilder Java sentinel {sentinel}"
            );
        }

        let mut builder = ProtocolCodecBuilderModel::new(PacketFlow::Clientbound);
        let add_entity = PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:add_entity");
        builder
            .add(add_entity.clone(), "ClientboundAddEntityPacket.STREAM_CODEC")
            .expect("matching flow");
        assert_eq!(
            builder.build(),
            vec![ProtocolCodecEntryModel {
                packet_type: add_entity,
                serializer: "ClientboundAddEntityPacket.STREAM_CODEC".to_string(),
            }]
        );
        let err = builder
            .add(
                PacketTypeModel::new(PacketFlow::Serverbound, "minecraft:chat"),
                "ServerboundChatPacket.STREAM_CODEC",
            )
            .unwrap_err();
        assert_eq!(
            err,
            "Invalid packet flow for packet serverbound/minecraft:chat, expected CLIENTBOUND"
        );
    }

    #[test]
    fn protocol_info_builder_matches_java_details_bundle_and_bind_contracts() {
        for sentinel in [
            "private final List<ProtocolInfoBuilder.CodecEntry<T, ?, B, C>> codecs = new ArrayList<>();",
            "this.codecs.add(new ProtocolInfoBuilder.CodecEntry<>(type, serializer, null));",
            "this.codecs.add(new ProtocolInfoBuilder.CodecEntry<>(type, serializer, modifier));",
            "StreamCodec<ByteBuf, D> delimitedCodec = StreamCodec.unit(delimiterPacket);",
            "this.bundlerInfo = BundlerInfo.createForPacket(bundlerPacket, constructor, delimiterPacket);",
            "ProtocolCodecBuilder<ByteBuf, T> codecBuilder = new ProtocolCodecBuilder<>(this.flow);",
            "output.accept(entry.type, i);",
            "final List<ProtocolInfoBuilder.CodecEntry<T, ?, B, C>> codecs = List.copyOf(this.codecs);",
            "return new ProtocolInfoBuilder.Implementation<>(",
            "finalSerializer = this.modifier.apply(this.serializer, context);",
            "StreamCodec<ByteBuf, P> baseCodec = finalSerializer.mapStream(contextWrapper);",
            "codecBuilder.add(this.type, baseCodec);",
        ] {
            assert!(
                PROTOCOL_INFO_BUILDER_JAVA.contains(sentinel),
                "missing ProtocolInfoBuilder Java sentinel {sentinel}"
            );
        }

        let mut builder =
            ProtocolInfoBuilderModel::new(ConnectionProtocol::Configuration, PacketFlow::Clientbound);
        let finish = PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:finish_configuration");
        let delimiter = PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:bundle_delimiter");
        builder
            .add_packet(finish.clone(), "FinishConfiguration.STREAM_CODEC")
            .add_packet_with_modifier(
                PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:custom_payload"),
                "CustomPayload.STREAM_CODEC",
                "registryFriendly",
            )
            .with_bundle_packet(
                PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:bundle"),
                delimiter.clone(),
            );

        let details = builder.build_details();
        assert_eq!(details.id, ConnectionProtocol::Configuration);
        assert_eq!(details.flow, PacketFlow::Clientbound);
        let mut listed = Vec::new();
        details.list_packets(|packet_type, network_id| listed.push((packet_type, network_id)));
        assert_eq!(
            listed,
            vec![
                ("clientbound/minecraft:finish_configuration", 0),
                ("clientbound/minecraft:custom_payload", 1),
                ("clientbound/minecraft:bundle_delimiter", 2),
            ]
        );

        let bound = builder.build_bound("ctx").expect("bound protocol");
        assert_eq!(bound.protocol, ConnectionProtocol::Configuration);
        assert_eq!(bound.bundler_info, Some("BundlerInfo.createForPacket(clientbound/minecraft:bundle)".to_string()));
        assert_eq!(
            bound.codec_entries,
            vec![
                ProtocolCodecEntryModel {
                    packet_type: finish,
                    serializer: "FinishConfiguration.STREAM_CODEC".to_string(),
                },
                ProtocolCodecEntryModel {
                    packet_type: PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:custom_payload"),
                    serializer: "registryFriendly(CustomPayload.STREAM_CODEC,ctx)".to_string(),
                },
                ProtocolCodecEntryModel {
                    packet_type: delimiter,
                    serializer: "StreamCodec.unit(delimiterPacket)".to_string(),
                },
            ]
        );
    }

    #[test]
    fn simple_and_context_unbound_protocols_match_java_bind_surface() {
        assert!(SIMPLE_UNBOUND_PROTOCOL_JAVA
            .contains("ProtocolInfo<T> bind(Function<ByteBuf, B> contextWrapper);"));
        assert!(SIMPLE_UNBOUND_PROTOCOL_JAVA.contains("extends ProtocolInfo.DetailsProvider"));
        assert!(UNBOUND_PROTOCOL_JAVA
            .contains("ProtocolInfo<T> bind(Function<ByteBuf, B> contextWrapper, C context);"));
        assert!(UNBOUND_PROTOCOL_JAVA.contains("extends ProtocolInfo.DetailsProvider"));

        let mut builder =
            ProtocolInfoBuilderModel::new(ConnectionProtocol::Login, PacketFlow::Serverbound);
        builder.add_packet(
            PacketTypeModel::new(PacketFlow::Serverbound, "minecraft:hello"),
            "ServerboundHelloPacket.STREAM_CODEC",
        );

        let simple = build_simple_unbound_protocol(builder.clone(), "unit");
        assert_eq!(simple.details().id, ConnectionProtocol::Login);
        assert_eq!(simple.bind("wrap").unwrap().codec_entries[0].serializer, "ServerboundHelloPacket.STREAM_CODEC");

        let contextual = build_unbound_protocol(builder);
        assert_eq!(contextual.details().flow, PacketFlow::Serverbound);
        assert_eq!(
            contextual.bind("registry", "session").unwrap().codec_entries[0].packet_type.to_string(),
            "serverbound/minecraft:hello"
        );
    }
}
