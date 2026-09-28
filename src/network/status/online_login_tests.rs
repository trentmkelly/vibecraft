use super::*;
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

fn read_plain_packet(stream: &mut TcpStream) -> Cursor<Vec<u8>> {
    let payload = read_packet_with_compression(stream, CompressionState::disabled()).unwrap();
    Cursor::new(payload)
}

/// Drives the client half: reads hello, sends key packet, returns cipher pair.
fn client_key_exchange(
    client: &mut TcpStream,
    corrupt_challenge: bool,
) -> (Vec<u8>, [u8; 16], MinecraftCipher, MinecraftCipher) {
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
    let secret = [9u8; 16];
    let challenge = if corrupt_challenge {
        vec![0, 0, 0, 0]
    } else {
        hello.challenge.clone()
    };
    let packet = ServerboundKeyPacket {
        key_bytes: public
            .encrypt(&mut rand::rngs::OsRng, Pkcs1v15Encrypt, &secret)
            .unwrap(),
        encrypted_challenge: public
            .encrypt(&mut rand::rngs::OsRng, Pkcs1v15Encrypt, &challenge)
            .unwrap(),
    };
    write_framed_packet(client, SERVERBOUND_KEY_PACKET_ID, |payload| {
        packet.write(payload)
    })
    .unwrap();
    (
        hello.public_key,
        secret,
        MinecraftCipher::new(secret),
        MinecraftCipher::new(secret),
    )
}

type RecordedRequests = Arc<Mutex<Vec<HasJoinedRequest>>>;
type ServerRun = (
    TcpStream,
    thread::JoinHandle<io::Result<OnlineLoginOutcome>>,
    RecordedRequests,
);

fn run_server(result: SessionServiceResult, prevent_proxy: bool) -> ServerRun {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut server_stream, _) = listener.accept().unwrap();
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
        negotiate_online_login(
            &mut server_stream,
            "Alex",
            "203.0.113.7",
            &properties,
            &mut limiter,
            service,
        )
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
    let OnlineLoginOutcome::Authenticated { result, mut stream } = handle.join().unwrap().unwrap()
    else {
        panic!("expected authenticated");
    };
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
        handle.join().unwrap().unwrap(),
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
        handle.join().unwrap().unwrap(),
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
    let err = handle.join().unwrap().err().unwrap();
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
