//! Transparent AES/CFB8 transport for an encrypted login connection.
//!
//! Java installs `CipherDecoder`/`CipherEncoder` in the Netty pipeline
//! (`Connection.setEncryptionKey`). The Rust play/configuration code drives a plain
//! [`TcpStream`], so [`enable_encryption`] keeps that contract: it returns a loopback
//! stream for the rest of the server to use while two pump threads move bytes
//! between it and the real client socket, decrypting inbound and encrypting outbound
//! data with the negotiated shared secret (`network::encryption`).

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use crate::network::encryption::MinecraftCipher;

/// Wraps `client` in AES/CFB8 encryption keyed (and IV'd) by `shared_secret`
/// (`Crypt.getCipher`: the key doubles as the IV) and returns the plaintext stream
/// the connection handlers should use from now on.
pub fn enable_encryption(client: &TcpStream, shared_secret: [u8; 16]) -> io::Result<TcpStream> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let server_side = TcpStream::connect(listener.local_addr()?)?;
    let (relay_side, _) = listener.accept()?;
    server_side.set_nodelay(true)?;
    relay_side.set_nodelay(true)?;

    let mut decrypt = MinecraftCipher::new(shared_secret);
    let mut encrypt = MinecraftCipher::new(shared_secret);

    let mut client_in = client.try_clone()?;
    let mut relay_out = relay_side.try_clone()?;
    thread::Builder::new()
        .name("Encrypted inbound relay".to_string())
        .spawn(move || {
            pump(&mut client_in, &mut relay_out, |bytes| {
                decrypt.apply_decrypt(bytes)
            });
        })?;

    let mut relay_in = relay_side;
    let mut client_out = client.try_clone()?;
    thread::Builder::new()
        .name("Encrypted outbound relay".to_string())
        .spawn(move || {
            pump(&mut relay_in, &mut client_out, |bytes| {
                encrypt.apply_encrypt(bytes)
            });
        })?;
    Ok(server_side)
}

/// Copies `from` to `to`, transforming each chunk in place, until either side
/// closes; then closes both so the peer thread and the remote end observe EOF.
fn pump(from: &mut TcpStream, to: &mut TcpStream, mut transform: impl FnMut(&mut [u8])) {
    let mut buffer = [0u8; 8192];
    loop {
        let read = match from.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => read,
        };
        transform(&mut buffer[..read]);
        if to.write_all(&buffer[..read]).is_err() {
            break;
        }
    }
    let _ = to.shutdown(Shutdown::Write);
    let _ = from.shutdown(Shutdown::Read);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_decrypts_inbound_and_encrypts_outbound() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut remote = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (accepted, _) = listener.accept().unwrap();
        let secret = [7u8; 16];
        let mut server = enable_encryption(&accepted, secret).unwrap();

        let mut remote_encrypt = MinecraftCipher::new(secret);
        let mut remote_decrypt = MinecraftCipher::new(secret);

        let mut inbound = b"hello server".to_vec();
        remote_encrypt.apply_encrypt(&mut inbound);
        remote.write_all(&inbound).unwrap();
        let mut plain = [0u8; 12];
        server.read_exact(&mut plain).unwrap();
        assert_eq!(&plain, b"hello server");

        server.write_all(b"hello client").unwrap();
        let mut outbound = [0u8; 12];
        remote.read_exact(&mut outbound).unwrap();
        assert_ne!(&outbound, b"hello client");
        remote_decrypt.apply_decrypt(&mut outbound);
        assert_eq!(&outbound, b"hello client");
    }

    #[test]
    fn closing_the_server_side_closes_the_client_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut remote = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (accepted, _) = listener.accept().unwrap();
        let server = enable_encryption(&accepted, [1u8; 16]).unwrap();
        drop(server);
        let mut byte = [0u8; 1];
        assert_eq!(remote.read(&mut byte).unwrap_or(0), 0);
    }
}
