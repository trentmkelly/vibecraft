#![allow(dead_code)]

use std::collections::BTreeMap;
use std::net::{SocketAddr, UdpSocket};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const GAME_TYPE: &str = "SMP";
const GAME_ID: &str = "MINECRAFT";
const CHALLENGE_TTL: Duration = Duration::from_secs(30);
const FULL_STAT_CACHE_TIME: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryServerInfo {
    pub server_name: String,
    pub world_name: String,
    pub server_version: String,
    pub plugin_names: String,
    pub host_ip: String,
    pub server_port: u16,
    pub player_count: usize,
    pub max_players: usize,
    pub player_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResponse {
    Challenge(Vec<u8>),
    BasicStatus(Vec<u8>),
    FullStat(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestChallenge {
    created_at: Instant,
    challenge: i32,
    ident: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct QueryProtocol {
    challenges: BTreeMap<SocketAddr, RequestChallenge>,
    next_challenge: i32,
    cached_full_stat: Option<CachedFullStat>,
}

#[derive(Debug, Clone)]
struct CachedFullStat {
    built_at: Instant,
    bytes: Vec<u8>,
}

impl QueryProtocol {
    pub fn new() -> Self {
        Self::with_challenge_seed(random_challenge_seed())
    }

    pub fn with_challenge_seed(seed: i32) -> Self {
        Self {
            challenges: BTreeMap::new(),
            next_challenge: seed.rem_euclid(0x01_00_00_00),
            cached_full_stat: None,
        }
    }

    pub fn handle_packet(
        &mut self,
        remote: SocketAddr,
        packet: &[u8],
        now: Instant,
        info: &QueryServerInfo,
    ) -> Option<QueryResponse> {
        self.prune_challenges(now);
        if packet.len() < 7 || packet[0] != 0xFE || packet[1] != 0xFD {
            return None;
        }

        let ident = [packet[3], packet[4], packet[5], packet[6]];
        match packet[2] {
            9 => Some(QueryResponse::Challenge(
                self.issue_challenge(remote, ident, now),
            )),
            0 => {
                if !self.valid_challenge(remote, packet) {
                    return None;
                }
                if packet.len() == 15 {
                    Some(QueryResponse::FullStat(
                        self.full_stat_response(ident, now, info),
                    ))
                } else {
                    Some(QueryResponse::BasicStatus(basic_status_response(
                        ident, info,
                    )))
                }
            }
            _ => None,
        }
    }

    pub fn challenge_count(&self) -> usize {
        self.challenges.len()
    }

    fn issue_challenge(&mut self, remote: SocketAddr, ident: [u8; 4], now: Instant) -> Vec<u8> {
        let challenge = self.next_challenge;
        self.next_challenge = (self.next_challenge + 1).rem_euclid(0x01_00_00_00);
        self.challenges.insert(
            remote,
            RequestChallenge {
                created_at: now,
                challenge,
                ident,
            },
        );
        challenge_response(ident, challenge)
    }

    fn valid_challenge(&self, remote: SocketAddr, packet: &[u8]) -> bool {
        let Some(challenge) = self.challenges.get(&remote) else {
            return false;
        };
        if packet.len() < 11 || challenge.ident != [packet[3], packet[4], packet[5], packet[6]] {
            return false;
        }
        challenge.challenge == read_i32_be(&packet[7..11])
    }

    fn prune_challenges(&mut self, now: Instant) {
        self.challenges
            .retain(|_, challenge| now.duration_since(challenge.created_at) < CHALLENGE_TTL);
    }

    fn full_stat_response(
        &mut self,
        ident: [u8; 4],
        now: Instant,
        info: &QueryServerInfo,
    ) -> Vec<u8> {
        if let Some(cached) = &self.cached_full_stat {
            if now.duration_since(cached.built_at) < FULL_STAT_CACHE_TIME {
                let mut bytes = cached.bytes.clone();
                bytes[1..5].copy_from_slice(&ident);
                return bytes;
            }
        }

        let bytes = full_stat_response(ident, info);
        self.cached_full_stat = Some(CachedFullStat {
            built_at: now,
            bytes: bytes.clone(),
        });
        bytes
    }
}

impl Default for QueryProtocol {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spawn_query_server(
    bind_ip: &str,
    port: u16,
    info: QueryServerInfo,
) -> Result<JoinHandle<()>, String> {
    let address = format!("{bind_ip}:{port}");
    let socket = UdpSocket::bind(&address)
        .map_err(|err| format!("Failed to bind query listener on {address}: {err}"))?;
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|err| format!("Failed to configure query listener timeout: {err}"))?;
    Ok(thread::Builder::new()
        .name("Query Listener".to_string())
        .spawn(move || run_query_loop(socket, info))
        .map_err(|err| format!("Failed to start query listener thread: {err}"))?)
}

fn run_query_loop(socket: UdpSocket, info: QueryServerInfo) {
    let mut protocol = QueryProtocol::new();
    let mut buffer = [0u8; 1460];
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((len, remote)) => {
                let now = Instant::now();
                let Some(response) = protocol.handle_packet(remote, &buffer[..len], now, &info)
                else {
                    continue;
                };
                let bytes = match response {
                    QueryResponse::Challenge(bytes)
                    | QueryResponse::BasicStatus(bytes)
                    | QueryResponse::FullStat(bytes) => bytes,
                };
                let _ = socket.send_to(&bytes, remote);
            }
            Err(err)
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(_) => break,
        }
    }
}

fn challenge_response(ident: [u8; 4], challenge: i32) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(9);
    out.extend_from_slice(&ident);
    out.extend_from_slice(challenge.to_string().as_bytes());
    out.push(0);
    out
}

fn basic_status_response(ident: [u8; 4], info: &QueryServerInfo) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0);
    out.extend_from_slice(&ident);
    write_c_string(&mut out, &info.server_name);
    write_c_string(&mut out, GAME_TYPE);
    write_c_string(&mut out, &info.world_name);
    write_c_string(&mut out, &info.player_count.to_string());
    write_c_string(&mut out, &info.max_players.to_string());
    out.extend_from_slice(&info.server_port.to_le_bytes());
    write_c_string(&mut out, &info.host_ip);
    out
}

fn full_stat_response(ident: [u8; 4], info: &QueryServerInfo) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0);
    out.extend_from_slice(&ident);
    write_c_string(&mut out, "splitnum");
    out.push(128);
    out.push(0);
    write_pair(&mut out, "hostname", &info.server_name);
    write_pair(&mut out, "gametype", GAME_TYPE);
    write_pair(&mut out, "game_id", GAME_ID);
    write_pair(&mut out, "version", &info.server_version);
    write_pair(&mut out, "plugins", &info.plugin_names);
    write_pair(&mut out, "map", &info.world_name);
    write_pair(&mut out, "numplayers", &info.player_count.to_string());
    write_pair(&mut out, "maxplayers", &info.max_players.to_string());
    write_pair(&mut out, "hostport", &info.server_port.to_string());
    write_pair(&mut out, "hostip", &info.host_ip);
    out.push(0);
    out.push(1);
    write_c_string(&mut out, "player_");
    out.push(0);
    for player in &info.player_names {
        write_c_string(&mut out, player);
    }
    out.push(0);
    out
}

fn write_pair(out: &mut Vec<u8>, key: &str, value: &str) {
    write_c_string(out, key);
    write_c_string(out, value);
}

fn write_c_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

fn read_i32_be(bytes: &[u8]) -> i32 {
    i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn random_challenge_seed() -> i32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    (nanos as i32).rem_euclid(0x01_00_00_00)
}

#[cfg(test)]
mod tests {
    use super::{QueryProtocol, QueryResponse, QueryServerInfo};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, Instant};

    fn info() -> QueryServerInfo {
        QueryServerInfo {
            server_name: "A Minecraft Server".to_string(),
            world_name: "world".to_string(),
            server_version: "26.1.2".to_string(),
            plugin_names: String::new(),
            host_ip: "127.0.0.1".to_string(),
            server_port: 25565,
            player_count: 2,
            max_players: 20,
            player_names: vec!["Alex".to_string(), "Steve".to_string()],
        }
    }

    fn remote(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    }

    #[test]
    fn query_challenge_response_matches_gs4_shape() {
        let mut protocol = QueryProtocol::with_challenge_seed(12345);
        let packet = [0xFE, 0xFD, 9, b'a', b'b', b'c', b'd'];

        let response = protocol
            .handle_packet(remote(25500), &packet, Instant::now(), &info())
            .unwrap();

        assert_eq!(
            response,
            QueryResponse::Challenge(b"\tabcd12345\0".to_vec())
        );
        assert_eq!(protocol.challenge_count(), 1);
    }

    #[test]
    fn query_basic_status_requires_matching_remote_challenge() {
        let now = Instant::now();
        let mut protocol = QueryProtocol::with_challenge_seed(7);
        let ident = [1, 2, 3, 4];
        protocol.handle_packet(
            remote(25500),
            &[0xFE, 0xFD, 9, ident[0], ident[1], ident[2], ident[3]],
            now,
            &info(),
        );

        let mut status = vec![0xFE, 0xFD, 0, ident[0], ident[1], ident[2], ident[3]];
        status.extend_from_slice(&7_i32.to_be_bytes());
        let response = protocol
            .handle_packet(remote(25500), &status, now, &info())
            .unwrap();
        let QueryResponse::BasicStatus(bytes) = response else {
            panic!("expected basic status response");
        };

        assert_eq!(&bytes[..5], &[0, 1, 2, 3, 4]);
        assert!(bytes
            .windows("A Minecraft Server\0".len())
            .any(|window| window == b"A Minecraft Server\0"));
        assert!(bytes
            .windows("SMP\0".len())
            .any(|window| window == b"SMP\0"));
        assert!(protocol
            .handle_packet(remote(25501), &status, now, &info())
            .is_none());
    }

    #[test]
    fn query_full_stat_includes_plugins_players_and_reuses_cached_body() {
        let now = Instant::now();
        let mut protocol = QueryProtocol::with_challenge_seed(42);
        let ident = *b"test";
        protocol.handle_packet(
            remote(25500),
            &[0xFE, 0xFD, 9, ident[0], ident[1], ident[2], ident[3]],
            now,
            &info(),
        );

        let mut full = vec![0xFE, 0xFD, 0, ident[0], ident[1], ident[2], ident[3]];
        full.extend_from_slice(&42_i32.to_be_bytes());
        full.extend_from_slice(&[0, 0, 0, 0]);
        let response = protocol
            .handle_packet(remote(25500), &full, now, &info())
            .unwrap();
        let QueryResponse::FullStat(bytes) = response else {
            panic!("expected full stat response");
        };

        assert_eq!(&bytes[..5], &[0, b't', b'e', b's', b't']);
        for field in [
            b"splitnum\0".as_slice(),
            b"game_id\0MINECRAFT\0",
            b"version\026.1.2\0",
            b"player_\0\0Alex\0Steve\0\0",
        ] {
            assert!(bytes.windows(field.len()).any(|window| window == field));
        }

        let other_ident = *b"othr";
        protocol.handle_packet(
            remote(25501),
            &[
                0xFE,
                0xFD,
                9,
                other_ident[0],
                other_ident[1],
                other_ident[2],
                other_ident[3],
            ],
            now,
            &info(),
        );
        let mut other_full = vec![
            0xFE,
            0xFD,
            0,
            other_ident[0],
            other_ident[1],
            other_ident[2],
            other_ident[3],
        ];
        other_full.extend_from_slice(&43_i32.to_be_bytes());
        other_full.extend_from_slice(&[0, 0, 0, 0]);
        let QueryResponse::FullStat(cached) = protocol
            .handle_packet(
                remote(25501),
                &other_full,
                now + Duration::from_secs(1),
                &info(),
            )
            .unwrap()
        else {
            panic!("expected cached full stat response");
        };
        assert_eq!(&cached[..5], &[0, b'o', b't', b'h', b'r']);
        assert_eq!(&cached[5..], &bytes[5..]);
    }

    #[test]
    fn query_challenges_expire_after_vanilla_window() {
        let now = Instant::now();
        let mut protocol = QueryProtocol::with_challenge_seed(1);
        let ident = *b"gone";
        protocol.handle_packet(
            remote(25500),
            &[0xFE, 0xFD, 9, ident[0], ident[1], ident[2], ident[3]],
            now,
            &info(),
        );

        let mut status = vec![0xFE, 0xFD, 0, ident[0], ident[1], ident[2], ident[3]];
        status.extend_from_slice(&1_i32.to_be_bytes());
        assert!(protocol
            .handle_packet(
                remote(25500),
                &status,
                now + Duration::from_secs(30),
                &info()
            )
            .is_none());
        assert_eq!(protocol.challenge_count(), 0);
    }
}
