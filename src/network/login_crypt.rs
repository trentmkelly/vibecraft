//! RSA key handling for the online-mode login handshake.
//!
//! Mirrors `net.minecraft.util.Crypt` (`generateKeyPair`, `decryptUsingKey`,
//! `decryptByteToSecretKey`) and the key pair `MinecraftServer.getKeyPair()`
//! keeps for the lifetime of the server.

use std::io;
use std::sync::OnceLock;

use rand::rngs::OsRng;
use rsa::pkcs8::EncodePublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

/// `Crypt.ASYMMETRIC_BITS`.
const ASYMMETRIC_BITS: usize = 1024;

/// The server's RSA key pair (`MinecraftServer.keyPair`).
pub struct ServerKeyPair {
    private: RsaPrivateKey,
    /// `PublicKey.getEncoded()`: the X.509 SubjectPublicKeyInfo DER bytes sent in
    /// `ClientboundHelloPacket` and hashed into the session server id.
    public_der: Vec<u8>,
}

impl ServerKeyPair {
    /// `Crypt.generateKeyPair()`: a fresh 1024-bit RSA key.
    pub fn generate() -> io::Result<Self> {
        let private = RsaPrivateKey::new(&mut OsRng, ASYMMETRIC_BITS).map_err(crypt_error)?;
        Self::from_private(private)
    }

    fn from_private(private: RsaPrivateKey) -> io::Result<Self> {
        let public = RsaPublicKey::from(&private);
        let public_der = public
            .to_public_key_der()
            .map_err(crypt_error)?
            .as_bytes()
            .to_vec();
        Ok(Self {
            private,
            public_der,
        })
    }

    /// The process-wide key pair, generated on first use like the Java server's
    /// startup-time `Crypt.generateKeyPair()` (`DedicatedServer.initServer`).
    pub fn shared() -> io::Result<&'static Self> {
        static KEY_PAIR: OnceLock<ServerKeyPair> = OnceLock::new();
        if let Some(pair) = KEY_PAIR.get() {
            return Ok(pair);
        }
        let pair = Self::generate()?;
        Ok(KEY_PAIR.get_or_init(|| pair))
    }

    /// `getPublic().getEncoded()`.
    pub fn public_key_der(&self) -> &[u8] {
        &self.public_der
    }

    /// `Crypt.decryptUsingKey(privateKey, input)`: RSA/ECB/PKCS1Padding decrypt.
    pub fn decrypt(&self, input: &[u8]) -> io::Result<Vec<u8>> {
        self.private
            .decrypt(Pkcs1v15Encrypt, input)
            .map_err(crypt_error)
    }

    /// Modulus size in bits (test hook for the `ASYMMETRIC_BITS` invariant).
    #[cfg(test)]
    fn modulus_bits(&self) -> usize {
        use rsa::traits::PublicKeyParts;
        self.private.size() * 8
    }
}

fn crypt_error(err: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("crypt error: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::traits::PublicKeyParts;

    #[test]
    fn generated_key_is_1024_bit_and_public_key_is_x509_der() {
        let pair = ServerKeyPair::generate().unwrap();
        assert_eq!(pair.modulus_bits(), 1024);
        let public = RsaPublicKey::from_public_key_der(pair.public_key_der()).unwrap();
        assert_eq!(public.size() * 8, 1024);
        // X.509 SubjectPublicKeyInfo begins with a DER SEQUENCE.
        assert_eq!(pair.public_key_der()[0], 0x30);
    }

    #[test]
    fn decrypt_round_trips_pkcs1_v15_client_encryption() {
        let pair = ServerKeyPair::generate().unwrap();
        let public = RsaPublicKey::from_public_key_der(pair.public_key_der()).unwrap();
        let encrypted = public
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &[1, 2, 3, 4])
            .unwrap();
        assert_eq!(pair.decrypt(&encrypted).unwrap(), vec![1, 2, 3, 4]);
        assert!(pair.decrypt(&[0u8; 128]).is_err());
    }
}
