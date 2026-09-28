//! Online-mode login negotiation: hello/key exchange, stream encryption and the
//! session-server check.
//!
//! Mirrors `ServerLoginPacketListenerImpl.handleHello` (online branch: state `KEY`,
//! `ClientboundHelloPacket("", publicKey, challenge, true)`) and `handleKey`
//! (challenge check, shared secret, `Connection.setEncryptionKey`, then the
//! "User Authenticator #n" thread calling `hasJoinedServer`).

use std::sync::atomic::AtomicUsize;

use super::*;
use crate::network::encrypted_relay::enable_encryption;
use crate::network::login::{
    ClientboundHelloPacket, ClientboundLoginDisconnectPacket, ServerboundKeyPacket,
    CLIENTBOUND_HELLO_PACKET_ID, SERVERBOUND_KEY_PACKET_ID,
};
use crate::network::login_crypt::ServerKeyPair;
use crate::player_online_auth::{
    minecraft_server_hash, AuthenticationDecision, OnlineAuthOptions, OnlineAuthenticator,
    ProfileResult, SessionService,
};

/// `ServerLoginPacketListenerImpl.MAX_TICKS_BEFORE_LOGIN` (600 ticks) as wall time.
const MAX_AUTHENTICATION_WAIT: Duration = Duration::from_secs(30);

/// Result of [`negotiate_online_login`].
pub(super) enum OnlineLoginOutcome {
    /// The session server verified the player. `stream` is the decrypted view of
    /// the connection that all further traffic must use.
    Authenticated {
        result: ProfileResult,
        stream: TcpStream,
    },
    /// A disconnect was already sent; the connection must be closed.
    Closed,
}

/// Runs the online-mode key exchange and authentication for `username`.
///
/// `remote_ip` is only forwarded to the session server when `prevent-proxy-connections`
/// is set (`getAddress()` in the Java authenticator thread).
pub(super) fn negotiate_online_login<S>(
    stream: &mut TcpStream,
    username: &str,
    remote_ip: &str,
    properties: &ServerProperties,
    rate_limiter: &mut PacketRateLimiter,
    session_service: S,
) -> io::Result<OnlineLoginOutcome>
where
    S: SessionService + Send + 'static,
{
    let key_pair = ServerKeyPair::shared()?;
    let challenge: [u8; 4] = rand_challenge();
    write_framed_packet(stream, CLIENTBOUND_HELLO_PACKET_ID, |payload| {
        ClientboundHelloPacket {
            server_id: String::new(),
            public_key: key_pair.public_key_der().to_vec(),
            challenge: challenge.to_vec(),
            should_authenticate: true,
        }
        .write(payload)
    })?;

    let key_packet = read_key_packet(stream, rate_limiter)?;
    // `packet.isChallengeValid`: any decrypt failure or mismatch is a "Protocol error".
    if key_pair
        .decrypt(&key_packet.encrypted_challenge)
        .ok()
        .as_deref()
        != Some(&challenge[..])
    {
        return Err(protocol_error());
    }
    // `Crypt.decryptByteToSecretKey` + `Crypt.getCipher`: the key is the AES key and IV.
    let shared_secret: [u8; 16] = key_pair
        .decrypt(&key_packet.key_bytes)
        .map_err(|_| protocol_error())?
        .try_into()
        .map_err(|_| protocol_error())?;
    let server_hash = minecraft_server_hash("", key_pair.public_key_der(), &shared_secret);

    let mut encrypted = enable_encryption(stream, shared_secret)?;
    encrypted.set_read_timeout(stream.read_timeout()?)?;
    encrypted.set_write_timeout(stream.write_timeout()?)?;

    let authenticator = OnlineAuthenticator::new(OnlineAuthOptions {
        online_mode: true,
        singleplayer: false,
        prevent_proxy_connections: properties.prevent_proxy_connections,
    });
    let decision = authenticate_on_thread(
        session_service,
        authenticator,
        username.to_string(),
        server_hash,
        remote_ip.to_string(),
    )?;
    match decision {
        AuthenticationDecision::Authenticated(result) => Ok(OnlineLoginOutcome::Authenticated {
            result,
            stream: encrypted,
        }),
        AuthenticationDecision::StartOfflineMode(profile) => {
            Ok(OnlineLoginOutcome::Authenticated {
                result: ProfileResult {
                    profile,
                    properties: Vec::new(),
                },
                stream: encrypted,
            })
        }
        AuthenticationDecision::Disconnect(translation_key) => {
            write_login_translation_disconnect(&mut encrypted, translation_key)?;
            Ok(OnlineLoginOutcome::Closed)
        }
    }
}

/// Java `new Thread("User Authenticator #" + id)`: the blocking session-server call
/// runs off the connection thread; the login is abandoned with `slow_login` if it
/// does not finish in time.
fn authenticate_on_thread<S>(
    mut service: S,
    authenticator: OnlineAuthenticator,
    username: String,
    server_hash: String,
    remote_ip: String,
) -> io::Result<AuthenticationDecision>
where
    S: SessionService + Send + 'static,
{
    static UNIQUE_THREAD_ID: AtomicUsize = AtomicUsize::new(0);
    let id = UNIQUE_THREAD_ID.fetch_add(1, Ordering::SeqCst) + 1;
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name(format!("User Authenticator #{id}"))
        .spawn(move || {
            let decision =
                authenticator.authenticate(&mut service, &username, server_hash, Some(&remote_ip));
            let _ = sender.send(decision);
        })?;
    receiver
        .recv_timeout(MAX_AUTHENTICATION_WAIT)
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "multiplayer.disconnect.slow_login"))
}

fn read_key_packet(
    stream: &mut TcpStream,
    rate_limiter: &mut PacketRateLimiter,
) -> io::Result<ServerboundKeyPacket> {
    let packet = read_packet_with_rate_limit(stream, CompressionState::disabled(), rate_limiter)?;
    let mut input = Cursor::new(packet);
    if read_var_i32(&mut input)? != SERVERBOUND_KEY_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unexpected key packet",
        ));
    }
    ServerboundKeyPacket::read(&mut input)
}

fn write_login_translation_disconnect(stream: &mut TcpStream, key: &str) -> io::Result<()> {
    write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
        ClientboundLoginDisconnectPacket {
            reason: ComponentJson(format!("{{\"translate\":\"{key}\"}}")),
        }
        .write(payload)
    })
}

fn protocol_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Protocol error")
}

/// `Ints.toByteArray(RandomSource.create().nextInt())`.
fn rand_challenge() -> [u8; 4] {
    rand::random()
}

#[cfg(test)]
#[path = "online_login_tests.rs"]
mod tests;
