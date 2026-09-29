use super::*;
use std::net::TcpListener;

fn loopback() -> (ClientStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let remote = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (accepted, _) = listener.accept().unwrap();
    (ClientStream::new(accepted), remote)
}

#[test]
fn plaintext_passes_through_until_the_key_is_set() {
    let (mut server, mut remote) = loopback();
    assert!(!server.is_encrypted());
    server.write_all(b"plain").unwrap();
    let mut wire = [0u8; 5];
    remote.read_exact(&mut wire).unwrap();
    assert_eq!(&wire, b"plain");
    remote.write_all(b"hello").unwrap();
    let mut inbound = [0u8; 5];
    server.read_exact(&mut inbound).unwrap();
    assert_eq!(&inbound, b"hello");
}

#[test]
fn encryption_key_decrypts_inbound_and_encrypts_outbound_across_clones() {
    let (mut server, mut remote) = loopback();
    let mut clone = server.try_clone().unwrap();
    let secret = [7u8; 16];
    server.set_encryption_key(secret);
    assert!(clone.is_encrypted());

    let mut remote_encrypt = MinecraftCipher::new(secret);
    let mut remote_decrypt = MinecraftCipher::new(secret);

    let mut inbound = b"hello server".to_vec();
    remote_encrypt.apply_encrypt(&mut inbound);
    remote.write_all(&inbound).unwrap();
    let mut plain = [0u8; 12];
    // Alternate handles: the keystream position is shared, not per clone.
    server.read_exact(&mut plain[..5]).unwrap();
    clone.read_exact(&mut plain[5..]).unwrap();
    assert_eq!(&plain, b"hello server");

    server.write_all(b"hello ").unwrap();
    clone.write_all(b"client").unwrap();
    let mut outbound = [0u8; 12];
    remote.read_exact(&mut outbound).unwrap();
    assert_ne!(&outbound, b"hello client");
    remote_decrypt.apply_decrypt(&mut outbound);
    assert_eq!(&outbound, b"hello client");
}

#[test]
fn shutdown_closes_the_remote_end() {
    let (server, mut remote) = loopback();
    server.set_encryption_key([1u8; 16]);
    server.shutdown(Shutdown::Both).unwrap();
    let mut byte = [0u8; 1];
    assert_eq!(remote.read(&mut byte).unwrap_or(0), 0);
}
