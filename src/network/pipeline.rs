#![allow(dead_code)]

use std::io::{self, Cursor};

use crate::network::compression::CompressionState;
use crate::network::dispatch::ConnectionProtocol;
use crate::network::encryption::MinecraftCipher;
use crate::network::varint::{read_var_i32, write_var_i32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPacket {
    pub id: i32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPacketMetadata {
    pub packet_type: &'static str,
    pub class_name: &'static str,
    pub terminal: bool,
    pub skippable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedProtocolPacket {
    pub metadata: ProtocolPacketMetadata,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolSwapAction {
    SetAutoRead(bool),
    AddBefore {
        name: &'static str,
        handler: &'static str,
    },
    AddAfter {
        name: &'static str,
        handler: &'static str,
    },
    RemoveSelf,
}

pub struct ProtocolSwapHandler;

impl ProtocolSwapHandler {
    pub fn handle_inbound_terminal_packet(terminal: bool) -> Vec<ProtocolSwapAction> {
        if terminal {
            vec![
                ProtocolSwapAction::SetAutoRead(false),
                ProtocolSwapAction::AddBefore {
                    name: "inbound_config",
                    handler: "UnconfiguredPipelineHandler.Inbound",
                },
                ProtocolSwapAction::RemoveSelf,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn handle_outbound_terminal_packet(terminal: bool) -> Vec<ProtocolSwapAction> {
        if terminal {
            vec![
                ProtocolSwapAction::AddAfter {
                    name: "outbound_config",
                    handler: "UnconfiguredPipelineHandler.Outbound",
                },
                ProtocolSwapAction::RemoveSelf,
            ]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineAction {
    FireChannelRead(&'static str),
    Write(&'static str),
    Release(&'static str),
    PromiseSuccess,
    Replace {
        name: &'static str,
        handler: &'static str,
    },
    SetAutoRead(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnconfiguredMessageKind {
    ByteBuf,
    Packet,
    Other(&'static str),
    InboundConfigurationTask(InboundConfigurationTask),
    OutboundConfigurationTask(OutboundConfigurationTask),
}

impl UnconfiguredMessageKind {
    fn label(&self) -> &'static str {
        match self {
            Self::ByteBuf => "ByteBuf",
            Self::Packet => "Packet",
            Self::Other(label) => label,
            Self::InboundConfigurationTask(_) => "InboundConfigurationTask",
            Self::OutboundConfigurationTask(_) => "OutboundConfigurationTask",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundConfigurationTask {
    actions: Vec<PipelineAction>,
}

impl InboundConfigurationTask {
    pub fn setup_protocol() -> Self {
        Self {
            actions: vec![
                PipelineAction::Replace {
                    name: "decoder",
                    handler: "PacketDecoder",
                },
                PipelineAction::SetAutoRead(true),
            ],
        }
    }

    pub fn and_then(mut self, other: Self) -> Self {
        self.actions.extend(other.actions);
        self
    }

    fn run(self) -> Vec<PipelineAction> {
        self.actions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundConfigurationTask {
    actions: Vec<PipelineAction>,
}

impl OutboundConfigurationTask {
    pub fn setup_protocol() -> Self {
        Self {
            actions: vec![PipelineAction::Replace {
                name: "encoder",
                handler: "PacketEncoder",
            }],
        }
    }

    pub fn and_then(mut self, other: Self) -> Self {
        self.actions.extend(other.actions);
        self
    }

    fn run(self) -> Vec<PipelineAction> {
        self.actions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnconfiguredPipelineError {
    Decoder {
        message: String,
        released: &'static str,
    },
    Encoder {
        message: String,
        released: &'static str,
    },
}

pub struct UnconfiguredPipelineHandler;

impl UnconfiguredPipelineHandler {
    pub fn setup_inbound_protocol() -> InboundConfigurationTask {
        InboundConfigurationTask::setup_protocol()
    }

    pub fn setup_outbound_protocol() -> OutboundConfigurationTask {
        OutboundConfigurationTask::setup_protocol()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnconfiguredInbound;

impl UnconfiguredInbound {
    pub fn channel_read(
        &self,
        msg: UnconfiguredMessageKind,
    ) -> Result<Vec<PipelineAction>, UnconfiguredPipelineError> {
        match msg {
            UnconfiguredMessageKind::ByteBuf | UnconfiguredMessageKind::Packet => {
                let label = msg.label();
                Err(UnconfiguredPipelineError::Decoder {
                    message: format!(
                        "Pipeline has no inbound protocol configured, can't process packet {label}"
                    ),
                    released: label,
                })
            }
            other => Ok(vec![PipelineAction::FireChannelRead(other.label())]),
        }
    }

    pub fn write(
        &self,
        msg: UnconfiguredMessageKind,
    ) -> Result<Vec<PipelineAction>, UnconfiguredPipelineError> {
        match msg {
            UnconfiguredMessageKind::InboundConfigurationTask(task) => {
                let mut actions = task.run();
                actions.push(PipelineAction::Release("InboundConfigurationTask"));
                actions.push(PipelineAction::PromiseSuccess);
                Ok(actions)
            }
            other => Ok(vec![PipelineAction::Write(other.label())]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnconfiguredOutbound;

impl UnconfiguredOutbound {
    pub fn write(
        &self,
        msg: UnconfiguredMessageKind,
    ) -> Result<Vec<PipelineAction>, UnconfiguredPipelineError> {
        match msg {
            UnconfiguredMessageKind::Packet => Err(UnconfiguredPipelineError::Encoder {
                message:
                    "Pipeline has no outbound protocol configured, can't process packet Packet"
                        .to_string(),
                released: "Packet",
            }),
            UnconfiguredMessageKind::OutboundConfigurationTask(task) => {
                let mut actions = task.run();
                actions.push(PipelineAction::Release("OutboundConfigurationTask"));
                actions.push(PipelineAction::PromiseSuccess);
                Ok(actions)
            }
            other => Ok(vec![PipelineAction::Write(other.label())]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketCodecError {
    SkipDecoder(String),
    SkipEncoder(String),
    Other(String),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketDecodeOutcome {
    pub packet: DecodedProtocolPacket,
    pub readable_bytes: usize,
    pub swap_actions: Vec<ProtocolSwapAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketEncodeOutcome {
    pub bytes: Vec<u8>,
    pub written_bytes: usize,
    pub swap_actions: Vec<ProtocolSwapAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketEncodeError {
    pub error: PacketCodecError,
    pub swap_actions: Vec<ProtocolSwapAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketDecoderHandler {
    protocol: ConnectionProtocol,
}

impl PacketDecoderHandler {
    pub fn new(protocol: ConnectionProtocol) -> Self {
        Self { protocol }
    }

    pub fn decode_with(
        &self,
        input: &mut Cursor<Vec<u8>>,
        decode: impl FnOnce(&mut Cursor<Vec<u8>>) -> Result<DecodedProtocolPacket, PacketCodecError>,
    ) -> Result<PacketDecodeOutcome, PacketCodecError> {
        let readable_bytes = input
            .get_ref()
            .len()
            .saturating_sub(input.position() as usize);
        let packet = match decode(input) {
            Ok(packet) => packet,
            Err(error @ PacketCodecError::SkipDecoder(_)) => {
                input.set_position(input.get_ref().len() as u64);
                return Err(error);
            }
            Err(error) => return Err(error),
        };

        let remaining = input
            .get_ref()
            .len()
            .saturating_sub(input.position() as usize);
        if remaining > 0 {
            return Err(PacketCodecError::Io(format!(
                "Packet {}/{} ({}) was larger than I expected, found {} bytes extra whilst reading packet {}",
                self.protocol.id(),
                packet.metadata.packet_type,
                packet.metadata.class_name,
                remaining,
                packet.metadata.packet_type
            )));
        }

        let swap_actions =
            ProtocolSwapHandler::handle_inbound_terminal_packet(packet.metadata.terminal);
        Ok(PacketDecodeOutcome {
            packet,
            readable_bytes,
            swap_actions,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketEncoderHandler {
    protocol: ConnectionProtocol,
}

impl PacketEncoderHandler {
    pub fn new(protocol: ConnectionProtocol) -> Self {
        Self { protocol }
    }

    pub fn encode_with(
        &self,
        packet: &DecodedProtocolPacket,
        encode: impl FnOnce(&mut Vec<u8>, &DecodedProtocolPacket) -> Result<(), PacketCodecError>,
    ) -> Result<PacketEncodeOutcome, PacketEncodeError> {
        let mut output = Vec::new();
        let result = encode(&mut output, packet);
        let swap_actions =
            ProtocolSwapHandler::handle_outbound_terminal_packet(packet.metadata.terminal);

        match result {
            Ok(()) => {
                let written_bytes = output.len();
                Ok(PacketEncodeOutcome {
                    bytes: output,
                    written_bytes,
                    swap_actions,
                })
            }
            Err(error) => {
                let error = if packet.metadata.skippable {
                    PacketCodecError::SkipEncoder(format!("{error:?}"))
                } else {
                    error
                };
                Err(PacketEncodeError {
                    error,
                    swap_actions,
                })
            }
        }
    }

    pub fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }
}

#[derive(Debug, Clone)]
pub struct NetworkPipeline {
    compression: CompressionState,
    inbound_cipher: Option<MinecraftCipher>,
    outbound_cipher: Option<MinecraftCipher>,
}

impl Default for NetworkPipeline {
    fn default() -> Self {
        Self {
            compression: CompressionState::disabled(),
            inbound_cipher: None,
            outbound_cipher: None,
        }
    }
}

impl NetworkPipeline {
    pub fn enable_compression(&mut self, threshold: i32) {
        self.compression = CompressionState::enabled(threshold);
    }

    pub fn enable_encryption(&mut self, shared_secret: [u8; 16]) {
        self.inbound_cipher = Some(MinecraftCipher::new(shared_secret));
        self.outbound_cipher = Some(MinecraftCipher::new(shared_secret));
    }

    pub fn encode_packet(&mut self, id: i32, payload: &[u8]) -> io::Result<Vec<u8>> {
        let mut packet = Vec::new();
        write_var_i32(&mut packet, id)?;
        packet.extend_from_slice(payload);

        let mut frame = self.compression.encode_packet(&packet)?;
        if let Some(cipher) = &mut self.outbound_cipher {
            cipher.apply_encrypt(&mut frame);
        }
        Ok(frame)
    }

    pub fn decode_packet(&mut self, frame: &[u8]) -> io::Result<DecodedPacket> {
        let mut bytes = frame.to_vec();
        if let Some(cipher) = &mut self.inbound_cipher {
            cipher.apply_decrypt(&mut bytes);
        }
        let packet = self.compression.decode_packet(&mut Cursor::new(bytes))?;
        let mut packet_input = Cursor::new(packet);
        let id = read_var_i32(&mut packet_input)?;
        let position = packet_input.position() as usize;
        let payload = packet_input.into_inner()[position..].to_vec();
        Ok(DecodedPacket { id, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(
        packet_type: &'static str,
        terminal: bool,
        skippable: bool,
    ) -> ProtocolPacketMetadata {
        ProtocolPacketMetadata {
            packet_type,
            class_name: "TestPacket",
            terminal,
            skippable,
        }
    }

    #[test]
    fn pipeline_round_trips_uncompressed_packet_id_and_payload() {
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();

        let frame = encoder.encode_packet(0x2a, b"payload").unwrap();
        let packet = decoder.decode_packet(&frame).unwrap();

        assert_eq!(
            packet,
            DecodedPacket {
                id: 0x2a,
                payload: b"payload".to_vec()
            }
        );
    }

    #[test]
    fn pipeline_applies_compression_before_encryption_and_reverses_on_decode() {
        let secret = *b"0123456789abcdef";
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();
        encoder.enable_compression(8);
        decoder.enable_compression(8);
        encoder.enable_encryption(secret);
        decoder.enable_encryption(secret);

        let payload = b"large enough packet payload for zlib".repeat(3);
        let frame = encoder.encode_packet(0x10, &payload).unwrap();
        assert!(!frame.windows(payload.len()).any(|window| window == payload));

        let decoded = decoder.decode_packet(&frame).unwrap();
        assert_eq!(decoded.id, 0x10);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn pipeline_keeps_encryption_state_across_multiple_frames() {
        let secret = *b"fedcba9876543210";
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();
        encoder.enable_encryption(secret);
        decoder.enable_encryption(secret);

        let first = encoder.encode_packet(1, b"one").unwrap();
        let second = encoder.encode_packet(2, b"two").unwrap();

        assert_eq!(decoder.decode_packet(&first).unwrap().payload, b"one");
        assert_eq!(decoder.decode_packet(&second).unwrap().payload, b"two");
    }

    #[test]
    fn packet_decoder_handler_matches_java_codec_wrapper_contract() {
        const PACKET_DECODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/PacketDecoder.java"
        );
        const PROTOCOL_SWAP_HANDLER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/ProtocolSwapHandler.java"
        );

        for sentinel in [
            "int readableBytes = input.readableBytes();",
            "packet = this.protocolInfo.codec().decode(input);",
            "if (e instanceof SkipPacketException)",
            "input.skipBytes(input.readableBytes());",
            "JvmProfiler.INSTANCE.onPacketReceived",
            "if (input.readableBytes() > 0)",
            "was larger than I expected, found ",
            "out.add(packet);",
            "ProtocolSwapHandler.handleInboundTerminalPacket(ctx, packet);",
        ] {
            assert!(
                PACKET_DECODER_JAVA.contains(sentinel),
                "missing PacketDecoder sentinel {sentinel}"
            );
        }
        for sentinel in [
            "ctx.channel().config().setAutoRead(false);",
            "ctx.pipeline().addBefore(ctx.name(), \"inbound_config\", new UnconfiguredPipelineHandler.Inbound());",
            "ctx.pipeline().remove(ctx.name());",
        ] {
            assert!(
                PROTOCOL_SWAP_HANDLER_JAVA.contains(sentinel),
                "missing inbound ProtocolSwapHandler sentinel {sentinel}"
            );
        }

        let decoder = PacketDecoderHandler::new(ConnectionProtocol::Play);
        let mut input = Cursor::new(vec![1, 2, 3]);
        let decoded = decoder
            .decode_with(&mut input, |input| {
                let mut payload = vec![0; 3];
                use std::io::Read;
                input.read_exact(&mut payload).unwrap();
                Ok(DecodedProtocolPacket {
                    metadata: metadata("minecraft:test", true, false),
                    payload,
                })
            })
            .unwrap();
        assert_eq!(decoded.readable_bytes, 3);
        assert_eq!(decoded.packet.payload, vec![1, 2, 3]);
        assert_eq!(
            decoded.swap_actions,
            vec![
                ProtocolSwapAction::SetAutoRead(false),
                ProtocolSwapAction::AddBefore {
                    name: "inbound_config",
                    handler: "UnconfiguredPipelineHandler.Inbound"
                },
                ProtocolSwapAction::RemoveSelf,
            ]
        );

        let mut input = Cursor::new(vec![1, 2, 3]);
        let too_large = decoder.decode_with(&mut input, |input| {
            input.set_position(1);
            Ok(DecodedProtocolPacket {
                metadata: metadata("minecraft:leftover", false, false),
                payload: vec![1],
            })
        });
        assert_eq!(
            too_large,
            Err(PacketCodecError::Io(
                "Packet play/minecraft:leftover (TestPacket) was larger than I expected, found 2 bytes extra whilst reading packet minecraft:leftover"
                    .to_string()
            ))
        );

        let mut input = Cursor::new(vec![1, 2, 3]);
        let skipped = decoder.decode_with(&mut input, |_input| {
            Err(PacketCodecError::SkipDecoder("skip".to_string()))
        });
        assert_eq!(
            skipped,
            Err(PacketCodecError::SkipDecoder("skip".to_string()))
        );
        assert_eq!(input.position(), 3);
    }

    #[test]
    fn packet_encoder_handler_matches_java_error_and_terminal_contract() {
        const PACKET_ENCODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/PacketEncoder.java"
        );
        const PROTOCOL_SWAP_HANDLER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/ProtocolSwapHandler.java"
        );

        for sentinel in [
            "PacketType<? extends Packet<? super T>> packetId = packet.type();",
            "this.protocolInfo.codec().encode(output, packet);",
            "int writtenBytes = output.readableBytes();",
            "JvmProfiler.INSTANCE.onPacketSent",
            "LOGGER.error(\"Error sending packet {}\", packetId, t);",
            "if (packet.isSkippable())",
            "throw new SkipPacketEncoderException(t);",
            "ProtocolSwapHandler.handleOutboundTerminalPacket(ctx, packet);",
        ] {
            assert!(
                PACKET_ENCODER_JAVA.contains(sentinel),
                "missing PacketEncoder sentinel {sentinel}"
            );
        }
        for sentinel in [
            "ctx.pipeline().addAfter(ctx.name(), \"outbound_config\", new UnconfiguredPipelineHandler.Outbound());",
            "ctx.pipeline().remove(ctx.name());",
        ] {
            assert!(
                PROTOCOL_SWAP_HANDLER_JAVA.contains(sentinel),
                "missing outbound ProtocolSwapHandler sentinel {sentinel}"
            );
        }

        let encoder = PacketEncoderHandler::new(ConnectionProtocol::Configuration);
        assert_eq!(encoder.protocol(), ConnectionProtocol::Configuration);
        let packet = DecodedProtocolPacket {
            metadata: metadata("minecraft:finish_configuration", true, false),
            payload: vec![4, 5],
        };
        let encoded = encoder
            .encode_with(&packet, |output, packet| {
                output.extend_from_slice(&packet.payload);
                Ok(())
            })
            .unwrap();
        assert_eq!(encoded.bytes, vec![4, 5]);
        assert_eq!(encoded.written_bytes, 2);
        assert_eq!(
            encoded.swap_actions,
            vec![
                ProtocolSwapAction::AddAfter {
                    name: "outbound_config",
                    handler: "UnconfiguredPipelineHandler.Outbound"
                },
                ProtocolSwapAction::RemoveSelf,
            ]
        );

        let skippable = DecodedProtocolPacket {
            metadata: metadata("minecraft:skippable", false, true),
            payload: Vec::new(),
        };
        let skipped = encoder
            .encode_with(&skippable, |_output, _packet| {
                Err(PacketCodecError::Other("boom".to_string()))
            })
            .unwrap_err();
        assert_eq!(
            skipped.error,
            PacketCodecError::SkipEncoder("Other(\"boom\")".to_string())
        );
        assert!(skipped.swap_actions.is_empty());

        let fatal = DecodedProtocolPacket {
            metadata: metadata("minecraft:fatal", true, false),
            payload: Vec::new(),
        };
        let fatal_error = encoder
            .encode_with(&fatal, |_output, _packet| {
                Err(PacketCodecError::Other("fatal".to_string()))
            })
            .unwrap_err();
        assert_eq!(
            fatal_error.error,
            PacketCodecError::Other("fatal".to_string())
        );
        assert_eq!(
            fatal_error.swap_actions,
            ProtocolSwapHandler::handle_outbound_terminal_packet(true)
        );
    }

    #[test]
    fn unconfigured_pipeline_handler_matches_java_inbound_and_outbound_contracts() {
        const UNCONFIGURED_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/UnconfiguredPipelineHandler.java"
        );

        for sentinel in [
            "return setupInboundHandler(new PacketDecoder<T>(protocolInfo));",
            "ctx.pipeline().replace(ctx.name(), \"decoder\", newHandler);",
            "ctx.channel().config().setAutoRead(true);",
            "return setupOutboundHandler(new PacketEncoder<T>(codecData));",
            "ctx.pipeline().replace(ctx.name(), \"encoder\", newHandler);",
            "if (!(msg instanceof ByteBuf) && !(msg instanceof Packet))",
            "ctx.fireChannelRead(msg);",
            "ReferenceCountUtil.release(msg);",
            "throw new DecoderException(\"Pipeline has no inbound protocol configured, can't process packet \" + msg);",
            "if (msg instanceof UnconfiguredPipelineHandler.InboundConfigurationTask configurationTask)",
            "promise.setSuccess();",
            "if (msg instanceof Packet)",
            "throw new EncoderException(\"Pipeline has no outbound protocol configured, can't process packet \" + msg);",
            "default UnconfiguredPipelineHandler.InboundConfigurationTask andThen",
            "default UnconfiguredPipelineHandler.OutboundConfigurationTask andThen",
        ] {
            assert!(
                UNCONFIGURED_JAVA.contains(sentinel),
                "missing UnconfiguredPipelineHandler sentinel {sentinel}"
            );
        }

        let inbound = UnconfiguredInbound;
        assert_eq!(
            inbound.channel_read(UnconfiguredMessageKind::Other("String")),
            Ok(vec![PipelineAction::FireChannelRead("String")])
        );
        assert_eq!(
            inbound.channel_read(UnconfiguredMessageKind::ByteBuf),
            Err(UnconfiguredPipelineError::Decoder {
                message:
                    "Pipeline has no inbound protocol configured, can't process packet ByteBuf"
                        .to_string(),
                released: "ByteBuf",
            })
        );
        assert_eq!(
            inbound.channel_read(UnconfiguredMessageKind::Packet),
            Err(UnconfiguredPipelineError::Decoder {
                message: "Pipeline has no inbound protocol configured, can't process packet Packet"
                    .to_string(),
                released: "Packet",
            })
        );

        let inbound_task = UnconfiguredPipelineHandler::setup_inbound_protocol();
        assert_eq!(
            inbound.write(UnconfiguredMessageKind::InboundConfigurationTask(
                inbound_task
            )),
            Ok(vec![
                PipelineAction::Replace {
                    name: "decoder",
                    handler: "PacketDecoder",
                },
                PipelineAction::SetAutoRead(true),
                PipelineAction::Release("InboundConfigurationTask"),
                PipelineAction::PromiseSuccess,
            ])
        );
        assert_eq!(
            inbound.write(UnconfiguredMessageKind::Other("FlushMarker")),
            Ok(vec![PipelineAction::Write("FlushMarker")])
        );

        let outbound = UnconfiguredOutbound;
        assert_eq!(
            outbound.write(UnconfiguredMessageKind::Packet),
            Err(UnconfiguredPipelineError::Encoder {
                message:
                    "Pipeline has no outbound protocol configured, can't process packet Packet"
                        .to_string(),
                released: "Packet",
            })
        );
        let outbound_task = UnconfiguredPipelineHandler::setup_outbound_protocol();
        assert_eq!(
            outbound.write(UnconfiguredMessageKind::OutboundConfigurationTask(
                outbound_task
            )),
            Ok(vec![
                PipelineAction::Replace {
                    name: "encoder",
                    handler: "PacketEncoder",
                },
                PipelineAction::Release("OutboundConfigurationTask"),
                PipelineAction::PromiseSuccess,
            ])
        );
        assert_eq!(
            outbound.write(UnconfiguredMessageKind::Other("ByteBuf")),
            Ok(vec![PipelineAction::Write("ByteBuf")])
        );

        let chained_inbound = InboundConfigurationTask::setup_protocol()
            .and_then(InboundConfigurationTask::setup_protocol());
        assert_eq!(
            inbound.write(UnconfiguredMessageKind::InboundConfigurationTask(
                chained_inbound
            )),
            Ok(vec![
                PipelineAction::Replace {
                    name: "decoder",
                    handler: "PacketDecoder",
                },
                PipelineAction::SetAutoRead(true),
                PipelineAction::Replace {
                    name: "decoder",
                    handler: "PacketDecoder",
                },
                PipelineAction::SetAutoRead(true),
                PipelineAction::Release("InboundConfigurationTask"),
                PipelineAction::PromiseSuccess,
            ])
        );

        let chained_outbound = OutboundConfigurationTask::setup_protocol()
            .and_then(OutboundConfigurationTask::setup_protocol());
        assert_eq!(
            outbound.write(UnconfiguredMessageKind::OutboundConfigurationTask(
                chained_outbound
            )),
            Ok(vec![
                PipelineAction::Replace {
                    name: "encoder",
                    handler: "PacketEncoder",
                },
                PipelineAction::Replace {
                    name: "encoder",
                    handler: "PacketEncoder",
                },
                PipelineAction::Release("OutboundConfigurationTask"),
                PipelineAction::PromiseSuccess,
            ])
        );
    }
}
