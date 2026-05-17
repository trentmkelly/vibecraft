#![allow(dead_code)]

use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes128;

#[derive(Debug, Clone)]
pub struct MinecraftCipher {
    cipher: Aes128,
    feedback: [u8; 16],
}

impl MinecraftCipher {
    pub fn new(shared_secret: [u8; 16]) -> Self {
        Self {
            cipher: Aes128::new(GenericArray::from_slice(&shared_secret)),
            feedback: shared_secret,
        }
    }

    pub fn apply_encrypt(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            let plain = *byte;
            let key = self.next_key_byte();
            let cipher_byte = plain ^ key;
            *byte = cipher_byte;
            self.push_feedback(cipher_byte);
        }
    }

    pub fn apply_decrypt(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            let cipher_byte = *byte;
            let key = self.next_key_byte();
            *byte = cipher_byte ^ key;
            self.push_feedback(cipher_byte);
        }
    }

    fn next_key_byte(&self) -> u8 {
        let mut block = GenericArray::clone_from_slice(&self.feedback);
        self.cipher.encrypt_block(&mut block);
        block[0]
    }

    fn push_feedback(&mut self, cipher_byte: u8) {
        self.feedback.copy_within(1.., 0);
        self.feedback[15] = cipher_byte;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_aes_cfb8_round_trips_login_stream_bytes() {
        let secret = *b"0123456789abcdef";
        let mut encrypt = MinecraftCipher::new(secret);
        let mut decrypt = MinecraftCipher::new(secret);
        let mut payload = b"login and configuration packets stay encrypted as one stream".to_vec();
        let original = payload.clone();

        encrypt.apply_encrypt(&mut payload);
        assert_ne!(payload, original);
        decrypt.apply_decrypt(&mut payload);

        assert_eq!(payload, original);
    }

    #[test]
    fn minecraft_aes_cfb8_preserves_state_across_packet_boundaries() {
        let secret = *b"fedcba9876543210";
        let original = b"first packet bytessecond packet bytes".to_vec();
        let mut one_shot = original.clone();
        let mut split = original.clone();

        MinecraftCipher::new(secret).apply_encrypt(&mut one_shot);

        let mut streaming = MinecraftCipher::new(secret);
        let (first, second) = split.split_at_mut(18);
        streaming.apply_encrypt(first);
        streaming.apply_encrypt(second);

        assert_eq!(split, one_shot);

        let mut decrypt = MinecraftCipher::new(secret);
        let (first, second) = split.split_at_mut(18);
        decrypt.apply_decrypt(first);
        decrypt.apply_decrypt(second);
        assert_eq!(split, original);
    }
}
