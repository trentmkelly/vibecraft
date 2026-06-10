use std::collections::{HashMap, HashSet};
use std::io;
#[cfg(test)]
use std::io::{Read, Write};

#[cfg(test)]
use crate::network::codec::{read_identifier, read_optional, write_identifier, write_optional};
#[cfg(test)]
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;

pub const MAX_COOKIE_PAYLOAD_SIZE: usize = 5120;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCookieRequestPacket {
    pub key: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCookieResponsePacket {
    pub key: Identifier,
    pub payload: Option<Vec<u8>>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStoreCookiePacket {
    pub key: Identifier,
    pub payload: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CookieState {
    pending_requests: HashSet<Identifier>,
    stored: HashMap<Identifier, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieResponseStatus {
    Stored(Vec<u8>),
    Empty,
}

#[cfg(test)]
impl ClientboundCookieRequestPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            key: read_identifier(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.key)
    }
}

impl ServerboundCookieResponsePacket {
    #[cfg(test)]
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            key: read_identifier(reader)?,
            payload: read_optional(reader, read_payload)?,
        })
    }

    #[cfg(test)]
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.key)?;
        write_optional(writer, self.payload.as_ref(), |writer, payload| {
            write_payload(writer, payload)
        })
    }
}

#[cfg(test)]
impl ClientboundStoreCookiePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            key: read_identifier(reader)?,
            payload: read_payload(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.key)?;
        write_payload(writer, &self.payload)
    }
}

impl CookieState {
    #[cfg(test)]
    pub fn request_cookie(&mut self, key: Identifier) -> ClientboundCookieRequestPacket {
        self.pending_requests.insert(key.clone());
        ClientboundCookieRequestPacket { key }
    }

    #[cfg(test)]
    pub fn store_cookie(
        &mut self,
        key: Identifier,
        payload: Vec<u8>,
    ) -> io::Result<ClientboundStoreCookiePacket> {
        if payload.len() > MAX_COOKIE_PAYLOAD_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cookie payload too large",
            ));
        }
        self.stored.insert(key.clone(), payload.clone());
        Ok(ClientboundStoreCookiePacket { key, payload })
    }

    pub fn handle_response(
        &mut self,
        response: ServerboundCookieResponsePacket,
    ) -> io::Result<CookieResponseStatus> {
        if !self.pending_requests.remove(&response.key) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected cookie response",
            ));
        }

        match response.payload {
            Some(payload) => {
                if payload.len() > MAX_COOKIE_PAYLOAD_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "cookie payload too large",
                    ));
                }
                self.stored.insert(response.key, payload.clone());
                Ok(CookieResponseStatus::Stored(payload))
            }
            None => {
                self.stored.remove(&response.key);
                Ok(CookieResponseStatus::Empty)
            }
        }
    }

    #[cfg(test)]
    pub fn get(&self, key: &Identifier) -> Option<&[u8]> {
        self.stored.get(key).map(Vec::as_slice)
    }

    #[cfg(test)]
    pub fn has_pending_request(&self, key: &Identifier) -> bool {
        self.pending_requests.contains(key)
    }
}

#[cfg(test)]
fn read_payload<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let len = read_var_i32(reader)?;
    if len < 0 || len as usize > MAX_COOKIE_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid cookie payload length",
        ));
    }
    let mut payload = vec![0u8; len as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

#[cfg(test)]
fn write_payload<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_COOKIE_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cookie payload too large",
        ));
    }
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

#[cfg(test)]
mod tests {
    use super::{
        ClientboundCookieRequestPacket, ClientboundStoreCookiePacket, CookieResponseStatus,
        CookieState, ServerboundCookieResponsePacket, MAX_COOKIE_PAYLOAD_SIZE,
    };
    use crate::registry::Identifier;
    use std::io::Cursor;

    #[test]
    fn round_trips_cookie_request() {
        let packet = ClientboundCookieRequestPacket {
            key: Identifier::parse("vibecraft:test").unwrap(),
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundCookieRequestPacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn round_trips_cookie_response_with_and_without_payload() {
        for payload in [Some(vec![1, 2, 3]), None] {
            let packet = ServerboundCookieResponsePacket {
                key: Identifier::parse("vibecraft:test").unwrap(),
                payload,
            };
            let mut bytes = Vec::new();
            packet.write(&mut bytes).unwrap();
            assert_eq!(
                ServerboundCookieResponsePacket::read(&mut Cursor::new(bytes)).unwrap(),
                packet
            );
        }
    }

    #[test]
    fn rejects_cookie_payloads_larger_than_vanilla_limit() {
        let packet = ServerboundCookieResponsePacket {
            key: Identifier::parse("vibecraft:test").unwrap(),
            payload: Some(vec![0; MAX_COOKIE_PAYLOAD_SIZE + 1]),
        };
        assert!(packet.write(&mut Vec::new()).is_err());
    }

    #[test]
    fn round_trips_store_cookie_at_vanilla_limit() {
        let packet = ClientboundStoreCookiePacket {
            key: Identifier::parse("vibecraft:test").unwrap(),
            payload: vec![7; MAX_COOKIE_PAYLOAD_SIZE],
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundStoreCookiePacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn rejects_oversized_store_cookie_payloads() {
        let packet = ClientboundStoreCookiePacket {
            key: Identifier::parse("vibecraft:test").unwrap(),
            payload: vec![0; MAX_COOKIE_PAYLOAD_SIZE + 1],
        };
        assert!(packet.write(&mut Vec::new()).is_err());
    }

    #[test]
    fn cookie_state_correlates_requests_and_responses() {
        let key = Identifier::parse("vibecraft:test").unwrap();
        let mut state = CookieState::default();
        let request = state.request_cookie(key.clone());
        assert_eq!(request.key, key);
        assert!(state.has_pending_request(&key));

        let status = state
            .handle_response(ServerboundCookieResponsePacket {
                key: key.clone(),
                payload: Some(vec![1, 2, 3]),
            })
            .unwrap();
        assert_eq!(status, CookieResponseStatus::Stored(vec![1, 2, 3]));
        assert_eq!(state.get(&key), Some([1, 2, 3].as_slice()));
        assert!(!state.has_pending_request(&key));
    }

    #[test]
    fn cookie_state_rejects_unexpected_responses() {
        let key = Identifier::parse("vibecraft:test").unwrap();
        let mut state = CookieState::default();
        let response = ServerboundCookieResponsePacket { key, payload: None };
        assert!(state.handle_response(response).is_err());
    }

    #[test]
    fn cookie_state_stores_and_clears_payloads() {
        let key = Identifier::parse("vibecraft:test").unwrap();
        let mut state = CookieState::default();
        let packet = state.store_cookie(key.clone(), vec![4, 5, 6]).unwrap();
        assert_eq!(packet.payload, vec![4, 5, 6]);
        assert_eq!(state.get(&key), Some([4, 5, 6].as_slice()));

        state.request_cookie(key.clone());
        let status = state
            .handle_response(ServerboundCookieResponsePacket {
                key: key.clone(),
                payload: None,
            })
            .unwrap();
        assert_eq!(status, CookieResponseStatus::Empty);
        assert_eq!(state.get(&key), None);
    }
}
