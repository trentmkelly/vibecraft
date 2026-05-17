pub const BUNDLE_SIZE_LIMIT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketFrame<T> {
    Packet(T),
    Bundle(Vec<T>),
    Delimiter,
    Terminal,
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

#[cfg(test)]
mod tests {
    use super::{unpack_bundle, BundlePacker, PacketFrame, BUNDLE_SIZE_LIMIT};

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
}
