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

#[derive(Debug, Clone)]
pub struct CipherBase {
    cipher: MinecraftCipher,
    heap_in: Vec<u8>,
    heap_out: Vec<u8>,
}

impl CipherBase {
    pub fn new(cipher: MinecraftCipher) -> Self {
        Self {
            cipher,
            heap_in: Vec::new(),
            heap_out: Vec::new(),
        }
    }

    fn buf_to_bytes(&mut self, input: &[u8]) -> &[u8] {
        if self.heap_in.len() < input.len() {
            self.heap_in.resize(input.len(), 0);
        }
        self.heap_in[..input.len()].copy_from_slice(input);
        &self.heap_in[..input.len()]
    }

    pub fn decipher(&mut self, input: &[u8]) -> Vec<u8> {
        let input_len = input.len();
        self.buf_to_bytes(input);
        if self.heap_out.len() < input_len {
            self.heap_out.resize(input_len, 0);
        }
        self.heap_out[..input_len].copy_from_slice(&self.heap_in[..input_len]);
        self.cipher.apply_decrypt(&mut self.heap_out[..input_len]);
        self.heap_out[..input_len].to_vec()
    }

    pub fn encipher(&mut self, input: &[u8], output: &mut Vec<u8>) {
        let input_len = input.len();
        self.buf_to_bytes(input);
        if self.heap_out.len() < input_len {
            self.heap_out.resize(input_len, 0);
        }
        self.heap_out[..input_len].copy_from_slice(&self.heap_in[..input_len]);
        self.cipher.apply_encrypt(&mut self.heap_out[..input_len]);
        output.extend_from_slice(&self.heap_out[..input_len]);
    }
}

#[derive(Debug, Clone)]
pub struct CipherDecoder {
    cipher: CipherBase,
}

impl CipherDecoder {
    pub fn new(cipher: MinecraftCipher) -> Self {
        Self {
            cipher: CipherBase::new(cipher),
        }
    }

    pub fn decode(&mut self, message: &[u8], output: &mut Vec<Vec<u8>>) {
        output.push(self.cipher.decipher(message));
    }
}

#[derive(Debug, Clone)]
pub struct CipherEncoder {
    cipher: CipherBase,
}

impl CipherEncoder {
    pub fn new(cipher: MinecraftCipher) -> Self {
        Self {
            cipher: CipherBase::new(cipher),
        }
    }

    pub fn encode(&mut self, message: &[u8], output: &mut Vec<u8>) {
        self.cipher.encipher(message, output);
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

    #[test]
    fn cipher_base_decoder_and_encoder_match_java_update_wrappers() {
        const CIPHER_BASE_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/CipherBase.java");
        const CIPHER_DECODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/CipherDecoder.java"
        );
        const CIPHER_ENCODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/CipherEncoder.java"
        );

        for sentinel in [
            "private byte[] heapIn = new byte[0];",
            "private byte[] heapOut = new byte[0];",
            "in.readBytes(this.heapIn, 0, readableBytes);",
            "heapOut.writerIndex(this.cipher.update(heapIn, 0, readableBytes, heapOut.array(), heapOut.arrayOffset()));",
            "out.writeBytes(this.heapOut, 0, this.cipher.update(heapIn, 0, readableBytes, this.heapOut));",
        ] {
            assert!(
                CIPHER_BASE_JAVA.contains(sentinel),
                "missing CipherBase sentinel {sentinel}"
            );
        }
        assert!(CIPHER_DECODER_JAVA.contains("out.add(this.cipher.decipher(ctx, msg));"));
        assert!(CIPHER_ENCODER_JAVA.contains("this.cipher.encipher(msg, out);"));

        let secret = *b"0123456789abcdef";
        let mut encoder = CipherEncoder::new(MinecraftCipher::new(secret));
        let mut decoder = CipherDecoder::new(MinecraftCipher::new(secret));
        let mut encoded = Vec::new();
        let mut decoded = Vec::new();

        encoder.encode(b"first", &mut encoded);
        let first_len = encoded.len();
        encoder.encode(b"second", &mut encoded);

        decoder.decode(&encoded[..first_len], &mut decoded);
        decoder.decode(&encoded[first_len..], &mut decoded);

        assert_eq!(decoded, vec![b"first".to_vec(), b"second".to_vec()]);
    }
}
