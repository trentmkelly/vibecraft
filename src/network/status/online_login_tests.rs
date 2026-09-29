use super::*;
use std::net::TcpStream;
use crate::network::encryption::MinecraftCipher;
use crate::player_access::NameAndId;
use crate::player_online_auth::{HasJoinedRequest, SessionServiceResult};
use rsa::pkcs8::DecodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

struct FakeSession {
    result: SessionServiceResult,
    requests: Arc<Mutex<Vec<HasJoinedRequest>>>,
}

impl SessionService for FakeSession {
    fn has_joined_server(&mut self, request: HasJoinedRequest) -> SessionServiceResult {
        self.requests.lock().unwrap().push(request);
        self.result.clone()
    }
}

fn read_plain_packet(stream: &mut impl Read) -> Cursor<Vec<u8>> {
    let payload = read_packet_with_compression(stream, CompressionState::disabled()).unwrap();
    Cursor::new(payload)
}

/// Drives the client half: reads hello, sends key packet, returns cipher pair.
fn client_key_exchange(
    client: &mut TcpStream,
    corrupt_challenge: bool,
) -> (Vec<u8>, [u8; 16], MinecraftCipher, MinecraftCipher) {
    let secret = [9u8; 16];
    let public_key = send_key_packet(client, corrupt_challenge, &secret);
    (
        public_key,
        secret,
        MinecraftCipher::new(secret),
        MinecraftCipher::new(secret),
    )
}

/// Reads the hello and answers with a key packet carrying `secret` (any length).
fn send_key_packet(client: &mut TcpStream, corrupt_challenge: bool, secret: &[u8]) -> Vec<u8> {
    let mut input = read_plain_packet(client);
    assert_eq!(
        read_var_i32(&mut input).unwrap(),
        CLIENTBOUND_HELLO_PACKET_ID
    );
    let hello = ClientboundHelloPacket::read(&mut input).unwrap();
    assert_eq!(hello.server_id, "");
    assert!(hello.should_authenticate);
    assert_eq!(hello.challenge.len(), 4);
    let public = RsaPublicKey::from_public_key_der(&hello.public_key).unwrap();
    let challenge = if corrupt_challenge {
        vec![0, 0, 0, 0]
    } else {
        hello.challenge.clone()
    };
    let packet = ServerboundKeyPacket {
        key_bytes: public
            .encrypt(&mut rand::rngs::OsRng, Pkcs1v15Encrypt, secret)
            .unwrap(),
        encrypted_challenge: public
            .encrypt(&mut rand::rngs::OsRng, Pkcs1v15Encrypt, &challenge)
            .unwrap(),
    };
    write_framed_packet(client, SERVERBOUND_KEY_PACKET_ID, |payload| {
        packet.write(payload)
    })
    .unwrap();
    hello.public_key
}

type RecordedRequests = Arc<Mutex<Vec<HasJoinedRequest>>>;
type ServerRun = (
    TcpStream,
    thread::JoinHandle<(io::Result<OnlineLoginOutcome>, ClientStream)>,
    RecordedRequests,
);

fn run_server(result: SessionServiceResult, prevent_proxy: bool) -> ServerRun {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (accepted, _) = listener.accept().unwrap();
    let mut server_stream = ClientStream::new(accepted);
    server_stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let service = FakeSession {
        result,
        requests: Arc::clone(&requests),
    };
    let handle = thread::spawn(move || {
        let mut properties = ServerProperties::load_or_default(Path::new(
            "definitely-missing-test-server.properties",
        ))
        .unwrap();
        properties.prevent_proxy_connections = prevent_proxy;
        let mut limiter = PacketRateLimiter::new(0, Instant::now());
        let outcome = negotiate_online_login(
            &mut server_stream,
            "Alex",
            "203.0.113.7",
            &properties,
            &mut limiter,
            service,
        );
        (outcome, server_stream)
    });
    (client, handle, requests)
}

#[test]
fn successful_login_encrypts_stream_and_sends_vanilla_server_hash() {
    let profile = NameAndId {
        uuid: "8667ba71-b85a-4004-af54-457a9734eed7".to_string(),
        name: "Alex".to_string(),
    };
    let (mut client, handle, requests) = run_server(
        SessionServiceResult::Joined(ProfileResult {
            profile: profile.clone(),
            properties: Vec::new(),
        }),
        true,
    );
    let (public_key, secret, mut encrypt, _) = client_key_exchange(&mut client, false);
    let (outcome, mut stream) = handle.join().unwrap();
    let OnlineLoginOutcome::Authenticated(result) = outcome.unwrap() else {
        panic!("expected authenticated");
    };
    assert!(stream.is_encrypted());
    assert_eq!(result.profile, profile);

    let requests = requests.lock().unwrap();
    assert_eq!(requests[0].username, "Alex");
    assert_eq!(
        requests[0].server_hash,
        minecraft_server_hash("", &public_key, &secret)
    );
    assert_eq!(requests[0].address.as_deref(), Some("203.0.113.7"));

    // Everything after the key packet is AES/CFB8 encrypted on the wire.
    stream.write_all(b"ping").unwrap();
    let mut wire = [0u8; 4];
    client.read_exact(&mut wire).unwrap();
    assert_ne!(&wire, b"ping");
    let mut decrypt = MinecraftCipher::new(secret);
    decrypt.apply_decrypt(&mut wire);
    assert_eq!(&wire, b"ping");
    let mut inbound = *b"pong";
    encrypt.apply_encrypt(&mut inbound);
    client.write_all(&inbound).unwrap();
    let mut plain = [0u8; 4];
    stream.read_exact(&mut plain).unwrap();
    assert_eq!(&plain, b"pong");
}

#[test]
fn unverified_session_sends_encrypted_unverified_username_disconnect() {
    let (mut client, handle, requests) = run_server(SessionServiceResult::NotJoined, false);
    let (_, _, _, mut decrypt) = client_key_exchange(&mut client, false);
    assert!(matches!(
        handle.join().unwrap().0.unwrap(),
        OnlineLoginOutcome::Closed
    ));
    assert_eq!(requests.lock().unwrap()[0].address, None);

    let mut wire = Vec::new();
    client.read_to_end(&mut wire).unwrap();
    decrypt.apply_decrypt(&mut wire);
    let mut input = Cursor::new(read_packet_from_bytes(&wire));
    assert_eq!(
        read_var_i32(&mut input).unwrap(),
        CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID
    );
    let text = String::from_utf8_lossy(input.get_ref()).to_string();
    assert!(text.contains("multiplayer.disconnect.unverified_username"));
}

#[test]
fn unavailable_session_service_disconnects_with_authservers_down() {
    let (mut client, handle, _) =
        run_server(SessionServiceResult::AuthenticationUnavailable, false);
    let (_, _, _, mut decrypt) = client_key_exchange(&mut client, false);
    assert!(matches!(
        handle.join().unwrap().0.unwrap(),
        OnlineLoginOutcome::Closed
    ));
    let mut wire = Vec::new();
    client.read_to_end(&mut wire).unwrap();
    decrypt.apply_decrypt(&mut wire);
    assert!(String::from_utf8_lossy(&wire).contains("multiplayer.disconnect.authservers_down"));
}

#[test]
fn wrong_challenge_is_a_protocol_error_and_never_reaches_the_session_service() {
    let (mut client, handle, requests) = run_server(SessionServiceResult::NotJoined, false);
    client_key_exchange(&mut client, true);
    let err = handle.join().unwrap().0.err().unwrap();
    assert_eq!(err.to_string(), "Protocol error");
    assert!(requests.lock().unwrap().is_empty());
}

/// Strips the length prefix of a single framed packet.
fn read_packet_from_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut input = Cursor::new(bytes);
    let length = read_var_i32(&mut input).unwrap() as usize;
    let start = input.position() as usize;
    bytes[start..start + length].to_vec()
}

#[test]
fn authenticated_profile_properties_survive_negotiation() {
    let profile = NameAndId {
        uuid: "8667ba71-b85a-4004-af54-457a9734eed7".to_string(),
        name: "Alex".to_string(),
    };
    let textures = crate::player_online_auth::ProfileProperty {
        name: "textures".to_string(),
        value: "base64-skin".to_string(),
        signature: Some("sig".to_string()),
    };
    let (mut client, handle, _) = run_server(
        SessionServiceResult::Joined(ProfileResult {
            profile,
            properties: vec![textures.clone()],
        }),
        false,
    );
    client_key_exchange(&mut client, false);
    let OnlineLoginOutcome::Authenticated(result) = handle.join().unwrap().0.unwrap() else {
        panic!("expected authenticated");
    };
    assert_eq!(result.properties, vec![textures]);
}

/// Java `Crypt.getCipher` uses the key as the CFB8 IV, which the JCE requires to be 16
/// bytes, so 24/32-byte AES keys fail with a `CryptException` ("Protocol error").
#[test]
fn non_16_byte_shared_secrets_are_protocol_errors_like_java() {
    for length in [8usize, 24, 32] {
        let (mut client, handle, requests) = run_server(SessionServiceResult::NotJoined, false);
        send_key_packet(&mut client, false, &vec![5u8; length]);
        let err = handle.join().unwrap().0.err().unwrap();
        assert_eq!(err.to_string(), "Protocol error");
        assert!(requests.lock().unwrap().is_empty());
    }
}

/// `ADD_PLAYER` carries the authenticated `GameProfile.properties`
/// (`ClientboundPlayerInfoUpdatePacket.Action.ADD_PLAYER`), not an empty list.
#[test]
fn player_info_add_player_entry_carries_profile_properties() {
    use crate::network::play::{ClientboundPlayerInfoUpdatePacket, PlayerInfoUpdateAction};
    let profile = NameAndId {
        uuid: "8667ba71-b85a-4004-af54-457a9734eed7".to_string(),
        name: "Alex".to_string(),
    };
    let textures = crate::player_online_auth::ProfileProperty {
        name: "textures".to_string(),
        value: "base64-skin".to_string(),
        signature: Some("sig".to_string()),
    };
    let mut bytes = Vec::new();
    write_player_info_initializing_packet(
        &mut bytes,
        &profile,
        std::slice::from_ref(&textures),
        GameMode::Survival,
    )
    .unwrap();
    let packet = ClientboundPlayerInfoUpdatePacket::read(&mut Cursor::new(bytes)).unwrap();
    assert!(packet.actions.contains(&PlayerInfoUpdateAction::AddPlayer));
    let added = packet.entries[0].profile.as_ref().unwrap();
    assert_eq!(added.name, "Alex");
    assert_eq!(added.properties.len(), 1);
    assert_eq!(added.properties[0].name, textures.name);
    assert_eq!(added.properties[0].value, textures.value);
    assert_eq!(added.properties[0].signature, textures.signature);
}
