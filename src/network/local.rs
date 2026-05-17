use std::collections::VecDeque;

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

#[cfg(test)]
mod tests {
    use super::LocalConnection;

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
}
