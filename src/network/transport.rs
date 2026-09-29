//! Client connection transport with an optional AES/CFB8 cipher layer.
//!
//! Java's `Connection.setEncryptionKey(decryptCipher, encryptCipher)` inserts a
//! `CipherDecoder` ("decrypt") and `CipherEncoder` ("encrypt") at the head of the Netty
//! pipeline, so every later byte in either direction is transparently (de)crypted
//! while the packet handlers stay oblivious. [`ClientStream`] is that pipeline head for
//! the blocking-socket handlers: a `Read + Write` wrapper over [`TcpStream`] that
//! decrypts inbound and encrypts outbound bytes once [`ClientStream::set_encryption_key`]
//! has been called. Clones (`try_clone`) share the cipher state, exactly as all users of
//! one Netty channel share its handlers, so keystream position stays consistent.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::network::encryption::MinecraftCipher;

/// One direction's optional cipher, shared between every clone of a connection.
type SharedCipher = Arc<Mutex<Option<MinecraftCipher>>>;

/// A client socket plus the (initially absent) `decrypt`/`encrypt` pipeline handlers.
#[derive(Debug)]
pub struct ClientStream {
    tcp: TcpStream,
    /// `CipherDecoder` state; applied to inbound bytes.
    decrypt: SharedCipher,
    /// `CipherEncoder` state; applied to outbound bytes. Also serialises writers so
    /// keystream order equals wire order.
    encrypt: SharedCipher,
}

impl ClientStream {
    /// Wraps a freshly accepted socket with no encryption installed.
    pub fn new(tcp: TcpStream) -> Self {
        Self {
            tcp,
            decrypt: Arc::default(),
            encrypt: Arc::default(),
        }
    }

    /// `Connection.setEncryptionKey`: from now on inbound bytes are decrypted and
    /// outbound bytes encrypted with AES/CFB8 keyed (and IV'd) by `shared_secret`
    /// (`Crypt.getCipher`). Applies to every clone of this stream.
    pub fn set_encryption_key(&self, shared_secret: [u8; 16]) {
        *lock(&self.decrypt) = Some(MinecraftCipher::new(shared_secret));
        *lock(&self.encrypt) = Some(MinecraftCipher::new(shared_secret));
    }

    /// Whether [`Self::set_encryption_key`] has been called
    /// (`Connection.isEncrypted`).
    #[cfg(test)]
    pub fn is_encrypted(&self) -> bool {
        lock(&self.encrypt).is_some()
    }

    /// A second handle to the same connection sharing cipher state.
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            tcp: self.tcp.try_clone()?,
            decrypt: Arc::clone(&self.decrypt),
            encrypt: Arc::clone(&self.encrypt),
        })
    }

    /// Peeks raw socket bytes. Only meaningful before encryption is enabled (the
    /// legacy-ping sniff); after that the bytes are ciphertext.
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.tcp.peek(buf)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.tcp.set_read_timeout(timeout)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.tcp.set_write_timeout(timeout)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.tcp.set_nonblocking(nonblocking)
    }

    pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.tcp.peer_addr()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.tcp.shutdown(how)
    }
}

impl From<TcpStream> for ClientStream {
    fn from(tcp: TcpStream) -> Self {
        Self::new(tcp)
    }
}

/// Locks a cipher slot; a poisoned lock only means another handler panicked, the
/// cipher state itself is still consistent.
fn lock(cipher: &SharedCipher) -> MutexGuard<'_, Option<MinecraftCipher>> {
    cipher
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Read for ClientStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Hold the decrypt lock across the socket read so concurrent readers cannot
        // consume keystream out of wire order.
        let mut cipher = lock(&self.decrypt);
        let read = self.tcp.read(buf)?;
        if let Some(cipher) = cipher.as_mut() {
            cipher.apply_decrypt(&mut buf[..read]);
        }
        Ok(read)
    }
}

impl Write for ClientStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut cipher = lock(&self.encrypt);
        match cipher.as_mut() {
            None => self.tcp.write(buf),
            Some(cipher) => {
                // The keystream advances per byte, so the whole buffer must reach the
                // socket once encrypted; report it fully consumed (Netty's
                // `CipherEncoder` likewise transforms whole messages).
                let mut encrypted = buf.to_vec();
                cipher.apply_encrypt(&mut encrypted);
                self.tcp.write_all(&encrypted)?;
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tcp.flush()
    }
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
