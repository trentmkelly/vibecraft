#![cfg_attr(
    all(test, not(vibecraft_has_decompiled_sources)),
    allow(dead_code)
)]

pub const BUNDLE_SIZE_LIMIT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketFrame<T> {
    Packet(T),
    Bundle(Vec<T>),
    Delimiter,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundlePacket<T> {
    packets: Vec<T>,
}

impl<T> BundlePacket<T> {
    pub fn new(packets: impl IntoIterator<Item = T>) -> Self {
        Self {
            packets: packets.into_iter().collect(),
        }
    }

    pub fn sub_packets(&self) -> &[T] {
        &self.packets
    }

    pub fn into_packets(self) -> Vec<T> {
        self.packets
    }
}

pub struct BundleDelimiterPacket;

impl BundleDelimiterPacket {
    pub fn handle(&self) -> ! {
        panic!("This packet should be handled by pipeline");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundlePacker<T> {
    current: Option<Vec<T>>,
}

impl<T> Default for BundlePacker<T> {
    fn default() -> Self {
        Self { current: None }
    }
}

impl<T> BundlePacker<T> {
    pub fn accept(&mut self, frame: PacketFrame<T>) -> Result<Option<PacketFrame<T>>, String> {
        match (&mut self.current, frame) {
            (Some(_), PacketFrame::Terminal) => {
                Err("Terminal message received in bundle".to_string())
            }
            (Some(_), PacketFrame::Delimiter) => {
                let packets = self.current.take().unwrap_or_default();
                Ok(Some(PacketFrame::Bundle(packets)))
            }
            (Some(packets), PacketFrame::Packet(packet)) => {
                if packets.len() >= BUNDLE_SIZE_LIMIT {
                    return Err("Too many packets in a bundle".to_string());
                }
                packets.push(packet);
                Ok(None)
            }
            (Some(_), PacketFrame::Bundle(_)) => Err("Nested bundle received".to_string()),
            (None, PacketFrame::Delimiter) => {
                self.current = Some(Vec::new());
                Ok(None)
            }
            (None, frame) => Ok(Some(frame)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundlePipelineResult<T> {
    pub frames: Vec<PacketFrame<T>>,
    pub remove_handler: bool,
}

impl<T> BundlePipelineResult<T> {
    fn single(frame: PacketFrame<T>, remove_handler: bool) -> Self {
        Self {
            frames: vec![frame],
            remove_handler,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketBundlePacker<T> {
    packer: BundlePacker<T>,
}

impl<T> Default for PacketBundlePacker<T> {
    fn default() -> Self {
        Self {
            packer: BundlePacker::default(),
        }
    }
}

impl<T> PacketBundlePacker<T> {
    pub fn decode(&mut self, frame: PacketFrame<T>) -> Result<BundlePipelineResult<T>, String> {
        let terminal = matches!(frame, PacketFrame::Terminal);
        match self.packer.accept(frame)? {
            Some(frame) => Ok(BundlePipelineResult::single(frame, terminal)),
            None => Ok(BundlePipelineResult {
                frames: Vec::new(),
                remove_handler: false,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PacketBundleUnpacker;

impl PacketBundleUnpacker {
    pub fn encode<T>(&self, frame: PacketFrame<T>) -> BundlePipelineResult<T> {
        match frame {
            PacketFrame::Bundle(packets) => {
                let mut frames = Vec::with_capacity(packets.len() + 2);
                frames.push(PacketFrame::Delimiter);
                frames.extend(packets.into_iter().map(PacketFrame::Packet));
                frames.push(PacketFrame::Delimiter);
                BundlePipelineResult {
                    frames,
                    remove_handler: false,
                }
            }
            PacketFrame::Terminal => BundlePipelineResult::single(PacketFrame::Terminal, true),
            other => BundlePipelineResult::single(other, false),
        }
    }
}

pub fn unpack_bundle<T>(frame: PacketFrame<T>) -> Vec<PacketFrame<T>> {
    match frame {
        PacketFrame::Bundle(packets) => {
            let mut out = Vec::with_capacity(packets.len() + 2);
            out.push(PacketFrame::Delimiter);
            out.extend(packets.into_iter().map(PacketFrame::Packet));
            out.push(PacketFrame::Delimiter);
            out
        }
        other => vec![other],
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::{
        unpack_bundle, BundleDelimiterPacket, BundlePacker, BundlePacket, PacketBundlePacker,
        PacketBundleUnpacker, PacketFrame, BUNDLE_SIZE_LIMIT,
    };

    #[test]
    fn packs_packets_between_delimiters() {
        let mut packer = BundlePacker::default();
        assert_eq!(packer.accept(PacketFrame::Delimiter).unwrap(), None);
        assert_eq!(packer.accept(PacketFrame::Packet(1)).unwrap(), None);
        assert_eq!(packer.accept(PacketFrame::Packet(2)).unwrap(), None);
        assert_eq!(
            packer.accept(PacketFrame::Delimiter).unwrap(),
            Some(PacketFrame::Bundle(vec![1, 2]))
        );
    }

    #[test]
    fn unpacks_bundle_with_delimiters() {
        assert_eq!(
            unpack_bundle(PacketFrame::Bundle(vec![1, 2])),
            vec![
                PacketFrame::Delimiter,
                PacketFrame::Packet(1),
                PacketFrame::Packet(2),
                PacketFrame::Delimiter,
            ]
        );
    }

    #[test]
    fn rejects_terminal_packet_inside_bundle() {
        let mut packer = BundlePacker::<u8>::default();
        packer.accept(PacketFrame::Delimiter).unwrap();
        assert!(packer.accept(PacketFrame::Terminal).is_err());
    }

    #[test]
    fn enforces_bundle_size_limit() {
        let mut packer = BundlePacker::default();
        packer.accept(PacketFrame::Delimiter).unwrap();
        for i in 0..BUNDLE_SIZE_LIMIT {
            packer.accept(PacketFrame::Packet(i)).unwrap();
        }
        assert!(packer
            .accept(PacketFrame::Packet(BUNDLE_SIZE_LIMIT))
            .is_err());
    }

    #[test]
    fn packet_bundle_packer_matches_java_decode_contract() {
        const PACKER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/PacketBundlePacker.java"
        );
        const BUNDLER_INFO_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/BundlerInfo.java"
        );

        for sentinel in [
            "private BundlerInfo.@Nullable Bundler currentBundler;",
            "verifyNonTerminalPacket(msg);",
            "Packet<?> bundlePacket = this.currentBundler.addPacket(msg);",
            "this.currentBundler = null;",
            "out.add(bundlePacket);",
            "BundlerInfo.Bundler bundler = this.bundlerInfo.startPacketBundling(msg);",
            "out.add(msg);",
            "if (msg.isTerminal())",
            "ctx.pipeline().remove(ctx.name());",
            "throw new DecoderException(\"Terminal message received in bundle\")",
        ] {
            assert!(
                PACKER_JAVA.contains(sentinel),
                "missing PacketBundlePacker sentinel {sentinel}"
            );
        }
        for sentinel in [
            "int BUNDLE_SIZE_LIMIT = 4096;",
            "return packet == delimiterPacket ? new BundlerInfo.Bundler()",
            "if (packet == delimiterPacket)",
            "return constructor.apply(this.bundlePackets);",
            "throw new IllegalStateException(\"Too many packets in a bundle\")",
        ] {
            assert!(
                BUNDLER_INFO_JAVA.contains(sentinel),
                "missing BundlerInfo sentinel {sentinel}"
            );
        }

        let mut handler = PacketBundlePacker::default();
        let start = handler.decode(PacketFrame::<u8>::Delimiter).unwrap();
        assert!(start.frames.is_empty());
        assert!(!start.remove_handler);
        assert_eq!(
            handler.decode(PacketFrame::Packet(1)).unwrap().frames,
            Vec::<PacketFrame<u8>>::new()
        );
        assert_eq!(
            handler.decode(PacketFrame::Packet(2)).unwrap().frames,
            Vec::<PacketFrame<u8>>::new()
        );
        let finished = handler.decode(PacketFrame::Delimiter).unwrap();
        assert_eq!(finished.frames, vec![PacketFrame::Bundle(vec![1, 2])]);
        assert!(!finished.remove_handler);

        let forwarded = handler.decode(PacketFrame::Packet(3)).unwrap();
        assert_eq!(forwarded.frames, vec![PacketFrame::Packet(3)]);
        assert!(!forwarded.remove_handler);

        let terminal = handler.decode(PacketFrame::Terminal).unwrap();
        assert_eq!(terminal.frames, vec![PacketFrame::Terminal]);
        assert!(terminal.remove_handler);

        let mut handler = PacketBundlePacker::<u8>::default();
        handler.decode(PacketFrame::Delimiter).unwrap();
        assert_eq!(
            handler.decode(PacketFrame::Terminal).unwrap_err(),
            "Terminal message received in bundle"
        );
    }

    #[test]
    fn packet_bundle_unpacker_and_packet_models_match_java_contract() {
        const UNPACKER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/PacketBundleUnpacker.java"
        );
        const BUNDLER_INFO_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/BundlerInfo.java"
        );
        const BUNDLE_PACKET_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/BundlePacket.java"
        );
        const BUNDLE_DELIMITER_PACKET_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/BundleDelimiterPacket.java"
        );

        for sentinel in [
            "this.bundlerInfo.unbundlePacket(msg, out::add);",
            "if (msg.isTerminal())",
            "ctx.pipeline().remove(ctx.name());",
        ] {
            assert!(
                UNPACKER_JAVA.contains(sentinel),
                "missing PacketBundleUnpacker sentinel {sentinel}"
            );
        }
        for sentinel in [
            "if (packet.type() == bundlePacketType)",
            "output.accept(delimiterPacket);",
            "bundlerPacket.subPackets().forEach(output);",
            "output.accept(packet);",
        ] {
            assert!(
                BUNDLER_INFO_JAVA.contains(sentinel),
                "missing BundlerInfo unbundle sentinel {sentinel}"
            );
        }
        assert!(BUNDLE_PACKET_JAVA.contains("private final Iterable<Packet<? super T>> packets;"));
        assert!(
            BUNDLE_PACKET_JAVA.contains("public final Iterable<Packet<? super T>> subPackets()")
        );
        assert!(BUNDLE_DELIMITER_PACKET_JAVA
            .contains("throw new AssertionError(\"This packet should be handled by pipeline\")"));

        let packet = BundlePacket::new([1, 2, 3]);
        assert_eq!(packet.sub_packets(), &[1, 2, 3]);
        assert_eq!(packet.into_packets(), vec![1, 2, 3]);

        let unpacker = PacketBundleUnpacker;
        let unpacked = unpacker.encode(PacketFrame::Bundle(vec![1, 2]));
        assert_eq!(
            unpacked.frames,
            vec![
                PacketFrame::Delimiter,
                PacketFrame::Packet(1),
                PacketFrame::Packet(2),
                PacketFrame::Delimiter,
            ]
        );
        assert!(!unpacked.remove_handler);

        let forwarded = unpacker.encode(PacketFrame::Packet(7));
        assert_eq!(forwarded.frames, vec![PacketFrame::Packet(7)]);
        assert!(!forwarded.remove_handler);

        let terminal = unpacker.encode(PacketFrame::<u8>::Terminal);
        assert_eq!(terminal.frames, vec![PacketFrame::Terminal]);
        assert!(terminal.remove_handler);
    }

    #[test]
    #[should_panic(expected = "This packet should be handled by pipeline")]
    fn bundle_delimiter_packet_panics_if_handled() {
        BundleDelimiterPacket.handle();
    }
}
