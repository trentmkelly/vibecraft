#![allow(dead_code)]

use std::collections::VecDeque;

use crate::network::bandwidth::{BandwidthDebugMonitor, BandwidthSampleLogger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalConnection {
    inbound: VecDeque<Vec<u8>>,
    outbound: VecDeque<Vec<u8>>,
    closed: bool,
}

impl LocalConnection {
    pub fn pair() -> (Self, Self) {
        (
            Self {
                inbound: VecDeque::new(),
                outbound: VecDeque::new(),
                closed: false,
            },
            Self {
                inbound: VecDeque::new(),
                outbound: VecDeque::new(),
                closed: false,
            },
        )
    }

    pub fn send(&mut self, packet: Vec<u8>) -> Result<(), String> {
        if self.closed {
            return Err("local connection is closed".to_string());
        }
        self.outbound.push_back(packet);
        Ok(())
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        self.inbound.pop_front()
    }

    pub fn flush_to(&mut self, peer: &mut LocalConnection) -> Result<(), String> {
        if self.closed || peer.closed {
            return Err("local connection is closed".to_string());
        }
        while let Some(packet) = self.outbound.pop_front() {
            peer.inbound.push_back(packet);
        }
        Ok(())
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.inbound.clear();
        self.outbound.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenByteBuf {
    contents: Vec<u8>,
    ref_count: usize,
    touches: Vec<Option<String>>,
}

impl HiddenByteBuf {
    pub fn new(contents: Vec<u8>) -> Self {
        Self {
            contents,
            ref_count: 1,
            touches: Vec::new(),
        }
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    pub fn ref_count(&self) -> usize {
        self.ref_count
    }

    pub fn retain(&mut self) -> &mut Self {
        self.ref_count += 1;
        self
    }

    pub fn retain_by(&mut self, increment: usize) -> &mut Self {
        self.ref_count += increment;
        self
    }

    pub fn touch(&mut self) -> &mut Self {
        self.touches.push(None);
        self
    }

    pub fn touch_with(&mut self, hint: impl Into<String>) -> &mut Self {
        self.touches.push(Some(hint.into()));
        self
    }

    pub fn release(&mut self) -> bool {
        self.release_by(1)
    }

    pub fn release_by(&mut self, decrement: usize) -> bool {
        self.ref_count = self.ref_count.saturating_sub(decrement);
        self.ref_count == 0
    }

    pub fn pack(message: LocalFrameMessage) -> LocalFrameMessage {
        match message {
            LocalFrameMessage::Bytes(bytes) => LocalFrameMessage::Hidden(Self::new(bytes)),
            other => other,
        }
    }

    pub fn unpack(message: LocalFrameMessage) -> LocalFrameMessage {
        match message {
            LocalFrameMessage::Hidden(hidden) => LocalFrameMessage::Bytes(hidden.contents),
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalFrameMessage {
    Bytes(Vec<u8>),
    Hidden(HiddenByteBuf),
    Other(String),
}

pub struct LocalFrameDecoder;

impl LocalFrameDecoder {
    pub fn channel_read(message: LocalFrameMessage) -> LocalFrameMessage {
        HiddenByteBuf::unpack(message)
    }
}

pub struct LocalFrameEncoder;

impl LocalFrameEncoder {
    pub fn write(message: LocalFrameMessage) -> LocalFrameMessage {
        HiddenByteBuf::pack(message)
    }
}

pub struct MonitoredLocalFrameDecoder<'a, L> {
    monitor: &'a BandwidthDebugMonitor<L>,
}

impl<'a, L: BandwidthSampleLogger> MonitoredLocalFrameDecoder<'a, L> {
    pub fn new(monitor: &'a BandwidthDebugMonitor<L>) -> Self {
        Self { monitor }
    }

    pub fn channel_read(&self, message: LocalFrameMessage) -> LocalFrameMessage {
        let message = HiddenByteBuf::unpack(message);
        if let LocalFrameMessage::Bytes(bytes) = &message {
            self.monitor.on_receive(bytes.len() as i32);
        }
        message
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{
        HiddenByteBuf, LocalConnection, LocalFrameDecoder, LocalFrameEncoder, LocalFrameMessage,
        MonitoredLocalFrameDecoder,
    };
    use crate::network::bandwidth::{BandwidthDebugMonitor, BandwidthSampleLogger};
    use crate::network::handler_names::HandlerNames;
    use std::sync::Mutex;

    #[test]
    fn local_connections_exchange_packets_without_tcp() {
        let (mut client, mut server) = LocalConnection::pair();
        client.send(vec![1, 2, 3]).unwrap();
        client.flush_to(&mut server).unwrap();
        assert_eq!(server.receive(), Some(vec![1, 2, 3]));
        assert_eq!(server.receive(), None);
    }

    #[test]
    fn closed_local_connection_rejects_sends() {
        let (mut client, _server) = LocalConnection::pair();
        client.close();
        assert!(client.send(vec![1]).is_err());
    }

    #[test]
    fn handler_names_match_java_constants() {
        const HANDLER_NAMES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/HandlerNames.java");

        for (name, value) in [
            ("DECOMPRESS", HandlerNames::DECOMPRESS),
            ("COMPRESS", HandlerNames::COMPRESS),
            ("DECODER", HandlerNames::DECODER),
            ("ENCODER", HandlerNames::ENCODER),
            ("INBOUND_CONFIG", HandlerNames::INBOUND_CONFIG),
            ("OUTBOUND_CONFIG", HandlerNames::OUTBOUND_CONFIG),
            ("SPLITTER", HandlerNames::SPLITTER),
            ("PREPENDER", HandlerNames::PREPENDER),
            ("DECRYPT", HandlerNames::DECRYPT),
            ("ENCRYPT", HandlerNames::ENCRYPT),
            ("UNBUNDLER", HandlerNames::UNBUNDLER),
            ("BUNDLER", HandlerNames::BUNDLER),
            ("PACKET_HANDLER", HandlerNames::PACKET_HANDLER),
            ("TIMEOUT", HandlerNames::TIMEOUT),
            ("LEGACY_QUERY", HandlerNames::LEGACY_QUERY),
            ("LATENCY", HandlerNames::LATENCY),
        ] {
            assert!(
                HANDLER_NAMES_JAVA
                    .contains(&format!("public static final String {name} = \"{value}\";")),
                "missing HandlerNames constant {name}"
            );
        }
    }

    #[test]
    fn hidden_byte_buf_packs_byte_buffers_and_delegates_reference_counting() {
        const HIDDEN_BYTE_BUF_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/HiddenByteBuf.java");

        for sentinel in [
            "public record HiddenByteBuf(ByteBuf contents) implements ReferenceCounted",
            "ByteBufUtil.ensureAccessible(contents)",
            "return msg instanceof ByteBuf buf ? new HiddenByteBuf(buf) : msg;",
            "return msg instanceof HiddenByteBuf buf ? ByteBufUtil.ensureAccessible(buf.contents) : msg;",
            "this.contents.retain(increment);",
            "this.contents.release(decrement);",
        ] {
            assert!(
                HIDDEN_BYTE_BUF_JAVA.contains(sentinel),
                "missing HiddenByteBuf sentinel {sentinel}"
            );
        }

        let packed = HiddenByteBuf::pack(LocalFrameMessage::Bytes(vec![1, 2, 3]));
        let LocalFrameMessage::Hidden(mut hidden) = packed else {
            panic!("bytes should pack into HiddenByteBuf");
        };
        assert_eq!(hidden.contents(), &[1, 2, 3]);
        assert_eq!(hidden.ref_count(), 1);
        hidden.retain().retain_by(2).touch().touch_with("hint");
        assert_eq!(hidden.ref_count(), 4);
        assert!(!hidden.release_by(3));
        assert!(hidden.release());

        assert_eq!(
            HiddenByteBuf::unpack(LocalFrameMessage::Hidden(hidden)),
            LocalFrameMessage::Bytes(vec![1, 2, 3])
        );
        assert_eq!(
            HiddenByteBuf::pack(LocalFrameMessage::Other("same".to_string())),
            LocalFrameMessage::Other("same".to_string())
        );
    }

    #[derive(Default)]
    struct TestLogger {
        samples: Mutex<Vec<i64>>,
    }

    impl BandwidthSampleLogger for TestLogger {
        fn log_sample(&self, sample: i64) {
            self.samples.lock().unwrap().push(sample);
        }
    }

    #[test]
    fn local_frame_handlers_pack_unpack_and_monitor_byte_buffers() {
        const LOCAL_FRAME_DECODER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/LocalFrameDecoder.java");
        const LOCAL_FRAME_ENCODER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/LocalFrameEncoder.java");
        const MONITORED_LOCAL_FRAME_DECODER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/MonitoredLocalFrameDecoder.java");

        assert!(
            LOCAL_FRAME_DECODER_JAVA.contains("ctx.fireChannelRead(HiddenByteBuf.unpack(msg));")
        );
        assert!(LOCAL_FRAME_ENCODER_JAVA.contains("ctx.write(HiddenByteBuf.pack(msg), promise);"));
        assert!(MONITORED_LOCAL_FRAME_DECODER_JAVA.contains("msg = HiddenByteBuf.unpack(msg);"));
        assert!(MONITORED_LOCAL_FRAME_DECODER_JAVA
            .contains("this.monitor.onReceive(in.readableBytes());"));

        let encoded = LocalFrameEncoder::write(LocalFrameMessage::Bytes(vec![4, 5, 6, 7]));
        assert!(matches!(encoded, LocalFrameMessage::Hidden(_)));
        let decoded = LocalFrameDecoder::channel_read(encoded);
        assert_eq!(decoded, LocalFrameMessage::Bytes(vec![4, 5, 6, 7]));

        let monitor = BandwidthDebugMonitor::new(TestLogger::default());
        let monitored = MonitoredLocalFrameDecoder::new(&monitor);
        let output =
            monitored.channel_read(LocalFrameEncoder::write(LocalFrameMessage::Bytes(vec![
                9, 8, 7,
            ])));
        assert_eq!(output, LocalFrameMessage::Bytes(vec![9, 8, 7]));
        assert_eq!(monitor.pending_bytes_received(), 3);
    }
}
